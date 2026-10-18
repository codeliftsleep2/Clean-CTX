use super::{CbmClient, CbmError, a2_support as fixture};
use serde_json::json;
use std::path::Path;
use tempfile::TempDir;

fn launch(root: &Path, log: &Path, mode: &str) -> CbmClient {
    fixture::launch(root, log, mode, None).0.unwrap().unwrap()
}

fn assert_same_directory(observed: &serde_json::Value, expected: &Path, message: &str) {
    let observed = Path::new(observed.as_str().expect("fixture cwd string"))
        .canonicalize()
        .expect("canonical observed fixture cwd");
    let expected = expected
        .canonicalize()
        .expect("canonical expected fixture cwd");
    assert_eq!(observed, expected, "{message}");
}

#[test]
fn cbm_a2_launch_uses_explicit_root_when_parent_differs() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    assert_ne!(
        std::env::current_dir().unwrap(),
        root.path().canonicalize().unwrap()
    );
    let _client = launch(root.path(), &log, "normal");
    let observed: serde_json::Value = serde_json::from_str(&fixture::wait_file(&log)).unwrap();
    assert_same_directory(
        &observed["cwd"],
        root.path(),
        "frontend inherited the parent CWD",
    );
}

#[test]
fn cbm_a2_baseline_matching_parent_root_and_account_environment() {
    let temp = TempDir::new().unwrap();
    let log = temp.path().join("launch.json");
    let _client = launch(&std::env::current_dir().unwrap(), &log, "normal");
    let observed: serde_json::Value = serde_json::from_str(&fixture::wait_file(&log)).unwrap();
    assert_eq!(observed["cwd"], json!(std::env::current_dir().unwrap()));
    assert_eq!(observed["cache"], json!(std::env::var_os("CBM_CACHE_DIR")));
}

#[test]
fn cbm_a2_stdout_worker_failure_rejects_partial_launch() {
    let root = TempDir::new().unwrap();
    let (result, pid) = fixture::launch(
        root.path(),
        &root.path().join("launch.json"),
        "normal",
        Some("cbm-stdout-reader"),
    );
    assert!(
        matches!(result, Err(CbmError::LaunchError(_))),
        "missing required stdout reader was accepted"
    );
    fixture::assert_process_gone(pid);
}

#[test]
fn cbm_a2_stderr_worker_failure_rejects_partial_launch() {
    let root = TempDir::new().unwrap();
    let (result, pid) = fixture::launch(
        root.path(),
        &root.path().join("launch.json"),
        "normal",
        Some("cbm-stderr-drain"),
    );
    assert!(
        matches!(result, Err(CbmError::LaunchError(_))),
        "missing required stderr drainer was accepted"
    );
    fixture::assert_process_gone(pid);
}

#[test]
fn cbm_a2_eof_retires_and_blocks_next_write() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let mut client = launch(root.path(), &log, "exit");
    fixture::wait_file(&log);
    assert!(client.call_tool_raw("query_graph", json!({})).is_err());
    let id = fixture::request_id(&client);
    assert_eq!(
        fixture::owned_resources(&client),
        0,
        "dead frontend resources remain reusable"
    );
    assert!(client.call_tool("query_graph", json!({})).is_err());
    assert_eq!(
        fixture::request_id(&client),
        id,
        "retired frontend attempted another request"
    );
}

#[test]
fn cbm_a2_explicit_retirement_is_idempotent_and_closes_resources() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let mut client = launch(root.path(), &log, "normal");
    fixture::wait_file(&log);
    fixture::retire(&mut client);
    fixture::retire(&mut client);
    assert_eq!(
        fixture::owned_resources(&client),
        0,
        "retirement retained owned frontend resources"
    );
    assert!(
        fixture::workers_finished(&client),
        "owned workers are still running"
    );
    let id = fixture::request_id(&client);
    assert!(client.call_tool_raw("query_graph", json!({})).is_err());
    assert_eq!(fixture::request_id(&client), id);
    drop(client);
}

