//! Existing frame acceptance/reading isolated for focused Phase A regressions.

use super::CbmError;
use serde_json::Value;
use std::io::{self, BufRead};

pub(super) fn is_response(raw: &str, _pending: u64) -> Result<bool, CbmError> {
    serde_json::from_str::<Value>(raw)
        .map(|_| true)
        .map_err(|error| CbmError::ParseError(error.to_string()))
}

pub(super) fn read_frame<R: BufRead>(reader: &mut R, _limit: usize) -> io::Result<Option<String>> {
    let mut line = String::new();
    match reader.read_line(&mut line)? {
        0 => Ok(None),
        _ => Ok(Some(line)),
    }
}

#[cfg(test)]
#[path = "../../tests/cbm/frontend_framing.rs"]
mod tests;
