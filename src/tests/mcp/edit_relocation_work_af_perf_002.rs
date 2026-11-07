//! Agent MaxHeadRoom: actual registered apply_edit relocation characterization.
use crate::mcp::{McpState, tools::dispatch_tools_call};
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};

#[test]
fn af002_edit_end_public_budget() {
    let rows = growth();
    assert_eq!(
        rows.iter()
            .map(|c| c.attachment_predicates + c.editable_predicates)
            .sum::<usize>(),
        0,
        "Edit exact-end redundant full-input join budget"
    );
    for counts in rows {
        assert_eq!(counts.end_setup_declarations, counts.declaration_captures);
        assert_eq!(counts.end_setup_units, counts.editable_units);
        assert_eq!(counts.end_attachment_lookups, counts.editable_units);
        assert_eq!(counts.end_editability_lookups, counts.syntax_declarations);
        assert_eq!(
            counts.end_retained_declarations,
            counts.declaration_captures
        );
        assert_eq!(counts.end_same_end_candidates, counts.editable_units);
        assert!(counts.end_declaration_capacity >= counts.end_retained_declarations);
        assert!(counts.end_buckets > 0 && counts.end_key_comparisons > 0);
    }
}

fn dispatch(state: &McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses()
        .pop()
        .expect("registered response")
}
fn setup(root: &tempfile::TempDir, source: &str) -> (McpState, String) {
    let file = root
        .path()
        .join("subject.ts")
        .to_string_lossy()
        .into_owned();
    std::fs::write(&file, source).unwrap();
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.persistence.enabled = true;
    config.persistence.auto_save = true;
    config.persistence.db_path = root.path().join("edit.db").to_string_lossy().into_owned();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    let response = dispatch(
        &state,
        "provide_code_context",
        json!({"filePath":file,
        "workspaceRoot":root.path().to_string_lossy(),"fidelity":"edit"}),
    );
    assert!(response.get("error").is_none(), "baseline: {response}");
    (state, file)
}
fn source(methods: usize, fields: usize, padding: usize) -> String {
    let mut text = String::from("// Unicode: café\r\nexport class Subject {\r\n");
    for i in 0..fields {
        text.push_str(&format!("  f{i}: number = {i};\r\n"));
    }
    for i in 0..methods {
        text.push_str(&format!("  m{i}() {{ return 1; }}\r\n"));
    }
    text.push_str("}\r\n// ");
    text.push_str(&"x".repeat(padding));
    text.push_str("\r\n");
    text
}
fn durable_bytes(state: &McpState, file: &str) -> Vec<u8> {
    state
        .persistence_store_lock()
        .as_ref()
        .unwrap()
        .sqlite()
        .unwrap()
        .baseline_binary(file)
        .unwrap()
        .expect("durable baseline")
}
#[test]
fn af002_edit_relocation_public_growth() {
    growth();
}
fn growth() -> Vec<work::EditWork> {
    let _serial = crate::protocol::handler_response_serial();
    let mut rows = Vec::new();
    for (methods, fields, operations, bare, padding) in [
        (1, 0, 1, false, 0),
        (2, 0, 1, false, 0),
        (8, 0, 1, false, 0),
        (32, 0, 1, false, 0),
        (128, 0, 1, false, 0),
        (256, 0, 1, false, 0),
        (16, 16, 1, false, 0),
        (16, 128, 1, false, 0),
        (32, 0, 8, false, 0),
        (32, 0, 16, false, 0),
        (16, 0, 1, true, 0),
        (16, 0, 1, false, 4096),
    ] {
        let root = tempfile::tempdir().unwrap();
        let prior = source(methods, fields, padding);
        let (state, file) = setup(&root, &prior);
        let alias = state.alias_for_path(&file).unwrap();
        let version = state.file_version(&alias).unwrap();
        let mut expected = prior.clone();
        let mut ops = Vec::new();
        for i in 0..operations {
            expected = expected.replace(
                &format!("  m{i}() {{ return 1; }}"),
                &format!("  m{i}() {{ return 2; }}"),
            );
            ops.push(json!({"type":"replace_body","target":if bare {format!("m{i}")}else{format!("Subject.m{i}")},
                "expectedOldText":"{ return 1; }","newText":"{ return 2; }"}));
        }
        work::reset();
        let start = std::time::Instant::now();
        let response = dispatch(
            &state,
            "apply_edit",
            json!({"filePath":file,
            "workspaceRoot":root.path().to_string_lossy(),"operations":ops,"verify":true}),
        );
        let us = start.elapsed().as_micros();
        let counts = work::snapshot().edit;
        eprintln!(
            "AF002_EDIT {}",
            json!({"phase":"success","methods":methods,"fields":fields,
            "operations":operations,"bare":bare,"padding":padding,"source_bytes":prior.len(),"work":counts,"us":us})
        );
        assert!(response.get("error").is_none(), "edit: {response}");
        assert_eq!(std::fs::read(&file).unwrap(), expected.as_bytes());
        assert_eq!(counts.candidate_compile_attempts, [1, 1]);
        assert_eq!(counts.declaration_parses, 1);
        assert_eq!(counts.table_builds, 1);
        assert_eq!(counts.editable_units, methods);
        assert_eq!(counts.resolution_calls, operations);
        // Characterize actual join work; the separate unchanged budget guards repair.
        let returned = response["result"]["structuredContent"]["operations"]
            .as_array()
            .unwrap();
        assert_eq!(returned.len(), operations);
        for (i, record) in returned.iter().enumerate() {
            let prefix = format!("  m{i}() ");
            let start = prior.find(&prefix).unwrap() + prefix.len();
            assert_eq!(record["startByte"], json!(start));
            assert_eq!(record["endByte"], json!(start + "{ return 2; }".len()));
            assert_eq!(record["newText"], json!("{ return 2; }"));
            assert_eq!(record["byteDelta"], json!(0));
        }
        let new_version = state.file_version(&alias).unwrap();
        assert!(new_version > version);
        let durable = state
            .persistence_store_lock()
            .as_ref()
            .unwrap()
            .sqlite()
            .unwrap()
            .load_durable_context(&file, None)
            .unwrap()
            .unwrap();
        assert_eq!(durable.ir.file_id, file);
        assert_eq!(durable.ir.version, new_version);
        let hash = crate::cache::content_hash(expected.as_bytes());
        assert_eq!(durable.source_hash, hash);
        assert_eq!(state.ir_context_read().get_source_hash(&alias), Some(&hash));
        for op in &durable.ir.instructions {
            if let crate::ir::opcodes::CoreOp::Body(_, body, Some(start), Some(end)) = op {
                assert_eq!(
                    body.as_bytes(),
                    &expected.as_bytes()[*start as usize..*end as usize]
                );
            }
        }
        rows.push(counts);
    }
    rows
}
#[test]
fn af002_edit_relocation_refusal_controls() {
    let _serial = crate::protocol::handler_response_serial();
    for (target, old) in [
        ("m0", "{ return 1; }"),
        ("Subject.f0", "{ return 1; }"),
        ("Subject.m0", "WRONG"),
    ] {
        let root = tempfile::tempdir().unwrap();
        let prior = source(8, 8, 0) + "export class Other { m0() { return 1; } }\r\n";
        let (state, file) = setup(&root, &prior);
        let alias = state.alias_for_path(&file).unwrap();
        let version = state.file_version(&alias);
        let durable = durable_bytes(&state, &file);
        let live = state.ir_context_read().get_ir(&alias).unwrap().clone();
        work::reset();
        let start = std::time::Instant::now();
        let response = dispatch(
            &state,
            "apply_edit",
            json!({"filePath":file,
            "workspaceRoot":root.path().to_string_lossy(),"operations":[{"type":"replace_body", "target":target,
            "expectedOldText":old,"newText":"{ return 2; }"}]}),
        );
        let counts = work::snapshot().edit;
        eprintln!(
            "AF002_EDIT {}",
            json!({"phase":"refusal","target":target,"work":counts,"us":start.elapsed().as_micros()})
        );
        assert!(
            response.get("error").is_some(),
            "expected refusal: {response}"
        );
        assert_eq!(counts.candidate_compile_attempts, [1, 0]);
        assert_eq!(state.file_version(&alias), version);
        assert_eq!(state.ir_context_read().get_ir(&alias).unwrap(), &live);
        assert_eq!(std::fs::read(&file).unwrap(), prior.as_bytes());
        assert_eq!(durable_bytes(&state, &file), durable);
    }
}
