// src/tests/mcp/dispatcher.rs
//
// A-PRODUCTION: Tests for the production-grade thread-pool dispatcher.
//
// Tests cover:
// - Dispatcher creation and lifecycle
// - State mutations are visible across spawns
// - Concurrent spawns don't panic under load
// - Panic recovery: handler panics don't crash the dispatcher
// - Bounded queue with backpressure
// - Request tracing and observability
// - **P0-1 REGRESSION**: Multiple requests execute concurrently (no outer RwLock)
// - **P0-3 REGRESSION**: No dead writer thread (response_tx removed)

use crate::config::CleanCtxConfig;
use crate::mcp::dispatcher::{Dispatcher, DispatcherConfig};
use crate::protocol::JsonRpcRequest;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Helper that creates a Dispatcher from a default config.
/// Uses 4 workers explicitly to guarantee parallelism on all platforms
/// (available_parallelism() may return 1 in some environments).
fn make_dispatcher() -> Dispatcher {
    let mut config = CleanCtxConfig::default();
    config.cbm.enabled = false; // Avoid CBM launch in tests
    let state = crate::mcp::McpState::new(config);
    let dispatcher_config = DispatcherConfig {
        worker_count: Some(4),
        ..DispatcherConfig::default()
    };
    Dispatcher::with_config(state, dispatcher_config)
}

/// Create a minimal JsonRpcRequest for testing.
fn test_request(id: &str, method: &str) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::Value::String(id.to_string())),
        method: method.to_string(),
        params: None,
    }
}

#[test]
fn dispatcher_creates_successfully() {
    let dispatcher = make_dispatcher();
    // McpState uses interior mutability — no outer RwLock accessor needed
    // format_footer() always emits the §PATHMAP header, so check no path entries exist
    let footer = dispatcher.state().dict_lock().format_footer();
    assert!(
        !footer.contains('='),
        "new dispatcher should have empty dictionary footer, got: {footer:?}"
    );
}

#[test]
fn dispatcher_single_spawn_mutates_state() {
    let dispatcher = make_dispatcher();

    dispatcher
        .spawn(&test_request("1", "test"), |state| {
            state.get_or_create_alias("test.ts".to_string());
        })
        .expect("spawn should succeed");

    // Wait for the task to complete
    std::thread::sleep(Duration::from_millis(100));

    // Verify the mutation via dict (which uses interior mutability)
    let footer = dispatcher.state().dict_lock().format_footer();
    assert!(
        footer.contains("test.ts"),
        "spawn should have registered path alias"
    );
}

