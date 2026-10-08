//! Owner-run black-box harness. Deadlines belong to tests, not CargoCheck policy.
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub(super) const SECRET: &str = "abc123456789";
const DEADLINE: Duration = Duration::from_secs(30);

pub(super) fn configured(name: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os(name).expect("set explicit owner-run test binary/Cargo paths"),
    );
    assert!(path.is_absolute() && path.is_file());
    path
}

pub(super) fn fixture(source: &str, build: Option<&str>) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir(root.path().join("src")).unwrap();
    std::fs::write(root.path().join("Cargo.toml"),
        "[package]\nname='cargo_check_lifecycle_fixture'\nversion='0.1.0'\nedition='2021'\n[workspace]\n").unwrap();
    std::fs::write(root.path().join("src/lib.rs"), source).unwrap();
    if let Some(build) = build {
        std::fs::write(root.path().join("build.rs"), build).unwrap();
    }
    std::fs::write(
        root.path().join(".clean-ctx.json"),
        json!({
            "cbm":{"enabled":false}, "persistence":{"enabled":false},
            "proxy":{"auto_start":false}, "cache":{"enabled":false},
            "observability":{"export_metrics":false},
        })
        .to_string(),
    )
    .unwrap();
    root
}

pub(super) fn command(root: &Path) -> Command {
    let mut command = Command::new(configured("CLEAN_CTX_TEST_BINARY"));
    command
        .current_dir(root)
        .env("CLEAN_CTX_PROJECT_ROOT", root)
        .env("CLEAN_CTX_CARGO_PATH", configured("CLEAN_CTX_TEST_CARGO"))
        .env("GITHUB_TOKEN", "PRIVATE_ENV_CREDENTIAL_CANARY")
        .env("HTTPS_PROXY", "PRIVATE_PROXY_CANARY")
        .env("RUSTFLAGS", "PRIVATE_RUSTFLAGS_CANARY")
        .env("RUSTC_WRAPPER", "PRIVATE_WRAPPER_CANARY")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("CARGO_ENCODED_RUSTFLAGS");
    command
}

fn stream<R: Read + Send + 'static>(mut stream: R) -> JoinHandle<Vec<u8>> {
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        bytes
    })
}

pub(super) struct Process {
    pub child: Child,
    pub input: Option<ChildStdin>,
    stdout: Option<JoinHandle<Vec<u8>>>,
    stderr: Option<JoinHandle<Vec<u8>>>,
}

impl Process {
    pub fn spawn(mut command: Command) -> Self {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Self {
            input: child.stdin.take(),
            stdout: Some(stream(child.stdout.take().unwrap())),
            stderr: Some(stream(child.stderr.take().unwrap())),
            child,
        }
    }

    pub fn finish(&mut self) -> (ExitStatus, Vec<u8>, Vec<u8>) {
        self.input.take();
        let deadline = Instant::now() + DEADLINE;
        let status = loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(
                Instant::now() < deadline,
                "owner-run child exceeded test deadline"
            );
            std::thread::sleep(Duration::from_millis(10));
        };
        let stdout = self
            .stdout
            .take()
            .map(|thread| thread.join().unwrap())
            .unwrap_or_default();
        let stderr = self.stderr.take().unwrap().join().unwrap();
        (status, stdout, stderr)
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.input.take();
        if self.child.try_wait().ok().flatten().is_none() {
            let deadline = Instant::now() + Duration::from_secs(5);
            while self.child.try_wait().ok().flatten().is_none() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(10));
            }
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = self.child.kill();
            }
        }
        let _ = self.child.wait();
    }
}

pub(super) struct Mcp {
    process: Process,
    responses: Receiver<Value>,
    reader: Option<JoinHandle<()>>,
}

