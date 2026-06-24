// src/protocol.rs
//
// JSON-RPC 2.0 protocol types and serialization.
//
// A-09 (thread safety): send_response uses a global stdout Mutex to
// prevent interleaved responses when multiple worker threads write
// concurrently. The main thread (stdin reader) never writes responses;
// only workers do, and they serialize through the mutex.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;

/// Global stdout mutex to prevent interleaved JSON-RPC responses
/// from concurrent worker threads.
static STDOUT_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// Send a JSON-RPC response to stdout.
///
/// Thread-safe: uses a global mutex to serialize all stdout writes.
/// Multiple worker threads can call this concurrently without
/// interleaving response lines.
pub fn send_response(val: &serde_json::Value) {
    use std::io::{self, Write};
    // Lock the global stdout mutex to prevent interleaved writes
    let _lock = STDOUT_MUTEX.lock().unwrap();
    let mut stdout = io::stdout().lock();
    if let Ok(payload) = serde_json::to_string(val) {
        let _ = writeln!(stdout, "{}", payload);
        let _ = stdout.flush();
    }
}
