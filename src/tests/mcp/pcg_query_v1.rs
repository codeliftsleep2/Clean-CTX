use super::*;
use crate::tokenizer::{TokenizerKind, create_tokenizer};
use crate::{
    config::CleanCtxConfig,
    mcp::{pcg_state::PcgState, tools},
    pcg::mcp_fixture::Fixture,
};
use std::{fs, path::Path};

fn state(path: Option<&Path>) -> McpState {
    let mut config = CleanCtxConfig::default();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    config.proxy.auto_start = false;
    config.cache.enabled = false;
    let mut state = McpState::new(config);
    state.pcg = PcgState::from_path(path.map(Path::to_path_buf));
    state
}
fn arguments(f: &Fixture) -> Value {
    json!({"workspaceRoot":f.root,"corpus":"fixture","generation":f.generation,
        "mode":"constraints","subject":{"workspace":"fixture","file":"decision.md","operation":"edit",
        "task":"modify","at":"2026-11-11T12:00:00Z"},"tokenizer":"o200k","includeHistory":false,
        "limits":{"records":4096,"edges":8192,"evidence_bytes":4194304,"token_budget":8192}})
}
fn call(state: &McpState, args: Value) -> (Value, String) {
    protocol::captured_responses().clear();
    protocol::begin_wire_capture();
    crate::mcp::router::dispatch(
        crate::protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(2)),
            method: "tools/call".into(),
            params: Some(json!({"name":"pcg_query_v1","arguments":args})),
        },
        state,
    );
    let value = protocol::captured_responses().pop().unwrap();
    let raw = protocol::take_captured_wire().pop().unwrap();
    (value, raw)
}
fn payload(response: &Value) -> Value {
    serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[test]
fn catalog_registry_and_disabled_legacy_parity() {
    let f = Fixture::new();
    let args = arguments(&f);
    let disabled = state(None);
    let enabled = state(Some(&f.profile_path));
    assert_eq!(tools::tool_list_for_state(&disabled), tools::tool_list());
    let enabled_tools = tools::tool_list_for_state(&enabled);
    assert_eq!(
        &enabled_tools[..enabled_tools.len() - 1],
        tools::tool_list().as_slice()
    );
    let tool = enabled_tools.last().unwrap();
    assert_eq!(tool["name"], "pcg_query_v1");
    assert_eq!(
        tool["annotations"],
        json!({"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false})
    );
    assert!(tool.get("outputSchema").is_none());
    assert_eq!(tool["inputSchema"]["additionalProperties"], false);
    for nested in ["subject", "limits"] {
        assert_eq!(
            tool["inputSchema"]["properties"][nested]["additionalProperties"],
            false
        );
    }
    assert!(
        crate::mcp::tool_handlers::registry::create_pcg_registry()
            .get("pcg_query_v1")
            .is_some()
    );
    assert!(
        crate::mcp::tool_handlers::registry::create_default_registry()
            .get("pcg_query_v1")
            .is_none()
    );
    assert_eq!(call(&disabled, args).0["error"]["code"], -32601);
    for state in [&disabled, &enabled] {
        protocol::captured_responses().clear();
        crate::mcp::router::dispatch(
            crate::protocol::JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(8)),
                method: "tools/call".into(),
                params: Some(json!({"name":"list_sessions","arguments":{}})),
            },
            state,
        );
        assert_eq!(
            protocol::captured_responses().pop().unwrap(),
            json!({"jsonrpc":"2.0","id":8,
            "result":{"content":[{"type":"text","text":"Persistence not enabled"}]}})
        );
        crate::mcp::router::dispatch(
            crate::protocol::JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(9)),
                method: "not_a_method".into(),
                params: None,
            },
            state,
        );
        assert_eq!(
            protocol::captured_responses().pop().unwrap(),
            json!({"jsonrpc":"2.0","id":9,
            "error":{"code":-32601,"message":"Method not found"}})
        );
    }
}

