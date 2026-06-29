// src/diff/builder.rs
//
// Snapshot construction: parse a source string with tree-sitter, walk the
// captures, and assemble a `CapturedStructure`.
//
// Phase 2: the tree-sitter capture walk, the language heuristic, and the
// marker construction logic now live in `crate::compression::*`. This
// file is reduced to:
//   1. Choosing the language (with a content-based fallback)
//   2. Driving the SHARED capture pipeline
//   3. Assembling the `CapturedStructure` from the resulting `CapEntry`s

use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field, extract_method_sig,
    extract_rust_struct_name,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::detect_language;
use crate::compression::markers::build_marker;
use crate::compression::Fidelity;
use crate::queries;

use super::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

/// A parser configuration: language factory, query string, and label.
type ParserConfig = (fn() -> tree_sitter::Language, &'static str, &'static str);

/// All supported parser configurations, in the order they should be tried.
const ALL_PARSERS: &[ParserConfig] = &[
    (crate::compression::language::safe_rust_language, queries::RS_QUERY, "rust"),
    (crate::compression::language::safe_csharp_language, queries::CS_QUERY, "csharp"),
    (crate::compression::language::safe_typescript_language, queries::TS_QUERY, "typescript"),
];

/// Build a structural snapshot by parsing the source with tree-sitter and
/// walking the captures. Mirrors the capture logic in `compressor::compress_file`
/// so the two are directly comparable.
///
/// Phase D: The fallback now tries all three supported languages (Rust, C#, TS)
/// instead of only TS ↔ C#. If the first parser chosen by `detect_language`
/// yields no captures, the remaining two are tried in priority order.
pub fn build_snapshot(
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    let (first_lang, first_query) = detect_language(source);

    // Phase D: Build an ordered list of parsers to try, starting with the
    // detected language, followed by the others (deduplicating).
    //
    // We match against the query string to identify which parser was chosen.
    let first_label = if first_query == queries::RS_QUERY {
        "rust"
    } else if first_query == queries::CS_QUERY {
        "csharp"
    } else {
        "typescript"
    };

    // Collect parsers to try: first choice first, then the others.
    let mut candidates: Vec<(&str, tree_sitter::Language, &str)> = Vec::with_capacity(3);
    // Push the detected parser first
    candidates.push((first_label, first_lang.clone(), first_query));
    // Push the remaining two in ALL_PARSERS order, skipping the detected one
    for (lang_fn, query, label) in ALL_PARSERS {
        if *label != first_label {
            candidates.push((label, lang_fn(), query));
        }
    }

    // Try each parser in order. Return the first one that produces captures,
    // or the last result (even if empty).
    let mut last_result: Option<Result<CapturedStructure, Box<dyn std::error::Error>>> = None;
    for (_label, lang, query) in &candidates {
        let result = try_build_with(lang.clone(), query, source, fidelity);
        match &result {
            Ok(snap) if !snap.classes.is_empty() || !snap.imports.is_empty() => {
                return result;
            }
            _ => {
                last_result = Some(result);
            }
        }
    }

    // If none produced meaningful captures, return the last attempt's result
    // (even if empty, so callers get a valid CapturedStructure).
    last_result.unwrap_or_else(|| {
        // Safety net: should never reach here since we always push 3 candidates
        try_build_with(first_lang, first_query, source, fidelity)
    })
}

fn try_build_with(
    language: tree_sitter::Language,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    // Run the SHARED capture pipeline. The closure maps each
    // (capture_name, raw_text) pair to the normalised text the diff
    // path wants stored in the resulting CapEntry. F-08: pass the
    // real `fidelity` through so the per-capture closures honour it.
    let all_captures = run_capture_pipeline(
        language,
        query_string,
        source,
        fidelity,
        |capture_name, raw, f| {
            match capture_name {
                "class.root" => Some(extract_class_name(raw)),
                // Rust type declarations: struct, enum, trait, impl
                "struct.root" | "enum.root" | "trait.root" | "impl.root" => {
                    Some(extract_rust_struct_name(raw))
                }
                "method.root" => Some(extract_method_sig(raw, f)),
                "field.root" => Some(extract_field(raw, f)),
                "import.root" | "mod.root" => Some(compact_import(raw, f)),
                "type.root" => Some(compact_expression(raw, f)),
                _ => Some(compact_expression(raw, f)),
            }
        },
    )?;

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
            "class.root" | "struct.root" | "enum.root" | "trait.root" | "impl.root" => {
                if let Some(last) = classes.last_mut() {
                    if last.fields.is_empty() && !pending_fields.is_empty() {
                        last.fields = std::mem::take(&mut pending_fields);
                    }
                } else if !pending_fields.is_empty() {
                    orphan_fields.append(&mut pending_fields);
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
                // Delegate marker construction to the SHARED module.
                if let Some(marker) = build_marker(&cap.name, &cap.text)
                    && pending_markers.last().map(|m| m != &marker).unwrap_or(true) {
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

#[cfg(test)]
#[path = "../tests/diff/builder.rs"]
mod tests;