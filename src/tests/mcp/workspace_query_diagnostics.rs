// Sparse LLM-facing discovery diagnostics for workspace_query (RED-DIAG1..14).
//
// The defect these regressions pin: every hydration-bearing response used to
// serialize ten flat diagnostic fields — ~900 characters of expected state on
// an ordinary successful query — that told the caller nothing beyond "discovery
// worked". The projection under test (`query::diagnostics::discovery_field`)
// omits expected state and reports only decision-relevant deviation, as ONE
// optional `discovery` object, and omits that object entirely when discovery
// followed the expected path.
//
// Every assertion is structural (serialized content, presence/absence), and the
// sizes are measured in serialized characters plus real tokenizer counts — the
// repository tokenizer, never wall-clock timing.

use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::{
    HYDRATION_MAX_PROJECT_COVERAGE, HydrationReport, ProjectCoverage,
    TEST_PROJECT_HYDRATION_SERIALIZE, set_test_project_search_results,
};
use crate::mcp::tools::dispatch_tools_call;
use crate::tokenizer::{TokenizerKind, create_tokenizer};
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::path::Path;

/// The ten flat diagnostic keys the pre-fix handlers emitted unconditionally on
/// every hydration-bearing response. None of them may appear again.
const LEGACY_FLAT_DIAGNOSTIC_KEYS: &[&str] = &[
    "hydration_attempted",
    "discovery_provider",
    "discovery_status",
    "discovery_completed",
    "fallback_occurred",
    "fallback_reason",
    "candidates_discovered",
    "candidates_compiled",
    "project_coverage",
    "project_coverage_truncated",
];

/// The discovery regressions share process-global injection state.
fn serialize_tests() -> std::sync::MutexGuard<'static, ()> {
    TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn coverage(
    project: &str,
    status: &'static str,
    readiness: Option<&'static str>,
    reason: Option<&'static str>,
) -> ProjectCoverage {
    ProjectCoverage {
        project: project.to_string(),
        status,
        readiness,
        reason,
    }
}

/// The expected report of an ordinary successful query: healthy CBM completed
/// discovery, every configured project searched while ready, no candidates, no
/// fallback. Nothing in it is decision-relevant beyond the query answer.
fn steady_state_report() -> HydrationReport {
    HydrationReport {
        hydration_attempted: true,
        discovery_provider: "cbm",
        discovery_status: "completed",
        project_coverage: vec![
            coverage("CoreDataApi", "searched", Some("ready"), None),
            coverage("SharedLib", "searched", Some("ready"), None),
        ],
        ..HydrationReport::default()
    }
}

/// The pre-fix flat diagnostic block, reproduced exactly as the old handlers
/// emitted it, so the reduction is measured against the real defect rather than
/// an estimate. Two of those fields no longer exist internally; they are
/// reconstructed faithfully from their proven equivalence:
/// `discovery_completed` was `discovery_status == "completed"`, and
/// `fallback_occurred` was `fallback_reason.is_some()`. The old
/// `project_coverage_truncated` was `project_coverage.len() > 16` at report
/// construction time.
fn legacy_flat_diagnostics(report: &HydrationReport) -> Value {
    json!({
        "hydration_attempted": report.hydration_attempted,
        "discovery_provider": report.discovery_provider,
        "discovery_status": report.discovery_status,
        "discovery_completed": report.discovery_status == "completed",
        "fallback_occurred": report.fallback_reason.is_some(),
        "fallback_reason": report.fallback_reason,
        "candidates_discovered": report.candidates_discovered,
        "candidates_compiled": report.candidates_compiled,
        "project_coverage": report.project_coverage,
        "project_coverage_truncated": report.project_coverage.len() > 16,
    })
}

/// Serialized diagnostic characters of a diagnostic value (`0` when the whole
/// diagnostic is absent — the meaning of "nothing noteworthy happened").
fn diagnostic_chars(value: &Option<Value>) -> usize {
    value
        .as_ref()
        .map(|value| value.to_string().len())
        .unwrap_or(0)
}

/// Token count of a diagnostic value through the repository tokenizer, so the
/// reported reduction is measured, not extrapolated from characters.
fn diagnostic_tokens(value: &Option<Value>) -> usize {
    match value {
        Some(value) => {
            let tokenizer = create_tokenizer(TokenizerKind::O200k).expect("o200k tokenizer");
            tokenizer.count_tokens(&value.to_string())
        }
        // Absent diagnostics cost nothing; never load a tokenizer to prove it.
        None => 0,
    }
}

