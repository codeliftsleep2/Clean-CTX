//! EF owns obligations; inventory/publication/compilation remain shared mechanics.
use super::*;
use crate::ir::csharp_resolution::{CSharpResolution, CSharpTypeIdentity};
use crate::mcp::McpState;
use crate::mcp::acquisition::{InventoryManifest, SourceKind};
use crate::mcp::semantic_compilation::dependencies::PublicationStamp;
use crate::mcp::tool_handlers::hydration::{
    dependency_roots, publish_dependency_frontier, source_inventory,
};
use crate::workspace::scope::WorkspaceScope;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug)]
enum Kind {
    ModelEntity,
    ExplicitConfiguration,
    ConfigurationEntity,
    ConfigurationContract,
    AssemblyMarker,
}

/// Original scope is retained even when resolution is currently unsuccessful.
struct Obligation {
    kind: Kind,
    context: DeclarationKey,
    scope: Option<WorkspaceScope>,
    reference: BaseReference,
    disposition: CSharpResolution<CSharpTypeIdentity>,
}

#[derive(Clone)]
pub(in crate::mcp::tool_handlers::query) struct Evidence {
    stamp: PublicationStamp,
    complete: bool,
    inventory: String,
    projects: Vec<(String, Option<PathBuf>)>,
    sources: BTreeSet<String>,
}

impl Evidence {
    pub(in crate::mcp::tool_handlers::query) fn publication_stamp(&self) -> PublicationStamp {
        self.stamp
    }
    pub(in crate::mcp::tool_handlers::query) fn sources(&self) -> &BTreeSet<String> {
        &self.sources
    }
    pub(in crate::mcp::tool_handlers::query) fn current(&self, index: &WorkspaceIndex) -> bool {
        self.stamp.is_current(index)
            && self
                .projects
                .iter()
                .all(|(file, project)| nearest_project(file) == *project)
    }

    pub(in crate::mcp::tool_handlers::query) fn apply(
        &self,
        index: &WorkspaceIndex,
        coverage: &mut Value,
    ) {
        // Ordinary edge coverage stays a lower bound even when obligations close.
        if self.complete && self.current(index) {
            return;
        }
        coverage["dependency_proof"] = json!({"status":"unproven"});
        coverage["source_complete"] = json!(false);
        coverage["omitted_possible"] = json!(true);
        coverage["result_semantics"] = json!("lower_bound");
        coverage["status"] = json!("semantic_dependencies_unresolved");
        let reason = if self.current(index) {
            "dependency_unresolved"
        } else {
            "dependency_publication_changed"
        };
        let mut reasons = coverage["reasons"].as_array().cloned().unwrap_or_default();
        if !reasons.contains(&json!(reason)) {
            reasons.push(json!(reason));
        }
        coverage["reasons"] = json!(reasons);
        if let Some(requirements) = coverage
            .get_mut("evidence_requirements")
            .and_then(Value::as_array_mut)
        {
            requirements.retain(|value| value != "dependencies_current");
        }
    }
}

pub(in crate::mcp::tool_handlers::query) fn prepare(
    state: &McpState,
    owners: &[String],
    root: Option<&str>,
    scope: Option<&WorkspaceScope>,
    reverse: bool,
    previous: Option<&Evidence>,
    identity: &str,
) -> Result<Evidence, String> {
    use crate::mcp::discovery_cache::DiscoveryMode;
    use crate::mcp::hydration_flight::HydrationFlightKey;
    use crate::workspace::index::SemanticFidelity;
    let roots = dependency_roots(state, root);
    // This is an EF synchronization namespace, not SourceDependencyClosure.
    // Followers execute attempt again and must establish their own evidence.
    let flight = HydrationFlightKey::new(
        roots
            .iter()
            .map(|root| root.to_string_lossy().into_owned())
            .collect(),
        DiscoveryMode::Declaration,
        format!("ef-model-v1:{identity}"),
        Some(SemanticFidelity::High),
        None,
    );
    state.hydration_flights.run(flight, || {
        attempt(state, owners, root, scope, reverse, previous)
    })
}

