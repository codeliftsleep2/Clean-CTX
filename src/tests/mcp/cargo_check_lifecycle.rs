//! Actual binary + real Cargo tests, explicitly owner-run after building.
use super::cargo_check_lifecycle_harness::*;
use serde_json::{Value, json};

#[test]
#[ignore = "requires explicit built binary and absolute Cargo; owner-run lifecycle gate"]
fn cargo_check_lifecycle_cli_real_cargo_success_failure_and_text() {
    let root = fixture("pub fn answer() -> u8 { 42 }\n", Some(ENVIRONMENT_BUILD));
    let mut command = command(root.path());
    command
        .arg("cargo-check")
        .arg("--workspace-root")
        .arg(root.path())
        .arg("--cargo-path")
        .arg(configured("CLEAN_CTX_TEST_CARGO"))
        .arg("--json");
    let (status, stdout, stderr) = Process::spawn(command).finish();
    assert_eq!(status.code(), Some(0));
    no_secret(&stdout);
    no_secret(&stderr);
    let result: Value = serde_json::from_slice(&stdout).unwrap();
    clean_result(&result);
    assert_eq!(result["root_outcome"]["exit_code"], 0);
    assert_eq!(result["cli_outcome"], "success");
    assert_eq!(result["authority"]["workspace_source"], "cli_argument");
    assert_eq!(
        result["semantic"]["cargo_evidence"]["build_finished_success"],
        true
    );

    std::fs::write(
        root.path().join("src/lib.rs"),
        "pub fn broken() { let _: u8 = \"Authorization: Bearer abc123456789\"; }\n",
    )
    .unwrap();
    let mut command = super::cargo_check_lifecycle_harness::command(root.path());
    command
        .args(["cargo-check", "--workspace-root"])
        .arg(root.path())
        .arg("--json");
    let (status, stdout, stderr) = Process::spawn(command).finish();
    assert_eq!(status.code(), Some(1));
    no_secret(&stdout);
    no_secret(&stderr);
    let result: Value = serde_json::from_slice(&stdout).unwrap();
    clean_result(&result);
    assert_eq!(result["root_outcome"]["exit_code"], 101);
    assert_eq!(
        result["semantic"]["cargo_evidence"]["build_finished_success"],
        false
    );
    let diagnostics = result["semantic"]["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic["code"] == "E0308")
    );
    for diagnostic in diagnostics {
        for span in diagnostic["primary_spans"].as_array().unwrap() {
            assert!(!std::path::Path::new(span["file"].as_str().unwrap()).is_absolute());
        }
    }

    let mut command = super::cargo_check_lifecycle_harness::command(root.path());
    command
        .args(["cargo-check", "--workspace-root"])
        .arg(root.path());
    let (status, stdout, stderr) = Process::spawn(command).finish();
    assert_eq!(status.code(), Some(1));
    no_secret(&stdout);
    no_secret(&stderr);
    assert!(stdout.len() <= 24 * 1024);
    assert!(String::from_utf8_lossy(&stdout).lines().count() <= 240);
    assert!(String::from_utf8_lossy(&stdout).contains("E0308"));
}

