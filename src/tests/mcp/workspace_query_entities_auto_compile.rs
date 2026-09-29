use crate::mcp::tools::{dispatch_tools_call, tool_list};
use crate::tests::assert_valid_mcp_envelope;
use serde_json::{Value, json};

fn pop_response() -> Value {
    crate::protocol::captured_responses()
        .pop()
        .expect("handler must send a response")
}

fn query_entities(
    state: &crate::mcp::McpState,
    root: &std::path::Path,
    file_path: &str,
    fidelity: &str,
) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "entities_in_file",
                "file_path": file_path,
                "workspaceRoot": root.to_string_lossy(),
                "fidelity": fidelity
            }
        }),
        state,
    );
    pop_response()
}

#[test]
fn entities_in_file_compiles_an_untracked_file_without_publishing_context() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("service.ts");
    std::fs::write(
        &path,
        "export class UserService { getUser(id: number): number { return id; } }\n",
    )
    .expect("source fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let response = query_entities(&state, root.path(), "service.ts", "high");
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("entities_in_file must succeed: {response}"));
    assert_valid_mcp_envelope(result);
    let entities = response["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entities array");
    assert!(
        entities.iter().any(|entity| {
            entity["domain"] == "builtin"
                && entity["entity_type"] == "Class"
                && entity["name"] == "UserService"
        }),
        "one-call query must compile and expose the file's class: {response}"
    );

    let canonical = crate::dictionary::path::canonical_identity_key(&path.to_string_lossy());
    assert_eq!(
        state.context_fidelity(&canonical),
        None,
        "query-only semantic compilation must not claim a rendered context"
    );
    assert_eq!(
        state.alias_for_path(&path.to_string_lossy()),
        None,
        "query-only semantic compilation must not create session alias ownership"
    );
}

#[test]
fn entities_in_file_rejects_invalid_fidelity() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("service.ts"),
        "export class UserService {}\n",
    )
    .expect("source fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let response = query_entities(&state, root.path(), "service.ts", "maximum");
    assert_eq!(
        response["error"]["code"], -32602,
        "invalid workspace-query fidelity must be rejected: {response}"
    );
}

#[test]
fn workspace_query_schema_advertises_entities_in_file_fidelity() {
    let workspace_query = tool_list()
        .into_iter()
        .find(|tool| tool["name"] == "workspace_query")
        .expect("workspace_query tool declaration");
    let fidelity = &workspace_query["inputSchema"]["properties"]["fidelity"];

    assert_eq!(
        fidelity["enum"],
        json!(["low", "medium", "high", "edit", "verbatim"]),
        "the public schema must advertise the accepted fidelity spellings"
    );
    assert!(
        fidelity["description"]
            .as_str()
            .is_some_and(|description| description.contains("entities_in_file")),
        "the schema must scope fidelity to entities_in_file: {fidelity}"
    );
}

struct InjectedSourceFailureReset;

impl Drop for InjectedSourceFailureReset {
    fn drop(&mut self) {
        *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
            .lock()
            .expect("source-failure injection lock") = None;
    }
}

#[test]
fn sufficient_fresh_projection_is_reused_without_recompilation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("service.ts"),
        "export class ReusedService {} // reuse-projection-marker\n",
    )
    .expect("source fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let first = query_entities(&state, root.path(), "service.ts", "high");
    assert!(first.get("error").is_none(), "initial compilation: {first}");

    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("source-failure injection lock") = Some("reuse-projection-marker".to_string());
    let _reset = InjectedSourceFailureReset;

    let second = query_entities(&state, root.path(), "service.ts", "low");
    assert!(
        second.get("error").is_none(),
        "a fresh High projection must satisfy a later Low request without recompilation: {second}"
    );
    assert_eq!(second["result"]["structuredContent"]["count"], 1);
}

#[test]
#[cfg(any(feature = "csharp", feature = "dotnet"))]
fn lower_fidelity_projection_is_replaced_for_a_higher_request() {
    let root = tempfile::tempdir().expect("workspace");
    std::fs::write(
        root.path().join("UsersController.cs"),
        r#"using Microsoft.AspNetCore.Mvc;
[ApiController]
public class UsersController : ControllerBase {
    [HttpGet]
    public IActionResult GetAll() { return Ok(); }
}
"#,
    )
    .expect("controller fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    let low = query_entities(&state, root.path(), "UsersController.cs", "low");
    let low_entities = low["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("Low entities");
    assert!(low_entities.iter().all(|entity| {
        !(entity["domain"] == "dotnet"
            && entity["entity_type"] == "Action"
            && entity["name"] == "GetAll")
    }));

    let medium = query_entities(&state, root.path(), "UsersController.cs", "medium");
    let medium_entities = medium["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("Medium entities");
    assert!(
        medium_entities.iter().any(|entity| {
            entity["domain"] == "dotnet"
                && entity["entity_type"] == "Action"
                && entity["name"] == "GetAll"
        }),
        "Medium must replace the insufficient Low projection: {medium}"
    );
}

#[test]
fn changed_source_replaces_stale_entities() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("service.ts");
    std::fs::write(&path, "export class OldService {}\n").expect("initial source");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let first = query_entities(&state, root.path(), "service.ts", "high");
    assert!(first.to_string().contains("OldService"));

    std::fs::write(&path, "export class ReplacementService {}\n").expect("changed source");
    let second = query_entities(&state, root.path(), "service.ts", "high");
    let rendered = second.to_string();
    assert!(rendered.contains("ReplacementService"), "{second}");
    assert!(
        !rendered.contains("OldService"),
        "stale entity survived: {second}"
    );
}

#[test]
fn empty_recompile_removes_prior_file_entities() {
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("service.ts");
    std::fs::write(&path, "export class RemovedService {}\n").expect("initial source");
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    let first = query_entities(&state, root.path(), "service.ts", "high");
    assert!(first.to_string().contains("RemovedService"));

    std::fs::write(&path, "// no semantic declarations remain\n").expect("empty source");
    let second = query_entities(&state, root.path(), "service.ts", "high");
    assert_eq!(
        second["result"]["structuredContent"]["count"], 0,
        "{second}"
    );
}

#[test]
fn provide_code_context_projection_is_reused_by_entities_in_file() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let path = root.path().join("provided.ts");
    std::fs::write(
        &path,
        "export class ProvidedService {} // provided-projection-marker\n",
    )
    .expect("source fixture");
    let state = crate::mcp::McpState::new(crate::tests::test_config());

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": "provided.ts",
                "workspaceRoot": root.path().to_string_lossy(),
                "fidelity": "high"
            }
        }),
        &state,
    );
    let provided = pop_response();
    assert!(
        provided.get("error").is_none(),
        "initial provide: {provided}"
    );

    *crate::mcp::tool_helpers::TEST_INJECTED_SOURCE_FAILURE
        .lock()
        .expect("source-failure injection lock") = Some("provided-projection-marker".to_string());
    let _reset = InjectedSourceFailureReset;

    let queried = query_entities(&state, root.path(), "provided.ts", "low");
    assert!(
        queried.get("error").is_none(),
        "the fresh High projection published by provide must satisfy the Low query: {queried}"
    );
    assert!(queried.to_string().contains("ProvidedService"));
}
