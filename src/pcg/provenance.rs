//! A complete required proof bundle for the later token projector. No summaries.
use super::{
    Error, Result, closure::Proof, identity, profile::Work, query::Acquisition, query_rows::Reader,
    query_witnesses::Witnesses, require, security,
};
use serde_json::json;

pub(super) fn materialize(
    reader: &mut Reader<'_>,
    proof: &Proof,
    witnesses: &Witnesses,
    work: &mut Work,
    output: &mut Acquisition,
) -> Result<()> {
    for id in &proof.evidence {
        let evidence = reader.evidence(id)?;
        let source = reader.source(&evidence.source)?;
        let bytes = witnesses
            .blobs
            .get(&evidence.source)
            .ok_or(Error("source_missing"))?;
        let start = usize::try_from(evidence.start).map_err(|_| Error("projection_corrupt"))?;
        let end = usize::try_from(evidence.end).map_err(|_| Error("projection_corrupt"))?;
        let span = bytes.get(start..end).ok_or(Error("projection_corrupt"))?;
        let text = std::str::from_utf8(span).map_err(|_| Error("projection_corrupt"))?;
        let stored = reader.quote(&evidence, work)?;
        require(
            stored == span && identity::sha(span) == evidence.hash,
            "projection_corrupt",
        )?;
        security::nonsecret(text)?;
        work.evidence_validations += 1;
        output.citations.push(json!({"id":id,"revision":evidence.revision,"source":source.data.path,
            "source_id":source.data.id,"source_revision":source.revision,"commit":source.data.commit,
            "blob":source.data.blob,"source_sha256":source.data.sha256,"start":evidence.start,"end":evidence.end,
            "quote_sha256":evidence.hash,"text":text,"freshness":"current","untrusted_evidence":true}));
    }
    output.citations.sort_by_key(|v| {
        (
            v["source_id"].as_str().unwrap_or("").to_owned(),
            v["source_revision"].as_str().unwrap_or("").to_owned(),
            v["start"].as_u64(),
            v["end"].as_u64(),
        )
    });
    for id in &proof.records {
        let record = reader.record(id)?;
        let mut data =
            serde_json::to_value(&record.data).map_err(|_| Error("projection_corrupt"))?;
        data["revision"] = json!(record.revision);
        data["evidence_refs"] = json!(
            record
                .data
                .evidence_refs
                .iter()
                .filter(|e| e.required)
                .collect::<Vec<_>>()
        );
        data["authority"]["assurance"] = json!("source_attributed");
        data["freshness"] = json!("current");
        data["citations"] = json!(
            std::iter::once(record.data.primary.clone())
                .chain(
                    record
                        .data
                        .evidence_refs
                        .iter()
                        .filter(|e| e.required)
                        .map(|e| e.id.clone())
                )
                .collect::<std::collections::BTreeSet<_>>()
        );
        output.required_records.push(data);
    }
    let mut edges: Vec<_> = proof.edges.values().collect();
    edges.sort_by_key(|e| (&e.kind, &e.from, &e.to, &e.id));
    for edge in edges {
        output.relationships.push(json!({"id":edge.id,"revision":edge.revision,"kind":edge.kind,
            "from":edge.from,"from_revision":edge.from_revision,"to":edge.to,"to_revision":edge.to_revision,
            "citation":edge.evidence,"gate":edge.gate}));
    }
    Ok(())
}
