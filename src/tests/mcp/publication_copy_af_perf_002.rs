//! A1: actual default publication copies, logical payload only; no repair budget.
use super::{assert_published_exact_call, assert_save_call, query};
use crate::work_observations_af_perf_002 as work;
use serde_json::json;

fn report(label: &str, axes: serde_json::Value, started: std::time::Instant) {
    let c = work::snapshot();
    eprintln!(
        "AF002_A1 {}",
        json!({"phase":label,"axes":axes,
        "sites":c.publication_copies,"sinks":c.publication_sinks,"legacy_accessor":c.artifact_publication_clones,
        "projection_owners":c.artifact_projection_owners,"us":started.elapsed().as_micros(),
        "accounting":"logical copied items and named string payload; not allocator bytes"})
    );
}

#[test]
fn af002_a1_default_csharp_publication_growth() {
    let _serial = crate::protocol::handler_response_serial();
    let mut axes = vec![(2, 1), (4, 1), (8, 1), (16, 1)];
    axes.extend([2, 4, 8, 16, 64, 256].into_iter().map(|calls| (2, calls)));
    axes.push((16, 16));
    for (owners, calls) in axes {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("Caller.cs"),
            format!(
                "class Caller {{ void Run(Target target) {{ {} }} }}",
                "target.Save();".repeat(calls)
            ),
        )
        .unwrap();
        std::fs::write(
            root.path().join("Target.cs"),
            "class Target { public void Save() {} }",
        )
        .unwrap();
        for i in 2..owners {
            std::fs::write(
                root.path().join(format!("Owner{i}.cs")),
                format!("class Owner{i} {{ Target target; }}"),
            )
            .unwrap();
        }
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = false;
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        let dimensions = json!({"owners":owners,"calls":calls});
        work::reset();
        let started = std::time::Instant::now();
        query(&state, root.path(), "TypeRef", "Target");
        report("cold-query", dimensions.clone(), started);
        assert_eq!(
            state
                .workspace_index_read()
                .file_map()
                .keys()
                .filter(|f| f.ends_with(".cs"))
                .count(),
            owners
        );
        state.mark_csharp_authority_dirty();
        state.reset_test_full_semantic_parses();
        work::reset();
        let started = std::time::Instant::now();
        assert_save_call(&query(&state, root.path(), "Method", "Save"));
        assert_published_exact_call(&state, root.path());
        report("dirty-query", dimensions.clone(), started);
        let dirty = work::snapshot();
        assert_eq!(
            dirty
                .publication_copies
                .get("reconciliation-index")
                .unwrap()
                .calls,
            owners
        );
        assert_eq!(state.test_full_semantic_parses(), 0);
        let alias = state
            .alias_for_path(&root.path().join("Caller.cs").to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let before = serde_json::to_value(&snapshot.edges).unwrap();
        work::reset();
        let started = std::time::Instant::now();
        assert_save_call(&query(&state, root.path(), "Method", "Save"));
        assert_published_exact_call(&state, root.path());
        report("warm-query", dimensions, started);
        assert!(
            work::snapshot().publication_copies.is_empty(),
            "unchanged warm query selected-copy-site control"
        );
        assert_eq!(
            serde_json::to_value(state.semantic_snapshot(&alias).unwrap().edges).unwrap(),
            before
        );
    }
}

#[test]
fn af002_a1_default_compression_publication_growth() {
    let _serial = crate::protocol::handler_response_serial();
    let mut axes: Vec<_> = [1, 2, 4, 8, 16, 64, 256]
        .into_iter()
        .map(|members| (members, 8, 1))
        .collect();
    axes.extend([16, 64, 256].into_iter().map(|bytes| (1, bytes, 1)));
    axes.extend([2, 4, 8, 16].into_iter().map(|requests| (1, 8, requests)));
    axes.push((16, 64, 4));
    for persistence in [false, true] {
        for &(members, name_bytes, requests) in &axes {
            let root = tempfile::tempdir().unwrap();
            let file = root.path().join("publication.ts");
            let methods: String = (0..members)
                .map(|i| {
                    format!(
                        "method{}{}(target: Target): void {{ target.save(); }}",
                        "x".repeat(name_bytes),
                        i
                    )
                })
                .collect();
            let source = format!(
                "export class Workload {{ {methods} }} class Target {{ save(): void {{}} }}"
            );
            std::fs::write(&file, &source).unwrap();
            let mut config = crate::tests::test_config();
            config.cbm.enabled = false;
            config.persistence.enabled = persistence;
            config.persistence.auto_save = persistence;
            config.persistence.db_path = root.path().join("a1.db").to_string_lossy().into_owned();
            let state = crate::mcp::McpState::new(config);
            *state.graph_bridge_lock() = None;
            work::reset();
            let started = std::time::Instant::now();
            for request in 0..requests {
                crate::protocol::captured_responses().clear();
                crate::mcp::tools::dispatch_tools_call(
                    &json!(request),
                    "compress_code_context",
                    &json!({"arguments":{"filePath":file,"workspaceRoot":root.path(),"fidelity":"high"}}),
                    &state,
                );
                let response = crate::protocol::captured_responses().pop().unwrap();
                assert!(response.get("error").is_none(), "{response}");
            }
            report(
                "compress",
                json!({"members":members,"requested_name_bytes":name_bytes,
                "requests":requests,"persistence":persistence,"source_bytes":source.len()}),
                started,
            );
            let counts = work::snapshot();
            assert!(
                counts
                    .publication_copies
                    .get("handler-live-ir")
                    .is_some_and(|c| c.calls > 0),
                "must reach successful IR publication"
            );
            let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
            let snapshot = state.semantic_snapshot(&alias).unwrap();
            assert_eq!(
                snapshot.source_hash,
                crate::cache::content_hash(source.as_bytes())
            );
            assert!(
                !snapshot.edges.is_empty(),
                "actual semantic payload required"
            );
            assert!(state.ir_context_read().get_ir(&alias).is_some());
        }
    }
}
