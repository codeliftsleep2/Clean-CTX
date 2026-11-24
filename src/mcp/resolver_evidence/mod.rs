//! Explicit opt-in public adapter for the existing E6-B operation, never v1 hydration.
mod budget;
pub(crate) mod schema;
mod state;
mod transport;
use crate::ir::source_observations::{
    claim_composition::execution::{self, Output, Query, Session},
    request_budget::{Cost, Limits, Request, Stop},
};
use crate::mcp::McpState;
use serde_json::{Value, json};
pub(crate) use state::State;
use std::{path::PathBuf, time::Instant};
#[derive(Debug)]
pub(crate) struct Error {
    pub code: i32,
    pub message: String,
    pub reason: String,
}
impl Error {
    fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
            reason: "invalid_request".into(),
        }
    }
    fn resource(message: impl Into<String>) -> Self {
        Self {
            code: -32004,
            message: message.into(),
            reason: "resource_exhausted".into(),
        }
    }
}
impl From<Stop> for Error {
    fn from(s: Stop) -> Self {
        Self::resource(format!("Request resource exhausted: {s:?}"))
    }
}
impl From<execution::Failure> for Error {
    fn from(e: execution::Failure) -> Self {
        match e {
            execution::Failure::Budget(s) => s.into(),
            other => Self::invalid(format!("{other:?}")),
        }
    }
}
pub(crate) fn is_v2(args: &Value) -> bool {
    args["contract_version"] == 2
        || args["queries"]
            .as_array()
            .is_some_and(|a| a.iter().any(|q| q["contract_version"] == 2))
}
pub(crate) fn eligible(args: &Value) -> bool {
    let version = |a: &Value| {
        a.get("contract_version")
            .is_none_or(|v| matches!(v.as_u64(), Some(1 | 2)))
    };
    is_v2(args)
        && version(args)
        && args["queries"].as_array().is_none_or(|a| {
            a.iter().all(|q| {
                q["contract_version"] == 2 && q["type"] == "resolver_registrations" && version(q)
            })
        })
}
pub(crate) fn handle(id: &Value, args: &Value, state: &McpState) {
    let started = Instant::now();
    let telemetry = crate::ir::source_observations::request_budget::telemetry::begin();
    let result = execute(id, args, state);
    match result {
        Ok(frame) => {
            crate::protocol::send_prepared_json(&frame.wire);
        }
        Err(e) => crate::protocol::send_response(
            &json!({"jsonrpc":"2.0","id":id,"error":{"code":e.code,"message":e.message,"data":{"reason":e.reason}}}),
        ),
    }
    state.resolver_evidence.retire(id);
    eprintln!(
        "[resolver-evidence-v2] counts={}",
        serde_json::json!({"id":id,"counts":telemetry.snapshot()})
    );
    eprintln!(
        "[resolver-evidence-v2] public_us={}",
        started.elapsed().as_micros()
    );
}
fn query(item: &Value, args: &Value) -> Query {
    let direction = item["direction"].as_str().expect("validated");
    Query {
        workspace_root: PathBuf::from(args["workspaceRoot"].as_str().expect("root")),
        contract_version: 2,
        mode: item["mode"].as_str().unwrap_or("interactive").into(),
        canonical_policy: item["canonical_policy"].as_str().unwrap_or("none").into(),
        direction: direction.into(),
        selector_basis: item["selector_basis"].as_str().expect("basis").into(),
        selector: item[if direction == "forward" {
            "route_path"
        } else {
            "resolver_name"
        }]
        .as_str()
        .expect("selector")
        .into(),
        source_files: item["source_files"].as_array().map(|a| {
            a.iter()
                .map(|v| PathBuf::from(v.as_str().expect("source")))
                .collect()
        }),
        within_path: args["withinPath"].as_str().map(str::to_owned),
    }
}
fn execute(id: &Value, args: &Value, state: &McpState) -> Result<transport::Frame, Error> {
    let batch = args.get("queries").is_some();
    if !schema::accepts(&schema::request(batch), args) {
        return Err(Error::invalid(
            "Request does not conform to the approved resolver evidence v2 schema",
        ));
    }
    if batch {
        let mut seen = std::collections::HashSet::new();
        for item in args["queries"].as_array().expect("batch") {
            if !seen.insert(item["id"].as_str().expect("ID")) {
                return Err(Error::invalid("Duplicate batch item ID"));
            }
        }
    }
    let requested = budget::Budget::parse(&args["budget"])?;
    let effective = requested.capped(state.resolver_evidence.required()?)?;
    let mut roots = vec![PathBuf::from(args["workspaceRoot"].as_str().expect("root"))];
    roots.extend(state.config.additional_roots.iter().map(PathBuf::from));
    let session = Session::new(roots, effective.limits()?)?;
    let request = session.ledger();
    state.resolver_evidence.bind(id, &request)?;
    let items = if batch {
        args["queries"]
            .as_array()
            .expect("batch")
            .iter()
            .collect::<Vec<_>>()
    } else {
        vec![args]
    };
    // Reserve the complete public envelope (both channels + escaping/framing)
    // before optional work. The rest remains one cumulative E6-B allowance.
    // Seven serialized-body copies is the JSON escaping upper envelope; this
    // is an implementation reservation formula, not a production budget.
    let framing = transport::framing_capacity(id, args, items.len())?;
    let capacity = effective
        .response_bytes
        .checked_sub(framing)
        .ok_or_else(|| {
            Error::resource("Minimum truthful public envelope exceeds response capacity")
        })?
        / 8;
    let wire_capacity = effective
        .response_bytes
        .checked_sub(capacity)
        .ok_or_else(|| Error::resource("Response capacity overflow"))?;
    let credit = request.response_capacity(wire_capacity)?;
    // Prepay copies and final wire storage before optional acquisition can use
    // remaining memory. All item DTO copies share this one finite envelope.
    let public_memory = capacity
        .checked_mul(32)
        .and_then(|n| n.checked_add(wire_capacity.checked_mul(4)?))
        .ok_or(Stop::RetainedBytes)?;
    let public_credit = request.reserve_nested(
        Cost {
            retained_bytes: public_memory,
            ..Cost::default()
        },
        true,
    )?;
    let ingress = serde_json::to_vec(args)
        .map_err(|e| Error::invalid(e.to_string()))?
        .len() as u64;
    let ingress_credit = request.reserve_nested(
        Cost {
            retained_bytes: ingress
                .checked_mul(16)
                .and_then(|v| v.checked_add(framing))
                .ok_or(Stop::RetainedBytes)?,
            ..Cost::default()
        },
        true,
    )?;
    let retained = vec![ingress_credit, public_credit];
    let mut values = Vec::new();
    let mut outputs: Vec<Output> = Vec::new();
    for item in items {
        let q = query(item, args);
        let result = session.execute_with_policy(q, &state.config);
        let outcome = match result {
            Ok(output) => {
                eprintln!(
                    "[resolver-evidence-v2] stages={}",
                    serde_json::json!({"id":id,"item":item.get("id"),"costs":output.measurements()})
                );
                let mut value = output.dto().value().clone();
                value["execution"]["requested_limits"] =
                    serde_json::to_value(&requested).expect("budget");
                value["execution"]["effective_limits"] =
                    serde_json::to_value(&effective).expect("budget");
                outputs.push(output);
                Ok(value)
            }
            Err(e) => Err(Error::from(e)),
        };
        if !batch {
            let dto = outcome?;
            return transport::prepare(id, dto, &request, credit, wire_capacity, retained, outputs);
        }
        values.push(match outcome{
            Ok(value)=>json!({"id":item["id"],"type":"resolver_registrations","contract_version":2,"status":"ok","result":value}),
            Err(e)=>json!({"id":item["id"],"type":"resolver_registrations","contract_version":2,"status":"error","error":{"code":e.code,"message":e.message,"reason":e.reason}}),
        });
    }
    let mut execution = execution::execution_snapshot(&request);
    execution["requested_limits"] = serde_json::to_value(requested).expect("budget");
    execution["effective_limits"] = serde_json::to_value(effective).expect("budget");
    let dto = json!({"schema":"clean-ctx/workspace-query-batch-answer","contract_version":2,"request_id":id.to_string(),"scope":{"workspace_root":args["workspaceRoot"],"additional_roots":state.config.additional_roots,"within_path":args.get("withinPath").cloned().unwrap_or(Value::Null)},"execution":execution,"results":values});
    transport::prepare(id, dto, &request, credit, wire_capacity, retained, outputs)
}

#[cfg(test)]
#[path = "../../tests/mcp/resolver_evidence_e6c.rs"]
mod tests;
