// src/spring_meta/detect.rs
//
// Spring Boot detection heuristic.
//
// The Meta-Layer must run **only** on Spring Boot files. Plain Java
// files should pay **zero** overhead — no Φ markers, no extra parse,
// no newlines.
//
// # Strategy
//
// A-11 (Meta-layer detection hardening): We now use AST-based detection
// as the primary strategy to eliminate false positives from comments
// and string literals. We parse the source with tree-sitter and query
// for actual annotation AST nodes, then check if any match Spring-
// specific annotation names.
//
// Fallback: If AST parsing fails for any reason, we fall back to the
// original string-based detection to maintain backward compatibility.

use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::compression::language::safe_java_language;

/// Tree-sitter query to find annotation nodes in Java AST.
/// We capture the full annotation text to check against our list.
const SPRING_ANNOTATION_QUERY: &str = r#"
    (annotation) @annotation
"#;

/// Spring-specific annotation names that we treat as a strong signal
/// of a Spring Boot file. A single match anywhere in the source is enough
/// to consider the file Spring.
const STRONG_ANNOTATIONS: &[&str] = &[
    "@RestController",
    "@Controller",
    "@Service",
    "@Repository",
    "@Configuration",
    "@SpringBootApplication",
    "@EnableAutoConfiguration",
    "@ComponentScan",
    "@RequestMapping",
    "@GetMapping",
    "@PostMapping",
    "@PutMapping",
    "@DeleteMapping",
    "@PatchMapping",
];

/// Spring-specific annotation names that are NOT unique to Spring
/// (e.g. `@Autowired` is also used in plain Spring, `@Value` can be
/// used in non-Spring contexts). These count as weak signals and must
/// be paired with a strong signal to trigger Meta-Layer output.
const WEAK_ANNOTATIONS: &[&str] = &[
    "@Autowired",
    "@Value(",
    "@ConfigurationProperties",
    "@Bean",
    "@Primary",
    "@Qualifier",
];

/// Detects the presence of the Spring `org.springframework` import.
/// Almost every Spring Boot file imports something from
/// `org.springframework`.
const SPRING_IMPORT: &str = "org.springframework";

/// Decide whether the given source code is from a Spring Boot file.
///
/// A-11 (Meta-layer detection hardening): We now use AST-based detection
/// as the primary strategy to eliminate false positives from comments
/// and string literals. We parse the source with tree-sitter and query
/// for actual annotation AST nodes, then check if any match Spring-
/// specific annotation names.
///
/// Fallback: If AST parsing fails for any reason, we fall back to the
/// original string-based detection to maintain backward compatibility.
///
/// A file is "Spring Boot" iff:
/// 1. It contains at least one **strong** annotation (`@RestController`,
///    `@Service`, `@Repository`, `@Configuration`, `@RequestMapping`, etc.), OR
/// 2. It imports from `org.springframework` AND has at least one
///    weak annotation (`@Autowired`, `@Value`, `@Bean`, etc.).
///
/// Plain `@Autowired` alone (no strong signal, no Spring import) returns
/// `false` — that annotation is also used in plain Spring Framework
/// contexts, and a false positive would inject meaningless `Φ` markers
/// into non-Spring output.
pub fn is_spring_file(source: &str) -> bool {
    // A-11: Try AST-based detection first (eliminates false positives
    // from comments and string literals).
    //
    // If AST parsing succeeds, we trust its result completely — even if
    // it found no annotations. This prevents false positives from
    // comments/string literals that would otherwise trigger the
    // string-based fallback.
    //
    // Fall back to string matching if AST parsing fails OR if the query
    // finds no annotations (query pattern may not match this grammar version).
    match ast_based_spring_detect_with_status(source) {
        AstResult::Detected => return true,
        AstResult::NotFound | AstResult::ParseFailed => {} // fall through to string fallback
    }

    // Fallback: string-based detection for backward compatibility
    // (only used when AST parsing fails).
    string_based_spring_detect(source)
}

/// Result of AST-based detection.
enum AstResult {
    /// Found a matching annotation in the AST.
    Detected,
    /// Parsed successfully but found no matching annotations.
    NotFound,
    /// AST parsing failed (fall back to string-based detection).
    ParseFailed,
}

