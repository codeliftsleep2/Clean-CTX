// RED regressions for query-time classification of cross-file C# class bases.
//
// Typed edges are derived ephemerally after hydration. The durable
// Class --HasBaseType--> TypeRef fact remains authoritative and queryable.

use super::*;

fn write(dir: &tempfile::TempDir, file: &str, source: &str) {
    std::fs::write(dir.path().join(file), source).unwrap();
}

fn query_edges(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    direction: &str,
    entity_type: &str,
    name: &str,
) -> serde_json::Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
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
    result["structuredContent"].clone()
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
                && edge["layer"] == "builtin"
        })
    })
}

fn assert_neutral(
    dir: &tempfile::TempDir,
    state: &crate::mcp::McpState,
    subject_type: &str,
    subject_name: &str,
    target: &str,
) {
    let neutral = query_edges(dir, state, "reverse_edges", "TypeRef", target);
    assert!(
        has_edge(
            &neutral,
            "HasBaseType",
            subject_type,
            subject_name,
            "TypeRef",
            target,
        ),
        "neutral written-base fact must remain queryable: {neutral:?}"
    );
}

#[test]
fn cold_reverse_interface_query_classifies_cross_file_implements() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "IFoo.cs", "public interface IFoo {}");
    write(&dir, "Foo.cs", "public class Foo : IFoo {}");

    let reverse = query_edges(&dir, &state, "reverse_edges", "Interface", "IFoo");
    assert!(
        has_edge(&reverse, "Implements", "Class", "Foo", "Interface", "IFoo"),
        "cold reverse query must derive the typed cross-file edge: {reverse:?}"
    );
    assert_neutral(&dir, &state, "Class", "Foo", "IFoo");
}

#[test]
fn cold_reverse_class_query_classifies_cross_file_extends() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "Base.cs", "public class BaseX {}");
    write(&dir, "Derived.cs", "public class DerivedX : BaseX {}");

    let reverse = query_edges(&dir, &state, "reverse_edges", "Class", "BaseX");
    assert!(
        has_edge(&reverse, "Extends", "Class", "DerivedX", "Class", "BaseX"),
        "cold reverse query must derive the typed cross-file edge: {reverse:?}"
    );
    assert_neutral(&dir, &state, "Class", "DerivedX", "BaseX");
}

#[test]
fn cold_forward_query_hydrates_target_before_classifying() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "IFoo.cs", "public interface IFoo {}");
    write(&dir, "Foo.cs", "public class Foo : IFoo {}");

    let forward = query_edges(&dir, &state, "forward_edges", "Class", "Foo");
    assert!(
        has_edge(&forward, "Implements", "Class", "Foo", "Interface", "IFoo"),
        "cold forward query must stage target hydration before classification: {forward:?}"
    );
    assert!(
        has_edge(&forward, "HasBaseType", "Class", "Foo", "TypeRef", "IFoo"),
        "classification must be additive to the neutral fact: {forward:?}"
    );
}

#[test]
fn class_interface_name_collision_remains_neutral() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "ClassShared.cs",
        "namespace A { public class Shared {} }",
    );
    write(
        &dir,
        "InterfaceShared.cs",
        "namespace B { public interface Shared {} }",
    );
    write(&dir, "Child.cs", "public class Child : Shared {}");

    for entity_type in ["Class", "Interface"] {
        let reverse = query_edges(&dir, &state, "reverse_edges", entity_type, "Shared");
        assert!(
            reverse["edges"]
                .as_array()
                .is_some_and(|edges| edges.iter().all(|edge| {
                    !matches!(edge["relation"].as_str(), Some("Extends" | "Implements"))
                })),
            "ambiguous target kind must not produce a typed edge: {reverse:?}"
        );
    }
    assert_neutral(&dir, &state, "Class", "Child", "Shared");
}

#[test]
fn supported_refresh_reclassifies_without_stale_typed_edges() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "Target.cs", "public interface Target {}");
    write(&dir, "Child.cs", "public class Child : Target {}");

    let as_interface = query_edges(&dir, &state, "reverse_edges", "Interface", "Target");
    assert!(has_edge(
        &as_interface,
        "Implements",
        "Class",
        "Child",
        "Interface",
        "Target",
    ));

    write(&dir, "Target.cs", "public class Target {}");
    crate::mcp::tool_handlers::hydration::reconcile_external_refresh_for_root(
        &state,
        &dir.path().to_string_lossy(),
    );

    let as_class = query_edges(&dir, &state, "reverse_edges", "Class", "Target");
    assert!(
        has_edge(&as_class, "Extends", "Class", "Child", "Class", "Target"),
        "target kind change must derive only the current classification: {as_class:?}"
    );
    let old_kind = query_edges(&dir, &state, "reverse_edges", "Interface", "Target");
    assert!(
        !has_edge(
            &old_kind,
            "Implements",
            "Class",
            "Child",
            "Interface",
            "Target",
        ),
        "ephemeral classification must leave no stale stored edge: {old_kind:?}"
    );
}

