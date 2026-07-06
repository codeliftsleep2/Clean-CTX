// src/dotnet_meta/detect.rs
//
// .NET / C# detection heuristic.
//
// The Meta-Layer must run **only** on .NET framework files. Plain C#
// files (utility classes, POCOs, etc.) should pay **zero** overhead —
// no Φ markers, no extra parse, no newlines.
//
// # Strategy
//
// A-11 (Meta-layer detection hardening): We now use AST-based detection
// as the primary strategy to eliminate false positives from comments
// and string literals. We parse the source with tree-sitter and query
// for actual `attribute` AST nodes, then check if any match .NET-
// specific attribute names.
//
// Fallback: If AST parsing fails for any reason, we fall back to the
// original string-based detection to maintain backward compatibility.

use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

use crate::compression::language::safe_csharp_language;

/// Strong .NET framework signals. A single match anywhere in the source
/// is enough to consider the file a .NET framework file.
const STRONG_SIGNALS: &[&str] = &[
    // ASP.NET Core
    "[ApiController]",
    "[Route(",
    "[HttpGet(",
    "[HttpPost(",
    "[HttpPut(",
    "[HttpDelete(",
    "[HttpPatch(",
    "[Authorize]",
    "[AllowAnonymous]",
    ": ControllerBase",
    ": Controller",
    // EF Core
    ": DbContext",
    "DbSet<",
    "[Key]",
    "[ForeignKey(",
    "[Table(",
    "[Column(",
    "[Required]",
    "[StringLength(",
    // AutoMapper
    ": Profile",
    "CreateMap<",
    // SignalR
    ": Hub",
    "IHubContext<",
    "HubCallerContext",
    // FluentValidation
    "AbstractValidator<",
    // Identity
    "UserManager<",
    "SignInManager<",
    "IdentityUser",
    "IdentityRole",
    // Caching
    "IMemoryCache",
    "IDistributedCache",
    "[ResponseCache",
    // Background Jobs
    "BackgroundJob",
    "RecurringJob",
    // Logging
    "ILogger<",
    "ILoggerFactory",
    // DI
    "AddScoped<",
    "AddSingleton<",
    "AddTransient<",
    "AddDbContext<",
    // JWT
    "AddAuthentication",
    "AddJwtBearer",
    // OpenTelemetry / Metrics
    "AddOpenTelemetry",
    "AddApplicationInsights",
];

/// .NET-specific attribute names that we treat as a strong signal
/// of a .NET framework file. A single match anywhere in the source
/// is enough to consider the file .NET.
const STRONG_C_SHARP_ATTRIBUTES: &[&str] = &[
    "ApiController",
    "Route",
    "HttpGet",
    "HttpPost",
    "HttpPut",
    "HttpDelete",
    "HttpPatch",
    "Authorize",
    "AllowAnonymous",
    "Key",
    "ForeignKey",
    "Table",
    "Column",
    "Required",
    "StringLength",
    "ResponseCache",
];

/// Tree-sitter query to find attribute nodes in C# AST.
/// In the C# tree-sitter grammar, attributes are represented as
/// `attribute` nodes containing an `attribute_name` and optional
/// `attribute_argument_list`.
const CS_ATTRIBUTE_QUERY: &str = r#"
    (attribute
        name: (identifier) @attr_name)
"#;

/// .NET-specific base class signals that we treat as strong indicators
/// of a .NET framework file.
const STRONG_BASE_CLASSES: &[&str] = &[
    "ControllerBase",
    "Controller",
    "DbContext",
    "Hub",
    "Profile",
];

/// Tree-sitter query to find class declarations with base types in C# AST.
/// Captures only the base type's name (not the entire class declaration).
/// The `@class` capture was removed to avoid unnecessary node extraction.
const CS_BASE_CLASS_QUERY: &str = r#"
    (class_declaration
        (base_list
            (simple_base_type
                name: (identifier) @base_name)))
"#;

/// Decide whether the given source code is from a .NET framework file.
///
/// A-11 (Meta-layer detection hardening): We now use AST-based detection
/// as the primary strategy to eliminate false positives from comments
/// and string literals. We parse the source with tree-sitter and query
/// for actual `attribute` AST nodes and `base_list` nodes, then check
/// if any match .NET-specific patterns.
///
/// Fallback: If AST parsing fails for any reason, we fall back to the
/// original string-based detection to maintain backward compatibility.
///
/// A file is ".NET framework" iff it contains at least one **strong**
/// signal from the list above, either detected via AST or string fallback.
///
/// Plain C# files (utility classes, POCOs, enums, etc.) return `false`
/// — they should not get any Φ markers.
pub fn is_dotnet_file(source: &str) -> bool {
    // A-11: Try AST-based detection first (eliminates false positives
    // from comments and string literals).
    if ast_based_dotnet_detect(source) {
        return true;
    }

    // Fallback: string-based detection for backward compatibility
    // (used when AST parsing fails or for non-CS files).
    string_based_dotnet_detect(source)
}

/// AST-based .NET detection using tree-sitter.
/// Parses the source as C# and queries for attribute and base class nodes.
/// Only actual AST nodes are considered — comments and string literals
/// are ignored by the parser.
fn ast_based_dotnet_detect(source: &str) -> bool {
    // Quick check: if source doesn't contain "[" or ":" at all, skip AST parsing
    if !source.contains('[') && !source.contains(": ") {
        return false;
    }

    let mut parser = Parser::new();
    let language = match safe_csharp_language() {
        Some(lang) => lang,
        None => return false, // C# feature not enabled
    };

    if parser.set_language(&language).is_err() {
        return false;
    }

    let tree = match parser.parse(source, None) {
        Some(tree) => tree,
        None => return false,
    };

    // Query 1: Check for .NET-specific attributes
    let query = match Query::new(&language, CS_ATTRIBUTE_QUERY) {
        Ok(q) => q,
        Err(_) => return false,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    while let Some(mat) = matches.next() {
        for capture in mat.captures.iter() {
            if let Ok(attr_text) = capture.node.utf8_text(source.as_bytes()) {
                for strong_attr in STRONG_C_SHARP_ATTRIBUTES {
                    if attr_text == *strong_attr {
                        return true;
                    }
                }
            }
        }
    }

    // Query 2: Check for .NET-specific base classes
    let base_query = match Query::new(&language, CS_BASE_CLASS_QUERY) {
        Ok(q) => q,
        Err(_) => return false, // If the query failed, rely on fallback
    };

    let mut cursor2 = QueryCursor::new();
    let mut base_matches = cursor2.matches(&base_query, tree.root_node(), source.as_bytes());

    while let Some(mat) = base_matches.next() {
        for capture in mat.captures.iter() {
            if let Ok(base_text) = capture.node.utf8_text(source.as_bytes()) {
                for strong_base in STRONG_BASE_CLASSES {
                    if base_text == *strong_base {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Original string-based .NET detection (fallback).
/// This is the pre-A-11 implementation kept for backward compatibility.
fn string_based_dotnet_detect(source: &str) -> bool {
    for signal in STRONG_SIGNALS {
        if source.contains(signal) {
            return true;
        }
    }
    false
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/detect.rs"]
mod tests;