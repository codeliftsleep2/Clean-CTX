// src/compaction/java.rs
//
// Java-specific extraction and compaction helpers.
// Handles: class, interface, enum, record, constructor, package declarations.

use crate::compaction::expression::compact_expression;
use crate::compaction::modifiers::{strip_modifiers, MODIFIERS_CLASS};
use crate::compression::Fidelity;

/// Extract a Java type name from interface, enum, or record declarations.
///
/// Input examples:
///   "public interface UserRepository extends JpaRepository<User, Long> { ... }"
///   "public enum Status { ACTIVE, INACTIVE }"
///   "public record UserDto(String name, int age) { ... }"
///
/// Output examples:
///   "UserRepository:JpaRepository"
///   "Status"
///   "UserDto"
pub fn extract_java_type_name(text: &str, capture_name: &str) -> String {
    // Take only the declaration line (everything before the first `{`)
    let decl = text.lines().next().unwrap_or(text);
    let decl = decl.split('{').next().unwrap_or(decl).trim();

    // Strip leading modifiers
    let rest = strip_modifiers(decl, MODIFIERS_CLASS);

    // Extract the type keyword (interface, enum, record, or class)
    let type_keyword = match capture_name {
        "interface.root" => "interface",
        "enum.root" => "enum",
        "record.root" => "record",
        _ => "class",
    };

    // Strip the type keyword
    let rest = rest
        .strip_prefix(type_keyword)
        .or_else(|| rest.strip_prefix("class "))
        .unwrap_or(&rest)
        .trim();

    // Extract name (take everything before "extends" or "implements" or `<` or `(`)
    let bare_name = rest
        .split_once("extends")
        .map(|(name, _)| name)
        .or_else(|| rest.split_once("implements").map(|(name, _)| name))
        .unwrap_or(rest);
    let bare_name = bare_name.split(['<', '(']).next().unwrap_or(bare_name);
    let bare_name = bare_name.trim_end_matches(['{', '}', ':']).trim();

    // For interfaces, extract extends clause
    if capture_name == "interface.root" {
        let extends = extract_base_types(rest, "extends");
        if extends.is_empty() {
            return bare_name.to_string();
        } else {
            return format!("{}:{}", bare_name, extends.join(","));
        }
    }

    // For enums and records, just return the name
    bare_name.to_string()
}

/// Extract a compact Java constructor signature.
///
/// Input:  "public UserService(UserRepository repo, AuthService auth) { ... }"
/// Low:    "UserService(UserRepository,AuthService)"
/// Medium: "UserService(UserRepository,AuthService)"
/// High:   "public UserService(UserRepository repo, AuthService auth)"
pub fn extract_java_constructor_sig(text: &str, fidelity: Fidelity) -> String {
    let sig_line = text.lines().next().unwrap_or(text);
    let sig_line = sig_line.split('{').next().unwrap_or(sig_line).trim();

    match fidelity {
        Fidelity::Low | Fidelity::Medium => {
            // Strip modifiers and class name, keep only params
            let s = strip_modifiers(sig_line, MODIFIERS_CLASS);

            // Extract constructor name and params
            // "ClassName(param1: Type1, param2: Type2)"
            if let Some(open) = s.find('(') {
                let close = s.rfind(')').unwrap_or(s.len());
                if open < close {
                    let params = &s[open + 1..close];
                    // Extract just type names (no parameter names)
                    // Java params are "Type name" (no colon separator)
                    let type_names: Vec<String> = params
                        .split(',')
                        .map(|p| {
                            let p = p.trim();
                            // Take first word as type name
                            p.split_whitespace()
                                .next()
                                .unwrap_or(p)
                                .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .collect();

                    // Get constructor name (before the parenthesis)
                    let name_part = s[..open].trim();
                    let name = name_part.split_whitespace().last().unwrap_or(name_part);

                    return format!("{}({})", name, type_names.join(","));
                }
            }

            // Fallback: just compact the expression
            compact_expression(sig_line, fidelity)
        }
        Fidelity::High => sig_line.to_string(),
    }
}

/// Compact a Java package declaration.
///
/// Input:  "package com.example.userservice;"
/// Low:    "com.example.userservice"
/// Medium: "package com.example.userservice"
/// High:   "package com.example.userservice"
pub fn compact_java_package(text: &str, fidelity: Fidelity) -> String {
    let line = text.lines().next().unwrap_or(text).trim();
    // Strip trailing semicolon
    let line = line.trim_end_matches(';').trim();

    match fidelity {
        Fidelity::Low => {
            // Extract just the package name
            if let Some(after_package) = line.strip_prefix("package ") {
                after_package.trim().to_string()
            } else {
                line.to_string()
            }
        }
        Fidelity::Medium | Fidelity::High => line.to_string(),
    }
}

/// Format a Java type entry (interface, enum, record) with fields.
///
/// Low:    "InterfaceName{field1;field2}"
/// Medium: "interface InterfaceName { field1; field2 }"
/// High:   "public interface InterfaceName {\n  field1\n  field2\n}"
pub fn format_java_type_entry(
    name: &str,
    capture_name: &str,
    fields: &[String],
    fidelity: Fidelity,
) -> String {
    let type_keyword = match capture_name {
        "interface.root" => "interface",
        "enum.root" => "enum",
        "record.root" => "record",
        _ => "class",
    };

    match fidelity {
        Fidelity::Low => {
            if fields.is_empty() {
                name.to_string()
            } else {
                format!("{}{{{}}}", name, fields.join(";"))
            }
        }
        Fidelity::Medium => {
            if fields.is_empty() {
                format!("{} {}", type_keyword, name)
            } else {
                format!("{} {} {{ {} }}", type_keyword, name, fields.join("; "))
            }
        }
        Fidelity::High => {
            if fields.is_empty() {
                format!("{} {} {{", type_keyword, name)
            } else {
                let field_lines = fields
                    .iter()
                    .map(|f| format!("  {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{} {} {{\n{}", type_keyword, name, field_lines)
            }
        }
    }
}

/// Pull the type names that follow a given keyword (`extends` or `implements`)
/// out of a declaration string.
fn extract_base_types(decl: &str, keyword: &str) -> Vec<String> {
    let Some(after) = decl.split_once(keyword) else {
        return Vec::new();
    };
    let segment = after.1;

    // Take everything up to the next keyword or `{` or end of string
    let segment = segment
        .split_once("implements")
        .map(|(l, _)| l)
        .unwrap_or(segment);
    let segment = segment
        .split_once("extends")
        .map(|(l, _)| l)
        .unwrap_or(segment);
    let segment = segment.split('{').next().unwrap_or(segment);
    // Strip generic parameters by taking content before `<`
    let segment = segment.split('<').next().unwrap_or(segment);

    segment
        .split(',')
        .map(|s| {
            s.trim()
                .split('<')
                .next()
                .unwrap_or(s.trim())
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
#[path = "../tests/compaction/java.rs"]
mod tests;