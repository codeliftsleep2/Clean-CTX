//! Whole-generation admission only; no query-time closure or applicability evaluator.
use super::{Error, Result, identity, model::*, require, security};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

fn member(v: &str, allowed: &[&str]) -> Result<()> {
    require(allowed.contains(&v), "manifest_value")
}
fn unique<'a>(ids: impl Iterator<Item = &'a String>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        identity::id(id)?;
        require(seen.insert(id), "duplicate_id")?;
    }
    Ok(())
}
pub(super) fn authority(v: &str) -> Result<usize> {
    [
        "hypothesis",
        "unverified_extraction",
        "attributable_observation",
        "verified_claim",
        "explicitly_approved_decision",
    ]
    .iter()
    .position(|c| *c == v)
    .ok_or(Error("authority_class"))
}
fn instant(v: &str) -> Result<()> {
    require(v.len() == 20 && v.ends_with('Z'), "selector_interval")?;
    let parsed = chrono::DateTime::parse_from_rfc3339(v).map_err(|_| Error("selector_interval"))?;
    require(
        parsed.format("%Y-%m-%dT%H:%M:%SZ").to_string() == v && !v.contains(":60"),
        "selector_interval",
    )
}
fn selector(s: &Selector, workspace: &str) -> Result<()> {
    member(&s.purpose, &["coverage", "applicability"])?;
    require(s.workspace == workspace, "selector_workspace")?;
    if let Some(file) = &s.file {
        security::relative(file)?;
    }
    for value in [&s.operation, &s.task].into_iter().flatten() {
        identity::id(value)?;
    }
    if let Some(entity) = &s.entity {
        require(
            entity.kind == "declaration"
                && entity.end > entity.start
                && entity.end <= i64::MAX as u64
                && !entity.name.is_empty()
                && !entity.declaration_kind.is_empty(),
            "selector_entity",
        )?;
        security::relative(&entity.file)?;
        identity::hex(&entity.source_sha256, 64)?;
        require(
            s.file.as_ref().is_none_or(|f| f == &entity.file),
            "selector_entity",
        )?;
    }
    for v in [&s.from, &s.until].into_iter().flatten() {
        instant(v)?;
    }
    require(
        !matches!((&s.from, &s.until), (Some(a), Some(b)) if a >= b),
        "selector_interval",
    )
}

pub(super) fn validate(m: &Manifest, p: &Profile) -> Result<()> {
    require(
        m.schema_version == 1 && m.parser_version == "pcg-json-v1",
        "parser_version",
    )?;
    identity::id(&m.corpus_id)?;
    identity::id(&m.source_namespace)?;
    require(
        m.enrollment.owner_label == p.owner_label
            && m.enrollment.policy_revision == p.policy_revision,
        "enrollment_policy",
    )?;
    unique(m.sources.iter().map(|s| &s.id))?;
    unique(m.selectors.iter().map(|s| &s.id))?;
    unique(m.evidence.iter().map(|s| &s.id))?;
    unique(m.records.iter().map(|s| &s.id))?;
    unique(m.edges.iter().map(|s| &s.id))?;
    require(
        (m.sources.len() + m.evidence.len() + m.records.len() + m.selectors.len()) as u64
            <= p.limits.records,
        "work_limit",
    )?;
    let ref_count: usize = m.records.iter().map(|r| r.evidence_refs.len()).sum();
    require(
        (m.edges.len() + ref_count) as u64 <= p.limits.edges,
        "work_limit",
    )?;
    let mut current_commit = None;
    let mut current_paths = BTreeMap::new();
    for s in &m.sources {
        security::relative(&s.path)?;
        identity::hex(&s.sha256, 64)?;
        member(&s.check, &["current", "historical"])?;
        if s.check == "current" {
            if let Some(commit) = current_commit {
                require(commit == &s.commit, "current_commit_conflict")?;
            }
            current_commit = Some(&s.commit);
            if let Some(hash) = current_paths.insert(&s.path, &s.sha256) {
                require(hash == &s.sha256, "current_alias_conflict")?;
            }
        }
    }
    require(current_commit.is_some(), "current_source_required")?;
    for s in &m.selectors {
        selector(s, &p.workspace_id)?;
    }
    require(
        m.selectors.iter().any(|s| s.purpose == "coverage"),
        "coverage_required",
    )?;
    let sources: BTreeSet<_> = m.sources.iter().map(|s| &s.id).collect();
    let evidence: BTreeSet<_> = m.evidence.iter().map(|e| &e.id).collect();
    for e in &m.evidence {
        require(
            sources.contains(&e.source) && e.end > e.start && e.end <= i64::MAX as u64,
            "evidence_source_span",
        )?;
        identity::hex(&e.sha256, 64)?;
        require(
            e.heading.as_ref().is_none_or(|h| !h.is_empty()),
            "evidence_heading",
        )?;
    }
    for r in &m.records {
        member(&r.role, &["claim", "decision", "constraint", "evidence"])?;
        authority(&r.authority.class)?;
        require(
            !r.authority.attributed_to.is_empty(),
            "authority_attribution",
        )?;
        member(
            &r.verification.outcome,
            &["unknown", "supported", "contradicted", "inconclusive"],
        )?;
        member(
            &r.verification.execution,
            &["not_run", "report_of_execution", "executed_artifact"],
        )?;
        member(
            &r.lifecycle,
            &["active", "superseded", "retracted", "falsified"],
        )?;
        for label in [&r.subject, &r.disposition, &r.verification.subject] {
            identity::id(label)?;
        }
        require(evidence.contains(&r.primary), "primary_missing")?;
        unique(r.evidence_refs.iter().map(|e| &e.id))?;
        for e in &r.evidence_refs {
            require(evidence.contains(&e.id), "required_ref_missing")?;
        }
    }
    relationships(m)?;
    conflicts(m)
}

