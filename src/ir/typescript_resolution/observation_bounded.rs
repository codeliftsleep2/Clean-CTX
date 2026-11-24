//! Integrated-only source acquisition and capped witness validation. V1 is untouched.
use super::request_budget::{Cost, Permit, Request, Reserved, Stop, Work};
use super::{Admitted, Candidate, Host, Refusal, Witness};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::Read,
    path::{Path, PathBuf},
};

pub(crate) fn envelope(bytes: usize, factor: u64) -> Result<u64, Stop> {
    (bytes as u64)
        .checked_mul(factor)
        .and_then(|v| v.checked_add(16384))
        .ok_or(Stop::RetainedBytes)
}
pub(crate) fn proof_cost(witnesses: &[Witness]) -> Result<u64, Stop> {
    let mut total = 4096u64;
    for w in witnesses {
        let paths: &[PathBuf] = match w {
            Witness::Directory { entries, .. } => entries,
            _ => &[],
        };
        let path = match w {
            Witness::Directory { path, .. }
            | Witness::Content { path, .. }
            | Witness::Missing { path }
            | Witness::Kind { path, .. }
            | Witness::Physical { path, .. }
            | Witness::Unreadable { path }
            | Witness::OutsideScope { path } => path,
        };
        for p in std::iter::once(path).chain(paths) {
            total = total
                .checked_add(path_cost(p)?)
                .ok_or(Stop::RetainedBytes)?;
        }
        if let Witness::Physical { target, .. } = w {
            total = total
                .checked_add(path_cost(target)?)
                .ok_or(Stop::RetainedBytes)?;
        }
    }
    Ok(total)
}
pub(crate) fn path_cost(path: &Path) -> Result<u64, Stop> {
    (path.as_os_str().len() as u64)
        .checked_mul(8)
        .and_then(|v| v.checked_add(256))
        .ok_or(Stop::RetainedBytes)
}
fn candidate_cost(candidate: &Candidate) -> Result<u64, Stop> {
    let mut bytes = proof_cost(&candidate.witnesses)?
        .checked_add(path_cost(&candidate.source)?)
        .ok_or(Stop::RetainedBytes)?;
    bytes = bytes
        .checked_add(
            (candidate.observations.capacity() as u64)
                .checked_mul(std::mem::size_of::<super::MappingObservation>() as u64)
                .ok_or(Stop::RetainedBytes)?,
        )
        .ok_or(Stop::RetainedBytes)?;
    bytes = bytes
        .checked_add(
            (candidate.omissions.capacity() as u64)
                .checked_mul(std::mem::size_of::<super::Omission>() as u64)
                .ok_or(Stop::RetainedBytes)?,
        )
        .ok_or(Stop::RetainedBytes)?;
    for record in &candidate.observations {
        let text = match &record.expression {
            super::WrittenExpression::ReferenceText { text, .. } => text.capacity(),
            super::WrittenExpression::Unsupported {
                text, syntax_kind, ..
            } => text
                .capacity()
                .checked_add(syntax_kind.capacity())
                .ok_or(Stop::RetainedBytes)?,
        };
        let n = record
            .key
            .capacity()
            .checked_add(record.route_path.capacity())
            .and_then(|v| v.checked_add(text))
            .ok_or(Stop::RetainedBytes)?;
        bytes = bytes
            .checked_add((n as u64).checked_mul(2).ok_or(Stop::RetainedBytes)?)
            .ok_or(Stop::RetainedBytes)?;
    }
    Ok(bytes)
}
pub(crate) fn dto_cost(source: &Admitted) -> Result<u64, Stop> {
    candidate_cost(&source.candidate)?
        .checked_mul(16)
        .and_then(|v| v.checked_add(16384))
        .ok_or(Stop::RetainedBytes)
}
pub(crate) fn capture(
    request: &Request,
    roots: &[PathBuf],
    file: &Path,
) -> Result<Reserved<Result<Candidate, Refusal>>, Stop> {
    assert!(request.is_integrated());
    let active = request.reserve_optional(Cost::default())?;
    for _ in roots {
        request.work(Work::Physical, false)?;
    }
    let mut host = match Host::new(roots) {
        Ok(host) => host,
        Err(_) => {
            return Ok(request.hold(
                Err(Refusal::UnreadableOrUnauthorized),
                active,
                request.checkpoint().err(),
            ));
        }
    };
    host.direct_budget = Some(request.clone());
    host.direct_source = true;
    let source = host.normalized(file);
    let bytes = source
        .as_ref()
        .filter(|p| application_path(&host.roots, p))
        .and_then(|p| if host.file(p) { host.read(p) } else { None });
    if let Some(stop) = host.direct_failure {
        return Err(stop);
    }
    let (Some(source), Some(bytes)) = (source, bytes) else {
        return Ok(request.hold(
            Err(Refusal::UnreadableOrUnauthorized),
            active,
            request.checkpoint().err(),
        ));
    };
    let parsed = request.reserve_input(Cost {
        units: 1,
        retained_bytes: envelope(bytes.len(), 64)?,
        ..Cost::default()
    })?;
    let language = if matches!(
        source.extension().and_then(|x| x.to_str()),
        Some("tsx" | "jsx")
    ) {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else {
        crate::compression::language::safe_typescript_language().ok_or(Stop::Units)?
    };
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).map_err(|_| Stop::Units)?;
    #[cfg(test)]
    crate::ir::typescript_resolution::e1_metrics::parse(&source, bytes.len());
    #[cfg(feature = "resolver_evidence_v2")]
    super::request_budget::telemetry::parse(&source);
    let result = super::super::syntax_frontend::parse(&mut parser, &bytes)
        .map(|(tree, _)| {
            let mut scan_stop = None;
            let (observations, omissions) = super::extract_with(tree.root_node(), &bytes, || {
                match request.reserve_input(Cost {
                    units: 1,
                    ..Cost::default()
                }) {
                    Ok(_charge) => true,
                    Err(stop) => {
                        scan_stop = Some(stop);
                        false
                    }
                }
            });
            Candidate {
                source,
                source_hash: crate::cache::content_hash(bytes.as_bytes()),
                observations,
                omissions,
                witnesses: host.witnesses.into_vec(),
                source_bytes: bytes.len(),
                scan_stop,
            }
        })
        .ok_or(Refusal::MalformedSource);
    let retained = request.reserve_input(Cost {
        retained_bytes: result
            .as_ref()
            .ok()
            .map(candidate_cost)
            .transpose()?
            .unwrap_or(0),
        ..Cost::default()
    })?;
    drop(parsed);
    #[cfg(test)]
    test_hooks::after_capture(file);
    drop(active);
    Ok(request.hold(result, retained, request.checkpoint().err()))
}

