// RED regressions for structured C# declaration-header authority.
//
// Production path:
// source -> tree-sitter captures -> canonical IR / builtin projection ->
// WorkspaceIndex -> workspace_query.

use super::*;

fn write(dir: &tempfile::TempDir, file: &str, source: &str) {
    std::fs::write(dir.path().join(file), source).unwrap();
}

fn publish(dir: &tempfile::TempDir, state: &crate::mcp::McpState, file: &str, source: &str) {
    write(dir, file, source);
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

fn query(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    query_type: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(2),
        "workspace_query",
        &json!({
            "arguments": {
                "type": query_type,
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
        panic!("{query_type} must succeed for {entity_type}/{name}: {response:?}")
    });
    assert_valid_mcp_envelope(result);
    result["structuredContent"].clone()
}

fn entities_in_file(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    file: &str,
) -> Vec<serde_json::Value> {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(3),
        "workspace_query",
        &json!({
            "arguments": {
                "type": "entities_in_file",
                "file_path": dir.path().join(file).to_string_lossy(),
                "workspaceRoot": dir.path().to_string_lossy()
            }
        }),
        state,
    );
    let response = pop_response();
    let result = response["result"]
        .as_object()
        .unwrap_or_else(|| panic!("entities_in_file must succeed: {response:?}"));
    assert_valid_mcp_envelope(result);
    result["structuredContent"]["entities"]
        .as_array()
        .cloned()
        .expect("structured entity array")
}

fn has_edge(
    result: &serde_json::Value,
    relation: &str,
    subject_type: &str,
    subject_name: &str,
    object_type: &str,
    object_name: &str,
) -> bool {
    result["edges"].as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            edge["relation"] == relation
                && edge["subject"]["domain"] == "builtin"
                && edge["subject"]["entity_type"] == subject_type
                && edge["subject"]["name"] == subject_name
                && edge["object"]["domain"] == "builtin"
                && edge["object"]["entity_type"] == object_type
                && edge["object"]["name"] == object_name
        })
    })
}

fn has_any_base_edge(result: &serde_json::Value) -> bool {
    result["edges"].as_array().is_some_and(|edges| {
        edges.iter().any(|edge| {
            matches!(
                edge["relation"].as_str(),
                Some("HasBaseType" | "Extends" | "Implements")
            )
        })
    })
}

#[test]
fn csharp_structured_headers_preserve_multiline_class_base_entries() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "Targets.cs",
        "public class UniqueMultiBase {} public interface IUniqueMultiOne {} public interface IUniqueMultiTwo {}",
    );
    publish(
        &dir,
        &state,
        "Child.cs",
        "public class UniqueMultiChild
: UniqueMultiBase,
IUniqueMultiOne,
IUniqueMultiTwo
{}",
    );

    let forward = query(&dir, &state, "forward_edges", "Class", "UniqueMultiChild");
    assert!(has_edge(
        &forward,
        "HasBaseType",
        "Class",
        "UniqueMultiChild",
        "TypeRef",
        "UniqueMultiBase",
    ));
    assert!(has_edge(
        &forward,
        "Extends",
        "Class",
        "UniqueMultiChild",
        "Class",
        "UniqueMultiBase",
    ));
    for interface in ["IUniqueMultiOne", "IUniqueMultiTwo"] {
        assert!(
            has_edge(
                &forward,
                "Implements",
                "Class",
                "UniqueMultiChild",
                "Interface",
                interface,
            ),
            "multiline direct base-list child must survive intact: {forward:?}"
        );
    }
}

#[test]
fn csharp_structured_headers_preserve_partial_class_identity_and_base() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "Base.cs", "public class UniquePartialBase {}");
    publish(
        &dir,
        &state,
        "Partial.cs",
        "public partial class UniquePartialChild : UniquePartialBase {}",
    );

    let forward = query(&dir, &state, "forward_edges", "Class", "UniquePartialChild");
    assert!(
        has_edge(
            &forward,
            "HasBaseType",
            "Class",
            "UniquePartialChild",
            "TypeRef",
            "UniquePartialBase",
        ),
        "partial must not hide or corrupt the Class identity/base list: {forward:?}"
    );
}

#[test]
fn csharp_structured_headers_exclude_constraints_and_comments() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "ICommentOnly.cs",
        "public interface IUniqueCommentOnly {}",
    );
    publish(
        &dir,
        &state,
        "NoBases.cs",
        "public class UniqueConstraintOnly<T> where T : class {}
public class UniqueLineComment // : IUniqueCommentOnly
{}
public class UniqueBlockComment /* : IUniqueCommentOnly */ {}",
    );

    for name in [
        "UniqueConstraintOnly",
        "UniqueLineComment",
        "UniqueBlockComment",
    ] {
        let forward = query(&dir, &state, "forward_edges", "Class", name);
        assert!(
            !has_any_base_edge(&forward),
            "constraint/comment text must never become a base fact for {name}: {forward:?}"
        );
    }
}

