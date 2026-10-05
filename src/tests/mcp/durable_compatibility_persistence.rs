use crate::mcp::sqlite_store::SqliteStore;
use std::path::Path;

fn in_memory_store() -> SqliteStore {
    SqliteStore::open(Path::new(":memory:")).expect("open in-memory SQLite store")
}

fn identities(
    file: &str,
    source: &str,
) -> crate::mcp::compatibility::identity::CompatibilityIdentities {
    crate::mcp::compatibility::derive_identities(
        source,
        Path::new(file),
        &crate::config::CleanCtxConfig::default(),
    )
    .expect("derive identities")
}

fn encoded_identities(
    identities: &crate::mcp::compatibility::identity::CompatibilityIdentities,
) -> [Option<String>; 4] {
    [
        serde_json::to_string(&identities.canonical_config).ok(),
        serde_json::to_string(&identities.canonical_producers).ok(),
        serde_json::to_string(&identities.semantic_config).ok(),
        serde_json::to_string(&identities.semantic_producers).ok(),
    ]
}

#[test]
fn newly_persisted_context_round_trips_all_four_identities() {
    let mut store = in_memory_store();
    let file = "/workspace/example.ts";
    let source = "export class Example {}";
    let identities = identities(file, source);
    let ir = crate::ir::compiler::CompiledIR {
        file_id: file.to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    store
        .save_context_with_compatibility(
            file,
            crate::compression::Fidelity::High,
            "",
            &crate::ir::binary_wire::encode(&ir),
            "source-hash",
            1,
            &[],
            0,
            0,
            &identities,
        )
        .expect("persist compatible context");

    let stored = store
        .stored_compatibility_json(file)
        .expect("load stored identities");
    assert_eq!(stored, encoded_identities(&identities));
}

#[test]
fn legacy_context_remains_structurally_readable_with_missing_identity() {
    let mut store = in_memory_store();
    let file = "/workspace/legacy.ts";
    let ir = crate::ir::compiler::CompiledIR {
        file_id: file.to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    store
        .save_context_with_semantics(
            file,
            crate::compression::Fidelity::High,
            "",
            &crate::ir::binary_wire::encode(&ir),
            "legacy-hash",
            1,
            &[],
            0,
            0,
        )
        .expect("persist legacy-shaped context");

    let loaded = store
        .load_durable_context(file, None)
        .expect("structural load")
        .expect("durable context");
    assert_eq!(loaded.ir.file_id, file);
    assert_eq!(
        loaded.compatibility,
        crate::mcp::compatibility::identity::PersistedCompatibilityIdentities {
            canonical_config: None,
            canonical_producers: None,
            semantic_config: None,
            semantic_producers: None,
        }
    );
}

#[test]
fn failed_replacement_cannot_pair_old_artifacts_with_new_identities() {
    let mut store = in_memory_store();
    let file = "/workspace/atomic.ts";
    let old = identities(file, "export class Old {}");
    let new = identities(file, "export class New { method() {} }");
    let ir = crate::ir::compiler::CompiledIR {
        file_id: file.to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    let binary = crate::ir::binary_wire::encode(&ir);
    store
        .save_context_with_compatibility(
            file,
            crate::compression::Fidelity::High,
            "",
            &binary,
            "old-hash",
            1,
            &[],
            0,
            0,
            &old,
        )
        .expect("persist old epoch");
    crate::mcp::sqlite_store::fail_next_semantic_save(file);
    assert!(
        store
            .save_context_with_compatibility(
                file,
                crate::compression::Fidelity::High,
                "",
                &binary,
                "new-hash",
                1,
                &[],
                0,
                0,
                &new,
            )
            .is_err()
    );
    assert_eq!(
        store.stored_compatibility_json(file).unwrap(),
        encoded_identities(&old)
    );
}

#[test]
fn edit_intent_round_trips_target_producer_epoch() {
    let mut store = in_memory_store();
    let file = "/workspace/edit.ts";
    let target_source = b"export class Edited {}";
    let identities = identities(file, std::str::from_utf8(target_source).unwrap());
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: "transition-1".to_string(),
        file_path: file.to_string(),
        prior_hash: "prior".to_string(),
        target_hash: "target".to_string(),
        prior_version: 1,
        target_version: 2,
        prior_source: b"export class Prior {}".to_vec(),
        target_source: target_source.to_vec(),
        target_ir: Vec::new(),
        target_edges: Vec::new(),
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    store
        .establish_compatible_edit_intent(&intent, &identities)
        .expect("persist edit intent");
    assert_eq!(
        store
            .stored_edit_intent_compatibility_json(file)
            .expect("load edit-intent identities"),
        encoded_identities(&identities)
    );
}

#[test]
fn compatibility_migration_adds_nullable_identity_columns() {
    let store = in_memory_store();

    let contexts = store.column_names("contexts").expect("contexts schema");
    assert!(contexts.contains(&"canonical_config_identity".to_string()));
    assert!(contexts.contains(&"canonical_producer_identity".to_string()));

    let snapshots = store
        .column_names("semantic_edge_snapshots")
        .expect("semantic snapshot schema");
    assert!(snapshots.contains(&"semantic_config_identity".to_string()));
    assert!(snapshots.contains(&"semantic_producer_identity".to_string()));

    let intents = store
        .column_names("edit_intents")
        .expect("edit intent schema");
    for expected in [
        "canonical_config_identity",
        "canonical_producer_identity",
        "semantic_config_identity",
        "semantic_producer_identity",
    ] {
        assert!(
            intents.contains(&expected.to_string()),
            "missing edit-intent identity column {expected}"
        );
    }
}

#[test]
fn compatibility_migration_resumes_when_columns_exist_without_version_row() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let database = directory.path().join("compatibility.db");
    {
        let store = SqliteStore::open(&database).expect("create migrated database");
        store
            .execute_batch("DELETE FROM _schema_version WHERE version = 6")
            .expect("simulate interrupted version recording");
    }

    let reopened = SqliteStore::open(&database).expect("resume compatibility migration");
    assert!(
        reopened
            .column_names("contexts")
            .expect("contexts schema")
            .contains(&"canonical_config_identity".to_string())
    );
}

#[test]
fn accepted_delta_keeps_canonical_epoch_and_writes_snapshot_local_semantic_identity() {
    let mut store = in_memory_store();
    let file = "/workspace/delta-epoch.ts";
    let baseline_identities = identities(file, "export class Baseline {}");
    let baseline = crate::ir::compiler::CompiledIR {
        file_id: file.to_string(),
        version: 1,
        instructions: Vec::new(),
    };
    let context_id = store
        .save_context_with_compatibility(
            file,
            crate::compression::Fidelity::High,
            "",
            &crate::ir::binary_wire::encode(&baseline),
            "hash-1",
            1,
            &[],
            0,
            0,
            &baseline_identities,
        )
        .expect("persist baseline epoch");

    let delta = crate::ir::delta::IRDelta {
        file: file.to_string(),
        from: 1,
        to: 2,
        ops: crate::ir::delta::DeltaOps {
            adds: Vec::new(),
            mods: Vec::new(),
            dels: Vec::new(),
        },
        intent: None,
    };
    let payload = crate::mcp::persistence_ir::PersistedDelta::normalize_legacy(&delta, file)
        .expect("persisted delta");
    let mut snapshot_identities = baseline_identities.clone();
    snapshot_identities
        .semantic_producers
        .relevant_producers
        .insert(
            crate::mcp::compatibility::identity::ProducerKey::BuiltinSemantic,
            99,
        );
    let snapshot = crate::mcp::state::durable_semantics::DurableSemanticSnapshot::new(
        file.to_string(),
        "hash-2".to_string(),
        2,
        &[],
    );
    store
        .append_delta_with_semantics(
            &context_id,
            &payload,
            "legacy",
            "",
            &snapshot,
            &snapshot_identities,
        )
        .expect("append compatible epoch delta");

    let loaded = store
        .load_durable_context(file, None)
        .expect("load durable chain")
        .expect("durable context");
    assert_eq!(loaded.ir.version, 2);
    assert_eq!(
        loaded.compatibility.canonical_config,
        Some(baseline_identities.canonical_config)
    );
    assert_eq!(
        loaded.compatibility.canonical_producers,
        Some(baseline_identities.canonical_producers)
    );
    assert_eq!(
        loaded.compatibility.semantic_config,
        Some(snapshot_identities.semantic_config)
    );
    assert_eq!(
        loaded.compatibility.semantic_producers,
        Some(snapshot_identities.semantic_producers)
    );
}
