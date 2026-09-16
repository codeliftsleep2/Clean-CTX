//! `cbm_proxy` integration for caller verification.
//!
//! Verification happens after CBM returns candidate evidence and before the
//! response is compressed. Supplementary CBM queries only recover candidate
//! file paths; source parsing remains the semantic authority.

use crate::cbm::GraphBridge;
use crate::cbm::caller_verify::{
    CallerVerificationSummary, CandidateSource, CsharpTargetSelector, annotate_caller_evidence,
    verify_csharp_callers,
};
use crate::mcp::McpState;
use serde_json::Value;
use std::collections::HashSet;

pub(crate) fn verify_proxy_response(
    bridge: &mut GraphBridge,
    state: &McpState,
    tool: &str,
    args: &Value,
    workspace_root: Option<&str>,
    raw: &str,
) -> String {
    let Some(mut envelope) = serde_json::from_str::<Value>(raw.trim()).ok() else {
        return raw.to_string();
    };
    let Some(text) = envelope
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
    else {
        return raw.to_string();
    };
    let Some(mut payload) = serde_json::from_str::<Value>(text).ok() else {
        return raw.to_string();
    };
    let Some(request) = verification_request(tool, args, &payload) else {
        return raw.to_string();
    };

    let mut summary = verify_request_sources(bridge, state, &request, workspace_root);
    if request.raw_candidates > 0 {
        summary.raw_candidates = request.raw_candidates;
    }
    annotate_caller_evidence(&mut payload, tool, &summary);
    annotate_surface_counts(&mut payload, tool, &summary);
    if let Some(text_slot) = envelope.pointer_mut("/result/content/0/text") {
        *text_slot = Value::String(payload.to_string());
    }
    envelope.to_string()
}

fn annotate_surface_counts(payload: &mut Value, tool: &str, summary: &CallerVerificationSummary) {
    let counts = serde_json::json!({
        "raw_candidates": summary.raw_candidates,
        "verified": summary.verified,
        "rejected_arity": summary.rejected_arity,
        "ambiguous": summary.ambiguous,
        "unverifiable": summary.unverifiable,
    });
    match tool {
        "search_graph" => {
            if let Some(results) = payload["results"].as_array_mut() {
                if results.len() == 1 {
                    if let Some(result) = results[0].as_object_mut() {
                        let raw = result.get("in_degree").cloned().unwrap_or(Value::Null);
                        result.insert("raw_in_degree".into(), raw);
                        result.insert(
                            "verified_in_degree".into(),
                            Value::from(summary.verified as u64),
                        );
                        result.insert("in_degree_evidence".into(), Value::String("cbm_raw".into()));
                    }
                }
            }
        }
        "trace_path" => payload["caller_counts"] = counts,
        "query_graph" => payload["inbound_call_counts"] = counts,
        _ => {}
    }
}

#[derive(Debug)]
struct VerificationRequest {
    target: Option<String>,
    raw_candidates: usize,
    project: Option<String>,
}

fn verification_request(tool: &str, args: &Value, payload: &Value) -> Option<VerificationRequest> {
    match tool {
        "trace_path" => {
            let direction = args["direction"].as_str().unwrap_or("both");
            if direction != "inbound" && direction != "both" {
                return None;
            }
            Some(VerificationRequest {
                target: args["function_name"].as_str().map(str::to_string),
                raw_candidates: payload["callers"].as_array().map_or(0, Vec::len),
                project: args["project"].as_str().map(str::to_string),
            })
        }
        "search_graph" => {
            let results: Vec<&Value> = payload["results"]
                .as_array()?
                .iter()
                .filter(|result| result.get("in_degree").is_some())
                .collect();
            if results.is_empty() {
                return None;
            }
            let target = (results.len() == 1)
                .then(|| results[0]["qualified_name"].as_str().map(str::to_string))
                .flatten();
            Some(VerificationRequest {
                target,
                raw_candidates: (results.len() == 1)
                    .then(|| numeric_usize(&results[0]["in_degree"]))
                    .flatten()
                    .unwrap_or(0),
                project: args["project"].as_str().map(str::to_string),
            })
        }
        "query_graph" => {
            let query = args["query"].as_str()?;
            if !query.to_ascii_uppercase().contains("CALLS") {
                return None;
            }
            Some(VerificationRequest {
                target: extract_qualified_target(query),
                raw_candidates: payload["rows"].as_array().map_or(0, Vec::len),
                project: args["project"].as_str().map(str::to_string),
            })
        }
        _ => None,
    }
}

