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

/// P0-5: Lock recovery macro for protocol-level mutexes.
/// Applied to STDOUT_MUTEX to prevent poisoned-lock panics from
/// crashing all future responses.
macro_rules! lock_or_recover_protocol {
    ($lock:expr, $name:expr) => {
        match $lock {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!(
                    "[clean-ctx] WARNING: Recovering from poisoned lock ({})",
                    $name
                );
                poisoned.into_inner()
            }
        }
    };
}

/// Send a JSON-RPC response to stdout.
///
/// Thread-safe: uses a global mutex to serialize all stdout writes.
/// Multiple worker threads can call this concurrently without
/// interleaving response lines.
///
/// P0-5: Uses lock_or_recover_protocol! to handle poisoned locks gracefully.
pub fn send_response(val: &serde_json::Value) {
    use std::io::{self, Write};
    // P0-5: Lock the global stdout mutex with poison recovery
    let _lock = lock_or_recover_protocol!(STDOUT_MUTEX.lock(), "stdout");
    let mut stdout = io::stdout().lock();
    if let Ok(payload) = serde_json::to_string(val) {
        let _ = writeln!(stdout, "{}", payload);
        let _ = stdout.flush();
    }
}
