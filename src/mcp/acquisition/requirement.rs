use crate::mcp::compatibility::identity::ProducerKey;
use crate::mcp::compatibility::producer::GenerationCatalog;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

/// Source-language owner kind participating in one acquisition obligation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SourceKind {
    CSharp,
    TypeScript,
    Rust,
    Java,
}

impl SourceKind {
    pub(crate) fn from_path(path: &std::path::Path) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        let layer = crate::layers::LayerRegistry::global()
            .language_layer_for_extension(extension)
            .filter(|layer| layer.language_ptr().is_some())?;
        match layer.name() {
            "csharp" => Some(Self::CSharp),
            "typescript" => Some(Self::TypeScript),
            "rust" => Some(Self::Rust),
            "java" => Some(Self::Java),
            _ => None,
        }
    }
}

/// Semantic producer capability whose source universe must be covered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum ProducerCapability {
    BuiltinTypeRefIncoming,
}

/// Incoming relation families emitted by the initial TypeRef producer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum RelationshipFamily {
    HasConstructorParameterType,
    HasBaseType,
}

/// Stable target information available before final semantic refinement.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct AcquisitionTarget {
    pub(crate) domain: String,
    pub(crate) entity_type: String,
    pub(crate) written_name: String,
    pub(crate) matching_version: u32,
}

impl AcquisitionTarget {
    pub(crate) fn exact_type_ref(written_name: impl Into<String>) -> Self {
        Self {
            domain: "builtin".into(),
            entity_type: "TypeRef".into(),
            written_name: written_name.into(),
            matching_version: 1,
        }
    }
}

/// One provider-neutral semantic acquisition obligation.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct CoverageUnit {
    pub(crate) canonical_root: String,
    pub(crate) source_kind: SourceKind,
    pub(crate) producer: ProducerCapability,
    pub(crate) producer_generation: String,
    pub(crate) relationships: BTreeSet<RelationshipFamily>,
}

impl CoverageUnit {
    pub(crate) fn builtin_type_ref(
        canonical_root: impl Into<String>,
        source_kind: SourceKind,
        producer_generation: impl Into<String>,
    ) -> Self {
        Self {
            canonical_root: canonical_root.into(),
            source_kind,
            producer: ProducerCapability::BuiltinTypeRefIncoming,
            producer_generation: producer_generation.into(),
            relationships: BTreeSet::from([
                RelationshipFamily::HasConstructorParameterType,
                RelationshipFamily::HasBaseType,
            ]),
        }
    }
}

/// Per-owner semantic evidence needed after source acquisition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum EvidenceRequirement {
    CurrentHighSemanticProjection,
}

/// Complete acquisition obligation for one incoming-relationship query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AcquisitionRequirement {
    pub(crate) target: AcquisitionTarget,
    pub(crate) scope_fingerprint: String,
    pub(crate) units: BTreeSet<CoverageUnit>,
    pub(crate) evidence: EvidenceRequirement,
}

impl AcquisitionRequirement {
    pub(crate) fn incoming_type_ref(
        target: impl Into<String>,
        scope_fingerprint: impl Into<String>,
        units: impl IntoIterator<Item = CoverageUnit>,
    ) -> Self {
        Self {
            target: AcquisitionTarget::exact_type_ref(target),
            scope_fingerprint: scope_fingerprint.into(),
            units: units.into_iter().collect(),
            evidence: EvidenceRequirement::CurrentHighSemanticProjection,
        }
    }
}

pub(crate) fn requirement_fingerprint(requirement: &AcquisitionRequirement) -> String {
    let mut hasher = Sha256::new();
    hash_part(&mut hasher, b"incoming-type-ref-v1");
    hash_part(&mut hasher, requirement.target.domain.as_bytes());
    hash_part(&mut hasher, requirement.target.entity_type.as_bytes());
    hash_part(&mut hasher, requirement.target.written_name.as_bytes());
    hasher.update(requirement.target.matching_version.to_le_bytes());
    hash_part(&mut hasher, requirement.scope_fingerprint.as_bytes());
    hasher.update([evidence_tag(requirement.evidence)]);
    for unit in &requirement.units {
        hash_part(&mut hasher, unit.canonical_root.as_bytes());
        hasher.update([source_kind_tag(unit.source_kind)]);
        hasher.update([producer_tag(unit.producer)]);
        hash_part(&mut hasher, unit.producer_generation.as_bytes());
        for relationship in &unit.relationships {
            hasher.update([relationship_tag(*relationship)]);
        }
        hasher.update([0xff]);
    }
    format!("{:x}", hasher.finalize())
}

fn hash_part(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_le_bytes());
    hasher.update(value);
}

fn source_kind_tag(value: SourceKind) -> u8 {
    match value {
        SourceKind::CSharp => 1,
        SourceKind::TypeScript => 2,
        SourceKind::Rust => 3,
        SourceKind::Java => 4,
    }
}

fn producer_tag(value: ProducerCapability) -> u8 {
    match value {
        ProducerCapability::BuiltinTypeRefIncoming => 1,
    }
}

fn relationship_tag(value: RelationshipFamily) -> u8 {
    match value {
        RelationshipFamily::HasConstructorParameterType => 1,
        RelationshipFamily::HasBaseType => 2,
    }
}

fn evidence_tag(value: EvidenceRequirement) -> u8 {
    match value {
        EvidenceRequirement::CurrentHighSemanticProjection => 1,
    }
}

pub(crate) fn producer_generation_fingerprint(
    source_kind: SourceKind,
    catalog: &GenerationCatalog,
) -> Option<String> {
    let language_keys = match source_kind {
        SourceKind::CSharp => [
            ProducerKey::CSharpCanonical,
            ProducerKey::CSharpSemanticInput,
        ],
        SourceKind::TypeScript => [
            ProducerKey::TypeScriptCanonical,
            ProducerKey::TypeScriptSemanticInput,
        ],
        SourceKind::Rust => [ProducerKey::RustCanonical, ProducerKey::RustSemanticInput],
        SourceKind::Java => [ProducerKey::JavaCanonical, ProducerKey::JavaSemanticInput],
    };
    let mut generations = vec![
        catalog.generation(ProducerKey::SharedCanonicalPipeline)?,
        catalog.generation(ProducerKey::TypeAliasTransform)?,
        catalog.generation(ProducerKey::GenericSemanticProjection)?,
        catalog.generation(language_keys[0])?,
        catalog.generation(language_keys[1])?,
    ];
    if source_kind == SourceKind::CSharp {
        generations.push(catalog.generation(ProducerKey::CSharpSemanticProjection)?);
    }

    let mut hasher = Sha256::new();
    hash_part(&mut hasher, b"builtin-type-ref-producer-v1");
    hasher.update([source_kind_tag(source_kind)]);
    for generation in generations {
        hasher.update(generation.to_le_bytes());
    }
    Some(format!("{:x}", hasher.finalize()))
}
