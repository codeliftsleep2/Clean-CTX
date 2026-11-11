//! Closed v1 enrollment types. Optional selectors are unrestricted when omitted.
use serde::{Deserialize, Serialize};

macro_rules! object {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, Deserialize, Serialize)]
        #[serde(deny_unknown_fields)]
        pub(super) struct $name { $(pub $field: $ty),* }
    };
}
object!(Limits {
    records: u64,
    edges: u64,
    evidence_bytes: u64,
    token_budget: u64
});
object!(Profile {
    schema_version: u32,
    partition_id: String,
    owner_label: String,
    workspace_id: String,
    workspace_root: String,
    database_path: String,
    manifest_path: String,
    approved_manifest_sha256: String,
    policy_revision: String,
    history_allowed: bool,
    limits: Limits,
});
object!(Manifest {
    schema_version: u32, parser_version: String, corpus_id: String, source_namespace: String,
    enrollment: Enrollment, sources: Vec<Source>, selectors: Vec<Selector>,
    evidence: Vec<Evidence>, records: Vec<Record>, edges: Vec<Edge>,
});
object!(Enrollment {
    owner_label: String,
    policy_revision: String,
    complete: bool
});
object!(Source {
    id: String,
    path: String,
    commit: String,
    blob: String,
    sha256: String,
    check: String
});
object!(Selector {
    id: String, purpose: String, workspace: String, file: Option<String>, entity: Option<Entity>,
    operation: Option<String>, task: Option<String>, from: Option<String>, until: Option<String>,
});
object!(Entity {
    kind: String,
    file: String,
    source_sha256: String,
    start: u64,
    end: u64,
    name: String,
    declaration_kind: String,
});
object!(Evidence { id: String, source: String, start: u64, end: u64, sha256: String, heading: Option<String> });
object!(Record {
    id: String, role: String, required: bool, primary: String, authority: Authority,
    verification: Verification, lifecycle: String, subject: String, disposition: String,
    evidence_refs: Vec<EvidenceRef>,
});
object!(Authority {
    class: String,
    attributed_to: String
});
object!(Verification {
    outcome: String,
    subject: String,
    execution: String
});
object!(EvidenceRef {
    id: String,
    required: bool
});
object!(Edge {
    id: String, kind: String, from: String, to: Option<String>, to_selector: Option<String>,
    evidence: String, gate: Option<Gate>,
});
object!(Gate {
    outcome: String,
    subject: String,
    authority: String
});

/// Exact row values, in approved DDL column order; used for insert and full audit.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Rows(
    pub std::collections::BTreeMap<&'static str, Vec<Vec<rusqlite::types::Value>>>,
);

pub(super) struct Generation {
    pub manifest: Manifest,
    pub id: String,
    pub rows: Rows,
}
