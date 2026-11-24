//! Bounded deterministic discovery. Explicit files never enumerate the workspace.
use super::{Failure, Query};
use crate::ir::source_observations::{
    bounded,
    request_budget::{Cost, Permit, Request, Stop, Work},
};
use crate::ir::typescript_resolution::Witness;
use crate::workspace::scope::WorkspaceScope;
use std::path::{Path, PathBuf};

pub(super) struct Inventory {
    pub files: Vec<PathBuf>,
    pub excluded: Vec<(PathBuf, String)>,
    pub omissions: Vec<String>,
    pub witnesses: Vec<Witness>,
    pub roots: Vec<PathBuf>,
    pub complete: bool,
    pub _reservations: Vec<Permit>,
    pub dto_reservation: Option<Permit>,
}
fn excluded(roots: &[PathBuf], path: &Path) -> bool {
    !bounded::application_path(roots, path)
}
pub(super) fn collect(
    request: &Request,
    roots: &[PathBuf],
    query: &Query,
    policy: Option<&crate::config::CleanCtxConfig>,
) -> Result<Inventory, Failure> {
    let _active = request.reserve_optional(Cost::default())?;
    let mut host = bounded::BoundedHost::new(request, roots).map_err(|e| {
        request
            .snapshot()
            .last_stop
            .map_or_else(|| Failure::Invalid(e), Failure::Budget)
    })?;
    let primary = roots[0]
        .to_str()
        .ok_or_else(|| Failure::Invalid("lossy primary path".into()))?;
    let additional = roots[1..]
        .iter()
        .map(|p| {
            p.to_str()
                .map(str::to_owned)
                .ok_or_else(|| Failure::Invalid("lossy root".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for _ in roots {
        request.work(Work::Physical, false)?;
    }
    let scope = if let Some(within) = &query.within_path {
        request.work(Work::Physical, false)?;
        request.work(Work::Metadata, false)?;
        WorkspaceScope::narrowed(Some(primary), &additional, within).map_err(Failure::Invalid)?
    } else {
        WorkspaceScope::new(Some(primary), &additional)
            .ok_or_else(|| Failure::Invalid("missing scope".into()))?
    };
    let mut result = Inventory {
        files: Vec::new(),
        excluded: Vec::new(),
        omissions: Vec::new(),
        witnesses: Vec::new(),
        roots: host.roots().to_vec(),
        complete: true,
        _reservations: Vec::new(),
        dto_reservation: None,
    };
    let mut pending = if let Some(files) = &query.source_files {
        files
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    roots[0].join(p)
                }
            })
            .collect::<Vec<_>>()
    } else {
        host.roots().to_vec()
    };
    let mut directories = std::collections::BTreeSet::new();
    while !pending.is_empty() {
        pending.sort_by(|a, b| b.cmp(a));
        let path = pending.pop().expect("pending path");
        let step = (|| -> Result<(), Failure> {
            request.work(Work::Exclusion, false)?;
            if excluded(roots, &path) {
                if query.source_files.is_some() {
                    return Err(Failure::Invalid(
                        "dependency/build/cache source refused".into(),
                    ));
                }
                result
                    .excluded
                    .push((path.clone(), "dependency/build/cache".into()));
                return Ok(());
            }
            if let Some(policy) = policy {
                request.work(Work::Exclusion, false)?;
                if policy.is_excluded(&path.to_string_lossy()) {
                    if query.source_files.is_some() {
                        return Err(Failure::Invalid(
                            "Explicit source excluded by session policy".into(),
                        ));
                    }
                    result
                        .excluded
                        .push((path.clone(), "session source-exclusion policy".into()));
                    return Ok(());
                }
            }
            request.work(Work::Metadata, false)?;
            if !host.allows_path(&path) {
                return Err(Failure::Invalid("source outside trusted roots".into()));
            }
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|e| Failure::Invalid(e.to_string()))?;
            if metadata.file_type().is_symlink() {
                if query.source_files.is_some() {
                    return Err(Failure::Invalid("explicit symlink source refused".into()));
                }
                result
                    .excluded
                    .push((path.clone(), "symlink not traversed".into()));
                return Ok(());
            }
            let physical = host
                .normalized(&path)
                .ok_or_else(|| Failure::Invalid("physical path outside authorized scope".into()))?;
            request.work(Work::Exclusion, false)?;
            if excluded(host.roots(), &physical) {
                if query.source_files.is_some() {
                    return Err(Failure::Invalid(
                        "physical dependency source refused".into(),
                    ));
                }
                result
                    .excluded
                    .push((path.clone(), "physical dependency target".into()));
                return Ok(());
            }
            if !scope.admits(
                physical
                    .to_str()
                    .ok_or_else(|| Failure::Invalid("lossy path".into()))?,
            ) {
                if query.source_files.is_some() {
                    return Err(Failure::Invalid(
                        "explicit source outside narrowed scope".into(),
                    ));
                }
                if !metadata.is_dir() {
                    return Ok(());
                }
            }
            if metadata.is_dir() {
                if query.source_files.is_some() {
                    return Err(Failure::Invalid("explicit source must be a file".into()));
                }
                if !directories.insert(physical.clone()) {
                    return Ok(());
                }
                let entries = host.entries(&physical).map_err(|e| {
                    host.failure()
                        .map_or_else(|| Failure::Invalid(e), Failure::Budget)
                })?;
                for entry in entries {
                    let permit = request.reserve_input(Cost {
                        units: 1,
                        edges: 1,
                        retained_bytes: bounded::path_cost(&entry)?,
                        ..Cost::default()
                    })?;
                    result._reservations.push(permit);
                    pending.push(entry);
                }
            } else if !metadata.is_file() {
                return Err(Failure::Invalid(
                    "non-regular application source refused".into(),
                ));
            } else if matches!(
                physical.extension().and_then(|v| v.to_str()),
                Some("ts" | "tsx")
            ) && !result.files.contains(&physical)
            {
                let permit = request.reserve_input(Cost {
                    retained_bytes: bounded::path_cost(&physical)?,
                    ..Cost::default()
                })?;
                result._reservations.push(permit);
                result.files.push(physical);
            } else if query.source_files.is_some() {
                return Err(Failure::Invalid(
                    "explicit source is not supported TypeScript".into(),
                ));
            }
            Ok(())
        })();
        if let Some(stop) = host.failure() {
            result.complete = false;
            result.omissions.push(format!("inventory budget: {stop:?}"));
            break;
        }
        match step {
            Ok(()) => {}
            Err(Failure::Budget(stop)) => {
                result.complete = false;
                result.omissions.push(format!("inventory budget: {stop:?}"));
                break;
            }
            Err(error) if query.source_files.is_some() => return Err(error),
            Err(error) => {
                result.complete = false;
                result
                    .omissions
                    .push(format!("{}: {error:?}", path.display()));
            }
        }
    }
    if !pending.is_empty() {
        result
            .omissions
            .push(format!("{} unexamined inventory entries", pending.len()));
    }
    result.files.sort();
    let (proof, held) = host.into_parts();
    result._reservations.extend(held);
    let retained = request.reserve_nested(
        Cost {
            retained_bytes: bounded::proof_cost(&proof)?,
            ..Cost::default()
        },
        true,
    )?;
    result._reservations.push(retained);
    result.witnesses = proof;
    result.dto_reservation = Some(
        request.reserve_nested(
            Cost {
                retained_bytes: bounded::proof_cost(&result.witnesses)?
                    .checked_mul(16)
                    .ok_or(Stop::RetainedBytes)?,
                ..Cost::default()
            },
            true,
        )?,
    );
    Ok(result)
}

impl From<Stop> for Failure {
    fn from(value: Stop) -> Self {
        Self::Budget(value)
    }
}
