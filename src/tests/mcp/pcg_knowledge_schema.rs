use super::*;
use std::collections::BTreeSet;

#[test]
fn knowledge_descriptor_exposes_flat_typed_request_contract() {
    let descriptor = knowledge_descriptor();
    let schema = &descriptor["inputSchema"];
    assert_eq!(descriptor["name"], "pcg_knowledge_v1");
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["required"], json!(["workspaceRoot", "mode"]));
    // Descriptor interoperability requirement, not a simulation of an opaque host.
    for keyword in ["oneOf", "anyOf", "allOf", "if", "then", "else"] {
        assert!(
            schema.get(keyword).is_none(),
            "shared request fields must be exposed by one flat root object: {keyword}"
        );
    }
    let fields = [
        "workspaceRoot",
        "mode",
        "id",
        "text",
        "track",
        "kind",
        "status",
        "role",
        "label",
        "edgeKind",
        "direction",
        "path",
        "sourceSha256",
        "maxNodes",
        "maxEdges",
        "references",
        "revision",
        "historical",
    ];
    let properties = schema["properties"].as_object().unwrap();
    assert_eq!(
        properties
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        fields.into_iter().collect::<BTreeSet<_>>()
    );
    for field in fields {
        let expected_type = if field == "historical" {
            "boolean"
        } else if field == "references" {
            "array"
        } else if ["maxNodes", "maxEdges"].contains(&field) {
            "integer"
        } else {
            "string"
        };
        assert_eq!(properties[field]["type"], expected_type, "{field}");
        assert!(
            properties[field]["description"]
                .as_str()
                .is_some_and(|description| !description.trim().is_empty()),
            "generic usage must describe {field} without conditional schema projection"
        );
    }
    assert_eq!(
        properties["mode"]["enum"],
        json!(["search", "get", "related", "context", "source", "acquire"])
    );
    for field in ["id", "text", "track", "label", "path", "workspaceRoot"] {
        assert_eq!(properties[field]["minLength"], 1, "{field}");
    }
    for field in ["maxNodes", "maxEdges"] {
        assert_eq!(properties[field]["minimum"], 1, "{field}");
    }
    assert_eq!(properties["sourceSha256"]["pattern"], "^[0-9a-f]{64}$");
    assert_eq!(
        properties["direction"]["enum"],
        json!(["incoming", "outgoing", "both"])
    );
    assert_eq!(
        properties["kind"]["enum"],
        json!(crate::pcg::documents::knowledge::KINDS)
    );
    assert_eq!(
        properties["status"]["enum"],
        json!(crate::pcg::documents::knowledge::STATUSES)
    );
    assert_eq!(
        properties["role"]["enum"],
        json!(crate::pcg::documents::knowledge::ROLES)
    );
    assert_eq!(
        properties["edgeKind"]["enum"],
        json!(crate::pcg::documents::knowledge::RELATIONS)
    );
    for phrase in ["search", "get", "related", "context", "source", "Rust"] {
        assert!(
            schema["description"].as_str().unwrap().contains(phrase),
            "{phrase}"
        );
    }
    for (field, phrases) in [
        (
            "id",
            vec!["required", "get", "related", "context", "source", "search"],
        ),
        ("text", vec!["search", "FTS5"]),
        ("track", vec!["search"]),
        ("kind", vec!["search"]),
        ("status", vec!["search"]),
        ("role", vec!["search", "get"]),
        ("label", vec!["role", "search", "get"]),
        ("edgeKind", vec!["related"]),
        ("direction", vec!["related", "both"]),
        ("path", vec!["source"]),
        ("sourceSha256", vec!["source", "refus"]),
        ("maxNodes", vec!["context"]),
        ("maxEdges", vec!["context"]),
    ] {
        let description = properties[field]["description"].as_str().unwrap();
        for phrase in phrases {
            assert!(description.contains(phrase), "{field}: missing {phrase}");
        }
    }
}

fn fixture() -> Fixture {
    let f = Fixture::new();
    let source = r#"<!-- pcg:A:begin -->
Pipeline ordering.
<!-- pcg:A:end -->
<!-- pcg:Q:begin -->
Historical attribution is not execution.
<!-- pcg:Q:end -->
```architecture-graph
node: DOC-001
kind: INVARIANT
status: IMPLEMENTED
title: Pipeline order
track: FOUNDATION
artifact: docs/a.md
depends_on:
  - DOC-002
knowledge_scope: invariant
knowledge_content:
  - statement self#A
  - qualification:standing self#Q
```
```architecture-graph
node: DOC-002
kind: AUTHORITY
status: IMPLEMENTED
title: Prerequisite
track: FOUNDATION
artifact: docs/a.md
knowledge_scope: authority
knowledge_content:
  - statement self#Q
```
"#;
    std::fs::write(f.root.join("docs/a.md"), source).unwrap();
    f
}