/// AST-based Spring detection with status reporting.
/// Returns `AstResult` to distinguish between "not found" and "parse failed".
fn ast_based_spring_detect_with_status(source: &str) -> AstResult {
    // Quick check: if source doesn't contain "@" at all, skip AST parsing
    if !source.contains('@') {
        return AstResult::NotFound;
    }

    let mut parser = Parser::new();
    let language = match safe_java_language() {
        Some(lang) => lang,
        None => return AstResult::ParseFailed, // Java feature not enabled
    };
    
    if parser.set_language(&language).is_err() {
        return AstResult::ParseFailed;
    }

    let tree = match parser.parse(source, None) {
        Some(tree) => tree,
        None => return AstResult::ParseFailed,
    };

    let query = match Query::new(&language, SPRING_ANNOTATION_QUERY) {
        Ok(q) => q,
        Err(_) => return AstResult::ParseFailed,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(mat) = matches.next() {
        for capture in mat.captures.iter() {
            if let Ok(annotation_text) = capture.node.utf8_text(source.as_bytes()) {
                // The tree-sitter Java grammar's `annotation` node captures
                // the annotation name (with or without @ depending on grammar version).
                // Try matching with both formats.
                let text = annotation_text.trim();
                
                // Try direct match with @ prefix
                for &strong_anno in STRONG_ANNOTATIONS {
                    if text == strong_anno || text.starts_with(strong_anno) {
                        return AstResult::Detected;
                    }
                }
                
                // Try match without @ prefix (tree-sitter may strip it)
                for &strong_anno in STRONG_ANNOTATIONS {
                    let anno_name = strong_anno.trim_start_matches('@');
                    if text == anno_name || text.starts_with(anno_name) {
                        return AstResult::Detected;
                    }
                }
                
                // Check for weak annotations (need org.springframework import too)
                for &weak_anno in WEAK_ANNOTATIONS {
                    if (text == weak_anno || text.starts_with(weak_anno))
                        && source.contains(SPRING_IMPORT)
                    {
                        return AstResult::Detected;
                    }
                }
                
                // Try weak match without @ prefix
                for &weak_anno in WEAK_ANNOTATIONS {
                    let anno_name = weak_anno.trim_start_matches('@').trim_end_matches('(');
                    if (text == anno_name || text.starts_with(anno_name))
                        && source.contains(SPRING_IMPORT)
                    {
                        return AstResult::Detected;
                    }
                }
            }
        }
    }

    AstResult::NotFound
}

/// Original string-based Spring detection (fallback).
/// This is the pre-A-11 implementation kept for backward compatibility.
/// 
/// A-11 enhancement: We now check if the match is in a comment to avoid
/// false positives from commented-out code or documentation.
fn string_based_spring_detect(source: &str) -> bool {
    // Tier 1: any strong annotation?
    for anno in STRONG_ANNOTATIONS {
        if source.contains(anno) {
            // Check if this match is in a comment or string literal
            if !is_in_comment_or_string(source, anno) {
                return true;
            }
        }
    }

    // Tier 2: Spring import + weak annotation pair?
    if source.contains(SPRING_IMPORT) {
        for weak in WEAK_ANNOTATIONS {
            if source.contains(weak) {
                // Check if this match is in a comment or string literal
                if !is_in_comment_or_string(source, weak) {
                    return true;
                }
            }
        }
    }

    false
}

/// Check if a pattern match appears to be in a comment or string literal.
/// This is a simple heuristic to avoid false positives from documentation
/// or commented-out code.
fn is_in_comment_or_string(source: &str, pattern: &str) -> bool {
    for line in source.lines() {
        if line.contains(pattern) {
            let trimmed = line.trim();
            // Single-line comment
            if trimmed.starts_with("//") {
                return true;
            }
            // Multi-line comment start
            if trimmed.starts_with("/*") {
                return true;
            }
            // Check if we're inside a multi-line comment (simplified check)
            if line.contains("*/") && line.find("/*") < line.find(pattern) {
                return true;
            }
            // Check if pattern is inside a string literal
            if is_in_string_literal(line, pattern) {
                return true;
            }
        }
    }
    false
}

/// Check if a pattern appears to be inside a string literal.
/// Looks for quote characters before and after the pattern.
fn is_in_string_literal(line: &str, pattern: &str) -> bool {
    if let Some(pattern_pos) = line.find(pattern) {
        // Count quotes before the pattern
        let before = &line[..pattern_pos];
        let quote_count = before.matches('"').count();
        // If odd number of quotes before pattern, we're inside a string
        if quote_count % 2 == 1 {
            return true;
        }
        // Also check for single quotes
        let single_quote_count = before.matches('\'').count();
        if single_quote_count % 2 == 1 {
            return true;
        }
    }
    false
}

