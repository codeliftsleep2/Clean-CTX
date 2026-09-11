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

#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::ops::{Deref, DerefMut};
#[cfg(test)]
use std::sync::{LazyLock, LockResult, MutexGuard, PoisonError};

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
///
/// Each test thread gets its own queue. This preserves the established
/// `.lock().clear()` / `.lock().pop()` API while preventing parallel handler
/// tests from erasing or stealing one another's responses.
#[cfg(test)]
pub(crate) static CAPTURED_RESPONSES: LazyLock<CapturedResponses> =
    LazyLock::new(CapturedResponses::new);

#[cfg(test)]
pub(crate) struct CapturedResponses {
    queues: Mutex<HashMap<std::thread::ThreadId, Vec<serde_json::Value>>>,
}

#[cfg(test)]
impl CapturedResponses {
    fn new() -> Self {
        Self {
            queues: Mutex::new(HashMap::new()),
        }
    }

    pub(crate) fn lock(&self) -> LockResult<CapturedResponsesGuard<'_>> {
        let thread = std::thread::current().id();
        match self.queues.lock() {
            Ok(mut queues) => {
                queues.entry(thread).or_default();
                Ok(CapturedResponsesGuard { queues, thread })
            }
            Err(poisoned) => {
                let mut queues = poisoned.into_inner();
                queues.entry(thread).or_default();
                Err(PoisonError::new(CapturedResponsesGuard { queues, thread }))
            }
        }
    }
}

#[cfg(test)]
pub(crate) struct CapturedResponsesGuard<'a> {
    queues: MutexGuard<'a, HashMap<std::thread::ThreadId, Vec<serde_json::Value>>>,
    thread: std::thread::ThreadId,
}

#[cfg(test)]
impl Deref for CapturedResponsesGuard<'_> {
    type Target = Vec<serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        self.queues
            .get(&self.thread)
            .expect("current test thread queue initialized")
    }
}

#[cfg(test)]
impl DerefMut for CapturedResponsesGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.queues
            .get_mut(&self.thread)
            .expect("current test thread queue initialized")
    }
}

/// Poison-tolerant guard for [`CAPTURED_RESPONSES`].
///
/// A test that fails while draining the sink must never cascade into
/// `PoisonError` storms across sibling handler tests (observed 2026-08-27
/// under full-suite parallelism: one empty-pop panic held the guard,
/// poisoned the sink, and failed three unrelated tests). The producer in
/// [`send_response`] already tolerates poison (`if let Ok`); consumers
/// get the same courtesy here.
///
/// Available in every test configuration because the protocol-level
/// concurrency regression is feature-independent.
#[cfg(test)]
pub(crate) fn captured_responses() -> CapturedResponsesGuard<'static> {
    match CAPTURED_RESPONSES.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Legacy shared gate for handler suites that also coordinate other
/// process-global test state. Response capture itself is thread-isolated, but
/// the established gate remains available to its existing Phase A/B callers.
///
/// Feature-gated to mirror its Phase A/B consumers.
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

#[cfg(test)]
#[path = "tests/protocol.rs"]
mod tests;
