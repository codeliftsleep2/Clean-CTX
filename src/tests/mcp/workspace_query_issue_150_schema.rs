#[test]
fn issue_150_workspace_query_schema_declares_sparse_discovery_diagnostics() {
    let tools = crate::mcp::tools::tool_list();
    let workspace_query = tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool");
    let properties = workspace_query["outputSchema"]["properties"]
        .as_object()
        .expect("output properties");

    for removed in [
        "hydration_attempted",
        "discovery_provider",
        "discovery_status",
        "discovery_completed",
        "fallback_occurred",
        "fallback_reason",
        "candidates_discovered",
        "candidates_compiled",
        "project_coverage",
        "project_coverage_truncated",
    ] {
        assert!(
            !properties.contains_key(removed),
            "obsolete field: {removed}"
        );
    }
    let fields = properties["discovery"]["properties"]
        .as_object()
        .expect("discovery properties");
    for field in [
        "provider",
        "status",
        "fallback_reason",
        "discovered",
        "compiled",
        "projects",
        "projects_truncated",
    ] {
        assert!(fields.contains_key(field), "missing schema field: {field}");
    }
    assert!(!fields.contains_key("attempted") && !fields.contains_key("fallback"));
    assert_eq!(fields["projects"]["type"], "array");
    assert_eq!(
        fields["projects"]["items"]["properties"]["status"]["enum"],
        serde_json::json!(["searched", "search_failed", "skipped"])
    );
    let description = fields["fallback_reason"]["description"].as_str().unwrap();
    for meaning in [
        "authoritative coverage",
        "responded successfully",
        "does not by itself indicate a CBM failure",
    ] {
        assert!(description.contains(meaning), "missing meaning: {meaning}");
    }
    let coverage = &properties["coverage"]["properties"];
    assert_eq!(
        coverage["semantic_scope"]["enum"],
        serde_json::json!(["supported_producer_patterns"])
    );
    assert_eq!(
        coverage["semantic_omissions"]["items"]["enum"],
        serde_json::json!([
            "ngrx_object_string_actions",
            "ngrx_switch_action_type_handlers"
        ])
    );
}
