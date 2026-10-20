use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::Path;

fn state(additional: &[&Path]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    crate::mcp::McpState::new(config)
}

fn call(state: &crate::mcp::McpState, tool: &str, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(147),
        tool,
        &json!({ "arguments": arguments }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("tool response")
}

fn entities(state: &crate::mcp::McpState, root: &Path, name: &str) -> Vec<Value> {
    let response = call(
        state,
        "workspace_query",
        json!({
            "workspaceRoot": root.to_string_lossy(),
            "type": "find_entities",
            "name": name
        }),
    );
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entities")
        .clone()
}

fn role_files(values: &[Value], name: &str) -> BTreeSet<String> {
    values
        .iter()
        .filter(|entity| {
            entity["domain"] == "dotnet"
                && entity["entity_type"] == "DbContext"
                && entity["name"] == name
        })
        .filter_map(|entity| entity["file"].as_str().map(str::to_owned))
        .collect()
}

fn refresh(state: &crate::mcp::McpState, root: &Path) {
    let response = call(
        state,
        "refresh_workspace",
        json!({ "workspaceRoot": root.to_string_lossy() }),
    );
    assert!(response.get("error").is_none(), "{response}");
    assert_eq!(response["result"]["structuredContent"]["refreshed"], true);
}

fn write_context(root: &Path, body: &str) {
    std::fs::write(root.join("Context.cs"), body).expect("context fixture");
}

#[test]
fn issue_149_unmodeled_import_preserves_dbcontext_role_and_relationship() {
    let root = tempfile::TempDir::new().expect("workspace root");
    write_context(
        root.path(),
        "using Microsoft.EntityFrameworkCore; using Some.External.Namespace; namespace Mini; public class Widget {} public class MiniContext : DbContext { public DbSet<Widget> Widgets { get; set; } }\n",
    );
    let state = state(&[]);

    assert_eq!(
        role_files(&entities(&state, root.path(), "MiniContext"), "MiniContext").len(),
        1,
        "an unrelated unmodeled import must not retract the resolved framework role"
    );
    let has_entity = state
        .workspace_index_read()
        .forward_edges_by_identity("dotnet", "DbContext", "MiniContext")
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::HasEntity)
        .count();
    assert_eq!(has_entity, 1, "the resolved role must retain HasEntity");
}

#[test]
fn issue_147_same_name_context_roles_remain_owner_distinct() {
    let first = tempfile::TempDir::new().expect("first root");
    let second = tempfile::TempDir::new().expect("second root");
    write_context(
        first.path(),
        "public class ContextA : global::Microsoft.EntityFrameworkCore.DbContext {}\n",
    );
    write_context(
        second.path(),
        "public class EntityB {} public class ContextA : global::Microsoft.EntityFrameworkCore.DbContext { public DbSet<EntityB> Entities { get; set; } }\n",
    );
    let state = state(&[second.path()]);

    let found = entities(&state, first.path(), "ContextA");
    let relationship_owner =
        state.semantic_owner_path(&second.path().join("Context.cs").to_string_lossy());
    let expected = BTreeSet::from([
        state.semantic_owner_path(&first.path().join("Context.cs").to_string_lossy()),
        relationship_owner.clone(),
    ]);
    assert_eq!(role_files(&found, "ContextA"), expected);

    let index = state.workspace_index_read();
    assert_eq!(
        index
            .declarations_by_identity("builtin", "Class", "ContextA")
            .len(),
        2,
        "both source declarations remain owner-distinct"
    );
    assert!(
        index
            .declarations_by_identity("dotnet", "DbContext", "ContextA")
            .is_empty(),
        "framework roles are not declarations"
    );
    let has_entity: Vec<_> = index
        .forward_edges_by_identity("dotnet", "DbContext", "ContextA")
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::HasEntity)
        .collect();
    assert_eq!(has_entity.len(), 1, "only Root B proves one relationship");
    assert_eq!(has_entity[0].object.name, "EntityB");
    assert_eq!(
        has_entity[0].subject.file.as_deref(),
        Some(relationship_owner.as_str())
    );
}

