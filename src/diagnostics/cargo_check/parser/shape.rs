//! Validate fields whose values become semantic claims; ignore unrelated future
//! metadata. Absence of optional fields remains compatible with the prior core.
use serde_json::{Map, Value};

fn optional(value: &Map<String, Value>, key: &str, check: impl FnOnce(&Value) -> bool) -> bool {
    value.get(key).is_none_or(check)
}

fn nullable_string(value: &Value) -> bool {
    value.is_null() || value.is_string()
}

fn string_array(value: &Value) -> bool {
    value
        .as_array()
        .is_some_and(|items| items.iter().all(Value::is_string))
}

pub(super) fn diagnostic(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    object.get("message").is_some_and(Value::is_string)
        && object.get("level").is_some_and(Value::is_string)
        && optional(object, "rendered", nullable_string)
        && optional(object, "code", |code| {
            code.is_null()
                || code
                    .as_object()
                    .is_some_and(|code| code.get("code").is_some_and(Value::is_string))
        })
        && optional(object, "spans", spans)
        && optional(object, "children", |children| {
            children.as_array().is_some_and(|children| {
                children.iter().all(|child| {
                    child.as_object().is_some_and(|child| {
                        child.get("message").is_some_and(Value::is_string)
                            && child.get("level").is_some_and(Value::is_string)
                            && optional(child, "spans", spans)
                    })
                })
            })
        })
}

fn spans(value: &Value) -> bool {
    value.as_array().is_some_and(|spans| {
        spans.iter().all(|span| {
            let Some(span) = span.as_object() else {
                return false;
            };
            span.get("file_name").is_some_and(Value::is_string)
                && span.get("is_primary").is_some_and(Value::is_boolean)
                && ["line_start", "line_end", "column_start", "column_end"]
                    .iter()
                    .all(|key| span.get(*key).and_then(Value::as_u64).is_some())
                && ["label", "suggested_replacement", "suggestion_applicability"]
                    .iter()
                    .all(|key| optional(span, key, nullable_string))
        })
    })
}

pub(super) fn artifact(object: &Map<String, Value>) -> bool {
    ["package_id", "manifest_path"]
        .iter()
        .all(|key| optional(object, key, Value::is_string))
        && ["target", "profile"]
            .iter()
            .all(|key| optional(object, key, Value::is_object))
        && ["features", "filenames"]
            .iter()
            .all(|key| optional(object, key, string_array))
        && optional(object, "executable", nullable_string)
        && optional(object, "fresh", Value::is_boolean)
}

pub(super) fn build_script(object: &Map<String, Value>) -> bool {
    ["package_id", "out_dir"]
        .iter()
        .all(|key| optional(object, key, Value::is_string))
        && ["linked_libs", "linked_paths", "cfgs"]
            .iter()
            .all(|key| optional(object, key, string_array))
        && optional(object, "env", |env| {
            env.as_array().is_some_and(|entries| {
                entries.iter().all(|entry| {
                    entry
                        .as_array()
                        .is_some_and(|pair| pair.len() == 2 && pair.iter().all(Value::is_string))
                })
            })
        })
}
