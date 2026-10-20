use super::{
    TEST_PROJECT_HYDRATION_SERIALIZE, clear_test_project_search_results,
    fail_test_publication_once, hydrate_workspace_index, reset_total_test_scan_calls,
    total_test_scan_calls,
};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn flight_identity_separates_distinct_hydration_work() {
    use crate::mcp::acquisition::AcquisitionTarget;
    use crate::mcp::discovery_cache::DiscoveryMode;
    use crate::mcp::hydration_flight::HydrationFlightKey;
    use crate::workspace::index::SemanticFidelity;

    let key = |roots: &[&str],
               discovery,
               name: &str,
               fidelity,
               proof_target: Option<AcquisitionTarget>| {
        HydrationFlightKey::new(
            roots.iter().map(|root| (*root).to_string()).collect(),
            discovery,
            name,
            fidelity,
            proof_target,
        )
    };
    let baseline = key(
        &["root-a"],
        DiscoveryMode::Declaration,
        "ContextA",
        Some(SemanticFidelity::High),
        None,
    );

    assert_ne!(
        baseline,
        key(
            &["root-b"],
            DiscoveryMode::Declaration,
            "ContextA",
            Some(SemanticFidelity::High),
            None,
        )
    );
    assert_ne!(
        baseline,
        key(
            &["root-a"],
            DiscoveryMode::InboundReference,
            "ContextA",
            Some(SemanticFidelity::High),
            None,
        )
    );
    assert_ne!(
        baseline,
        key(
            &["root-a"],
            DiscoveryMode::Declaration,
            "ContextB",
            Some(SemanticFidelity::High),
            None,
        )
    );
    assert_ne!(
        baseline,
        key(
            &["root-a"],
            DiscoveryMode::Declaration,
            "ContextA",
            Some(SemanticFidelity::Low),
            None,
        )
    );
    assert_ne!(
        baseline,
        key(
            &["root-a"],
            DiscoveryMode::Declaration,
            "ContextA",
            Some(SemanticFidelity::High),
            Some(AcquisitionTarget::exact_type_ref("ContextA")),
        )
    );
}

struct ResetFlightTest(Arc<crate::mcp::McpState>);

impl Drop for ResetFlightTest {
    fn drop(&mut self) {
        self.0.hydration_flights.test_release();
        clear_test_project_search_results();
    }
}

fn filesystem_state() -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn hydrate(state: &crate::mcp::McpState, root: &str, name: &str) -> super::HydrationReport {
    hydrate_workspace_index(state, "find_entities", name, Some(root)).expect("hydration attempt")
}

#[test]
fn equivalent_concurrent_cold_hydration_executes_expensive_work_once() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(root.path().join("Target.ts"), "export class Target {}\n")
        .expect("target source");
    let state = Arc::new(filesystem_state());
    let _reset = ResetFlightTest(Arc::clone(&state));
    reset_total_test_scan_calls(&state);
    state.reset_test_full_semantic_parses();
    state.hydration_flights.test_arm();

    let root_text = root.path().to_string_lossy().into_owned();
    let first_state = Arc::clone(&state);
    let first_root = root_text.clone();
    let first = std::thread::spawn(move || hydrate(&first_state, &first_root, "Target"));
    state.hydration_flights.test_wait_for_leader();
    let second_state = Arc::clone(&state);
    let second_root = root_text.clone();
    let second = std::thread::spawn(move || hydrate(&second_state, &second_root, "Target"));
    state.hydration_flights.test_wait_for_followers(1);
    state.hydration_flights.test_release();

    let first = first.join().expect("leader hydration");
    let second = second.join().expect("follower hydration");
    assert!(first.is_complete() && second.is_complete());
    assert_eq!(
        total_test_scan_calls(&state),
        1,
        "filesystem inventory must coalesce"
    );
    assert_eq!(
        state.test_full_semantic_parses(),
        1,
        "owner compilation must coalesce"
    );
    assert_eq!(
        first.candidates_compiled + second.candidates_compiled,
        1,
        "the follower must not publish an additional equivalent projection"
    );
}

#[test]
fn incomplete_leader_does_not_grant_follower_completion() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().expect("workspace");
    let target = root.path().join("Target.ts");
    std::fs::write(&target, "export class Target {}\n").expect("target source");
    let state = Arc::new(filesystem_state());
    let _reset = ResetFlightTest(Arc::clone(&state));
    fail_test_publication_once(&target.to_string_lossy());
    reset_total_test_scan_calls(&state);
    state.hydration_flights.test_arm();

    let root_text = root.path().to_string_lossy().into_owned();
    let first_state = Arc::clone(&state);
    let first_root = root_text.clone();
    let first = std::thread::spawn(move || hydrate(&first_state, &first_root, "Target"));
    state.hydration_flights.test_wait_for_leader();
    let second_state = Arc::clone(&state);
    let second_root = root_text.clone();
    let second = std::thread::spawn(move || hydrate(&second_state, &second_root, "Target"));
    state.hydration_flights.test_wait_for_followers(1);
    state.hydration_flights.test_release();

    let first = first.join().expect("incomplete leader");
    let second = second.join().expect("retrying follower");
    assert!(
        !first.is_complete(),
        "the injected leader failure remains incomplete"
    );
    assert!(
        second.is_complete(),
        "the follower must recheck and retry residual work"
    );
    assert_eq!(
        total_test_scan_calls(&state),
        2,
        "incomplete discovery is not inherited; the follower performs a fresh attempt"
    );
}

#[test]
fn unrelated_hydration_keys_execute_while_another_key_is_active() {
    let _serial = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(root.path().join("Alpha.ts"), "export class Alpha {}\n").unwrap();
    std::fs::write(root.path().join("Beta.ts"), "export class Beta {}\n").unwrap();
    let state = Arc::new(filesystem_state());
    let _reset = ResetFlightTest(Arc::clone(&state));
    state.hydration_flights.test_arm();

    let root_text = root.path().to_string_lossy().into_owned();
    let alpha_state = Arc::clone(&state);
    let alpha_root = root_text.clone();
    let alpha = std::thread::spawn(move || hydrate(&alpha_state, &alpha_root, "Alpha"));
    state.hydration_flights.test_wait_for_leader();

    let (sent, received) = std::sync::mpsc::channel();
    let beta_state = Arc::clone(&state);
    let beta_root = root_text;
    let beta = std::thread::spawn(move || {
        let report = hydrate(&beta_state, &beta_root, "Beta");
        sent.send(report.is_complete())
            .expect("beta completion signal");
    });
    assert_eq!(
        received.recv_timeout(Duration::from_secs(10)),
        Ok(true),
        "an unrelated hydration key must complete while Alpha remains paused"
    );
    state.hydration_flights.test_release();
    assert!(alpha.join().expect("alpha hydration").is_complete());
    beta.join().expect("beta hydration");
}