fn verify_request_sources(
    bridge: &mut GraphBridge,
    state: &McpState,
    request: &VerificationRequest,
    workspace_root: Option<&str>,
) -> CallerVerificationSummary {
    let Some(target) = request.target.as_deref() else {
        return CallerVerificationSummary::unverifiable(None);
    };
    let Some((target_path, candidate_paths)) =
        candidate_paths(bridge, target, request.project.as_deref())
    else {
        return CallerVerificationSummary::unverifiable(Some(target));
    };
    let Some(target_source) = read_trusted_source(state, &target_path, workspace_root) else {
        return CallerVerificationSummary::unverifiable(Some(target));
    };
    let loaded: Vec<(String, String)> = candidate_paths
        .iter()
        .filter_map(|path| {
            read_trusted_source(state, path, workspace_root).map(|source| (path.clone(), source))
        })
        .collect();
    if loaded.is_empty() && request.raw_candidates > 0 {
        return CallerVerificationSummary::unverifiable(Some(target));
    }
    let failed_sources = candidate_paths.len().saturating_sub(loaded.len());
    let views: Vec<CandidateSource<'_>> = loaded
        .iter()
        .map(|(file, text)| CandidateSource { file, text })
        .collect();
    let mut summary =
        verify_csharp_callers(&target_source, CsharpTargetSelector::new(target), &views);
    if failed_sources > 0 {
        summary.unverifiable += failed_sources;
        summary.resolution = crate::cbm::caller_verify::CallerVerificationStatus::Unverifiable;
    }
    summary
}

fn candidate_paths(
    bridge: &mut GraphBridge,
    target: &str,
    project: Option<&str>,
) -> Option<(String, Vec<String>)> {
    let query = candidate_path_query(target);
    let mut query_args = serde_json::json!({"query": query});
    if let Some(project) = project {
        query_args["project"] = Value::String(project.to_string());
    }
    let raw = bridge.proxy_call("query_graph", query_args).ok()?;
    let payload = inner_payload(&raw)?;
    let rows = payload["rows"].as_array()?;
    let mut target_path = None;
    let mut callers = HashSet::new();
    for row in rows.iter().filter_map(Value::as_array) {
        if let Some(path) = row
            .first()
            .and_then(Value::as_str)
            .filter(|path| path.ends_with(".cs"))
        {
            callers.insert(path.to_string());
        }
        if target_path.is_none() {
            target_path = row
                .get(1)
                .and_then(Value::as_str)
                .filter(|path| path.ends_with(".cs"))
                .map(str::to_string);
        }
    }
    Some((target_path?, callers.into_iter().collect()))
}

fn candidate_path_query(target: &str) -> String {
    let escaped = target.replace('\\', "\\\\").replace('\'', "\\'");
    format!(
        "MATCH (caller)-[:CALLS]->(target) \
         WHERE target.qualified_name = '{escaped}' \
         RETURN caller.file_path, target.file_path"
    )
}

fn read_trusted_source(
    state: &McpState,
    path: &str,
    workspace_root: Option<&str>,
) -> Option<String> {
    let resolved = crate::mcp::tool_helpers::resolve_file_path_checked(
        path,
        workspace_root,
        &state.config.additional_roots,
    )
    .ok()?;
    state
        .read_source(&resolved)
        .ok()
        .map(|source| source.to_string())
}

fn inner_payload(raw: &str) -> Option<Value> {
    let envelope: Value = serde_json::from_str(raw.trim()).ok()?;
    let text = envelope.pointer("/result/content/0/text")?.as_str()?;
    serde_json::from_str(text).ok()
}