/// Dispatch `workspace_query` through the real MCP entry point and return its
/// `structuredContent`.
fn structured(state: &McpState, arguments: Value) -> Map<String, Value> {
    let params = json!({ "arguments": arguments });
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), "workspace_query", &params, state);
    let mut captured = crate::protocol::captured_responses();
    let response = captured.pop();
    drop(captured);
    let response = response.expect("workspace_query must send a response");
    let result = response["result"]
        .as_object()
        .expect("workspace_query result object");
    crate::tests::assert_valid_mcp_envelope(result);
    result["structuredContent"]
        .as_object()
        .expect("structuredContent")
        .clone()
}

/// A workspace whose single configured CBM project answers with zero candidates,
/// so hydration completes through CBM alone and reports nothing noteworthy: the
/// exact steady state the previous ten flat fields described.
fn steady_state_workspace(root: &Path) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        root,
        &[],
    ));
    let slug = cbm_project_slug(&root.canonicalize().expect("test root canonicalizes"));
    set_test_project_search_results(HashMap::from([(slug, Ok(Vec::new()))]));
    state
}

/// Seed the WorkspaceIndex exactly as the production write path leaves it: real
/// Clean-CTX-authored edges whose asserting file lies inside the workspace, plus
/// one real call fact so the semantic payload carries call evidence.
fn seed_index(state: &McpState, root: &Path) {
    let controller = root
        .join("UserController.java")
        .to_string_lossy()
        .into_owned();
    let service = root.join("UserService.java").to_string_lossy().into_owned();
    let mut index = state.workspace_index_lock();
    index.add_edges(
        &controller,
        vec![SemanticEdge {
            relation: SemanticRelation::Autowired,
            subject: EntityRef::new("spring", "Controller", "UserController")
                .with_file(controller.clone()),
            object: EntityRef::new("spring", "Service", "UserService")
                .with_file(controller.clone()),
            layer: "spring",
            call_evidence: None,
        }],
    );
    index.add_edges(
        &service,
        vec![
            SemanticEdge {
                relation: SemanticRelation::EndpointMapsTo,
                subject: EntityRef::new("spring", "Service", "UserService")
                    .with_file(service.clone()),
                object: EntityRef::new("spring", "Endpoint", "/api/users")
                    .with_file(service.clone()),
                layer: "spring",
                call_evidence: None,
            },
            SemanticEdge {
                relation: SemanticRelation::Calls,
                subject: EntityRef::new("builtin", "Method", "OrderByCaller")
                    .with_file(service.clone()),
                object: EntityRef::new("builtin", "Method", "OrderBy").with_file(service.clone()),
                layer: "builtin",
                call_evidence: Some(CallEvidence::new(2, false)),
            },
        ],
    );
}

/// A steady-state workspace with a seeded index; the caller keeps the returned
/// guard alive so the workspace root outlives the queries.
fn seeded_steady_state() -> (McpState, tempfile::TempDir) {
    let root = tempfile::TempDir::new().expect("temp workspace root");
    let state = steady_state_workspace(root.path());
    seed_index(&state, root.path());
    (state, root)
}

/// The projected `discovery` object, or `None` when the response carries none.
fn discovery_of(sc: &Map<String, Value>) -> Option<&Value> {
    sc.get("discovery")
}

/// No response may serialize any of the ten pre-fix flat diagnostic fields.
fn assert_no_legacy_flat_diagnostics(sc: &Map<String, Value>) {
    for key in LEGACY_FLAT_DIAGNOSTIC_KEYS {
        assert!(
            sc.get(*key).is_none(),
            "flat diagnostic '{key}' must no longer be serialized: {sc:?}"
        );
    }
}

// ── RED-DIAG1: steady-state diagnostics are omitted entirely ────────────

