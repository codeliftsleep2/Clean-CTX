//! Local process fixtures and failure injection; no installed CBM is executed.
use super::{CbmClient, CbmError};
use std::cell::RefCell;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Clone, Default)]
struct Options {
    log: PathBuf,
    mode: String,
    fail_worker: Option<&'static str>,
    child_id: Option<u32>,
}
thread_local! { static OPTIONS: RefCell<Options> = RefCell::new(Options::default()); }

pub(super) fn configure_command(command: &mut Command) {
    OPTIONS.with(|options| {
        let options = options.borrow();
        if !options.log.as_os_str().is_empty() {
            command
                .env("CLEAN_CTX_A2_FIXTURE_LOG", &options.log)
                .env("CLEAN_CTX_A2_FIXTURE_MODE", &options.mode);
        }
    });
}
pub(super) fn record_child(id: u32) {
    OPTIONS.with(|options| options.borrow_mut().child_id = Some(id));
}
pub(super) fn check_worker(name: &str) -> io::Result<()> {
    if OPTIONS.with(|options| options.borrow().fail_worker == Some(name)) {
        Err(io::Error::other(format!("injected {name} setup failure")))
    } else {
        Ok(())
    }
}

pub(super) fn launch(
    root: &Path,
    log: &Path,
    mode: &str,
    fail: Option<&'static str>,
) -> (Result<Option<CbmClient>, CbmError>, u32) {
    OPTIONS.with(|options| {
        *options.borrow_mut() = Options {
            log: log.to_path_buf(),
            mode: mode.into(),
            fail_worker: fail,
            child_id: None,
        }
    });
    let result = CbmClient::try_launch(
        &std::env::current_exe().unwrap(),
        root,
        Duration::from_millis(300),
        3,
        0,
    );
    if mode == "flood" && result.as_ref().is_ok_and(|client| client.is_some()) {
        wait_file(&log.with_extension("queue-full"));
    }
    let id = OPTIONS.with(|options| options.borrow().child_id.unwrap_or(0));
    OPTIONS.with(|options| *options.borrow_mut() = Options::default());
    (result, id)
}

pub(super) fn owned_resources(client: &CbmClient) -> usize {
    client.frontend.as_ref().map_or(0, |frontend| {
        1 + usize::from(frontend.stdin.is_some())
            + usize::from(frontend.stdout_rx.is_some())
            + usize::from(frontend.stdout_reader.is_some())
            + usize::from(frontend.stderr_drainer.is_some())
    })
}
pub(super) fn retire(client: &mut CbmClient) {
    client.retire_for_a2_test();
}
pub(super) fn request_id(client: &CbmClient) -> u64 {
    client.request_id.load(std::sync::atomic::Ordering::SeqCst)
}
pub(super) fn workers_finished(client: &CbmClient) -> bool {
    client
        .last_retirement
        .is_some_and(|outcome| outcome.workers_joined)
}

pub(super) fn wait_file(path: &Path) -> String {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Ok(text) = std::fs::read_to_string(path) {
            if !text.is_empty() {
                return text;
            }
        }
        assert!(
            Instant::now() < deadline,
            "fixture did not produce {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

// Existing ctor dependency permits a portable fixture process before libtest
// writes its banners. The marker is set only on an individual test child.
#[ctor::ctor]
fn fixture_entry() {
    let Some(log) = std::env::var_os("CLEAN_CTX_A2_FIXTURE_LOG") else {
        return;
    };
    let log = PathBuf::from(log);
    let mode = std::env::var("CLEAN_CTX_A2_FIXTURE_MODE").unwrap();
    let cwd = std::env::current_dir().unwrap();
    std::fs::write(
        &log,
        serde_json::to_vec(&serde_json::json!({
            "cwd": cwd, "pid": std::process::id(),
            "cache": std::env::var_os("CBM_CACHE_DIR"),
        }))
        .unwrap(),
    )
    .unwrap();
    let mut launches = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log.with_extension("launches"))
        .unwrap();
    writeln!(launches, "{}", std::process::id()).unwrap();
    if mode == "exit" {
        std::process::exit(0);
    }
    if mode == "flood" {
        let mut out = io::stdout().lock();
        for _ in 0..300 {
            writeln!(
                out,
                "{{\"jsonrpc\":\"2.0\",\"method\":\"notifications/message\"}}"
            )
            .unwrap();
        }
        out.flush().unwrap();
    }
    let mut line = String::new();
    while io::stdin().read_line(&mut line).unwrap() != 0 {
        let request: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        let mut requests = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log.with_extension("requests"))
            .unwrap();
        writeln!(requests, "{}", line.trim()).unwrap();
        if mode == "stall" {
            std::thread::sleep(Duration::from_secs(30));
        }
        let id = if mode == "poison" {
            serde_json::json!(999)
        } else {
            request["id"].clone()
        };
        println!(
            "{}",
            serde_json::json!({"jsonrpc":"2.0", "id":id, "result":{"content":[{"type":"text", "text":"{\"columns\":[],\"rows\":[]}"}]}})
        );
        io::stdout().flush().unwrap();
        line.clear();
    }
    std::process::exit(0);
}

pub(super) fn bridge(root: &Path, extras: &[PathBuf], log: &Path) -> crate::cbm::GraphBridge {
    OPTIONS.with(|options| {
        *options.borrow_mut() = Options {
            log: log.to_path_buf(),
            mode: "normal".into(),
            ..Options::default()
        }
    });
    let config = crate::cbm::CbmConfig {
        binary_path: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        query_timeout_ms: 1000,
        ..Default::default()
    };
    let bridge = crate::cbm::GraphBridge::try_create_with_roots(&config, root, extras);
    OPTIONS.with(|options| *options.borrow_mut() = Options::default());
    bridge
}

pub(super) fn assert_process_gone(id: u32) {
    assert_ne!(id, 0, "fixture process was never started");
    assert!(!process_alive(id), "direct frontend {id} was not reaped");
}
#[cfg(unix)]
fn process_alive(id: u32) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    // Signal zero observes existence without terminating any process.
    unsafe { kill(id as i32, 0) == 0 }
}
#[cfg(windows)]
fn process_alive(id: u32) -> bool {
    use std::ffi::c_void;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(access: u32, inherit: i32, id: u32) -> *mut c_void;
        fn GetExitCodeProcess(process: *mut c_void, code: *mut u32) -> i32;
        fn CloseHandle(handle: *mut c_void) -> i32;
    }
    unsafe {
        let process = OpenProcess(0x1000, 0, id);
        if process.is_null() {
            return false;
        }
        let mut code = 0;
        let alive = GetExitCodeProcess(process, &mut code) != 0 && code == 259;
        CloseHandle(process);
        alive
    }
}

pub(super) fn queue_marker() -> Option<PathBuf> {
    OPTIONS.with(|options| {
        let options = options.borrow();
        (options.mode == "flood").then(|| options.log.with_extension("queue-full"))
    })
}
pub(super) fn observe_frame(marker: &Option<PathBuf>, observed: &mut usize) {
    *observed += 1;
    if *observed == 257 {
        if let Some(marker) = marker {
            std::fs::write(marker, "256 frames queued; next send blocked").unwrap();
        }
    }
}
