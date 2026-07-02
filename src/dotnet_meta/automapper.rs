// src/dotnet_meta/automapper.rs
//
// AutoMapper extraction — Profiles, CreateMap, mappings, projections.
//
// Detects:
// - `Profile` classes
// - `CreateMap<TSource, TDestination>()` mappings
// - `ForMember()` configurations
// - `Ignore()` ignored members
// - `ProjectTo<T>()` projections

use super::markers::{build_ignore_line, build_mapfrom_line, build_mapper_line, build_projection_line};
use crate::dotnet_meta::MetaBlock;
use crate::compression::Fidelity;

/// Extract AutoMapper markers from a single class capture.
///
/// Returns `None` when the class is not an AutoMapper Profile.
pub fn extract_automapper(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    let mut lines = Vec::new();

    // Detect Profile class
    let is_profile = class_source.contains(": Profile");

    if !is_profile {
        return None;
    }

    // Extract class name
    let class_name = extract_class_name(class_source)?;

    // Emit Profile marker
    lines.push(build_mapper_line(&class_name));

    // Extract CreateMap mappings
    lines.extend(extract_mappings(class_source));

    // Extract ignored members
    if fidelity != Fidelity::Low {
        lines.extend(extract_ignored_members(class_source));
    }

    // Extract projections
    if fidelity == Fidelity::High {
        lines.extend(extract_projections(class_source));
    }

    if lines.is_empty() {
        None
    } else {
        Some(MetaBlock { lines })
    }
}

/// Extract the class name from a class declaration.
fn extract_class_name(source: &str) -> Option<String> {
    let patterns = [
        "public class ",
        "internal class ",
        "private class ",
        "protected class ",
        "class ",
    ];

    for pattern in &patterns {
        if let Some(pos) = source.find(pattern) {
            let start = pos + pattern.len();
            let rest = &source[start..];
            let end = rest
                .find(|c: char| c == ':' || c == '<' || c.is_whitespace() || c == '{')
                .unwrap_or(rest.len());
            let name = rest[..end].trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Extract CreateMap<TSource, TDestination>() mappings.
fn extract_mappings(class_source: &str) -> Vec<String> {
    let mut mappings = Vec::new();

    // Look for CreateMap<...> patterns
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("CreateMap<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "CreateMap<".len()..];

        // Extract source and destination types
        if let Some(generic_end) = rest.find('>') {
            let types = rest[..generic_end].trim().to_string();

            // Split by comma to get source and destination
            if let Some(comma_pos) = types.find(',') {
                let source = types[..comma_pos].trim().to_string();
                let dest = types[comma_pos + 1..].trim().to_string();
                mappings.push(build_mapfrom_line(&source, &dest));
            }
        }

        search_start = actual_pos + 1;
    }

    mappings
}

/// Extract ignored members from ForMember().Ignore() calls.
fn extract_ignored_members(class_source: &str) -> Vec<String> {
    let mut ignored = Vec::new();

    // Look for .Ignore() patterns
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find(".Ignore()") {
        let actual_pos = search_start + pos;
        let before = &class_source[..actual_pos];

        // Look backwards for ForMember(dest => dest.MemberName)
        if let Some(for_member_pos) = before.rfind("ForMember") {
            let for_member_rest = &before[for_member_pos + "ForMember".len()..];

            // Look for dest => dest.MemberName or src => src.MemberName
            if let Some(arrow_pos) = for_member_rest.find("=>") {
                let after_arrow = &for_member_rest[arrow_pos + 2..];
                if let Some(dot_pos) = after_arrow.find('.') {
                    let member_rest = &after_arrow[dot_pos + 1..];
                    let member_name = member_rest
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .trim_end_matches(',')
                        .trim_end_matches(')')
                        .to_string();

                    if !member_name.is_empty() {
                        ignored.push(build_ignore_line(&member_name));
                    }
                }
            }
        }

        search_start = actual_pos + 1;
    }

    // Deduplicate and limit
    ignored.dedup();
    ignored.truncate(5);
    ignored
}

/// Extract ProjectTo<T>() projections.
fn extract_projections(class_source: &str) -> Vec<String> {
    let mut projections = Vec::new();

    // Look for ProjectTo<...> patterns
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("ProjectTo<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "ProjectTo<".len()..];

        if let Some(generic_end) = rest.find('>') {
            let target_type = rest[..generic_end].trim().to_string();
            projections.push(build_projection_line(&target_type));
        }

        search_start = actual_pos + 1;
    }

    projections.dedup();
    projections.truncate(3);
    projections
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/automapper.rs"]
mod tests;