fn relationships(m: &Manifest) -> Result<()> {
    let records: BTreeMap<_, _> = m.records.iter().map(|r| (&r.id, r)).collect();
    let selectors: BTreeMap<_, _> = m.selectors.iter().map(|s| (&s.id, s)).collect();
    let mut bound: BTreeMap<&String, Vec<&Selector>> = BTreeMap::new();
    for e in m.edges.iter().filter(|e| e.kind == "CONSTRAINS") {
        let selector = selectors
            .get(e.to_selector.as_ref().ok_or(Error("binding_target"))?)
            .ok_or(Error("binding_target"))?;
        bound.entry(&e.from).or_default().push(selector);
    }
    let evidence: BTreeSet<_> = m.evidence.iter().map(|e| &e.id).collect();
    let mut successors = BTreeSet::new();
    for e in &m.edges {
        let from = records.get(&e.from).ok_or(Error("edge_endpoint"))?;
        require(evidence.contains(&e.evidence), "edge_provenance")?;
        require(e.to.is_some() != e.to_selector.is_some(), "edge_endpoint")?;
        require((e.kind == "BLOCKED_BY") == e.gate.is_some(), "gate_schema")?;
        if e.kind == "CONSTRAINS" {
            require(
                ["decision", "constraint"].contains(&from.role.as_str()),
                "binding_role",
            )?;
            let s = selectors
                .get(e.to_selector.as_ref().ok_or(Error("binding_target"))?)
                .ok_or(Error("binding_target"))?;
            require(s.purpose == "applicability", "binding_target")?;
            continue;
        }
        require(e.to_selector.is_none(), "edge_endpoint")?;
        let to = records
            .get(e.to.as_ref().ok_or(Error("edge_endpoint"))?)
            .ok_or(Error("edge_endpoint"))?;
        let proposition =
            |r: &Record| ["decision", "constraint", "claim"].contains(&r.role.as_str());
        match e.kind.as_str() {
            "DEPENDS_ON" | "BLOCKED_BY" => {
                require(proposition(from) && proposition(to), "edge_role")?
            }
            "USES_EVIDENCE" => require(to.role == "evidence", "edge_role")?,
            "VALIDATES" | "FALSIFIES" => require(
                ["evidence", "claim"].contains(&from.role.as_str()) && proposition(to),
                "edge_role",
            )?,
            "SUPERSEDES" => {
                require(
                    proposition(from)
                        && from.role == to.role
                        && from.subject == to.subject
                        && to.lifecycle == "superseded"
                        && authority(&from.authority.class)? >= 2
                        && successors.insert(&to.id),
                    "supersession",
                )?;
                let old = bound.get(&to.id).map(Vec::as_slice).unwrap_or_default();
                let new = bound.get(&from.id).map(Vec::as_slice).unwrap_or_default();
                require(
                    !old.is_empty() && old.iter().all(|o| covered_by(new, o)),
                    "supersession_scope",
                )?;
            }
            "PRESERVES" | "IMPLEMENTS" => (),
            _ => return Err(Error("edge_kind")),
        }
        if let Some(gate) = &e.gate {
            require(
                gate.outcome == "supported" && authority(&gate.authority)? >= 2,
                "gate_schema",
            )?;
            identity::id(&gate.subject)?;
        }
    }
    for r in &m.records {
        require(
            r.lifecycle != "superseded" || successors.contains(&r.id),
            "supersession_successor",
        )?;
    }
    acyclic(m, &["DEPENDS_ON", "BLOCKED_BY"])?;
    acyclic(m, &["SUPERSEDES"])
}

