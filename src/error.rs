// src/error.rs
//
// Structured error types for Clean-CTX.
//
// H-05 (FAANG audit): Previously errors propagated as `Box<dyn Error>`
// or raw `String`, making it impossible to distinguish transient
// (retryable) failures from permanent ones. This module provides a
// unified `CleanCtxError` enum with:
//
//   - `is_retryable()` — whether the operation can be retried
//   - `status_code()` — JSON-RPC error code for MCP responses
//   - `From` impls for common error types (std::io::Error, etc.)
//
// All subsystems should convert their errors into `CleanCtxError`
// at their public boundary.

use std::fmt;

/// Unified error type for all Clean-CTX subsystems.
///
/// Each variant carries a human-readable message and a flag indicating
/// whether the operation is retryable. The `status_code()` method
/// returns the appropriate JSON-RPC error code for MCP responses.
#[derive(Debug, Clone)]
pub enum CleanCtxError {
    /// I/O errors (file not found, permission denied, etc.).
    /// Typically permanent — retrying won't help.
    Io(String),
    /// CBM graph bridge errors (binary not found, timeout, etc.).
    /// May be transient (timeout) or permanent (binary missing).
    Cbm { message: String, retryable: bool },
    /// Compression pipeline errors (parse failure, invalid fidelity, etc.).
    /// Typically permanent — the input is malformed.
    Compression(String),
    /// IR compilation errors (unsupported language, parse failure, etc.).
    /// Typically permanent.
    Ir(String),
    /// Persistence/SQLite errors (DB corruption, disk full, etc.).
    /// May be transient (disk full → free space) or permanent.
    Persistence { message: String, retryable: bool },
    /// Configuration errors (invalid JSON, missing fields, etc.).
    /// Permanent — fix the config file.
    Config(String),
    /// Internal/unknown errors that don't fit other categories.
    Internal(String),
}

impl CleanCtxError {
    /// Whether the operation can be retried.
    ///
    /// Returns `true` for transient failures (timeouts, temporary
    /// resource exhaustion) and `false` for permanent failures
    /// (invalid input, missing binary, config errors).
    pub fn is_retryable(&self) -> bool {
        match self {
            CleanCtxError::Io(_) => false,
            CleanCtxError::Cbm { retryable, .. } => *retryable,
            CleanCtxError::Compression(_) => false,
            CleanCtxError::Ir(_) => false,
            CleanCtxError::Persistence { retryable, .. } => *retryable,
            CleanCtxError::Config(_) => false,
            CleanCtxError::Internal(_) => false,
        }
    }

    /// JSON-RPC error code for MCP responses.
    ///
    ///   - -32603: Internal error (generic, non-retryable)
    ///   - -32602: Invalid params (config, compression, IR)
    ///   - -32000: Server error (retryable — CBM timeout, DB busy)
    ///   - -32001: Service unavailable (CBM not found, DB closed)
    pub fn status_code(&self) -> i64 {
        match self {
            CleanCtxError::Io(_) => -32603,
            CleanCtxError::Cbm { retryable, .. } => {
                if *retryable {
                    -32000
                } else {
                    -32001
                }
            }
            CleanCtxError::Compression(_) => -32602,
            CleanCtxError::Ir(_) => -32602,
            CleanCtxError::Persistence { retryable, .. } => {
                if *retryable {
                    -32000
                } else {
                    -32603
                }
            }
            CleanCtxError::Config(_) => -32602,
            CleanCtxError::Internal(_) => -32603,
        }
    }

    /// Create a retryable CBM error.
    pub fn cbm_retryable(msg: impl Into<String>) -> Self {
        CleanCtxError::Cbm {
            message: msg.into(),
            retryable: true,
        }
    }

    /// Create a permanent CBM error.
    pub fn cbm_permanent(msg: impl Into<String>) -> Self {
        CleanCtxError::Cbm {
            message: msg.into(),
            retryable: false,
        }
    }

    /// Create a retryable persistence error.
    pub fn persistence_retryable(msg: impl Into<String>) -> Self {
        CleanCtxError::Persistence {
            message: msg.into(),
            retryable: true,
        }
    }

