use super::*;
use crate::{config::CleanCtxConfig, pcg::document_fixture::Fixture};

#[path = "pcg_document_sources.rs"]
mod source_payloads;

fn state(f: Option<&Fixture>) -> McpState {
    let mut config = CleanCtxConfig::default();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    config.proxy.auto_start = false;
    config.cache.enabled = false;
    let mut state = McpState::new(config);
    state.pcg = crate::mcp::pcg_state::PcgState::from_path(None);
    state.pcg.documents = f.map(|f| crate::pcg::documents::Session::new(f.profile.clone()));
    state
}
fn call(state: &McpState, args: Value) -> (Value, String) {
    protocol::captured_responses().clear();
    protocol::begin_wire_capture();
    crate::mcp::router::dispatch(
        crate::protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: Some(json!(7)),
            method: "tools/call".into(),
            params: Some(json!({"name":"pcg_documents_v1","arguments":args})),
        },
        state,
    );
    (
        protocol::captured_responses().pop().unwrap(),
        protocol::take_captured_wire().pop().unwrap(),
    )
}

#[test]
fn registered_document_lifecycle_exact_wire_and_legacy_parity() {
    let f = Fixture::new();
    let disabled = state(None);
    let enabled = state(Some(&f));
    assert_eq!(
        crate::mcp::tools::tool_list_for_state(&disabled),
        crate::mcp::tools::tool_list()
    );
    let list = crate::mcp::tools::tool_list_for_state(&enabled);
    let d = list.last().unwrap();
    assert_eq!(d["name"], "pcg_documents_v1");
    assert_eq!(d["annotations"]["readOnlyHint"], false);
    assert_eq!(d["inputSchema"]["additionalProperties"], false);
    assert!(
        !d["inputSchema"]["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "tokenBudget")
    );
    let q = f.request("search", None);
    let args = serde_json::to_value(&q).unwrap();
    assert_eq!(call(&disabled, args.clone()).0["error"]["code"], -32601);
    assert_eq!(
        Fixture::decode(&serde_json::to_string(&call(&enabled, args.clone()).0["result"]).unwrap())
            ["documents"],
        json!([])
    );
    let mut declaration = Fixture::declaration("DOC-001", "INVESTIGATING");
    f.write("a.md", &declaration);
    let (result, wire) = call(&enabled, args.clone());
    assert!(result.get("error").is_none());
    assert!(result["result"].get("structuredContent").is_none());
    let direct = crate::pcg::documents::Session::new(f.profile.clone())
        .query(&q)
        .unwrap();
    assert_eq!(
        wire.trim(),
        format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":7,\"result\":{}}}",
            direct.serialized_result
        )
    );
    let tokenizer =
        crate::tokenizer::create_tokenizer(crate::tokenizer::TokenizerKind::O200k).unwrap();
    assert_eq!(
        direct.used_tokens,
        tokenizer.count_tokens(&direct.serialized_result)
    );
    let mut without_budget = args.clone();
    without_budget
        .as_object_mut()
        .unwrap()
        .remove("tokenBudget");
    assert_eq!(call(&enabled, without_budget).1, wire);
    declaration["status"] = "FROZEN".into();
    f.write("a.md", &declaration);
    let restarted = state(Some(&f));
    let q = f.request("history", Some("DOC-001"));
    let (result, _) = call(&restarted, serde_json::to_value(q).unwrap());
    let p: Value =
        serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(p["historical_documents"].as_array().unwrap().len(), 2);
    assert_eq!(
        p["historical_documents"][1]["declaration"]["status"],
        "FROZEN"
    );
    let mut bad = args.clone();
    bad["unknown"] = true.into();
    assert_eq!(call(&enabled, bad).0["error"]["code"], -32602);
    let mut bad = args;
    bad["tokenizer"] = "claude".into();
    assert_eq!(call(&enabled, bad).0["error"]["code"], -32602);
}

