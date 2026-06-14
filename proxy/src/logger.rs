// proxy/src/logger.rs
//
// Request/response body logging for debugging and verification.
//
// When LOG_BODIES=1, writes:
//   {LOG_DIR}/{reqId}.req.json     — post-mutation request JSON (auth redacted)
//   {LOG_DIR}/{reqId}.resp.log     — raw response bytes

use serde_json::Value;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tracing::info;

/// Statistics tracked across log operations.
#[derive(Debug, Clone, Default)]
pub struct LogStats {
    /// Number of request bodies logged.
    pub requests_logged: u64,

    /// Number of response bodies logged.
    pub responses_logged: u64,

    /// Total bytes written to log files.
    pub bytes_written: u64,
}

/// Sanitize sensitive fields from a request JSON for safe logging.
///
/// Redacts known API key field names if present in the JSON body.
/// Note: Anthropic API keys are typically sent via the `x-api-key` HTTP header,
/// not in the JSON body. This is defense-in-depth.
pub fn sanitize_request(body: &Value) -> Value {
    let mut sanitized = body.clone();

    // Redact API key if present as a top-level field
    if let Some(obj) = sanitized.as_object_mut() {
        for key in &["x_api_key", "x-api-key", "api_key", "anthropic_api_key", "authorization"] {
            if obj.contains_key(*key) {
                obj.insert(key.to_string(), Value::String("[REDACTED]".to_string()));
            }
        }
    }

    sanitized
}

/// Write a request JSON body to a log file.
pub async fn log_request(
    log_dir: &PathBuf,
    req_id: &str,
    body: &Value,
    stats: &mut LogStats,
) -> std::io::Result<()> {
    // Ensure log directory exists
    tokio::fs::create_dir_all(log_dir).await?;

    let sanitized = sanitize_request(body);
    let json_str = serde_json::to_string_pretty(&sanitized)
        .unwrap_or_else(|_| "{}".to_string());

    let file_path = log_dir.join(format!("{req_id}.req.json"));
    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(json_str.as_bytes()).await?;

    stats.requests_logged += 1;
    stats.bytes_written += json_str.len() as u64;

    info!("[log] Request body written to {:?}", file_path);
    Ok(())
}

/// Write a response body to a log file.
pub async fn log_response(
    log_dir: &PathBuf,
    req_id: &str,
    body: &[u8],
    stats: &mut LogStats,
) -> std::io::Result<()> {
    // Ensure log directory exists
    tokio::fs::create_dir_all(log_dir).await?;

    let file_path = log_dir.join(format!("{req_id}.resp.log"));
    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(body).await?;

    stats.responses_logged += 1;
    stats.bytes_written += body.len() as u64;

    info!("[log] Response body written to {:?}", file_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_redacts_api_key() {
        let body = json!({
            "model": "claude-sonnet-4-20250514",
            "x_api_key": "sk-ant-abc123",
            "messages": []
        });

        let sanitized = sanitize_request(&body);
        assert_eq!(sanitized["x_api_key"], "[REDACTED]");
        assert_eq!(sanitized["model"], "claude-sonnet-4-20250514");
    }

    #[test]
    fn test_sanitize_preserves_unchanged() {
        let body = json!({
            "model": "claude-sonnet-4-20250514",
            "messages": [{"role": "user", "content": "Hi"}]
        });

        let sanitized = sanitize_request(&body);
        assert_eq!(sanitized, body);
    }
}