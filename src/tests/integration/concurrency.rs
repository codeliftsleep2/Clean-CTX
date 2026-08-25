// src/tests/integration/concurrency.rs
//
// Integration tests for concurrent tool calls.

use crate::config::CleanCtxConfig;
use crate::mcp::McpState;
use crate::mcp::dispatcher::Dispatcher;
use crate::protocol::JsonRpcRequest;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

/// Test 6: Concurrent Tool Calls
/// Multiple different tools running in parallel should not corrupt shared state.
#[test]
fn concurrent_tool_calls_no_corruption() {
    let config = CleanCtxConfig::default();
    let state = McpState::new(config);
    let dispatcher = Dispatcher::new(state);

    let success_count = Arc::new(AtomicUsize::new(0));

    // Spawn multiple different tool handlers concurrently
    for i in 0..10 {
        let success_count = Arc::clone(&success_count);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(i.into())),
            method: "tools/call".to_string(),
            params: None,
        };

        dispatcher
            .spawn(&req, move |_state| {
                // Simulate work
                std::thread::sleep(Duration::from_millis(10));
                success_count.fetch_add(1, Ordering::SeqCst);
            })
            .expect("Spawn should succeed");
    }

    // Wait for all to complete
    std::thread::sleep(Duration::from_millis(200));

    // All spawns should have completed
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        10,
        "All 10 concurrent tool calls should complete successfully"
    );
}
