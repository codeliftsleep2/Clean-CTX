//! Bounded direct value-namespace evidence. Never canonical Angular admission.
use super::request_budget::{Cost, Request, Reserved, Stop};
use super::{Admitted, Span, WrittenExpression};
use crate::ir::typescript_resolution::{
    Resolver, Witness,
    syntax::{Binding, Unit},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub source: PathBuf,
    pub source_hash: String,
    pub span: Span,
    pub name: String,
    pub kind: String,
}
impl From<crate::ir::typescript_resolution::DeclarationId> for Declaration {
    fn from(id: crate::ir::typescript_resolution::DeclarationId) -> Self {
        Self {
            source: id.source,
            source_hash: id.source_hash,
            span: Span {
                start: id.start,
                end: id.end,
            },
            name: id.name,
            kind: id.kind,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Obligation {
    Budget(Stop),
    SourceChanged,
    UnsupportedExpression,
    UnsupportedLexicalScope,
    MissingBinding,
    ConflictingDeclarations,
    TypeOnly,
    UnsupportedAlias,
    UnclosedStarExports,
    Cycle,
    UnsupportedSyntax,
    Resolution(String),
    ContextualContributorsUnchecked,
}
#[derive(Clone, Debug)]
pub struct Edge {
    pub request: PathBuf,
    pub specifier: String,
    pub target: PathBuf,
}
#[derive(Clone, Debug)]
pub struct Manifest {
    pub path: PathBuf,
    pub hash: String,
    pub name: String,
    pub version: Option<String>,
}
/// Exact direct declaration reached, plus all unsatisfied binding obligations.
/// Imported endpoints do not establish full TS runtime binding without context.
pub struct Evidence {
    authorization_roots: Vec<PathBuf>,
    type_space: bool,
    annotation: Option<Span>,
    contextual_modules: Vec<String>,
    source: PathBuf,
    source_hash: String,
    mapping_span: Span,
    target: Option<Declaration>,
    edges: Vec<Edge>,
    manifests: Vec<Manifest>,
    obligations: Vec<Obligation>,
    witnesses: Vec<Witness>,
    validation_bytes: u64,
    _retained: Option<Reserved<()>>,
    _bounded_proof: Option<super::request_budget::Permit>,
}
#[path = "direct_evidence.rs"]
mod owned_evidence;
/// Explicit per-parse/result retained envelope; no production defaults.
#[derive(Clone, Copy)]
pub struct Envelopes {
    pub parse_retained_bytes: u64,
    pub result_retained_bytes: u64,
}

struct Walk<'a> {
    type_space: bool,
    request: &'a Request,
    resolver: Resolver,
    units: BTreeMap<PathBuf, Reserved<Unit>>,
    active: BTreeSet<(PathBuf, String, bool)>,
    edges: Vec<Edge>,
    external: bool,
    envelopes: Envelopes,
}
impl Walk<'_> {
    fn check(&self) -> Result<(), Obligation> {
        self.request.checkpoint().map_err(Obligation::Budget)
    }
    fn unit(&mut self, file: &Path) -> Result<(), Obligation> {
        self.check()?;
        if self.units.contains_key(file) {
            return Ok(());
        }
        let reading = self
            .request
            .reserve_optional(Cost::default())
            .map_err(Obligation::Budget)?;
        let text = self
            .resolver
            .host
            .read(file)
            .ok_or_else(|| self.failure("source unavailable"))?;
        drop(reading);
        let parsed = self
            .request
            .run_optional(
                Cost {
                    units: 1,
                    retained_bytes: if self.request.is_integrated() {
                        super::bounded::envelope(text.len(), 64).map_err(Obligation::Budget)?
                    } else {
                        self.envelopes.parse_retained_bytes
                    },
                    ..Cost::default()
                },
                |_| {
                    // Unit binding keys preserve spelling. Until identifier-escape
                    // equivalence is certified here, do not infer uniqueness from
                    // distinct raw spellings (including escaped competing names).
                    if text.contains("\\u") {
                        None
                    } else {
                        Unit::parse(file, &text)
                    }
                },
            )
            .map_err(Obligation::Budget)?;
        if let Some(stop) = parsed.completed_after_stop {
            return Err(Obligation::Budget(stop));
        }
        let parsed = parsed.map(|unit| unit.ok_or(Obligation::UnsupportedSyntax));
        if parsed.value().is_err() {
            return Err(Obligation::UnsupportedSyntax);
        }
        let unit = parsed.map(|u| u.expect("checked unit"));
        if unit.value().invalid {
            return Err(Obligation::UnsupportedSyntax);
        }
        self.units.insert(file.to_path_buf(), unit);
        Ok(())
    }
    fn failure(&self, reason: &str) -> Obligation {
        self.resolver
            .host
            .direct_failure
            .map_or_else(|| Obligation::Resolution(reason.into()), Obligation::Budget)
    }
    fn module(
        &mut self,
        file: &Path,
        specifier: &str,
        mode: crate::ir::typescript_resolution::module_resolution::ModuleRequestKind,
    ) -> Result<PathBuf, Obligation> {
        self.check()?;
        let result = self
            .request
            .run_optional(
                Cost {
                    edges: 1,
                    ..Cost::default()
                },
                |_| self.resolver.module_request(file, specifier, mode),
            )
            .map_err(Obligation::Budget)?;
        if let Some(stop) = result.completed_after_stop {
            return Err(Obligation::Budget(stop));
        }
        if let Some(stop) = self.resolver.host.direct_failure {
            return Err(Obligation::Budget(stop));
        }
        let target = result
            .value()
            .as_ref()
            .map_err(|e| self.failure(e))?
            .clone();
        self.edges.push(Edge {
            request: file.to_path_buf(),
            specifier: specifier.into(),
            target: target.clone(),
        });
        self.external = true;
        Ok(target)
    }
    fn binding(
        &mut self,
        file: &Path,
        name: &str,
        exported: bool,
        member: Option<&str>,
    ) -> Result<Declaration, Obligation> {
        self.check()?;
        // Every binding hop is charged, including local aliases and cycle attempts.
        let charge = self
            .request
            .reserve_optional(Cost {
                edges: 1,
                ..Cost::default()
            })
            .map_err(Obligation::Budget)?;
        drop(charge);
        let visit = (file.to_path_buf(), name.to_owned(), exported);
        if !self.active.insert(visit.clone()) {
            return Err(Obligation::Cycle);
        }
        let result = self.binding_inner(file, name, exported, member);
        self.active.remove(&visit);
        result
    }
    fn binding_inner(
        &mut self,
        file: &Path,
        name: &str,
        exported: bool,
        member: Option<&str>,
    ) -> Result<Declaration, Obligation> {
        self.unit(file)?;
        let unit = self.units.get(file).expect("loaded unit").value();
        let bindings = if exported {
            &unit.exports
        } else {
            &unit.locals
        };
        let Some(bindings) = bindings.get(name) else {
            return Err(if exported && !unit.stars.is_empty() {
                Obligation::UnclosedStarExports
            } else {
                Obligation::MissingBinding
            });
        };
        if bindings.len() != 1 {
            return Err(Obligation::ConflictingDeclarations);
        }
        let binding = bindings[0].clone();
        match binding {
            Binding::Declaration(id, runtime)
                if (runtime || self.type_space) && member.is_none() =>
            {
                Ok(id.into())
            }
            Binding::Declaration(_, false) => Err(Obligation::TypeOnly),
            Binding::Import {
                module,
                imported,
                mode,
                type_only,
            } if (!type_only || self.type_space) && member.is_none() => {
                let path = self.module(file, &module, mode)?;
                self.binding(&path, &imported, true, None)
            }
            Binding::Namespace {
                module,
                mode,
                type_only,
            } if !type_only || self.type_space => {
                let member = member.ok_or(Obligation::UnsupportedExpression)?;
                let path = self.module(file, &module, mode)?;
                self.binding(&path, member, true, None)
            }
            Binding::Import {
                type_only: true, ..
            }
            | Binding::Namespace {
                type_only: true, ..
            }
            | Binding::Require {
                type_only: true, ..
            } => Err(Obligation::TypeOnly),
            Binding::Alias(parts) if parts.len() == 1 && member.is_none() => {
                self.binding(file, &parts[0], false, None)
            }
            _ => Err(Obligation::UnsupportedAlias),
        }
    }
}

/// Capture direct evidence for one exact already admitted mapping observation.
/// Existing observations remain owned by the caller even if this result is partial.
pub fn capture(
    request: &Request,
    observation: &Admitted,
    index: usize,
    roots: &[PathBuf],
    envelopes: Envelopes,
) -> Result<Evidence, Obligation> {
    capture_kind(request, observation, index, roots, envelopes, false, None)
}
pub(crate) fn capture_kind(
    request: &Request,
    observation: &Admitted,
    index: usize,
    roots: &[PathBuf],
    envelopes: Envelopes,
    type_space: bool,
    public_demand: Option<(&str, &str)>,
) -> Result<Evidence, Obligation> {
    let Some(record) = observation.observations().get(index) else {
        return Err(Obligation::MissingBinding);
    };
    let retained = request
        .run_optional(
            Cost {
                retained_bytes: envelopes.result_retained_bytes,
                ..Cost::default()
            },
            |_| (),
        )
        .map_err(Obligation::Budget)?;
    let root_task = request
        .reserve_optional(Cost::default())
        .map_err(Obligation::Budget)?;
    for _ in roots {
        request
            .work(super::request_budget::Work::Physical, false)
            .map_err(Obligation::Budget)?;
    }
    let mut resolver = Resolver::new(roots).map_err(Obligation::Resolution)?;
    drop(root_task);
    resolver.host.direct_budget = Some(request.clone());
    let file = observation.source().to_path_buf();
    let mut walk = Walk {
        type_space,
        request,
        resolver,
        units: BTreeMap::new(),
        active: BTreeSet::new(),
        edges: Vec::new(),
        external: false,
        envelopes,
    };
    let mut annotation = None;
    let outcome = (|| {
        walk.unit(&file)?;
        if !walk.units.get(&file).expect("loaded unit").value().external {
            // Script/global contributors are deliberately not closed by this slice.
            walk.external = true;
        }
        let text = walk
            .resolver
            .host
            .read(&file)
            .ok_or(Obligation::SourceChanged)?;
        if crate::cache::content_hash(text.as_bytes()) != observation.source_hash() {
            return Err(Obligation::SourceChanged);
        }
        if type_space {
            let tree = walk
                .units
                .get(&file)
                .expect("unit")
                .value()
                .tree
                .as_ref()
                .ok_or(Obligation::UnsupportedSyntax)?;
            let (name, member, span) = role_annotation::select(tree, record.object, &text)?;
            if let Some((module, export)) = public_demand {
                let unit = walk.units.get(&file).expect("unit").value();
                if !role_annotation::direct_import(unit, &name, member.as_deref(), module, export) {
                    return Err(Obligation::UnsupportedAlias);
                }
            }
            annotation = Some(span);
            return walk.binding(&file, &name, false, member.as_deref());
        }
        let WrittenExpression::ReferenceText {
            text: expression,
            span,
        } = &record.expression
        else {
            return Err(Obligation::UnsupportedExpression);
        };
        let tree = walk
            .units
            .get(&file)
            .expect("loaded unit")
            .value()
            .tree
            .as_ref()
            .ok_or(Obligation::UnsupportedSyntax)?;
        let node = tree
            .root_node()
            .named_descendant_for_byte_range(span.start, span.end)
            .ok_or(Obligation::UnsupportedSyntax)?;
        if node.start_byte() != span.start || node.end_byte() != span.end {
            return Err(Obligation::UnsupportedSyntax);
        }
        let mut parent = node.parent();
        while let Some(p) = parent {
            if matches!(
                p.kind(),
                "statement_block"
                    | "function_declaration"
                    | "function_expression"
                    | "arrow_function"
                    | "method_definition"
                    | "class_body"
                    | "internal_module"
                    | "module"
                    | "catch_clause"
                    | "for_statement"
                    | "for_in_statement"
            ) {
                return Err(Obligation::UnsupportedLexicalScope);
            }
            parent = p.parent();
        }
        // Only exact identifier or one namespace member. No string-derived aliases.
        let mut parts = expression.split('.');
        let name = parts.next().ok_or(Obligation::UnsupportedExpression)?;
        let member = parts.next();
        if parts.next().is_some() || expression.contains(char::is_whitespace) {
            return Err(Obligation::UnsupportedExpression);
        }
        walk.binding(&file, name, false, member)
    })();
    let mut obligations = Vec::new();
    let target = match outcome {
        Ok(id) => Some(id),
        Err(e) => {
            obligations.push(e);
            None
        }
    };
    if target.is_some() && walk.external {
        obligations.push(Obligation::ContextualContributorsUnchecked);
    }
    let mut manifests = Vec::new();
    for (p, bytes) in &walk.resolver.host.sources {
        if p.file_name().and_then(|n| n.to_str()) != Some("package.json") {
            continue;
        }
        let Some(bytes) = bytes else {
            continue;
        };
        let permit = match request.reserve_optional(Cost {
            units: 1,
            ..Cost::default()
        }) {
            Ok(p) => p,
            Err(stop) => {
                obligations.push(Obligation::Budget(stop));
                break;
            }
        };
        let Ok(ordered) = crate::ir::typescript_resolution::package::Ordered::parse(bytes) else {
            continue;
        };
        if let Some(name) = ordered.get("name").and_then(|v| v.string()) {
            manifests.push(Manifest {
                path: p.clone(),
                hash: crate::cache::content_hash(bytes.as_bytes()),
                name: name.into(),
                version: ordered
                    .get("version")
                    .and_then(|v| v.string())
                    .map(str::to_owned),
            });
        }
        drop(permit);
    }
    let validation_bytes: u64 = walk
        .resolver
        .host
        .sources
        .values()
        .flatten()
        .map(|s| s.len() as u64)
        .sum();
    let validation_bytes = validation_bytes + observation.candidate.source_bytes as u64;
    let contextual_modules: Vec<String> = walk
        .units
        .values()
        .flat_map(|u| {
            let u = u.value();
            u.namespace_bodies
                .keys()
                .filter(|id| id.kind == "ambient_module")
                .map(|id| id.name.clone())
                .chain(u.augmentations.iter().map(|(name, _)| name.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    let authorization_roots = walk.resolver.host.roots.clone();
    let mut witnesses = walk.resolver.host.witnesses.into_vec();
    // Never validate new source bytes as support for an older observation identity.
    witnesses.extend(observation.candidate.witnesses.iter().cloned());
    let bounded_proof = if request.is_integrated() {
        let mut size = super::bounded::proof_cost(&witnesses).map_err(Obligation::Budget)?;
        for text in manifests
            .iter()
            .flat_map(|m| std::iter::once(m.name.as_str()).chain(m.version.as_deref()))
            .chain(walk.edges.iter().map(|e| e.specifier.as_str()))
            .chain(contextual_modules.iter().map(String::as_str))
            .chain(target.iter().map(|d| d.name.as_str()))
        {
            size = size
                .checked_add(
                    (text.len() as u64)
                        .checked_mul(4)
                        .ok_or(Obligation::Budget(Stop::RetainedBytes))?,
                )
                .ok_or(Obligation::Budget(Stop::RetainedBytes))?;
        }
        Some(
            request
                .reserve_input(Cost {
                    retained_bytes: size,
                    ..Cost::default()
                })
                .map_err(Obligation::Budget)?,
        )
    } else {
        None
    };
    // No source bodies/program/tree survive the direct evidence boundary.
    Ok(Evidence {
        authorization_roots,
        type_space,
        annotation,
        contextual_modules,
        source: file,
        source_hash: observation.source_hash().into(),
        mapping_span: record.mapping_pair,
        target,
        edges: walk.edges,
        manifests,
        obligations,
        witnesses,
        validation_bytes,
        _retained: Some(retained),
        _bounded_proof: bounded_proof,
    })
}

#[path = "role_annotation.rs"]
mod role_annotation;

#[cfg(test)]
#[path = "../../tests/ir/written_source_identity_e4.rs"]
mod tests;
