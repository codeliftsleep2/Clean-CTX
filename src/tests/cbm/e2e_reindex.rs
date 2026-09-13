use super::*;

// -- E2E: apply_edit -> automatic reindex -> fresh graph ------------
//
// CBM-EDIT-001 invariant: after a successful apply_edit returns,
// subsequent CBM graph queries observe the filesystem state
// produced by that edit.
//
// apply_edit marks the owning project dirty. The first subsequent freshness
// gate performs the reindex synchronously before the graph query proceeds, so
// there is no timing dependency.

/// Prove automatic reindex after apply_edit by modifying a method body
/// and verifying the CBM graph is still queryable after reindex.
#[serial(cbm_live)]
#[test]
fn e2e_apply_edit_triggers_reindex_and_graph_is_fresh() {
    if !cbm_binary_exists() {
        eprintln!("Skipping - CBM not installed");
        return;
    }
    let state = shared_live_state();
    let fixture_root = multiroot::shared_fixture_root();
    let fixture_path = fixture_root.join("src/service.ts");
    let file_path_str = fixture_path.to_string_lossy().into_owned();
    let fixture_str = fixture_root.to_string_lossy().to_string();
    let fx_slug = {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.resolve_project_id(&fixture_str)
    };
    let _fixture_workspace = activate_shared_workspace(&state, &fixture_root);
    let indexed_before_edit = {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        !b.search("doSomething").is_empty()
    };
    assert!(
        indexed_before_edit,
        "doSomething must be indexed before edit"
    );
    crate::mcp::tool_handlers::core::handle_provide_code_context(
        &serde_json::json!(1),
        &serde_json::json!({"arguments": {"filePath": file_path_str.clone(), "fidelity": "edit"}}),
        &state,
    );
    crate::mcp::tool_handlers::edit::handle_apply_edit(
        &serde_json::json!(2),
        &serde_json::json!({"arguments": {"filePath": file_path_str.clone(), "operations": [{
            "type": "replace_body",
            "target": "MyService.doSomething",
            "expectedOldText": "{\n    return 42;\n  }",
            "newText": "{\n    return 99;\n  }"
        }]}}),
        &state,
    );
    let on_disk = std::fs::read_to_string(&fixture_path).expect("fixture must exist after edit");
    assert!(
        on_disk.contains("return 99;"),
        "edit must have written new body to disk:\n{on_disk}"
    );

    // ── Verify lazy freshness contract ──────────────────────────────
    // After apply_edit, the project must be dirty (no synchronous reindex).
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert!(
            entry.dirty_generation > entry.indexed_generation,
            "project must be dirty after apply_edit (dirty={}, indexed={})",
            entry.dirty_generation,
            entry.indexed_generation,
        );
    }

    // First graph search: explicitly ensure freshness (as the production
    // MCP handler does via ensure_indexed_or_error), then search.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let idx_status = b.ensure_indexed();
        match idx_status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            _ => panic!("ensure_indexed must return Ready after lazy reindex: {idx_status:?}"),
        }
        let result = b.search("doSomething");
        assert!(
            !result.is_empty(),
            "doSomething must be in graph after lazy reindex"
        );
    }

    // After the search, the project must be clean (indexed = dirty).
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert_eq!(
            entry.dirty_generation, entry.indexed_generation,
            "project must be clean after lazy reindex (dirty={}, indexed={})",
            entry.dirty_generation, entry.indexed_generation,
        );
    }

    // Second graph search: explicitly ensure freshness, then search.
    // The project is already clean (ensure_indexed in the first block
    // advanced indexed_generation), so ensure_indexed is a fast no-op.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        b.set_project(&fx_slug);
        b.invalidate_cache();
        let idx_status = b.ensure_indexed();
        match idx_status {
            Ok(crate::cbm::bridge::IndexingStatus::Ready) => {}
            _ => panic!("ensure_indexed must return Ready on clean project: {idx_status:?}"),
        }
        let result = b.search("doSomething");
        assert!(
            !result.is_empty(),
            "doSomething must still be in graph on second search"
        );
    }

    // After the second search, the project must still be clean.
    {
        let mut guard = state.graph_bridge_lock();
        let b = guard.as_mut().expect("live bridge");
        let f_map = b.freshness.lock().unwrap_or_else(|p| p.into_inner());
        let entry = f_map
            .get(&fx_slug)
            .expect("project must have freshness entry");
        assert_eq!(
            entry.dirty_generation, entry.indexed_generation,
            "project must remain clean after second search",
        );
    }
}
