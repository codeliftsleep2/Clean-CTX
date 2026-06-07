// src/diff/builder.rs
//
// Snapshot construction: parse a source string with tree-sitter, walk the
// captures, and assemble a `CapturedStructure`.

use tree_sitter::{Language, Parser as TSParser, Query, QueryCursor};

use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field, extract_method_sig,
};
use crate::compressor::Fidelity;
use crate::queries;

use super::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

/// Build a structural snapshot by parsing the source with tree-sitter and
/// walking the captures. Mirrors the capture logic in `compressor::compress_file`
/// so the two are directly comparable.
pub fn build_snapshot(
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    let (language, query_string): (Language, &str) = detect_language(source);
    // Try the chosen language first; if it yields no captures (e.g. wrong
    // file content for the heuristic), fall back to the other parser.
    match try_build_with(language, query_string, source, fidelity) {
        Ok(snap) if !snap.classes.is_empty() || !snap.imports.is_empty() => Ok(snap),
        _ => {
            let other_lang = if query_string == queries::TS_QUERY {
                tree_sitter_c_sharp::language()
            } else {
                tree_sitter_typescript::language_typescript()
            };
            let other_query = if query_string == queries::TS_QUERY {
                queries::CS_QUERY
            } else {
                queries::TS_QUERY
            };
            try_build_with(other_lang, other_query, source, fidelity)
        }
    }
}

fn try_build_with(
    language: Language,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    let mut parser = TSParser::new();
    parser.set_language(language)?;
    let tree = parser.parse(source, None).ok_or("AST Generation Error")?;
    let source_bytes = source.as_bytes();

    let query = Query::new(language, query_string)?;
    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source_bytes);

    #[derive(Debug)]
    struct Cap {
        name: String,
        text: String,
        start_byte: usize,
    }
    let mut all_captures: Vec<Cap> = Vec::new();
    for mat in matches {
        for capture in mat.captures {
            let capture_name = query.capture_names()[capture.index as usize].to_string();
            if let Ok(text_slice) = capture.node.utf8_text(source_bytes) {
                let raw = text_slice.to_string();
                let processed = if capture_name == "class.root" {
                    extract_class_name(&raw)
                } else if capture_name == "method.root" {
                    extract_method_sig(&raw, fidelity)
                } else if capture_name == "field.root" {
                    extract_field(&raw, fidelity)
                } else if capture_name == "import.root" {
                    compact_import(&raw, fidelity)
                } else {
                    compact_expression(&raw, fidelity)
                };
                all_captures.push(Cap {
                    name: capture_name,
                    text: processed,
                    start_byte: capture.node.start_byte(),
                });
            }
        }
    }
    all_captures.sort_by(|a, b| a.start_byte.cmp(&b.start_byte));

    let mut classes: Vec<CapturedClass> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut orphan_fields: Vec<String> = Vec::new();
    let mut pending_fields: Vec<String> = Vec::new();
    let mut pending_markers: Vec<String> = Vec::new();

    for cap in &all_captures {
        match cap.name.as_str() {
            "import.root" => {
                if !cap.text.is_empty() {
                    imports.push(cap.text.clone());
                }
            }
            "class.root" => {
                if let Some(last) = classes.last_mut() {
                    if last.fields.is_empty() && !pending_fields.is_empty() {
                        last.fields = std::mem::take(&mut pending_fields);
                    }
                } else if !pending_fields.is_empty() {
                    orphan_fields.extend(pending_fields.drain(..));
                }
                classes.push(CapturedClass {
                    name: cap.text.clone(),
                    fields: Vec::new(),
                    methods: Vec::new(),
                });
                pending_markers.clear();
            }
            "method.root" => {
                if let Some(last) = classes.last_mut() {
                    last.methods.push(CapturedMethod {
                        sig: cap.text.clone(),
                        markers: std::mem::take(&mut pending_markers),
                    });
                }
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    pending_fields.push(cap.text.clone());
                }
            }
            _ => {
                if fidelity == Fidelity::Low {
                    continue;
                }
                let marker = match cap.name.as_str() {
                    "throw.root" => format!("⊕!{}", cap.text),
                    "for.root" => "⊕loop".to_string(),
                    "if.root" => "⊕guard".to_string(),
                    "while.root" => "⊕loop".to_string(),
                    "return.root" => format!("⊕⇒{}", cap.text),
                    _ => continue,
                };
                if pending_markers.last().map(|m| m != &marker).unwrap_or(true) {
                    pending_markers.push(marker);
                }
            }
        }
    }

    if let Some(last) = classes.last_mut() {
        if last.fields.is_empty() && !pending_fields.is_empty() {
            last.fields = pending_fields;
        }
    } else if !pending_fields.is_empty() {
        orphan_fields.extend(pending_fields);
    }

    Ok(CapturedStructure {
        imports,
        classes,
        orphan_fields,
    })
}

fn detect_language(source: &str) -> (Language, &'static str) {
    let looks_csharp = source.contains("namespace ")
        || source.contains("using System")
        || source.contains("public class ")
        || source.contains("private void ");
    if looks_csharp {
        (tree_sitter_c_sharp::language(), queries::CS_QUERY)
    } else {
        (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
    }
}
