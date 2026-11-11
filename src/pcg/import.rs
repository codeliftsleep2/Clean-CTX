//! One bounded admission/audit operation. No persistent source-validation cache.
use super::{
    Error, Result, enrollment, git,
    model::Generation,
    profile::{Trusted, Work},
    require, store,
};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path, time::Instant};

#[derive(Serialize)]
pub(super) struct Report {
    pub generation: String,
    pub operation: &'static str,
    pub authority_assurance: &'static str,
    pub freshness_guarantee: &'static str,
    pub continuous_currentness: bool,
    pub atomic_filesystem_snapshot: bool,
    pub work: Work,
    pub milliseconds: BTreeMap<&'static str, u128>,
    pub database_bytes: u64,
    pub wal_bytes: u64,
}

pub(super) struct Prepared {
    pub trusted: Trusted,
    pub generation: Generation,
    pub expected: Option<String>,
    pub report: Report,
}

pub(super) fn prepare(path: &Path, audit: bool) -> Result<Prepared> {
    let trusted = Trusted::load(path)?;
    let mut work = Work::new(trusted.profile.limits.evidence_bytes);
    let mut times = BTreeMap::new();
    let start = Instant::now();
    let bytes = trusted.manifest(&mut work)?;
    let manifest = enrollment::decode(&bytes, &trusted)?;
    work.definitions = (manifest.sources.len()
        + manifest.evidence.len()
        + manifest.records.len()
        + manifest.selectors.len()) as u64;
    work.assertions_and_refs = (manifest.edges.len()
        + manifest
            .records
            .iter()
            .map(|r| r.evidence_refs.len())
            .sum::<usize>()) as u64;
    times.insert("manifest", start.elapsed().as_millis());
    let start = Instant::now();
    let expected = store::expected_head(&trusted, &manifest.corpus_id)?;
    times.insert("reopen_metadata", start.elapsed().as_millis());
    let start = Instant::now();
    let blobs = git::sources(&trusted, &manifest, &mut work)?;
    git::current(&trusted, &manifest, &mut work, true)?;
    times.insert("source_verification", start.elapsed().as_millis());
    let start = Instant::now();
    let generation = enrollment::normalize(manifest, &blobs, &trusted, &mut work)?;
    times.insert("evidence_and_generation", start.elapsed().as_millis());
    let report = Report {
        generation: generation.id.clone(),
        operation: if audit { "audit" } else { "import" },
        authority_assurance: "source_attributed",
        freshness_guarantee: "observed_boundaries_only",
        continuous_currentness: false,
        atomic_filesystem_snapshot: false,
        work,
        milliseconds: times,
        database_bytes: 0,
        wal_bytes: 0,
    };
    Ok(Prepared {
        trusted,
        generation,
        expected,
        report,
    })
}

fn completion(p: &mut Prepared) -> Result<()> {
    git::current(
        &p.trusted,
        &p.generation.manifest,
        &mut p.report.work,
        false,
    )?;
    p.trusted.recheck()?;
    // Re-read approval bytes at commit/audit completion; debit actual sidecar work.
    p.trusted.manifest(&mut p.report.work)?;
    Ok(())
}

pub(super) fn finish(mut p: Prepared, rebuild: bool, audit: bool) -> Result<Report> {
    if audit {
        let start = Instant::now();
        let connection = store::reader(&p.trusted, &p.generation.manifest.corpus_id)?;
        require(
            store::head(&connection, &p.trusted, &p.generation.manifest.corpus_id)?
                == Some(p.generation.id.clone()),
            "generation_mismatch",
        )?;
        store::compare(&connection, &p.trusted, &p.generation)?;
        completion(&mut p)?;
        p.report
            .milliseconds
            .insert("full_audit", start.elapsed().as_millis());
    } else {
        let (mut connection, new) = store::writer(&p.trusted)?;
        let transaction = store::begin(&mut connection)?;
        store::initialize(&transaction, new)?;
        require(
            store::head(&transaction, &p.trusted, &p.generation.manifest.corpus_id)? == p.expected,
            "import_head_changed",
        )?;
        store::preserve_decisions(&transaction, &p.trusted, &p.generation)?;
        let start = Instant::now();
        let old = store::existing(&transaction, &p.trusted, &p.generation)?;
        if let Some(id) = &old {
            require(id == &p.generation.id, "idempotency_conflict")?;
        }
        if rebuild {
            store::clear(&transaction, &p.trusted, &p.generation.manifest.corpus_id)?;
            store::insert(&transaction, &p.trusted, &p.generation)?;
            p.report.operation = "rebuild";
        } else if old.is_some() {
            // An identical key must match every stored row, not just its header.
            store::compare(&transaction, &p.trusted, &p.generation)?;
            p.report.operation = "idempotent_import";
        } else {
            store::insert(&transaction, &p.trusted, &p.generation)?;
        }
        p.report
            .milliseconds
            .insert("sqlite_insertion_or_compare", start.elapsed().as_millis());
        completion(&mut p)?;
        store::publish(&transaction, &p.trusted, &p.generation)?;
        let start = Instant::now();
        store::commit(transaction)?;
        p.report
            .milliseconds
            .insert("commit", start.elapsed().as_millis());
        p.report.database_bytes = std::fs::metadata(&p.trusted.profile.database_path)
            .map_err(|_| Error("projection_unavailable"))?
            .len();
        p.report.wal_bytes = std::fs::metadata(format!("{}-wal", p.trusted.profile.database_path))
            .map(|m| m.len())
            .unwrap_or(0);
    }
    if audit {
        p.report.database_bytes = std::fs::metadata(&p.trusted.profile.database_path)
            .map_err(|_| Error("projection_unavailable"))?
            .len();
        p.report.wal_bytes = std::fs::metadata(format!("{}-wal", p.trusted.profile.database_path))
            .map(|m| m.len())
            .unwrap_or(0);
    }
    Ok(p.report)
}
