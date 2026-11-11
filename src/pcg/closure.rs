//! Mandatory proof traversal. Descriptive evidence is never a prerequisite.
use super::{
    Error, Result,
    query::{Acquisition, Mode},
    query_rows::{Reader, RelationRow},
    require,
    selectors::Truth,
    validation,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub(super) struct Proof {
    pub records: BTreeSet<String>,
    pub edges: BTreeMap<String, RelationRow>,
    pub evidence: BTreeSet<String>,
}

pub(super) fn collect(
    reader: &mut Reader<'_>,
    roots: &BTreeSet<String>,
    applicability: &BTreeMap<String, Truth>,
    bindings: &BTreeMap<String, RelationRow>,
    result: &mut Acquisition,
) -> Result<Proof> {
    let mut proof = Proof {
        records: BTreeSet::new(),
        edges: BTreeMap::new(),
        evidence: BTreeSet::new(),
    };
    let mut queue: VecDeque<_> = roots.iter().cloned().collect();
    let mut premises = BTreeSet::new();
    let mut refuted = BTreeSet::new();
    let mut checked_subjects = BTreeSet::new();
    for edge in bindings.values().filter(|e| roots.contains(&e.from)) {
        proof.evidence.insert(edge.evidence.clone());
        proof.edges.insert(edge.id.clone(), edge.clone());
    }
    while let Some(id) = queue.pop_front() {
        if !proof.records.insert(id.clone()) {
            continue;
        }
        let record = reader.record(&id)?;
        require(
            record.data.lifecycle == "active" || reader.trusted.profile.history_allowed,
            "history_required",
        )?;
        proof.evidence.insert(record.data.primary.clone());
        proof.evidence.extend(
            record
                .data
                .evidence_refs
                .iter()
                .filter(|e| e.required)
                .map(|e| e.id.clone()),
        );
        let outgoing = reader.relations(&id, false, &["DEPENDS_ON", "BLOCKED_BY", "SUPERSEDES"])?;
        for edge in outgoing {
            let target = reader.record(&edge.to)?;
            require(
                ["claim", "decision", "constraint"].contains(&record.data.role.as_str())
                    && ["claim", "decision", "constraint"].contains(&target.data.role.as_str()),
                "projection_corrupt",
            )?;
            if edge.kind == "SUPERSEDES" {
                require(
                    record.data.role == target.data.role
                        && record.data.subject == target.data.subject
                        && target.data.lifecycle == "superseded",
                    "projection_corrupt",
                )?;
            } else {
                premises.insert(edge.to.clone());
                if let Some(gate) = &edge.gate {
                    require(gate.outcome == "supported", "projection_corrupt")?;
                    let required = validation::authority(&gate.authority)?;
                    require(required >= 2, "projection_corrupt")?;
                    if target.data.verification.outcome != gate.outcome
                        || target.data.verification.subject != gate.subject
                        || validation::authority(&target.data.authority.class)? < required
                    {
                        result.issue("gate_unsatisfied", true);
                    }
                }
            }
            queue.push_back(edge.to.clone());
            proof.evidence.insert(edge.evidence.clone());
            proof.edges.insert(edge.id.clone(), edge);
        }
        let incoming = reader.relations(&id, true, &["VALIDATES", "FALSIFIES"])?;
        let mut support = false;
        let mut refutation = false;
        for edge in incoming {
            let source = reader.record(&edge.from)?;
            require(
                ["claim", "evidence"].contains(&source.data.role.as_str()),
                "projection_corrupt",
            )?;
            if source.data.lifecycle == "active" {
                support |= edge.kind == "VALIDATES";
                refutation |= edge.kind == "FALSIFIES";
            }
            queue.push_back(edge.from.clone());
            proof.evidence.insert(edge.evidence.clone());
            proof.edges.insert(edge.id.clone(), edge);
        }
        if refutation {
            refuted.insert(id.clone());
            if support
                || (record.data.lifecycle == "active"
                    && (record.data.verification.outcome == "supported"
                        || ["decision", "constraint"].contains(&record.data.role.as_str())))
            {
                result.issue("authority_conflict", true);
            }
        }
        if reader.request.mode == Mode::Impact {
            for edge in reader.relations(&id, true, &["DEPENDS_ON", "BLOCKED_BY"])? {
                queue.push_back(edge.from.clone());
                proof.evidence.insert(edge.evidence.clone());
                proof.edges.insert(edge.id.clone(), edge);
            }
        }
        if ["decision", "constraint"].contains(&record.data.role.as_str())
            && record.data.lifecycle == "active"
        {
            if record.data.verification.outcome == "contradicted" {
                result.issue("premise_unsatisfied", true);
            }
            let subject_key = (record.data.subject.clone(), record.data.role.clone());
            if applicability.get(&id) != Some(&Truth::True) || !checked_subjects.insert(subject_key)
            {
                continue;
            }
            for peer_id in reader.peers(&record.data)? {
                if peer_id == id {
                    continue;
                }
                let peer = reader.record(&peer_id)?;
                let opposite = record.data.disposition != peer.data.disposition
                    || (record.data.verification.subject == peer.data.verification.subject
                        && [
                            record.data.verification.outcome.as_str(),
                            peer.data.verification.outcome.as_str(),
                        ]
                        .contains(&"supported")
                        && [
                            record.data.verification.outcome.as_str(),
                            peer.data.verification.outcome.as_str(),
                        ]
                        .contains(&"contradicted"));
                if !opposite {
                    continue;
                }
                match applicability.get(&peer_id).copied().unwrap_or(Truth::False) {
                    Truth::True => {
                        queue.push_back(peer_id);
                        result.issue("authority_conflict", true);
                    }
                    Truth::Unknown => {
                        result.issue("applicability_unknown", false);
                        result.coverage.applicability = false;
                    }
                    Truth::False => (),
                }
            }
        }
    }
    for id in premises {
        let target = reader.record(&id)?;
        if refuted.contains(&id)
            || target.data.lifecycle != "active"
            || target.data.verification.outcome == "contradicted"
            || validation::authority(&target.data.authority.class)? < 2
        {
            result.issue("premise_unsatisfied", true);
        }
    }
    acyclic(&proof)?;
    Ok(proof)
}

fn acyclic(proof: &Proof) -> Result<()> {
    for kinds in [&["DEPENDS_ON", "BLOCKED_BY"][..], &["SUPERSEDES"][..]] {
        let mut degree: BTreeMap<_, usize> = proof.records.iter().map(|id| (id, 0)).collect();
        let mut next: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for edge in proof
            .edges
            .values()
            .filter(|e| kinds.contains(&e.kind.as_str()))
        {
            *degree
                .get_mut(&edge.to)
                .ok_or(Error("projection_corrupt"))? += 1;
            next.entry(&edge.from).or_default().push(&edge.to);
        }
        let mut queue: VecDeque<_> = degree
            .iter()
            .filter(|(_, n)| **n == 0)
            .map(|(id, _)| *id)
            .collect();
        let mut visited = 0;
        while let Some(id) = queue.pop_front() {
            visited += 1;
            for target in next.get(id).into_iter().flatten() {
                let count = degree.get_mut(target).ok_or(Error("projection_corrupt"))?;
                *count -= 1;
                if *count == 0 {
                    queue.push_back(*target);
                }
            }
        }
        require(visited == proof.records.len(), "projection_corrupt")?;
    }
    Ok(())
}
