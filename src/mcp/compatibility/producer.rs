//! Scoped producer-generation catalog. Constants remain owned by their
//! producers; this catalog only assembles the active runtime view.

use super::identity::ProducerKey;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct GenerationCatalog {
    generations: BTreeMap<ProducerKey, u32>,
}

impl GenerationCatalog {
    pub(crate) fn current() -> Self {
        let mut generations = BTreeMap::new();
        generations.insert(
            ProducerKey::SharedCanonicalPipeline,
            crate::ir::pipeline::CANONICAL_PIPELINE_GENERATION,
        );
        generations.insert(
            ProducerKey::TypeAliasTransform,
            crate::ir::type_aliases::TYPE_ALIAS_TRANSFORM_GENERATION,
        );
        generations.insert(
            ProducerKey::BuiltinSemantic,
            crate::layers::meta::builtin::SEMANTIC_PRODUCER_GENERATION,
        );
        generations.insert(
            ProducerKey::GenericSemanticProjection,
            crate::ir::semantic_projection::GENERIC_SEMANTIC_PROJECTION_GENERATION,
        );

        #[cfg(feature = "typescript")]
        {
            generations.insert(
                ProducerKey::TypeScriptCanonical,
                crate::ir::layers::typescript::CANONICAL_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::TypeScriptSemanticInput,
                crate::ir::layers::typescript::SEMANTIC_INPUT_GENERATION,
            );
        }
        #[cfg(feature = "csharp")]
        {
            generations.insert(
                ProducerKey::CSharpCanonical,
                crate::ir::layers::csharp::CANONICAL_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::CSharpSemanticInput,
                crate::ir::layers::csharp::SEMANTIC_INPUT_GENERATION,
            );
        }
        #[cfg(feature = "rust")]
        {
            generations.insert(
                ProducerKey::RustCanonical,
                crate::ir::layers::rust::CANONICAL_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::RustSemanticInput,
                crate::ir::layers::rust::SEMANTIC_INPUT_GENERATION,
            );
        }
        #[cfg(feature = "java")]
        {
            generations.insert(
                ProducerKey::JavaCanonical,
                crate::ir::layers::java::CANONICAL_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::JavaSemanticInput,
                crate::ir::layers::java::SEMANTIC_INPUT_GENERATION,
            );
        }
        #[cfg(feature = "angular")]
        {
            generations.insert(
                ProducerKey::AngularMarkers,
                crate::layers::meta::ANGULAR_MARKER_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::AngularSemantic,
                crate::layers::meta::ANGULAR_SEMANTIC_PRODUCER_GENERATION,
            );
        }
        #[cfg(feature = "dotnet")]
        {
            generations.insert(
                ProducerKey::DotNetMarkers,
                crate::dotnet_meta::MARKER_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::DotNetSemantic,
                crate::dotnet_meta::SEMANTIC_PRODUCER_GENERATION,
            );
        }
        #[cfg(feature = "spring_boot")]
        {
            generations.insert(
                ProducerKey::SpringBootMarkers,
                crate::spring_meta::MARKER_PRODUCER_GENERATION,
            );
            generations.insert(
                ProducerKey::SpringBootSemantic,
                crate::spring_meta::SEMANTIC_PRODUCER_GENERATION,
            );
        }
        Self { generations }
    }

    pub(crate) fn generation(&self, key: ProducerKey) -> Option<u32> {
        self.generations.get(&key).copied()
    }

    #[cfg(test)]
    pub(crate) fn with_generation(mut self, key: ProducerKey, generation: u32) -> Self {
        self.generations.insert(key, generation);
        self
    }

    #[cfg(test)]
    pub(crate) fn without(mut self, key: ProducerKey) -> Self {
        self.generations.remove(&key);
        self
    }
}