#[test]
fn red_diag1_steady_state_diagnostics_omitted() {
    let report = steady_state_report();
    let projected = super::discovery_field(&report);

    assert!(
        projected.is_none(),
        "an expected discovery outcome must produce NO discovery key: {projected:?}"
    );
    assert_eq!(
        diagnostic_chars(&projected),
        0,
        "no diagnostic characters at all"
    );
    assert_eq!(diagnostic_tokens(&projected), 0);

    // Omitted, not lost: the internal record still describes the whole pass.
    assert!(report.hydration_attempted);
    assert_eq!(report.discovery_status, "completed");
    assert_eq!(
        report.project_coverage.len(),
        2,
        "healthy coverage stays available internally"
    );

    // Pre-fix baseline for the very same report, measured identically.
    let legacy = Some(legacy_flat_diagnostics(&report));
    let legacy_chars = diagnostic_chars(&legacy);
    let legacy_tokens = diagnostic_tokens(&legacy);
    assert!(
        legacy_chars > 0 && legacy_tokens > 0,
        "the pre-fix block was non-empty: {legacy_chars} chars / {legacy_tokens} tokens"
    );
}

// ── RED-DIAG2: a non-CBM provider is surfaced, alone ────────────────────

#[test]
fn red_diag2_fallback_provider_surfaced() {
    let report = HydrationReport {
        discovery_provider: "cbm_and_filesystem",
        ..steady_state_report()
    };
    let discovery = super::discovery_field(&report).expect("a provider deviation is reported");
    assert_eq!(
        discovery,
        json!({ "provider": "cbm_and_filesystem" }),
        "only the deviating field appears; no default value joins it"
    );
    assert!(
        discovery.get("status").is_none(),
        "completion is not restated"
    );
    assert!(
        discovery.get("fallback").is_none(),
        "the redundant fallback boolean is gone"
    );
    assert!(discovery.get("discovered").is_none());
    assert!(discovery.get("compiled").is_none());
    assert!(discovery.get("projects").is_none());
}

// ── RED-DIAG3: the fallback reason survives verbatim ───────────────────

#[test]
fn red_diag3_fallback_reason_surfaced() {
    for reason in [
        "cbm_unavailable",
        "cbm_discovery_failed",
        "cbm_partial_failure",
        "cbm_scope_unavailable",
        "filesystem_unavailable",
    ] {
        let report = HydrationReport {
            fallback_reason: Some(reason),
            ..steady_state_report()
        };
        let discovery = super::discovery_field(&report).expect("fallback is reported");
        assert_eq!(
            discovery,
            json!({ "fallback_reason": reason }),
            "the exact code survives, and its presence IS the fallback fact"
        );
    }

    // No fallback → no reason, and no boolean restating its absence.
    assert!(super::discovery_field(&steady_state_report()).is_none());
}

// ── RED-DIAG4: candidate counts appear only when non-zero ──────────────

#[test]
fn red_diag4_candidate_counts_surfaced() {
    let both = HydrationReport {
        candidates_discovered: 5,
        candidates_compiled: 3,
        ..steady_state_report()
    };
    assert_eq!(
        super::discovery_field(&both).expect("counts are reported"),
        json!({ "discovered": 5, "compiled": 3 }),
        "both counts survive; zero-valued siblings stay omitted"
    );

    // `compiled` is NOT derivable from `discovered`: deduplication,
    // already-indexed exclusion and compilation failure all make it smaller, so a
    // discovered-only outcome must keep its count.
    let discovered_only = HydrationReport {
        candidates_discovered: 5,
        ..steady_state_report()
    };
    assert_eq!(
        super::discovery_field(&discovered_only).expect("reported"),
        json!({ "discovered": 5 })
    );
}

// ── RED-DIAG5: only exceptional projects are returned, minimized ───────

#[test]
fn red_diag5_exceptional_project_retained() {
    let report = HydrationReport {
        project_coverage: vec![
            coverage("HealthyOne", "searched", Some("ready"), None),
            coverage(
                "Failing",
                "search_failed",
                Some("ready"),
                Some("search_failed"),
            ),
            coverage("HealthyTwo", "searched", Some("ready"), None),
        ],
        ..steady_state_report()
    };
    let discovery = super::discovery_field(&report).expect("a failed project is reported");
    assert_eq!(
        discovery["projects"],
        json!([{ "project": "Failing", "status": "search_failed" }]),
        "healthy projects disappear; a failed search does not restate its status as a reason"
    );
    assert!(
        discovery["projects"]
            .as_array()
            .expect("projects array")
            .iter()
            .all(Value::is_object),
        "exceptional coverage stays structured, never a prose string"
    );
}

// ── RED-DIAG6: distinct readiness / reason states are not collapsed ────