#[test]
fn class_resolver_does_not_classify_struct_or_record_neutral_facts() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "IFoo.cs", "public interface IFoo {}");
    write(
        &dir,
        "Values.cs",
        "public struct Value : IFoo {} public record Item : IFoo;",
    );

    let reverse = query_edges(&dir, &state, "reverse_edges", "Interface", "IFoo");
    assert!(
        reverse["edges"]
            .as_array()
            .is_some_and(|edges| edges.iter().all(|edge| {
                !(matches!(
                    edge["subject"]["entity_type"].as_str(),
                    Some("Struct" | "Record")
                ) && matches!(edge["relation"].as_str(), Some("Extends" | "Implements")))
            })),
        "Class refinement must not consume Struct/Record neutral facts: {reverse:?}"
    );
    assert_neutral(&dir, &state, "Struct", "Value", "IFoo");
    assert_neutral(&dir, &state, "Record", "Item", "IFoo");
}

#[test]
fn empty_class_reverse_query_points_to_neutral_type_ref() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(&dir, "Base.cs", "public class BaseX {}");

    let reverse = query_edges(&dir, &state, "reverse_edges", "Class", "BaseX");
    assert_eq!(
        reverse["coverage"]["alternative_query"],
        json!({
            "type": "reverse_edges",
            "domain": "builtin",
            "entity_type": "TypeRef",
            "name": "BaseX",
            "relation": "HasBaseType",
            "meaning": "C# classes naming this written type in an unresolved base list"
        })
    );
}

#[test]
fn qualified_base_resolves_by_structured_namespace_identity() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "Right.cs",
        "namespace RightNs { public class QualifiedBase {} }",
    );
    write(
        &dir,
        "Wrong.cs",
        "namespace WrongNs { public interface QualifiedBase {} }",
    );
    write(
        &dir,
        "Child.cs",
        "namespace ConsumerNs { public class QualifiedChild : RightNs.QualifiedBase {} }",
    );

    let reverse = query_edges(&dir, &state, "reverse_edges", "Class", "QualifiedBase");
    assert!(
        has_edge(
            &reverse,
            "Extends",
            "Class",
            "QualifiedChild",
            "Class",
            "QualifiedBase",
        ),
        "the written qualifier must select the declaration in RightNs: {reverse:?}"
    );
    assert_neutral(
        &dir,
        &state,
        "Class",
        "QualifiedChild",
        "RightNs.QualifiedBase",
    );
}

#[test]
fn constructed_generic_base_resolves_by_name_namespace_and_arity() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "GenericBase.cs",
        "namespace GenericNs; public class GenericBase<T> {}",
    );
    write(
        &dir,
        "GenericChild.cs",
        "namespace GenericNs; public class GenericChild : GenericBase<int> {}",
    );

    let reverse = query_edges(&dir, &state, "reverse_edges", "Class", "GenericBase");
    assert!(
        has_edge(
            &reverse,
            "Extends",
            "Class",
            "GenericChild",
            "Class",
            "GenericBase",
        ),
        "a constructed generic reference must resolve to the arity-one declaration: {reverse:?}"
    );
    assert_neutral(&dir, &state, "Class", "GenericChild", "GenericBase<int>");
}

#[test]
fn generic_arity_mismatch_remains_neutral() {
    let dir = tempfile::TempDir::new().unwrap();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    write(
        &dir,
        "GenericBase.cs",
        "namespace GenericNs; public class GenericBase<TFirst, TSecond> {}",
    );
    write(
        &dir,
        "GenericChild.cs",
        "namespace GenericNs; public class GenericChild : GenericBase<int> {}",
    );

    let reverse = query_edges(&dir, &state, "reverse_edges", "Class", "GenericBase");
    assert!(
        reverse["edges"].as_array().is_some_and(|edges| edges
            .iter()
            .all(|edge| !matches!(edge["relation"].as_str(), Some("Extends" | "Implements")))),
        "an arity mismatch must never be guessed into a typed edge: {reverse:?}"
    );
    assert_neutral(&dir, &state, "Class", "GenericChild", "GenericBase<int>");
}
