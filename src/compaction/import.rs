// src/compaction/import.rs
//
// Import declaration compaction and symbol-name extraction.

use crate::compression::Fidelity;

/// Compact an import declaration.
///
/// Input:  "import { UserService, AuthService } from './services';"
/// Low:    "UserService,AuthService"          (just the names, no path)
/// Medium: "import {UserService,AuthService} from './services'"
/// High:   "import { UserService, AuthService } from './services'"
pub fn compact_import(text: &str, fidelity: Fidelity) -> String {
    let line = text.lines().next().unwrap_or(text).trim();
    // Strip trailing semicolon
    let line = line.trim_end_matches(';');

    match fidelity {
        Fidelity::Low => {
            // Extract only the imported symbol names
            extract_import_names(line)
        }
        Fidelity::Medium => {
            // Collapse spaces inside braces, keep path
            let line = line
                .replace("{ ", "{")
                .replace(" }", "}")
                .replace(", ", ",");
            line
        }
        Fidelity::High => line.to_string(),
    }
}

/// Extract import symbol names from an import declaration line.
///
/// Handles:
///   import { A, B, C } from '...'    → "A,B,C"
///   import DefaultExport from '...'  → "DefaultExport"
///   import * as NS from '...'        → "NS"
///   import '...'                     → ""  (side-effect import)
pub fn extract_import_names(line: &str) -> String {
    // Named imports: { A, B }
    if let (Some(open), Some(close)) = (line.find('{'), line.find('}')) {
        if open < close {
            return line[open + 1..close]
                .split(',')
                .map(|s| {
                    // Handle "Foo as Bar" aliases — keep the alias
                    s.split(" as ")
                     .last()
                     .unwrap_or(s)
                     .trim()
                     .to_string()
                })
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(",");
        }
    }

    // Namespace import: import * as NS from '...'
    if let Some(as_pos) = line.find("* as ") {
        let rest = &line[as_pos + 5..];
        let name = rest.split_whitespace().next().unwrap_or("").to_string();
        if !name.is_empty() {
            return name;
        }
    }

    // Default import: import Foo from '...'
    // Token after "import " and before "from"
    if let Some(after_import) = line.strip_prefix("import ") {
        let name = after_import
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| c == '\'' || c == '"');
        if !name.is_empty() && name != "from" && !name.starts_with('\'') && !name.starts_with('"') {
            return name.to_string();
        }
    }

    // Side-effect import (no symbols)
    String::new()
}