fn content(request: &Request, path: &Path, expected_hash: &str) -> Result<bool, Stop> {
    request.work(Work::Metadata, true)?;
    let Ok(mut file) = std::fs::File::open(path) else {
        return Ok(false);
    };
    #[cfg(windows)]
    {
        request.work(Work::Physical, true)?;
        let Ok(opened) = super::super::host_windows::opened_path(&file) else {
            return Ok(false);
        };
        request.work(Work::Physical, true)?;
        let Ok(current) = path.canonicalize() else {
            return Ok(false);
        };
        if super::super::host::shape(&opened) != super::super::host::shape(&current) {
            return Ok(false);
        }
    }
    request.work(Work::Metadata, true)?;
    let Ok(metadata) = file.metadata() else {
        return Ok(false);
    };
    let expected = metadata.len();
    #[cfg(test)]
    test_hooks::after_metadata(path);
    let limit = expected.checked_add(1).ok_or(Stop::ValidationBytes)?;
    let mut remaining = limit;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 4096];
    let mut read = 0u64;
    while remaining > 0 {
        request.checkpoint_final()?;
        let size = remaining.min(buffer.len() as u64) as usize;
        let _capacity = request.reserve_nested(
            Cost {
                validation_bytes: size as u64,
                ..Cost::default()
            },
            true,
        )?;
        let Ok(n) = file.read(&mut buffer[..size]) else {
            return Ok(false);
        };
        if n == 0 {
            break;
        }
        #[cfg(test)]
        crate::ir::typescript_resolution::e1_metrics::read(path, n, true);
        #[cfg(feature = "resolver_evidence_v2")]
        super::request_budget::telemetry::read(n, true);
        read += n as u64;
        hash.update(&buffer[..n]);
        remaining -= n as u64;
    }
    Ok(read == expected && format!("{:x}", hash.finalize()) == expected_hash)
}
pub(crate) struct Boundary<'a> {
    request: &'a Request,
    seen: HashMap<Witness, bool>,
    retained: Vec<Permit>,
    elapsed: std::time::Duration,
}
impl<'a> Boundary<'a> {
    pub(crate) fn new(request: &'a Request) -> Self {
        Self {
            request,
            seen: HashMap::new(),
            retained: Vec::new(),
            elapsed: std::time::Duration::ZERO,
        }
    }
    pub(crate) fn validate(&mut self, witnesses: &[Witness]) -> Result<Reserved<bool>, Stop> {
        let start = std::time::Instant::now();
        let result = self.validate_inner(witnesses, false);
        self.elapsed += start.elapsed();
        result
    }
    /// The integrated Source/DTO owner holds the implementation-computed allocation
    /// envelope through this boundary and response lifetime, including cache clones.
    pub(crate) fn pre_reserved(&mut self, witnesses: &[Witness]) -> Result<Reserved<bool>, Stop> {
        let start = std::time::Instant::now();
        let result = self.validate_inner(witnesses, true);
        self.elapsed += start.elapsed();
        result
    }
    pub(crate) fn elapsed(&self) -> std::time::Duration {
        self.elapsed
    }
    fn validate_inner(
        &mut self,
        witnesses: &[Witness],
        credited: bool,
    ) -> Result<Reserved<bool>, Stop> {
        let request = self.request;
        let permit = request.reserve_finalization(Cost {
            retained_bytes: if credited {
                0
            } else {
                envelope(witnesses.len(), 64)?
            },
            ..Cost::default()
        })?;
        let mut valid = true;
        for witness in witnesses {
            if let Some(current) = self.seen.get(witness) {
                if !*current {
                    valid = false;
                    break;
                }
                continue;
            }
            if !credited {
                self.retained.push(request.reserve_nested(
                    Cost {
                        retained_bytes: proof_cost(std::slice::from_ref(witness))?,
                        ..Cost::default()
                    },
                    true,
                )?);
            }
            let _charge = request.reserve_nested(
                Cost {
                    validation_checks: 1,
                    ..Cost::default()
                },
                true,
            )?;
            #[cfg(test)]
            crate::work_observations_af_perf_002::witness(witness);
            #[cfg(feature = "resolver_evidence_v2")]
            super::request_budget::telemetry::witness();
            valid = match witness {
                Witness::Content { path, hash } => content(request, path, hash)?,
                Witness::Physical { path, target } => {
                    request.work(Work::Physical, true)?;
                    path.canonicalize().is_ok_and(|p| {
                        super::super::host::shape(&p) == super::super::host::shape(target)
                    })
                }
                Witness::Missing { path } => {
                    request.work(Work::Metadata, true)?;
                    std::fs::symlink_metadata(path)
                        .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                }
                Witness::Kind {
                    path,
                    file,
                    directory,
                } => {
                    request.work(Work::Metadata, true)?;
                    std::fs::metadata(path)
                        .is_ok_and(|m| m.is_file() == *file && m.is_dir() == *directory)
                }
                Witness::Directory { path, entries } => {
                    request.work(Work::Directory, true)?;
                    let mut current = Vec::new();
                    let mut retained: Vec<Permit> = Vec::new();
                    if let Ok(mut directory) = std::fs::read_dir(path) {
                        let mut readable = true;
                        loop {
                            request.work(Work::DirectoryEntry, true)?;
                            let Some(item) = directory.next() else {
                                break;
                            };
                            let Ok(item) = item else {
                                readable = false;
                                break;
                            };
                            let path = item.path();
                            if !credited {
                                retained.push(request.reserve_nested(
                                    Cost {
                                        retained_bytes: path_cost(&path)?,
                                        ..Cost::default()
                                    },
                                    true,
                                )?);
                            }
                            current.push(path);
                            if current.len() > entries.len() {
                                readable = false;
                                break;
                            }
                        }
                        current.sort();
                        readable && current == *entries
                    } else {
                        false
                    }
                }
                Witness::Unreadable { .. } | Witness::OutsideScope { .. } => false,
            };
            self.seen.insert(witness.clone(), valid);
            if !valid {
                break;
            }
        }
        let late = request.checkpoint_final().err();
        Ok(request.hold(valid, permit, late))
    }
}
pub(crate) fn validate(request: &Request, witnesses: &[Witness]) -> Result<Reserved<bool>, Stop> {
    Boundary::new(request).validate(witnesses)
}

