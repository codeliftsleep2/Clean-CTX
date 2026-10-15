//! Independent top-level NgRx Selector declarations; no reference resolution.
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::SourceSpan;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::BTreeMap;

pub(crate) const QUERY: &str = r#"
(program (import_statement
    (import_clause (named_imports
        (import_specifier name: (identifier) @ngrx.selector.import_name !alias)))
    source: (string . (string_fragment) @ngrx.selector.import_source .)) @ngrx.selector.import)
(program (import_statement (import_clause (named_imports
    (import_specifier alias: (identifier) @ngrx.selector.alias_binding)))))
(program (export_statement declaration:
    (lexical_declaration kind: "const"
        (variable_declarator
            name: (identifier) @ngrx.selector.name
            value: (call_expression
                function: (identifier) @ngrx.selector.factory
                arguments: (arguments) @ngrx.selector.arguments) @ngrx.selector.initializer)
        @ngrx.selector.declaration)))
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorDeclarationEvidence {
    pub file: String,
    pub source_hash: String,
    pub name: String,
    pub declaration_span: SourceSpan,
    pub name_span: SourceSpan,
    pub initializer_span: SourceSpan,
    pub factory_span: SourceSpan,
    pub import_span: SourceSpan,
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("parser span is ordered")
}

pub(crate) fn collect(
    source: &str,
    canonical_path: Option<&str>,
    captures: &[CapturedNode],
) -> Vec<SelectorDeclarationEvidence> {
    let Some(file) = canonical_path.filter(|path| !path.is_empty()) else {
        return Vec::new();
    };
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    for capture in captures
        .iter()
        .filter(|capture| capture.name.starts_with("ngrx.selector."))
    {
        matches
            .entry(capture.match_index)
            .or_default()
            .insert(&capture.name, capture);
    }
    if captures.iter().any(|capture| {
        capture.name == "ngrx.selector.alias_binding" && is_selector_factory(&capture.raw_text)
    }) {
        return Vec::new();
    }
    let imports: Vec<_> = matches
        .values()
        .filter_map(|matched| {
            let name = matched.get("ngrx.selector.import_name")?;
            is_selector_factory(&name.raw_text).then(|| {
                (
                    name.raw_text.as_str(),
                    matched.get("ngrx.selector.import_source").copied(),
                    matched.get("ngrx.selector.import").copied(),
                )
            })
        })
        .collect();
    let source_hash = crate::cache::content_hash(source.as_bytes());
    matches
        .values()
        .filter_map(|matched| {
            let name = matched.get("ngrx.selector.name")?;
            let factory = matched.get("ngrx.selector.factory")?;
            let declaration = matched.get("ngrx.selector.declaration")?;
            let initializer = matched.get("ngrx.selector.initializer")?;
            if !is_selector_factory(&factory.raw_text) {
                return None;
            }
            let factory_imports: Vec<_> = imports
                .iter()
                .filter(|(imported, module, _)| {
                    *imported == factory.raw_text
                        && module.is_some_and(|module| module.raw_text == "@ngrx/store")
                })
                .collect();
            let [(_, _, Some(import))] = factory_imports.as_slice() else {
                return None;
            };
            Some(SelectorDeclarationEvidence {
                file: file.into(),
                source_hash: source_hash.clone(),
                name: name.raw_text.clone(),
                declaration_span: span(declaration),
                name_span: span(name),
                initializer_span: span(initializer),
                factory_span: span(factory),
                import_span: span(import),
            })
        })
        .collect()
}

fn is_selector_factory(name: &str) -> bool {
    matches!(name, "createSelector" | "createFeatureSelector")
}

impl SelectorDeclarationEvidence {
    pub(crate) fn edge(&self) -> Option<SemanticEdge> {
        if self.file.is_empty()
            || self.source_hash.is_empty()
            || self.declaration_span.start_byte() > self.name_span.start_byte()
            || self.name_span.end_byte() > self.declaration_span.end_byte()
            || self.declaration_span.start_byte() > self.initializer_span.start_byte()
            || self.initializer_span.end_byte() > self.declaration_span.end_byte()
            || self.initializer_span.start_byte() > self.factory_span.start_byte()
            || self.factory_span.end_byte() > self.initializer_span.end_byte()
            || self.import_span.start_byte() >= self.import_span.end_byte()
        {
            return None;
        }
        let entity = EntityRef::new("ngrx", "Selector", &self.name);
        Some(SemanticEdge {
            relation: SemanticRelation::Defines,
            subject: entity.clone(),
            object: entity,
            layer: "ngrx",
            call_evidence: None,
        })
    }
}
