//! Closed native diagnostic operation; producer evidence never enters protocol errors.
use crate::diagnostics::cargo_check::*;
use crate::protocol::send_response;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Mutex;

pub(crate) struct CargoCheckSession {
    admission: Result<CargoCheckInvocation, &'static str>,
    lifecycle: Mutex<Lifecycle>,
}

#[derive(Default)]
struct Lifecycle {
    shutting_down: bool,
    active: HashMap<String, (CargoCheckCancellation, bool)>,
}

impl Default for CargoCheckSession {
    fn default() -> Self {
        Self::new(Err("workspace_authority_missing"))
    }
}

impl CargoCheckSession {
    pub(crate) fn new(admission: Result<CargoCheckInvocation, &'static str>) -> Self {
        let admission = admission.and_then(|invocation| {
            if cfg!(any(target_os = "linux", windows)) {
                Ok(invocation)
            } else {
                Err("ownership_platform_unsupported")
            }
        });
        Self {
            admission,
            lifecycle: Mutex::new(Lifecycle::default()),
        }
    }

    pub(crate) fn admission_failure(&self) -> Option<&'static str> {
        self.admission.as_ref().err().copied()
    }

    pub(crate) fn cancel(&self, id: &Value) {
        let lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((token, _)) = lifecycle.active.get(&id.to_string()) {
            token.request(CancellationSource::Host);
        }
    }

    pub(crate) fn shutdown(&self) {
        let mut lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        lifecycle.shutting_down = true;
        for (token, _) in lifecycle.active.values() {
            token.request(CancellationSource::Host);
        }
    }

    // Reserve before enqueueing, so cancellation cannot race a queued request.
    pub(crate) fn reserve(&self, id: &Value) -> bool {
        let mut lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        let key = id.to_string();
        if lifecycle.active.contains_key(&key) {
            return false;
        }
        let token = CargoCheckCancellation::default();
        if lifecycle.shutting_down {
            token.request(CancellationSource::Host);
        }
        lifecycle.active.insert(key, (token, false));
        true
    }

    pub(crate) fn abandon(&self, id: &Value) {
        let mut lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        let key = id.to_string();
        if lifecycle
            .active
            .get(&key)
            .is_some_and(|(_, started)| !started)
        {
            lifecycle.active.remove(&key);
        }
    }

    fn begin(&self, id: &Value) -> Result<ActiveRequest<'_>, ()> {
        let mut lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        let key = id.to_string();
        let token = if let Some((token, started)) = lifecycle.active.get_mut(&key) {
            if *started {
                return Err(());
            }
            *started = true;
            token.clone()
        } else {
            let token = CargoCheckCancellation::default();
            if lifecycle.shutting_down {
                token.request(CancellationSource::Host);
            }
            lifecycle.active.insert(key.clone(), (token.clone(), true));
            token
        };
        Ok(ActiveRequest {
            session: self,
            key,
            token,
        })
    }

    fn call(&self, id: &Value) -> Value {
        let active = match self.begin(id) {
            Ok(active) => active,
            Err(()) => return failure_result("request_already_active", None, None),
        };
        let invocation = match &self.admission {
            Ok(invocation) => invocation,
            Err(failure) => return failure_result(failure, None, None),
        };
        let execution = match execute_cargo_check(invocation, &active.token) {
            Ok(execution) => execution,
            Err(error) => {
                return match error {
                    ExecutionError::Authority(_) => {
                        failure_result("authority_changed_before_start", None, None)
                    }
                    ExecutionError::Start { .. } => {
                        failure_result("owned_execution_start_failed", None, None)
                    }
                    ExecutionError::UnsupportedPlatform => {
                        failure_result("ownership_platform_unsupported", None, None)
                    }
                    ExecutionError::CancelledBeforeStart {
                        cancellation_source,
                    } => failure_result("cancelled_before_start", Some(cancellation_source), None),
                };
            }
        };
        let is_error = cli_disposition(&execution, false) != CliDisposition::Success;
        match project_cargo_check(&execution, CargoCheckPolicy::APPROVED) {
            Ok(projection) => match serde_json::from_slice::<Value>(projection.structured_json()) {
                Ok(structured) => json!({
                    "content":[{"type":"text","text":projection.content()}],
                    "structuredContent":structured,
                    "isError":is_error,
                }),
                Err(_) => failure_result("projection_serialization_failed", None, Some(&execution)),
            },
            Err(_) => failure_result("projection_budget_failed", None, Some(&execution)),
        }
    }
}

struct ActiveRequest<'a> {
    session: &'a CargoCheckSession,
    key: String,
    token: CargoCheckCancellation,
}

impl Drop for ActiveRequest<'_> {
    fn drop(&mut self) {
        self.session
            .lifecycle
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .active
            .remove(&self.key);
    }
}

fn failure_result(
    failure: &'static str,
    cancellation: Option<CancellationSource>,
    execution: Option<&CargoCheckExecution>,
) -> Value {
    json!({
        "content":[{"type":"text","text":format!("CargoCheck unavailable or failed: {failure}")}],
        "structuredContent":{
            "failure":failure,
            "cancellation_source":cancellation,
            "root_outcome":execution.and_then(|e| e.root_outcome),
            "cleanup":execution.map(|e| &e.cleanup),
            "capture":execution.map(|e| &e.capture),
            "parser_coverage":execution.map(|e| &e.semantic.parser_coverage),
            "cargo_evidence":execution.map(|e| &e.semantic.cargo_evidence),
        },
        "isError":true,
    })
}

pub(crate) fn definition() -> Value {
    json!({
        "name":"cargo_check",
        "description":"Run exactly cargo check --message-format=json in the operator-approved workspace using the operator-approved absolute Cargo executable. Returns bounded sanitized compiler diagnostics and authoritative process, capture, coverage, and cleanup facts. Takes no arguments. May execute build scripts and access dependency networks. Missing startup authority makes this tool unavailable.",
        "inputSchema":{"type":"object","properties":{},"additionalProperties":false},
    })
}

pub(crate) fn handle(id: &Value, params: &Value, state: &super::McpState) {
    if params.get("arguments").is_some_and(|arguments| {
        !arguments
            .as_object()
            .is_some_and(|object| object.is_empty())
    }) {
        state.cargo_check.abandon(id);
        send_response(&json!({"jsonrpc":"2.0","id":id,"error":{
            "code":-32602,"message":"cargo_check accepts only an empty argument object"
        }}));
        return;
    }
    send_response(&json!({"jsonrpc":"2.0","id":id,"result":state.cargo_check.call(id)}));
}

#[cfg(test)]
#[path = "../tests/mcp/cargo_check_mcp.rs"]
mod tests;