fn acyclic(m: &Manifest, kinds: &[&str]) -> Result<()> {
    let mut degree: BTreeMap<_, usize> = m.records.iter().map(|r| (&r.id, 0)).collect();
    let mut next: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for e in m.edges.iter().filter(|e| kinds.contains(&e.kind.as_str())) {
        let to = e.to.as_ref().ok_or(Error("edge_endpoint"))?;
        *degree.get_mut(to).ok_or(Error("edge_endpoint"))? += 1;
        next.entry(&e.from).or_default().push(to);
    }
    let mut queue: VecDeque<_> = degree
        .iter()
        .filter(|(_, n)| **n == 0)
        .map(|(id, _)| *id)
        .collect();
    let mut visited = 0;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        for to in next.get(id).into_iter().flatten() {
            let n = degree.get_mut(to).ok_or(Error("edge_endpoint"))?;
            *n -= 1;
            if *n == 0 {
                queue.push_back(*to);
            }
        }
    }
    require(visited == degree.len(), "required_cycle")
}

fn dimensions(s: &Selector) -> [Option<String>; 4] {
    [
        s.file
            .clone()
            .or_else(|| s.entity.as_ref().map(|e| e.file.clone())),
        s.entity
            .as_ref()
            .map(|e| identity::canonical(&serde_json::to_value(e).expect("entity"))),
        s.operation.clone(),
        s.task.clone(),
    ]
}
fn covered_by(new: &[&Selector], old: &Selector) -> bool {
    let old_dimensions = dimensions(old);
    let mut intervals: Vec<_> = new
        .iter()
        .filter(|s| {
            dimensions(s)
                .iter()
                .zip(&old_dimensions)
                .all(|(a, b)| a.is_none() || a == b)
        })
        .collect();
    intervals.sort_by(|a, b| a.from.cmp(&b.from));
    let mut cursor = old.from.as_ref();
    for interval in intervals {
        if interval
            .from
            .as_ref()
            .is_some_and(|start| cursor.is_none_or(|end| start > end))
        {
            return false;
        }
        let Some(end) = interval.until.as_ref() else {
            return true;
        };
        if cursor.is_none_or(|previous| end > previous) {
            cursor = Some(end);
        }
        if old
            .until
            .as_ref()
            .is_some_and(|required| cursor.is_some_and(|end| end >= required))
        {
            return true;
        }
    }
    false
}
fn overlaps(a: &Selector, b: &Selector) -> bool {
    dimensions(a)
        .iter()
        .zip(dimensions(b).iter())
        .all(|(a, b)| a.is_none() || b.is_none() || a == b)
        && !matches!((&a.until, &b.from), (Some(end), Some(start)) if end <= start)
        && !matches!((&b.until, &a.from), (Some(end), Some(start)) if end <= start)
}
fn conflicts(m: &Manifest) -> Result<()> {
    // Construct binding ownership once, rather than rebuild the corpus per pair.
    let selectors: BTreeMap<_, _> = m.selectors.iter().map(|s| (&s.id, s)).collect();
    let mut bound: BTreeMap<&String, Vec<&Selector>> = BTreeMap::new();
    for e in m.edges.iter().filter(|e| e.kind == "CONSTRAINS") {
        bound
            .entry(&e.from)
            .or_default()
            .push(selectors[e.to_selector.as_ref().ok_or(Error("binding_target"))?]);
    }
    let mut groups: BTreeMap<_, Vec<&Record>> = BTreeMap::new();
    for r in m.records.iter().filter(|r| {
        r.lifecycle == "active" && ["decision", "constraint"].contains(&r.role.as_str())
    }) {
        groups.entry((&r.subject, &r.role)).or_default().push(r);
    }
    for group in groups.values() {
        for (i, a) in group.iter().enumerate() {
            for b in &group[i + 1..] {
                let contradiction = a.disposition != b.disposition
                    || (a.verification.subject == b.verification.subject
                        && [
                            a.verification.outcome.as_str(),
                            b.verification.outcome.as_str(),
                        ]
                        .contains(&"supported")
                        && [
                            a.verification.outcome.as_str(),
                            b.verification.outcome.as_str(),
                        ]
                        .contains(&"contradicted"));
                if contradiction {
                    let a_scopes = bound.get(&a.id);
                    let b_scopes = bound.get(&b.id);
                    let overlap = match (a_scopes, b_scopes) {
                        (Some(a), Some(b)) => a.iter().any(|x| b.iter().any(|y| overlaps(x, y))),
                        _ => true,
                    };
                    require(!overlap, "authority_conflict")?;
                }
            }
        }
    }
    Ok(())
}
