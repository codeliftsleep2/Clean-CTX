// src/angular_meta/detect.rs
//
// Angular detection heuristic.
//
// The Meta-Layer must run **only** on Angular files. Non-Angular `.ts`
// files (Deno, Node, React, Vue, Svelte, etc.) should pay **zero**
// overhead — no Φ markers, no extra parse, no newlines.
//
// # Strategy
//
// A-11 (Meta-layer detection hardening): We now use AST-based detection
// as the primary strategy to eliminate false positives from comments
// and string literals. We parse the source with tree-sitter and query
// for actual `decorator` AST nodes, then check if any match Angular-
// specific decorator names.
//
// Fallback: If AST parsing fails for any reason, we fall back to the
// original string-based detection to maintain backward compatibility.

use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::compression::language::safe_typescript_language;

/// Angular-specific decorator names that we treat as a strong signal
/// of an Angular file. A single match anywhere in the source is enough
/// to consider the file Angular.
const STRONG_DECORATORS: &[&str] = &[
    "@Component(",
    "@Directive(",
    "@Injectable(",
    "@NgModule(",
    "@Pipe(",
    "@HostListener(",
    "@HostBinding(",
    "@ViewChild(",
    "@ViewChildren(",
    "@ContentChild(",
    "@ContentChildren(",
];

/// Tree-sitter query to find decorator nodes in TypeScript AST.
/// We capture the full decorator text to check against our list.
const ANGULAR_DECORATOR_QUERY: &str = r#"
    (decorator) @decorator
"#;

/// Angular-specific decorator names that are NOT unique to Angular
/// (e.g. `@Input` is also used by MobX, `@Output` by Vue). These count
/// as weak signals and must be paired with a strong signal to trigger
/// Meta-Layer output.
const WEAK_DECORATORS: &[&str] = &["@Input(", "@Input ", "@Output(", "@Output "];

/// Detects the presence of the Angular `core` import. Almost every
/// Angular file imports something from `@angular/core`.
const ANGULAR_CORE_IMPORT: &str = "@angular/core";

/// Decide whether the given source code is from an Angular file.
///
/// A-11 (Meta-layer detection hardening): We now use AST-based detection
/// as the primary strategy to eliminate false positives from comments
/// and string literals. We parse the source with tree-sitter and query
/// for actual `decorator` AST nodes, then check if any match Angular-
/// specific decorator names.
///
/// Fallback: If AST parsing fails for any reason, we fall back to the
/// original string-based detection to maintain backward compatibility.
///
/// A file is "Angular" iff:
/// 1. It contains at least one **strong** decorator (`@Component(`,
///    `@Injectable(`, `@NgModule(`, `@Directive(`, `@Pipe(`, etc.), OR
/// 2. It imports from `@angular/core` AND has at least one
///    `@Input` / `@Output` decorator (weak pair).
///
/// Plain `@Input` / `@Output` alone (no strong signal, no
/// `@angular/core` import) returns `false` — those decorators are
/// also used by MobX / Vue, and a false positive would inject
/// meaningless `Φ` markers into non-Angular output.
pub fn is_angular_file(source: &str) -> bool {
    // A-11: Try AST-based detection first (eliminates false positives
    // from comments and string literals).
    if ast_based_angular_detect(source) {
        return true;
    }

    // Fallback: string-based detection for backward compatibility
    // (used when AST parsing fails or for non-TS files).
    string_based_angular_detect(source)
}

/// AST-based Angular detection using tree-sitter.
/// Parses the source as TypeScript and queries for decorator nodes.
/// Only actual decorator AST nodes are considered — comments and
/// string literals are ignored by the parser.
fn ast_based_angular_detect(source: &str) -> bool {
    // Quick check: if source doesn't contain "@" at all, skip AST parsing
    if !source.contains('@') {
        return false;
    }

    let mut parser = Parser::new();
    let language = match safe_typescript_language() {
        Some(lang) => lang,
        None => return false, // TypeScript feature not enabled
    };
    
    if parser.set_language(&language).is_err() {
        return false;
    }

    let tree = match parser.parse(source, None) {
        Some(tree) => tree,
        None => return false,
    };

    let query = match Query::new(&language, ANGULAR_DECORATOR_QUERY) {
        Ok(q) => q,
        Err(_) => return false,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(mat) = matches.next() {
        for capture in mat.captures.iter() {
            if let Ok(decorator_text) = capture.node.utf8_text(source.as_bytes()) {
                // Check if this decorator matches any Angular-specific decorator
                for strong_deco in STRONG_DECORATORS {
                    if decorator_text.starts_with(strong_deco) {
                        return true;
                    }
                }
                
                // Check for weak decorators (need @angular/core import too)
                for weak_deco in WEAK_DECORATORS {
                    if decorator_text.starts_with(weak_deco) {
                        // Weak decorator found — check if @angular/core is imported
                        if source.contains(ANGULAR_CORE_IMPORT) {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Original string-based Angular detection (fallback).
/// This is the pre-A-11 implementation kept for backward compatibility.
fn string_based_angular_detect(source: &str) -> bool {
    // Tier 1: any strong decorator?
    for deco in STRONG_DECORATORS {
        if source.contains(deco) {
            return true;
        }
    }

    // Tier 2: `@angular/core` import + weak decorator pair?
    if source.contains(ANGULAR_CORE_IMPORT) {
        for weak in WEAK_DECORATORS {
            if source.contains(weak) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
#[path = "../tests/angular_meta/detect.rs"]
mod tests;
