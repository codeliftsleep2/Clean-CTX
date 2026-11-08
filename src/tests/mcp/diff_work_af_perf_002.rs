//! Agent MaxHeadRoom: actual public snapshot diff copy and same-name matching work.
use crate::compression::Fidelity;
use crate::diff::{build_snapshot, diff_snapshots, diff_summary, format_diff};
use crate::mcp::{McpState, tools::dispatch_tools_call};
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};

#[path = "diff_shortcut_budget_af_perf_002.rs"]
mod shortcut_budget;

#[path = "diff_shortcut_protection_af_perf_002.rs"]
mod shortcut_protection;

#[path = "overload_budget_af_perf_002.rs"]
mod overload_budget;

fn dispatch(state: &McpState, file: &str, root: &tempfile::TempDir) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "diff_code_context",
        &json!({"arguments":{
        "filePath":file,"workspaceRoot":root.path().to_string_lossy(),"fidelity":"high"}}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("public diff response");
    assert!(response.get("error").is_none(), "{response}");
    response
}
fn body(response: &Value) -> &str {
    response["result"]["content"][0]["text"].as_str().unwrap()
}
fn state(root: &tempfile::TempDir) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}
fn oracle(file: &str, before: &str, after: &str) -> (String, usize, usize) {
    let baseline = build_snapshot(before, Fidelity::High).unwrap();
    let current = build_snapshot(after, Fidelity::High).unwrap();
    let classes = current.classes.len();
    let methods = current
        .classes
        .iter()
        .map(|c| c.methods.len())
        .sum::<usize>()
        + current.orphan_methods.len();
    let actions = diff_snapshots(&baseline, &current);
    let (added, removed, modified, unchanged) = diff_summary(&actions);
    (
        format!(
            "// --- AST Diff: {file} ---\n// +{added} -{removed} ~{modified} ={unchanged} (classes/methods/fields/imports)\n{}",
            format_diff(&actions, Fidelity::High)
        ),
        classes,
        methods,
    )
}
fn report(phase: &str, axes: &Value, start: std::time::Instant) {
    eprintln!(
        "AF002_DIFF {}",
        json!({"phase":phase,"axes":axes,"us":start.elapsed().as_micros(),
        "work":work::snapshot().diff,"scope":"public operation; named clone payload and actual predicates, not allocator bytes or equality CPU work"})
    );
}
fn ts_source(classes: usize, methods: usize, payload: usize, changed: bool) -> String {
    let mut source = String::from("// snapshot diff Unicode: café\n");
    for c in 0..classes {
        source.push_str(&format!(
            "export class Subject{c} {{\n value: number = 1;\n"
        ));
        for m in 0..methods {
            let value = if changed && c == 0 && m == 0 { 2 } else { 1 };
            source.push_str(&format!(
                " m{m}() {{ const detail = '{}'; return {value}; }}\n",
                "x".repeat(payload)
            ));
        }
        source.push_str("}\n");
    }
    source
}
#[test]
fn af002_diff_public_snapshot_copy_growth() {
    let _serial = crate::protocol::handler_response_serial();
    for (classes, methods, payload, repeats) in [
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
        let new = ts_source(classes, methods, payload, true);
        std::fs::write(&path, &old).unwrap();
        let file = std::fs::canonicalize(&path)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let state = state(&root);
        let (expected, actual_classes, actual_methods) = oracle(&file, &old, &new);
        assert_eq!(actual_classes, classes);
        assert_eq!(actual_methods, classes * methods);
        let axes = json!({"classes":classes,"methods_per_class":methods,"payload_per_body":payload,"warm_requests":repeats,
            "before_bytes":old.len(),"after_bytes":new.len()});
        work::reset();
        let start = std::time::Instant::now();
        let cold = dispatch(&state, &file, &root);
        report("cold", &axes, start);
        assert!(body(&cold).contains("No baseline snapshot"));
        assert_eq!(work::snapshot().diff.snapshot_builds, 1);
        assert_eq!(work::snapshot().diff.snapshot_clones, [0, 0]);
        std::fs::write(&path, &new).unwrap();
        state.invalidate_source_cache(&file);
        work::reset();
        let start = std::time::Instant::now();
        let changed = dispatch(&state, &file, &root);
        report("changed", &axes, start);
        assert_eq!(body(&changed), expected);
        let counts = work::snapshot().diff;
        assert_eq!(counts.snapshot_builds, 1);
        assert_eq!(counts.comparisons, 1);
        assert_eq!(counts.snapshot_clones, [0, 1]);
        let key = format!("{file}::{}", Fidelity::High as u8);
        let hash = crate::cache::content_hash(new.as_bytes());
        {
            let cache = state.cache_read();
            assert_eq!(cache.get_baseline_hash(&key), Some(hash.as_str()));
            assert_eq!(cache.get_baseline(&key).unwrap().classes.len(), classes);
        }
        let unchanged = format!(
            "// --- AST Diff ---\n// No changes since last snapshot ({classes} classes).\n// Hash: {}",
            &hash[..12]
        );
        work::reset();
        let start = std::time::Instant::now();
        for _ in 0..repeats {
            assert_eq!(body(&dispatch(&state, &file, &root)), unchanged);
        }
        report("byte-identical", &axes, start);
        let counts = work::snapshot().diff;
        assert_eq!(counts.snapshot_builds, 0);
        assert_eq!(counts.parser_attempts, 0);
        assert_eq!(counts.comparisons, 0);
        assert_eq!(counts.snapshot_clones, [0, 0]);
        assert_eq!(counts.cloned_records[0], [0; 4]);
        assert_eq!(counts.cloned_string_bytes[0], 0);
    }
}
fn cs_source(methods: usize, width: usize, mode: &str, changed: bool) -> String {
    let mut source = format!(
        "// phase {}\nusing System;\nclass Subject {{\n",
        if changed { "new" } else { "old" }
    );
    let mut order: Vec<_> = (0..methods).collect();
    if changed && mode == "reordered" {
        order.reverse();
    }
    for i in order {
        let value = if changed && (mode == "all-body" || (mode == "one-body" && i == 0)) {
            2
        } else {
            1
        };
        source.push_str(&format!(
            " public int Run{}(Type{i} value) {{ return {value}; }}\n",
            i / width
        ));
    }
    source.push_str("}\n");
    source
}
#[test]
fn af002_diff_public_overload_matching() {
    let _serial = crate::protocol::handler_response_serial();
    for (methods, width, mode) in [
        (1, 1, "all-body"),
        (2, 2, "all-body"),
        (8, 1, "all-body"),
        (8, 8, "all-body"),
        (32, 8, "all-body"),
        (32, 32, "all-body"),
        (128, 128, "all-body"),
        (32, 32, "reordered"),
        (32, 32, "comment"),
        (8, 8, "one-body"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("subject.cs");
        let old = cs_source(methods, width, mode, false);
        let new = cs_source(methods, width, mode, true);
        std::fs::write(&path, &old).unwrap();
        let file = std::fs::canonicalize(&path)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let state = state(&root);
        let (expected, classes, actual_methods) = oracle(&file, &old, &new);
        assert_eq!(classes, 1);
        assert_eq!(actual_methods, methods);
        let produced = build_snapshot(&new, Fidelity::High).unwrap();
        let keys: std::collections::BTreeSet<_> = produced.classes[0]
            .methods
            .iter()
            .map(|m| crate::diff::keys::method_key(&m.sig))
            .collect();
        assert_eq!(
            keys.len(),
            methods / width,
            "actual same-name group population"
        );
        dispatch(&state, &file, &root);
        std::fs::write(&path, &new).unwrap();
        state.invalidate_source_cache(&file);
        work::reset();
        let start = std::time::Instant::now();
        let response = dispatch(&state, &file, &root);
        report(
            "overload-change",
            &json!({"methods":methods,"group_width":width,"mode":mode}),
            start,
        );
        assert_eq!(body(&response), expected);
        let counts = work::snapshot().diff;
        assert_eq!(counts.comparisons, 1);
        assert_eq!(counts.snapshot_clones, [0, 1]);
        assert_eq!(counts.group_records, [methods, methods]);
        if mode == "comment" || mode == "reordered" {
            assert_eq!(counts.signature_candidate_visits, 0);
        }
    }
}