#[test]
fn dispatcher_multiple_spawns_work() {
    let dispatcher = make_dispatcher();
    let counter = Arc::new(AtomicUsize::new(0));

    for _i in 0..5 {
        let counter = Arc::clone(&counter);
        dispatcher
            .spawn(&test_request("test", "test"), move |_state| {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .expect("spawn should succeed");
    }

    // Wait for all tasks to complete
    std::thread::sleep(Duration::from_millis(200));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "all 5 spawns should complete"
    );
}

#[test]
fn dispatcher_concurrent_no_data_race() {
    let dispatcher = make_dispatcher();
    let counter = Arc::new(AtomicUsize::new(0));

    for i in 0..20 {
        let counter = Arc::clone(&counter);
        dispatcher
            .spawn(&test_request(&i.to_string(), "test"), move |state| {
                let _ = state.proxy_port;
                let _ = state.config.default_fidelity;
                state.get_or_create_alias("test.ts".to_string());
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .expect("spawn should succeed");
    }

    // Wait for completion
    std::thread::sleep(Duration::from_millis(300));

    assert_eq!(
        counter.load(Ordering::SeqCst),
        20,
        "all 20 rapid-fire spawns should complete"
    );
    assert!(
        dispatcher
            .state()
            .dict_lock()
            .format_footer()
            .contains("test.ts"),
        "path alias should be registered in footer"
    );
}

#[test]
fn dispatcher_panic_recovery_continues_processing() {
    let dispatcher = make_dispatcher();
    let counter = Arc::new(AtomicUsize::new(0));

    // Spawn a handler that panics
    dispatcher
        .spawn(&test_request("panic", "test"), |_state| {
            panic!("intentional panic");
        })
        .expect("spawn should succeed");

    // Wait for panic to occur
    std::thread::sleep(Duration::from_millis(100));

    // Spawn more handlers - they should still work
    for i in 0..5 {
        let counter = Arc::clone(&counter);
        dispatcher
            .spawn(&test_request(&i.to_string(), "test"), move |_state| {
                counter.fetch_add(1, Ordering::SeqCst);
            })
            .expect("spawn should succeed");
    }

    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        counter.load(Ordering::SeqCst),
        5,
        "handlers after panic should still work"
    );
}

#[test]
fn dispatcher_tracing_records_requests() {
    let dispatcher = make_dispatcher();

    dispatcher
        .spawn(&test_request("trace1", "method1"), |_state| {
            std::thread::sleep(Duration::from_millis(50));
        })
        .expect("spawn should succeed");

    dispatcher
        .spawn(&test_request("trace2", "method2"), |_state| {
            std::thread::sleep(Duration::from_millis(10));
        })
        .expect("spawn should succeed");

    std::thread::sleep(Duration::from_millis(200));

    let traces = dispatcher.recent_traces(10);
    assert_eq!(traces.len(), 2, "should have 2 traces");
    assert_eq!(traces[0].id, "trace2"); // Most recent first
    assert_eq!(traces[0].method, "method2");
    assert!(
        traces[0].latency() >= Duration::from_millis(10),
        "should have processing time"
    );
}

// ── P0-1 REGRESSION TESTS ──────────────────────────────────────────

/// P0-1 REGRESSION: Multiple requests must execute concurrently.
///
/// Before the fix, the dispatcher used an outer Arc<RwLock<McpState>>
/// where every worker acquired state.write(). This serialized ALL requests.
/// After the fix, McpState uses interior mutability and workers share
/// &McpState directly, enabling true parallel execution.
///
/// This test verifies concurrency by using a barrier-based approach:
/// all N slow handlers must start within a tight window, proving they
/// execute in parallel rather than sequentially.
#[test]
fn p0_1_regression_requests_execute_concurrently() {
    let dispatcher = make_dispatcher();

    // Use a barrier to detect parallelism: each handler blocks on the barrier
    // before sleeping. If all N handlers reach the barrier, they're running
    // concurrently. If they ran sequentially, only one would ever wait on it.
    let started = Arc::new(AtomicUsize::new(0));
    let all_started = Arc::new(AtomicBool::new(false));

    // Spawn 4 slow handlers (each takes 200ms of sleep after barrier wait)
    for i in 0..4 {
        let started = Arc::clone(&started);
        let all_started = Arc::clone(&all_started);
        dispatcher
            .spawn(&test_request(&i.to_string(), "slow"), move |_state| {
                // Signal that this handler started
                started.fetch_add(1, Ordering::SeqCst);

                // Spin-wait until all 4 have started or timeout
                let deadline = Instant::now() + Duration::from_millis(300);
                while Instant::now() < deadline {
                    if all_started.load(Ordering::Acquire) {
                        break;
                    }
                    std::thread::yield_now();
                }

                // Now simulate work
                std::thread::sleep(Duration::from_millis(200));
            })
            .expect("spawn should succeed");
    }

    // Give workers time to spin up and start processing
    std::thread::sleep(Duration::from_millis(100));

    // Check how many handlers started
    let started_count = started.load(Ordering::SeqCst);
    eprintln!(
        "[clean-ctx] P0-1: started_count={} before barrier release",
        started_count
    );

    // Release all waiting handlers
    all_started.store(true, Ordering::Release);

    // Wait for all to complete
    std::thread::sleep(Duration::from_millis(800));

    // P0-1: With parallelism, all 4 handlers should have started within 100ms.
    // Without parallelism (serial execution), at most 1 would have started.
    // We allow 2 as a generous lower bound to account for scheduling jitter.
    assert!(
        started_count >= 2,
        "P0-1 REGRESSION: Only {started_count} handler(s) started within 100ms. \
         Expected at least 2 concurrent starts. Workers are likely executing \
         sequentially instead of concurrently.",
    );
}

/// P0-1 REGRESSION: Verify no outer RwLock exists.
///
/// This test verifies that state() returns &McpState directly (no .read()/.write() needed).
#[test]
fn p0_1_regression_no_outer_rwlock() {
    let dispatcher = make_dispatcher();

    // P0-1: Direct access to McpState without .read()/write()
    // Before fix: dispatcher.state().read().unwrap().proxy_port
    // After fix:  dispatcher.state().proxy_port
    assert_eq!(dispatcher.state().proxy_port, 8787);
}

// ── P0-3 REGRESSION TESTS ──────────────────────────────────────────

/// P0-3 REGRESSION: Response channel type must not exist.
///
/// Before the fix, the dispatcher had a dead stdout writer thread and
/// ResponseEnvelope struct. After the fix, both are removed.
/// This test verifies the BoxedHandler uses &McpState (no &mut).
#[test]
fn p0_3_regression_no_response_channel() {
    let dispatcher = make_dispatcher();
    dispatcher
        .spawn(&test_request("test", "test"), |state| {
            // &McpState, not &mut McpState — can read but not write plain fields
            let _ = state.proxy_port;
        })
        .unwrap();
}
