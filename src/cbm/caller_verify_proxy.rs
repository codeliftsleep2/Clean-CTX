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
use crate::cbm::bridge::QueryResult;
use crate::cbm::caller_verify::{
    CallerVerificationSummary, CandidateSource, CsharpTargetSelector, annotate_caller_evidence,
    verify_csharp_callers,
};
use crate::cbm::caller_verify_arity::ParseMemo;
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
    let mut attempt = verify_request_sources(
        bridge,
        state,
        &request,
        workspace_root,
        &mut ParseMemo::new(),
    );
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
        caller_verify_search::verify_search_results(payload, &targets, &mut |request, memo| {
            verify_request_sources(bridge, state, request, workspace_root, memo)
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

/// Verify one target's CBM candidates against source.
///
/// `memo` is the batch-local parse memo the caller owns (one per response), so a
/// file reached by several results is parsed once. It carries no cross-request
/// state: cached *candidate discovery* never makes verification truth stale.
pub(crate) fn verify_request_sources(
    bridge: &mut GraphBridge,
    state: &McpState,
    request: &VerificationRequest,
    workspace_root: Option<&str>,
    memo: &mut ParseMemo,
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
    let Some((target_resolved, target_source)) =
        read_trusted_source(state, &target_path, workspace_root)
    else {
        return VerificationAttempt::unverifiable(
            Some(target),
            Some(VerificationGap::TargetSourceUnreadable),
        );
    };
    let loaded: Vec<LoadedCandidate> = candidate_paths
        .iter()
        .filter_map(|cbm_file| {
            read_trusted_source(state, cbm_file, workspace_root).map(|(resolved, source)| {
                LoadedCandidate {
                    cbm_file: cbm_file.clone(),
                    resolved_file: resolved,
                    source,
                }
            })
        })
        .collect();
    if loaded.is_empty() && request.raw_candidates > 0 {
        return VerificationAttempt::unverifiable(
            Some(target),
            Some(VerificationGap::CandidateSourcesUnreadable),
        );
    }
    let failed_sources = candidate_paths.len().saturating_sub(loaded.len());
    // Each candidate keeps CBM's proposed caller file beside the path Clean-CTX
    // actually read, so verification can report the identity it verified instead
    // of only counting it.
    let views: Vec<CandidateSource<'_>> = loaded
        .iter()
        .map(|candidate| CandidateSource {
            file: &candidate.cbm_file,
            resolved_file: &candidate.resolved_file,
            text: &candidate.source,
        })
        .collect();
    let mut summary = verify_csharp_callers(
        &target_source,
        &target_resolved,
        CsharpTargetSelector::new(target),
        &views,
        memo,
    );
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

/// One readable candidate: CBM's proposed caller file, the trusted path Clean-CTX
/// read, and that file's source.
struct LoadedCandidate {
    cbm_file: String,
    resolved_file: String,
    source: String,
}

/// Discover the CBM candidate caller files for one target.
///
/// One query, two transports. When the request names a project, the *cached*
/// project-explicit graph query answers
/// ([`GraphBridge::query_graph_scoped`]): repeated verification of the same
/// symbol — a broad search's second pass, a re-issued request — then costs no
/// CBM round-trip, and the cache key carries the project, so two repositories
/// that share a symbol name can never serve each other's candidates. The Cypher
/// carries the remaining discovery semantics: relationship direction
/// (`[:CALLS]->`), the target symbol (`WHERE target.qualified_name = ...`), and
/// the projection.
///
/// A request that names **no** project keeps the original raw proxy call
/// unchanged: such a query is answered by CBM's own default project, which the
/// bridge's active project does not describe, so there is no project to key an
/// entry by — caching it under any key could serve one project's candidates to
/// another. Candidate discovery is therefore cached exactly where the cache
/// contract can express what the query means.
pub(crate) fn candidate_paths(
    bridge: &mut GraphBridge,
    target: &str,
    project: Option<&str>,
) -> Option<(String, Vec<String>)> {
    let query = candidate_path_query(target);
    if let Some(project) = project {
        let resolved = bridge.resolve_project_id(project);
        return caller_paths_from_query(&bridge.query_graph_scoped(&query, &resolved));
    }
    let raw = bridge
        .proxy_call("query_graph", serde_json::json!({ "query": query }))
        .ok()?;
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

/// Caller/target paths from the cached `query_graph` view of the candidate query.
///
/// `convert_query_rows` maps the two-column candidate projection
/// (`caller.file_path, target.file_path`) to a node whose `id`/`name` is the
/// caller path and whose `file` is the target path — the same two cells the raw
/// proxy path above reads positionally, so both transports yield the same
/// candidate set.
fn caller_paths_from_query(result: &QueryResult) -> Option<(String, Vec<String>)> {
    let mut target_path = None;
    let mut callers = HashSet::new();
    for node in &result.nodes {
        if node.id.ends_with(".cs") {
            callers.insert(node.id.clone());
        }
        if target_path.is_none() {
            target_path = Some(node.file.clone()).filter(|path| path.ends_with(".cs"));
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

/// Read a candidate/target source through the trusted-path gate.
///
/// Returns the resolved path that was actually read beside the text: verification
/// reports the file it verified against, uses that same canonical path as its
/// parse-memo key, and never has to re-derive the identity later.
fn read_trusted_source(
    state: &McpState,
    path: &str,
    workspace_root: Option<&str>,
) -> Option<(String, String)> {
    let resolved = crate::mcp::tool_helpers::resolve_file_path_checked(
        path,
        workspace_root,
        &state.config.additional_roots,
    )
    .ok()?;
    let source = state.read_source(&resolved).ok()?;
    Some((resolved, source.to_string()))
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
