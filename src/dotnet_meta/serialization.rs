// src/dotnet_meta/serialization.rs
//
// JSON serialization extraction — JsonPropertyName, JsonIgnore, DataMember, etc.
//
// Detects:
// - `[JsonPropertyName]` / `[JsonIgnore]` / `[JsonConverter]`
// - `[DataMember]` / `[IgnoreDataMember]`
// - Property-level serialization attributes

use super::markers::{build_json_line, build_property_line};
use crate::compression::Fidelity;
use crate::dotnet_meta::MetaBlock;

/// Extract serialization markers from a single class capture.
///
/// Returns `None` when the class has no serialization attributes.
pub fn extract_serialization(class_source: &str, fidelity: Fidelity) -> Option<MetaBlock> {
    let mut lines = Vec::new();

    // Check if class has any serialization attributes
    let has_serialization = class_source.contains("[JsonPropertyName")
        || class_source.contains("[JsonIgnore]")
        || class_source.contains("[JsonConverter]")
        || class_source.contains("[DataMember]")
        || class_source.contains("[IgnoreDataMember]")
        || class_source.contains("[Serializable]");

    if !has_serialization {
        return None;
    }

    // Extract class-level JSON configuration
    if fidelity != Fidelity::Low {
        lines.extend(extract_json_config(class_source));
    }

    // Extract property-level attributes
    if fidelity != Fidelity::Low {
        lines.extend(extract_property_attributes(class_source));
    }

    if lines.is_empty() {
        None
    } else {
        Some(MetaBlock { lines })
    }
}

/// Extract class-level JSON configuration.
fn extract_json_config(class_source: &str) -> Vec<String> {
    let mut configs = Vec::new();

    // Look for [JsonConverter] on class
    if let Some(pos) = class_source.find("[JsonConverter") {
        let rest = &class_source[pos + "[JsonConverter".len()..];
        if let Some(generic_pos) = rest.find('<') {
            let converter_rest = &rest[generic_pos + 1..];
            if let Some(generic_end) = converter_rest.find('>') {
                let converter = converter_rest[..generic_end].trim().to_string();
                configs.push(build_json_line(&format!("converter: {}", converter)));
            }
        }
    }

    // Look for [Serializable]
    if class_source.contains("[Serializable]") {
        configs.push(build_json_line("Serializable"));
    }

    configs
}

/// Extract property-level serialization attributes.
fn extract_property_attributes(class_source: &str) -> Vec<String> {
    let mut properties = Vec::new();

    // Look for [JsonPropertyName("...")]
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("[JsonPropertyName(") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "[JsonPropertyName(".len()..];

        // Extract the JSON property name
        if let Some(quote_start) = rest.find('"') {
            let name_rest = &rest[quote_start + 1..];
            if let Some(quote_end) = name_rest.find('"') {
                let json_name = name_rest[..quote_end].to_string();
                properties.push(build_property_line(&json_name));
            }
        }

        search_start = actual_pos + 1;
    }

    // Look for [DataMember]
    if class_source.contains("[DataMember]") {
        // Extract property names with [DataMember]
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find("[DataMember]") {
            let actual_pos = search_start + pos;
            let after = &class_source[actual_pos + "[DataMember]".len()..];

            if let Some(prop_pos) = after.find("public ") {
                let prop_decl = &after[prop_pos + "public ".len()..];
                if let Some(semicolon) = prop_decl.find(';') {
                    let prop = &prop_decl[..semicolon];
                    let prop_name = prop.split_whitespace().last().unwrap_or("");
                    if !prop_name.is_empty() {
                        properties.push(build_property_line(prop_name));
                    }
                }
            }

            search_start = actual_pos + 1;
        }
    }

    // Deduplicate and limit
    properties.dedup();
    properties.truncate(10);
    properties
}

#[cfg(test)]
#[path = "../tests/dotnet_meta/serialization.rs"]
mod tests;