#[test]
fn invalid_document_diagnostics_do_not_return_stale_authority() {
    let f = Fixture::new();
    let state = state(Some(&f));
    let args = serde_json::to_value(f.request("search", None)).unwrap();
    let mut d = Fixture::declaration("DOC-001", "FROZEN");
    f.write("a.md", &d);
    call(&state, args.clone());
    d["status"] = "TYPO".into();
    f.write("a.md", &d);
    let (result, _) = call(&state, args);
    let p: Value =
        serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(p["result_state"], "blocked");
    assert_eq!(p["catalog_current"], false);
    assert_eq!(p["documents"], json!([]));
    assert!(
        p["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["code"] == "E_INVALID_STATUS")
    );
}

#[test]
fn invalid_document_also_blocks_same_workspace_legacy_pcg_authority() {
    let baseline = crate::pcg::mcp_fixture::Fixture::new();
    let mut f = Fixture::new();
    f.root = baseline.root.clone();
    std::fs::create_dir(f.root.join("docs")).unwrap();
    std::fs::write(f.root.join("evidence.txt"), b"Exact evidence.\n").unwrap();
    let mut profile: Value = serde_json::from_slice(&std::fs::read(&f.profile).unwrap()).unwrap();
    profile["workspace_root"] = serde_json::to_value(&f.root).unwrap();
    std::fs::write(&f.profile, serde_json::to_vec(&profile).unwrap()).unwrap();
    let mut d = Fixture::declaration("DOC-001", "FROZEN");
    f.write("a.md", &d);
    let mut enabled = state(Some(&f));
    enabled.pcg.session = Some(crate::pcg::session::Session::new(
        baseline.profile_path.clone(),
    ));
    let args = json!({"workspaceRoot":baseline.root,"corpus":"fixture","generation":baseline.generation,
        "mode":"constraints","subject":{"workspace":"fixture","file":"decision.md","operation":"edit","task":"modify","at":"2026-11-11T12:00:00Z"},
        "tokenizer":"o200k","includeHistory":false,"limits":{"records":4096,"edges":8192,"evidence_bytes":4194304,"token_budget":8192}});
    let invoke = |state: &McpState| {
        protocol::captured_responses().clear();
        crate::mcp::router::dispatch(
            crate::protocol::JsonRpcRequest {
                jsonrpc: "2.0".into(),
                id: Some(json!(3)),
                method: "tools/call".into(),
                params: Some(json!({"name":"pcg_query_v1","arguments":args})),
            },
            state,
        );
        protocol::captured_responses().pop().unwrap()
    };
    assert!(invoke(&enabled).get("result").is_some());
    d["status"] = "TYPO".into();
    f.write("a.md", &d);
    let blocked = invoke(&enabled);
    assert_eq!(blocked["error"]["code"], -32603);
    assert!(blocked.get("result").is_none());
    let q = f.request("history", Some("DOC-001"));
    let (response, _) = call(&enabled, serde_json::to_value(q).unwrap());
    let p: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(p["documents"], json!([]));
    assert_eq!(p["historical_documents"].as_array().unwrap().len(), 1);
    assert_eq!(p["catalog_current"], false);
}

#[test]
fn registered_context_discovers_transitive_constraints_before_authoring() {
    let f = Fixture::new();
    let state = state(Some(&f));
    let mut a = Fixture::declaration("DOC-081", "INVESTIGATING");
    let mut b = Fixture::declaration("DOC-068", "FROZEN");
    let c = Fixture::declaration("DOC-042", "FROZEN");
    a["relationships"] = json!([{"kind":"IMPLEMENTS","target":"DOC-068"}]);
    b["relationships"] = json!([{"kind":"DEPENDS_ON","target":"DOC-042"}]);
    f.write("a.md", &a);
    f.write("b.md", &b);
    f.write("c.md", &c);
    let mut q = f.request("context", Some("DOC-081"));
    q.max_nodes = Some(8);
    q.max_edges = Some(16);
    let (result, _) = call(&state, serde_json::to_value(q).unwrap());
    let p: Value =
        serde_json::from_str(result["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(p["context"]["complete"], true);
    assert_eq!(p["documents"].as_array().unwrap().len(), 3);
    assert!(
        p["context"]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .any(|path| path["node"] == "DOC-042" && path["steps"].as_array().unwrap().len() == 2)
    );
}
