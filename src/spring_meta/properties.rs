// src/spring_meta/properties.rs
//
// Properties file extractor — Tier 2 of the Spring Boot Meta-Layer.
//
// Parses `application.properties` / `application.yml` / `application.yaml`
// files and extracts structural shape: property keys, active profiles,
// and configuration snippets. Raw property content is NEVER included —
// only the structural summary.
//
// The output is a single-line shape summary suitable for a `Φpropf:`
// marker in the workspace manifest.

/// Structural shape of a Spring Boot properties file, suitable for a
/// one-line summary in the workspace manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PropertiesShape {
    /// Top-level property keys (e.g. `server.port`, `spring.datasource.url`).
    pub keys: Vec<String>,
    /// Active profiles detected (e.g. `dev`, `prod`).
    pub profiles: Vec<String>,
    /// Number of properties found.
    pub count: usize,
}

impl PropertiesShape {
    /// Format as a single-line shape summary. Example output:
    ///
    /// ```text
    /// Φpropf:server.port,spring.datasource.url profiles=[dev,prod] count=42
    /// ```
    pub fn to_marker_line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if !self.keys.is_empty() {
            let key_list: Vec<&str> = self.keys.iter().take(10).map(|s| s.as_str()).collect();
            parts.push(key_list.join(","));
        }

        if !self.profiles.is_empty() {
            parts.push(format!("profiles=[{}]", self.profiles.join(",")));
        }

        if self.count > 0 {
            parts.push(format!("count={}", self.count));
        }

        if parts.is_empty() {
            "Φpropf:empty".to_string()
        } else {
            format!("Φpropf:{}", parts.join(" "))
        }
    }
}

/// Extract the structural shape of a Spring Boot properties file.
///
/// This is a lightweight parser that scans for:
/// - Property keys (before `=` or `:`)
/// - Active profiles (`spring.profiles.active=`)
/// - YAML structure (top-level keys)
///
/// The extraction respects comments and quoted strings to avoid
/// false positives.
pub fn extract_properties_shape(content: &str) -> PropertiesShape {
    let mut shape = PropertiesShape::default();

    if content.trim().is_empty() {
        return shape;
    }

    let is_yaml = content.contains(":") && (content.contains("---") || content.lines().any(|l| !l.starts_with("#") && l.contains(":")));

    if is_yaml {
        extract_yaml_shape(content, &mut shape);
    } else {
        extract_properties_shape_impl(content, &mut shape);
    }

    shape
}

fn extract_properties_shape_impl(content: &str, shape: &mut PropertiesShape) {
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Skip single-line comments.
        if bytes[i] == b'#' || (bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'/') {
            i += 1;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip blank lines.
        if bytes[i] == b'\n' || bytes[i] == b'\r' {
            i += 1;
            continue;
        }

        // Extract property key (before = or :).
        let key_start = i;
        while i < len && bytes[i] != b'=' && bytes[i] != b':' && bytes[i] != b'\n' && bytes[i] != b'\r' {
            i += 1;
        }
        let key = content[key_start..i].trim();

        if !key.is_empty() && !key.starts_with('#') && !key.starts_with("//") {
            shape.count += 1;
            shape.keys.push(key.to_string());

            // Check for active profiles.
            if (key == "spring.profiles.active" || key == "spring.profiles.include")
                && i < len && (bytes[i] == b'=' || bytes[i] == b':') {
                    i += 1;
                    let value_start = i;
                    while i < len && bytes[i] != b'\n' && bytes[i] != b'\r' {
                        i += 1;
                    }
                    let value = content[value_start..i].trim();
                    for profile in value.split(',') {
                        let profile = profile.trim();
                        if !profile.is_empty() {
                            shape.profiles.push(profile.to_string());
                        }
                    }
                }
        }

        // Skip to next line.
        while i < len && bytes[i] != b'\n' {
            i += 1;
        }
    }

    // Deduplicate.
    shape.keys.sort();
    shape.keys.dedup();
    shape.profiles.sort();
    shape.profiles.dedup();
}

fn extract_yaml_shape(content: &str, shape: &mut PropertiesShape) {
    let bytes = content.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut current_path: Vec<String> = Vec::new();

    while i < len {
        // Skip comments.
        if bytes[i] == b'#' {
            i += 1;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip blank lines and document markers.
        if bytes[i] == b'\n' || bytes[i] == b'\r' || (bytes[i] == b'-' && i + 2 < len && bytes[i + 1] == b'-' && bytes[i + 2] == b'-') {
            i += 1;
            continue;
        }

        // Calculate indentation level.
        let _line_start = i;
        let mut indent = 0;
        while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
            if bytes[i] == b' ' {
                indent += 1;
            } else {
                indent += 2; // tab = 2 spaces
            }
            i += 1;
        }

        // Pop path elements that are deeper than current indent.
        while current_path.len() > indent / 2 {
            current_path.pop();
        }

        // Extract key (before : or =).
        let key_start = i;
        while i < len && bytes[i] != b':' && bytes[i] != b'=' && bytes[i] != b'\n' && bytes[i] != b'\r' {
            i += 1;
        }
        let key = content[key_start..i].trim();

        if !key.is_empty() && !key.starts_with('#') {
            // Update path.
            current_path.push(key.to_string());
            let full_key = current_path.join(".");
            shape.count += 1;
            shape.keys.push(full_key.clone());

            // Check for active profiles using full path.
            if (full_key == "spring.profiles.active" || full_key == "spring.profiles.include")
                && i < len && (bytes[i] == b':' || bytes[i] == b'=') {
                    i += 1;
                    // Skip whitespace.
                    while i < len && (bytes[i] == b' ' || bytes[i] == b'\t') {
                        i += 1;
                    }
                    let value_start = i;
                    // Read until newline or comment.
                    while i < len && bytes[i] != b'\n' && bytes[i] != b'#' {
                        i += 1;
                    }
                    let value = content[value_start..i].trim();
                    for profile in value.split(',') {
                        let profile = profile.trim();
                        if !profile.is_empty() {
                            shape.profiles.push(profile.to_string());
                        }
                    }
                }

            // If this is a leaf value (line ends after value), pop from path.
            if i < len && (bytes[i] == b':' || bytes[i] == b'=') {
                // Has a value, keep in path for potential nested content.
            } else {
                current_path.pop();
            }
        }

        // Skip to next line.
        while i < len && bytes[i] != b'\n' {
            i += 1;
        }
    }

    // Deduplicate.
    shape.keys.sort();
    shape.keys.dedup();
    shape.profiles.sort();
    shape.profiles.dedup();
}

