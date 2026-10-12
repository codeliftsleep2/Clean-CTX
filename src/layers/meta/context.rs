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
    /// Parser-owned declaration identifiers when a language provides them.
    pub structured_declarations: Option<&'a [(String, String)]>,
    pub fidelity: Fidelity,
    pub config: Option<&'a CleanCtxConfig>,
    pub lexical_regions: &'a LexicalRegions,
    /// Compiler-owned evidence; absence means no upstream parse was supplied.
    pub(crate) invocation_evidence:
        Option<&'a crate::ir::invocation_evidence::InvocationEvidenceView<'a>>,
    pub(crate) angular_initializers:
        Option<&'a [crate::angular_meta::inject_evidence::ClassInitializerEvidence]>,
    pub(crate) ngrx_class_actions:
        Option<&'a [crate::angular_meta::ngrx::legacy::ClassActionEvidence]>,
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
            structured_declarations: None,
            fidelity,
            config,
            lexical_regions,
            invocation_evidence: None,
            angular_initializers: None,
            ngrx_class_actions: None,
        }
    }
}