impl Mcp {
    pub fn spawn(root: &Path, explicit: bool) -> Self {
        let mut command = command(root);
        if explicit {
            command
                .args(["--workspace-root"])
                .arg(root)
                .args(["--cargo-path"])
                .arg(configured("CLEAN_CTX_TEST_CARGO"));
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, responses) = channel();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let line = line.unwrap();
                let response =
                    serde_json::from_str(&line).expect("stdout must contain only JSON-RPC");
                if tx.send(response).is_err() {
                    break;
                }
            }
        });
        let process = Process {
            input: child.stdin.take(),
            stdout: None,
            stderr: Some(stream(child.stderr.take().unwrap())),
            child,
        };
        Self {
            process,
            responses,
            reader: Some(reader),
        }
    }

    pub fn send(&mut self, request: Value) {
        let input = self.process.input.as_mut().unwrap();
        writeln!(input, "{request}").unwrap();
        input.flush().unwrap();
    }

    pub fn receive(&self, id: i64) -> Value {
        let response = self
            .responses
            .recv_timeout(DEADLINE)
            .expect("MCP response within test deadline");
        assert_eq!(response["id"], id);
        response
    }

    pub fn check(&mut self, id: i64) {
        self.send(json!({"jsonrpc":"2.0","id":id,"method":"tools/call",
            "params":{"name":"cargo_check","arguments":{}}}));
    }

    pub fn cancel(&mut self, id: i64) {
        self.send(json!({"jsonrpc":"2.0","method":"notifications/cancelled",
            "params":{"requestId":id,"reason":"PRIVATE_CANCEL_CANARY"}}));
    }

    pub fn finish(&mut self) -> Vec<u8> {
        let (status, _, stderr) = self.process.finish();
        assert!(status.success());
        self.reader.take().unwrap().join().unwrap();
        stderr
    }
}

pub(super) fn wait_for_file(path: &Path) {
    let deadline = Instant::now() + DEADLINE;
    while !path.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        path.exists(),
        "real build script must start before cancellation"
    );
}

pub(super) fn clean_result(value: &Value) {
    assert_eq!(value["cleanup"]["descendant_quiescence_observed"], true);
    assert_eq!(value["cleanup"]["cleanup_uncertain"], false);
    assert_eq!(value["capture"]["spool_used"], false);
    assert_eq!(value["intrinsic_timeout"], false);
    assert_eq!(
        value["authority"]["command"],
        "cargo check --message-format=json"
    );
    let structured = serde_json::to_vec(value).unwrap();
    assert!(structured.len() <= 512 * 1024);
    assert!(!String::from_utf8_lossy(&structured).contains(SECRET));
    assert!(!String::from_utf8_lossy(&structured).contains("PRIVATE_ENV_CREDENTIAL_CANARY"));
}

pub(super) fn no_secret(bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    for marker in [
        SECRET,
        "PRIVATE_ENV_CREDENTIAL_CANARY",
        "PRIVATE_PROXY_CANARY",
        "PRIVATE_RUSTFLAGS_CANARY",
        "PRIVATE_WRAPPER_CANARY",
        "PRIVATE_CANCEL_CANARY",
    ] {
        assert!(!text.contains(marker), "producer/value marker leaked");
    }
}

pub(super) const WAITING_BUILD: &str = r#"
fn main() {
    std::fs::write("build-script-started", std::process::id().to_string()).unwrap();
    for _ in 0..300 { std::thread::sleep(std::time::Duration::from_millis(100)); }
}
"#;

pub(super) const ENVIRONMENT_BUILD: &str = r#"
fn main() {
    for name in ["GITHUB_TOKEN", "HTTPS_PROXY", "RUSTFLAGS"] {
        assert!(std::env::var_os(name).is_none(), "operator environment was forwarded");
    }
    assert_eq!(std::env::var("RUSTUP_AUTO_INSTALL").unwrap(), "0");
    assert_eq!(std::env::var("RUSTC_WRAPPER").unwrap_or_default(), "");
    println!("cargo:warning=Authorization: Bearer abc123456789");
}
"#;
