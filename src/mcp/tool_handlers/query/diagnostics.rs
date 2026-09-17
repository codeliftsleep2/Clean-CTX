// src/mcp/tool_handlers/query/diagnostics.rs
//
// The LLM-facing projection of the hydration discovery diagnostics.
//
// `HydrationReport` is deliberately rich: it records everything the discovery
// pass did, for internal debugging, telemetry and the discovery regressions.
// Almost all of that is EXPECTED state on an ordinary successful query, and
// serializing it on every answer spent context the caller never needed — ten
// flat fields (~900 characters) of "everything went normally" on every
// `find_entities` / `forward_edges` / `reverse_edges` / `transitive_dependencies`
// response.
//
// The projection therefore inverts the default: expected state is omitted and
// only deviation is reported, as ONE optional `structuredContent.discovery`
// object. There is no empty object — the key is ABSENT when discovery followed
// the expected path, and absence means exactly that.
//
//   absent                  discovery ran as expected (healthy CBM, completed,
//                           ready projects, no candidates)
//   { "provider": ... }     a provider other than CBM supplied coverage
//   { "status": ... }       discovery did not complete across every root
//   { "fallback_reason": }  filesystem fallback was engaged for at least one root
//   { "discovered": 5 }     this query discovered candidate files
//   { "compiled": 3 }       this query compiled candidate files
//   { "projects": [ ... ] } ONLY the projects whose coverage was not
//                           "searched while ready"
//
// Minimum-state rule. A field is serialized only when it is BOTH exceptional
// and not inferable from the fields already present:
//
//   field             emitted when                 why not inferable
//   -------------     ------------                 -----------------
//   provider          != "cbm"                     `fallback_reason` says why the
//                                                  fallback was engaged, never who
//                                                  supplied coverage: CBM can cover
//                                                  every project it owns while
//                                                  another root still falls back
//                                                  (provider "cbm" + a reason), and
//                                                  "none" reports that nothing
//                                                  covered the query at all
//   status            != complete                  the single completeness signal
//                                                  (partial = some coverage ran,
//                                                  unavailable = nothing ran); no
//                                                  separate boolean restates it
//   fallback_reason   present                      the compact event + its cause
//   discovered        > 0                          candidates found
//   compiled          > 0                          candidates compiled; NOT derivable
//                                                  from `discovered` (deduplication,
//                                                  already-indexed exclusion and
//                                                  compilation failure all make it
//                                                  strictly smaller)
//   projects          non-empty                    which project, and whether its
//                                                  contribution may be incomplete
//   projects_truncated > 0                         exceptional entries beyond the bound
//
// Removed from the previous flat shape:
//
//   hydration_attempted        -> folded into `status`: nothing-ran is exactly the
//                                 `unavailable` state, so no boolean restates it
//   discovery_provider         -> `provider`
//   discovery_status           -> `status` (compact external vocabulary:
//                                 complete / partial / unavailable)
//   discovery_completed        -> REMOVED: provably redundant with
//                                 `discovery_status` (see `hydration.rs`)
//   fallback_occurred          -> REMOVED: `fallback_reason.is_some()` was exactly
//                                 equivalent on every discovery return path, and
//                                 the reason is the strictly more informative
//                                 encoding of the same event
//   fallback_reason            -> `fallback_reason`
//   candidates_discovered      -> `discovered`
//   candidates_compiled        -> `compiled`
//   project_coverage           -> `projects`, exceptional entries only, each
//                                 minimized (see `ProjectDiagnostics`)
//   project_coverage_truncated -> `projects_truncated`
//
// Exceptional project coverage stays STRUCTURED — distinctions such as
// searched/still_indexing, search_failed and skipped/additional_root_not_registered
// are never collapsed into prose — and `HYDRATION_MAX_PROJECT_COVERAGE` is applied
// to those exceptional entries only, with the overflow reported as a count.
//
// Project identity is deliberately left as the stable CBM project identity. There
// is no shorter already-existing display name for a configured project inside the
// report, and inventing one (a basename, say) can collide between two configured
// roots, so shortening it would require new identity semantics.

use super::super::hydration::{HYDRATION_MAX_PROJECT_COVERAGE, HydrationReport, ProjectCoverage};
use serde::Serialize;
use serde_json::Value;

/// The sparse, exception-only discovery diagnostic of one `workspace_query`
/// response.
///
/// Every field is optional and omitted while it holds its expected value, so
/// `DiscoveryDiagnostics::default()` — nothing to report — is never serialized.
#[derive(Default, PartialEq, Eq, Serialize)]
pub(crate) struct DiscoveryDiagnostics {
    /// Expected `"cbm"`; only another provider is reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<&'static str>,
    /// Expected complete; only `partial`/`unavailable` are reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<&'static str>,
    /// Absent while CBM completed discovery for every root. Its presence is also
    /// the one encoding of "filesystem fallback was engaged"; the redundant
    /// `fallback_occurred` boolean is gone.
    #[serde(skip_serializing_if = "Option::is_none")]
    fallback_reason: Option<&'static str>,
    /// Reported only when this query actually discovered candidates.
    #[serde(skip_serializing_if = "Option::is_none")]
    discovered: Option<usize>,
    /// Reported only when this query actually compiled candidates.
    #[serde(skip_serializing_if = "Option::is_none")]
    compiled: Option<usize>,
    /// Exceptional project coverage only, each minimized, bounded by
    /// `HYDRATION_MAX_PROJECT_COVERAGE`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    projects: Vec<ProjectDiagnostics>,
    /// Exceptional entries dropped by that bound; never emitted when zero.
    #[serde(skip_serializing_if = "Option::is_none")]
    projects_truncated: Option<usize>,
}

