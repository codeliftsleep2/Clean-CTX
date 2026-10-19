//! Two-root duplicate-name authority falsification.
//!
//! This fixture preserves source occurrence provenance and neutral written-base
//! facts. It deliberately does not turn a shared name into one physical identity
//! or treat an external TypeRef as a locally declared class.

use super::{classification, csharp_resolution};
use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::{GraphBridge, GraphNode};
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::tool_handlers::hydration::{
    TestProjectReadiness, clear_test_project_search_results, hydrate_workspace_index,
    set_test_inbound_project_search_results, set_test_project_readiness,
    set_test_project_search_results,
};
use crate::workspace::scope::WorkspaceScope;
use std::collections::{BTreeSet, HashMap};
use std::path::Path;

static TEST_SERIALIZE: &std::sync::Mutex<()> =
    &crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;

#[derive(Clone, Copy, Debug)]
enum ProviderState {
    Ready,
    StillIndexing,
    Unavailable,
}

struct ResetProjectSearch;

impl Drop for ResetProjectSearch {
    fn drop(&mut self) {
        clear_test_project_search_results();
    }
}

fn node(file: &str) -> GraphNode {
    GraphNode {
        id: file.into(),
        label: "Class".into(),
        name: "ProjectContext".into(),
        file: file.into(),
        properties: HashMap::new(),
    }
}

fn project_slug(root: &Path) -> String {
    cbm_project_slug(&root.canonicalize().expect("canonical test root"))
}

fn write_context(root: &Path, namespace: &str) {
    std::fs::write(
        root.join("ProjectContext.cs"),
        format!(
            "using Microsoft.EntityFrameworkCore; namespace {namespace} {{ public class ProjectContext : DbContext {{}} }}\n"
        ),
    )
    .expect("ProjectContext fixture");
}

fn state_with_roots(primary: &Path, additional: &Path) -> crate::mcp::McpState {
    let additional_roots = vec![additional.to_path_buf()];
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional_roots
        .iter()
        .map(|root| root.to_string_lossy().into_owned())
        .collect();
    let state = crate::mcp::McpState::new(config);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        primary,
        &additional_roots,
    ));
    state
}

fn configure_provider(primary: &Path, additional: &Path, provider: ProviderState) {
    let primary_slug = project_slug(primary);
    let additional_slug = project_slug(additional);
    let (primary_nodes, additional_nodes, readiness) = match provider {
        ProviderState::Ready => (
            vec![node("ProjectContext.cs")],
            vec![node("ProjectContext.cs")],
            TestProjectReadiness::Ready,
        ),
        ProviderState::StillIndexing => (
            vec![node("ProjectContext.cs")],
            Vec::new(),
            TestProjectReadiness::StillIndexing,
        ),
        ProviderState::Unavailable => (Vec::new(), Vec::new(), TestProjectReadiness::Unavailable),
    };
    set_test_project_search_results(HashMap::from([
        (primary_slug.clone(), Ok(primary_nodes)),
        (additional_slug.clone(), Ok(additional_nodes)),
    ]));
    set_test_inbound_project_search_results(HashMap::from([
        (primary_slug.clone(), Ok(Vec::new())),
        (additional_slug.clone(), Ok(Vec::new())),
    ]));
    set_test_project_readiness(HashMap::from([
        (primary_slug, readiness),
        (additional_slug, readiness),
    ]));
}