    /// Create a permanent persistence error.
    pub fn persistence_permanent(msg: impl Into<String>) -> Self {
        CleanCtxError::Persistence {
            message: msg.into(),
            retryable: false,
        }
    }
}

impl fmt::Display for CleanCtxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CleanCtxError::Io(msg) => write!(f, "IO error: {msg}"),
            CleanCtxError::Cbm { message, retryable } => {
                if *retryable {
                    write!(f, "CBM error (retryable): {message}")
                } else {
                    write!(f, "CBM error: {message}")
                }
            }
            CleanCtxError::Compression(msg) => write!(f, "Compression error: {msg}"),
            CleanCtxError::Ir(msg) => write!(f, "IR error: {msg}"),
            CleanCtxError::Persistence { message, retryable } => {
                if *retryable {
                    write!(f, "Persistence error (retryable): {message}")
                } else {
                    write!(f, "Persistence error: {message}")
                }
            }
            CleanCtxError::Config(msg) => write!(f, "Config error: {msg}"),
            CleanCtxError::Internal(msg) => write!(f, "Internal error: {msg}"),
        }
    }
}

impl std::error::Error for CleanCtxError {}

impl From<std::io::Error> for CleanCtxError {
    fn from(e: std::io::Error) -> Self {
        CleanCtxError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for CleanCtxError {
    fn from(e: serde_json::Error) -> Self {
        CleanCtxError::Internal(format!("JSON serialization error: {e}"))
    }
}

impl From<crate::ir::compiler::CompileError> for CleanCtxError {
    fn from(e: crate::ir::compiler::CompileError) -> Self {
        CleanCtxError::Ir(e.to_string())
    }
}

/// Convert a `CleanCtxError` into a JSON-RPC error response Value.
/// Convenience for MCP tool handlers.
pub fn to_jsonrpc_error(id: &serde_json::Value, error: &CleanCtxError) -> serde_json::Value {
    crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        error.status_code(),
        error.to_string(),
        Some(serde_json::json!({ "retryable": error.is_retryable() })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_is_not_retryable() {
        let err = CleanCtxError::Io("file not found".into());
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32603);
    }

    #[test]
    fn test_cbm_retryable() {
        let err = CleanCtxError::cbm_retryable("timeout");
        assert!(err.is_retryable());
        assert_eq!(err.status_code(), -32000);
        assert!(err.to_string().contains("retryable"));
    }

    #[test]
    fn test_cbm_permanent() {
        let err = CleanCtxError::cbm_permanent("binary not found");
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32001);
    }

    #[test]
    fn test_compression_error() {
        let err = CleanCtxError::Compression("invalid fidelity".into());
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32602);
    }

    #[test]
    fn test_ir_error() {
        let err = CleanCtxError::Ir("unsupported language".into());
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32602);
    }

    #[test]
    fn test_persistence_retryable() {
        let err = CleanCtxError::persistence_retryable("disk full");
        assert!(err.is_retryable());
        assert_eq!(err.status_code(), -32000);
    }

    #[test]
    fn test_persistence_permanent() {
        let err = CleanCtxError::persistence_permanent("corrupt DB");
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32603);
    }

    #[test]
    fn test_config_error() {
        let err = CleanCtxError::Config("invalid JSON".into());
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32602);
    }

    #[test]
    fn test_internal_error() {
        let err = CleanCtxError::Internal("unexpected null".into());
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), -32603);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err: CleanCtxError = io_err.into();
        assert!(matches!(err, CleanCtxError::Io(_)));
        assert!(!err.is_retryable());
    }

    #[test]
    fn test_to_jsonrpc_error() {
        let id = serde_json::json!(1);
        let err = CleanCtxError::cbm_retryable("timeout");
        let json = to_jsonrpc_error(&id, &err);
        assert_eq!(json["id"], 1);
        assert_eq!(json["error"]["code"], -32000);
        assert_eq!(json["error"]["data"]["retryable"], true);
    }

    #[test]
    fn test_display_io() {
        let err = CleanCtxError::Io("permission denied".into());
        assert_eq!(err.to_string(), "IO error: permission denied");
    }

    #[test]
    fn test_display_compression() {
        let err = CleanCtxError::Compression("parse failed".into());
        assert_eq!(err.to_string(), "Compression error: parse failed");
    }
}
