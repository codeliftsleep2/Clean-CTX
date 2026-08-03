// src/tests/mcp/e2e_server.rs
//
// Black-box E2E test: spawn the clean-ctx binary, send a JSON-RPC
// request over stdin, and validate the response.
//
// These tests require the binary to be built first (`cargo build`).
// They are marked `#[ignore]` by default to avoid requiring the
// binary in every test run.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

/// Path to the debug binary (the most common build target).
fn binary_path() -> String {
    let ext = if cfg!(windows) { ".exe" } else { "" };
    if cfg!(debug_assertions) {
        format!("target/debug/clean-ctx{ext}")
    } else {
        format!("target/release/clean-ctx{ext}")
    }
}

/// Spawn the clean-ctx binary, send a JSON-RPC request, and read the response.
fn spawn_and_send(request: &str, timeout_secs: u64) -> (String, std::process::Child) {
    let mut child = Command::new(binary_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn clean-ctx binary — run `cargo build` first");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    writeln!(stdin, "{}", request).expect("Failed to write to stdin");
    stdin.flush().ok();

    // Read response (accumulate lines until valid JSON)
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).unwrap_or(0) == 0 {
            break; // EOF
        }
        response.push_str(&line);
        if serde_json::from_str::<serde_json::Value>(&response).is_ok() {
            break;
        }
        if std::time::Instant::now() > deadline {
            break;
        }
    }

    (response, child)
}

/// Black-box E2E test: verify the server responds to a valid tools/list request.
#[ignore]
#[test]
fn test_e2e_tools_list() {
    // Spawn and send a valid JSON-RPC request
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
    let (response, mut child) = spawn_and_send(request, 10);

    // Validate response
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .expect("Response should be valid JSON");
    assert_eq!(parsed["jsonrpc"], "2.0", "Should be JSON-RPC 2.0");
    assert_eq!(parsed["id"], 1, "Should echo request ID");
    assert!(parsed.get("result").is_some(), "Should have a result field");
    assert!(parsed.get("error").is_none(), "Should not have an error field");

    // The result should contain a tools list
    let result = parsed.get("result").unwrap();
    assert!(result.get("tools").is_some(), "Result should have tools array");
    let tools = result["tools"].as_array().unwrap();
    assert!(!tools.is_empty(), "Should have at least one tool");

    // Clean shutdown
    let _ = child.kill();
    let _ = child.wait();
}

/// Black-box E2E test: verify the server returns an error for an unknown method.
#[ignore]
#[test]
fn test_e2e_unknown_method() {
    let request = r#"{"jsonrpc":"2.0","id":2,"method":"nonexistent_tool"}"#;
    let (response, mut child) = spawn_and_send(request, 5);

    let parsed: serde_json::Value = serde_json::from_str(&response)
        .expect("Response should be valid JSON");
    assert_eq!(parsed["jsonrpc"], "2.0");
    assert_eq!(parsed["id"], 2);
    assert!(parsed.get("error").is_some(), "Should have an error for unknown tool");

    // Clean shutdown
    let _ = child.kill();
    let _ = child.wait();
}

/// Test that clean-ctx init creates the expected output.
#[ignore]
#[test]
fn test_e2e_init_subcommand() {
    let dir = tempfile::TempDir::new().unwrap();
    let binary = binary_path();

    let mut child = Command::new(binary)
        .arg("init")
        .current_dir(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn clean-ctx init");

    let output = child.wait_with_output().expect("Failed to wait for init");
    assert!(output.status.success(), "init subcommand should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Created config"), "Should indicate config was created");

    // Verify the config file was created
    let config_path = dir.path().join(".clean-ctx.json");
    assert!(config_path.exists(), "Config file should exist");

    let config_content = std::fs::read_to_string(&config_path).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&config_content).unwrap();
    assert_eq!(parsed["enabled"], true, "Config should have enabled=true");
}

/// Test that clean-ctx --config-dump produces output.
#[ignore]
#[test]
fn test_e2e_config_dump() {
    let dir = tempfile::TempDir::new().unwrap();
    let binary = binary_path();

    // First init to create config
    let init_status = Command::new(&binary)
        .arg("init")
        .current_dir(dir.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("Failed to run init");
    assert!(init_status.success());

    // Now dump config
    let mut child = Command::new(&binary)
        .arg("--config-dump")
        .current_dir(dir.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn config-dump");

    let output = child.wait_with_output().expect("Failed to wait for config-dump");
    assert!(output.status.success(), "config-dump should succeed");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Resolved Configuration"), "Should show config dump");
}