#[test]
fn issue_147_dbcontext_role_follows_refresh_rename_base_removal_and_deletion() {
    let root = tempfile::TempDir::new().expect("workspace root");
    let file = root.path().join("Context.cs");
    write_context(
        root.path(),
        "public class RoleContext : global::Microsoft.EntityFrameworkCore.DbContext {}\n",
    );
    let state = state(&[]);

    assert_eq!(
        role_files(&entities(&state, root.path(), "RoleContext"), "RoleContext").len(),
        1
    );
    assert_eq!(
        role_files(&entities(&state, root.path(), "RoleContext"), "RoleContext").len(),
        1,
        "warm lookup reuses one durable occurrence"
    );

    write_context(
        root.path(),
        "public class RenamedContext : global::Microsoft.EntityFrameworkCore.DbContext {}\n",
    );
    refresh(&state, root.path());
    assert!(role_files(&entities(&state, root.path(), "RoleContext"), "RoleContext").is_empty());
    assert_eq!(
        role_files(
            &entities(&state, root.path(), "RenamedContext"),
            "RenamedContext"
        )
        .len(),
        1
    );

    write_context(
        root.path(),
        "public class PlainBase {} public class RenamedContext : PlainBase {}\n",
    );
    refresh(&state, root.path());
    let without_role = entities(&state, root.path(), "RenamedContext");
    assert!(role_files(&without_role, "RenamedContext").is_empty());
    assert!(
        without_role
            .iter()
            .any(|entity| { entity["domain"] == "builtin" && entity["entity_type"] == "Class" })
    );

    write_context(
        root.path(),
        "public class DeletedContext : global::Microsoft.EntityFrameworkCore.DbContext {}\n",
    );
    refresh(&state, root.path());
    assert_eq!(
        role_files(
            &entities(&state, root.path(), "DeletedContext"),
            "DeletedContext"
        )
        .len(),
        1
    );
    std::fs::remove_file(file).expect("delete context fixture");
    refresh(&state, root.path());
    assert!(
        role_files(
            &entities(&state, root.path(), "DeletedContext"),
            "DeletedContext"
        )
        .is_empty()
    );
}

#[test]
fn r6_cross_file_ancestry_reconciles_role_and_relationship_lifecycle() {
    let root = tempfile::TempDir::new().expect("workspace root");
    std::fs::write(
        root.path().join("Base.cs"),
        "using Microsoft.EntityFrameworkCore; public class BaseContext : DbContext {}\n",
    )
    .expect("base fixture");
    std::fs::write(
        root.path().join("Derived.cs"),
        "public class Entity {} public class DerivedContext : BaseContext { public DbSet<Entity> Entities { get; set; } }\n",
    )
    .expect("derived fixture");
    let state = state(&[]);

    assert_eq!(
        role_files(&entities(&state, root.path(), "BaseContext"), "BaseContext").len(),
        1
    );
    assert_eq!(
        role_files(
            &entities(&state, root.path(), "DerivedContext"),
            "DerivedContext"
        )
        .len(),
        1,
        "a resolved cross-file ancestry chain must publish the derived durable role"
    );
    assert_eq!(
        state
            .workspace_index_read()
            .forward_edges_by_identity("dotnet", "DbContext", "DerivedContext")
            .into_iter()
            .filter(|edge| edge.relation == SemanticRelation::HasEntity)
            .count(),
        1,
        "the reconciled role must authorize its existing relationship payload"
    );

    std::fs::write(
        root.path().join("Base.cs"),
        "public class PlainBase {} public class BaseContext : PlainBase {}\n",
    )
    .expect("remove EF ancestry");
    refresh(&state, root.path());
    assert!(
        role_files(
            &entities(&state, root.path(), "DerivedContext"),
            "DerivedContext"
        )
        .is_empty(),
        "changing an upstream base must retract the derived role"
    );
    assert!(
        state
            .workspace_index_read()
            .forward_edges_by_identity("dotnet", "DbContext", "DerivedContext")
            .into_iter()
            .all(|edge| edge.relation != SemanticRelation::HasEntity),
        "relationship authority must retract with the ancestry proof"
    );
}
