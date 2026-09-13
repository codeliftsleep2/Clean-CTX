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

        let meta_results = registry.run_meta_layers_pipeline_with_path(
            &state.source,
            path,
            &class_captures,
            state.fidelity,
            None,
        );
        append_marker_aliases(state, &meta_results);

        let mut semantic_edges = registry.collect_semantic_edges_with_path(
            &state.source,
            path,
            &class_entries,
            state.fidelity,
            None,
        );
        for edge in &mut semantic_edges {
            if edge.subject.file.is_none() {
                edge.subject.file = Some(file_provenance.clone());
            }
            if edge.object.file.is_none() {
                edge.object.file = Some(file_provenance.clone());
            }
        }
        state.semantic_edges.extend(semantic_edges);
        Ok(())
    }
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
        for line in output.rendered.lines() {
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
