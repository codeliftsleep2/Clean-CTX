use crate::ir::compiler::CompiledIR;
use crate::ir::invocation_evidence::InvocationEvidenceSet;
use crate::layers::meta::semantic::{SemanticDependency, SemanticEdge};
use crate::mcp::compatibility::identity::ProducerKey;
use std::collections::BTreeMap;
use std::sync::Arc;

#[cfg(feature = "csharp")]
#[derive(Debug)]
pub(crate) struct CSharpSemanticArtifact {
    declarations: Vec<crate::ir::csharp_exact_calls::Declaration>,
    resolution: crate::ir::csharp_resolution::CSharpCompilationUnit,
}

#[derive(Debug)]
pub(crate) struct ProducerGenerationFingerprint {
    canonical: BTreeMap<ProducerKey, u32>,
    semantic: BTreeMap<ProducerKey, u32>,
}

#[derive(Debug)]
pub(crate) struct ArtifactIdentity {
    canonical_owner: String,
    source_hash: String,
    fidelity: crate::compression::Fidelity,
    producer_generations: ProducerGenerationFingerprint,
}

#[derive(Debug)]
pub(crate) struct SemanticCompilationArtifact {
    identity: ArtifactIdentity,
    source_snapshot: Arc<String>,
    compiled: CompiledIR,
    semantic_edges: Vec<SemanticEdge>,
    semantic_dependencies: Vec<SemanticDependency>,
    invocation_evidence: Option<InvocationEvidenceSet>,
    #[cfg(feature = "csharp")]
    csharp: Option<CSharpSemanticArtifact>,
}

impl SemanticCompilationArtifact {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_compiler(
        compiled: CompiledIR,
        compiler: &mut crate::ir::compiler::IRCompiler,
        source_hash: String,
        canonical_owner: String,
        source: &str,
        fidelity: crate::compression::Fidelity,
        path: &std::path::Path,
        config: &crate::config::CleanCtxConfig,
        extension: &str,
    ) -> Result<Self, crate::error::CleanCtxError> {
        let producer_generations = ProducerGenerationFingerprint::current(source, path, config)
            .map_err(|error| crate::error::CleanCtxError::Ir(error.to_string()))?;
        let semantic_edges = std::mem::take(&mut compiler.semantic_edges);
        let semantic_dependencies = std::mem::take(&mut compiler.semantic_dependencies);
        let invocation_evidence = compiler.invocation_evidence.take();
        #[cfg(feature = "csharp")]
        let csharp = (extension == "cs").then(|| CSharpSemanticArtifact {
            declarations: crate::ir::csharp_exact_calls::declaration_authority(
                &compiled.instructions,
            ),
            resolution: compiler.csharp_compilation_unit.take().unwrap_or_default(),
        });
        #[cfg(not(feature = "csharp"))]
        let _ = extension;

        Ok(Self {
            identity: ArtifactIdentity {
                canonical_owner,
                source_hash,
                fidelity,
                producer_generations,
            },
            source_snapshot: Arc::new(source.to_string()),
            compiled,
            semantic_edges,
            semantic_dependencies,
            invocation_evidence,
            #[cfg(feature = "csharp")]
            csharp,
        })
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        CompiledIR,
        Vec<SemanticEdge>,
        Vec<SemanticDependency>,
        String,
    ) {
        let Self {
            identity,
            source_snapshot,
            compiled,
            semantic_edges,
            semantic_dependencies,
            invocation_evidence,
            #[cfg(feature = "csharp")]
            csharp,
        } = self;
        let ArtifactIdentity {
            canonical_owner,
            source_hash,
            fidelity,
            producer_generations,
        } = identity;
        let ProducerGenerationFingerprint {
            canonical,
            semantic,
        } = producer_generations;
        drop((
            canonical_owner,
            fidelity,
            canonical,
            semantic,
            source_snapshot,
        ));
        drop(invocation_evidence);
        #[cfg(feature = "csharp")]
        if let Some(CSharpSemanticArtifact {
            declarations,
            resolution,
        }) = csharp
        {
            drop((declarations, resolution));
        }
        (compiled, semantic_edges, semantic_dependencies, source_hash)
    }

    #[cfg(test)]
    pub(crate) fn compiled(&self) -> &CompiledIR {
        &self.compiled
    }

    #[cfg(test)]
    pub(crate) fn semantic_edges(&self) -> &[SemanticEdge] {
        &self.semantic_edges
    }

    #[cfg(test)]
    pub(crate) fn semantic_dependencies(&self) -> &[SemanticDependency] {
        &self.semantic_dependencies
    }

    pub(crate) fn source_hash(&self) -> &str {
        &self.identity.source_hash
    }

    pub(crate) fn canonical_owner(&self) -> &str {
        &self.identity.canonical_owner
    }

