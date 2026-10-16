// src/mcp/mod.rs
//
// MCP server module. Re-exports the top-level `run()` entry point.
//
// Sub-modules:
//   - server     : stdin/stdout read loop
//   - router     : JSON-RPC method dispatch
//   - handlers   : method handler implementations (initialize, tools/list, etc.)
//   - tools      : tool definitions and tools/call dispatch
//   - prompts    : prompt definitions and content
//   - workspace  : workspace-level operations (compress_workspace_dir, collect_source_files)
//   - state      : per-session state shared by all tool handlers (F-05)

pub(crate) mod buffered_store;
pub(crate) mod cache_hints;
pub(crate) mod compatibility;
pub(crate) mod content_economics;
pub(crate) mod context_store;
pub(crate) mod discovery_cache;
pub mod dispatcher;
mod handlers;
pub(crate) mod heuristics;
pub(crate) mod persistence_ir;
pub(crate) mod prompts;
pub(crate) mod proxy_stats;
mod router;
mod server;
pub(crate) mod session_stats;
pub(crate) mod sqlite_store;
pub(crate) mod state;
pub(crate) mod token_economics;
pub(crate) mod tool_annotations;
pub(crate) mod tool_dispatch;
pub(crate) mod tool_handlers;
pub(crate) mod tool_helpers;
pub(crate) mod tool_schemas;
pub(crate) mod tools;

pub use state::McpState;

/// Run the MCP server. This is the entry point called from `main()`.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    server::run()
}

// Regression and audit-fix tests compress src/main.rs and other .rs files,
// so they require the "rust" feature to be enabled.
#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/regression.rs"]
mod regression;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/audit_fixes.rs"]
mod audit_fixes;

// Black-box E2E tests that spawn the built binary and exchange JSON-RPC.
// Tests are `#[ignore]` by default (require `cargo build` first); run with
// `cargo test -- --ignored`.
#[cfg(test)]
#[path = "../tests/mcp/e2e_server.rs"]
mod e2e_server;

// apply_edit black-box E2E (docs/plans/APPLY_EDIT_PLAN.md Phase 4). Same
// spawn-the-binary model as `e2e_server`; `#[ignore]`d by default.
#[cfg(test)]
#[path = "../tests/mcp/apply_edit.rs"]
mod apply_edit_tests;

#[cfg(all(test, feature = "csharp", feature = "typescript"))]
#[path = "../tests/mcp/insert_before_declaration.rs"]
mod insert_before_declaration_tests;

#[cfg(all(test, feature = "csharp", feature = "typescript"))]
#[path = "../tests/mcp/apply_edit_issue_136.rs"]
mod apply_edit_issue_136_tests;

#[cfg(all(test, feature = "csharp"))]
#[path = "../tests/mcp/semantic_signature_evidence.rs"]
mod semantic_signature_evidence_tests;

#[cfg(test)]
#[path = "../tests/mcp/issue_137_diff_commits.rs"]
mod issue_137_diff_commits_tests;

#[cfg(all(test, feature = "csharp", feature = "dotnet"))]
#[path = "../tests/mcp/di_omission.rs"]
mod di_omission_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/decorated_owner_identity.rs"]
mod decorated_owner_identity_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/framework_declaration_publication.rs"]
mod framework_declaration_publication_tests;
#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/semantic_dependency_lifecycle_issue_139.rs"]
mod semantic_dependency_lifecycle_issue_139_tests;
#[cfg(test)]
#[path = "../tests/mcp/workspace_query_issue_139.rs"]
mod workspace_query_issue_139_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_di_pipe.rs"]
mod angular_di_pipe_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_di_inject_token.rs"]
mod angular_di_inject_token_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_di_inject_token_matrix.rs"]
mod angular_di_inject_token_matrix_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_declaration.rs"]
mod ngrx_legacy_declaration_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_declaration_matrix.rs"]
mod ngrx_legacy_declaration_matrix_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_selector_declaration.rs"]
mod ngrx_selector_declaration_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_direct_named_action_refinement.rs"]
mod ngrx_direct_named_action_refinement_tests;
#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_direct_named_selector_refinement.rs"]
mod ngrx_direct_named_selector_refinement_tests;
#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_modern_service_dispatch_ownership.rs"]
mod ngrx_modern_service_dispatch_ownership_tests;
#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_one_hop_reexport_refinement.rs"]
mod ngrx_one_hop_reexport_refinement_tests;
#[cfg(all(test, feature = "angular", feature = "typescript"))]
#[path = "../tests/mcp/ngrx_selector_reference_binding.rs"]
mod ngrx_selector_reference_binding_tests;

#[cfg(all(test, feature = "angular", feature = "typescript"))]
#[path = "../tests/mcp/ngrx_store_receiver_qualification.rs"]
mod ngrx_store_receiver_qualification_tests;

#[cfg(all(test, feature = "angular", feature = "typescript"))]
#[path = "../tests/mcp/ngrx_service_select_ownership.rs"]
mod ngrx_service_select_ownership_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/workspace_query_declarations_in_file.rs"]
mod workspace_query_declarations_in_file_tests;

// Persistent-process scenarios are isolated from the in-process/schema
// contract suite and remain ignored unless the binary has been built first.
#[cfg(test)]
#[path = "../tests/mcp/apply_edit_e2e.rs"]
mod apply_edit_e2e_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_transition.rs"]
mod ngrx_legacy_transition_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_discriminant.rs"]
mod ngrx_legacy_discriminant_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_object_dispatch.rs"]
mod ngrx_legacy_object_dispatch_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_legacy_transition_matrix.rs"]
mod ngrx_legacy_transition_matrix_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_constructor_dispatch.rs"]
mod ngrx_constructor_dispatch_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_constructor_dispatch_matrix.rs"]
mod ngrx_constructor_dispatch_matrix_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_service_dispatch.rs"]
mod ngrx_service_dispatch_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/ngrx_service_dispatch_matrix.rs"]
mod ngrx_service_dispatch_matrix_tests;
