//! `provide_code_context` orchestration.
//!
//! One typed evaluator owns the complete single-file lifecycle. This module
//! projects that outcome into the legacy MCP response and transmits exactly
//! once; batch orchestration can reuse the evaluator without recursively
//! invoking a response-sending handler.

use crate::mcp::McpState;
use crate::mcp::tool_helpers::inject_baseline_breakpoint;
use crate::protocol::send_response;
use serde_json::Value;

mod batch;
mod evaluate;
pub(super) mod outcome;

pub(crate) fn handle_provide_code_context(id: &Value, params: &Value, state: &McpState) {
    if params["arguments"].get("files").is_some() {
        send_response(&batch::response(id, &params["arguments"], state));
        return;
    }
    if params["arguments"].get("responseMode").is_some() {
        send_response(
            &outcome::ProvideFailure::invalid(
                "'responseMode' is valid only with the batched 'files' request form.",
            )
            .response(id),
        );
        return;
    }
    let response = match evaluate::evaluate(params, state) {
        Ok(context) => {
            let mut response = context.legacy_response(id);
            inject_baseline_breakpoint(&mut response, state, &context.text);
            response
        }
        Err(error) => error.response(id),
    };
    send_response(&response);
}
