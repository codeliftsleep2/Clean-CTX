//! Prepare an entire generation once; exact rows are shared by insertion and audit.
use super::{
    Error, Result,
    identity::{self, canonical, hash, set},
    model::*,
    profile::{Trusted, Work},
    require, security, validation,
};
use rusqlite::types::Value as Sql;
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub(super) fn decode(bytes: &[u8], trusted: &Trusted) -> Result<Manifest> {
    let value = identity::parse(bytes)?;
    security::json_nonsecret(&value)?;
    let definitions: usize = ["sources", "selectors", "evidence", "records"]
        .iter()
        .filter_map(|key| value.get(*key).and_then(Value::as_array))
        .map(Vec::len)
        .sum();
    let raw_refs: usize = value
        .get("records")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|r| r.get("evidence_refs").and_then(Value::as_array))
        .map(Vec::len)
        .sum();
    let raw_edges = value
        .get("edges")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    require(
        definitions as u64 <= trusted.profile.limits.records
            && (raw_refs + raw_edges) as u64 <= trusted.profile.limits.edges,
        "work_limit",
    )?;
    // Nullable interval endpoints are required fields, unlike optional dimensions.
    if let Some(selectors) = value.get("selectors").and_then(Value::as_array) {
        for s in selectors {
            require(
                s.get("from").is_some() && s.get("until").is_some(),
                "selector_interval",
            )?;
        }
    }
    for (array, optional) in [
        ("selectors", &["file", "entity", "operation", "task"][..]),
        ("evidence", &["heading"][..]),
        ("edges", &["to", "to_selector", "gate"][..]),
    ] {
        for item in value
            .get(array)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for key in optional {
                require(
                    item.get(*key).is_none_or(|v| !v.is_null()),
                    "manifest_schema",
                )?;
            }
        }
    }
    let mut manifest: Manifest =
        serde_json::from_value(value).map_err(|_| Error("manifest_schema"))?;
    for record in &mut manifest.records {
        record.evidence_refs.sort_by(|a, b| a.id.cmp(&b.id));
        for pair in record.evidence_refs.windows(2) {
            require(
                pair[0].id != pair[1].id || pair[0].required == pair[1].required,
                "conflicting_ref",
            )?;
        }
        record.evidence_refs.dedup_by(|a, b| a.id == b.id);
    }
    validation::validate(&manifest, &trusted.profile)?;
    Ok(manifest)
}

fn text(s: impl ToString) -> Sql {
    Sql::Text(s.to_string())
}
fn optional(s: &Option<String>) -> Sql {
    s.as_ref().map(text).unwrap_or(Sql::Null)
}
fn boolean(b: bool) -> Sql {
    Sql::Integer(i64::from(b))
}