/// The minimized LLM-facing coverage entry of ONE exceptional project.
///
/// This is not the internal `ProjectCoverage`: the internal record keeps the
/// full observed readiness/reason of every project, while this entry answers only
/// the questions a caller can act on — which project was affected, what happened,
/// and whether its contribution may be incomplete.
#[derive(Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ProjectDiagnostics {
    project: String,
    status: &'static str,
    /// Present only for a completed search on a project that was not ready: the
    /// search's result may be incomplete. For a failed or skipped search the
    /// readiness changes no conclusion and is dropped.
    #[serde(skip_serializing_if = "Option::is_none")]
    readiness: Option<&'static str>,
    /// Present only when it adds a distinction the status does not already carry
    /// (a failed search used to repeat its status verbatim as its reason).
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<&'static str>,
}

impl DiscoveryDiagnostics {
    /// Project a completed hydration report onto its LLM-facing shape.
    ///
    /// `None` means "discovery followed the expected path": the caller emits no
    /// `discovery` key at all. A report that never ran discovery
    /// (`HydrationReport::default()`, the report of a non-hydration-eligible
    /// surface) projects to `None` too — it carries an empty provider because
    /// there is no discovery outcome to describe.
    fn from_report(report: &HydrationReport) -> Option<Self> {
        if report.discovery_provider.is_empty() {
            return None;
        }
        let mut projects: Vec<ProjectDiagnostics> = report
            .project_coverage
            .iter()
            .filter(|entry| !is_expected_coverage(entry))
            .map(project_diagnostics)
            .collect();
        // Bound the EXCEPTIONAL entries: a healthy project must not consume the
        // diagnostic budget of an exceptional one.
        let projects_truncated = projects
            .len()
            .saturating_sub(HYDRATION_MAX_PROJECT_COVERAGE);
        projects.truncate(HYDRATION_MAX_PROJECT_COVERAGE);

        let diagnostics = Self {
            provider: (report.discovery_provider != "cbm").then_some(report.discovery_provider),
            status: completeness_status(report),
            fallback_reason: report.fallback_reason,
            discovered: (report.candidates_discovered > 0).then_some(report.candidates_discovered),
            compiled: (report.candidates_compiled > 0).then_some(report.candidates_compiled),
            projects,
            projects_truncated: (projects_truncated > 0).then_some(projects_truncated),
        };
        (diagnostics != Self::default()).then_some(diagnostics)
    }
}

/// The one compact completeness signal, or `None` when discovery completed
/// across every configured root (the expected state).
///
/// The internal discovery pass distinguishes `completed` / `partial` / `failed`,
/// where `failed` is exactly "nothing ran at all": `discovery_status` yields it
/// only when no operation was attempted, and every path that reports `completed`
/// or `partial` has attempted discovery. Projecting `failed` to `unavailable`
/// therefore loses no distinction and makes a separate `attempted` boolean
/// redundant — "nothing ran" is carried by the status word itself.
fn completeness_status(report: &HydrationReport) -> Option<&'static str> {
    if report.discovery_status == "completed" {
        None
    } else if report.hydration_attempted {
        Some("partial")
    } else {
        Some("unavailable")
    }
}

/// Expected, healthy coverage for one configured project: the search ran and the
/// project was ready, so nothing about this entry is decision-relevant. Any
/// other combination — a failed search, a skipped project, an unresolved
/// readiness, or a search that ran while the project was still indexing — is
/// exceptional and is reported.
fn is_expected_coverage(entry: &ProjectCoverage) -> bool {
    entry.status == "searched" && entry.readiness == Some("ready") && entry.reason.is_none()
}

/// Minimize one exceptional coverage entry.
fn project_diagnostics(entry: &ProjectCoverage) -> ProjectDiagnostics {
    ProjectDiagnostics {
        project: entry.project.clone(),
        status: entry.status,
        // A completed search on a project that was not ready is the one case
        // where readiness explains a possible incompleteness of the answer.
        readiness: if entry.status == "searched" {
            entry.readiness
        } else {
            None
        },
        // `search_failed` used to be repeated verbatim as its own reason.
        reason: entry.reason.filter(|reason| *reason != entry.status),
    }
}

/// The optional LLM-facing `discovery` value of a `workspace_query`
/// `structuredContent`.
///
/// `None` — the normal case — means discovery followed the expected path and
/// "nothing noteworthy happened"; the caller then emits no `discovery` key at
/// all (never an empty object). This is the ONE place that decides how much
/// discovery detail an LLM sees; every hydration-eligible handler shares it.
pub(crate) fn discovery_field(report: &HydrationReport) -> Option<Value> {
    DiscoveryDiagnostics::from_report(report)
        .and_then(|diagnostics| serde_json::to_value(diagnostics).ok())
}

#[cfg(all(test, feature = "rust"))]
#[path = "../../../tests/mcp/workspace_query_diagnostics.rs"]
mod tests_diagnostics;