#[test]
fn red_diag6_readiness_distinction_retained() {
    let report = HydrationReport {
        project_coverage: vec![
            coverage("Healthy", "searched", Some("ready"), None),
            coverage("ReadinessFailed", "searched", Some("failed"), None),
            coverage("StillIndexing", "searched", Some("still_indexing"), None),
            coverage("Unavailable", "skipped", None, Some("cbm_unavailable")),
            coverage(
                "Unregistered",
                "skipped",
                None,
                Some("additional_root_not_registered"),
            ),
        ],
        ..steady_state_report()
    };
    let discovery = super::discovery_field(&report).expect("reported");
    assert_eq!(
        discovery["projects"],
        json!([
            { "project": "ReadinessFailed", "status": "searched", "readiness": "failed" },
            { "project": "StillIndexing", "status": "searched", "readiness": "still_indexing" },
            { "project": "Unavailable", "status": "skipped", "reason": "cbm_unavailable" },
            {
                "project": "Unregistered",
                "status": "skipped",
                "reason": "additional_root_not_registered"
            },
        ]),
        "searched-but-not-ready and skipped-for-different-reasons stay distinct and structured"
    );
}

// ── RED-DIAG7: exceptional-only truncation, reported structurally ──────

#[test]
fn red_diag7_project_truncation_is_structured_and_exception_only() {
    let mut entries: Vec<ProjectCoverage> = (0..3)
        .map(|index| {
            coverage(
                &format!("Healthy{index:02}"),
                "searched",
                Some("ready"),
                None,
            )
        })
        .collect();
    entries.extend((0..(HYDRATION_MAX_PROJECT_COVERAGE + 4)).map(|index| {
        coverage(
            &format!("Failed{index:02}"),
            "search_failed",
            Some("ready"),
            Some("search_failed"),
        )
    }));
    let report = HydrationReport {
        project_coverage: entries,
        ..steady_state_report()
    };

    let discovery = super::discovery_field(&report).expect("reported");
    let projects = discovery["projects"].as_array().expect("projects array");
    assert_eq!(
        projects.len(),
        HYDRATION_MAX_PROJECT_COVERAGE,
        "only the bound's worth of exceptional entries is reported"
    );
    assert_eq!(
        discovery["projects_truncated"],
        json!(4),
        "the overflow is a structured count, never a '+4 more' string inside the array"
    );
    assert!(
        projects
            .iter()
            .all(|entry| entry["status"] == "search_failed"),
        "healthy projects never consume the exceptional budget"
    );

    // Healthy coverage alone is never a diagnostic, however much of it there is.
    let healthy_only = HydrationReport {
        project_coverage: (0..(HYDRATION_MAX_PROJECT_COVERAGE + 4))
            .map(|index| {
                coverage(
                    &format!("Healthy{index:02}"),
                    "searched",
                    Some("ready"),
                    None,
                )
            })
            .collect(),
        ..steady_state_report()
    };
    assert!(super::discovery_field(&healthy_only).is_none());
}

// ── RED-DIAG8: `discovery_completed` is gone; status is the signal ─────

#[test]
fn red_diag8_discovery_completed_is_gone_status_is_the_signal() {
    let partial = HydrationReport {
        discovery_status: "partial",
        ..steady_state_report()
    };
    let discovery = super::discovery_field(&partial).expect("partial discovery is reported");
    assert_eq!(discovery, json!({ "status": "partial" }));
    assert!(discovery.get("discovery_completed").is_none());
    assert!(discovery.get("completed").is_none());
    assert!(
        discovery.get("attempted").is_none(),
        "the removed attempted boolean is folded into the status word"
    );

    // "Nothing ran at all" is the `unavailable` status — one field, not a status
    // plus a boolean restating it.
    let unavailable = HydrationReport {
        hydration_attempted: false,
        discovery_provider: "none",
        discovery_status: "failed",
        fallback_reason: Some("filesystem_unavailable"),
        ..steady_state_report()
    };
    assert_eq!(
        super::discovery_field(&unavailable).expect("reported"),
        json!({
            "provider": "none",
            "status": "unavailable",
            "fallback_reason": "filesystem_unavailable"
        })
    );

    // A completed pass reports no status at all.
    assert!(super::discovery_field(&steady_state_report()).is_none());
}

// The response-level half of these regressions drives the real MCP dispatch
// path (`workspace_query_diagnostics_response.rs`). It is a child module of this
// test module, so `use super::*` inherits every fixture and helper above.
#[path = "workspace_query_diagnostics_response.rs"]
mod response;
