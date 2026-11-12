//! Opt-in transport projection only. Store revisions and traversal stay unchanged.
use crate::pcg::{Error, Result, identity, require};
use serde_json::{Value, json};
use std::collections::BTreeMap;

fn intern(
    sources: &mut BTreeMap<(String, String), Value>,
    path: String,
    sha: String,
    text: Value,
) -> Result<String> {
    let key = (path.clone(), sha.clone());
    if let Some(source) = sources.get(&key) {
        require(source["text"] == text, "document_source_identity")?;
        return source["source_ref"]
            .as_str()
            .map(str::to_owned)
            .ok_or(Error("document_response"));
    }
    require(
        identity::sha(text.as_str().ok_or(Error("document_response"))?.as_bytes()) == sha,
        "document_source_identity",
    )?;
    let reference = identity::hash(&json!({"path":path,"sha256":sha}));
    sources.insert(
        key,
        json!({"source_ref":reference,"path":path,"sha256":sha,"text":text}),
    );
    Ok(reference)
}

/// One response-local pool; O(payload bytes + occurrences * log(unique identities)).
/// Hash each distinct identity once; compare each reuse exactly. No persistent cache.
/// Returns the number of constructed source entries for deterministic work coverage.
pub(super) fn source_refs(payload: &mut Value) -> Result<usize> {
    let mut sources = BTreeMap::new();
    for node in payload["documents"]
        .as_array_mut()
        .ok_or(Error("document_response"))?
    {
        let node = node.as_object_mut().ok_or(Error("document_response"))?;
        let path = node["path"]
            .as_str()
            .ok_or(Error("document_response"))?
            .to_owned();
        let sha = node["source_sha256"]
            .as_str()
            .ok_or(Error("document_response"))?
            .to_owned();
        let text = node
            .remove("source_text")
            .ok_or(Error("document_response"))?;
        let reference = intern(&mut sources, path, sha, text)?;
        node.insert("source_ref".into(), reference.into());
        for evidence in node
            .get_mut("evidence")
            .and_then(Value::as_array_mut)
            .ok_or(Error("document_response"))?
        {
            let evidence = evidence.as_object_mut().ok_or(Error("document_response"))?;
            if !evidence.get("text").is_some_and(Value::is_string) {
                continue;
            }
            let path = evidence["path"]
                .as_str()
                .ok_or(Error("document_response"))?
                .to_owned();
            let sha = evidence["sha256"]
                .as_str()
                .ok_or(Error("document_response"))?
                .to_owned();
            let text = evidence.remove("text").ok_or(Error("document_response"))?;
            let reference = intern(&mut sources, path, sha, text)?;
            evidence.insert("source_ref".into(), reference.into());
        }
    }
    let count = sources.len();
    payload["sources"] = sources.into_values().collect::<Vec<_>>().into();
    payload["version"] = 2.into();
    payload["response_format"] = "source_refs_v1".into();
    Ok(count)
}

#[cfg(test)]
#[path = "../../tests/pcg/document_response.rs"]
mod tests;
