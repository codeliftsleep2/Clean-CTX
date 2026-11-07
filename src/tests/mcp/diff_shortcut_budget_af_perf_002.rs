//! Agent MaxHeadRoom: frozen public semantics before the shortcut-only correction.
use super::*;

fn cache_identity(state: &McpState, key: &str, source: &str) -> Value {
    let expected = build_snapshot(source, Fidelity::High).unwrap();
    let cache = state.cache_read();
    let actual = cache.get_baseline(key).expect("stored baseline");
    assert_eq!(actual, &expected, "complete cached snapshot identity");
    let hash = crate::cache::content_hash(source.as_bytes());
    assert_eq!(cache.get_baseline_hash(key), Some(hash.as_str()));
    json!({"hash":hash,"snapshot":format!("{actual:?}")})
}

fn normalize_response(response: &Value, file: &str) -> Value {
    let mut normalized = response.clone();
    let text = normalized["result"]["content"][0]["text"].as_str().unwrap();
    normalized["result"]["content"][0]["text"] = json!(text.replace(file, "<FILE>"));
    normalized
}

fn run_cases() -> usize {
    let mut redundant = 0;
    for (classes, methods, payload, repeats) in [
        (0, 0, 0, 1),
        (1, 1, 0, 1),
        (1, 8, 0, 1),
        (1, 64, 0, 1),
        (1, 256, 0, 1),
        (4, 16, 0, 1),
        (16, 1, 0, 1),
        (1, 16, 64, 1),
        (1, 16, 4096, 1),
        (1, 16, 0, 4),
        (1, 16, 0, 16),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("subject.ts");
        let old = ts_source(classes, methods, payload, false);
        // The empty structure still has a real byte change and a stored empty baseline.
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
        let (expected_change, actual_classes, actual_methods) = oracle(&file, &old, &new);
        assert_eq!(
            (actual_classes, actual_methods),
            (classes, classes * methods)
        );
        work::reset();
        let cold = dispatch(&state, &file, &root);
        assert_eq!(
            body(&cold),
            format!(
                "// --- AST Diff ---\n// No baseline snapshot for this file yet.\n// Current state stored as baseline ({classes} classes).\n// Call diff_code_context again after the file changes to see the delta."
            )
        );
        assert_eq!(work::snapshot().diff.snapshot_clones, [0, 0]);
        assert_eq!(
            cold["result"]["_meta"]["cache_hints"]["breakpoints"][0]["breaker"],
            "rolling"
        );
        assert_eq!(
            cold["result"]["_meta"]["cache_hints"]["breakpoints"][0]["region"],
            "tail"
        );
        let cold_cache = cache_identity(&state, &key, &old);
        std::fs::write(&path, &new).unwrap();
        state.invalidate_source_cache(&file);
        work::reset();
        let changed = dispatch(&state, &file, &root);
        assert_eq!(body(&changed), expected_change);
        assert_eq!(
            work::snapshot().diff.snapshot_clones,
            [0, 1],
            "independent changed-source clone"
        );
        assert!(
            changed["result"].get("_meta").is_none(),
            "existing rolling breakpoint dedup"
        );
        let changed_cache = cache_identity(&state, &key, &new);
        let pointer = {
            let cache = state.cache_read();
            cache.get_baseline(&key).unwrap() as *const crate::diff::CapturedStructure as usize
        };
        let hash = crate::cache::content_hash(new.as_bytes());
        let expected_warm = format!(
            "// --- AST Diff ---\n// No changes since last snapshot ({classes} classes).\n// Hash: {}",
            &hash[..12]
        );
        let mut warm_responses = Vec::new();
        work::reset();
        for _ in 0..repeats {
            let response = dispatch(&state, &file, &root);
            assert_eq!(body(&response), expected_warm);
            assert!(response["result"].get("_meta").is_none());
            warm_responses.push(normalize_response(&response, &file));
            let cache = state.cache_read();
            assert_eq!(
                cache.get_baseline(&key).unwrap() as *const crate::diff::CapturedStructure as usize,
                pointer,
                "unchanged owner identity"
            );
        }
        let measured = work::snapshot().diff;
        assert_eq!(
            (
                measured.snapshot_builds,
                measured.parser_attempts,
                measured.comparisons
            ),
            (0, 0, 0)
        );
        assert_eq!(measured.snapshot_clones[1], 0);
        redundant += measured.snapshot_clones[0];
        let warm_cache = cache_identity(&state, &key, &new);
        assert_eq!(changed_cache, warm_cache);
        eprintln!(
            "AF002_SHORTCUT_SEM {}",
            json!({"axes":[classes,methods,payload,repeats],
            "cold":normalize_response(&cold,&file),"changed":normalize_response(&changed,&file),
            "warm":warm_responses,"cold_cache":cold_cache,"changed_cache":changed_cache,"warm_cache":warm_cache})
        );
        eprintln!(
            "AF002_SHORTCUT_WORK {}",
            json!({"axes":[classes,methods,payload,repeats],"warm":measured})
        );
    }
    redundant
}

#[test]
fn af002_diff_shortcut_semantics() {
    let _serial = crate::protocol::handler_response_serial();
    run_cases();
}

#[test]
fn af002_diff_shortcut_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let redundant = run_cases();
    assert_eq!(
        redundant, 0,
        "Diff shortcut redundant snapshot clone budget"
    );
}
