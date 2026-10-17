use std::collections::BTreeSet;

/// Source-language owner kind participating in one acquisition obligation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SourceKind {
    CSharp,
    TypeScript,
    Rust,
    Java,
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