fn attempt(
    state: &McpState,
    owners: &[String],
    root: Option<&str>,
    scope: Option<&WorkspaceScope>,
    reverse: bool,
    previous: Option<&Evidence>,
) -> Result<Evidence, String> {
    let roots = dependency_roots(state, root);
    loop {
        #[cfg(test)]
        work_metrics::record_ef_dependency_inventory();
        let inventory = source_inventory(state, &roots);
        if let Some(previous) = previous
            && previous.complete
            && inventory.complete
            && previous.inventory == inventory.digest
            && previous.current(&state.workspace_index_read())
            && previous
                .projects
                .iter()
                .all(|(file, project)| nearest_project(file) == *project)
        {
            return Ok(previous.clone());
        }
        let mut selected: BTreeSet<String> = owners.iter().cloned().collect();
        if reverse {
            selected.extend(
                inventory
                    .entries
                    .iter()
                    .filter(|entry| {
                        entry.source_kind == SourceKind::CSharp
                            && entry.source.contains("OnModelCreating")
                            && scope.is_none_or(|scope| scope.admits(&entry.owner))
                    })
                    .map(|entry| entry.owner.clone()),
            );
        }
        let mut processed = BTreeSet::new();
        let mut published = true;
        let (complete, stamp, projects) = loop {
            let frontier: Vec<_> = selected.difference(&processed).cloned().collect();
            published &= publish_dependency_frontier(state, &frontier, root)?;
            processed.extend(frontier.iter().cloned());
            // Newly acquired context/configuration owners retain #155 authority.
            for owner in &frontier {
                state.prepare_csharp_semantic_owner(owner, root)?;
            }
            let index = state.workspace_index_read();
            let stamp = PublicationStamp::capture(state);
            let catalog = Catalog::build_dependency(state, &index, scope, &inventory);
            let (requirements, projects, parsed) = obligations(&catalog, &index, &selected, scope);
            drop(index);
            let before = selected.len();
            for requirement in &requirements {
                select(&inventory, requirement, &mut selected);
            }
            // The supported configuration predicate is source-bearing: direct
            // IEntityTypeConfiguration spelling or an alias target spelling.
            // Compilation proves membership; this spelling only narrows admission.
            for entry in &inventory.entries {
                if entry.source_kind == SourceKind::CSharp
                    && entry.source.contains("IEntityTypeConfiguration")
                    && scope.is_none_or(|scope| scope.admits(&entry.owner))
                    && nearest_project(&entry.owner)
                        .is_some_and(|project| projects.contains(&project))
                {
                    selected.insert(entry.owner.clone());
                }
            }
            if selected.len() == before {
                let projects: Vec<_> = selected
                    .iter()
                    .map(|file| (file.clone(), nearest_project(file)))
                    .collect();
                break (
                    published
                        && parsed
                        && inventory.complete
                        && requirements.iter().all(Obligation::resolved),
                    stamp,
                    projects,
                );
            }
        };
        state.prepare_exact_csharp_calls()?;
        // Source reads/publication may invalidate the captured root observation.
        // Rebuild against the surviving inventory rather than reuse that proof.
        #[cfg(test)]
        work_metrics::record_ef_dependency_inventory();
        let final_inventory = source_inventory(state, &roots);
        if inventory.digest != final_inventory.digest
            || !stamp.is_current(&state.workspace_index_read())
            || projects
                .iter()
                .any(|(file, project)| nearest_project(file) != *project)
        {
            continue;
        }
        let sources = final_inventory
            .entries
            .iter()
            .filter(|entry| {
                state
                    .csharp_call_authority
                    .artifact(&entry.owner)
                    .is_some_and(|artifact| artifact.source_hash() == entry.source_hash)
            })
            .map(|entry| entry.owner.clone())
            .collect();
        return Ok(Evidence {
            stamp,
            complete: complete && final_inventory.complete,
            inventory: final_inventory.digest,
            projects,
            sources,
        });
    }
}

fn obligations(
    catalog: &Catalog,
    index: &WorkspaceIndex,
    selected: &BTreeSet<String>,
    scope: Option<&WorkspaceScope>,
) -> (Vec<Obligation>, BTreeSet<PathBuf>, bool) {
    let sources = source_indexes(catalog, None);
    let mut result = Vec::new();
    let mut projects = BTreeSet::new();
    let mut explicit = BTreeSet::new();
    let mut contracts = BTreeSet::new();
    let mut complete = true;
    for context in &catalog.declarations {
        if !selected.contains(&context.file) || !has_ef_role(index, context) {
            continue;
        }
        let source = &sources;
        #[cfg(test)]
        work_metrics::record_ef_dependency_syntax_parse();
        let Some(model) = OnModelCreating::parse(&context.source) else {
            complete = false;
            continue;
        };
        for (kind, written) in model
            .direct_entities
            .into_iter()
            .map(|written| (Kind::ModelEntity, written))
            .chain(
                model
                    .explicit_configurations
                    .into_iter()
                    .map(|written| (Kind::ExplicitConfiguration, written)),
            )
            .chain(
                model
                    .assembly_markers
                    .into_iter()
                    .map(|written| (Kind::AssemblyMarker, written)),
            )
        {
            let Some(obligation) = obligation(catalog, source, context, kind, &written, scope)
            else {
                complete = false;
                continue;
            };
            if matches!(obligation.kind, Kind::AssemblyMarker) {
                if let Some(marker) =
                    declaration_for_resolution(catalog, source, obligation.disposition.clone())
                {
                    if let Some(project) = nearest_project(&context.file)
                        && nearest_project(&marker.file).as_ref() == Some(&project)
                    {
                        projects.insert(project);
                    } else {
                        complete = false;
                    }
                } else {
                    complete = false;
                }
            }
            if matches!(obligation.kind, Kind::ExplicitConfiguration)
                && let Some(configuration) =
                    declaration_for_resolution(catalog, source, obligation.disposition.clone())
            {
                explicit.insert((configuration.path.clone(), configuration.arity));
            }
            result.push(obligation);
        }
    }
    // Explicit declarations and assembly candidates both require the entity
    // named by their *resolved* framework contract, not their class spelling.
    for configuration in &catalog.declarations {
        if !selected.contains(&configuration.file) {
            continue;
        }
        let source = &sources;
        for reference in catalog.references_for_owner(&configuration.file, &configuration.name) {
            let potential = resolved_ef_configuration_entity(
                reference,
                &crate::ir::csharp_symbols::CSharpSourceSymbolIndex::default(),
            );
            if potential.is_some() {
                contracts.insert((configuration.path.clone(), configuration.arity));
                if symbol_key(catalog, configuration).is_none_or(|key| {
                    !matches!(
                        source.get(&key),
                        Some(crate::ir::csharp_symbols::CSharpSymbolComposition::Resolved(_))
                    )
                }) {
                    complete = false;
                }
                if let Some(requirement) = obligation(
                    catalog,
                    source,
                    configuration,
                    Kind::ConfigurationContract,
                    &reference.written,
                    scope,
                ) {
                    result.push(requirement);
                } else {
                    complete = false;
                }
            }
            if let Some(written) = resolved_ef_configuration_entity(reference, source) {
                if let Some(requirement) = obligation(
                    catalog,
                    source,
                    configuration,
                    Kind::ConfigurationEntity,
                    &written,
                    scope,
                ) {
                    result.push(requirement);
                } else {
                    complete = false;
                }
            }
        }
    }
    complete &= explicit.is_subset(&contracts);
    (result, projects, complete)
}

