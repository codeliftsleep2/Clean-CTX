// Durable restore_context MCP handler.

use super::common::{ContentKind, checked_hierarchy_or_respond, contract_fields_for_hierarchy};
use crate::mcp::McpState;
use crate::mcp::compatibility::validator::{CompatibilityFailure, validate_current_context};
use crate::mcp::tool_helpers::inject_baseline_breakpoint;
use crate::protocol::send_response;
use serde_json::Value;

#[cfg(test)]
pub(super) mod historical_adoption_race_test_support {
    use std::sync::{Condvar, Mutex};

    #[derive(Default)]
    struct PauseState {
        owner: Option<String>,
        paused: bool,
        released: bool,
    }

    static STATE: Mutex<PauseState> = Mutex::new(PauseState {
        owner: None,
        paused: false,
        released: false,
    });
    static SIGNAL: Condvar = Condvar::new();

    pub(crate) fn arm(file_path: &str) {
        *STATE.lock().expect("historical adoption pause state") = PauseState {
            owner: Some(crate::dictionary::path::canonical_identity_key(file_path)),
            paused: false,
            released: false,
        };
    }

    pub(crate) fn pause_before_live_adoption(file_path: &str) {
        let owner = crate::dictionary::path::canonical_identity_key(file_path);
        let mut state = STATE.lock().expect("historical adoption pause state");
        if state.owner.as_deref() != Some(owner.as_str()) || state.paused {
            return;
        }
        state.paused = true;
        SIGNAL.notify_all();
        while !state.released {
            state = SIGNAL.wait(state).expect("historical adoption pause wait");
        }
        state.owner = None;
    }

    pub(crate) fn wait_until_paused() {
        let mut state = STATE.lock().expect("historical adoption pause state");
        while !state.paused {
            state = SIGNAL.wait(state).expect("historical adoption pause wait");
        }
    }

    pub(crate) fn release() {
        let mut state = STATE.lock().expect("historical adoption pause state");
        state.released = true;
        SIGNAL.notify_all();
    }
}

pub(crate) fn handle_restore_context(id: &Value, params: &Value, state: &McpState) {
    let requested = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if requested.is_empty() {
        return send_restore_error(id, "Missing required parameter: filePath");
    }

    let workspace_root = crate::mcp::tool_helpers::arg_str(params, "workspaceRoot");
    let requested = match resolve_durable_path_checked(
        requested,
        workspace_root,
        &state.config.additional_roots,
    ) {
        Ok(path) => path,
        Err(error) => return send_restore_error(id, &error),
    };

    let durable_path = state
        .alias_for_path(&requested)
        .and_then(|alias| state.persisted_path(&alias))
        .unwrap_or_else(|| state.durable_owner_path(&requested));
    if state.semantic_owner_path(&requested) != state.semantic_owner_path(&durable_path) {
        return send_restore_error(id, "Requested file does not match its durable identity");
    }
    let publication = state.begin_semantic_publication(&durable_path);

    if let Err(error) = state.recover_pending_edit(&durable_path) {
        return send_restore_error(id, &error);
    }

    // Validate every durable artifact before mutating live session state.
    let restored = {
        let guard = state.persistence_store_lock();
        let Some(store) = guard.as_ref() else {
            return send_restore_error(id, "Persistence is not enabled");
        };
        let Some(sqlite) = store.sqlite() else {
            return send_restore_error(id, "Persistence DB is unavailable");
        };
        match sqlite.load_durable_context(&durable_path, None) {
            Ok(Some(restored)) => restored,
            Ok(None) => return send_restore_error(id, "No persisted context for requested file"),
            Err(error) => return send_compatibility_error(id, &error),
        }
    };

    let source = match state.read_source(&durable_path) {
        Ok(source) => source,
        Err(error) => {
            return send_restore_error(id, &format!("Cannot read current source: {error}"));
        }
    };
    let required_fidelity = restored.fidelity;
    let compatible = match validate_current_context(
        restored,
        &source,
        std::path::Path::new(&durable_path),
        &state.config,
        required_fidelity,
    ) {
        Ok(compatible) => compatible,
        Err(error) => return send_compatibility_error(id, &error),
    };
    let canonical = compatible.canonical;
    let semantic = compatible.semantic;
    if checked_hierarchy_or_respond(id, &canonical.ir).is_none() {
        return;
    }
    let alias = state.get_or_create_alias(durable_path.clone());
    let mut session_ir = canonical.ir;
    session_ir.file_id.clone_from(&alias);
    let hierarchy = match checked_hierarchy_or_respond(id, &session_ir) {
        Some(hierarchy) => hierarchy,
        None => return,
    };
    let compact = || {
        super::content::presentation_document(&session_ir, &hierarchy, canonical.fidelity, state)
    };
    let source_matches =
        state.cache_read().compute_hash(source.as_bytes()) == canonical.source_hash;
    let tokenizer_kind = crate::mcp::tools::parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let economic = super::content::economical_presentation_document(
        &session_ir,
        &hierarchy,
        canonical.fidelity,
        &source,
        state,
        tokenizer_kind,
        tokenizer_box.as_deref(),
    );
    let selected_raw = matches!(
        economic.selected,
        crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
    );
    let (full, raw_passthrough) = if selected_raw && source_matches {
        (economic.text, true)
    } else {
        (compact(), false)
    };
    let edge_count = semantic.semantic_edges.len();

    #[cfg(test)]
    historical_adoption_race_test_support::pause_before_live_adoption(&durable_path);
    let published = publication.commit(
        // Compatibility validation above owns historical eligibility. The
        // ticket orders this restore operation, not the age of its snapshot.
        || Ok::<_, String>(true),
        || {
            state
                .ir_context_lock()
                .load_ir(session_ir.clone(), Some(canonical.source_hash.clone()));
            state.remember_persisted_path(&alias, &durable_path);
            state.remember_context_fidelity(&alias, canonical.fidelity);
            state.publish_compatible_semantic_projection(&alias, &durable_path, semantic);
            state
                .llm_text_cache_lock()
                .insert(alias.clone(), full.clone());
            Ok(())
        },
    );
    match published {
        Ok(Some(())) => {}
        Ok(None) => {
            return send_restore_error(
                id,
                "Historical restore was superseded by a newer same-owner authority operation; retry the request",
            );
        }
        Err(error) => return send_restore_error(id, &error),
    }
    let (content_kind, byte_exact) = contract_fields_for_hierarchy(canonical.fidelity, &hierarchy);

    let mut response = serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{ "type": "text", "text": full }],
            "ir": crate::ir::hierarchical::hierarchy_to_wire_reduced(&session_ir, &hierarchy),
            "_meta": {
                "version": session_ir.version, "restored": true,
                "file": durable_path,
                "instruction_count": session_ir.instructions.len(),
                "semantic_edge_count": edge_count,
                "content_kind": if raw_passthrough { ContentKind::RawPassthrough } else { content_kind },
                "byte_exact": if raw_passthrough {
                    serde_json::json!(["document"])
                } else {
                    serde_json::to_value(byte_exact).unwrap_or_default()
                }
            }
        }
    });
    inject_baseline_breakpoint(&mut response, state, &full);
    send_response(&response);
}

