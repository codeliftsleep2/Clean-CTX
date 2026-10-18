use crate::ir::compiler::CompiledIR;
use crate::ir::invocation_evidence::InvocationEvidenceSet;
use crate::layers::meta::semantic::{SemanticDependency, SemanticEdge};
use crate::mcp::compatibility::identity::ProducerKey;
use std::collections::BTreeMap;
use std::sync::Arc;

pub(crate) type CompiledSemantic = (
    CompiledIR,
    Vec<SemanticEdge>,
    Vec<SemanticDependency>,
    String,
);

#[cfg(feature = "csharp")]
#[derive(Debug)]
pub(crate) struct CSharpSemanticArtifact {
    declarations: Vec<crate::ir::csharp_exact_calls::Declaration>,
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

    pub(crate) fn into_compiled_semantic(self) -> CompiledSemantic {
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
        drop((canonical_owner, fidelity, canonical, semantic, source_snapshot));
        drop(invocation_evidence);
        #[cfg(feature = "csharp")]
        if let Some(CSharpSemanticArtifact { declarations }) = csharp {
            drop(declarations);
        }
        (
            compiled,
            semantic_edges,
            semantic_dependencies,
            source_hash,
        )
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

    #[cfg(test)]
    pub(crate) fn canonical_owner(&self) -> &str {
        &self.identity.canonical_owner
    }

    #[cfg(test)]
    pub(crate) fn source_snapshot(&self) -> &str {
        self.source_snapshot.as_str()
    }

    #[cfg(test)]
    pub(crate) fn fidelity(&self) -> crate::compression::Fidelity {
        self.identity.fidelity
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

    #[cfg(all(test, feature = "csharp"))]
    pub(crate) fn csharp_declarations(
        &self,
    ) -> Option<&[crate::ir::csharp_exact_calls::Declaration]> {
        self.csharp
            .as_ref()
            .map(|artifact| artifact.declarations.as_slice())
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