fn numeric_usize(value: &Value) -> Option<usize> {
    value
        .as_u64()
        .and_then(|number| usize::try_from(number).ok())
        .or_else(|| value.as_str()?.parse().ok())
}

fn extract_qualified_target(query: &str) -> Option<String> {
    let lower = query.to_ascii_lowercase();
    let marker = "qualified_name";
    let marker_start = lower.find(marker)?;
    let alias = lower[..marker_start]
        .trim_end_matches('.')
        .rsplit(|character: char| !character.is_alphanumeric() && character != '_')
        .next()?;
    let compact: String = lower
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    if !compact.contains(&format!("calls]->({alias}")) {
        return None;
    }
    let start = marker_start + marker.len();
    let remainder = &query[start..];
    let equals = remainder.find('=')? + 1;
    let quoted = remainder[equals..].trim_start();
    let quote = quoted
        .chars()
        .next()
        .filter(|ch| *ch == '\'' || *ch == '"')?;
    let value = &quoted[quote.len_utf8()..];
    let end = value.find(quote)?;
    Some(value[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        annotate_surface_counts, candidate_path_query, extract_qualified_target,
        verification_request,
    };
    use crate::cbm::caller_verify::{CallerVerificationStatus, CallerVerificationSummary};
    use serde_json::json;

    #[test]
    fn extracts_qualified_target_from_inbound_query() {
        let query = "MATCH (c)-[:CALLS]->(target) WHERE target.qualified_name = 'A.Foo' RETURN c";
        assert_eq!(extract_qualified_target(query).as_deref(), Some("A.Foo"));
        let outbound =
            "MATCH (caller)-[:CALLS]->(callee) WHERE caller.qualified_name = 'A.Foo' RETURN callee";
        assert_eq!(extract_qualified_target(outbound), None);
    }

    #[test]
    fn candidate_path_query_is_label_neutral_and_escapes_identity() {
        let query = candidate_path_query("A.O'Reilly\\Foo");
        assert!(query.contains("MATCH (caller)-[:CALLS]->(target)"));
        assert!(!query.contains(":Function"));
        assert!(query.contains("A.O\\'Reilly\\\\Foo"));
    }

    #[test]
    fn caller_surfaces_build_consistent_verification_requests() {
        let trace = verification_request(
            "trace_path",
            &json!({"function_name":"A.Foo","direction":"inbound"}),
            &json!({"callers":[{}, {}]}),
        )
        .expect("inbound trace should be verified");
        let search = verification_request(
            "search_graph",
            &json!({}),
            &json!({"results":[{"qualified_name":"A.Foo","in_degree":2}]}),
        )
        .expect("in-degree search should be verified");
        let query = verification_request(
            "query_graph",
            &json!({"query":"MATCH (c)-[:CALLS]->(t) WHERE t.qualified_name = 'A.Foo' RETURN c"}),
            &json!({"rows":[[], []]}),
        )
        .expect("inbound CALLS query should be verified");

        for request in [trace, search, query] {
            assert_eq!(request.target.as_deref(), Some("A.Foo"));
            assert_eq!(request.raw_candidates, 2);
        }
    }

    #[test]
    fn search_keeps_raw_degree_and_adds_verified_degree() {
        let mut payload = json!({"results":[{"in_degree":39}]});
        let summary = CallerVerificationSummary {
            target_qualified_name: Some("A.OrderBy".into()),
            target_explicit_arity: Some(2),
            raw_candidates: 39,
            verified: 3,
            rejected_arity: 36,
            ambiguous: 0,
            unverifiable: 0,
            verified_caller_files: 1,
            compatible_caller_files: 1,
            resolution: CallerVerificationStatus::VerifiedCompatible,
        };
        annotate_surface_counts(&mut payload, "search_graph", &summary);
        assert_eq!(payload["results"][0]["in_degree"], 39);
        assert_eq!(payload["results"][0]["raw_in_degree"], 39);
        assert_eq!(payload["results"][0]["verified_in_degree"], 3);
        assert_eq!(payload["results"][0]["in_degree_evidence"], "cbm_raw");
    }
}
