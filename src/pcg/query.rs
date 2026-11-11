//! Read-only acquisition over one immutable projection. MCP/token projection is later work.
pub use super::selectors::{Declaration, Subject};
use super::{
    Error, Result, closure, identity,
    profile::{Trusted, Work},
    provenance,
    query_rows::Reader,
    query_witnesses, require, security,
    selectors::{self, Truth},
    store,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Constraints,
    Closure,
    Impact,
    Provenance,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub records: u64,
    pub edges: u64,
    pub evidence_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub workspace_root: PathBuf,
    pub corpus: String,
    pub generation: String,
    pub mode: Mode,
    pub subject: Subject,
    pub records: Vec<String>,
    pub include_history: bool,
    pub limits: Limits,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Ready,
    Incomplete,
    Blocked,
    Unavailable,
}

#[derive(Debug, Default, Serialize)]
pub struct Coverage {
    pub enrollment: bool,
    pub identity: bool,
    pub applicability: bool,
    pub required_closure: bool,
    pub evidence: bool,
    pub working_start: bool,
    pub working_completion: bool,
}

#[derive(Debug, Default, Serialize)]
pub struct Counts {
    pub records: u64,
    pub selectors: u64,
    pub bindings: u64,
    pub relations: u64,
    pub refs: u64,
    pub metadata_bytes: u64,
    pub evidence_validations: u64,
    pub witness_bytes: u64,
}

/// Acquisition readiness is not context completeness or authorization to execute.
#[derive(Debug, Serialize)]
pub struct Acquisition {
    pub state: State,
    pub generation: String,
    pub coverage: Coverage,
    pub reasons: BTreeSet<String>,
    pub required_records: Vec<Value>,
    pub relationships: Vec<Value>,
    pub citations: Vec<Value>,
    pub counts: Counts,
    pub counts_disclosed: bool,
    pub context_complete: bool,
    pub continuous_currentness: bool,
    pub atomic_filesystem_snapshot: bool,
    pub projection_integrity: &'static str,
}

impl Acquisition {
    pub(super) fn new(generation: String) -> Self {
        Self {
            state: State::Incomplete,
            generation,
            coverage: Coverage::default(),
            reasons: BTreeSet::new(),
            required_records: vec![],
            relationships: vec![],
            citations: vec![],
            counts: Counts::default(),
            counts_disclosed: true,
            context_complete: false,
            continuous_currentness: false,
            atomic_filesystem_snapshot: false,
            projection_integrity: "admission_and_accessed_rows",
        }
    }
    pub(super) fn issue(&mut self, code: &str, blocked: bool) {
        self.reasons.insert(code.to_owned());
        if blocked {
            self.state = State::Blocked;
        }
    }
}

fn validate(trusted: &Trusted, q: &Request) -> Result<()> {
    require(q.workspace_root.is_absolute(), "scope_refused")?;
    security::no_links(&q.workspace_root, false)?;
    require(
        q.workspace_root
            .canonicalize()
            .map_err(|_| Error("scope_refused"))?
            == trusted.root
            && q.subject.workspace == trusted.profile.workspace_id,
        "scope_refused",
    )?;
    identity::id(&q.corpus)?;
    identity::hex(&q.generation, 64)?;
    q.subject.validate()?;
    require(
        (q.mode == Mode::Constraints) == q.records.is_empty(),
        "invalid_roots",
    )?;
    let mut ids = BTreeSet::new();
    for id in &q.records {
        identity::id(id)?;
        require(ids.insert(id), "duplicate_id")?;
    }
    let p = &trusted.profile.limits;
    require(
        q.limits.records > 0
            && q.limits.records <= p.records
            && q.limits.edges > 0
            && q.limits.edges <= p.edges
            && q.limits.evidence_bytes > 0
            && q.limits.evidence_bytes <= p.evidence_bytes,
        "request_limits",
    )?;
    require(
        !q.include_history || trusted.profile.history_allowed,
        "history_denied",
    )
}

/// Consumes only existing read-only rows. Never imports, repairs, hydrates or flushes.
pub fn acquire(profile_path: &Path, request: &Request) -> Result<Acquisition> {
    acquire_with(profile_path, request, &mut |_, _, _, _, _| Ok(()))
}

/// Operation-local packing seam; the transaction and witnesses remain owned here.
pub(super) type Packing<'a> = dyn FnMut(
        &mut Reader<'_>,
        &mut query_witnesses::Witnesses,
        &mut Work,
        &closure::Proof,
        &mut Acquisition,
    ) -> Result<()>
    + 'a;

