//! Borrowed, compilation-scoped evidence shared by meta-layers.

use std::path::Path;

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::layers::meta::semantic::SemanticEdge;
use crate::meta_util::LexicalRegions;

use super::MetaLayerOutput;

/// Marker and semantic results produced by one applicable meta-layer.
#[derive(Debug, Default)]
pub struct MetaLayerEvaluation {
    pub output: Option<MetaLayerOutput>,
    pub semantic_edges: Vec<SemanticEdge>,
}

/// Immutable inputs and shared lexical evidence for one meta-layer pass.
pub struct MetaLayerContext<'a> {
    pub source: &'a str,
    pub path: &'a Path,
    pub class_captures: &'a [String],
    pub paired_class_captures: &'a [(String, String)],
    pub fidelity: Fidelity,
    pub config: Option<&'a CleanCtxConfig>,
    pub lexical_regions: &'a LexicalRegions,
}

impl<'a> MetaLayerContext<'a> {
    pub fn new(
        source: &'a str,
        path: &'a Path,
        class_captures: &'a [String],
        paired_class_captures: &'a [(String, String)],
        fidelity: Fidelity,
        config: Option<&'a CleanCtxConfig>,
        lexical_regions: &'a LexicalRegions,
    ) -> Self {
        Self {
            source,
            path,
            class_captures,
            paired_class_captures,
            fidelity,
            config,
            lexical_regions,
        }
    }
}