pub(super) fn normalize(
    m: Manifest,
    blobs: &BTreeMap<String, Vec<u8>>,
    trusted: &Trusted,
    work: &mut Work,
) -> Result<Generation> {
    let mut sources = BTreeMap::new();
    for s in &m.sources {
        sources.insert(
            s.id.clone(),
            hash(&json!({"source_id":s.id,"path":s.path,"commit":s.commit,
            "blob":s.blob,"sha256":s.sha256,"check":s.check})),
        );
    }
    let mut evidence = BTreeMap::new();
    let mut quotes = BTreeMap::new();
    let evidence_start = std::time::Instant::now();
    for e in &m.evidence {
        let source = blobs.get(&e.source).ok_or(Error("evidence_source"))?;
        let start = usize::try_from(e.start).map_err(|_| Error("evidence_span"))?;
        let end = usize::try_from(e.end).map_err(|_| Error("evidence_span"))?;
        let quote = source.get(start..end).ok_or(Error("evidence_span"))?;
        let string = std::str::from_utf8(quote).map_err(|_| Error("evidence_span"))?;
        require(identity::sha(quote) == e.sha256, "evidence_hash")?;
        if let Some(heading) = &e.heading {
            require(
                string.starts_with(heading) && string.matches(heading).count() == 1,
                "evidence_heading",
            )?;
        }
        security::nonsecret(string)?;
        work.evidence_bytes += quote.len() as u64;
        work.evidence_validations += 1;
        evidence.insert(
            e.id.clone(),
            hash(
                &json!({"evidence_id":e.id,"source_revision":sources[&e.source],
            "start":e.start,"end":e.end,"quote_sha256":e.sha256}),
            ),
        );
        quotes.insert(e.id.clone(), quote.to_vec());
    }
    work.evidence_hash_microseconds = evidence_start.elapsed().as_micros();
    let generation_start = std::time::Instant::now();
    let mut records = BTreeMap::new();
    for r in &m.records {
        let refs = set(r
            .evidence_refs
            .iter()
            .map(|e| json!({"evidence_revision":evidence[&e.id],"required":e.required})));
        records.insert(r.id.clone(), hash(&json!({"record_id":r.id,"role":r.role,"required":r.required,
            "primary_evidence_revision":evidence[&r.primary],"authority":r.authority,"verification":r.verification,
            "lifecycle":r.lifecycle,"subject":r.subject,"disposition":r.disposition,"sorted_evidence_refs":refs})));
    }
    let selectors: BTreeMap<_, _> = m
        .selectors
        .iter()
        .map(|s| (s.id.clone(), hash(&json!(s))))
        .collect();
    let mut edges = BTreeMap::new();
    for e in &m.edges {
        let value = if let Some(selector) = &e.to_selector {
            json!({"edge_id":e.id,"kind":e.kind,"from_record_revision":records[&e.from],
                "selector_revision":selectors[selector],"provenance_evidence_revision":evidence[&e.evidence]})
        } else {
            json!({"edge_id":e.id,"kind":e.kind,"from_record_revision":records[&e.from],
                "to_record_revision":records[e.to.as_ref().ok_or(Error("edge_endpoint"))?],
                "gate":e.gate,"provenance_evidence_revision":evidence[&e.evidence]})
        };
        edges.insert(e.id.clone(), hash(&value));
    }
    let p = &trusted.profile;
    let id = hash(
        &json!({"partition":p.partition_id,"corpus":m.corpus_id,"raw_manifest_sha256":p.approved_manifest_sha256,
        "policy_revision":p.policy_revision,"parser_version":m.parser_version,
        "sorted_source_revisions":set(sources.values().map(|s| json!(s))),
        "sorted_record_revisions":set(records.values().map(|s| json!(s))),
        "sorted_edge_or_binding_revisions":set(edges.values().map(|s| json!(s))),
        "sorted_selector_revisions":set(selectors.values().map(|s| json!(s)))}),
    );
    let mut rows = BTreeMap::new();
    for table in [
        "generation",
        "source_revision",
        "evidence_revision",
        "record_revision",
        "evidence_ref",
        "selector",
        "binding",
        "relation",
    ] {
        rows.insert(table, Vec::new());
    }
    let prefix = vec![text(&p.partition_id), text(&m.corpus_id), text(&id)];
    let mut add = |table: &'static str, suffix: Vec<Sql>| {
        let mut row = prefix.clone();
        row.extend(suffix);
        rows.get_mut(table).expect("known table").push(row);
    };
    let commit = &m
        .sources
        .iter()
        .find(|s| s.check == "current")
        .ok_or(Error("current_source_required"))?
        .commit;
    add(
        "generation",
        vec![
            text(&p.approved_manifest_sha256),
            text(&p.policy_revision),
            text(&m.parser_version),
            text(commit),
            text(&id),
            boolean(m.enrollment.complete),
        ],
    );
    for s in &m.sources {
        add(
            "source_revision",
            vec![
                text(&s.id),
                text(&sources[&s.id]),
                text(&s.path),
                text(&s.commit),
                text(&s.blob),
                text(&s.sha256),
                text(&s.check),
            ],
        );
    }
    for e in &m.evidence {
        add(
            "evidence_revision",
            vec![
                text(&e.id),
                text(&evidence[&e.id]),
                text(&e.source),
                text(&sources[&e.source]),
                Sql::Integer(e.start as i64),
                Sql::Integer(e.end as i64),
                text(&e.sha256),
                Sql::Blob(quotes.remove(&e.id).ok_or(Error("evidence_source"))?),
            ],
        );
    }
    for r in &m.records {
        add(
            "record_revision",
            vec![
                text(&r.id),
                text(&records[&r.id]),
                text(&r.role),
                boolean(r.required),
                text(&r.primary),
                text(&evidence[&r.primary]),
                text(&r.authority.class),
                text(&r.authority.attributed_to),
                text(&r.verification.outcome),
                text(&r.verification.subject),
                text(&r.verification.execution),
                text(&r.lifecycle),
                text(&r.subject),
                text(&r.disposition),
            ],
        );
        for e in &r.evidence_refs {
            add(
                "evidence_ref",
                vec![
                    text(&r.id),
                    text(&records[&r.id]),
                    text(&e.id),
                    text(&evidence[&e.id]),
                    boolean(e.required),
                ],
            );
        }
    }
    for s in &m.selectors {
        add(
            "selector",
            vec![
                text(&s.id),
                text(&selectors[&s.id]),
                text(&s.purpose),
                text(&s.workspace),
                optional(&s.file),
                s.entity
                    .as_ref()
                    .map(|e| text(canonical(&json!(e))))
                    .unwrap_or(Sql::Null),
                optional(&s.operation),
                optional(&s.task),
                optional(&s.from),
                optional(&s.until),
                text(canonical(&json!(s))),
            ],
        );
    }
    for e in &m.edges {
        let mut values = vec![text(&e.id), text(&edges[&e.id])];
        if let Some(selector) = &e.to_selector {
            values.extend([
                text(&e.from),
                text(&records[&e.from]),
                text(selector),
                text(&selectors[selector]),
                text(&e.evidence),
                text(&evidence[&e.evidence]),
            ]);
            add("binding", values);
        } else {
            let to = e.to.as_ref().ok_or(Error("edge_endpoint"))?;
            values.extend([
                text(&e.kind),
                text(&e.from),
                text(&records[&e.from]),
                text(to),
                text(&records[to]),
                text(&e.evidence),
                text(&evidence[&e.evidence]),
                e.gate
                    .as_ref()
                    .map(|g| text(canonical(&json!(g))))
                    .unwrap_or(Sql::Null),
            ]);
            add("relation", values);
        }
    }
    // Physical insertion order is stable too; equality uses complete typed row sets.
    for values in rows.values_mut() {
        values.sort_by_cached_key(|v| format!("{v:?}"));
    }
    work.generation_microseconds = generation_start.elapsed().as_micros();
    Ok(Generation {
        manifest: m,
        id,
        rows: Rows(rows),
    })
}
