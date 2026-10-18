use crate::mcp::McpState;
use crate::mcp::semantic_compilation::artifact::SemanticCompilationArtifact;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::{Condvar, Mutex, MutexGuard};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CSharpCallAuthorityStatus {
    Current,
    Dirty,
    Reconciling,
}

#[derive(Debug)]
struct AuthorityInner {
    generation: u64,
    status: CSharpCallAuthorityStatus,
    fingerprint: Option<String>,
}

#[derive(Debug)]
pub(crate) struct CSharpCallAuthorityState {
    inner: Mutex<AuthorityInner>,
    changed: Condvar,
    artifacts: Mutex<BTreeMap<String, Arc<SemanticCompilationArtifact>>>,
    #[cfg(test)]
    reconciliation_count: std::sync::atomic::AtomicUsize,
}

impl Default for CSharpCallAuthorityState {
    fn default() -> Self {
        Self {
            inner: Mutex::new(AuthorityInner {
                generation: 0,
                status: CSharpCallAuthorityStatus::Dirty,
                fingerprint: None,
            }),
            changed: Condvar::new(),
            artifacts: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            reconciliation_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl CSharpCallAuthorityState {
    fn lock(&self) -> MutexGuard<'_, AuthorityInner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn mark_dirty(&self) {
        let mut inner = self.lock();
        inner.generation = inner.generation.wrapping_add(1);
        if inner.status != CSharpCallAuthorityStatus::Reconciling {
            inner.status = CSharpCallAuthorityStatus::Dirty;
        }
        self.changed.notify_all();
    }

    pub(crate) fn generation(&self) -> u64 {
        self.lock().generation
    }

    pub(crate) fn retain_artifact(&self, artifact: Arc<SemanticCompilationArtifact>) {
        self.artifacts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(artifact.canonical_owner().to_string(), artifact);
    }

    pub(crate) fn artifact(&self, owner: &str) -> Option<Arc<SemanticCompilationArtifact>> {
        self.artifacts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(owner)
            .cloned()
    }

    #[cfg(test)]
    pub(crate) fn clear_artifacts(&self) {
        self.artifacts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    #[cfg(test)]
    pub(crate) fn reset_reconciliation_count(&self) {
        self.reconciliation_count
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(crate) fn reconciliation_count(&self) -> usize {
        self.reconciliation_count
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn record_reconciliation(&self) {
        self.reconciliation_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    #[cfg(test)]
    pub(crate) fn status(&self) -> CSharpCallAuthorityStatus {
        self.lock().status
    }

    #[cfg(test)]
    pub(crate) fn fingerprint(&self) -> Option<String> {
        self.lock().fingerprint.clone()
    }

    pub(crate) fn prepare(&self, state: &McpState) -> Result<u64, String> {
        loop {
            let generation = {
                let mut inner = self.lock();
                match inner.status {
                    CSharpCallAuthorityStatus::Current
                        if inner.fingerprint.as_deref().is_some_and(|fingerprint| {
                            state
                                .workspace_index_read()
                                .has_current_csharp_exact_call_coverage(
                                    fingerprint,
                                    inner.generation,
                                )
                        }) =>
                    {
                        return Ok(inner.generation);
                    }
                    CSharpCallAuthorityStatus::Current => {
                        inner.status = CSharpCallAuthorityStatus::Reconciling;
                        inner.generation
                    }
                    CSharpCallAuthorityStatus::Dirty => {
                        inner.status = CSharpCallAuthorityStatus::Reconciling;
                        inner.generation
                    }
                    CSharpCallAuthorityStatus::Reconciling => {
                        drop(
                            self.changed
                                .wait(inner)
                                .unwrap_or_else(std::sync::PoisonError::into_inner),
                        );
                        continue;
                    }
                }
            };

            let result = state.project_csharp_exact_calls_for_generation(generation);
            let mut inner = self.lock();
            let generation_is_current = inner.generation == generation
                && inner.status == CSharpCallAuthorityStatus::Reconciling;
            if generation_is_current {
                inner.status = if let Ok(fingerprint) = &result {
                    inner.fingerprint = Some(fingerprint.clone());
                    CSharpCallAuthorityStatus::Current
                } else {
                    CSharpCallAuthorityStatus::Dirty
                };
            } else if inner.status == CSharpCallAuthorityStatus::Reconciling {
                inner.status = CSharpCallAuthorityStatus::Dirty;
            }
            self.changed.notify_all();
            drop(inner);
            if !generation_is_current {
                continue;
            }
            result?;
        }
    }
}

impl McpState {
    pub(crate) fn mark_csharp_authority_dirty(&self) {
        self.csharp_call_authority.mark_dirty();
        self.workspace_index_lock()
            .dirty_csharp_exact_call_coverage();
    }

    pub(crate) fn prepare_exact_csharp_calls(&self) -> Result<u64, String> {
        self.csharp_call_authority.prepare(self)
    }

    pub(crate) fn csharp_authority_generation(&self) -> u64 {
        self.csharp_call_authority.generation()
    }
}
