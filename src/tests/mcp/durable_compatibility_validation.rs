use super::{
    CompatibilityComponent, CompatibilityFailure, UntrustedDurableContext, validate_canonical,
    validate_current_context, validate_semantic,
};
use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::compiler::CompiledIR;
use crate::mcp::compatibility::identity::{
    CompatibilityIdentities, PersistedCompatibilityIdentities, ProducerKey,
};
use crate::mcp::sqlite_store::SqliteStore;
use sha2::{Digest, Sha256};
use std::path::Path;

const FILE: &str = "/workspace/example.ts";
const SOURCE: &str = "export class Example {}";

fn current_identities() -> CompatibilityIdentities {
    crate::mcp::compatibility::derive_identities(
        SOURCE,
        Path::new(FILE),
        &CleanCtxConfig::default(),
    )
    .expect("derive identities")
}

fn persisted(identities: &CompatibilityIdentities) -> PersistedCompatibilityIdentities {
    PersistedCompatibilityIdentities {
        canonical_config: Some(identities.canonical_config.clone()),
        canonical_producers: Some(identities.canonical_producers.clone()),
        semantic_config: Some(identities.semantic_config.clone()),
        semantic_producers: Some(identities.semantic_producers.clone()),
    }
}

fn loaded(identities: PersistedCompatibilityIdentities) -> UntrustedDurableContext {
    UntrustedDurableContext {
        ir: CompiledIR {
            file_id: FILE.to_string(),
            version: 1,
            instructions: Vec::new(),
        },
        semantic_edges: Vec::new(),
        source_hash: format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        fidelity: Fidelity::High,
        compatibility: identities,
    }
}

fn persisted_store() -> SqliteStore {
    let mut store = SqliteStore::open(Path::new(":memory:")).expect("open SQLite store");
    let ir = CompiledIR {
        file_id: FILE.to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    store
        .save_context_with_compatibility(
            FILE,
            Fidelity::High,
            "",
            &crate::ir::binary_wire::encode(&ir),
            &format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
            1,
            &[],
            0,
            0,
            &current_identities(),
        )
        .expect("persist compatible context");
    store
}

#[test]
fn compatible_context_yields_typed_canonical_and_semantic_evidence() {
    let compatible = validate_current_context(
        loaded(persisted(&current_identities())),
        SOURCE,
        Path::new(FILE),
        &CleanCtxConfig::default(),
        Fidelity::Medium,
    )
    .expect("compatible durable context");

    assert_eq!(compatible.canonical.ir.file_id, FILE);
    assert_eq!(compatible.canonical.fidelity, Fidelity::High);
    assert_eq!(compatible.semantic.fidelity, Fidelity::High);
    assert!(compatible.semantic.semantic_edges.is_empty());
    assert_eq!(
        compatible.canonical.source_hash,
        compatible.semantic.source_hash
    );
}

#[test]
fn source_and_fidelity_mismatches_are_distinct() {
    let identities = current_identities();
    let mut wrong_source = loaded(persisted(&identities));
    wrong_source.source_hash = "historical".to_string();
    assert!(matches!(
        validate_current_context(
            wrong_source,
            SOURCE,
            Path::new(FILE),
            &CleanCtxConfig::default(),
            Fidelity::Low,
        ),
        Err(CompatibilityFailure::SourceMismatch { .. })
    ));

    assert!(matches!(
        validate_current_context(
            loaded(persisted(&identities)),
            SOURCE,
            Path::new(FILE),
            &CleanCtxConfig::default(),
            Fidelity::Edit,
        ),
        Err(CompatibilityFailure::InsufficientFidelity {
            persisted: Fidelity::High,
            required: Fidelity::Edit,
        })
    ));
}

#[test]
fn configuration_mismatches_are_scoped_to_canonical_or_semantic_authority() {
    let current = current_identities();
    let mut canonical = persisted(&current);
    canonical
        .canonical_config
        .as_mut()
        .unwrap()
        .type_aliases
        .digest = "changed".to_string();
    assert_eq!(
        validate_canonical(&loaded(canonical), &current).unwrap_err(),
        CompatibilityFailure::CanonicalConfigurationIncompatible
    );

    let mut semantic = persisted(&current);
    semantic.semantic_config.as_mut().unwrap().schema_version += 1;
    assert_eq!(
        validate_semantic(&loaded(semantic), &current).unwrap_err(),
        CompatibilityFailure::SemanticConfigurationIncompatible
    );
}

#[test]
fn producer_set_comparison_rejects_addition_and_removal_bidirectionally() {
    let current = current_identities();
    let mut removed = persisted(&current);
    removed
        .canonical_producers
        .as_mut()
        .unwrap()
        .relevant_producers
        .remove(&ProducerKey::SharedCanonicalPipeline);
    assert_eq!(
        validate_canonical(&loaded(removed), &current).unwrap_err(),
        CompatibilityFailure::CanonicalProducerIncompatible
    );

    let mut added = persisted(&current);
    added
        .semantic_producers
        .as_mut()
        .unwrap()
        .relevant_producers
        .insert(ProducerKey::AngularSemantic, 1);
    assert_eq!(
        validate_semantic(&loaded(added), &current).unwrap_err(),
        CompatibilityFailure::SemanticProducerIncompatible
    );
}

#[test]
fn every_missing_legacy_component_has_a_distinct_diagnostic() {
    let current = current_identities();
    let mut missing = persisted(&current);
    missing.canonical_config = None;
    assert_eq!(
        validate_canonical(&loaded(missing), &current).unwrap_err(),
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::CanonicalConfiguration,
        }
    );

    let mut missing = persisted(&current);
    missing.canonical_producers = None;
    assert_eq!(
        validate_canonical(&loaded(missing), &current).unwrap_err(),
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::CanonicalProducers,
        }
    );

    let mut missing = persisted(&current);
    missing.semantic_config = None;
    assert_eq!(
        validate_semantic(&loaded(missing), &current).unwrap_err(),
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::SemanticConfiguration,
        }
    );

    let mut missing = persisted(&current);
    missing.semantic_producers = None;
    assert_eq!(
        validate_semantic(&loaded(missing), &current).unwrap_err(),
        CompatibilityFailure::MissingLegacyIdentity {
            component: CompatibilityComponent::SemanticProducers,
        }
    );
}

