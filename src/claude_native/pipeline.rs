use super::{
    bash,
    facts::{ClaudeNativeFacts, FieldFacts, PassThroughReason, ProcessingOutcome},
};
use crate::native_text::{
    ansi::normalize_terminal_text,
    git_diff_filter::{command_selects_git_diff, filter_git_diff},
    redaction::redact_recognized_secrets,
};
use serde_json::{Value, json};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    time::Instant,
};

pub(super) fn process(event: &Value) -> (Option<Value>, ClaudeNativeFacts) {
    process_inner(event, PipelineFault::None)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PipelineFault {
    None,
    #[cfg(test)]
    Transform,
    #[cfg(test)]
    Reconstruction,
    #[cfg(test)]
    Validation,
}

fn process_inner(event: &Value, fault: PipelineFault) -> (Option<Value>, ClaudeNativeFacts) {
    #[cfg(not(test))]
    let _ = fault;
    let started = Instant::now();
    let reason = classify_unsupported(event);
    let Some(bash) = bash::recognize(event) else {
        return (
            None,
            ClaudeNativeFacts::passed(reason, started.elapsed().as_micros()),
        );
    };
    #[cfg(test)]
    if fault == PipelineFault::Transform {
        return (
            None,
            ClaudeNativeFacts::passed(
                PassThroughReason::TransformError,
                started.elapsed().as_micros(),
            ),
        );
    }
    let transformed = catch_unwind(AssertUnwindSafe(|| transform(&bash)));
    let Ok((stdout, stderr, fields)) = transformed else {
        return (
            None,
            ClaudeNativeFacts::passed(
                PassThroughReason::TransformError,
                started.elapsed().as_micros(),
            ),
        );
    };
    if stdout == bash.stdout && stderr == bash.stderr {
        return (
            None,
            ClaudeNativeFacts::passed(PassThroughReason::Unchanged, started.elapsed().as_micros()),
        );
    }
    let mut rebuilt = bash.response.clone();
    rebuilt.insert("stdout".into(), Value::String(stdout));
    rebuilt.insert("stderr".into(), Value::String(stderr));
    #[cfg(test)]
    if fault == PipelineFault::Reconstruction {
        rebuilt.remove("interrupted");
        return (
            None,
            ClaudeNativeFacts::passed(
                PassThroughReason::ReconstructionFailed,
                started.elapsed().as_micros(),
            ),
        );
    }
    #[cfg(test)]
    if fault == PipelineFault::Validation {
        rebuilt.insert("isImage".into(), Value::Bool(true));
    }
    if !bash::validate_reconstruction(bash.response, &rebuilt) {
        return (
            None,
            ClaudeNativeFacts::passed(
                PassThroughReason::ValidationFailed,
                started.elapsed().as_micros(),
            ),
        );
    }
    let response = json!({"hookSpecificOutput": {
        "hookEventName": "PostToolUse",
        "updatedToolOutput": Value::Object(rebuilt)
    }});
    let facts = ClaudeNativeFacts {
        outcome: ProcessingOutcome::Replaced,
        tool: Some("Bash"),
        adapter_id: "claude-post-tool-use",
        adapter_version: 1,
        schema_id: Some(bash::SCHEMA_ID),
        validation_succeeded: true,
        pass_through_reason: None,
        fields,
        duration_micros: started.elapsed().as_micros(),
    };
    (Some(response), facts)
}

#[cfg(test)]
fn process_with_fault(event: &Value, fault: PipelineFault) -> (Option<Value>, ClaudeNativeFacts) {
    process_inner(event, fault)
}

fn transform(bash: &bash::BashSuccess<'_>) -> (String, String, Vec<FieldFacts>) {
    let stdout_normalized = normalize_terminal_text(bash.stdout);
    let stdout_redacted = redact_recognized_secrets(&stdout_normalized.text);
    let filtered =
        command_selects_git_diff(bash.command).then(|| filter_git_diff(&stdout_redacted.text));
    let stdout = filtered
        .as_ref()
        .map_or_else(|| stdout_redacted.text.clone(), |v| v.text.clone());
    let stdout_facts = FieldFacts {
        field: "stdout",
        normalization: stdout_normalized.facts,
        redaction: stdout_redacted.facts,
        filter: filtered.and_then(|v| v.facts),
    };

    let stderr_normalized = normalize_terminal_text(bash.stderr);
    let stderr_redacted = redact_recognized_secrets(&stderr_normalized.text);
    let stderr_facts = FieldFacts {
        field: "stderr",
        normalization: stderr_normalized.facts,
        redaction: stderr_redacted.facts,
        filter: None,
    };
    (
        stdout,
        stderr_redacted.text,
        vec![stdout_facts, stderr_facts],
    )
}

fn classify_unsupported(event: &Value) -> PassThroughReason {
    let Some(object) = event.as_object() else {
        return PassThroughReason::InvalidEnvelope;
    };
    match object.get("hook_event_name").and_then(Value::as_str) {
        Some("PostToolUse") => {}
        _ => return PassThroughReason::UnsupportedEvent,
    }
    if object.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return PassThroughReason::UnsupportedTool;
    }
    let Some(input) = object.get("tool_input").and_then(Value::as_object) else {
        return PassThroughReason::UnsupportedInput;
    };
    if input.get("command").and_then(Value::as_str).is_none() {
        return PassThroughReason::UnsupportedInput;
    }
    let Some(response) = object.get("tool_response").and_then(Value::as_object) else {
        return PassThroughReason::UnsupportedSchema;
    };
    if response.get("interrupted") == Some(&Value::Bool(true)) {
        return PassThroughReason::Interrupted;
    }
    if response.get("isImage") == Some(&Value::Bool(true)) {
        return PassThroughReason::ImageResult;
    }
    PassThroughReason::UnsupportedSchema
}

#[cfg(test)]
#[path = "../tests/claude_native/pipeline.rs"]
mod tests;
