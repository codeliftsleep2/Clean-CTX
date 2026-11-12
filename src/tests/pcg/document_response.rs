use super::*;

fn node(path: &str, text: &str) -> Value {
    json!({"path":path,"source_sha256":identity::sha(text.as_bytes()),
        "source_text":text,"revision":"individually retained","evidence":[]})
}

#[test]
fn exact_identity_separates_paths_and_hashes_and_preserves_bytes() {
    let text = "Unicode λ 日本語\r\n\0end";
    let mut payload = json!({"documents":[node("a.md",text),node("b.md",text),node("a.md","changed")],
        "historical_documents":[]});
    assert_eq!(source_refs(&mut payload).unwrap(), 3);
    assert_ne!(
        payload["documents"][0]["source_ref"],
        payload["documents"][1]["source_ref"]
    );
    assert_ne!(
        payload["documents"][0]["source_ref"],
        payload["documents"][2]["source_ref"]
    );
    assert!(
        payload["sources"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["text"] == text)
    );
    assert_eq!(payload["historical_documents"], json!([]));
}

#[test]
fn repeated_identity_constructs_one_pool_entry_per_response() {
    for count in [1, 8, 64] {
        let mut payload = json!({"documents":vec![node("a.md","exact");count]});
        assert_eq!(source_refs(&mut payload).unwrap(), 1);
        assert_eq!(payload["documents"].as_array().unwrap().len(), count);
        assert_eq!(payload["sources"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn conflicting_identity_or_wrong_hash_refuses_projection() {
    let mut conflicting = node("a.md", "different");
    conflicting["source_sha256"] = identity::sha(b"exact").into();
    let mut payload = json!({"documents":[node("a.md","exact"),conflicting]});
    assert_eq!(
        source_refs(&mut payload),
        Err(Error("document_source_identity"))
    );
    let mut wrong = node("a.md", "text");
    wrong["source_sha256"] = identity::sha(b"other").into();
    assert_eq!(
        source_refs(&mut json!({"documents":[wrong]})),
        Err(Error("document_source_identity"))
    );
}
