use super::*;
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn config(auto_launch: bool) -> crate::config::CleanCtxConfig {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = true;
    config.cbm.auto_launch = auto_launch;
    config.cbm.binary_path = Some("definitely-missing-cbm-fixture".into());
    config
}

fn wait_for_attempts(state: &crate::mcp::McpState, expected: usize) {
    let deadline = Instant::now() + Duration::from_secs(1);
    while state.cbm_admission_attempts_for_test() != expected {
        assert!(
            Instant::now() < deadline,
            "expected {expected} admission attempt(s), observed {}",
            state.cbm_admission_attempts_for_test()
        );
        std::thread::yield_now();
    }
}

#[test]
fn successful_clean_initialize_triggers_auto_launch_once() {
    let state = crate::mcp::McpState::new(config(true));
    state.skip_cbm_provider_build_for_test();
    assert_eq!(state.cbm_admission_attempts_for_test(), 0);

    crate::mcp::handlers::handle_initialize(&serde_json::json!(1), &state);
    wait_for_attempts(&state, 1);
    crate::mcp::handlers::handle_initialize(&serde_json::json!(2), &state);
    assert_eq!(state.cbm_admission_attempts_for_test(), 1);
}

#[test]
fn auto_launch_false_waits_for_first_provider_consumer() {
    let state = crate::mcp::McpState::new(config(false));
    state.skip_cbm_provider_build_for_test();
    crate::mcp::handlers::handle_initialize(&serde_json::json!(1), &state);
    assert_eq!(state.cbm_admission_attempts_for_test(), 0);

    state.prepare_cbm_for_consumer();
    wait_for_attempts(&state, 1);
}

#[test]
fn concurrent_consumers_coalesce_onto_one_admission_attempt() {
    let state = Arc::new(crate::mcp::McpState::new(config(false)));
    state.skip_cbm_provider_build_for_test();
    let callers: Vec<_> = (0..8)
        .map(|_| {
            let state = Arc::clone(&state);
            std::thread::spawn(move || state.prepare_cbm_for_consumer())
        })
        .collect();
    for caller in callers {
        caller.join().expect("consumer thread");
    }
    wait_for_attempts(&state, 1);
}

#[test]
fn admission_is_observably_initializing_and_shutdown_joins_its_owner() {
    let lifecycle = CbmLifecycle::new(
        config(false).cbm,
        std::env::current_dir().expect("cwd"),
        &[],
    );
    lifecycle.skip_provider_build_for_test();
    let gate = lifecycle.block_admission_for_test();
    lifecycle.start_for_consumer();
    let deadline = Instant::now() + Duration::from_secs(1);
    while lifecycle.admission_attempts_for_test() != 1 {
        assert!(Instant::now() < deadline, "admission did not start");
        std::thread::yield_now();
    }
    assert_eq!(lifecycle.status(), crate::cbm::CbmStatus::Initializing);

    let (done_tx, done_rx) = mpsc::channel();
    let shutdown_owner = lifecycle.clone();
    let shutdown = std::thread::spawn(move || {
        shutdown_owner.shutdown();
        done_tx.send(()).expect("shutdown completion");
    });
    assert!(
        done_rx.recv_timeout(Duration::from_millis(50)).is_err(),
        "shutdown returned while admission remained in flight"
    );

    gate.wait();
    done_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("owned admission shutdown must complete");
    shutdown.join().expect("shutdown thread");
    assert_eq!(lifecycle.status(), crate::cbm::CbmStatus::Unavailable);
}

#[test]
fn provider_consumer_fails_closed_while_admission_is_initializing() {
    let _responses = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(config(false));
    state.skip_cbm_provider_build_for_test();
    let gate = state.block_cbm_admission_for_test();
    state.prepare_cbm_for_consumer();
    wait_for_attempts(&state, 1);

    crate::cbm::handlers::handle_graph_search(
        &serde_json::json!(1),
        &serde_json::json!({"arguments": {"query": "GraphBridge"}}),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("provider consumer response");
    assert_eq!(response["error"]["code"], -32603);
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("initializing")),
        "in-flight admission was not reported truthfully: {response}"
    );
    assert_eq!(state.cbm_admission_attempts_for_test(), 1);

    gate.wait();
}

#[test]
fn status_probe_is_report_only_when_lazy_launch_is_dormant() {
    let _responses = crate::protocol::handler_response_serial();
    crate::protocol::captured_responses().clear();
    let state = crate::mcp::McpState::new(config(false));
    state.skip_cbm_provider_build_for_test();

    crate::cbm::handlers::handle_get_cbm_status(
        &serde_json::json!(1),
        &serde_json::json!({}),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("status response");
    assert_eq!(response["result"]["_meta"]["cbm_status"], "unavailable");
    assert_eq!(state.cbm_admission_attempts_for_test(), 0);
}
