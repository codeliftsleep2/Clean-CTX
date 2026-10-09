// RED regressions for neutral C# Struct/Record written-base-type facts.
//
// Production path:
// source -> C# captures -> CSharpSemanticProjection -> WorkspaceIndex -> workspace_query.

use super::*;

fn publish(dir: &tempfile::TempDir, state: &crate::mcp::McpState, file: &str, source: &str) {
    std::fs::write(dir.path().join(file), source).unwrap();
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": file,
                "fidelity": "high",
                "workspaceRoot": dir.path().to_string_lossy()
            }
        }),
        state,
    );
    let response = pop_response();
    assert!(
        response.get("result").is_some(),
        "publication must succeed for {file}: {response:?}"
    );
}

fn edges(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    direction: &str,
    entity_type: &str,
    name: &str,
) -> Vec<serde_json::Value> {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({
            "arguments": {
                "type": direction,
                "domain": "builtin",
                "entity_type": entity_type,
                "name": name,
                "workspaceRoot": dir.path().to_string_lossy()
            }
        }),
        state,
    );
    let response = pop_response();
    let result = response["result"].as_object().unwrap_or_else(|| {
        panic!("{direction} must succeed for {entity_type}/{name}: {response:?}")
    });
    assert_valid_mcp_envelope(result);
    result["structuredContent"]["edges"]
        .as_array()
        .cloned()
        .expect("structured edge array")
}

fn matching_edges(
    edges: &[serde_json::Value],
    subject_type: &str,
    subject_name: &str,
    object_name: &str,
) -> usize {
    edges
        .iter()
        .filter(|edge| {
            edge["relation"] == "HasBaseType"
                && edge["subject"]["domain"] == "builtin"
                && edge["subject"]["entity_type"] == subject_type
                && edge["subject"]["name"] == subject_name
                && edge["object"]["domain"] == "builtin"
                && edge["object"]["entity_type"] == "TypeRef"
                && edge["object"]["name"] == object_name
                && edge["layer"] == "builtin"
                && edge["subject"]["file"].as_str().is_some()
        })
        .count()
}

fn assert_forward_and_reverse(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    subject_type: &str,
    subject_name: &str,
    object_name: &str,
) {
    let forward = edges(dir, state, "forward_edges", subject_type, subject_name);
    assert_eq!(
        matching_edges(&forward, subject_type, subject_name, object_name),
        1,
        "forward query must expose exactly one neutral base-type fact: {forward:?}"
    );
    let reverse = edges(dir, state, "reverse_edges", "TypeRef", object_name);
    assert_eq!(
        matching_edges(&reverse, subject_type, subject_name, object_name),
        1,
        "reverse query must expose exactly one neutral base-type fact: {reverse:?}"
    );
}

fn assert_no_typed_or_class_owner(edges: &[serde_json::Value], owner_name: &str) {
    assert!(
        edges.iter().all(|edge| {
            !(matches!(edge["relation"].as_str(), Some("Extends" | "Implements"))
                || (edge["relation"] == "HasBaseType"
                    && edge["subject"]["entity_type"] == "Class"
                    && edge["subject"]["name"] == owner_name))
        }),
        "neutral struct/record facts must not fabricate typed inheritance or Class identity: {edges:?}"
    );
}

#[test]
fn csharp_struct_base_types_are_queryable_without_class_identity() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Structs.cs",
        "namespace P { public interface IFoo {} public struct Value : IFoo {} }",
    );

    assert_forward_and_reverse(&dir, &state, "Struct", "Value", "IFoo");
    assert_no_typed_or_class_owner(
        &edges(&dir, &state, "forward_edges", "Struct", "Value"),
        "Value",
    );
    assert_eq!(
        matching_edges(
            &edges(&dir, &state, "forward_edges", "Class", "Value"),
            "Class",
            "Value",
            "IFoo",
        ),
        0,
        "a struct base list must never project through Class identity"
    );
}

#[test]
fn csharp_record_forms_preserve_every_structured_written_type() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Records.cs",
        r#"
namespace P;
public interface IFoo {}
public interface IMap<TLeft, TRight> {}
public record BaseRecord;
public record Derived
    : BaseRecord,
      IMap<string,int>;
public record struct Packet
    : IFoo;
"#,
    );

    for written in ["BaseRecord", "IMap<string,int>"] {
        assert_forward_and_reverse(&dir, &state, "Record", "Derived", written);
    }
    assert_forward_and_reverse(&dir, &state, "Record", "Packet", "IFoo");
    for owner in ["Derived", "Packet"] {
        assert_no_typed_or_class_owner(
            &edges(&dir, &state, "forward_edges", "Record", owner),
            owner,
        );
    }
}

#[test]
fn csharp_nested_base_types_bind_to_the_nearest_owner() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Nested.cs",
        r#"
public interface IOuter {}
public interface IInner {}
public struct Outer : IOuter
{
    public record Inner : IInner;
}
"#,
    );

    assert_forward_and_reverse(&dir, &state, "Struct", "Outer", "IOuter");
    assert_forward_and_reverse(&dir, &state, "Record", "Inner", "IInner");
    assert_eq!(
        matching_edges(
            &edges(&dir, &state, "forward_edges", "Struct", "Outer"),
            "Struct",
            "Outer",
            "IInner",
        ),
        0,
        "a nested record base type must not attach to its enclosing struct"
    );
}

#[test]
fn csharp_struct_base_type_republication_retracts_and_deduplicates() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(&dir, &state, "Value.cs", "public struct Value : IOld {}");
    assert_forward_and_reverse(&dir, &state, "Struct", "Value", "IOld");

    publish(&dir, &state, "Value.cs", "public struct Value : INew {}");
    publish(&dir, &state, "Value.cs", "public struct Value : INew {}");

    let current = edges(&dir, &state, "forward_edges", "Struct", "Value");
    assert_eq!(matching_edges(&current, "Struct", "Value", "IOld"), 0);
    assert_eq!(
        matching_edges(&current, "Struct", "Value", "INew"),
        1,
        "repeated publication must retain exactly one current fact: {current:?}"
    );
    assert_eq!(
        matching_edges(
            &edges(&dir, &state, "reverse_edges", "TypeRef", "IOld"),
            "Struct",
            "Value",
            "IOld",
        ),
        0,
        "replacement must retract the old reverse edge"
    );
}

#[test]
fn csharp_struct_record_projection_uses_the_v0_9_1_producer_generation() {
    assert_eq!(
        crate::ir::semantic_projection::CSHARP_SEMANTIC_PROJECTION_GENERATION,
        2,
        "this behavior change must increment the existing CSharpSemanticProjection generation"
    );
}