fn assert_authority(
    state: &crate::mcp::McpState,
    primary: &Path,
    expected_files: &BTreeSet<String>,
) {
    let index = state.workspace_index_read();
    let declarations = index.declarations_by_identity("builtin", "Class", "ProjectContext");
    let declaration_files: BTreeSet<_> = declarations
        .iter()
        .filter_map(|declaration| declaration.file.clone())
        .collect();
    assert_eq!(declaration_files, *expected_files);

    let scope = WorkspaceScope::new(
        Some(&primary.to_string_lossy()),
        &state.config.additional_roots,
    )
    .expect("workspace scope");
    let neutral =
        index.forward_edges_by_identity_in_scope("builtin", "Class", "ProjectContext", &scope);
    let base_files: BTreeSet<_> = neutral
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::HasBaseType
                && edge.object.domain == "builtin"
                && edge.object.entity_type == "TypeRef"
                && edge.object.name == "DbContext"
        })
        .filter_map(|edge| edge.subject.file.clone())
        .collect();
    assert_eq!(base_files, *expected_files);

    let catalog = csharp_resolution::Catalog::build(state, &index, Some(&scope));
    let identity = super::identity::ResolvedIdentity {
        domain: "builtin".into(),
        entity_type: "Class".into(),
        name: "ProjectContext".into(),
    };
    let query_visible =
        classification::forward(&index, &identity, Some(&scope), true, Some(&catalog));
    let visible_base_files: BTreeSet<_> = query_visible
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::HasBaseType)
        .filter_map(|edge| edge.subject.file.clone())
        .collect();
    assert_eq!(visible_base_files, *expected_files);
    assert!(
        query_visible
            .iter()
            .all(|edge| edge.relation != SemanticRelation::Extends),
        "an external DbContext TypeRef is not a unique admitted Class declaration"
    );
    assert!(
        index
            .declarations_by_identity("dotnet", "DbContext", "ProjectContext")
            .is_empty(),
        "source Class declarations must not be rewritten as dotnet/DbContext declarations"
    );
}

fn run_case(provider: ProviderState, reverse_roots: bool) {
    let first = tempfile::TempDir::new().expect("first root");
    let second = tempfile::TempDir::new().expect("second root");
    write_context(first.path(), "FirstRoot");
    write_context(second.path(), "SecondRoot");
    let (primary, additional) = if reverse_roots {
        (second.path(), first.path())
    } else {
        (first.path(), second.path())
    };
    let state = state_with_roots(primary, additional);
    configure_provider(primary, additional, provider);
    state.reset_test_csharp_reconciliation_count();

    let cold = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&primary.to_string_lossy()),
    )
    .expect("cold hydration");
    assert!(cold.is_complete(), "cold Clean authority: {provider:?}");
    assert_eq!(cold.candidates_compiled, 2);
    assert_eq!(state.test_csharp_reconciliation_count(), 0);

    let expected_files = BTreeSet::from([
        state.semantic_owner_path(&first.path().join("ProjectContext.cs").to_string_lossy()),
        state.semantic_owner_path(&second.path().join("ProjectContext.cs").to_string_lossy()),
    ]);
    assert_authority(&state, primary, &expected_files);

    let warm = hydrate_workspace_index(
        &state,
        "find_entities",
        "ProjectContext",
        Some(&primary.to_string_lossy()),
    )
    .expect("warm hydration");
    assert!(warm.is_complete(), "warm Clean authority: {provider:?}");
    assert_eq!(warm.candidates_compiled, 0);
    assert_eq!(state.test_csharp_reconciliation_count(), 0);
    assert_authority(&state, primary, &expected_files);
}

#[test]
fn two_root_projectcontext_preserves_independent_source_authority() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    for provider in [
        ProviderState::Ready,
        ProviderState::StillIndexing,
        ProviderState::Unavailable,
    ] {
        run_case(provider, false);
        run_case(provider, true);
    }
}