pub(crate) fn source_bytes(source: &Admitted) -> usize {
    source.candidate.source_bytes
}
pub(crate) fn scan_complete(source: &Admitted) -> bool {
    source.candidate.scan_stop.is_none()
}
pub(crate) fn scan_stop(source: &Admitted) -> Option<Stop> {
    source.candidate.scan_stop
}
pub(crate) fn witnesses(source: &Admitted) -> &[Witness] {
    &source.candidate.witnesses
}
pub(crate) fn application_path(roots: &[PathBuf], path: &Path) -> bool {
    if path
        .components()
        .any(|p| p.as_os_str().eq_ignore_ascii_case("node_modules"))
    {
        return false;
    }
    let relative = roots
        .iter()
        .filter_map(|root| path.strip_prefix(root).ok())
        .min_by_key(|p| p.components().count())
        .unwrap_or(path);
    !relative.components().any(|p| {
        [
            ".git",
            "bin",
            "obj",
            "dist",
            "build",
            "target",
            ".venv",
            "venv",
            "__pycache__",
        ]
        .iter()
        .any(|v| p.as_os_str().eq_ignore_ascii_case(v))
    })
}
#[cfg(test)]
#[path = "../../tests/ir/resolver_evidence_e6b_hooks.rs"]
pub(crate) mod test_hooks;

