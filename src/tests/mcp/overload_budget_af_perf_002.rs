//! Agent MaxHeadRoom: genuine original public matcher-search budget.
use super::*;
use crate::diff::differ::overload_equivalence::original;

fn dispatch_fidelity(
    state: &McpState,
    file: &str,
    root: &tempfile::TempDir,
    fidelity: Fidelity,
) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "diff_code_context",
        &json!({"arguments":{
        "filePath":file,"workspaceRoot":root.path().to_string_lossy(),"fidelity":fidelity}}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("public response");
    assert!(response.get("error").is_none(), "{response}");
    response
}
#[test]
fn af002_overload_public_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let mut predicates = [0usize; 2];
    for (methods, width, mode) in [
        (1, 1, "all-body"),
        (8, 1, "all-body"),
        (8, 8, "all-body"),
        (32, 8, "all-body"),
        (32, 32, "all-body"),
        (128, 128, "all-body"),
        (32, 32, "comment"),
        (32, 32, "reordered"),
        (8, 8, "one-body"),
    ] {
        for fidelity in [
            Fidelity::Low,
            Fidelity::Medium,
            Fidelity::High,
            Fidelity::Edit,
            Fidelity::Verbatim,
        ] {
            let root = tempfile::tempdir().unwrap();
            let path = root.path().join("subject.cs");
            let before = cs_source(methods, width, mode, false);
            let after = cs_source(methods, width, mode, true);
            std::fs::write(&path, &before).unwrap();
            let file = std::fs::canonicalize(&path)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            let state = state(&root);
            let old = build_snapshot(&before, fidelity).unwrap();
            let new = build_snapshot(&after, fidelity).unwrap();
            assert_eq!(
                new.classes.iter().map(|c| c.methods.len()).sum::<usize>(),
                methods
            );
            work::reset();
            let actions = original::diff_snapshots(&old, &new);
            let expected_pairs = work::snapshot().overload.pairs;
            let (added, removed, modified, unchanged) = diff_summary(&actions);
            let expected = format!(
                "// --- AST Diff: {file} ---\n// +{added} -{removed} ~{modified} ={unchanged} (classes/methods/fields/imports)\n{}",
                format_diff(&actions, fidelity)
            );
            let cold = dispatch_fidelity(&state, &file, &root, fidelity);
            std::fs::write(&path, &after).unwrap();
            state.invalidate_source_cache(&file);
            work::reset();
            let start = std::time::Instant::now();
            let mut response = dispatch_fidelity(&state, &file, &root, fidelity);
            let us = start.elapsed().as_micros();
            let observed = work::snapshot();
            assert_eq!(body(&response), expected);
            assert_eq!(
                observed.overload.pairs, expected_pairs,
                "public occurrence-index trace"
            );
            let actual = work::unobserved(|| diff_snapshots(&old, &new));
            assert_eq!(actual, actions, "complete public-boundary action oracle");
            let key = format!("{file}::{}", fidelity as u8);
            let hash = crate::cache::content_hash(after.as_bytes());
            {
                let cache = state.cache_read();
                assert_eq!(cache.get_baseline(&key), Some(&new));
                assert_eq!(cache.get_baseline_hash(&key), Some(hash.as_str()));
            }
            predicates[0] += observed.diff.exact_candidate_visits;
            predicates[1] += observed.diff.signature_candidate_visits;
            response["result"]["content"][0]["text"] =
                json!(body(&response).replace(&file, "<FILE>"));
            eprintln!(
                "AF002_OVERLOAD_SEM {}",
                json!({"axes":[methods,width,mode],"fidelity":fidelity,
                "cold":cold,"response":response,"actions":actions,"pairs":expected_pairs,"hash":hash,"snapshot":format!("{new:?}")})
            );
            eprintln!(
                "AF002_OVERLOAD_WORK {}",
                json!({"axes":[methods,width,mode],"fidelity":fidelity,"us":us,
                "original":observed.diff,"replacement":observed.overload})
            );
        }
    }
    assert_eq!(
        predicates,
        [0, 0],
        "Overload redundant candidate search budget"
    );
}
