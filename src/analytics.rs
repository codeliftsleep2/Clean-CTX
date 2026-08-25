// src/analytics.rs
//
// Token analytics: exact local token counts using pluggable tokenizers
// (R-19). The default is cl100k (the same model family used by GPT-4
// and many Claude context estimators), but callers can select o200k
// (GPT-4o), claude, or llama3 via tool arguments or config.
//
// Phase 1 (FAANG audit F-01): the BPE engine used to be re-loaded on every
// call via `tiktoken_rs::cl100k_base().unwrap()`. A failed BPE load would
// take the entire MCP server process down with a SIGABRT. The engine is
// now cached in a process-global `OnceLock` so it loads exactly once and
// a load failure surfaces as a recoverable `BpeInitError` at startup time
// rather than a mid-request panic.
//
// F-22 (FAANG audit): tiktoken-rs 0.11 embeds the cl100k BPE merge data
// directly in the binary via `include_bytes!`, so there is no filesystem
// dependency. The binary works correctly on read-only filesystems
// (e.g. Docker `--read-only`) and in sandboxed environments. The
// `bpe_or_init()` call at server startup serves as a defence-in-depth
// check that the embedded data is intact.
//
// R-19 (Pluggable tokenizers): `calculate_savings` now accepts an
// optional `&dyn Tokenizer` parameter. When `None`, the legacy
// cl100k BPE engine is used for backward compatibility.

use std::fmt;
use std::sync::OnceLock;

use tiktoken_rs::CoreBPE;

use crate::tokenizer::Tokenizer;

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
        tiktoken_rs::cl100k_base().expect("cl100k BPE data must be loadable at startup")
    })
}

pub struct TokenMetadata {
    pub raw_tokens: usize,
    pub compressed_tokens: usize,
    pub savings_percentage: f64,
}

/// Measures exact local token counts using the provided tokenizer.
///
/// When `tokenizer` is `None`, falls back to the legacy cl100k BPE
/// engine for backward compatibility.
///
/// R-19: The new `tokenizer` parameter allows callers to use o200k,
/// claude, or llama3 tokenizers for more accurate token counts.
pub fn calculate_savings(
    raw_text: &str,
    compressed_text: &str,
    tokenizer: Option<&dyn Tokenizer>,
) -> TokenMetadata {
    let (raw_tokens, compressed_tokens) = if let Some(tok) = tokenizer {
        (
            tok.count_tokens(raw_text),
            tok.count_tokens(compressed_text),
        )
    } else {
        let bpe = bpe();
        (
            bpe.encode_with_special_tokens(raw_text).len(),
            bpe.encode_with_special_tokens(compressed_text).len(),
        )
    };

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
