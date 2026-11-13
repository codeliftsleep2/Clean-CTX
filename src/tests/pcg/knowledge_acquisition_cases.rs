use super::*;

fn acquisition(f: &Fixture, refs: Vec<Value>) -> Value {
    let q: knowledge::Request = serde_json::from_value(json!({
        "workspaceRoot":f.root,"mode":"acquire","references":refs
    }))
    .unwrap();
    query(f, &q)
}

#[test]
fn explicit_acquisition_preserves_exact_selection_qualifications_and_batch_failures() {
    let f = fixture();
    let graph = query(&f, &request(&f, "context", Some("DOC-001")));
    let a = graph["units"][0]["components"][0]["reference"].clone();
    let b = graph["units"][1]["components"][0]["reference"].clone();
    let mut invalid = a.clone();
    invalid["end"] = json!(999999);
    let batch = acquisition(&f, vec![a.clone(), b.clone(), a.clone(), invalid]);
    assert_eq!(batch["result_state"], "incomplete");
    assert_eq!(batch["complete"], false);
    let results = batch["acquisitions"].as_array().unwrap();
    assert_eq!(results.len(), 4);
    assert_eq!(results[0]["reference"], a);
    assert_eq!(results[1]["reference"], b);
    assert_eq!(results[2]["duplicate_of"], 0);
    assert_eq!(
        results[0]["components"][0]["text"],
        "PassPipeline has an explicit order.\n"
    );
    assert_eq!(
        results[0]["components"][1]["text"],
        "Historical validation is not current certification.\n"
    );
    assert_eq!(results[0]["components"][1]["selected_by_reference"], false);
    assert_eq!(results[1]["components"].as_array().unwrap().len(), 1);
    assert_eq!(results[3]["result_state"], "refused");
    assert_eq!(results[3]["components"], json!([]));
    assert_eq!(
        results[3]["diagnostics"][0]["code"],
        "source_reference_invalid"
    );
    assert_eq!(batch["units"], json!([]));
    assert_eq!(batch["relationships"], json!([]));
    assert_eq!(batch["sources"], json!([]));
}

#[test]
fn explicit_acquisition_refuses_reference_forgery_and_current_revision_drift() {
    let f = fixture();
    let initial = query(&f, &request(&f, "get", Some("DOC-001")));
    let reference = initial["units"][0]["components"][0]["reference"].clone();
    let mut full_history = initial["units"][0]["source_request"].clone();
    full_history["historical"] = true.into();
    for (field, value) in [
        ("ordinal", json!(999)),
        ("role", json!("rationale")),
        ("label", json!("forged")),
        ("path", json!("../outside")),
        ("sourceSha256", json!("0".repeat(64))),
        ("start", json!(0)),
        ("contentSha256", json!("0".repeat(64))),
        ("selector", json!("QUAL")),
        ("revision", json!("0".repeat(64))),
        ("id", json!("DOC-002")),
    ] {
        let mut invalid = reference.clone();
        invalid[field] = value;
        let result = acquisition(&f, vec![invalid]);
        assert_eq!(result["complete"], false, "{field}");
        assert_eq!(
            result["acquisitions"][0]["components"],
            json!([]),
            "{field}"
        );
    }
    std::fs::write(
        f.root.join("docs/shared.md"),
        source().replace(
            "PassPipeline has an explicit order.",
            "Changed source authority.",
        ),
    )
    .unwrap();
    let stale = acquisition(&f, vec![reference.clone()]);
    assert_eq!(
        stale["acquisitions"][0]["diagnostics"][0]["code"],
        "source_revision_changed"
    );
    let mut historical = reference;
    historical["historical"] = true.into();
    let old = acquisition(&f, vec![historical]);
    assert_eq!(old["complete"], true);
    assert_eq!(old["acquisitions"][0]["freshness"], "historical_snapshot");
    assert_eq!(
        old["acquisitions"][0]["components"][0]["text"],
        "PassPipeline has an explicit order.\n"
    );
    assert_eq!(
        old["acquisitions"][0]["components"][1]["text"],
        "Historical validation is not current certification.\n"
    );
    let historical_source: knowledge::Request = serde_json::from_value(full_history).unwrap();
    let full = query(&f, &historical_source);
    assert_eq!(full["freshness"], "historical_snapshot");
    assert_eq!(full["sources"][0]["text"], source());
    let broken = source().replace("<!-- pcg:PIPE-A:end -->", "");
    std::fs::write(f.root.join("docs/shared.md"), broken).unwrap();
    let mut historical_again = initial["units"][0]["components"][0]["reference"].clone();
    historical_again["historical"] = true.into();
    let retained = acquisition(&f, vec![historical_again]);
    assert_eq!(retained["catalog_current"], false);
    assert_eq!(retained["complete"], true);
    assert_eq!(
        retained["acquisitions"][0]["freshness"],
        "historical_snapshot"
    );
    let current = query(&f, &request(&f, "get", Some("DOC-001")));
    assert_eq!(current["result_state"], "blocked");
    assert_eq!(current["units"], json!([]));
}

