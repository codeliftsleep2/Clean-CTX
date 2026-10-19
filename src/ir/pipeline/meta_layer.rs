//! Framework meta-layer orchestration pass.
//!
//! This module owns only the existing capture projection, marker dispatch,
//! semantic-edge collection, and file-provenance attachment concern.

use std::path::Path;

use super::{IRPass, PassContext, PassError};
use crate::ir::opcodes::CoreOp;

/// Pass 3: framework-specific meta-layer processing.
pub struct MetaLayerPass;

impl MetaLayerPass {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MetaLayerPass {
    fn default() -> Self {
        Self::new()
    }
}

impl IRPass for MetaLayerPass {
    fn name(&self) -> &str {
        "meta_layer"
    }

    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let class_entries = class_entries(state);
        let class_captures: Vec<String> =
            class_entries.iter().map(|(_, text)| text.clone()).collect();
        let file_provenance = state
            .canonical_path
            .clone()
            .unwrap_or_else(|| state.file_id.clone());
        let path = Path::new(&file_provenance);
        let registry = crate::layers::LayerRegistry::global();
        let lexical_regions = crate::meta_util::LexicalRegions::new(&state.source);
        let structured_declarations: Vec<(String, String)> = state
            .captures
            .iter()
            .filter(|capture| {
                matches!(
                    capture.name.as_str(),
                    "class.root" | "interface.root" | "struct.root" | "enum.root" | "record.root"
                )
            })
            .map(|capture| (capture.name.clone(), capture.text.clone()))
            .collect();
        let structured_class_bases = if state.query_string == crate::queries::CS_QUERY {
            structured_csharp_class_bases(&state.captures)
        } else {
            Vec::new()
        };
        #[cfg(all(feature = "csharp", feature = "dotnet"))]
        let csharp_resolved_classes = if state.query_string == crate::queries::CS_QUERY {
            crate::ir::csharp_resolution::resolve_compilation_class_ancestry(
                &state.captures,
                &file_provenance,
                &crate::dotnet_meta::semantic::known_csharp_type_identities(),
                &state.instructions,
            )
        } else {
            Vec::new()
        };
        #[cfg(all(feature = "csharp", not(feature = "dotnet")))]
        let csharp_resolved_classes = Vec::new();
        let mut context = crate::layers::meta::MetaLayerContext::new(
            &state.source,
            path,
            &class_captures,
            &class_entries,
            state.fidelity,
            state.config.as_ref(),
            &lexical_regions,
        );
        let invocation_set = crate::ir::invocation_evidence::InvocationEvidenceSet::new(
            file_provenance.clone(),
            &state.source,
            state.invocation_occurrences.clone(),
        );
        let invocation_view = invocation_set
            .view(&file_provenance, &state.source)
            .expect("compilation-local evidence matches its source");
        // Some(empty) is authoritative: never reparse a production source.
        context.invocation_evidence = Some(&invocation_view);
        if state.query_string == crate::queries::TS_QUERY {
            context.angular_initializers = Some(&state.angular_initializers);
            context.ngrx_class_actions = Some(&state.ngrx_class_actions);
            context.ngrx_selector_declarations = Some(&state.ngrx_selector_declarations);
            context.ngrx_legacy_transitions = Some(&state.ngrx_legacy_transitions);
            context.ngrx_legacy_discriminants = Some(&state.ngrx_legacy_discriminants);
            context.ngrx_constructor_dispatches = Some(&state.ngrx_constructor_dispatches);
            context.ngrx_service_dispatches = Some(&state.ngrx_service_dispatches);
        }
        if state.query_string == crate::queries::CS_QUERY {
            context.structured_declarations = Some(&structured_declarations);
            context.structured_class_bases = Some(&structured_class_bases);
            #[cfg(feature = "csharp")]
            {
                context.csharp_resolved_classes = Some(&csharp_resolved_classes);
            }
        }