#[test]
fn centralized_failure_taxonomy_keeps_non_identity_rejections_distinct() {
    let failures = [
        CompatibilityFailure::PhysicalSchemaIncompatibility {
            detail: "schema".to_string(),
        },
        CompatibilityFailure::StructuralSnapshotIncoherence {
            detail: "snapshot".to_string(),
        },
        CompatibilityFailure::HistoricalSourceNotCurrent {
            historical: "h1".to_string(),
            current: "h2".to_string(),
        },
    ];
    assert_ne!(
        std::mem::discriminant(&failures[0]),
        std::mem::discriminant(&failures[1])
    );
    assert_ne!(
        std::mem::discriminant(&failures[1]),
        std::mem::discriminant(&failures[2])
    );
}

#[test]
fn durable_load_classifies_malformed_identity_as_physical_schema_failure() {
    let store = persisted_store();
    store
        .execute_batch("UPDATE contexts SET canonical_config_identity = '{malformed' ")
        .expect("corrupt identity encoding");
    assert!(matches!(
        store.load_durable_context(FILE, None),
        Err(CompatibilityFailure::PhysicalSchemaIncompatibility { .. })
    ));
}

#[test]
fn durable_load_classifies_cross_artifact_mismatch_as_structural_incoherence() {
    let store = persisted_store();
    store
        .execute_batch("UPDATE semantic_edge_snapshots SET source_hash = 'different'")
        .expect("corrupt snapshot coherence");
    assert!(matches!(
        store.load_durable_context(FILE, None),
        Err(CompatibilityFailure::StructuralSnapshotIncoherence { .. })
    ));
}

#[test]
fn validation_uses_immutable_input_and_leaves_untrusted_state_unchanged() {
    let current = current_identities();
    let loaded = loaded(persisted(&current));
    let original_file = loaded.ir.file_id.clone();
    let original_hash = loaded.source_hash.clone();
    let original_fidelity = loaded.fidelity;

    validate_canonical(&loaded, &current).expect("canonical validation");
    validate_semantic(&loaded, &current).expect("semantic validation");

    assert_eq!(loaded.ir.file_id, original_file);
    assert_eq!(loaded.source_hash, original_hash);
    assert_eq!(loaded.fidelity, original_fidelity);
    assert!(loaded.semantic_edges.is_empty());
}
