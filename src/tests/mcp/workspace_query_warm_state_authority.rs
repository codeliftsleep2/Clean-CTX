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
