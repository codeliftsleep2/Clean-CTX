// src/analytics.rs
//
// Token analytics: exact local token counts using the tiktoken cl100k model
// (the same model family used by GPT-4 and many Claude context estimators).
//
// Phase 1 (FAANG audit F-01): the BPE engine used to be re-loaded on every
// call via `tiktoken_rs::cl100k_base().unwrap()`. A failed BPE load would
// take the entire MCP server process down with a SIGABRT. The engine is
// now cached in a process-global `OnceLock` so it loads exactly once and
// a load failure surfaces as a recoverable `BpeInitError` at startup time
// rather than a mid-request panic.

use std::fmt;
use std::sync::OnceLock;

use tiktoken_rs::CoreBPE;

/// Returned by [`bpe`] when the BPE data cannot be loaded.
#[derive(Debug)]
pub struct BpeInitError(pub String);

impl fmt::Display for BpeInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to load cl100k BPE data: {}", self.0)
    }
}

impl std::error::Error for BpeInitError {}

/// Process-global cached `cl100k` BPE engine. Loaded on first use and
/// reused for the lifetime of the server.
static BPE: OnceLock<CoreBPE> = OnceLock::new();

/// Eagerly initialise the global BPE engine. Call this once at server
/// startup so that any data-load failure surfaces as a clean error
/// (via [`bpe_or_init`]) instead of a mid-request panic on first use.
pub fn bpe_or_init() -> Result<&'static CoreBPE, BpeInitError> {
    if let Some(bpe) = BPE.get() {
        return Ok(bpe);
    }
    tiktoken_rs::cl100k_base()
        .map_err(|e| BpeInitError(e.to_string()))
        .map(|bpe| {
            // Race-tolerant: another thread may have inserted first; that's
            // fine because `CoreBPE` is `Arc`-like and we just want a
            // long-lived `&'static` reference. `get_or_init` gives us
            // exactly that contract.
            BPE.get_or_init(|| bpe)
        })
}

/// Returns the cached BPE engine, initialising it on first call.
///
/// **Panics only on programmer error**: if a server was started without
/// calling [`bpe_or_init`] *and* a subsequent request beats the startup
/// check to the punch. The recommended startup path is:
///
/// ```ignore
//  if let Err(e) = clean_ctx::analytics::bpe_or_init() {
//      eprintln!("[clean-ctx] {}", e);
//      std::process::exit(2);
//  }
// ```
pub fn bpe() -> &'static CoreBPE {
    BPE.get_or_init(|| {
        // `cl100k_base` is fallible (it loads BPE merge data from disk or
        // a baked-in source). Surfacing that as a `Result` from
        // `bpe_or_init` is the supported way to handle load failures at
        // startup. Falling through to `.expect` here is a defence-in-depth
        // measure for the (rare) case where `bpe()` is called without
        // going through startup init.
        tiktoken_rs::cl100k_base()
            .expect("cl100k BPE data must be loadable at startup")
    })
}

pub struct TokenMetadata {
    pub raw_tokens: usize,
    pub compressed_tokens: usize,
    pub savings_percentage: f64,
}

/// Measures exact local token counts completely offline using the tiktoken cl100k model.
///
/// Uses the cached BPE engine from [`bpe`]; see [`bpe_or_init`] for the
/// preferred startup-time initialisation that converts load failures into
/// a recoverable error.
pub fn calculate_savings(raw_text: &str, compressed_text: &str) -> TokenMetadata {
    let bpe = bpe();

    // Count exact token vector lengths
    let raw_tokens = bpe.encode_with_special_tokens(raw_text).len();
    let compressed_tokens = bpe.encode_with_special_tokens(compressed_text).len();

    let savings_percentage = if raw_tokens > 0 {
        let saved = raw_tokens.saturating_sub(compressed_tokens);
        (saved as f64 / raw_tokens as f64) * 100.0
    } else {
        0.0
    };

    TokenMetadata {
        raw_tokens,
        compressed_tokens,
        savings_percentage,
    }
}

#[cfg(test)]
#[path = "tests/analytics.rs"]
mod tests;
