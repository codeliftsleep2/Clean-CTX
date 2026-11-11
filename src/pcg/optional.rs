//! Bounded descriptive additions. Required proof and COMPLETION always have priority.
use super::{
    Error, Result, closure::Proof, profile::Work, provenance, query::Acquisition,
    query_rows::Reader, query_witnesses::Witnesses, require,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
pub(super) struct Support {
    pub records: Vec<serde_json::Value>,
    pub history: Vec<serde_json::Value>,
    pub relationships: Vec<serde_json::Value>,
    pub citations: Vec<serde_json::Value>,
    pub omissions: BTreeSet<String>,
    pub standalone: BTreeSet<String>,
}

/// Acquire direct descriptive support and optional refs only; never promote them.
pub(super) fn collect(
    reader: &mut Reader<'_>,
    witnesses: &mut Witnesses,
    work: &mut Work,
    required: &Proof,
) -> Result<Support> {
    let mut output = Support::default();
    if !reader.request.include_history {
        output.omissions.insert("not_requested".into());
    }
    let ceiling = work.ceiling;
    // The first observation supplies the exact distinct-byte reservation for COMPLETION.
    work.ceiling = ceiling.saturating_sub(work.current_bytes);
    let mut seen = required.evidence.clone();
    let mut seen_records = required.records.clone();
    let outcome = (|| {
        for id in &required.records {
            let record = reader.record(id)?;
            for reference in record.data.evidence_refs.iter().filter(|e| !e.required) {
                add(
                    reader,
                    witnesses,
                    work,
                    &mut seen,
                    &mut output,
                    Proof {
                        records: BTreeSet::new(),
                        edges: BTreeMap::new(),
                        evidence: BTreeSet::from([reference.id.clone()]),
                    },
                    false,
                )?;
            }
            for edge in
                reader.relations(id, false, &["USES_EVIDENCE", "PRESERVES", "IMPLEMENTS"])?
            {
                if !seen_records.insert(edge.to.clone()) {
                    continue;
                }
                let target = reader.record(&edge.to)?;
                let primary = reader.evidence(&target.data.primary)?;
                let historical = target.data.lifecycle != "active"
                    || reader.source(&primary.source)?.data.check == "historical";
                if historical
                    && (!reader.request.include_history || !reader.trusted.profile.history_allowed)
                {
                    output.omissions.insert("not_requested".into());
                    continue;
                }
                let mut evidence =
                    BTreeSet::from([target.data.primary.clone(), edge.evidence.clone()]);
                evidence.extend(
                    target
                        .data
                        .evidence_refs
                        .iter()
                        .filter(|e| e.required)
                        .map(|e| e.id.clone()),
                );
                add(
                    reader,
                    witnesses,
                    work,
                    &mut seen,
                    &mut output,
                    Proof {
                        records: BTreeSet::from([edge.to.clone()]),
                        edges: BTreeMap::from([(edge.id.clone(), edge)]),
                        evidence,
                    },
                    historical,
                )?;
            }
        }
        Ok(())
    })();
    work.ceiling = ceiling;
    match outcome {
        Err(Error("work_limit")) => {
            output.omissions.insert("work_limit".into());
        }
        Err(e) => return Err(e),
        Ok(()) => (),
    }
    Ok(output)
}

fn add(
    reader: &mut Reader<'_>,
    witnesses: &mut Witnesses,
    work: &mut Work,
    seen: &mut BTreeSet<String>,
    output: &mut Support,
    mut proof: Proof,
    history: bool,
) -> Result<()> {
    proof.evidence.retain(|id| !seen.contains(id));
    let outcome = (|| {
        for id in &proof.evidence {
            let evidence = reader.evidence(id)?;
            let source = reader.source(&evidence.source)?;
            if source.data.check == "historical"
                && (!reader.request.include_history || !reader.trusted.profile.history_allowed)
            {
                return Err(Error("history_denied"));
            }
            if !witnesses.blobs.contains_key(&source.data.id) {
                // No off-path historical acquisition: only already witnessed sources
                // can supply optional text in this slice. Omission is explicit.
                return Err(Error("optional_source_unavailable"));
            }
        }
        let mut bytes = 0u64;
        for id in &proof.evidence {
            let evidence = reader.evidence(id)?;
            bytes = bytes
                .checked_add(evidence.end - evidence.start)
                .ok_or(Error("work_limit"))?;
        }
        require(
            bytes <= work.ceiling.saturating_sub(work.bytes),
            "work_limit",
        )?;
        let mut bundle = Acquisition::new(reader.request.generation.clone());
        provenance::materialize(reader, &proof, witnesses, work, &mut bundle)?;
        Ok(bundle)
    })();
    match outcome {
        Ok(bundle) => {
            if proof.records.is_empty() {
                output.standalone.extend(proof.evidence.iter().cloned());
            }
            seen.extend(proof.evidence);
            if history {
                output.history.extend(bundle.required_records);
            } else {
                output.records.extend(bundle.required_records);
            }
            output.relationships.extend(bundle.relationships);
            output.citations.extend(bundle.citations);
        }
        Err(Error("work_limit")) => return Err(Error("work_limit")),
        Err(e) if ["projection_corrupt", "material_denied"].contains(&e.0) => return Err(e),
        Err(_) => {
            output.omissions.insert("unavailable".into());
        }
    }
    Ok(())
}
