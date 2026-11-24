//! Exact approved additive schemas; advertised only for a configured opt-in session.
use serde_json::{Value, json};
pub(super) fn request(batch: bool) -> Value {
    serde_json::from_str(if batch {
        include_str!("schemas/batch-request.json")
    } else {
        include_str!("schemas/request.json")
    })
    .expect("reviewed schema")
}
pub(super) fn response(batch: bool) -> Value {
    serde_json::from_str(if batch {
        include_str!("schemas/batch-response.json")
    } else {
        include_str!("schemas/response.json")
    })
    .expect("reviewed schema")
}
pub(crate) fn advertise(tools: &mut [Value], state: &super::State) {
    if !state.enabled() {
        return;
    }
    for tool in tools.iter_mut().filter(|v| v["name"] == "workspace_query") {
        let mut legacy = tool["inputSchema"].take();
        legacy["properties"]["contract_version"] = json!({"enum":[1]});
        legacy["properties"]["queries"]["items"]["properties"]["contract_version"] =
            json!({"enum":[1]});
        tool["inputSchema"] =
            json!({"type":"object","oneOf":[legacy,request(false),request(true)]});
        let mut output = tool["outputSchema"].take();
        output["not"] =
            json!({"properties":{"contract_version":{"const":2}},"required":["contract_version"]});
        tool["outputSchema"] =
            json!({"type":"object","oneOf":[output,response(false),response(true)]});
        tool["description"] = json!(format!(
            "{} Opt-in resolver_registrations contract_version 2: interactive discovery only, canonical_policy none. Forward route_literal (including empty route_path); reverse written_reference text (resolver_name), not canonical identity. Optional explicit source_files avoids unrelated enumeration. Required finite request-wide budget is capped by operator ceilings. Proven written facts and physical endpoints are distinct from qualified Angular interpretations and canonical registrations. Coverage may be open/unknown; all qualifications appear in both structured and text output. Exhaustive mode/canonical lookup unsupported. Homogeneous v2 batches share one ledger; mixed v1/v2 batches refuse before acquisition.",
            tool["description"].as_str().unwrap_or_default()
        ));
        tool["resolverEvidenceCapabilities"] = json!({"contract_versions":[1,2],"modes":["interactive"],"canonical_policies":["none"],"default_contract_version":1,"activation":"operator_profile","ceiling":state.ceiling()});
    }
}
/// Closed request-schema interpreter for the exact reviewed JSON Schema vocabulary.
/// No schema compilation, external references, acquisition or filesystem access.
pub(super) fn accepts(s: &Value, v: &Value) -> bool {
    if let Some(c) = s.get("const") {
        if c != v {
            return false;
        }
    }
    if let Some(a) = s["enum"].as_array() {
        if !a.contains(v) {
            return false;
        }
    }
    if let Some(t) = s["type"].as_str() {
        if !match t {
            "object" => v.is_object(),
            "array" => v.is_array(),
            "string" => v.is_string(),
            "integer" => v.as_u64().is_some(),
            "boolean" => v.is_boolean(),
            "null" => v.is_null(),
            _ => false,
        } {
            return false;
        }
    }
    if let Some(a) = s["required"].as_array() {
        if a.iter().any(|k| v.get(k.as_str().unwrap()).is_none()) {
            return false;
        }
    }
    if let Some(o) = v.as_object() {
        for (k, v) in o {
            if let Some(p) = s["properties"].get(k) {
                if !accepts(p, v) {
                    return false;
                }
            } else if s["additionalProperties"] == false {
                return false;
            }
        }
    }
    if let Some(a) = v.as_array() {
        if s["minItems"].as_u64().is_some_and(|n| (a.len() as u64) < n)
            || s["maxItems"].as_u64().is_some_and(|n| a.len() as u64 > n)
        {
            return false;
        }
        if s["uniqueItems"] == true && a.iter().enumerate().any(|(i, v)| a[..i].contains(v)) {
            return false;
        }
        if let Some(item) = s.get("items") {
            if a.iter().any(|v| !accepts(item, v)) {
                return false;
            }
        }
    }
    if let Some(t) = v.as_str() {
        if s["minLength"]
            .as_u64()
            .is_some_and(|n| (t.chars().count() as u64) < n)
        {
            return false;
        }
    }
    if let Some(n) = v.as_u64() {
        if s["minimum"].as_u64().is_some_and(|min| n < min)
            || s["maximum"].as_u64().is_some_and(|max| n > max)
        {
            return false;
        }
    }
    if let Some(a) = s["oneOf"].as_array() {
        if a.iter().filter(|s| accepts(s, v)).count() != 1 {
            return false;
        }
    }
    if let Some(a) = s["anyOf"].as_array() {
        if !a.iter().any(|s| accepts(s, v)) {
            return false;
        }
    }
    if let Some(a) = s["allOf"].as_array() {
        if a.iter().any(|s| !accepts(s, v)) {
            return false;
        }
    }
    if let Some(n) = s.get("not") {
        if accepts(n, v) {
            return false;
        }
    }
    if let Some(i) = s.get("if") {
        if accepts(i, v) {
            if let Some(t) = s.get("then") {
                if !accepts(t, v) {
                    return false;
                }
            }
        }
    }
    true
}

pub(crate) fn initialization(mut response: Value, state: &super::State) -> Value {
    if state.enabled() {
        response["result"]["capabilities"]["experimental"] = json!({"clean-ctx/resolver-evidence":{"contract_versions":[1,2],"modes":["interactive"],"canonical_policies":["none"],"operator_configured":true}});
    }
    response
}
