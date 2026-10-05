use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(db: &std::path::Path, additional_roots: &[&std::path::Path]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.persistence.enabled = true;
    config.persistence.db_path = db.to_string_lossy().into_owned();
    config.additional_roots = additional_roots
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    crate::mcp::McpState::new(config)
}

fn dispatch(state: &crate::mcp::McpState, id: i64, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(id), tool, &json!({ "arguments": arguments }), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}

fn persist(state: &crate::mcp::McpState, file: &str, workspace_root: &std::path::Path) {
    let response = dispatch(
        state,
        1,
        "compress_code_context",
        json!({
            "filePath": file,
            "workspaceRoot": workspace_root,
            "fidelity": "edit"
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
}

#[test]
fn restore_resolves_relative_path_against_workspace_root() {
    let _serial = crate::protocol::handler_response_serial();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    let source_dir = root.join("src");
    std::fs::create_dir_all(&source_dir).unwrap();
    let file = source_dir.join("A.ts");
    std::fs::write(&file, "export class A {}\n").unwrap();
    let db = temp.path().join("relative.db");

    let producer = state(&db, &[]);
    persist(&producer, &file.to_string_lossy(), &root);
    drop(producer);

    let restarted = state(&db, &[]);
    let response = dispatch(
        &restarted,
        2,
        "restore_context",
        json!({ "filePath": "src/A.ts", "workspaceRoot": root }),
    );
    assert!(response.get("error").is_none(), "{response}");
    let owner = file.to_string_lossy();
    let alias = restarted
        .alias_for_path(&owner)
        .expect("restored owner alias");
    assert!(restarted.ir_context_read().has_file(&alias));
    assert_eq!(
        restarted.persisted_path(&alias).as_deref(),
        Some(owner.as_ref())
    );
}

#[test]
fn durable_admission_does_not_require_the_source_file_to_exist() {
    let root = tempfile::tempdir().unwrap();
    let missing = root.path().join("deleted.ts");
    let resolved = super::resolve_durable_path_checked(
        &missing.to_string_lossy(),
        Some(&root.path().to_string_lossy()),
        &[],
    )
    .expect("an authorized durable owner remains admissible after source deletion");
    assert_eq!(resolved, missing.to_string_lossy());
}

#[test]
fn missing_durable_owner_cannot_escape_through_parent_components() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    std::fs::create_dir_all(root.join("nested")).unwrap();
    let escaped = root.join("nested").join("..").join("..").join("outside.ts");
    let rejected = super::resolve_durable_path_checked(
        &escaped.to_string_lossy(),
        Some(&root.to_string_lossy()),
        &[],
    );
    assert!(
        rejected
            .as_deref()
            .is_err_and(|error| error.contains("outside workspace root")),
        "{rejected:?}"
    );
}

#[cfg(unix)]
#[test]
fn dangling_symlink_is_not_treated_as_an_authorized_missing_leaf() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("repo");
    let outside = temp.path().join("outside").join("missing.ts");
    std::fs::create_dir_all(&root).unwrap();
    let link = root.join("linked.ts");
    symlink(&outside, &link).unwrap();

    let rejected = super::resolve_durable_path_checked(
        &link.to_string_lossy(),
        Some(&root.to_string_lossy()),
        &[],
    );
    assert!(
        rejected
            .as_deref()
            .is_err_and(|error| error.contains("cannot be resolved safely")),
        "{rejected:?}"
    );
}

#[test]
fn restore_rejects_removed_additional_root_before_recovery_or_publication() {
    let _serial = crate::protocol::handler_response_serial();
    let temp = tempfile::tempdir().unwrap();
    let root_a = temp.path().join("repo-a");
    let root_b = temp.path().join("repo-b");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    let file_path = root_b.join("B.ts");
    let file = file_path.to_string_lossy().into_owned();
    let prior = b"export class B { value = 1; }\n".to_vec();
    let target = b"export class B { value = 2; }\n".to_vec();
    std::fs::write(&file_path, &prior).unwrap();
    let db = temp.path().join("admission.db");

    let producer = state(&db, &[&root_b]);
    persist(&producer, &file, &root_a);
    let alias = producer.alias_for_path(&file).unwrap();
    let prior_hash = producer
        .ir_context_read()
        .get_source_hash(&alias)
        .cloned()
        .unwrap();
    let prior_version = producer.file_version(&alias).unwrap();
    let target_source = std::str::from_utf8(&target).unwrap();
    let (mut target_ir, target_edges, target_hash) =
        crate::mcp::tool_helpers::compile_source_ir_candidate(
            &file,
            target_source,
            crate::compression::Fidelity::Edit,
            &producer,
        )
        .unwrap();
    target_ir.file_id.clone_from(&file);
    let identities =
        crate::mcp::compatibility::derive_identities(target_source, &file_path, &producer.config)
            .unwrap();
    let intent = crate::mcp::sqlite_store::EditIntent {
        transition_id: "unauthorized-restore".to_string(),
        file_path: file.clone(),
        prior_hash,
        target_hash,
        prior_version,
        target_version: target_ir.version,
        prior_source: prior,
        target_source: target,
        target_ir: crate::ir::binary_wire::encode(&target_ir),
        target_edges,
        fidelity: crate::compression::Fidelity::Edit,
        stage_path: String::new(),
    };
    producer
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .establish_compatible_edit_intent(&intent, &identities)
        .unwrap();
    drop(producer);

    let restarted = state(&db, &[]);
    let rejected = dispatch(
        &restarted,
        2,
        "restore_context",
        json!({ "filePath": file, "workspaceRoot": root_a }),
    );
    assert!(
        rejected["error"]["message"]
            .as_str()
            .is_some_and(|message| message.contains("outside workspace root")),
        "{rejected}"
    );
    assert!(restarted.alias_for_path(&file).is_none());
    assert!(restarted.ir_context_read().get_ir(&file).is_none());
    assert!(restarted.semantic_edges(&file).is_none());
    assert!(restarted.persisted_path(&file).is_none());
    assert_eq!(restarted.workspace_index_read().edge_count(), 0);
    assert!(restarted.llm_text_cache_lock().is_empty());
    assert!(
        restarted
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .has_edit_intent(&file)
            .unwrap(),
        "rejection must occur before pending-edit recovery mutates durable state"
    );
}
