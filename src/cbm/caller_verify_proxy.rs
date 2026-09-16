//! `cbm_proxy` integration for caller verification.
//!
//! Verification happens after CBM returns candidate evidence and before the
//! response is compressed. Supplementary CBM queries only recover candidate
//! file paths; source parsing remains the semantic authority.
//!
//! Two orchestration paths feed the *same* shared verifier:
//!
//! * single-target surfaces (`trace_path`, `query_graph`) — one
//!   [`VerificationRequest`] per response, verified here;
//! * `search_graph` — one request per eligible result, orchestrated by
//!   [`crate::cbm::caller_verify_search`].

use crate::cbm::GraphBridge;
use crate::cbm::caller_verify::{
    CallerVerificationSummary, CandidateSource, CsharpTargetSelector, annotate_caller_evidence,
    verify_csharp_callers,
};
use crate::cbm::caller_verify_search;
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
    let annotated = if tool == "search_graph" {
        verify_search_surface(bridge, state, args, workspace_root, &mut payload)
    } else {
        verify_single_surface(bridge, state, tool, args, workspace_root, &mut payload)
    };
    if !annotated {
        return raw.to_string();
    }
    if let Some(text_slot) = envelope.pointer_mut("/result/content/0/text") {
        *text_slot = Value::String(payload.to_string());
    }
    envelope.to_string()
}

/// Single-target surfaces (`trace_path`, `query_graph`): one request per
/// response, verified exactly as before the plural `search_graph` path existed.
///
/// Returns `false` when the surface carries no verification target, in which
/// case the caller returns CBM's response untouched.
fn verify_single_surface(
    bridge: &mut GraphBridge,
    state: &McpState,
    tool: &str,
    args: &Value,
    workspace_root: Option<&str>,
    payload: &mut Value,
) -> bool {
    let Some(request) = verification_request(tool, args, payload) else {
        return false;
    };
    let mut attempt = verify_request_sources(bridge, state, &request, workspace_root);
    if request.raw_candidates > 0 {
        attempt.summary.raw_candidates = request.raw_candidates;
    }
    annotate_caller_evidence(payload, tool, &attempt.summary);
    annotate_surface_counts(payload, tool, &attempt.summary);
    true
}

/// `search_graph` returns a *set* of symbols, so caller verification belongs to
/// each result: every result carrying `in_degree` is verified on its own.
///
/// Returns `false` when no result carries `in_degree`, in which case there is
/// nothing to verify and CBM's response is returned untouched.
fn verify_search_surface(
    bridge: &mut GraphBridge,
    state: &McpState,
    args: &Value,
    workspace_root: Option<&str>,
    payload: &mut Value,
) -> bool {
    let targets = caller_verify_search::search_targets(payload, args["project"].as_str());
    if targets.is_empty() {
        return false;
    }
    let aggregate =
        caller_verify_search::verify_search_results(payload, &targets, &mut |request| {
            verify_request_sources(bridge, state, request, workspace_root)
        });
    caller_verify_search::annotate_search_evidence(payload, &aggregate);
    true
}

/// Surface-level counts for the single-target surfaces whose response carries
/// no per-result place to attach evidence.
///
/// `search_graph` is intentionally absent: its counts live in the aggregate
/// block and its evidence lives on each result
/// ([`crate::cbm::caller_verify_search`]).
fn annotate_surface_counts(payload: &mut Value, tool: &str, summary: &CallerVerificationSummary) {
    let counts = serde_json::json!({
        "raw_candidates": summary.raw_candidates,
        "verified": summary.verified,
        "rejected_arity": summary.rejected_arity,
        "ambiguous": summary.ambiguous,
        "unverifiable": summary.unverifiable,
    });
    match tool {
        "trace_path" => payload["caller_counts"] = counts,
        "query_graph" => payload["inbound_call_counts"] = counts,
        _ => {}
    }
}

/// One verification target.
///
/// The single-target surfaces build exactly one of these; `search_graph` builds
/// one per eligible result and reuses the same type and verifier.
#[derive(Debug)]
pub(crate) struct VerificationRequest {
    pub(crate) target: Option<String>,
    pub(crate) raw_candidates: usize,
    pub(crate) project: Option<String>,
}

/// Outcome of one verification attempt.
///
/// `summary` is the shared verifier's result. `gap` records *where* source
/// evidence could not be established, so a surface can report a truthful reason
/// instead of an unexplained "unverifiable"; it is `None` whenever the verifier
/// actually ran.
#[derive(Debug)]
pub(crate) struct VerificationAttempt {
    pub(crate) summary: CallerVerificationSummary,
    pub(crate) gap: Option<VerificationGap>,
}

impl VerificationAttempt {
    fn unverifiable(target: Option<&str>, gap: Option<VerificationGap>) -> Self {
        Self {
            summary: CallerVerificationSummary::unverifiable(target),
            gap,
        }
    }
}

/// Stage at which a verification attempt could not establish source evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VerificationGap {
    /// CBM held no CALLS relationship for the symbol, so neither a target
    /// source file nor candidate caller files could be recovered.
    NoCallEvidence,
    /// A target path was recovered but its source could not be read through the
    /// trusted-path gate.
    TargetSourceUnreadable,
    /// Caller paths were recovered but none of their sources could be read.
    CandidateSourcesUnreadable,
    /// Some caller sources could not be read, so the counts are incomplete.
    PartialCandidateSources,
}

impl VerificationGap {
    /// Machine-readable reason reported next to an unverified result.
    pub(crate) fn reason(self) -> &'static str {
        match self {
            Self::NoCallEvidence => "no_cbm_call_evidence",
            Self::TargetSourceUnreadable => "target_source_unreadable",
            Self::CandidateSourcesUnreadable => "candidate_sources_unreadable",
            Self::PartialCandidateSources => "partial_candidate_sources",
        }
    }
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
) -> VerificationAttempt {
    let Some(target) = request.target.as_deref() else {
        return VerificationAttempt::unverifiable(None, None);
    };
    let Some((target_path, candidate_paths)) =
        candidate_paths(bridge, target, request.project.as_deref())
    else {
        return VerificationAttempt::unverifiable(
            Some(target),
            Some(VerificationGap::NoCallEvidence),
        );
    };
    let Some(target_source) = read_trusted_source(state, &target_path, workspace_root) else {
        return VerificationAttempt::unverifiable(
            Some(target),
            Some(VerificationGap::TargetSourceUnreadable),
        );
    };
    let loaded: Vec<(String, String)> = candidate_paths
        .iter()
        .filter_map(|path| {
            read_trusted_source(state, path, workspace_root).map(|source| (path.clone(), source))
        })
        .collect();
    if loaded.is_empty() && request.raw_candidates > 0 {
        return VerificationAttempt::unverifiable(
            Some(target),
            Some(VerificationGap::CandidateSourcesUnreadable),
        );
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
        return VerificationAttempt {
            summary,
            gap: Some(VerificationGap::PartialCandidateSources),
        };
    }
    VerificationAttempt { summary, gap: None }
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
#[path = "../tests/cbm/caller_verify_proxy.rs"]
mod tests;