fn obligation(
    catalog: &Catalog,
    source: &crate::ir::csharp_symbols::CSharpSourceSymbolIndex,
    owner: &Declaration,
    kind: Kind,
    written: &str,
    scope: Option<&WorkspaceScope>,
) -> Option<Obligation> {
    let mut reference = catalog
        .declaration_scopes
        .get(&(owner.file.clone(), owner.path.clone(), owner.arity))?
        .clone();
    reference.written = written.to_string();
    // Canonical resolution owns arity/qualification; this syntax shape only
    // selects discovery hints and deliberately has separate work accounting.
    let source_text = format!("class __CleanCtxProbe : {written} {{}}");
    #[cfg(test)]
    work_metrics::record_ef_dependency_syntax_parse();
    let language = crate::compression::language::safe_csharp_language()?;
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).ok()?;
    let tree = parser.parse(&source_text, None)?;
    reference.shape = find_first_base_shape(tree.root_node(), source_text.as_bytes())?;
    let external = [configuration_contract()];
    let disposition = resolve_reference(
        &reference,
        source,
        if matches!(kind, Kind::ConfigurationContract) {
            &external
        } else {
            &[]
        },
    );
    Some(Obligation {
        kind,
        context: declaration_key(owner),
        scope: scope.cloned(),
        reference,
        disposition,
    })
}

fn configuration_contract() -> CSharpTypeIdentity {
    CSharpTypeIdentity::known_external_with_arity(
        "Microsoft.EntityFrameworkCore.IEntityTypeConfiguration",
        1,
    )
}

impl Obligation {
    fn resolved(&self) -> bool {
        match &self.disposition {
            CSharpResolution::Resolved { identity, .. } => {
                !matches!(self.kind, Kind::ConfigurationContract)
                    || *identity == configuration_contract()
            }
            _ => false,
        }
    }
}

fn select(inventory: &InventoryManifest, obligation: &Obligation, selected: &mut BTreeSet<String>) {
    // The structured requirement retains kind/context/scope/disposition; terms
    // below are only a conservative frontier, never identity or proof.
    let mut terms = BTreeSet::from([obligation.reference.shape.name.clone()]);
    terms.insert(obligation.context.path.last().cloned().unwrap_or_default());
    for alias in &obligation.reference.aliases {
        // Namespace aliases already retain the referenced terminal type. Only
        // a type-alias spelling needs its target name added as another hint.
        if alias.alias != obligation.reference.shape.name {
            continue;
        }
        #[cfg(test)]
        work_metrics::record_ef_dependency_syntax_parse();
        if let Some(name) =
            crate::mcp::tool_handlers::query::csharp_resolution::terminal_lookup_name(&alias.target)
        {
            terms.insert(name);
        }
    }
    selected.extend(
        inventory
            .entries
            .iter()
            .filter(|entry| {
                entry.source_kind == SourceKind::CSharp
                    && obligation
                        .scope
                        .as_ref()
                        .is_none_or(|scope| scope.admits(&entry.owner))
                    && terms
                        .iter()
                        .any(|term| !term.is_empty() && entry.source.contains(term))
            })
            .map(|entry| entry.owner.clone()),
    );
}