#[test]
fn strict_arguments_and_protocol_errors() {
    let f = Fixture::new();
    let args = arguments(&f);
    let state = state(Some(&f.profile_path));
    let mut cases = Vec::new();
    for field in [
        "workspaceRoot",
        "corpus",
        "generation",
        "mode",
        "subject",
        "tokenizer",
        "limits",
        "includeHistory",
    ] {
        let mut q = args.clone();
        q.as_object_mut().unwrap().remove(field);
        cases.push(q);
    }
    for (object, key, value) in [
        ("", "unexpected", json!(true)),
        ("subject", "tenant", json!("forged")),
        ("limits", "retry", json!(1)),
        ("subject", "file", Value::Null),
        ("limits", "records", json!(1.5)),
        ("limits", "token_budget", json!(0)),
        ("limits", "token_budget", json!(8193)),
        ("", "records", json!([])),
        ("", "generation", json!("bad")),
        ("subject", "at", json!("2026-99-99T00:00:00Z")),
    ] {
        let mut q = args.clone();
        if object.is_empty() {
            q[key] = value;
        } else {
            q[object][key] = value;
        }
        cases.push(q);
    }
    let mut q = args.clone();
    q["mode"] = json!("closure");
    cases.push(q);
    for q in cases {
        let response = call(&state, q).0;
        assert_eq!(response["error"]["code"], -32602, "{response}");
        assert!(response.get("result").is_none());
    }
    for name in ["claude", "llama3", "unknown"] {
        let mut q = args.clone();
        q["tokenizer"] = json!(name);
        assert_eq!(
            call(&state, q).0["error"]["message"],
            "unsupported_tokenizer"
        );
    }
    let mut q = args.clone();
    q["limits"]["token_budget"] = json!(1);
    assert_eq!(
        call(&state, q).0["error"]["message"],
        "token_budget_too_small"
    );
    let mut q = args;
    q["workspaceRoot"] = json!(f.dir.path());
    let response = call(&state, q).0;
    assert_eq!(
        response["error"],
        json!({"code":-32602,"message":"pcg_invalid_request"})
    );
}

#[test]
fn registered_dispatch_delivers_exact_metered_bytes() {
    let f = Fixture::new();
    let mut args = arguments(&f);
    let state = state(Some(&f.profile_path));
    let mut expected_probe = String::new();
    for kind in ["cl100k", "o200k"] {
        for mode in ["constraints", "closure", "impact", "provenance"] {
            args["tokenizer"] = json!(kind);
            args["mode"] = json!(mode);
            if mode == "constraints" {
                args.as_object_mut().unwrap().remove("records");
            } else {
                args["records"] = json!(["decision"]);
            }
            let (q, tokenizer, budget) = parse(&json!({"arguments":args})).unwrap();
            let expected = projection::project(&f.profile_path, &q, &tokenizer, budget).unwrap();
            let (response, wire) = call(&state, args.clone());
            let result = wire
                .strip_prefix("{\"jsonrpc\":\"2.0\",\"id\":2,\"result\":")
                .unwrap()
                .strip_suffix('}')
                .unwrap();
            assert_eq!(result.as_bytes(), expected.serialized_result.as_bytes());
            let engine = create_tokenizer(TokenizerKind::from_str_opt(kind).unwrap()).unwrap();
            assert_eq!(engine.count_tokens(result), expected.used_tokens);
            assert!(expected.used_tokens as u64 <= budget);
            assert_eq!(
                payload(&response)["result_state"],
                "complete_within_registered_scope"
            );
            assert_eq!(response["result"].as_object().unwrap().len(), 2);
            assert!(response["result"].get("structuredContent").is_none());
            if kind == "o200k" && mode == "constraints" {
                expected_probe = expected.serialized_result;
            }
        }
    }
    // Persist one tiny, source-bound fixture for the separately reported real-stdio probe.
    args["mode"] = json!("constraints");
    args.as_object_mut().unwrap().remove("records");
    fs::write(
        f.root.join(".clean-ctx.json"),
        serde_json::to_vec(&state.config).unwrap(),
    )
    .unwrap();
    let output = Path::new("target/pcg-175/m4a-verify");
    fs::create_dir_all(output).unwrap();
    fs::write(output.join("probe-input.json"),serde_json::to_vec_pretty(&json!({"profile":f.profile_path,
        "workspace":f.root,"arguments":args,"expected_result":expected_probe,"legacy_tools":tools::tool_list()})).unwrap()).unwrap();
    let _retained = f.dir.keep();
}

