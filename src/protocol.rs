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
        // Phase A retirement tests: record every outbound response so
        // handler tests can assert on payload CONTENT (the handlers
        // otherwise only write to stdout, which libtest cannot inspect).
        // Test-only — release builds never allocate this sink.
        #[cfg(test)]
        {
            if let Ok(mut q) = CAPTURED_RESPONSES.lock() {
                q.push(val.clone());
            }
        }
        let _ = writeln!(stdout, "{}", payload);
        let _ = stdout.flush();
    }
}

/// Test-only response capture sink (Phase A retirement regression work).
/// Pushed by [`send_response`] under `cfg(test)`; drained by handler tests.
#[cfg(test)]
pub(crate) static CAPTURED_RESPONSES: Mutex<Vec<serde_json::Value>> = Mutex::new(Vec::new());

/// Poison-tolerant guard for [`CAPTURED_RESPONSES`].
///
/// A test that fails while draining the sink must never cascade into
/// `PoisonError` storms across sibling handler tests (observed 2026-08-27
/// under full-suite parallelism: one empty-pop panic held the guard,
/// poisoned the sink, and failed three unrelated tests). The producer in
/// [`send_response`] already tolerates poison (`if let Ok`); consumers
/// get the same courtesy here.
///
/// Gated to the `rust` feature to mirror its consumers: the only drainers
/// (`phase_a_retirement_tests`, `phase_b_retirement_tests`) compile under
/// `#[cfg(all(test, feature = "rust"))]` — a bare `cfg(test)` gate would
/// leave these items dead (and warn) in default-feature test builds.
#[cfg(all(test, feature = "rust"))]
pub(crate) fn captured_responses() -> std::sync::MutexGuard<'static, Vec<serde_json::Value>> {
    match CAPTURED_RESPONSES.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Serializes every test that dispatches tool calls and drains
/// [`CAPTURED_RESPONSES`].
///
/// The sink is process-global; concurrent `clear()` → dispatch → `pop()`
/// sequences from sibling tests race (a sibling's `clear()` erases an
/// in-flight response, or another test steals it via its own `pop()`),
/// which surfaced as a spurious "handler must have sent exactly one
/// response" panic followed by a `PoisonError` cascade. Phase A originally
/// serialized only its own file; Phase B joined the contract late with no
/// gate at all — both now share this single lock.
///
/// Feature-gated to mirror its consumers (see `captured_responses`).
#[cfg(all(test, feature = "rust"))]
pub(crate) static HANDLER_RESPONSE_SERIAL: Mutex<()> = Mutex::new(());

/// Poison-tolerant guard for [`HANDLER_RESPONSE_SERIAL`] — see its docs.
#[cfg(all(test, feature = "rust"))]
pub(crate) fn handler_response_serial() -> std::sync::MutexGuard<'static, ()> {
    match HANDLER_RESPONSE_SERIAL.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}
