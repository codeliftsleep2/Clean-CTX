// Production-path structural inheritance regressions.
//
// Each case crosses the real boundary:
// source -> canonical CoreOp -> generic SemanticEdge -> WorkspaceIndex ->
// registered workspace_query dispatch.

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

fn has_edge(
    edges: &[serde_json::Value],
    relation: &str,
    subject_type: &str,
    subject_name: &str,
    object_type: &str,
    object_name: &str,
) -> bool {
    edges.iter().any(|edge| {
        edge["relation"] == relation
            && edge["subject"]["domain"] == "builtin"
            && edge["subject"]["entity_type"] == subject_type
            && edge["subject"]["name"] == subject_name
            && edge["object"]["domain"] == "builtin"
            && edge["object"]["entity_type"] == object_type
            && edge["object"]["name"] == object_name
            && edge["layer"] == "builtin"
            && edge["subject"]["file"].as_str().is_some()
    })
}

fn assert_forward_and_reverse(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    relation: &str,
    subject_type: &str,
    subject_name: &str,
    object_type: &str,
    object_name: &str,
) {
    let forward = edges(dir, state, "forward_edges", subject_type, subject_name);
    assert!(
        has_edge(
            &forward,
            relation,
            subject_type,
            subject_name,
            object_type,
            object_name
        ),
        "forward query omitted {subject_name} --{relation}--> {object_name}: {forward:?}"
    );
    let reverse = edges(dir, state, "reverse_edges", object_type, object_name);
    assert!(
        has_edge(
            &reverse,
            relation,
            subject_type,
            subject_name,
            object_type,
            object_name
        ),
        "reverse query omitted {subject_name} --{relation}--> {object_name}: {reverse:?}"
    );
}

#[cfg(feature = "typescript")]
#[test]
fn typescript_inheritance_survives_to_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "inheritance.ts",
        r#"
export class Base {}
export interface Root {}
export interface ChildContract extends Root {}
export class Child extends Base implements ChildContract {}
"#,
    );

    assert_forward_and_reverse(&dir, &state, "Extends", "Class", "Child", "Class", "Base");
    assert_forward_and_reverse(
        &dir,
        &state,
        "Implements",
        "Class",
        "Child",
        "Interface",
        "ChildContract",
    );
    assert_forward_and_reverse(
        &dir,
        &state,
        "Extends",
        "Interface",
        "ChildContract",
        "Interface",
        "Root",
    );
}

#[cfg(feature = "typescript")]
#[test]
fn javascript_class_extends_survives_to_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "inheritance.js",
        "class Base {}\nclass Child extends Base {}\n",
    );
    assert_forward_and_reverse(&dir, &state, "Extends", "Class", "Child", "Class", "Base");
}

#[cfg(feature = "java")]
#[test]
fn java_inheritance_survives_to_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Inheritance.java",
        r#"
class Base {}
interface Root {}
interface ChildContract extends Root {}
class Child extends Base implements ChildContract {}
"#,
    );
    assert_forward_and_reverse(&dir, &state, "Extends", "Class", "Child", "Class", "Base");
    assert_forward_and_reverse(
        &dir,
        &state,
        "Implements",
        "Class",
        "Child",
        "Interface",
        "ChildContract",
    );
    assert_forward_and_reverse(
        &dir,
        &state,
        "Extends",
        "Interface",
        "ChildContract",
        "Interface",
        "Root",
    );
}

#[cfg(feature = "csharp")]
#[test]
fn csharp_inheritance_and_interface_only_base_list_survive_to_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Inheritance.cs",
        r#"
public class Base {}
public interface IRoot {}
public interface IChildContract : IRoot {}
public class Child : Base, IChildContract {}
public class InterfaceOnly : IChildContract {}
public class Unresolved : ExternalContract {}
namespace ClassNamespace { public class Shared {} }
namespace InterfaceNamespace { public interface Shared {} }
public class Ambiguous : Shared {}
"#,
    );
    assert_forward_and_reverse(&dir, &state, "Extends", "Class", "Child", "Class", "Base");
    for subject in ["Child", "InterfaceOnly"] {
        assert_forward_and_reverse(
            &dir,
            &state,
            "Implements",
            "Class",
            subject,
            "Interface",
            "IChildContract",
        );
    }
    assert_forward_and_reverse(
        &dir,
        &state,
        "Extends",
        "Interface",
        "IChildContract",
        "Interface",
        "IRoot",
    );
    assert!(
        edges(&dir, &state, "forward_edges", "Class", "Unresolved").is_empty(),
        "an unresolved first C# base-list target must not become a false Extends or Implements edge"
    );
    assert!(
        edges(&dir, &state, "forward_edges", "Class", "Ambiguous").is_empty(),
        "a class/interface name collision must remain BaseTypeRef and produce no workspace edge"
    );
}

#[cfg(feature = "rust")]
#[test]
fn rust_trait_implementation_survives_to_workspace_query() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "inheritance.rs",
        "trait Processor { fn process(&self); }\nstruct Worker;\nimpl Processor for Worker { fn process(&self) {} }\n",
    );
    assert_forward_and_reverse(
        &dir,
        &state,
        "Implements",
        "Class",
        "Worker",
        "Interface",
        "Processor",
    );
}
