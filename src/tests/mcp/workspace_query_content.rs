use super::render;
use serde_json::json;

fn payload(text: &str) -> serde_json::Value {
    let (header, json) = text.split_once('\n').unwrap();
    assert_eq!(
        header,
        "// WORKSPACE-QUERY v1; structuredContent remains authoritative"
    );
    serde_json::from_str(json).unwrap()
}

#[test]
fn edge_content_preserves_authoritative_facts_and_adds_render_occurrences() {
    let structured = json!({
        "edges": [{
            "relation": "Injects",
            "subject": { "domain": "angular", "entity_type": "Service", "name": "Alpha", "file": "a.ts" },
            "object": { "domain": "angular", "entity_type": "Service", "name": "Repository", "file": "a.ts" },
            "layer": "angular"
        }],
        "count": 1,
        "discovery": { "provider": "filesystem" }
    });
    let rendered = render(
        "reverse_edges",
        &json!({ "workspaceRoot": "C:/repo", "withinPath": "src" }),
        &structured,
        &["C:/shared".into()],
    );
    let value = payload(&rendered);

    assert_eq!(value["query"]["direction"], "incoming");
    assert_eq!(value["scope"]["workspace_root"], "C:/repo");
    assert_eq!(value["scope"]["within_path"], "src");
    assert_eq!(value["scope"]["additional_roots"][0], "C:/shared");
    assert_eq!(value["result"]["edges"][0]["occurrence"], 0);
    assert_eq!(value["result"]["edges"][0]["relation"], "Injects");
    assert_eq!(value["result"]["edges"][0]["subject"]["file"], "a.ts");
    assert_eq!(value["completeness"]["zero_result"], false);
    assert_eq!(value["completeness"]["discovery"], structured["discovery"]);
    assert_eq!(structured["edges"][0].get("occurrence"), None);
}

#[test]
fn transitive_and_zero_results_preserve_current_non_path_semantics() {
    let transitive = json!({
        "dependencies": [["angular", "Service", "Repository"]],
        "count": 1,
        "depth_used": 3
    });
    let value = payload(&render(
        "transitive_dependencies",
        &json!({ "domain": "angular", "entity_type": "Service", "name": "Alpha", "depth": 3 }),
        &transitive,
        &[],
    ));
    assert_eq!(value["result"]["dependencies"][0]["occurrence"], 0);
    assert_eq!(
        value["result"]["dependencies"][0]["value"],
        json!(["angular", "Service", "Repository"])
    );
    assert!(value["result"].get("paths").is_none());

    let zero = payload(&render(
        "reverse_edges",
        &json!({ "name": "Missing" }),
        &json!({ "edges": [], "count": 0 }),
        &[],
    ));
    assert_eq!(zero["completeness"]["zero_result"], true);
    assert_eq!(zero["result"]["edges"], json!([]));
}