        #[cfg(all(feature = "csharp", feature = "dotnet"))]
        if state.query_string == crate::queries::CS_QUERY {
            state.instructions.extend(
                crate::dotnet_meta::semantic::project_resolved_signalr_execution_contexts(
                    &csharp_resolved_classes,
                    &file_provenance,
                ),
            );
        }

        let (meta_results, mut semantic_edges, semantic_dependencies) =
            registry.evaluate_meta_layers_context(&context);
        append_marker_aliases(state, &meta_results);
        for edge in &mut semantic_edges {
            if edge.subject.file.is_none() {
                edge.subject.file = Some(file_provenance.clone());
            }
            if edge.object.file.is_none() {
                edge.object.file = Some(file_provenance.clone());
            }
        }
        state.semantic_edges.extend(semantic_edges);
        state.semantic_dependencies.extend(semantic_dependencies);
        Ok(())
    }
}

fn structured_csharp_class_bases(
    captures: &[crate::compression::capture_pipeline::CapEntry],
) -> Vec<crate::layers::meta::StructuredClassBase> {
    const TYPE_ROOTS: [&str; 5] = [
        "class.root",
        "interface.root",
        "struct.root",
        "enum.root",
        "record.root",
    ];
    captures
        .iter()
        .filter(|capture| capture.name == "class.root")
        .filter_map(|owner| {
            let first_base = captures
                .iter()
                .filter(|candidate| {
                    candidate.name == "csharp.base_type"
                        && captures
                            .iter()
                            .filter(|possible_owner| {
                                TYPE_ROOTS.contains(&possible_owner.name.as_str())
                                    && possible_owner.start_byte <= candidate.start_byte
                                    && candidate.end_byte <= possible_owner.end_byte
                            })
                            .min_by_key(|possible_owner| {
                                possible_owner.end_byte - possible_owner.start_byte
                            })
                            .is_some_and(|nearest| {
                                nearest.start_byte == owner.start_byte
                                    && nearest.end_byte == owner.end_byte
                            })
                })
                .min_by_key(|candidate| candidate.start_byte)?;
            let written = first_base.raw_text.trim();
            (!owner.text.is_empty() && !written.is_empty()).then(|| {
                crate::layers::meta::StructuredClassBase {
                    owner: owner.text.clone(),
                    written: written.to_string(),
                }
            })
        })
        .collect()
}

fn class_entries(state: &PassContext) -> Vec<(String, String)> {
    state
        .captures
        .iter()
        .filter(|capture| {
            matches!(
                capture.name.as_str(),
                "class.root"
                    | "interface.root"
                    | "struct.root"
                    | "enum.root"
                    | "trait.root"
                    | "record.root"
                    | "impl.root"
            )
        })
        .map(|capture| {
            (
                capture.name.clone(),
                crate::meta_util::class_source_from_capture(&state.source, capture).to_string(),
            )
        })
        .collect()
}

fn append_marker_aliases(
    state: &mut PassContext,
    outputs: &[crate::layers::meta::MetaLayerOutput],
) {
    for output in outputs {
        // Structured marker entries keep domain-owned multiline values intact.
        let markers = if let Some(block) = &output.angular_block {
            block
                .lines
                .iter()
                .chain(block.sections.iter().flat_map(|section| &section.lines))
                .map(String::as_str)
                .collect::<Vec<_>>()
        } else {
            output.rendered.lines().collect::<Vec<_>>()
        };
        for line in markers {
            let line = line.trim();
            if line.is_empty() || !line.starts_with('Φ') {
                continue;
            }
            let content = line.strip_prefix('Φ').unwrap_or(line);
            if let Some((prefix, text)) = content.split_once(':') {
                state
                    .instructions
                    .push(CoreOp::TypeAlias(format!("@{prefix}"), text.to_string()));
            }
        }
    }
}

#[cfg(all(test, feature = "angular"))]
#[path = "../../tests/ir/pipeline_meta_layer.rs"]
mod tests;