fn payload(state: &McpState, args: Value) -> Value {
    let response = knowledge_call(state, args);
    assert!(response.get("error").is_none(), "{response}");
    let result: Value =
        serde_json::from_str(response["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
    assert_eq!(result["result_state"], "ready", "{result}");
    assert_eq!(result["catalog_current"], true);
    assert_eq!(result["complete"], true);
    result
}

#[test]
fn registered_knowledge_request_validation_preserves_modes_and_refusals() {
    let f = fixture();
    let enabled = state(Some(&f));
    let get = json!({"workspaceRoot":f.root,"mode":"get","id":"DOC-001"});
    // Unfiltered search is valid; no text/track requirement may be invented.
    let all = payload(&enabled, json!({"workspaceRoot":f.root,"mode":"search"}));
    assert_eq!(all["units"].as_array().unwrap().len(), 2);
    let search = payload(
        &enabled,
        json!({"workspaceRoot":f.root,"mode":"search","text":"Pipeline"}),
    );
    assert_eq!(search["units"].as_array().unwrap().len(), 1);
    let unit = payload(&enabled, search["units"][0]["get_request"].clone());
    let acquired = payload(&enabled, unit["units"][0]["acquire_request"].clone());
    assert_eq!(
        acquired["acquisitions"][0]["components"][0]["text"],
        "Pipeline ordering.\n"
    );
    let selected = payload(
        &enabled,
        json!({"workspaceRoot":f.root,"mode":"get","id":"DOC-001","role":"qualification","label":"standing"}),
    );
    assert_eq!(selected["units"][0]["unit_complete"], false);
    let related = payload(
        &enabled,
        json!({"workspaceRoot":f.root,"mode":"related","id":"DOC-001","edgeKind":"DEPENDS_ON","direction":"outgoing"}),
    );
    assert_eq!(related["relationships"][0]["source"], "DOC-001");
    assert_eq!(related["relationships"][0]["target"], "DOC-002");
    let context = payload(
        &enabled,
        json!({"workspaceRoot":f.root,"mode":"context","id":"DOC-001","maxNodes":2,"maxEdges":1}),
    );
    assert_eq!(context["units"].as_array().unwrap().len(), 2);
    assert_eq!(context["relationships"].as_array().unwrap().len(), 1);
    let source_request = unit["units"][0]["source_request"].clone();
    let source = payload(&enabled, source_request.clone());
    assert_eq!(
        source["sources"][0]["text"],
        std::fs::read_to_string(f.root.join("docs/a.md")).unwrap()
    );

    for field in ["workspaceRoot", "mode", "id"] {
        let mut invalid = get.clone();
        invalid.as_object_mut().unwrap().remove(field);
        assert_eq!(
            knowledge_call(&enabled, invalid)["error"]["code"],
            -32602,
            "{field}"
        );
    }
    for field in ["workspaceRoot", "mode", "id", "role"] {
        let mut invalid = get.clone();
        invalid[field] = Value::Null;
        assert_eq!(
            knowledge_call(&enabled, invalid)["error"]["code"],
            -32602,
            "null {field}"
        );
    }
    for (field, value) in [
        ("text", json!("Pipeline")),
        ("track", json!("FOUNDATION")),
        ("kind", json!("INVARIANT")),
        ("status", json!("IMPLEMENTED")),
        ("edgeKind", json!("DEPENDS_ON")),
        ("direction", json!("incoming")),
        ("path", json!("docs/a.md")),
        ("sourceSha256", json!("0".repeat(64))),
        ("maxNodes", json!(1)),
        ("maxEdges", json!(1)),
        ("query", json!("Pipeline")),
        ("tokenBudget", json!(1)),
        ("mode", json!("history")),
        ("id", json!("")),
        ("role", json!("unknown")),
        ("label", json!("standing")),
    ] {
        let mut invalid = get.clone();
        invalid[field] = value;
        assert_eq!(
            knowledge_call(&enabled, invalid)["error"]["code"],
            -32602,
            "{field}"
        );
    }
    for patch in [
        json!({"mode":"search","id":"DOC-001"}),
        json!({"mode":"search","text":" "}),
        json!({"mode":"search","kind":"UNKNOWN"}),
        json!({"mode":"search","status":"UNKNOWN"}),
        json!({"mode":"related","id":"DOC-001","direction":"sideways"}),
        json!({"mode":"related","id":"DOC-001","edgeKind":"UNKNOWN"}),
        json!({"mode":"context","id":"DOC-001","role":"statement"}),
        json!({"mode":"context","id":"DOC-001","maxNodes":0}),
        json!({"mode":"context","id":"DOC-001","maxNodes":129}),
        json!({"mode":"context","id":"DOC-001","maxEdges":0}),
        json!({"mode":"context","id":"DOC-001","maxEdges":513}),
        json!({"mode":"context","id":"DOC-001","maxNodes":-1}),
        json!({"mode":"source","id":"DOC-001","sourceSha256":"invalid"}),
    ] {
        let mut invalid = patch;
        invalid["workspaceRoot"] = json!(f.root);
        assert_eq!(
            knowledge_call(&enabled, invalid.clone())["error"]["code"],
            -32602,
            "{invalid}"
        );
    }
    let mut stale = source_request;
    stale["sourceSha256"] = json!("0".repeat(64));
    assert_eq!(
        knowledge_call(&enabled, stale)["error"]["message"],
        "pcg/source_revision_changed"
    );
}
