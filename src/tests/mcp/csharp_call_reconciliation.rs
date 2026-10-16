use crate::compression::Fidelity;
use crate::mcp::McpState;
use crate::workspace::index::SemanticFidelity;

const TARGET_IDENTITY: &str = "Target::Save()->void";
const CALLER_IDENTITY: &str = "Caller::Run(Target)->void";

fn state() -> McpState {
    McpState::new(crate::tests::test_config())
}

fn publish(state: &McpState, file: &std::path::Path) {
    let file = file.to_string_lossy();
    state.invalidate_source_cache(&file);
    let (_, edges, _, hash) =
        crate::mcp::tool_helpers::compile_file_ir_candidate(&file, Fidelity::High, state)
            .expect("candidate compilation");
    let canonical = state.semantic_owner_path(&file);
    state.workspace_index_lock().replace_semantic_projection(
        &canonical,
        edges,
        SemanticFidelity::High,
        hash,
    );
    state
        .reconcile_csharp_after_authority_change(&canonical)
        .expect("C# reconciliation");
}

fn callers(state: &McpState) -> Vec<String> {
    state
        .workspace_index_read()
        .reverse_edges_by_identity("csharp", "Method", TARGET_IDENTITY)
        .into_iter()
        .map(|edge| edge.subject.name.clone())
        .collect()
}

fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().unwrap();
    let caller = root.path().join("Caller.cs");
    let target = root.path().join("Target.cs");
    std::fs::write(
        &caller,
        "class Caller { void Run(Target target) { target.Save(); } }",
    )
    .unwrap();
    std::fs::write(&target, "class Target { public void Save() {} }").unwrap();
    (root, caller, target)
}

#[test]
fn cross_file_exact_call_is_file_order_independent() {
    let (_root, caller, target) = fixture();
    let caller_first = state();
    publish(&caller_first, &caller);
    assert!(callers(&caller_first).is_empty());
    publish(&caller_first, &target);
    assert_eq!(callers(&caller_first), [CALLER_IDENTITY]);
    caller_first.reconcile_csharp_exact_calls().unwrap();
    assert_eq!(
        callers(&caller_first),
        [CALLER_IDENTITY],
        "a warm repeat must preserve the cold exact result"
    );

    let target_first = state();
    publish(&target_first, &target);
    publish(&target_first, &caller);
    assert_eq!(callers(&target_first), [CALLER_IDENTITY]);
}

#[test]
fn removal_and_signature_change_revoke_exact_calls() {
    let (_root, caller, target) = fixture();
    let state = state();
    publish(&state, &caller);
    publish(&state, &target);
    assert_eq!(callers(&state), [CALLER_IDENTITY]);

    let canonical_target = state.semantic_owner_path(&target.to_string_lossy());
    state.workspace_index_lock().remove_file(&canonical_target);
    state
        .reconcile_csharp_after_authority_change(&canonical_target)
        .unwrap();
    assert!(callers(&state).is_empty());

    std::fs::write(
        &target,
        "class Target { public void Save(string value) {} }",
    )
    .unwrap();
    publish(&state, &target);
    assert!(callers(&state).is_empty());
}

#[test]
fn newly_introduced_ambiguity_revokes_and_removal_reestablishes() {
    let (root, caller, target) = fixture();
    let duplicate = root.path().join("Duplicate.cs");
    std::fs::write(&duplicate, "class Target { public void Save() {} }").unwrap();
    let state = state();
    publish(&state, &caller);
    publish(&state, &target);
    assert_eq!(callers(&state), [CALLER_IDENTITY]);

    publish(&state, &duplicate);
    assert!(
        callers(&state).is_empty(),
        "two declaration authorities must revoke the formerly exact edge"
    );

    let canonical_duplicate = state.semantic_owner_path(&duplicate.to_string_lossy());
    state
        .workspace_index_lock()
        .remove_file(&canonical_duplicate);
    state
        .reconcile_csharp_after_authority_change(&canonical_duplicate)
        .unwrap();
    assert_eq!(callers(&state), [CALLER_IDENTITY]);
}