pub(super) fn acquire_with(
    profile_path: &Path,
    request: &Request,
    packing: &mut Packing<'_>,
) -> Result<Acquisition> {
    let trusted = Trusted::load(profile_path)?;
    validate(&trusted, request)?;
    let mut result = Acquisition::new(request.generation.clone());
    let outcome = read(&trusted, request, &mut result, packing);
    if let Err(error) = outcome {
        if error.0 == "work_limit" {
            result.coverage.evidence = false;
            result.issue("work_limit", false);
        } else if [
            "source_stale",
            "source_missing",
            "source_inaccessible",
            "source_invalid",
            "history_required",
        ]
        .contains(&error.0)
        {
            result.coverage.evidence = false;
            if result.coverage.working_start && !result.coverage.working_completion {
                result.issue("source_changed_during_read", false);
            }
            result.issue(error.0, true);
        } else {
            result.state = State::Unavailable;
            let reason = match error.0 {
                "generation_mismatch" => "generation_mismatch",
                "profile_changed" => "profile_changed",
                _ => "projection_unavailable",
            };
            result.issue(reason, false);
        }
    }
    if result.state != State::Ready {
        result.required_records.clear();
        result.relationships.clear();
        result.citations.clear();
    }
    if result.state != State::Ready || !trusted.profile.history_allowed {
        result.counts = Counts::default();
        result.counts_disclosed = false;
    }
    Ok(result)
}

fn read(
    trusted: &Trusted,
    q: &Request,
    result: &mut Acquisition,
    packing: &mut Packing<'_>,
) -> Result<()> {
    trusted.recheck()?;
    // Failure to open the projection is not failure of an enrolled source witness.
    let connection =
        store::reader(trusted, &q.corpus).map_err(|_| Error("projection_unavailable"))?;
    require(
        store::head(&connection, trusted, &q.corpus)? == Some(q.generation.clone()),
        "generation_mismatch",
    )?;
    let mut reader = Reader::new(&connection, trusted, q);
    let mut work = Work::new(q.limits.evidence_bytes);
    let outcome = (|| {
        result.coverage.identity = q.subject.entity.is_none();
        if !result.coverage.identity {
            result.issue("identity_unavailable", false);
        }
        let complete = reader.enrollment_complete()?;
        let mut matched_coverage = false;
        let mut applicability = BTreeMap::new();
        let mut bindings = BTreeMap::new();
        for selector in reader.candidates()? {
            let truth = selectors::evaluate(&selector, &q.subject);
            if selector.purpose == "coverage" {
                matched_coverage |= truth == Truth::True;
            } else {
                for edge in reader.bindings(&selector.id)? {
                    let record = reader.record(&edge.from)?;
                    require(record.revision == edge.from_revision, "projection_corrupt")?;
                    let entry = applicability
                        .entry(edge.from.clone())
                        .or_insert(Truth::False);
                    *entry = entry.union(truth);
                    if truth == Truth::True {
                        bindings.insert(edge.id.clone(), edge);
                    }
                }
            }
        }
        result.coverage.enrollment = complete && matched_coverage;
        if !complete {
            result.issue("enrollment_incomplete", false);
        }
        if !matched_coverage {
            result.issue("scope_not_enrolled", false);
        }
        let mut roots = BTreeSet::new();
        for (id, truth) in &applicability {
            let record = reader.record(id)?;
            if record.data.required && record.data.lifecycle == "active" {
                match truth {
                    Truth::True => {
                        roots.insert(id.clone());
                    }
                    Truth::Unknown => result.issue("applicability_unknown", false),
                    Truth::False => (),
                }
            }
        }
        result.coverage.applicability = !result.reasons.contains("applicability_unknown");
        for id in &q.records {
            if !reader.exists(id)? {
                result.coverage.identity = false;
                result.issue("record_not_enrolled", false);
                continue;
            }
            let record = reader.record(id)?;
            let primary = reader.evidence(&record.data.primary)?;
            let historical_source = reader.source(&primary.source)?.data.check == "historical";
            if (record.data.lifecycle != "active" || historical_source) && !q.include_history {
                result.issue("history_required", true);
                continue;
            }
            roots.insert(id.clone());
        }
        let proof = closure::collect(&mut reader, &roots, &applicability, &bindings, result)?;
        result.coverage.required_closure =
            result.coverage.identity && result.coverage.applicability;
        let mut witnesses = query_witnesses::start(&mut reader, &proof.evidence, &mut work)?;
        result.coverage.working_start = true;
        result.coverage.evidence = true;
        provenance::materialize(&mut reader, &proof, &witnesses, &mut work, result)?;
        packing(&mut reader, &mut witnesses, &mut work, &proof, result)?;
        query_witnesses::complete(&reader, &witnesses, &mut work)?;
        result.coverage.working_completion = true;
        if result.reasons.is_empty() {
            result.state = State::Ready;
        }
        Ok(())
    })();
    result.counts = reader.counts;
    result.counts.witness_bytes = work.bytes;
    result.counts.evidence_validations = work.evidence_validations;
    outcome
}