fn send_restore_error(id: &Value, message: &str) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        message,
        None,
    ));
}

fn send_compatibility_error(id: &Value, error: &CompatibilityFailure) {
    send_response(&crate::mcp::tool_helpers::jsonrpc_error(
        id.clone(),
        -32603,
        format!("Durable restore rejected: {error}"),
        Some(serde_json::json!({
            "reason": error.reason(),
            "component": error.component()
        })),
    ));
}

/// Admit a durable owner without requiring the source file to still exist.
/// Its containing directory must exist so symlinks and `..` are resolved.
fn resolve_durable_path_checked(
    path: &str,
    workspace_root: Option<&str>,
    additional_roots: &[String],
) -> Result<String, String> {
    use crate::mcp::tool_helpers::{resolve_file_path, resolve_file_path_checked};

    let resolved = resolve_file_path(path, workspace_root);
    if std::path::Path::new(&resolved).exists() {
        return resolve_file_path_checked(path, workspace_root, additional_roots);
    }
    let candidate = std::path::Path::new(&resolved);
    if candidate.symlink_metadata().is_ok() {
        return Err(format!(
            "path cannot be resolved safely within workspace root: {resolved}"
        ));
    }
    let parent = candidate
        .parent()
        .ok_or_else(|| format!("path has no containing directory: {resolved}"))?;
    let canonical_parent = parent
        .canonicalize()
        .map_err(|_| format!("path parent does not exist: {}", parent.display()))?;
    let trusted = resolve_file_path(workspace_root.unwrap_or("."), None);
    let canonical_root = std::path::Path::new(&trusted)
        .canonicalize()
        .map_err(|_| format!("workspace root does not exist: {trusted}"))?;
    let authorized = canonical_parent.starts_with(&canonical_root)
        || additional_roots.iter().any(|root| {
            std::path::Path::new(root)
                .canonicalize()
                .is_ok_and(|root| canonical_parent.starts_with(root))
        });
    if !authorized {
        return Err(format!(
            "path outside workspace root: {resolved} (workspace root: {})",
            canonical_root.display()
        ));
    }
    Ok(resolved)
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/durable_restore_compatibility.rs"]
mod compatibility_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/restore_path_admission.rs"]
mod path_admission_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../../../tests/mcp/historical_adoption_publication_race.rs"]
mod historical_adoption_publication_race_tests;