#[test]
#[ignore = "requires explicit built binary and absolute Cargo; owner-run lifecycle gate"]
fn cargo_check_lifecycle_mcp_real_cargo_round_trip_and_session_reuse() {
    let root = fixture("pub fn answer() -> u8 { 42 }\n", Some(ENVIRONMENT_BUILD));
    let mut host = Mcp::spawn(root.path(), true);
    host.send(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}));
    assert!(host.receive(1)["result"]["capabilities"]["tools"].is_object());
    host.send(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}));
    let catalog = host.receive(2);
    assert!(
        catalog["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "cargo_check"
                && tool["inputSchema"]["additionalProperties"] == false)
    );
    host.check(3);
    let success = host.receive(3);
    assert_eq!(success["result"]["isError"], false);
    clean_result(&success["result"]["structuredContent"]);
    assert_eq!(
        success["result"]["structuredContent"]["authority"]["workspace_source"],
        "startup_option"
    );
    no_secret(&serde_json::to_vec(&success).unwrap());
    std::fs::write(
        root.path().join("src/lib.rs"),
        "pub fn broken() { let _: u8 = \"Authorization: Bearer abc123456789\"; }\n",
    )
    .unwrap();
    host.check(4);
    let failure = host.receive(4);
    assert_eq!(failure["result"]["isError"], true);
    assert_eq!(
        failure["result"]["structuredContent"]["root_outcome"]["exit_code"],
        101
    );
    clean_result(&failure["result"]["structuredContent"]);
    no_secret(&serde_json::to_vec(&failure).unwrap());
    std::fs::write(root.path().join("src/lib.rs"), "pub fn recovered() {}\n").unwrap();
    host.check(5);
    assert_eq!(host.receive(5)["result"]["isError"], false);
    no_secret(&host.finish());
}

#[test]
#[ignore = "requires explicit built binary and absolute Cargo; owner-run lifecycle gate"]
fn cargo_check_lifecycle_mcp_real_build_script_cancellation_and_recovery() {
    let root = fixture("pub fn answer() -> u8 { 42 }\n", Some(WAITING_BUILD));
    let mut host = Mcp::spawn(root.path(), false);
    host.check(10);
    wait_for_file(&root.path().join("build-script-started"));
    host.cancel(10);
    let cancelled = host.receive(10);
    assert_eq!(cancelled["result"]["isError"], true);
    let result = &cancelled["result"]["structuredContent"];
    clean_result(result);
    assert_eq!(result["cleanup"]["cancellation_source"], "host");
    assert_eq!(result["cleanup"]["forced_termination"], true);
    assert_eq!(result["authority"]["workspace_source"], "environment");
    no_secret(&serde_json::to_vec(&cancelled).unwrap());
    std::fs::write(root.path().join("build.rs"), "fn main() {}\n").unwrap();
    host.check(11);
    assert_eq!(host.receive(11)["result"]["isError"], false);
    no_secret(&host.finish());
}

#[test]
#[ignore = "requires explicit built binary and absolute Cargo; owner-run lifecycle gate"]
fn cargo_check_lifecycle_mcp_eof_cancels_running_real_build_script() {
    let root = fixture("pub fn answer() -> u8 { 42 }\n", Some(WAITING_BUILD));
    let mut host = Mcp::spawn(root.path(), true);
    host.check(20);
    wait_for_file(&root.path().join("build-script-started"));
    no_secret(&host.finish());
    let cancelled = host.receive(20);
    assert_eq!(
        cancelled["result"]["structuredContent"]["cleanup"]["cancellation_source"],
        "host"
    );
    clean_result(&cancelled["result"]["structuredContent"]);
}

#[test]
#[cfg(target_os = "linux")]
#[ignore = "requires explicit built binary and absolute Cargo; owner-run lifecycle gate"]
fn cargo_check_lifecycle_cli_sigint_cancels_real_build_script() {
    let root = fixture("pub fn answer() -> u8 { 42 }\n", Some(WAITING_BUILD));
    let mut command = command(root.path());
    command
        .args(["cargo-check", "--workspace-root"])
        .arg(root.path())
        .arg("--json");
    let mut process = Process::spawn(command);
    wait_for_file(&root.path().join("build-script-started"));
    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(process.child.id() as i32),
        nix::sys::signal::Signal::SIGINT,
    )
    .unwrap();
    let (status, stdout, stderr) = process.finish();
    assert_eq!(status.code(), Some(4));
    no_secret(&stdout);
    no_secret(&stderr);
    let result: Value = serde_json::from_slice(&stdout).unwrap();
    clean_result(&result);
    assert_eq!(result["cleanup"]["cancellation_source"], "user");
    assert_eq!(result["cleanup"]["forced_termination"], true);
}
