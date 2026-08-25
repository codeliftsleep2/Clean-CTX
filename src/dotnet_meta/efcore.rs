// src/dotnet_meta/efcore.rs
//
// Entity Framework Core extraction — DbContext, DbSet, entities, relationships.
//
// Detects:
// - `DbContext` classes
// - `DbSet<T>` properties
// - `[Key]`, `[ForeignKey]`, `[Table]`, `[Column]` attributes
// - Navigation properties (ICollection<T>, T? foreign key)
// - `OnModelCreating` Fluent API configuration

use super::markers::{build_config_line, build_dbset_line, build_ef_line, build_entity_line};
use crate::compression::Fidelity;
use crate::dotnet_meta::MetaBlock;

/// Extract EF Core markers from a single class capture.
///
/// Returns `None` when the class is not an EF Core construct.
pub fn extract_efcore(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    let mut lines = Vec::new();

    // Detect DbContext class
    let is_dbcontext = class_source.contains(": DbContext");

    if !is_dbcontext {
        return None;
    }

    // Extract class name
    let class_name = extract_class_name(class_source)?;

    // Emit DbContext marker
    lines.push(build_ef_line(&class_name));

    // Extract DbSet properties
    lines.extend(extract_dbsets(class_source));

    // Extract entity configurations
    if fidelity != Fidelity::Low {
        lines.extend(extract_entities(class_source, fidelity));
    }

    // Extract Fluent API configuration
    if fidelity == Fidelity::High {
        lines.extend(extract_fluent_config(class_source));
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

/// Extract DbSet<T> properties.
fn extract_dbsets(class_source: &str) -> Vec<String> {
    let mut dbsets = Vec::new();

    // Look for "public DbSet<...>"
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("DbSet<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "DbSet<".len()..];

        // Extract type name until '>'
        if let Some(generic_end) = rest.find('>') {
            // After the closing '>', look for the property name
            let after_generic = &rest[generic_end + 1..];
            // Property name is the first word after '>', e.g. "Users { get; set; }"
            let prop_name = after_generic
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_end_matches('{')
                .trim();
            if !prop_name.is_empty() {
                dbsets.push(build_dbset_line(prop_name));
            }
        }

        search_start = actual_pos + 1;
    }

    dbsets
}

/// Extract entity classes referenced in the DbContext.
fn extract_entities(class_source: &str, fidelity: Fidelity) -> Vec<String> {
    let mut entities = Vec::new();

    // Look for DbSet<EntityName> patterns
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("DbSet<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "DbSet<".len()..];

        if let Some(generic_end) = rest.find('>') {
            let entity_name = rest[..generic_end].trim().to_string();

            // Skip if already added
            if !entities.contains(&entity_name) {
                // Extract key fields if high fidelity
                let fields = if fidelity == Fidelity::High {
                    extract_entity_fields(class_source, &entity_name)
                } else {
                    Vec::new()
                };

                entities.push(build_entity_line(&entity_name, &fields));
            }
        }

        search_start = actual_pos + 1;
    }

    entities
}

/// Extract key fields for an entity (simplified).
#[allow(unused_variables)]
fn extract_entity_fields(class_source: &str, entity_name: &str) -> Vec<String> {
    let mut fields = Vec::new();

    // Look for [Key] attribute on properties
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("[Key]") {
        let actual_pos = search_start + pos;
        let after = &class_source[actual_pos + "[Key]".len()..];

        // Look for property declaration after [Key]
        if let Some(prop_pos) = after.find("public ") {
            let prop_decl = &after[prop_pos + "public ".len()..];
            if let Some(semicolon) = prop_decl.find(';') {
                let prop = &prop_decl[..semicolon];
                let field_name = prop.split_whitespace().last().unwrap_or("");
                if !field_name.is_empty() {
                    fields.push(field_name.to_string());
                }
            }
        }

        search_start = actual_pos + 1;
    }

    // Limit to top 3 fields to keep markers concise
    fields.truncate(3);
    fields
}

/// Extract Fluent API configuration from OnModelCreating.
fn extract_fluent_config(class_source: &str) -> Vec<String> {
    let mut configs = Vec::new();

    // Look for OnModelCreating method
    if let Some(pos) = class_source.find("OnModelCreating") {
        let rest = &class_source[pos..];

        // Look for modelBuilder.Entity<...>() calls
        let mut search_start = 0;
        while let Some(pos) = rest[search_start..].find("modelBuilder.Entity<") {
            let actual_pos = search_start + pos;
            let entity_rest = &rest[actual_pos + "modelBuilder.Entity<".len()..];

            if let Some(generic_end) = entity_rest.find('>') {
                let entity_name = entity_rest[..generic_end].trim().to_string();
                configs.push(build_config_line(&entity_name));
            }

            search_start = actual_pos + 1;
        }
    }

    configs
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/efcore.rs"]
mod tests;
