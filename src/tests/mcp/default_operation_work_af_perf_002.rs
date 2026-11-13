//! Public discovery characterization, not an optimization RED or latency claim.
use super::{Work, reset, snapshot};
use crate::compression::Fidelity;
use crate::ir::{CompiledIR, binary_wire, string_table::StringTable};
use crate::mcp::{McpState, context_store::ContextStore, tools::dispatch_tools_call};
use serde_json::{Value, json};

#[path = "binary_clone_public_af_perf_002.rs"]
mod binary_clone;

fn state(root: &tempfile::TempDir) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root
        .path()
        .join("discovery.db")
        .to_string_lossy()
        .into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("registered response");
    assert!(response.get("error").is_none(), "{response}");
    response
}
fn source(classes: usize, methods: usize, padding: usize) -> String {
    let mut source = String::new();
    for c in 0..classes {
        source.push_str(&format!("export class Subject{c} {{\n"));
        for m in 0..methods {
            source.push_str(&format!(
                " m{m}(): number {{ const payload = '{}'; return {m}; }}\n",
                "共有".repeat(padding)
            ));
        }
        source.push_str("}\n");
    }
    source
}
fn baseline(state: &McpState, file: &str) -> (Vec<u8>, CompiledIR) {
    let guard = state.persistence_store_lock();
    let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
    let bytes = sqlite
        .baseline_binary(file)
        .unwrap()
        .expect("durable bytes");
    let ir = binary_wire::decode(&bytes).expect("complete binary");
    assert_eq!(ir.file_id, file);
    (bytes, ir)
}
fn assert_encoding(work: &Work, ir: &CompiledIR, calls: usize) {
    let mut table = StringTable::from_instructions(&ir.instructions);
    table.intern(&ir.file_id);
    let bytes = table.strings().iter().map(String::len).sum::<usize>();
    assert_eq!(work.binary_calls, calls);
    assert_eq!(work.new_table_strings, table.len() * calls);
    assert_eq!(work.new_table_string_bytes, bytes * calls);
    assert_eq!(work.string_table_builds, calls);
    assert!(work.operand_lookups >= calls);
    assert!(work.reverse_capacity >= work.reverse_entries);
}
fn report(phase: &str, axes: &Value, work: &Work, elapsed: std::time::Duration, response: &Value) {
    eprintln!(
        "AF002_DEFAULT {}",
        json!({
            "phase":phase,"axes":axes,"work":work,"public_us":elapsed.as_micros(),
            "response":response,"scope":"logical copy lengths, map operations and capacity; not allocation events or inspected bytes"
        })
    );
}

