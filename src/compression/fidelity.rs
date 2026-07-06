// src/compression/fidelity.rs
//
// The `Fidelity` enum and its string parser. Previously this lived inline
// in `compressor.rs`; both `compressor.rs` and `diff/builder.rs` depend on
// it, so it now lives in the shared `compression` namespace.
//
// Phase 1 (FAANG audit F-03): `Fidelity::parse` used to silently map
// typos like `"hihg"`, `""`, `"🚀"` to `Fidelity::Low`. The user had
// no idea they got the wrong compression. The parser now returns a
// `Result<Self, FidelityParseError>` so callers can decide between
// hard-fail (the MCP tool path) and soft-fallback (e.g. config defaults).
// A `parse_or_default` helper preserves the old behaviour for callers
// that explicitly opt into it.

use std::fmt;

/// Compression fidelity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fidelity {
    /// Maximum compression — strips keywords, async, fields, errors (current default)
    Low,
    /// Balanced — preserves async, field types, errors, control flow markers
    Medium,
    /// Minimal compression — preserves as much semantic depth as possible
    High,
}

impl serde::Serialize for Fidelity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(match self {
            Fidelity::Low => "low",
            Fidelity::Medium => "medium",
            Fidelity::High => "high",
        })
    }
}

impl<'de> serde::Deserialize<'de> for Fidelity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Fidelity::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Returned by [`Fidelity::parse`] when the input is not one of the
/// recognised fidelity strings. The offending value is preserved in
/// [`FidelityParseError::0`] for diagnostic purposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidelityParseError(pub String);

impl fmt::Display for FidelityParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "unknown fidelity '{}' (expected 'low', 'medium', or 'high')",
            self.0
        )
    }
}

impl std::error::Error for FidelityParseError {}

impl Fidelity {
    /// Parse a fidelity string. Case-insensitive. Returns
    /// `FidelityParseError` for unrecognised values so callers can
    /// decide between hard-fail (e.g. JSON-RPC `-32602 Invalid params`)
    /// and silent fallback.
    pub fn parse(s: &str) -> Result<Self, FidelityParseError> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Fidelity::Low),
            "medium" => Ok(Fidelity::Medium),
            "high" => Ok(Fidelity::High),
            other => Err(FidelityParseError(other.to_string())),
        }
    }

    /// Back-compat default: parse and fall back to [`Fidelity::Low`] on
    /// an unrecognised value, emitting a warning to stderr so the
    /// operator at least sees what happened. Used by callers that
    /// explicitly opt into the lenient behaviour (e.g. config file
    /// loaders). The MCP `tools/call` entry points MUST NOT use this
    /// — they should return `-32602` instead.
    pub fn parse_or_default(s: &str) -> Self {
        Self::parse(s).unwrap_or_else(|err| {
            eprintln!(
                "[clean-ctx] Warning: {} — defaulting to 'low'",
                err
            );
            Fidelity::Low
        })
    }
}

#[cfg(test)]
#[path = "../tests/compression/fidelity.rs"]
mod tests;
