//! Pinned frontend protocol admission; replacement and query schemas are deferred.
use super::{CbmClient, CbmError};
use crate::cbm::config::CbmStatus;
use serde_json::{Value, json};
use std::time::Instant;

const PROTOCOL_VERSION: &str = "2024-11-05";
const REQUIRED_TOOLS: &[&str] = &[
    "index_repository",
    "search_graph",
    "query_graph",
    "trace_path",
    "get_architecture",
    "list_projects",
    "detect_changes",
];

impl CbmClient {
    pub(super) fn ensure_usable(&self) -> Result<(), CbmError> {
        if self.frontend.is_none() {
            return Err(CbmError::ConnectionLost("frontend retired".into()));
        }
        if !self.usable {
            return Err(CbmError::AdmissionError {
                code: None,
                message: "frontend has not completed admission".into(),
            });
        }
        Ok(())
    }

    /// One admission attempt on the existing owned transport; never relaunches.
    pub(crate) fn admit(&mut self) -> Result<(), CbmError> {
        if self.frontend.is_none() {
            return Err(CbmError::ConnectionLost("frontend retired".into()));
        }
        if self.usable {
            return self.ensure_frontend();
        }
        self.status = CbmStatus::Degraded("admitting".into());
        let deadline = Instant::now() + self.timeout;
        let result = self.admit_inner(deadline);
        match result {
            Ok(()) => {
                self.usable = true;
                self.admitted_once = true;
                self.status = CbmStatus::Available;
                Ok(())
            }
            Err(error) => {
                self.retire_frontend(&format!("admission failed: {error}"));
                self.status = CbmStatus::Degraded(format!(
                    "admission failed: {error}; transport: {:?}",
                    self.status
                ));
                Err(error)
            }
        }
    }

    fn admit_inner(&mut self, deadline: Instant) -> Result<(), CbmError> {
        let response = self.admission_request(
            "initialize",
            json!({
                "protocolVersion": PROTOCOL_VERSION, "capabilities": {},
                "clientInfo": {"name":"clean-ctx", "version":env!("CARGO_PKG_VERSION")}
            }),
            deadline,
        )?;
        validate_initialize(&response)?;
        self.send_initialized(deadline)?;
        // Pinned no-cursor tools/list returns the complete registry. No query
        // paging, cursor collection or optional-tool requirements are added.
        let tools = self.admission_request("tools/list", json!({}), deadline)?;
        validate_tools(&tools)?;
        if Instant::now() >= deadline {
            return Err(CbmError::Timeout(self.timeout));
        }
        self.ensure_frontend_with_deadline(deadline)?;
        Ok(())
    }

    /// A terminal startup can still have a queued provider refusal. Consume it
    /// through the same A1 classifier before A2 releases the owned receiver.
    pub(super) fn terminal_startup_error(&self, deadline: Instant) -> CbmError {
        let Some(receiver) = self
            .frontend
            .as_ref()
            .and_then(|frontend| frontend.stdout_rx.as_ref())
        else {
            return CbmError::ConnectionLost("frontend retired".into());
        };
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return CbmError::Timeout(self.timeout);
            }
            match receiver.recv_timeout(remaining) {
                Ok(Ok(frame)) => match super::framing::is_response(frame.trim(), 0) {
                    Ok(false) => continue,
                    Ok(true) => {
                        return CbmError::ParseError(
                            "unsolicited response before initialization".into(),
                        );
                    }
                    Err(error) => return error,
                },
                Ok(Err(error)) if error.kind() == std::io::ErrorKind::InvalidData => {
                    return CbmError::ParseError(error.to_string());
                }
                Ok(Err(error)) => return CbmError::ConnectionLost(error.to_string()),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    return CbmError::Timeout(self.timeout);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return CbmError::ConnectionLost("frontend exited before admission".into());
                }
            }
        }
    }

    fn admission_request(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
    ) -> Result<Value, CbmError> {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(CbmError::Timeout(self.timeout));
        }
        let raw = self.send_rpc_request(method, params, remaining)?;
        // send_rpc_request already performs bounded A1 validation/correlation.
        let envelope: Value = serde_json::from_str(raw.trim())
            .map_err(|error| CbmError::ParseError(error.to_string()))?;
        if let Some(error) = envelope.get("error") {
            return Err(CbmError::RpcError {
                code: error["code"].as_i64().unwrap_or(-1),
                message: error["message"].as_str().unwrap_or_default().into(),
            });
        }
        envelope
            .get("result")
            .cloned()
            .ok_or_else(|| CbmError::ParseError("admission result absent".into()))
    }
}

fn validate_initialize(result: &Value) -> Result<(), CbmError> {
    if result["protocolVersion"].as_str() != Some(PROTOCOL_VERSION) {
        return Err(CbmError::CapabilityError(
            "unsupported negotiated protocol version".into(),
        ));
    }
    if result["serverInfo"]["name"].as_str() != Some("codebase-memory-mcp")
        || result["serverInfo"]["version"]
            .as_str()
            .is_none_or(str::is_empty)
        || !result["capabilities"]["tools"].is_object()
    {
        return Err(CbmError::CapabilityError(
            "invalid server identity or missing tools capability".into(),
        ));
    }
    Ok(())
}

fn validate_tools(result: &Value) -> Result<(), CbmError> {
    let tools = result["tools"].as_array().ok_or_else(|| {
        CbmError::CapabilityError("tools/list did not return a tools array".into())
    })?;
    if result
        .get("nextCursor")
        .is_some_and(|cursor| !cursor.is_null())
    {
        return Err(CbmError::CapabilityError(
            "unpaginated tools/list returned an incomplete registry".into(),
        ));
    }
    for required in REQUIRED_TOOLS {
        let mut matching = tools
            .iter()
            .filter(|tool| tool["name"].as_str() == Some(required));
        let tool = matching.next().ok_or_else(|| {
            CbmError::CapabilityError(format!("required tool missing: {required}"))
        })?;
        if matching.next().is_some() || tool["inputSchema"]["type"].as_str() != Some("object") {
            return Err(CbmError::CapabilityError(format!(
                "required tool definition invalid: {required}"
            )));
        }
    }
    Ok(())
}
