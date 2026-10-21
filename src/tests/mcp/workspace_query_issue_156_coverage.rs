//! Agent MaxHeadRoom: unresolved EF evidence must preserve optional schema fields.
use super::{expect, fixture, state, target};

#[test]
#[serial_test::serial]
fn issue_156_unresolved_coverage_preserves_optional_requirements() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path(), "builder.Entity<Missing>();");
    let result = target(&state(&[]), root.path());
    expect(&result, &[]);
    let coverage = &result["coverage"];
    assert_eq!(coverage["dependency_proof"]["status"], "unproven");
    assert_eq!(coverage["status"], "semantic_dependencies_unresolved");

    let tools = crate::mcp::tools::tool_list();
    let properties = &tools
        .iter()
        .find(|tool| tool["name"] == "workspace_query")
        .unwrap()["outputSchema"]["properties"]["coverage"]["properties"];
    assert_eq!(properties["evidence_requirements"]["type"], "array");
    assert!(
        coverage.get("evidence_requirements").is_none(),
        "unresolved EF evidence inserted an absent optional field: {result:#}"
    );
}
