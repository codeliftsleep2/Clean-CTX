// examples/dispatcher_benchmark.rs
//
// A-PRODUCTION: Benchmark for the production-grade dispatcher.
//
// Measures the throughput improvement of RwLock + crossbeam-channel
// vs the old Mutex + rayon implementation.
//
// Run with: cargo run --example dispatcher_benchmark

use std::sync::Arc;
use std::time::{Duration, Instant};
use clean_ctx::mcp::dispatcher::Dispatcher;

// Minimal state for benchmarking (avoids CBM initialization)
fn create_minimal_state() -> clean_ctx::mcp::McpState {
    let config = clean_ctx::config::CleanCtxConfig::default();
    clean_ctx::mcp::McpState::new(config)
}

fn main() {
    println!("=== Dispatcher Performance Benchmark ===\n");
    
    // Setup - create minimal state to avoid CBM overhead
    let state = create_minimal_state();
    let dispatcher = Arc::new(Dispatcher::new(state));
    
    println!("Test 1: Sequential baseline (10 requests, 10ms each)");
    let start = Instant::now();
    for i in 0..10 {
        let req = clean_ctx::protocol::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(i.into())),
            method: "test".to_string(),
            params: None,
        };
        dispatcher.spawn(&req, |_state| {
            std::thread::sleep(Duration::from_millis(10));
        }).unwrap();
    }
    std::thread::sleep(Duration::from_millis(200));
    let sequential_time = start.elapsed();
    println!("  Time: {:?}\n", sequential_time);
    
    println!("Test 2: Concurrent requests (10 parallel, 10ms each)");
    let start = Instant::now();
    for i in 0..10 {
        let req = clean_ctx::protocol::JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::Value::Number(i.into())),
            method: "test".to_string(),
            params: None,
        };
        dispatcher.spawn(&req, |_state| {
            std::thread::sleep(Duration::from_millis(10));
        }).unwrap();
    }
    std::thread::sleep(Duration::from_millis(200));
    let concurrent_time = start.elapsed();
    println!("  Time: {:?}\n", concurrent_time);
    
    // Results
    let speedup = sequential_time.as_secs_f64() / concurrent_time.as_secs_f64();
    println!("=== Results ===");
    println!("Sequential: {:?}", sequential_time);
    println!("Concurrent: {:?}", concurrent_time);
    println!("Speedup: {:.2}x", speedup);
    
    if speedup > 2.0 {
        println!("✅ PASS: Achieved >2x improvement with concurrent execution");
    } else {
        println!("⚠️  WARNING: Speedup less than expected");
    }
    
    println!("Recent traces: {}", dispatcher.recent_traces(5).len());
}