#[test]
fn af002_default_public_encoding_work() {
    let _serial = crate::protocol::handler_response_serial();
    for (classes, methods, padding, repeats) in [
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
        let file = root
            .path()
            .join("subject.ts")
            .to_string_lossy()
            .into_owned();
        let source = source(classes, methods, padding);
        std::fs::write(&file, &source).unwrap();
        let state = state(&root);
        let axes = json!({"classes":classes,"methods_per_class":methods,
            "body_utf8_repetitions":padding,"repeats":repeats,"source_bytes":source.len()});
        reset();
        let start = std::time::Instant::now();
        let response = dispatch(
            &state,
            "compress_code_context",
            json!({
                "filePath":file,"workspaceRoot":root.path().to_string_lossy(),"fidelity":"edit"
            }),
        );
        let elapsed = start.elapsed();
        let observed = snapshot();
        let (cold_bytes, ir) = baseline(&state, &file);
        assert_eq!(
            ir.instructions
                .iter()
                .filter(|op| matches!(op, crate::ir::CoreOp::DefClass(..)))
                .count(),
            classes
        );
        assert_eq!(
            ir.instructions
                .iter()
                .filter(|op| matches!(op, crate::ir::CoreOp::DefMethod(..)))
                .count(),
            classes * methods
        );
        assert_eq!(
            ir.instructions
                .iter()
                .filter(|op| matches!(op, crate::ir::CoreOp::Body(..)))
                .count(),
            classes * methods
        );
        assert_encoding(&observed, &ir, 1);
        assert_eq!(observed.binary_output_bytes, cold_bytes.len());
        report("cold_compress", &axes, &observed, elapsed, &response);

        let alias = state.alias_for_path(&file).expect("live alias");
        let tuples = state.ir_context_read().get_ir(&alias).cloned().unwrap();
        let hash = state
            .ir_context_read()
            .get_source_hash(&alias)
            .cloned()
            .unwrap();
        assert_eq!(ir.instructions.len(), tuples.len());
        reset();
        let start = std::time::Instant::now();
        let response = dispatch(&state, "save_context", json!({"filePath":file}));
        let elapsed = start.elapsed();
        let observed = snapshot();
        assert_encoding(&observed, &ir, 1);
        assert_eq!(baseline(&state, &file).0, cold_bytes);
        report("checkpoint", &axes, &observed, elapsed, &response);

        reset();
        let start = std::time::Instant::now();
        let mut responses = Vec::new();
        for _ in 0..repeats {
            let response = dispatch(&state, "save_context", json!({"filePath":file}));
            assert_eq!(response["result"]["_meta"]["saved"], 0, "{response}");
            assert_eq!(response["result"]["_meta"]["already_durable"], true);
            responses.push(response);
        }
        let elapsed = start.elapsed();
        let observed = snapshot();
        assert_encoding(&observed, &ir, repeats);
        assert_eq!(baseline(&state, &file).0, cold_bytes);
        assert_eq!(state.ir_context_read().get_ir(&alias), Some(&tuples));
        assert_eq!(state.ir_context_read().get_source_hash(&alias), Some(&hash));
        assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
        report(
            "already_durable",
            &axes,
            &observed,
            elapsed,
            &json!(responses),
        );

        reset();
        let start = std::time::Instant::now();
        let response = dispatch(&state, "context_history", json!({"filePath":file}));
        let elapsed = start.elapsed();
        let observed = snapshot();
        assert_eq!(observed.metadata_queries, 1);
        assert_eq!(observed.metadata_rows, 1);
        assert_eq!(observed.binary_calls, 0);
        assert!(
            response["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("IR Baseline: yes")
        );
        report("checkpoint_history", &axes, &observed, elapsed, &response);
    }
}

#[test]
fn af002_default_public_history_payload_work() {
    let _serial = crate::protocol::handler_response_serial();
    for (repetitions, requests) in [
        (0, 1),
        (16, 1),
        (1024, 1),
        (65536, 1),
        (1024, 4),
        (1024, 16),
    ] {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("stored.ts").to_string_lossy().into_owned();
        let state = state(&root);
        let pretty = "共有".repeat(repetitions);
        // Seed an ordinary committed ContextStore row; no source/semantic ownership invented.
        {
            let guard = state.persistence_store_lock();
            let mut sqlite = guard.as_ref().unwrap().sqlite().unwrap();
            sqlite
                .save_context(
                    &file,
                    Fidelity::High,
                    &pretty,
                    None,
                    "discovery-hash",
                    17,
                    9,
                )
                .unwrap();
        }
        let edges = state.workspace_index_read().edge_count();
        reset();
        let start = std::time::Instant::now();
        let mut responses = Vec::new();
        for _ in 0..requests {
            let response = dispatch(&state, "context_history", json!({"filePath":file}));
            let text = response["result"]["content"][0]["text"].as_str().unwrap();
            assert_eq!(
                text,
                format!(
                    "File: {file}\n  IR Baseline: no\n  Context Store: yes\n  Context Version: 1\n  Cache Hit Rate: 0/0 (0%)\n  Cache Tokens Saved: 0"
                )
            );
            responses.push(response);
        }
        let elapsed = start.elapsed();
        let observed = snapshot();
        assert_eq!(observed.metadata_queries, requests);
        assert_eq!(observed.metadata_rows, requests);
        // Metadata reads validate presentation bytes without allocating a discarded String.
        assert_eq!(observed.validated_pretty_bytes, pretty.len() * requests);
        assert_eq!(observed.discarded_pretty_bytes, 0);
        assert_eq!(observed.discarded_pretty_strings, 0);
        assert!(state.alias_for_path(&file).is_none());
        assert_eq!(state.workspace_index_read().edge_count(), edges);
        report(
            "seeded_metadata_history",
            &json!({"pretty_bytes":pretty.len(),"requests":requests,
            "setup":"ContextStore committed row; not compiled source"}),
            &observed,
            elapsed,
            &json!(responses),
        );
    }
}
