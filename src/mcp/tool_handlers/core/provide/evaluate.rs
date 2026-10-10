//! Authoritative evaluation of one `provide_code_context` item.

use super::outcome::{ProvideFailure, ProvideResult, ProvidedContext};
use crate::mcp::McpState;
use crate::mcp::tool_helpers::{
    compile_file_ir_focused, count_tokens_with_tokenizer, resolve_file_path_checked,
};
use crate::mcp::tools::parse_tokenizer_arg;
use serde_json::{Value, json};
use std::collections::HashSet;

fn raw_token_cache_key(source_hash: &str, tokenizer: crate::tokenizer::TokenizerKind) -> String {
    format!("{source_hash}::{tokenizer}")
}

fn cached_raw_token_count(
    state: &McpState,
    source_hash: &str,
    tokenizer_kind: crate::tokenizer::TokenizerKind,
    source: &str,
    tokenizer: Option<&dyn crate::tokenizer::Tokenizer>,
) -> usize {
    let cache_key = raw_token_cache_key(source_hash, tokenizer_kind);
    if let Some(count) = state.cache_read().get_raw_token_count(&cache_key) {
        return count;
    }
    let count = count_tokens_with_tokenizer(source, tokenizer);
    state.cache_write().store_raw_token_count(&cache_key, count);
    count
}

