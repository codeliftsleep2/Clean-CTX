use crate::cbm::{CbmConfig, CbmStatus, GraphBridge};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

#[cfg(test)]
use std::sync::Barrier;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    Dormant,
    Initializing,
    Ready,
    Degraded,
    Unavailable,
    ShuttingDown,
}

struct Control {
    phase: Phase,
    status: CbmStatus,
    worker: Option<JoinHandle<()>>,
}

struct Inner {
    config: CbmConfig,
    project_root: PathBuf,
    additional_roots: Vec<PathBuf>,
    bridge: Mutex<Option<GraphBridge>>,
    control: Mutex<Control>,
    shutdown: AtomicBool,
    #[cfg(test)]
    skip_provider_build: AtomicBool,
    #[cfg(test)]
    admission_attempts: AtomicUsize,
    #[cfg(test)]
    admission_gate: Mutex<Option<Arc<Barrier>>>,
}

/// The single session owner for CBM launch, admission, status, and shutdown.
#[derive(Clone)]
pub(crate) struct CbmLifecycle {
    inner: Arc<Inner>,
}

impl CbmLifecycle {
    pub(crate) fn new(
        config: CbmConfig,
        project_root: PathBuf,
        additional_roots: &[String],
    ) -> Self {
        let phase = if config.enabled {
            Phase::Dormant
        } else {
            Phase::Unavailable
        };
        Self {
            inner: Arc::new(Inner {
                config,
                project_root,
                additional_roots: additional_roots.iter().map(PathBuf::from).collect(),
                bridge: Mutex::new(None),
                control: Mutex::new(Control {
                    phase,
                    status: CbmStatus::Unavailable,
                    worker: None,
                }),
                shutdown: AtomicBool::new(false),
                #[cfg(test)]
                skip_provider_build: AtomicBool::new(false),
                #[cfg(test)]
                admission_attempts: AtomicUsize::new(0),
                #[cfg(test)]
                admission_gate: Mutex::new(None),
            }),
        }
    }

    /// Trigger configured post-initialize auto-launch exactly once.
    pub(crate) fn start_after_initialize(&self) {
        if self.inner.config.auto_launch {
            self.start_once();
        }
    }

    /// Trigger lazy launch for the first provider-dependent consumer.
    pub(crate) fn start_for_consumer(&self) {
        self.start_once();
    }

    fn start_once(&self) {
        let mut control = lock(&self.inner.control);
        if control.phase != Phase::Dormant || self.inner.shutdown.load(Ordering::Acquire) {
            return;
        }
        control.phase = Phase::Initializing;
        control.status = CbmStatus::Initializing;
        let inner = Arc::clone(&self.inner);
        match std::thread::Builder::new()
            .name("cbm-admission".into())
            .spawn(move || run_admission(inner))
        {
            Ok(worker) => control.worker = Some(worker),
            Err(error) => {
                control.phase = Phase::Degraded;
                control.status = CbmStatus::Degraded(format!(
                    "provider admission worker could not start: {error}"
                ));
            }
        }
    }

    pub(crate) fn status(&self) -> CbmStatus {
        if let Some(bridge) = lock(&self.inner.bridge).as_ref() {
            return bridge.status().clone();
        }
        self.unadmitted_status()
    }

    /// Read lifecycle status when the caller already proved the bridge absent.
    /// This intentionally does not acquire `bridge`, preventing recursive
    /// locking in fail-closed provider paths.
    pub(crate) fn unadmitted_status(&self) -> CbmStatus {
        lock(&self.inner.control).status.clone()
    }

    pub(crate) fn bridge_lock(&self) -> MutexGuard<'_, Option<GraphBridge>> {
        lock(&self.inner.bridge)
    }

    #[cfg(test)]
    pub(crate) fn skip_provider_build_for_test(&self) {
        self.inner
            .skip_provider_build
            .store(true, Ordering::Release);
    }

    #[cfg(test)]
    pub(crate) fn admission_attempts_for_test(&self) -> usize {
        self.inner.admission_attempts.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn block_admission_for_test(&self) -> Arc<Barrier> {
        let gate = Arc::new(Barrier::new(2));
        *lock(&self.inner.admission_gate) = Some(Arc::clone(&gate));
        gate
    }

    /// Join any in-flight admission before retiring the admitted frontend.
    pub(crate) fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        let worker = {
            let mut control = lock(&self.inner.control);
            control.phase = Phase::ShuttingDown;
            control.worker.take()
        };
        if let Some(worker) = worker {
            if worker.join().is_err() {
                eprintln!("[clean-ctx-cbm] Admission worker panicked during shutdown");
            }
        }
        lock(&self.inner.bridge).take();
        let mut control = lock(&self.inner.control);
        control.phase = Phase::Unavailable;
        control.status = CbmStatus::Unavailable;
    }
}

fn run_admission(inner: Arc<Inner>) {
    #[cfg(test)]
    {
        inner.admission_attempts.fetch_add(1, Ordering::SeqCst);
        let gate = lock(&inner.admission_gate).clone();
        if let Some(gate) = gate {
            gate.wait();
        }
    }
    #[cfg(test)]
    if inner.skip_provider_build.load(Ordering::Acquire) {
        finish_admission(&inner, None, CbmStatus::Unavailable);
        return;
    }
    let (bridge, status) =
        build_bridge(&inner.config, &inner.project_root, &inner.additional_roots);
    finish_admission(&inner, bridge, status);
}

fn finish_admission(inner: &Inner, bridge: Option<GraphBridge>, status: CbmStatus) {
    if inner.shutdown.load(Ordering::Acquire) {
        drop(bridge);
        return;
    }

    *lock(&inner.bridge) = bridge;
    let mut control = lock(&inner.control);
    control.phase = if status.is_available() {
        Phase::Ready
    } else if matches!(status, CbmStatus::Degraded(_)) {
        Phase::Degraded
    } else {
        Phase::Unavailable
    };
    control.status = status;
}

fn build_bridge(
    config: &CbmConfig,
    project_root: &Path,
    additional_roots: &[PathBuf],
) -> (Option<GraphBridge>, CbmStatus) {
    let mut bridge = GraphBridge::try_create_with_roots(config, project_root, additional_roots);
    let cache_db_path = config
        .cache_db_path
        .as_ref()
        .map(PathBuf::from)
        .or_else(|| match config.cache_scope {
            crate::cbm::config::CacheScope::Global => {
                Some(PathBuf::from(".clean-ctx/cbm-graph-cache.db"))
            }
            crate::cbm::config::CacheScope::PerWorkspace => {
                Some(project_root.join(".clean-ctx/cbm-graph-cache.db"))
            }
        });
    if let Some(db_path) = cache_db_path {
        match crate::cbm::cache_store::GraphCacheStore::open(&db_path) {
            Ok(store) => {
                eprintln!("[clean-ctx] CBM graph cache: {}", db_path.display());
                bridge.attach_disk_cache(store);
            }
            Err(error) => {
                eprintln!("[clean-ctx] WARNING: Failed to open CBM graph cache DB: {error}");
            }
        }
    }

    let status = bridge.status().clone();
    if bridge.is_available() {
        eprintln!("[clean-ctx] CBM graph intelligence: available");
        (Some(bridge), status)
    } else {
        if status.summary() != "unavailable" {
            eprintln!("[clean-ctx] CBM graph intelligence: {}", status.summary());
        }
        (None, status)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
#[path = "../tests/mcp/cbm_lifecycle.rs"]
mod tests;