#[test]
fn cbm_a2_protocol_poison_uses_coordinated_retirement() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let mut client = launch(root.path(), &log, "poison");
    assert!(matches!(
        client.call_tool_raw("query_graph", json!({})),
        Err(CbmError::ParseError(_))
    ));
    assert_eq!(
        fixture::owned_resources(&client),
        0,
        "A1 poison retained pipes or workers"
    );
}

#[test]
fn cbm_a2_timeout_retires_without_reusing_dead_transport() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let mut client = launch(root.path(), &log, "stall");
    assert!(
        matches!(
            client.call_tool("query_graph", json!({})),
            Err(CbmError::Timeout(_))
        ),
        "original timeout was lost to dead-pipe retry"
    );
    assert_eq!(
        fixture::request_id(&client),
        2,
        "timeout retried the retired frontend"
    );
    assert_eq!(fixture::owned_resources(&client), 0);
}

#[test]
fn cbm_a2_retirement_releases_full_stdout_queue() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let mut client = launch(root.path(), &log, "flood");
    fixture::wait_file(&log);
    fixture::retire(&mut client);
    assert!(
        fixture::workers_finished(&client),
        "queue-blocked reader was not joined"
    );
    assert_eq!(fixture::owned_resources(&client), 0);
}

#[test]
fn cbm_a2_baseline_additional_roots_share_one_frontend_and_explicit_routing() {
    let primary = TempDir::new().unwrap();
    let additional = TempDir::new().unwrap();
    let log = primary.path().join("launch.json");
    let mut bridge = fixture::bridge(primary.path(), &[additional.path().to_path_buf()], &log);
    fixture::wait_file(&log);
    let primary_id = bridge.resolve_project_id(primary.path().to_str().unwrap());
    let additional_id = bridge.resolve_project_id(additional.path().to_str().unwrap());
    bridge.query_graph_scoped("MATCH (n) RETURN n.name", &primary_id);
    bridge.query_graph_scoped("MATCH (n) RETURN n.name", &additional_id);
    assert!(bridge.take_last_error().is_none());
    let requests: Vec<serde_json::Value> = std::fs::read_to_string(log.with_extension("requests"))
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for project in [&primary_id, &additional_id] {
        assert!(
            requests
                .iter()
                .any(|request| request["params"]["name"] == "query_graph"
                    && request["params"]["arguments"]["project"] == *project)
        );
    }
    assert_eq!(
        std::fs::read_to_string(log.with_extension("launches"))
            .unwrap()
            .lines()
            .count(),
        1
    );
}

#[test]
fn cbm_a2_bridge_primary_root_is_frontend_cwd_despite_additional_roots() {
    let primary = TempDir::new().unwrap();
    let additional = TempDir::new().unwrap();
    let log = primary.path().join("launch.json");
    let _bridge = fixture::bridge(primary.path(), &[additional.path().to_path_buf()], &log);
    let observed: serde_json::Value = serde_json::from_str(&fixture::wait_file(&log)).unwrap();
    assert_same_directory(&observed["cwd"], primary.path(), "frontend used wrong root");
}

#[test]
fn cbm_a2_baseline_retirement_preserves_other_frontend_session() {
    let root = TempDir::new().unwrap();
    let mut first = launch(root.path(), &root.path().join("first.json"), "normal");
    let mut other = launch(root.path(), &root.path().join("other.json"), "normal");
    fixture::retire(&mut first);
    drop(first);
    assert!(other.call_tool_raw("query_graph", json!({})).is_ok());
}

#[test]
fn cbm_indexing_worker_is_joined_before_bridge_retires_frontend() {
    let root = TempDir::new().unwrap();
    let log = root.path().join("launch.json");
    let bridge = fixture::bridge_with_mode(root.path(), &[], &log, "a2-slow-index");
    let pid = {
        let client = bridge.client.lock().unwrap();
        fixture::child_id(client.as_ref().expect("admitted frontend"))
    };
    let requests = log.with_extension("requests");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    loop {
        let indexing_started =
            std::fs::read_to_string(&requests).is_ok_and(|text| text.contains("index_repository"));
        if indexing_started {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "fixture indexing request did not start"
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    drop(bridge);
    fixture::assert_process_gone(pid);
}