/// Narrow inventory facade over the existing physical/source authority.
pub(crate) struct BoundedHost {
    host: Host,
}
impl BoundedHost {
    pub(crate) fn new(request: &Request, roots: &[PathBuf]) -> Result<Self, String> {
        if !request.is_integrated() {
            return Err("integrated ledger required".into());
        }
        for _ in roots {
            request
                .work(Work::Physical, false)
                .map_err(|e| format!("{e:?}"))?;
        }
        let mut host = Host::new(roots)?;
        host.direct_budget = Some(request.clone());
        Ok(Self { host })
    }
    pub(crate) fn roots(&self) -> &[PathBuf] {
        &self.host.roots
    }
    pub(crate) fn allows_path(&self, path: &Path) -> bool {
        self.host
            .boundary(&super::super::host::lexical(&super::super::host::shape(
                path,
            )))
            .is_some()
    }
    pub(crate) fn normalized(&mut self, path: &Path) -> Option<PathBuf> {
        self.host.normalized(path)
    }
    pub(crate) fn entries(&mut self, path: &Path) -> Result<Vec<PathBuf>, String> {
        self.host.entries(path)
    }
    pub(crate) fn failure(&self) -> Option<Stop> {
        self.host.direct_failure
    }
    pub(crate) fn into_parts(self) -> (Vec<Witness>, Vec<Permit>) {
        (
            self.host.witnesses.into_vec(),
            self.host.direct_reservations,
        )
    }
}
