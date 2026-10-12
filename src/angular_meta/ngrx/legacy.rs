//! Independent, top-level class Action declarations; no reference resolution.
use crate::compression::capture_pipeline::CapturedNode;
use crate::ir::invocation_evidence::SourceSpan;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) const QUERY: &str = r#"
    (program (import_statement
        (import_clause (named_imports
            (import_specifier name: (identifier) @ngrx.legacy.import_name !alias)))
        source: (string . (string_fragment) @ngrx.legacy.import_source .)) @ngrx.legacy.import)
    (program (import_statement (import_clause (named_imports
        (import_specifier alias: (identifier) @ngrx.legacy.alias_binding)))))
    (program [
        (class_declaration (class_heritage (implements_clause
            (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        (abstract_class_declaration (class_heritage (implements_clause
            (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        (export_statement declaration: [
            (class_declaration (class_heritage (implements_clause
                (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
            (abstract_class_declaration (class_heritage (implements_clause
                (type_identifier) @ngrx.legacy.interface))) @ngrx.legacy.owner
        ])
    ])
    [(class_declaration type_parameters: (type_parameters
        (type_parameter name: (type_identifier) @ngrx.legacy.shadow))) @ngrx.legacy.shadow_owner
     (abstract_class_declaration type_parameters: (type_parameters
        (type_parameter name: (type_identifier) @ngrx.legacy.shadow))) @ngrx.legacy.shadow_owner]
    (program [
        (interface_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (type_alias_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (abstract_class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
        (enum_declaration name: (identifier) @ngrx.legacy.local_binding)
        (export_statement declaration: [
            (interface_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (type_alias_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (abstract_class_declaration name: (type_identifier) @ngrx.legacy.local_binding)
            (enum_declaration name: (identifier) @ngrx.legacy.local_binding)
        ])
    ])
"#;

#[derive(Debug, Clone)]
pub(crate) struct ClassActionEvidence {
    pub name: String,
    pub class_span: SourceSpan,
    pub implements_span: SourceSpan,
    pub import_span: SourceSpan,
}

fn span(capture: &CapturedNode) -> SourceSpan {
    SourceSpan::new(capture.start_byte, capture.end_byte).expect("parser span is ordered")
}

/// Same-match import and implements evidence, joined to canonical #139 class
/// identity by exact declaration span. Nested/aliased/qualified forms are not
/// resolved. The existing framework applicability gate remains unchanged.
pub(crate) fn collect(source: &str, captures: &[CapturedNode]) -> Vec<ClassActionEvidence> {
    if !super::has_ngrx_imports(source) {
        return Vec::new();
    }
    let mut matches: BTreeMap<usize, BTreeMap<&str, &CapturedNode>> = BTreeMap::new();
    for capture in captures
        .iter()
        .filter(|c| c.name.starts_with("ngrx.legacy."))
    {
        matches
            .entry(capture.match_index)
            .or_default()
            .insert(&capture.name, capture);
    }
    if captures.iter().any(|capture| {
        matches!(
            capture.name.as_str(),
            "ngrx.legacy.local_binding" | "ngrx.legacy.alias_binding"
        ) && capture.raw_text == "Action"
    }) {
        return Vec::new();
    }
    let imports: Vec<_> = matches
        .values()
        .filter_map(|matched| {
            let name = matched.get("ngrx.legacy.import_name")?;
            (name.raw_text == "Action").then(|| {
                (
                    matched.get("ngrx.legacy.import_source").copied(),
                    matched.get("ngrx.legacy.import").copied(),
                )
            })
        })
        .collect();
    let [(Some(module), Some(import))] = imports.as_slice() else {
        return Vec::new();
    };
    if module.raw_text != "@ngrx/store" {
        return Vec::new();
    }
    let owners: BTreeMap<_, _> = captures
        .iter()
        .filter(|c| c.name == "class.root")
        .map(|c| ((c.start_byte, c.end_byte), c.text.as_str()))
        .collect();
    let shadowed: BTreeSet<_> = matches
        .values()
        .filter_map(|matched| {
            let parameter = matched.get("ngrx.legacy.shadow")?;
            let owner = matched.get("ngrx.legacy.shadow_owner")?;
            (parameter.raw_text == "Action").then_some((owner.start_byte, owner.end_byte))
        })
        .collect();
    let mut declarations = BTreeMap::new();
    for matched in matches.values() {
        let (Some(owner), Some(interface)) = (
            matched.get("ngrx.legacy.owner"),
            matched.get("ngrx.legacy.interface"),
        ) else {
            continue;
        };
        let key = (owner.start_byte, owner.end_byte);
        if interface.raw_text != "Action" || shadowed.contains(&key) {
            continue;
        }
        let Some(name) = owners.get(&key) else {
            continue;
        };
        declarations
            .entry(key)
            .or_insert_with(|| ClassActionEvidence {
                name: (*name).into(),
                class_span: span(owner),
                implements_span: span(interface),
                import_span: span(import),
            });
    }
    declarations.into_values().collect()
}

/// Legacy declarations own their facts independently; no call site can create
/// one. Reuse the NgRx shape/publication mechanics without pretending createAction.
pub(crate) fn include(
    shape: Option<super::NgRxShape>,
    declarations: &[ClassActionEvidence],
) -> Option<super::NgRxShape> {
    if declarations.is_empty() {
        return shape;
    }
    let mut shape = shape.unwrap_or_default();
    shape.class_actions.extend(
        declarations
            .iter()
            .filter(|decl| {
                decl.class_span.start_byte() <= decl.implements_span.start_byte()
                    && decl.implements_span.end_byte() <= decl.class_span.end_byte()
                    && decl.import_span.start_byte() < decl.import_span.end_byte()
            })
            .map(|decl| super::ClassActionDecl {
                name: decl.name.clone(),
            }),
    );
    (!shape.is_empty()).then_some(shape)
}

#[cfg(all(test, feature = "typescript"))]
#[path = "../../tests/angular_meta/ngrx_legacy.rs"]
mod tests;
