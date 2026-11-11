//! Cache canonical fragments and their MCP text-string representation once per operation.
use super::{Error, Result, identity};
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) struct Fragment {
    pub json: String,
    pub escaped: String,
}
impl Fragment {
    pub fn value(value: &Value) -> Result<Self> {
        let json = identity::canonical(value);
        let quoted = serde_json::to_string(&json).map_err(|_| Error("wire_encoding"))?;
        Ok(Self {
            json,
            escaped: quoted[1..quoted.len() - 1].into(),
        })
    }
    pub fn array<'a>(items: impl Iterator<Item = &'a Self>) -> Self {
        let mut json = String::from("[");
        let mut escaped = String::from("[");
        for (index, item) in items.enumerate() {
            if index > 0 {
                json.push(',');
                escaped.push(',');
            }
            json.push_str(&item.json);
            escaped.push_str(&item.escaped);
        }
        json.push(']');
        escaped.push(']');
        Self { json, escaped }
    }
}

/// Exactly one content block, no structuredContent or duplicate evidence channel.
pub(super) fn result(fields: BTreeMap<&str, Fragment>) -> Result<String> {
    let mut text = String::from("{");
    for (index, (key, value)) in fields.into_iter().enumerate() {
        if index > 0 {
            text.push(',');
        }
        text.push_str(&Fragment::value(&Value::String(key.into()))?.escaped);
        text.push(':');
        text.push_str(&value.escaped);
    }
    text.push('}');
    Ok(format!(
        "{{\"content\":[{{\"type\":\"text\",\"text\":\"{text}\"}}],\"isError\":false}}"
    ))
}