#[test]
fn states_fail_closed_and_leave_source_owners_unchanged() {
    let f = Fixture::new();
    let args = arguments(&f);
    let state = state(Some(&f.profile_path));
    let before = fs::read(f.database()).unwrap();
    let mut q = args.clone();
    q["subject"].as_object_mut().unwrap().remove("task");
    assert_eq!(payload(&call(&state, q).0)["result_state"], "incomplete");
    let mut q = args.clone();
    q["generation"] = json!("0".repeat(64));
    assert_eq!(payload(&call(&state, q).0)["result_state"], "unavailable");
    let mut q = args.clone();
    q["subject"]["entity"] = json!({"kind":"declaration","file":"decision.md",
        "source_sha256":"0".repeat(64),"start":0,"end":1,"name":"decl","declaration_kind":"class"});
    assert_eq!(payload(&call(&state, q).0)["result_state"], "incomplete");
    let mut q = args.clone();
    q["limits"]["token_budget"] = json!(600);
    // Select a control-sized budget below the complete fixture's independently measured size.
    let (request, kind, limit) = parse(&json!({"arguments":args})).unwrap();
    let full = projection::project(&f.profile_path, &request, &kind, limit).unwrap();
    let mut blocked = None;
    for limit in (300..full.used_tokens).step_by(50) {
        q["limits"]["token_budget"] = json!(limit);
        let response = call(&state, q.clone()).0;
        if response.get("result").is_some() {
            let p = payload(&response);
            if p["result_state"] == "blocked" {
                blocked = Some(p);
                break;
            }
        }
    }
    let p = blocked.expect("a bounded required-overflow control must fit");
    assert_eq!(p["reasons"][0]["code"], "required_budget_exceeded");
    assert!(p["required_records"].as_array().unwrap().is_empty());
    fs::write(f.root.join("decision.md"), "stale source").unwrap();
    let p = payload(&call(&state, args).0);
    assert_eq!(p["result_state"], "blocked");
    for key in [
        "required_records",
        "relationships",
        "citations",
        "support",
        "history",
    ] {
        assert!(p[key].as_array().unwrap().is_empty());
    }
    assert_eq!(before, fs::read(f.database()).unwrap());
    assert!(state.source_cache.lock().unwrap().is_empty());
    assert!(state.llm_text_cache.lock().unwrap().is_empty());
    assert!(state.persistence_store.lock().unwrap().is_none());
}

#[test]
fn profile_is_pinned_and_unavailable_database_stays_discoverable() {
    let mut f = Fixture::new();
    let args = arguments(&f);
    let pinned = state(Some(&f.profile_path));
    f.profile["owner_label"] = json!("replacement");
    f.write_profile();
    assert_eq!(
        call(&pinned, args.clone()).0["error"],
        json!({"code":-32603,"message":"pcg_control_unavailable"})
    );
    let f = Fixture::new();
    let args = arguments(&f);
    let configured = state(Some(&f.profile_path));
    fs::remove_file(f.database()).unwrap();
    assert!(
        tools::tool_list_for_state(&configured)
            .iter()
            .any(|v| v["name"] == "pcg_query_v1")
    );
    let p = payload(&call(&configured, args).0);
    assert_eq!(p["result_state"], "unavailable");
    assert!(p["citations"].as_array().unwrap().is_empty());
}
