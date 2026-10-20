//! Shared retained/rendered/durable C# dependency boundary.
use super::artifact::SemanticCompilationArtifact;
use crate::ir::compiler::CompiledIR;
use crate::ir::csharp_resolution::{
    ancestry_obligations, ancestry_obligations_for_files, resolve_compilation_units,
};
use crate::layers::meta::semantic::{SemanticDependency, SemanticEdge};
use crate::mcp::McpState;
use crate::mcp::csharp_calls::{authority::merge_declarations, csharp_resolution_partition};
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Copy)]
pub(crate) struct PublicationStamp(u64);

impl PublicationStamp {
    pub(crate) fn for_generation(generation: u64) -> Self {
        Self(generation)
    }
    pub(crate) fn capture(state: &McpState) -> Self {
        Self(state.csharp_authority_generation())
    }
    pub(crate) fn is_current(self, index: &crate::workspace::index::WorkspaceIndex) -> bool {
        index.has_csharp_publication_generation(self.0)
    }
}

impl McpState {
    pub(crate) fn prepare_csharp_semantic_owner(
        &self,
        owner: &str,
        root: Option<&str>,
    ) -> Result<crate::mcp::tool_handlers::hydration::DependencyEvidence, String> {
        crate::mcp::tool_handlers::hydration::acquire_csharp_dependencies(self, owner, root)
    }
}

pub(crate) fn footprint(
    state: &McpState,
    owner: &str,
    artifacts: &[Arc<SemanticCompilationArtifact>],
) -> Vec<SemanticDependency> {
    let partition = csharp_resolution_partition(state, owner);
    let peers: Vec<_> = artifacts
        .iter()
        .filter(|artifact| {
            csharp_resolution_partition(state, artifact.canonical_owner()) == partition
        })
        .collect();
    let units: Vec<_> = peers
        .iter()
        .filter_map(|artifact| artifact.csharp_resolution_unit().cloned())
        .collect();
    let external = crate::dotnet_meta::semantic::known_csharp_type_identities();
    let obligations = ancestry_obligations(&units, owner, &external);
    peers
        .into_iter()
        .filter(|artifact| {
            artifact.canonical_owner() != owner
                && (obligations.owners.contains(artifact.canonical_owner())
                    || obligations
                        .discovery_terms
                        .iter()
                        .any(|term| artifact.source_snapshot().contains(term)))
        })
        .map(|artifact| SemanticDependency {
            file_path: artifact.canonical_owner().to_string(),
            source_hash: Some(artifact.source_hash().to_string()),
        })
        .collect()
}

pub(crate) fn footprints(
    state: &McpState,
    artifacts: &[Arc<SemanticCompilationArtifact>],
) -> BTreeMap<String, Vec<SemanticDependency>> {
    let mut partitions = BTreeMap::<String, Vec<_>>::new();
    for artifact in artifacts {
        partitions
            .entry(csharp_resolution_partition(
                state,
                artifact.canonical_owner(),
            ))
            .or_default()
            .push(artifact);
    }
    let mut result = BTreeMap::new();
    let external = crate::dotnet_meta::semantic::known_csharp_type_identities();
    for peers in partitions.into_values() {
        let units: Vec<_> = peers
            .iter()
            .filter_map(|peer| peer.csharp_resolution_unit().cloned())
            .collect();
        let files: Vec<_> = peers.iter().map(|peer| peer.canonical_owner()).collect();
        for (owner, obligations) in ancestry_obligations_for_files(&units, &files, &external) {
            let dependencies = peers
                .iter()
                .filter(|peer| {
                    peer.canonical_owner() != owner
                        && (obligations.owners.contains(peer.canonical_owner())
                            || obligations
                                .discovery_terms
                                .iter()
                                .any(|term| peer.source_snapshot().contains(term)))
                })
                .map(|peer| SemanticDependency {
                    file_path: peer.canonical_owner().to_string(),
                    source_hash: Some(peer.source_hash().to_string()),
                })
                .collect();
            result.insert(owner, dependencies);
        }
    }
    result
}

/// Rendering fidelity remains owned by the supplied candidate. Dependency
/// authority comes from the already acquired canonical retained generation.
pub(crate) fn current_parts(
    artifact: SemanticCompilationArtifact,
    state: &McpState,
) -> Result<
    (
        CompiledIR,
        Vec<SemanticEdge>,
        Vec<SemanticDependency>,
        String,
    ),
    String,
> {
    if artifact.csharp_resolution_unit().is_none() {
        return Ok(artifact.into_parts());
    }
    let stamp = PublicationStamp::capture(state);
    let owners: Vec<_> = state
        .workspace_index_read()
        .file_map()
        .keys()
        .cloned()
        .collect();
    let artifacts: Vec<_> = owners
        .iter()
        .filter_map(|owner| state.csharp_call_authority.artifact(owner))
        .collect();
    let partition = csharp_resolution_partition(state, artifact.canonical_owner());
    let mut units: Vec<_> = artifacts
        .iter()
        .filter(|peer| {
            peer.canonical_owner() != artifact.canonical_owner()
                && csharp_resolution_partition(state, peer.canonical_owner()) == partition
        })
        .filter_map(|peer| peer.csharp_resolution_unit().cloned())
        .collect();
    units.extend(artifact.csharp_resolution_unit().cloned());
    let classes = resolve_compilation_units(
        &units,
        &crate::dotnet_meta::semantic::known_csharp_type_identities(),
    );
    let edges =
        artifact.project_csharp_generation_semantics(&merge_declarations(&artifacts), &classes)?;
    let mut dependencies = artifact.semantic_dependencies_owned();
    dependencies.extend(footprint(state, artifact.canonical_owner(), &artifacts));
    if !stamp.is_current(&state.workspace_index_read())
        || !dependencies.iter().all(SemanticDependency::is_current)
    {
        return Err("C# dependency publication changed during semantic projection; retry".into());
    }
    Ok((
        artifact.project_csharp_generation_ir(&classes),
        edges,
        dependencies,
        artifact.source_hash().to_string(),
    ))
}
