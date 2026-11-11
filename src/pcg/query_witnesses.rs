//! Required immutable evidence plus two corpus-current filesystem observations.
use super::{Error, Result, git, model::*, profile::Work, query_rows::Reader, require};
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct Witnesses {
    pub manifest: Manifest,
    pub blobs: BTreeMap<String, Vec<u8>>,
}

fn source_error(error: Error) -> Error {
    match error.0 {
        "work_limit" | "source_stale" | "source_inaccessible" => error,
        "history_denied" => Error("history_required"),
        "git_object_invalid" | "git_commit_invalid" | "git_membership" => {
            Error("source_inaccessible")
        }
        _ => Error("source_invalid"),
    }
}

pub(super) fn start(
    reader: &mut Reader<'_>,
    evidence: &BTreeSet<String>,
    work: &mut Work,
) -> Result<Witnesses> {
    let mut sources: BTreeMap<_, _> = reader
        .current_sources()?
        .into_iter()
        .map(|s| (s.id.clone(), s))
        .collect();
    require(!sources.is_empty(), "projection_corrupt")?;
    let commit = sources
        .values()
        .next()
        .ok_or(Error("projection_corrupt"))?
        .commit
        .clone();
    require(
        sources
            .values()
            .all(|s| s.commit == commit && s.check == "current"),
        "projection_corrupt",
    )?;
    for id in evidence {
        let evidence = reader.evidence(id)?;
        let source = reader.source(&evidence.source)?;
        if source.data.check == "historical" && !reader.trusted.profile.history_allowed {
            return Err(Error("history_required"));
        }
        sources.insert(source.data.id.clone(), source.data);
    }
    let profile = &reader.trusted.profile;
    let manifest = Manifest {
        schema_version: 1,
        parser_version: "pcg-json-v1".into(),
        corpus_id: reader.request.corpus.clone(),
        source_namespace: "query-witnesses".into(),
        enrollment: Enrollment {
            owner_label: profile.owner_label.clone(),
            policy_revision: profile.policy_revision.clone(),
            complete: false,
        },
        sources: sources.into_values().collect(),
        selectors: vec![],
        evidence: vec![],
        records: vec![],
        edges: vec![],
    };
    let blobs = git::sources(reader.trusted, &manifest, work).map_err(source_error)?;
    git::current(reader.trusted, &manifest, work, true).map_err(source_error)?;
    Ok(Witnesses { manifest, blobs })
}

pub(super) fn complete(reader: &Reader<'_>, witnesses: &Witnesses, work: &mut Work) -> Result<()> {
    git::current(reader.trusted, &witnesses.manifest, work, false).map_err(source_error)?;
    for source in witnesses
        .manifest
        .sources
        .iter()
        .filter(|s| s.check == "historical")
    {
        require(reader.trusted.profile.history_allowed, "history_required")?;
        // Check current permission/availability without re-reading immutable bytes.
        git::command(reader.trusted, &["cat-file", "-e", &source.blob], 1).map_err(source_error)?;
    }
    reader.trusted.recheck()?;
    Ok(())
}
