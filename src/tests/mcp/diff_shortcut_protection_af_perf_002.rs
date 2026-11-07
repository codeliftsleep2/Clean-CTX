//! Agent MaxHeadRoom: finite clone-site protection and original public/cache identity.
use super::*;

fn protected(source: &str) -> bool {
    let Some(start) = source.find("    let source_hash = cache.compute_hash(source.as_bytes());")
    else {
        return false;
    };
    let Some(end) = source[start..].find("    let current = build_snapshot(source, fidelity)?;")
    else {
        return false;
    };
    let compact = |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect::<String>();
    let expected = r#"
    let source_hash = cache.compute_hash(source.as_bytes());
    if let Some(stored_hash) = cache.get_baseline_hash(&cache_key)
        && stored_hash == source_hash
        && let Some(baseline_snap) = cache.get_baseline(&cache_key)
    {
        // Content is byte-identical to the stored baseline — no
        // structural changes possible.
        let class_count = baseline_snap.classes.len();
        return Ok(format!(
            "// --- AST Diff ---\n// No changes since last snapshot ({} classes).\n// Hash: {}",
            class_count,
            &source_hash[..12],
        ));
    }
    "#;
    let rest = &source[start + end..];
    compact(&source[start..start + end]) == compact(expected)
        && rest.contains("diff_snapshot_copy(1, _snapshot)")
        && rest
            .split("let body = match baseline")
            .next()
            .unwrap()
            .contains(".cloned()")
}

#[test]
fn afperf_diff_shortcut_static() {
    let source = include_str!("../../mcp/tool_helpers.rs");
    assert!(protected(source), "Diff shortcut static clone protection");
    let regression = source.replacen(
        "Some(baseline_snap) = cache.get_baseline(&cache_key)",
        "Some(baseline_snap) = cache.get_baseline(&cache_key).cloned()",
        1,
    );
    assert!(
        !protected(&regression),
        "real clone self-mutation must be rejected"
    );
    let delegated = source.replacen(
        "baseline_snap.classes.len()",
        "baseline_snap.clone().classes.len()",
        1,
    );
    assert!(
        !protected(&delegated),
        "local delegated clone must be rejected"
    );
}

fn identity(state: &McpState, key: &str, source: &str) -> Value {
    let expected = build_snapshot(source, Fidelity::High).unwrap();
    let cache = state.cache_read();
    let actual = cache.get_baseline(key).unwrap();
    assert_eq!(actual, &expected);
    let hash = crate::cache::content_hash(source.as_bytes());
    assert_eq!(cache.get_baseline_hash(key), Some(hash.as_str()));
    json!({"hash":hash,"snapshot":format!("{actual:?}")})
}
fn normalized(response: &Value, file: &str) -> Value {
    let mut result = response.clone();
    result["result"]["content"][0]["text"] = json!(body(response).replace(file, "<FILE>"));
    result
}

#[test]
fn afperf_diff_shortcut_frozen_public() {
    let _serial = crate::protocol::handler_response_serial();
    let frozen: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/diff_shortcut_responses.json")).unwrap();
    assert_eq!(frozen.len(), 11);
    for row in frozen {
        let axes: Vec<usize> = serde_json::from_value(row["axes"].clone()).unwrap();
        let (classes, methods, payload, repeats) = (axes[0], axes[1], axes[2], axes[3]);
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("subject.ts");
        let old = ts_source(classes, methods, payload, false);
        let new = format!(
            "{}// next phase\n",
            ts_source(classes, methods, payload, true)
        );
        std::fs::write(&path, &old).unwrap();
        let file = std::fs::canonicalize(&path)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        config.cache.enabled = true;
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = McpState::new(config);
        *state.graph_bridge_lock() = None;
        let key = format!("{file}::{}", Fidelity::High as u8);
        let cold = dispatch(&state, &file, &root);
        let cold_cache = identity(&state, &key, &old);
        std::fs::write(&path, &new).unwrap();
        state.invalidate_source_cache(&file);
        let changed = dispatch(&state, &file, &root);
        let changed_cache = identity(&state, &key, &new);
        let mut warm = Vec::new();
        for _ in 0..repeats {
            warm.push(normalized(&dispatch(&state, &file, &root), &file));
        }
        let actual = json!({"axes":axes,"cold":normalized(&cold,&file),"changed":normalized(&changed,&file),
            "warm":warm,"cold_cache":cold_cache,"changed_cache":changed_cache,"warm_cache":identity(&state,&key,&new)});
        assert_eq!(actual, row, "complete original public/cache identity");
    }
}
