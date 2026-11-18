//! Correlated JSON-RPC envelopes and bounded newline frames for the CBM frontend.

use super::CbmError;
use serde_json::Value;
use std::io::{self, BufRead};

pub(super) fn is_response(raw: &str, pending: u64) -> Result<bool, CbmError> {
    let value: Value =
        serde_json::from_str(raw).map_err(|error| CbmError::ParseError(error.to_string()))?;
    let object = value
        .as_object()
        .ok_or_else(|| invalid("expected an object"))?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(invalid("expected JSON-RPC 2.0"));
    }
    if let Some(method) = object.get("method") {
        if method.as_str().is_none_or(str::is_empty)
            || object.contains_key("id")
            || object.contains_key("result")
            || object.contains_key("error")
            || object
                .get("params")
                .is_some_and(|params| !params.is_object() && !params.is_array())
        {
            return Err(invalid(
                "unsupported server request or malformed notification",
            ));
        }
        eprintln!("[clean-ctx-cbm] Ignoring frontend notification while awaiting response");
        return Ok(false);
    }
    let result = object.contains_key("result");
    let error = object.get("error");
    if result == error.is_some() {
        return Err(invalid("expected exactly one of result or error"));
    }
    if let Some(error) = error {
        if error.get("code").and_then(Value::as_i64).is_none()
            || error.get("message").and_then(Value::as_str).is_none()
        {
            return Err(invalid("malformed RPC error"));
        }
        if object.get("id").is_some_and(Value::is_null) {
            if error["code"].as_i64() == Some(-32001) {
                return Err(CbmError::AdmissionError {
                    code: Some(-32001),
                    message: error["message"].as_str().unwrap_or_default().into(),
                });
            }
            return Err(invalid(&format!(
                "uncorrelated startup/lifecycle error: {}",
                error["message"].as_str().unwrap_or_default()
            )));
        }
    }
    if object.get("id").and_then(Value::as_u64) != Some(pending) {
        return Err(invalid("response ID does not match the pending request"));
    }
    Ok(true)
}

fn invalid(message: &str) -> CbmError {
    CbmError::ParseError(format!("CBM protocol: {message}"))
}

pub(super) fn read_frame<R: BufRead>(reader: &mut R, limit: usize) -> io::Result<Option<String>> {
    let mut frame = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return if frame.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "unterminated CBM frame",
                ))
            };
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let count = newline.map_or(available.len(), |position| position + 1);
        if count > limit.saturating_sub(frame.len()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "CBM frame exceeds receive limit",
            ));
        }
        frame.extend_from_slice(&available[..count]);
        reader.consume(count);
        if newline.is_some() {
            return String::from_utf8(frame)
                .map(Some)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
        }
    }
}

#[cfg(test)]
#[path = "../../tests/cbm/frontend_framing.rs"]
mod tests;