#[test]
fn csharp_structured_headers_stop_constraints_at_the_base_list_boundary() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "Base.cs", "public class UniqueConstraintBase {}");
    publish(
        &dir,
        &state,
        "Child.cs",
        "public class UniqueConstrainedChild<T> : UniqueConstraintBase where T : class {}",
    );

    let forward = query(
        &dir,
        &state,
        "forward_edges",
        "Class",
        "UniqueConstrainedChild",
    );
    assert!(has_edge(
        &forward,
        "HasBaseType",
        "Class",
        "UniqueConstrainedChild",
        "TypeRef",
        "UniqueConstraintBase",
    ));
    assert!(
        !has_edge(
            &forward,
            "HasBaseType",
            "Class",
            "UniqueConstrainedChild",
            "TypeRef",
            "UniqueConstraintBase where T : class",
        ),
        "constraint text must not pollute the written base name: {forward:?}"
    );
}

#[test]
fn csharp_structured_headers_share_canonical_and_builtin_declared_identity() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Primary.cs",
        "public record UniquePositionalRecord(int X) : UniqueExternalRecordBase;
public class UniquePrimaryClass(int x) : UniqueExternalClassBase {}",
    );

    let entities = entities_in_file(&dir, &state, "Primary.cs");
    for (entity_type, name) in [
        ("Record", "UniquePositionalRecord"),
        ("Class", "UniquePrimaryClass"),
    ] {
        assert!(
            entities.iter().any(|entity| {
                entity["domain"] == "builtin"
                    && entity["entity_type"] == entity_type
                    && entity["name"] == name
            }),
            "builtin identity must use the structured declared identifier {name}: {entities:?}"
        );
    }
    assert!(
        entities.iter().all(|entity| {
            !matches!(
                entity["name"].as_str(),
                Some("UniquePositionalRecord(int" | "UniquePrimaryClass(int")
            )
        }),
        "truncated primary-constructor identities must not be registered: {entities:?}"
    );

    for (entity_type, name, target) in [
        (
            "Record",
            "UniquePositionalRecord",
            "UniqueExternalRecordBase",
        ),
        ("Class", "UniquePrimaryClass", "UniqueExternalClassBase"),
    ] {
        let forward = query(&dir, &state, "forward_edges", entity_type, name);
        assert!(
            has_edge(
                &forward,
                "HasBaseType",
                entity_type,
                name,
                "TypeRef",
                target,
            ),
            "base fact must attach to the same declared identity: {forward:?}"
        );
    }
}

#[test]
fn csharp_structured_headers_preserve_qualified_and_generic_refs_neutrally() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    publish(
        &dir,
        &state,
        "Refs.cs",
        "public class UniqueGenericChild : GenericBase<int> {}
public class UniqueQualifiedChild : UniqueNs.QualifiedBase {}",
    );

    for (child, written) in [
        ("UniqueGenericChild", "GenericBase<int>"),
        ("UniqueQualifiedChild", "UniqueNs.QualifiedBase"),
    ] {
        let forward = query(&dir, &state, "forward_edges", "Class", child);
        assert!(
            has_edge(&forward, "HasBaseType", "Class", child, "TypeRef", written,),
            "written reference must remain intact: {forward:?}"
        );
        assert!(
            forward["edges"].as_array().is_some_and(|edges| edges
                .iter()
                .all(|edge| !matches!(edge["relation"].as_str(), Some("Extends" | "Implements")))),
            "qualified/generic reference must remain neutral without exact authority: {forward:?}"
        );
    }
}

#[test]
fn csharp_structured_headers_derived_edge_establishes_query_capability() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "Target.cs",
        "public interface IUniqueCoverageTarget {}",
    );
    write(
        &dir,
        "Child.cs",
        "public class UniqueCoverageChild : IUniqueCoverageTarget {}",
    );

    let reverse = query(
        &dir,
        &state,
        "reverse_edges",
        "Interface",
        "IUniqueCoverageTarget",
    );
    assert!(has_edge(
        &reverse,
        "Implements",
        "Class",
        "UniqueCoverageChild",
        "Interface",
        "IUniqueCoverageTarget",
    ));
    assert_eq!(
        reverse["coverage"]["capability_established"], true,
        "a response-derived typed edge establishes this query capability: {reverse:?}"
    );
    assert_eq!(
        reverse["coverage"]["status"], "established_indexed_capability",
        "coverage must not contradict its non-empty typed answer: {reverse:?}"
    );
    assert!(
        reverse["coverage"].get("alternative_query").is_none(),
        "a successful typed answer must not suggest an alternative query: {reverse:?}"
    );
}

#[test]
fn csharp_structured_headers_version_existing_producer_keys() {
    assert_eq!(
        crate::ir::layers::csharp::CANONICAL_PRODUCER_GENERATION,
        2,
        "structured Class identity/base lowering changes canonical output"
    );
    assert_eq!(
        crate::ir::layers::csharp::SEMANTIC_INPUT_GENERATION,
        2,
        "corrected canonical facts change projected semantic output"
    );
    assert_eq!(
        crate::ir::semantic_projection::CSHARP_SEMANTIC_PROJECTION_GENERATION,
        3,
        "corrected positional Record ownership changes the existing projection output"
    );
}
