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

#[cfg(test)]
mod publication_race_test_support {
    use std::sync::{Condvar, Mutex};

    #[derive(Default)]
    struct PauseState {
        path: Option<String>,
        paused: bool,
        released: bool,
    }

    static STATE: Mutex<PauseState> = Mutex::new(PauseState {
        path: None,
        paused: false,
        released: false,
    });
    static SIGNAL: Condvar = Condvar::new();

    pub(super) fn arm(path: &str) {
        let mut state = STATE.lock().expect("publication pause state");
        *state = PauseState {
            path: Some(crate::dictionary::path::canonical_identity_key(path)),
            paused: false,
            released: false,
        };
    }

    pub(super) fn pause_after_compile(path: &str) {
        let canonical = crate::dictionary::path::canonical_identity_key(path);
        let mut state = STATE.lock().expect("publication pause state");
        if state.path.as_deref() != Some(canonical.as_str()) || state.paused {
            return;
        }
        state.paused = true;
        SIGNAL.notify_all();
        while !state.released {
            state = SIGNAL.wait(state).expect("publication pause wait");
        }
        state.path = None;
    }

    pub(super) fn wait_until_paused() {
        let mut state = STATE.lock().expect("publication pause state");
        while !state.paused {
            state = SIGNAL.wait(state).expect("publication pause wait");
        }
    }

    pub(super) fn release() {
        let mut state = STATE.lock().expect("publication pause state");
        state.released = true;
        SIGNAL.notify_all();
    }
}

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

#[cfg(test)]
#[path = "../../../tests/mcp/provide_publication_race.rs"]
mod publication_race_tests;
