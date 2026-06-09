// src/angular_meta/style.rs
//
// Style file extractor — Tier 2 of the Meta-Layer.
//
// Parses `.scss` / `.css` / `.sass` / `.less` files and extracts
// structural shape: class selectors, SCSS/CSS variables, and
// at-rules (`@include`, `@mixin`). Raw CSS content is NEVER
// included — only the structural summary.
//
// The output is a single-line shape summary suitable for a `Φsty:`
// marker in the workspace manifest.

/// Structural shape of a style file, suitable for a one-line
/// summary in the workspace manifest.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleShape {
    /// Top-level class selectors (e.g. `.card`, `.btn-primary`).
    pub class_selectors: Vec<String>,
    /// SCSS/CSS variables (e.g. `$primary-color`, `--bg-color`).
    pub variables: Vec<String>,
    /// At-rules referenced (e.g. `@include`, `@mixin`).
    pub at_rules: Vec<String>,
    /// F-FULL-13: marker for parser failure on corrupt input.
    pub parse_failed: bool,
}

impl StyleShape {
    /// Format as a single-line shape summary. Example output:
    ///
    /// ```text
    /// Φsty:.card,.btn $primary-color,$bg @include,@mixin
    /// ```
    pub fn to_marker_line(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        if !self.class_selectors.is_empty() {
            let selectors: Vec<&str> = self
                .class_selectors
                .iter()
                .take(8)
                .map(|s| s.as_str())
                .collect();
            parts.push(selectors.join(","));
        }

        if !self.variables.is_empty() {
            let vars: Vec<&str> = self.variables.iter().take(6).map(|s| s.as_str()).collect();
            parts.push(format!("${}", vars.join(",")));
        }

        if !self.at_rules.is_empty() {
            let rules: Vec<&str> = self.at_rules.iter().take(4).map(|s| s.as_str()).collect();
            parts.push(format!("@{}", rules.join(",")));
        }

        // F-FULL-13: distinguish parser failure from empty style
        if self.parse_failed {
            return "Φsty:PARSE_ERROR".to_string();
        }
        if parts.is_empty() {
            "Φsty:empty".to_string()
        } else {
            format!("Φsty:{}", parts.join(" "))
        }
    }
}

/// Extract the structural shape of a CSS/SCSS style file.
///
/// This is a lightweight regex-free parser that scans for:
/// - Class selectors (`.name`)
/// - SCSS variables (`$var-name`)
/// - CSS custom properties (`--var-name`)
/// - At-rules (`@include`, `@mixin`, `@import`, etc.)
///
/// The extraction respects quoted strings and comments to avoid
/// false positives.
pub fn extract_style_shape(css: &str) -> StyleShape {
    let mut shape = StyleShape::default();

    if css.trim().is_empty() {
        return shape;
    }

    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let c = bytes[i];

        // Skip single-line comments.
        if c == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            i += 2;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }

        // Skip multi-line comments.
        if c == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        }

        // Skip quoted strings.
        if c == b'"' || c == b'\'' {
            let quote = c;
            i += 1;
            while i < len && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < len {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            i += 1;
            continue;
        }

        // Class selector: `.name` (but not inside a property value
        // context — we only capture top-level-ish selectors).
        if c == b'.' && i + 1 < len && bytes[i + 1].is_ascii_alphabetic() {
            i += 1;
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            let name = &css[start..i];
            if !name.is_empty() {
                shape.class_selectors.push(format!(".{}", name));
            }
            continue;
        }

        // SCSS variable: `$var-name`.
        if c == b'$' && i + 1 < len && bytes[i + 1].is_ascii_alphabetic() {
            i += 1;
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            let name = &css[start..i];
            if !name.is_empty() {
                shape.variables.push(format!("${}", name));
            }
            continue;
        }

        // CSS custom property: `--var-name`.
        if c == b'-' && i + 1 < len && bytes[i + 1] == b'-' {
            i += 2;
            let start = i;
            while i < len && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'-' || bytes[i] == b'_') {
                i += 1;
            }
            let name = &css[start..i];
            if !name.is_empty() {
                shape.variables.push(format!("--{}", name));
            }
            continue;
        }

        // At-rule: `@name`.
        if c == b'@' && i + 1 < len && bytes[i + 1].is_ascii_alphabetic() {
            i += 1;
            let start = i;
            while i < len && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let name = &css[start..i];
            if !name.is_empty() {
                // Only collect common at-rules, not @media, @keyframes, etc.
                // (`@forward` is SCSS module re-export; F-ANG-19.)
                if matches!(name, "include" | "mixin" | "import" | "use" | "forward")
                    && !shape.at_rules.contains(&format!("@{}", name))
                {
                    shape.at_rules.push(format!("@{}", name));
                }
            }
            continue;
        }

        i += 1;
    }

    // Deduplicate.
    shape.class_selectors.sort();
    shape.class_selectors.dedup();
    shape.variables.sort();
    shape.variables.dedup();

    shape
}

#[cfg(test)]
#[path = "../tests/angular_meta/style.rs"]
mod tests;