pub(super) fn evaluate(params: &Value, state: &McpState) -> ProvideResult {
    use std::time::Instant;
    let overall_start = Instant::now();
    let focus_methods: Option<HashSet<String>> =
        params["arguments"]["focusMethods"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|value| value.as_str().map(String::from))
                .collect()
        });
    let explicit_fidelity = params["arguments"]["fidelity"].as_str();
    let explicit_intent = params["arguments"]["intent"].as_str();
    let focus_was_supplied = focus_methods.is_some();
    let focus_is_empty = focus_methods.as_ref().is_some_and(HashSet::is_empty);

    if focus_was_supplied {
        let fidelity_conflicts = explicit_fidelity.is_some_and(|value| value != "edit");
        let intent_conflicts = explicit_intent.is_some_and(|value| value != "edit");
        let empty_without_explicit_edit =
            focus_is_empty && explicit_fidelity != Some("edit") && explicit_intent != Some("edit");
        if fidelity_conflicts || intent_conflicts || empty_without_explicit_edit {
            return Err(ProvideFailure::invalid(
                "focusMethods requires Edit fidelity; use fidelity \"edit\" or intent \"edit\". A non-empty focus may imply Edit only when neither fidelity nor intent is supplied.",
            ));
        }
    }

    let inferred_focus_fidelity = (focus_methods
        .as_ref()
        .is_some_and(|focus| !focus.is_empty())
        && explicit_fidelity.is_none()
        && explicit_intent.is_none())
    .then_some("edit");
    let file_path_str = crate::mcp::tool_helpers::arg_str_or_empty(params, "filePath");
    if file_path_str.is_empty() {
        return Err(ProvideFailure::invalid(
            "Missing required parameter: filePath",
        ));
    }
    let workspace_root = crate::mcp::tool_helpers::arg_str(params, "workspaceRoot");
    let resolved_path = resolve_file_path_checked(
        file_path_str,
        workspace_root,
        &state.config.additional_roots,
    )
    .map_err(ProvideFailure::invalid)?;
    let semantic_owner = state.semantic_owner_path(&resolved_path);
    let durable_owner = state.durable_owner_path(&resolved_path);
    if state.config.is_excluded(&resolved_path) {
        return Err(ProvideFailure::internal(format!(
            "File excluded by config: {file_path_str}"
        )));
    }
    let publication = state.begin_semantic_publication(&semantic_owner);
    state
        .preflight_semantic_publication(&resolved_path)
        .map_err(ProvideFailure::internal)?;
    let limits = &state.config.resource_limits;
    if let Ok(metadata) = std::fs::metadata(&resolved_path) {
        limits
            .check_file_size(metadata.len())
            .map_err(ProvideFailure::internal)?;
    }

    let source_arc = state
        .read_source(&resolved_path)
        .map_err(|error| ProvideFailure::internal(format!("Cannot read file: {error}")))?;
    let source = source_arc.as_str();
    let alias = state.get_or_create_alias(semantic_owner.clone());

    #[cfg(feature = "angular")]
    if let Some(result) = super::super::provide_angular::try_evaluate_angular_template(
        params,
        state,
        &resolved_path,
        source,
    ) {
        return Ok(result);
    }

    state.remember_persisted_path(&alias, &resolved_path);
    let heuristics_start = Instant::now();
    let stored_fidelity = state.context_fidelity(&alias);
    let ir_read = state.ir_context_read();
    let decision = crate::mcp::heuristics::decide(
        &resolved_path,
        explicit_fidelity.or(inferred_focus_fidelity),
        explicit_intent,
        &state.config,
        &ir_read,
        source,
        Some(&alias),
        stored_fidelity,
    )
    .map_err(ProvideFailure::invalid)?;
    drop(ir_read);
    let heuristics_ms = heuristics_start.elapsed().as_millis() as u64;

    let effective_fidelity = decision.fidelity;
    let edit_focus = (effective_fidelity == crate::compression::Fidelity::Edit)
        .then_some(focus_methods.as_ref())
        .flatten();
    let (content_kind, byte_exact) =
        super::super::common::contract_fields_focused(effective_fidelity, focus_methods.as_ref());
    let is_angular = decision.is_angular;
    let tokenizer_kind = parse_tokenizer_arg(params, &state.config);
    let tokenizer_box = crate::tokenizer::create_tokenizer(tokenizer_kind).ok();
    let tokenizer_ref: Option<&dyn crate::tokenizer::Tokenizer> = tokenizer_box.as_deref();
    let source_hash = state.cache_read().compute_hash(source.as_bytes());
    let raw_tokens =
        cached_raw_token_count(state, &source_hash, tokenizer_kind, source, tokenizer_ref);

    if effective_fidelity == crate::compression::Fidelity::Verbatim {
        let full = source.to_string();
        state.record_compression(
            &resolved_path,
            raw_tokens,
            raw_tokens,
            "verbatim",
            is_angular,
            "full",
            None,
            "verbatim",
        );
        return Ok(ProvidedContext::new(
            full,
            json!({
                "strategy": "full", "fidelity": "verbatim",
                "decision_summary": decision.summary(),
                "content_kind": super::super::common::ContentKind::VerbatimDocument,
                "byte_exact": ["document"], "degradation": null, "verbatim": true
            }),
        ));
    }

    let mut te_prediction = "bypass";
    let mut te_threshold = 0usize;
    if effective_fidelity == crate::compression::Fidelity::Edit {
        let extension = std::path::Path::new(&resolved_path)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("");
        let prediction = crate::mcp::token_economics::should_attempt_compression(
            raw_tokens,
            effective_fidelity,
            extension,
        );
        te_prediction = if prediction {
            "favorable"
        } else {
            "unfavorable"
        };
        te_threshold =
            crate::mcp::token_economics::compression_threshold(effective_fidelity, extension);
    }

    let _span = tracing::info_span!(
        "provide_code_context",
        file_path = %resolved_path,
        fidelity = %format!("{:?}", effective_fidelity),
        strategy = "full",
        cbm_status = %state.cbm_status.summary(),
        is_angular = %is_angular,
        prediction = %te_prediction,
        threshold = %te_threshold,
    )
    .entered();

    let compile_start = Instant::now();
    let cached_ir = if state.context_fidelity(&alias) == Some(effective_fidelity) {
        let ir_context = state.ir_context_read();
        if ir_context.is_source_unchanged(&alias, &source_hash) {
            ir_context
                .get_ir(&alias)
                .cloned()
                .zip(ir_context.file_version(&alias))
        } else {
            None
        }
    } else {
        None
    };
    let ir_result = if let Some((instructions, version)) = cached_ir {
        state.semantic_edges(&alias).map(|semantic_edges| {
            super::super::common::compiled_from_tuples(alias.clone(), version, instructions)
                .map(|ir| (ir, semantic_edges, source_hash.clone()))
        })
    } else {
        None
    }
    .unwrap_or_else(|| {
        compile_file_ir_focused(&resolved_path, effective_fidelity, state, None)
            .map_err(|error| error.to_string())
    });
    let compile_ms = compile_start.elapsed().as_millis() as u64;

    let (mut ir, semantic_edges, source_hash) = match ir_result {
        Ok(compiled) => compiled,
        Err(error) => {
            tracing::warn!(
                error = %error,
                path = %resolved_path,
                "IR compilation failed in provide_code_context; returning structured ir_unavailable error"
            );
            return Err(ProvideFailure::new(
                -32603,
                format!(
                    "IR compilation unavailable for {resolved_path}: {error}. SCHEMA-vNext structural output cannot be produced for this input; retry with fidelity \"verbatim\" for the raw, uncompressed source or read the source directly."
                ),
                Some(json!({
                    "reason": "ir_unavailable",
                    "path": resolved_path,
                    "ir_compiler": error,
                })),
            ));
        }
    };

    let render_start = Instant::now();
    let hierarchy = resolve_focus(&mut ir, edit_focus)?;
    let candidate =
        super::super::content::presentation_document(&ir, &hierarchy, effective_fidelity, state);
    let economic = super::super::content::select_complete_content(
        source,
        candidate,
        edit_focus.is_some(),
        tokenizer_kind,
        tokenizer_ref,
        raw_tokens,
    );
    let raw_tokens = economic.raw_tokens;
    let candidate_tokens = economic.candidate_tokens;
    let selected_tokens = economic.selected_tokens();
    let saved_tokens = economic.saved_tokens();
    let raw_passthrough = matches!(
        economic.selected,
        crate::mcp::content_economics::SelectedRepresentation::RawPassthrough
    );
    #[cfg(test)]
    super::publication_race_test_support::pause_after_compile(&resolved_path);
    let canonical_path = semantic_owner;
    let expected_source_hash = source_hash.clone();
    let published = publication
        .commit(
            || {
                Ok::<_, &'static str>(state.read_source(&resolved_path).is_ok_and(|current| {
                    state.cache_read().compute_hash(current.as_bytes()) == expected_source_hash
                }))
            },
            || {
                super::super::provide_persistence::persist_read_baseline(
                    state,
                    &durable_owner,
                    effective_fidelity,
                    &ir,
                    &semantic_edges,
                    &source_hash,
                    source,
                    raw_tokens,
                    candidate_tokens,
                )?;
                state
                    .ir_context_lock()
                    .load_ir(ir.clone(), Some(source_hash.clone()));
                state.remember_context_fidelity(&ir.file_id, effective_fidelity);
                {
                    let mut index = state.workspace_index_lock();
                    match crate::workspace::index::SemanticFidelity::from_compilation(
                        effective_fidelity,
                    ) {
                        Some(semantic_fidelity) => index.replace_semantic_projection(
                            &canonical_path,
                            semantic_edges.clone(),
                            semantic_fidelity,
                            source_hash,
                        ),
                        None => {
                            index.remove_file(&canonical_path);
                            index.add_edges(&canonical_path, semantic_edges.clone());
                        }
                    }
                }
                state.remember_semantic_edges(&ir.file_id, semantic_edges);
                state
                    .llm_text_cache_lock()
                    .insert(ir.file_id.clone(), economic.text.clone());
                Ok(())
            },
        )
        .map_err(ProvideFailure::internal)?;
    if published.is_none() {
        return Err(ProvideFailure::internal(
            "Context publication was superseded by a newer same-file source snapshot; retry the request",
        ));
    }
    let render_ms = render_start.elapsed().as_millis() as u64;
    state.record_compression(
        &canonical_path,
        raw_tokens,
        selected_tokens,
        &format!("{:?}", effective_fidelity).to_lowercase(),
        is_angular,
        "full",
        None,
        if raw_passthrough {
            "raw_passthrough"
        } else {
            "ir_compression"
        },
    );
    let visible_content_kind = if raw_passthrough {
        super::super::common::ContentKind::RawPassthrough
    } else {
        content_kind
    };
    let visible_byte_exact = if raw_passthrough {
        json!(["document"])
    } else {
        serde_json::to_value(byte_exact).unwrap_or_default()
    };
    let visible_text = economic.text;
    let result = ProvidedContext::new(
        visible_text,
        json!({
            "version": ir.version,
            "strategy": "full",
            "fidelity": format!("{:?}", effective_fidelity).to_lowercase(),
            "is_angular": is_angular,
            "decision_summary": decision.summary(),
            "content_kind": visible_content_kind,
            "byte_exact": visible_byte_exact,
            "degradation": null
        }),
    );
    let total_ms = overall_start.elapsed().as_millis() as u64;
    tracing::info!(
        heuristics_ms,
        compile_ms,
        render_ms,
        total_ms,
        raw_tokens,
        comp_tokens = selected_tokens,
        savings_pct = if raw_tokens > 0 {
            (saved_tokens as f64 / raw_tokens as f64 * 100.0) as u64
        } else {
            0
        },
        "provide_code_context full complete"
    );
    Ok(result)
}

fn resolve_focus(
    ir: &mut crate::ir::compiler::CompiledIR,
    focus: Option<&HashSet<String>>,
) -> Result<crate::ir::hierarchical::HierarchicalIR, ProvideFailure> {
    let hierarchy = crate::ir::hierarchical::try_ir_to_hierarchical(ir)
        .map_err(|error| ProvideFailure::projection(&error))?;
    let Some(selectors) = focus else {
        return Ok(hierarchy);
    };
    let method_ids = crate::ir::focus::resolve_focus_method_ids(&hierarchy, selectors)
        .map_err(|error| ProvideFailure::invalid(error.to_string()))?;
    crate::ir::focus::retain_focused_bodies(ir, &method_ids);
    crate::ir::hierarchical::try_ir_to_hierarchical(ir)
        .map_err(|error| ProvideFailure::projection(&error))
}
