//! Frozen original public/durable identity: seven bounded cases, including repeats.
use crate::ir::compiler::CompiledIR;
use crate::work_observations_af_perf_002 as work;
use serde_json::{Value, json};
fn normalize(mut value: Value, root: &str) -> Value {
    fn walk(value: &mut Value, root: &str) {
        match value {
            Value::String(text) => *text = text.replace(root, "<ROOT>"),
            Value::Array(items) => items.iter_mut().for_each(|v| walk(v, root)),
            Value::Object(items) => items.values_mut().for_each(|v| walk(v, root)),
            _ => {}
        }
    }
    walk(&mut value, root);
    value
}

fn response_identity(
    mut response: Value,
    root: &str,
    seen: &mut std::collections::BTreeSet<String>,
) -> Value {
    let text = response["result"]["content"][0]["text"].as_str().unwrap();
    let breaker = crate::mcp::cache_hints::compute_baseline_breaker(text);
    let first_emission = seen.insert(breaker.clone());
    let path = "/result/_meta/cache_hints/breakpoints";
    assert_eq!(
        response.pointer(path).is_some(),
        first_emission,
        "preserve first emission and repeated-content breakpoint deduplication"
    );
    if let Some(hints) = response.pointer(path) {
        let hints = hints
            .as_array()
            .expect("present cache hints must be an array");
        assert_eq!(hints.len(), 1);
        assert_eq!(hints[0]["region"], "baseline");
        assert_eq!(hints[0]["breaker"], breaker, "actual unnormalized breaker");
    }
    response = normalize(response, root);
    let breaker = crate::mcp::cache_hints::compute_baseline_breaker(
        response["result"]["content"][0]["text"].as_str().unwrap(),
    );
    if let Some(hints) = response.pointer_mut(path) {
        for bp in hints
            .as_array_mut()
            .expect("present cache hints must be an array")
        {
            bp["breaker"] = json!(breaker);
        }
    }
    response
}

#[test]
fn afperf_a1_frozen_public_and_durable_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let fixtures: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/a1_clone_semantic_responses.json")).unwrap();
    let mut checked = 0;
    for fixture in fixtures
        .iter()
        .filter(|row| row["members"] == 1 || row["requests"] == 4)
    {
        let enabled = fixture["enabled"].as_bool().unwrap();
        let auto_save = fixture["auto_save"].as_bool().unwrap();
        let fidelity = fixture["fidelity"].as_str().unwrap();
        let members = fixture["members"].as_u64().unwrap() as usize;
        let requests = fixture["requests"].as_u64().unwrap() as usize;
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("clone-budget.ts");
        let methods: String = (0..members)
            .map(|i| format!("method{i}(target: Target): void {{ target.save(); }}"))
            .collect();
        let source =
            format!("export class Workload {{ {methods} }} class Target {{ save(): void {{}} }}");
        std::fs::write(&file, &source).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.cache.enabled = true;
        config.persistence.enabled = enabled;
        config.persistence.auto_save = auto_save;
        config.persistence.db_path = root
            .path()
            .join("a1-budget.db")
            .to_string_lossy()
            .into_owned();
        let state = crate::mcp::McpState::new(config);
        *state.graph_bridge_lock() = None;
        let physical = state.durable_owner_path(&file.to_string_lossy());
        let mut responses = Vec::new();
        let mut seen_breakers = std::collections::BTreeSet::new();
        let mut durable_rows = Vec::new();
        work::reset();
        for request in 0..requests {
            crate::protocol::captured_responses().clear();
            crate::mcp::tools::dispatch_tools_call(
                &json!(request),
                "compress_code_context",
                &json!({"arguments":{"filePath":file,"workspaceRoot":root.path(),"fidelity":fidelity}}),
                &state,
            );
            let response = crate::protocol::captured_responses().pop().unwrap();
            assert!(response.get("error").is_none(), "{response}");
            let alias = state.alias_for_path(&file.to_string_lossy()).unwrap();
            assert_ne!(
                alias, physical,
                "fixture must distinguish presentation and physical identity"
            );
            let mut expected = {
                let live = state.ir_context_read();
                let tuples = live.get_ir(&alias).expect("live alias-owned tuples");
                let instructions = tuples
                    .iter()
                    .map(|tuple| {
                        crate::ir::wire::tuple_to_op(tuple)
                            .expect("every canonical live tuple must decode")
                    })
                    .collect();
                CompiledIR {
                    file_id: physical.clone(),
                    version: live.file_version(&alias).expect("live alias-owned version"),
                    instructions,
                }
            };
            assert_eq!(
                response["result"]["file"], alias,
                "public presentation alias"
            );
            assert_eq!(
                response["result"]["v"], expected.version,
                "public/live version"
            );
            let expected_bytes = crate::ir::binary_wire::encode(&expected);
            let durable = if enabled {
                let guard = state.persistence_store_lock();
                let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
                let bytes = sqlite.baseline_binary(&physical).unwrap();
                if auto_save || fidelity == "edit" {
                    let actual_bytes = bytes.unwrap();
                    assert_eq!(actual_bytes, expected_bytes, "exact actual durable binary");
                    assert_eq!(
                        crate::ir::binary_wire::decode(&actual_bytes).unwrap(),
                        expected
                    );
                    let loaded = sqlite
                        .load_durable_context(&physical, None)
                        .unwrap()
                        .unwrap();
                    assert_eq!(
                        loaded.ir, expected,
                        "exact physical owner/instructions/version"
                    );
                    assert_eq!(
                        loaded.source_hash,
                        crate::cache::content_hash(source.as_bytes())
                    );
                    let snapshot = state.semantic_snapshot(&alias).unwrap();
                    assert_eq!(
                        serde_json::to_value(loaded.semantic_edges).unwrap(),
                        serde_json::to_value(snapshot.edges).unwrap()
                    );
                    expected.file_id = "<PHYSICAL_OWNER>".into();
                    json!({"owner":physical,"version":loaded.ir.version,
                        "source_hash":loaded.source_hash,"fidelity":format!("{:?}",loaded.fidelity),
                        "normalized_binary":crate::ir::binary_wire::encode(&expected)})
                } else {
                    assert!(
                        bytes.is_none(),
                        "auto-save off must not manufacture a checkpoint"
                    );
                    Value::Null
                }
            } else {
                Value::Null
            };
            responses.push(response_identity(
                response,
                &root.path().to_string_lossy(),
                &mut seen_breakers,
            ));
            durable_rows.push(normalize(durable, &root.path().to_string_lossy()));
        }
        let actual = json!({"enabled":enabled,"auto_save":auto_save,"fidelity":fidelity,
            "members":members,"requests":requests,"responses":responses,"durable":durable_rows});
        assert_eq!(
            &actual, fixture,
            "frozen original complete durable/public identity"
        );
        checked += 1;
    }
    assert_eq!(
        checked, 7,
        "all selected original semantic cases must execute"
    );
}
