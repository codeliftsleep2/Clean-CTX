use serde_json::{Map, Value};

pub(super) const SCHEMA_ID: &str = "bash-success-v1";

pub(super) struct BashSuccess<'a> {
    pub command: &'a str,
    pub response: &'a Map<String, Value>,
    pub stdout: &'a str,
    pub stderr: &'a str,
}

pub(super) fn recognize(event: &Value) -> Option<BashSuccess<'_>> {
    let object = event.as_object()?;
    if object.get("hook_event_name")?.as_str()? != "PostToolUse"
        || object.get("tool_name")?.as_str()? != "Bash"
    {
        return None;
    }
    let input = object.get("tool_input")?.as_object()?;
    let response = object.get("tool_response")?.as_object()?;
    let command = input.get("command")?.as_str()?;
    let stdout = response.get("stdout")?.as_str()?;
    let stderr = response.get("stderr")?.as_str()?;
    if response.get("interrupted")?.as_bool()? || response.get("isImage")?.as_bool()? {
        return None;
    }
    Some(BashSuccess {
        command,
        response,
        stdout,
        stderr,
    })
}

pub(super) fn validate_reconstruction(
    original: &Map<String, Value>,
    rebuilt: &Map<String, Value>,
) -> bool {
    if rebuilt.get("stdout").and_then(Value::as_str).is_none()
        || rebuilt.get("stderr").and_then(Value::as_str).is_none()
        || rebuilt.get("interrupted") != Some(&Value::Bool(false))
        || rebuilt.get("isImage") != Some(&Value::Bool(false))
        || original.len() != rebuilt.len()
    {
        return false;
    }
    original.iter().all(|(key, value)| {
        matches!(key.as_str(), "stdout" | "stderr") || rebuilt.get(key) == Some(value)
    })
}

#[cfg(test)]
#[path = "../tests/claude_native/bash_schema.rs"]
mod tests;