#[test]
fn explicit_acquisition_shared_authority_work_is_operation_local_and_bounded() {
    let f = fixture();
    let unit = query(&f, &request(&f, "get", Some("DOC-001")));
    let reference = unit["units"][0]["components"][0]["reference"].clone();
    for count in [1, 8, 32] {
        let result = acquisition(&f, vec![reference.clone(); count]);
        assert_eq!(result["acquisitions"].as_array().unwrap().len(), count);
        assert_eq!(result["complete"], true);
        assert_eq!(
            result["acquisition_work"],
            json!({"revision_sets":1,"sources":1,"components":2,"intervals":2})
        );
    }
    let empty: knowledge::Request =
        serde_json::from_value(json!({"workspaceRoot":f.root,"mode":"acquire","references":[]}))
            .unwrap();
    assert_eq!(
        Session::new(f.profile.clone())
            .knowledge(&empty)
            .unwrap_err()
            .0,
        "invalid_request"
    );
    let mut malformed = reference;
    malformed["query"] = "alias not supported".into();
    assert!(
        serde_json::from_value::<knowledge::Request>(
            json!({"workspaceRoot":f.root,"mode":"acquire","references":[malformed]})
        )
        .is_err()
    );
}

#[test]
fn explicit_acquisition_shared_source_and_qualification_intervals_scale_once() {
    for count in [2, 8, 32] {
        let f = Fixture::new();
        let mut text =
            String::from("<!-- pcg:Q:begin -->\nShared qualification.\n<!-- pcg:Q:end -->\n");
        for index in 1..=count {
            text.push_str(&format!("<!-- pcg:P{index}:begin -->\nProposition {index}.\n<!-- pcg:P{index}:end -->\n```architecture-graph\nnode: DOC-{index:03}\nkind: INVARIANT\nstatus: IMPLEMENTED\ntitle: P{index}\ntrack: FOUNDATION\nartifact: docs/growth.md\nknowledge_scope: invariant\nknowledge_content:\n  - statement self#P{index}\n  - qualification self#Q\n```\n"));
        }
        std::fs::write(f.root.join("docs/growth.md"), text).unwrap();
        let graph = query(&f, &request(&f, "search", None));
        let refs = graph["units"]
            .as_array()
            .unwrap()
            .iter()
            .map(|unit| unit["components"][0]["reference"].clone())
            .collect();
        let acquired = acquisition(&f, refs);
        assert_eq!(acquired["complete"], true);
        assert_eq!(acquired["acquisitions"].as_array().unwrap().len(), count);
        assert_eq!(
            acquired["acquisition_work"],
            json!({"revision_sets":count,"sources":1,"components":count*2,"intervals":count+1})
        );
    }
}