    pub(crate) fn fidelity(&self) -> crate::compression::Fidelity {
        self.identity.fidelity
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn authority_identity_material(&self) -> String {
        format!(
            "{}\0{}\0{:?}\0{:?}",
            self.identity.canonical_owner,
            self.identity.source_hash,
            self.identity.producer_generations.canonical,
            self.identity.producer_generations.semantic
        )
    }

    pub(crate) fn published_semantics(
        &self,
    ) -> (Vec<SemanticEdge>, Vec<SemanticDependency>, String) {
        (
            self.semantic_edges.clone(),
            self.semantic_dependencies.clone(),
            self.identity.source_hash.clone(),
        )
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn semantic_dependencies_owned(&self) -> Vec<SemanticDependency> {
        self.semantic_dependencies.clone()
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn csharp_declarations(
        &self,
    ) -> Option<&[crate::ir::csharp_exact_calls::Declaration]> {
        self.csharp
            .as_ref()
            .map(|artifact| artifact.declarations.as_slice())
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn csharp_resolution_unit(
        &self,
    ) -> Option<&crate::ir::csharp_resolution::CSharpCompilationUnit> {
        self.csharp.as_ref().map(|artifact| &artifact.resolution)
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn project_exact_csharp_calls(
        &self,
        authority: &[crate::ir::csharp_exact_calls::Declaration],
    ) -> Result<Vec<SemanticEdge>, String> {
        let Some(evidence) = self.invocation_evidence.as_ref() else {
            return Err(format!(
                "C# artifact lacks structured invocation evidence: {}",
                self.identity.canonical_owner
            ));
        };
        let evidence = evidence
            .view(
                &self.identity.canonical_owner,
                self.source_snapshot.as_str(),
            )
            .map_err(|error| format!("C# artifact evidence mismatch: {error:?}"))?;
        let mut edges: Vec<_> = self
            .semantic_edges
            .iter()
            .filter(|edge| edge.layer != "csharp-exact-call")
            .cloned()
            .collect();
        edges.extend(crate::ir::csharp_exact_calls::project_with_authority(
            self.source_snapshot.as_str(),
            &self.compiled.instructions,
            evidence.occurrences(),
            &self.identity.canonical_owner,
            authority,
        ));
        Ok(edges)
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn project_csharp_generation_semantics(
        &self,
        authority: &[crate::ir::csharp_exact_calls::Declaration],
        resolved_classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    ) -> Result<Vec<SemanticEdge>, String> {
        let mut edges = self.project_exact_csharp_calls(authority)?;
        edges.retain(|edge| {
            !(edge.subject.domain == "dotnet"
                && ((edge.subject.entity_type == "DbContext"
                    && matches!(
                        edge.relation,
                        crate::layers::meta::semantic::SemanticRelation::Defines
                            | crate::layers::meta::semantic::SemanticRelation::HasEntity
                    ))
                    || (edge.subject.entity_type == "MapperProfile"
                        && matches!(
                            edge.relation,
                            crate::layers::meta::semantic::SemanticRelation::MapsFrom
                                | crate::layers::meta::semantic::SemanticRelation::MapsTo
                        ))
                    || (edge.subject.entity_type == "Hub"
                        && edge.relation
                            == crate::layers::meta::semantic::SemanticRelation::HubMethodTargets)
                    || (edge.subject.entity_type == "Controller"
                        && matches!(
                            edge.relation,
                            crate::layers::meta::semantic::SemanticRelation::Defines
                                | crate::layers::meta::semantic::SemanticRelation::HasRoute
                                | crate::layers::meta::semantic::SemanticRelation::ControllerAction
                        ))))
        });
        edges.extend(
            crate::dotnet_meta::semantic::project_resolved_dbcontext_edges(
                resolved_classes,
                self.canonical_owner(),
                self.fidelity(),
            ),
        );
        edges.extend(
            crate::dotnet_meta::semantic::project_resolved_automapper_edges(
                resolved_classes,
                self.canonical_owner(),
            ),
        );
        edges.extend(
            crate::dotnet_meta::semantic::project_resolved_signalr_edges(
                resolved_classes,
                self.canonical_owner(),
                self.fidelity(),
            ),
        );
        edges.extend(crate::dotnet_meta::semantic::project_resolved_aspnet_edges(
            resolved_classes,
            self.canonical_owner(),
            self.fidelity(),
        ));
        edges.extend(
            crate::dotnet_meta::semantic::project_attribute_authorized_aspnet_edges(
                resolved_classes,
                self.canonical_owner(),
                self.fidelity(),
            ),
        );
        Ok(edges)
    }

    #[cfg(feature = "csharp")]
    pub(crate) fn project_csharp_generation_ir(
        &self,
        resolved_classes: &[crate::ir::csharp_resolution::CSharpResolvedClass],
    ) -> CompiledIR {
        let mut compiled = self.compiled.clone();
        compiled.instructions.retain(|instruction| {
            !matches!(
                instruction,
                crate::ir::opcodes::CoreOp::ExecutionContext(
                    _,
                    crate::ir::opcodes::ExecutionContextKind::Realtime
                )
            )
        });
        compiled.instructions.extend(
            crate::dotnet_meta::semantic::project_resolved_signalr_execution_contexts(
                resolved_classes,
                self.canonical_owner(),
            ),
        );
        compiled
    }

    #[cfg(test)]
    pub(crate) fn source_snapshot(&self) -> &str {
        self.source_snapshot.as_str()
    }

    #[cfg(test)]
    pub(crate) fn producer_generation_count(&self) -> usize {
        self.identity.producer_generations.canonical.len()
            + self.identity.producer_generations.semantic.len()
    }

    #[cfg(test)]
    pub(crate) fn invocation_evidence(&self) -> Option<&InvocationEvidenceSet> {
        self.invocation_evidence.as_ref()
    }
}

impl ProducerGenerationFingerprint {
    pub(crate) fn current(
        source: &str,
        path: &std::path::Path,
        config: &crate::config::CleanCtxConfig,
    ) -> Result<Self, crate::mcp::compatibility::IdentityDerivationError> {
        let identities = crate::mcp::compatibility::derive_identities(source, path, config)?;
        Ok(Self {
            canonical: identities.canonical_producers.relevant_producers,
            semantic: identities.semantic_producers.relevant_producers,
        })
    }
}
