use super::*;

fn shared_fixture() -> Fixture {
    let f = Fixture::new();
    let mut a = Fixture::declaration("DOC-001", "IMPLEMENTED");
    a["relationships"] = json!([{"kind":"DEPENDS_ON","target":"DOC-002"}]);
    let b = Fixture::declaration("DOC-002", "IMPLEMENTED");
    f.write("shared.md", &a);
    let mut text = std::fs::read_to_string(f.root.join("docs/shared.md")).unwrap();
    f.write("shared.md", &b);
    text.push_str(&std::fs::read_to_string(f.root.join("docs/shared.md")).unwrap());
    text.push_str(&"\nExact shared source: 日本語 and λ.\n".repeat(150));
    std::fs::write(f.root.join("docs/shared.md"), text).unwrap();
    std::fs::write(
        f.root.join("evidence.txt"),
        "Shared artifact λ.\n".repeat(100),
    )
    .unwrap();
    f
}

fn expand(mut payload: Value) -> Value {
    let sources = payload.as_object_mut().unwrap().remove("sources").unwrap();
    let resolve = |reference: &Value| {
        sources
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["source_ref"] == *reference)
            .unwrap()["text"]
            .clone()
    };
    for node in payload["documents"].as_array_mut().unwrap() {
        let reference = node.as_object_mut().unwrap().remove("source_ref").unwrap();
        node["source_text"] = resolve(&reference);
        for evidence in node["evidence"].as_array_mut().unwrap() {
            if let Some(reference) = evidence.as_object_mut().unwrap().remove("source_ref") {
                evidence["text"] = resolve(&reference);
            }
        }
    }
    payload.as_object_mut().unwrap().remove("response_format");
    payload["version"] = json!(1);
    payload
}

#[test]
fn negotiated_sources_reconstruct_legacy() {
    let f = shared_fixture();
    let state = state(Some(&f));
    let q = f.request("context", Some("DOC-001"));
    let args = serde_json::to_value(&q).unwrap();
    let (legacy, _) = call(&state, args.clone());
    let legacy = Fixture::decode(&serde_json::to_string(&legacy["result"]).unwrap());
    let mut args = args;
    args["responseFormat"] = json!("source_refs_v1");
    let (response, wire) = call(&state, args.clone());
    assert!(
        response.get("error").is_none(),
        "negotiated lossless context must be supported: {response}"
    );
    let packed = Fixture::decode(&serde_json::to_string(&response["result"]).unwrap());
    assert_eq!(packed["version"], 2);
    assert_eq!(packed["response_format"], "source_refs_v1");
    assert_eq!(packed["sources"].as_array().unwrap().len(), 2);
    assert_eq!(packed["documents"].as_array().unwrap().len(), 2);
    assert_eq!(packed["context"]["complete"], true);
    assert_eq!(expand(packed.clone()), legacy);
    let (repeat, repeat_wire) = call(&state, args.clone());
    assert_eq!(wire, repeat_wire);
    assert_eq!(response, repeat);
    let session = crate::pcg::documents::Session::new(f.profile.clone());
    let request = serde_json::from_value(args).unwrap();
    let result = session.query(&request).unwrap();
    let tokenizer =
        crate::tokenizer::create_tokenizer(crate::tokenizer::TokenizerKind::O200k).unwrap();
    assert_eq!(
        result.used_tokens,
        tokenizer.count_tokens(&result.serialized_result)
    );
    assert!(result.used_tokens < tokenizer.count_tokens(&serde_json::to_string(&legacy).unwrap()));
    let mut exact = request;
    exact.token_budget = result.used_tokens as u64;
    let exact_result = session.query(&exact).unwrap();
    let exact_payload = Fixture::decode(&exact_result.serialized_result);
    assert_eq!(exact_payload["result_state"], "ready");
    assert!(exact_result.used_tokens as u64 <= exact.token_budget);
    exact.token_budget = 1000;
    let blocked = Fixture::decode(&session.query(&exact).unwrap().serialized_result);
    assert_eq!(blocked["result_state"], "blocked");
    assert_eq!(blocked["documents"], json!([]));
    assert_eq!(blocked["sources"], json!([]));
    assert_eq!(blocked["context"]["complete"], false);
    assert_eq!(blocked["context"]["relationships"], json!([]));
    assert_eq!(blocked["historical_documents"], json!([]));
    assert!(
        packed["sources"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["path"].is_string() && s["sha256"].is_string())
    );
}

#[test]
fn source_format_negotiation_and_history_remain_explicit() {
    let f = shared_fixture();
    let state = state(Some(&f));
    for format in [json!("unknown"), Value::Null] {
        let mut args = serde_json::to_value(f.request("context", Some("DOC-001"))).unwrap();
        args["responseFormat"] = format;
        assert_eq!(call(&state, args).0["error"]["code"], -32602);
    }
    let history = serde_json::to_value(f.request("history", Some("DOC-001"))).unwrap();
    let legacy = call(&state, history.clone()).0;
    let payload = Fixture::decode(&serde_json::to_string(&legacy["result"]).unwrap());
    assert!(payload["documents"].as_array().unwrap().is_empty());
    assert!(
        !payload["historical_documents"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(payload.get("sources").is_none());
    let mut invalid = history;
    invalid["responseFormat"] = json!("source_refs_v1");
    assert_eq!(call(&state, invalid).0["error"]["code"], -32602);
}
