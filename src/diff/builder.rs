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
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::detect_language;
use crate::compression::markers::build_marker;
use crate::compression::Fidelity;
use crate::queries;

use super::snapshot::{CapturedClass, CapturedMethod, CapturedStructure};

/// Build a structural snapshot by parsing the source with tree-sitter and
/// walking the captures. Mirrors the capture logic in `compressor::compress_file`
/// so the two are directly comparable.
pub fn build_snapshot(
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    let (language, query_string) = detect_language(source);
    // Try the chosen language first; if it yields no captures (e.g. wrong
    // file content for the heuristic), fall back to the other parser.
    match try_build_with(language, query_string, source, fidelity) {
        Ok(snap) if !snap.classes.is_empty() || !snap.imports.is_empty() => Ok(snap),
        _ => {
            let (other_lang, other_query) = if query_string == queries::TS_QUERY {
                (tree_sitter_c_sharp::language(), queries::CS_QUERY)
            } else {
                (tree_sitter_typescript::language_typescript(), queries::TS_QUERY)
            };
            try_build_with(other_lang, other_query, source, fidelity)
        }
    }
}

fn try_build_with(
    language: tree_sitter::Language,
    query_string: &str,
    source: &str,
    fidelity: Fidelity,
) -> Result<CapturedStructure, Box<dyn std::error::Error>> {
    // Run the SHARED capture pipeline. The closure maps each
    // (capture_name, raw_text) pair to the normalised text the diff
    // path wants stored in the resulting CapEntry.
    let all_captures = run_capture_pipeline(
        language,
        query_string,
        source,
        |capture_name, raw, _low| {
            if capture_name == "class.root" {
                Some(extract_class_name(raw))
            } else if capture_name == "method.root" {
                Some(extract_method_sig(raw, fidelity))
            } else if capture_name == "field.root" {
                Some(extract_field(raw, fidelity))
            } else if capture_name == "import.root" {
                Some(compact_import(raw, fidelity))
            } else {
                Some(compact_expression(raw, fidelity))
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
            "class.root" => {
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
mod tests {
    use super::*;

    #[test]
    fn build_snapshot_parses_a_simple_class() {
        let src = r#"
            export class Foo {
                public greet(name: string): string { return "hi " + name; }
            }
        "#;
        let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
        assert!(!snap.classes.is_empty(), "expected at least one class");
        let foo = &snap.classes[0];
        assert_eq!(foo.name, "Foo");
    }

    #[test]
    fn build_snapshot_handles_empty_source() {
        let snap = build_snapshot("", Fidelity::Low).expect("build_snapshot");
        assert!(snap.classes.is_empty());
        assert!(snap.imports.is_empty());
    }

    #[test]
    fn build_snapshot_falls_back_to_other_language() {
        // C# content where the TS-first heuristic would fail to find
        // anything; the fallback should still produce a snapshot.
        let src = r#"
            namespace MyApp {
                public class Greeter {
                    public string Greet(string name) { return "hi " + name; }
                }
            }
        "#;
        let snap = build_snapshot(src, Fidelity::Low).expect("build_snapshot");
        // Either the C# path or the TS fallback must produce classes.
        assert!(!snap.classes.is_empty() || !snap.imports.is_empty());
    }
}
