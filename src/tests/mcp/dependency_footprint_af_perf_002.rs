//! A4 default dependency operations: reuse, obligations and conservative peer matching.
use super::{assert_published_exact_call, assert_save_call, query};
use crate::work_observations_af_perf_002 as work;
use serde_json::json;

#[test]
fn af002_a4_public_lookup_budget() {
    af002_a4_default_footprint_dimensions();
    let w = work::snapshot().a4;
    assert_eq!(
        w.seed_class_visits + w.family_class_visits + w.base_find_visits,
        0,
        "A4 redundant public retrieval budget"
    );
}

fn dependencies(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    positives: usize,
    owners: usize,
    depth: usize,
) {
    let file = root.join("Target.cs").to_string_lossy().into_owned();
    let alias = state.alias_for_path(&file).unwrap();
    let mut actual = state.semantic_snapshot(&alias).unwrap().dependencies;
    let mut paths = std::collections::BTreeSet::from([root.join("Caller.cs")]);
    paths.extend((0..positives).map(|i| root.join(format!("Owner{}.cs", i + 2))));
    paths.extend((0..depth).map(|i| root.join(format!("Owner{}.cs", 2 + i % (owners - 2)))));
    let mut expected: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let owner = state.semantic_owner_path(&path.to_string_lossy());
            crate::layers::meta::semantic::SemanticDependency {
                source_hash: Some(crate::cache::content_hash(&std::fs::read(&path).unwrap())),
                file_path: owner,
            }
        })
        .collect();
    expected.sort_by(|a, b| a.file_path.cmp(&b.file_path));
    // Snapshot order follows artifact traversal, which differs between batch
    // reconciliation and single-owner compression. Compare exact records while
    // preserving multiplicity; this characterization imposes no sorting policy.
    actual.sort_by(|a, b| (&a.file_path, &a.source_hash).cmp(&(&b.file_path, &b.source_hash)));
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        serde_json::to_value(expected).unwrap(),
        "exact conservative Target dependency multiplicity and source hashes"
    );
}
fn report(phase: &str, axes: &serde_json::Value, start: std::time::Instant) {
    eprintln!(
        "AF002_A4 {}",
        json!({"phase":phase,"axes":axes,
        "work":work::snapshot().a4,"us":start.elapsed().as_micros(),
        "boundary":"actual public operation; logical visits and bytes offered, not allocator/CPU bytes"})
    );
}
#[test]
fn af002_a4_default_footprint_dimensions() {
    let _serial = crate::protocol::handler_response_serial();
    // F, total classes, matching extra owners, padding bytes, ancestry depth.
    let cases = [
        (2, 32, 0, 0, 0),
        (4, 32, 0, 0, 0),
        (8, 32, 0, 0, 0),
        (16, 32, 0, 0, 0),
        (2, 2, 0, 0, 0),
        (2, 8, 0, 0, 0),
        (2, 64, 0, 0, 0),
        (2, 256, 0, 0, 0),
        (8, 8, 0, 0, 0),
        (8, 8, 1, 0, 0),
        (8, 8, 3, 0, 0),
        (8, 8, 6, 0, 0),
        (4, 4, 0, 64, 0),
        (4, 4, 0, 4096, 0),
        (4, 4, 0, 16384, 0),
        (16, 64, 7, 256, 0),
        (4, 16, 0, 0, 1),
        (4, 16, 0, 0, 4),
        (4, 16, 0, 0, 8),
    ];
    for (owners, classes, positives, padding, depth) in cases {
        let root = tempfile::tempdir().unwrap();
        let mut files = Vec::new();
        let mut source_bytes = 0;
        for i in 0..owners {
            let (name, mut source) = match i {
                0 => (
                    "Caller.cs".to_owned(),
                    "class Caller { void Run(Target target) { target.Save(); } }".to_owned(),
                ),
                1 => (
                    "Target.cs".to_owned(),
                    format!(
                        "class Target {} {{ public void Save() {{}} }}",
                        if depth > 0 { ": Base1" } else { "" }
                    ),
                ),
                _ => (
                    format!("Owner{i}.cs"),
                    format!(
                        "class Owner{i} {{ {} reference; }}",
                        if i - 2 < positives { "Target" } else { "int" }
                    ),
                ),
            };
            if i >= 2 {
                for base in 1..=depth {
                    if (base - 1) % (owners - 2) == i - 2 {
                        source.push_str(&format!(
                            " class Base{base} {} {{}}",
                            if base < depth {
                                format!(": Base{}", base + 1)
                            } else {
                                String::new()
                            }
                        ));
                    }
                }
            }
            for n in (i..classes - owners - depth).step_by(owners) {
                source.push_str(&format!(" class Extra{i}_{n} {{}}"));
            }
            source.push_str(" // ");
            source.push_str(&"x".repeat(padding));
            source_bytes += source.len();
            let path = root.path().join(name);
            std::fs::write(&path, source).unwrap();
            files.push(path);
        }
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        config
            .additional_roots
            .push(root.path().to_string_lossy().into_owned());
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        // Admit every legitimate owner through production file-query entry, not injected artifacts.
        for path in &files {
            crate::protocol::captured_responses().clear();
            crate::mcp::tools::dispatch_tools_call(
                &json!(0),
                "workspace_query",
                &json!({"arguments":{"type":"entities_in_file","file_path":path,"workspaceRoot":root.path()}}),
                &state,
            );
            let response = crate::protocol::captured_responses().pop().unwrap();
            assert!(response.get("error").is_none(), "{response}");
        }
        assert_eq!(
            state
                .workspace_index_read()
                .file_map()
                .keys()
                .filter(|p| p.ends_with(".cs"))
                .count(),
            owners
        );
        let axes = json!({"owners":owners,"classes":classes,"matching_extra_owners":positives,
            "padding_bytes_per_owner":padding,"ancestry_depth":depth,"actual_source_bytes":source_bytes});
        state.mark_csharp_authority_dirty();
        state.reset_test_full_semantic_parses();
        work::reset();
        let start = std::time::Instant::now();
        assert_save_call(&query(&state, root.path(), "Method", "Save"));
        report("dirty-batch-query", &axes, start);
        let dirty = work::snapshot().a4;
        assert_eq!(dirty.footprint_calls[1], 1);
        assert_eq!(
            dirty.symbol_builds[1], 1,
            "existing one index per partition build"
        );
        assert_eq!(
            dirty.symbol_part_visits[1], classes,
            "actual admitted class population"
        );
        assert_eq!(state.test_full_semantic_parses(), 0);
        assert_published_exact_call(&state, root.path());
        dependencies(&state, root.path(), positives, owners, depth);
        work::reset();
        let start = std::time::Instant::now();
        assert_save_call(&query(&state, root.path(), "Method", "Save"));
        report("unchanged-warm-query", &axes, start);
        assert_eq!(
            work::snapshot().a4,
            Default::default(),
            "selected footprint work must remain unreachable on warm query"
        );
        assert_published_exact_call(&state, root.path());
        work::reset();
        crate::protocol::captured_responses().clear();
        let start = std::time::Instant::now();
        crate::mcp::tools::dispatch_tools_call(
            &json!(2),
            "compress_code_context",
            &json!({"arguments":{"filePath":root.path().join("Target.cs"),"workspaceRoot":root.path(),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_none(), "{response}");
        report("single-owner-compression", &axes, start);
        assert_eq!(
            work::snapshot().a4.footprint_calls[0],
            1,
            "reach separate dependency publication operation"
        );
        dependencies(&state, root.path(), positives, owners, depth);
        assert_published_exact_call(&state, root.path());
    }
}