#[cfg(feature = "dotnet")]
fn public_find_entities(
    state: &crate::mcp::McpState,
    root: &Path,
    name: &str,
    declarations_only: bool,
) -> Vec<serde_json::Value> {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(147),
        "workspace_query",
        &serde_json::json!({ "arguments": {
            "workspaceRoot": root.to_string_lossy(),
            "type": "find_entities",
            "name": name,
            "domain": "dotnet",
            "entity_type": "DbContext",
            "declarations_only": declarations_only
        }}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entities")
        .clone()
}

#[cfg(feature = "dotnet")]
#[test]
fn issue_147_structured_dbcontext_role_does_not_require_relationship_payload() {
    let root = tempfile::TempDir::new().expect("workspace root");
    std::fs::write(
        root.path().join("InfrastructureContext.csproj"),
        "<Project Sdk=\"Microsoft.NET.Sdk\" />\n",
    )
    .expect("project fixture");
    std::fs::write(
        root.path().join("InfrastructureContext.cs"),
        r#"
public class InfrastructureContext : global::Microsoft.EntityFrameworkCore.DbContext
{
    public T Select<T>() where T : class
    {
        throw new System.NotImplementedException();
    }
}

public class FakeContext : DbContext
{
    private const string Marker = ": DbContext";
}
"#,
    )
    .expect("DbContext authority fixture");

    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    let state = crate::mcp::McpState::new(config);

    let entities = public_find_entities(&state, root.path(), "InfrastructureContext", false);
    assert!(
        entities.iter().any(|entity| {
            entity["domain"] == "builtin"
                && entity["entity_type"] == "Class"
                && entity["name"] == "InfrastructureContext"
        }),
        "the source declaration remains visible as builtin/Class"
    );
    assert!(
        entities.iter().any(|entity| {
            entity["domain"] == "dotnet"
                && entity["entity_type"] == "DbContext"
                && entity["name"] == "InfrastructureContext"
                && entity["file"].as_str().is_some()
        }),
        "structured fully-qualified EF base authority must publish a durable DbContext role"
    );

    let index = state.workspace_index_read();
    assert!(
        index
            .declarations_by_identity("builtin", "Class", "InfrastructureContext")
            .iter()
            .any(|entity| entity.file.is_some()),
        "source declaration authority stays builtin/Class"
    );
    assert!(
        index
            .declarations_by_identity("dotnet", "DbContext", "InfrastructureContext")
            .is_empty(),
        "the durable framework role must not become source declaration authority"
    );
    assert_eq!(
        index
            .forward_edges_by_identity("dotnet", "DbContext", "InfrastructureContext")
            .into_iter()
            .filter(|edge| edge.relation == SemanticRelation::HasEntity)
            .count(),
        0,
        "role identity alone must not fabricate outgoing HasEntity edges"
    );
    assert_eq!(
        index
            .reverse_edges_by_identity("dotnet", "DbContext", "InfrastructureContext")
            .into_iter()
            .filter(|edge| edge.relation == SemanticRelation::HasEntity)
            .count(),
        0,
        "role identity alone must not fabricate incoming HasEntity edges"
    );
    drop(index);

    assert!(
        public_find_entities(&state, root.path(), "InfrastructureContext", true).is_empty(),
        "declarations_only dotnet/DbContext remains empty under Issue #145"
    );

    let fake = public_find_entities(&state, root.path(), "FakeContext", false);
    assert!(
        fake.iter().any(|entity| {
            entity["domain"] == "builtin"
                && entity["entity_type"] == "Class"
                && entity["name"] == "FakeContext"
        }),
        "the negative control remains a source class"
    );
    assert!(
        fake.iter()
            .all(|entity| { entity["domain"] != "dotnet" || entity["entity_type"] != "DbContext" }),
        "an unqualified spelling without supported framework authority must not gain the role"
    );
}

#[cfg(feature = "dotnet")]
fn write_classified_context(root: &Path, namespace: &str) {
    std::fs::write(
        root.join(format!("{namespace}.csproj")),
        "<Project Sdk=\"Microsoft.NET.Sdk\" />\n",
    )
    .expect("project fixture");
    std::fs::write(
        root.join("ContextA.cs"),
        format!(
            "using Microsoft.EntityFrameworkCore; namespace {namespace} {{ public interface IContextA {{}} public class EntityA {{}} public class ContextA : DbContext, IContextA {{ public DbSet<EntityA> Entities {{ get; set; }} }} }}\n"
        ),
    )
    .expect("classified ContextA fixture");
}

#[cfg(feature = "dotnet")]
fn configure_context_provider(primary: &Path, additional: &Path, available: bool) {
    let readiness = if available {
        TestProjectReadiness::Ready
    } else {
        TestProjectReadiness::Unavailable
    };
    let nodes = available.then(|| {
        vec![GraphNode {
            id: "ContextA.cs".into(),
            label: "Class".into(),
            name: "ContextA".into(),
            file: "ContextA.cs".into(),
            properties: HashMap::new(),
        }]
    });
    let primary_slug = project_slug(primary);
    let additional_slug = project_slug(additional);
    set_test_project_search_results(HashMap::from([
        (primary_slug.clone(), Ok(nodes.clone().unwrap_or_default())),
        (additional_slug.clone(), Ok(nodes.unwrap_or_default())),
    ]));
    set_test_inbound_project_search_results(HashMap::from([
        (primary_slug.clone(), Ok(Vec::new())),
        (additional_slug.clone(), Ok(Vec::new())),
    ]));
    set_test_project_readiness(HashMap::from([
        (primary_slug, readiness),
        (additional_slug, readiness),
    ]));
}

#[cfg(feature = "dotnet")]
fn public_context_occurrence_files(
    state: &crate::mcp::McpState,
    primary: &Path,
) -> BTreeSet<String> {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &serde_json::json!(146),
        "workspace_query",
        &serde_json::json!({ "arguments": {
            "workspaceRoot": primary.to_string_lossy(),
            "type": "find_entities",
            "name": "ContextA"
        }}),
        state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("workspace_query response");
    assert!(response.get("error").is_none(), "{response}");
    response["result"]["structuredContent"]["entities"]
        .as_array()
        .expect("entities")
        .iter()
        .filter(|entity| {
            entity["domain"] == "dotnet"
                && entity["entity_type"] == "DbContext"
                && entity["name"] == "ContextA"
        })
        .filter_map(|entity| entity["file"].as_str().map(str::to_owned))
        .collect()
}

#[cfg(feature = "dotnet")]
fn run_cross_root_dbcontext_case(provider_available: bool, reverse_roots: bool) {
    let first = tempfile::TempDir::new().expect("first root");
    let second = tempfile::TempDir::new().expect("second root");
    write_classified_context(first.path(), "FirstRoot");
    write_classified_context(second.path(), "SecondRoot");
    let (primary, additional) = if reverse_roots {
        (second.path(), first.path())
    } else {
        (first.path(), second.path())
    };
    let state = state_with_roots(primary, additional);
    configure_context_provider(primary, additional, provider_available);

    let cold = hydrate_workspace_index(
        &state,
        "find_entities",
        "ContextA",
        Some(&primary.to_string_lossy()),
    )
    .expect("cold hydration");
    assert!(cold.is_complete(), "cold hydration must cover both roots");
    assert_eq!(cold.candidates_compiled, 2, "both owners must compile");

    let expected = BTreeSet::from([
        state.semantic_owner_path(&first.path().join("ContextA.cs").to_string_lossy()),
        state.semantic_owner_path(&second.path().join("ContextA.cs").to_string_lossy()),
    ]);
    assert_eq!(public_context_occurrence_files(&state, primary), expected);

    let warm = hydrate_workspace_index(
        &state,
        "find_entities",
        "ContextA",
        Some(&primary.to_string_lossy()),
    )
    .expect("warm hydration");
    assert!(warm.is_complete(), "warm hydration must remain complete");
    assert_eq!(warm.candidates_compiled, 0, "warm owners must stay current");
    assert_eq!(public_context_occurrence_files(&state, primary), expected);
}

#[cfg(feature = "dotnet")]
#[test]
fn issue_146_equivalent_cross_root_dbcontexts_keep_independent_occurrences() {
    let _serial = TEST_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _reset = ResetProjectSearch;
    for provider_available in [true, false] {
        run_cross_root_dbcontext_case(provider_available, false);
        run_cross_root_dbcontext_case(provider_available, true);
    }
}
