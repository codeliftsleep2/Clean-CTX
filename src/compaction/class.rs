// src/compaction/class.rs
//
// Class-level extraction and formatting helpers.

use crate::compression::Fidelity;

/// Extract just the class name (and optional base/interface list) from the
/// raw text of a class declaration node.
///
/// Input examples:
///   "export class FooService extends BaseService implements IFoo { ... }"
///   "class Bar { ... }"
///   "abstract class Baz<T> { ... }"
///
/// Output examples (Low):   "FooService"
/// Output examples (Medium): "FooService:BaseService"
/// Output examples (High):   "FooService:BaseService,IFoo"
pub fn extract_class_name(text: &str) -> String {
    // Take only the declaration line (everything before the first `{`)
    let decl = text.lines().next().unwrap_or(text);
    let decl = decl.split('{').next().unwrap_or(decl).trim();

    // Strip leading modifiers: export, default, abstract, public, sealed, …
    let keywords = ["export default ", "export ", "abstract ", "sealed ",
                    "public ", "private ", "protected ", "static "];
    let mut rest = decl;
    for kw in &keywords {
        rest = rest.strip_prefix(kw).unwrap_or(rest);
    }
    // Strip "class " keyword
    let rest = rest.strip_prefix("class ").unwrap_or(rest).trim();

    // Split on whitespace: first token is "Name<T>" or "Name"
    let name_token = rest.split_whitespace().next().unwrap_or(rest);
    // Strip generic parameters for Low fidelity
    let bare_name = name_token.split('<').next().unwrap_or(name_token);

    // Collect extends / implements if present
    let extends = extract_base_types(rest, "extends");
    let implements = extract_base_types(rest, "implements");

    match (extends.is_empty(), implements.is_empty()) {
        (true, true) => bare_name.to_string(),
        (false, true) => format!("{}:{}", bare_name, extends.join(",")),
        (true, false) => format!("{}:{}", bare_name, implements.join(",")),
        (false, false) => format!("{}:{},{}", bare_name, extends.join(","), implements.join(",")),
    }
}

/// Format a class entry line, embedding any accumulated field signatures.
///
/// Low:    "ClassName{field1;field2}"  (compact, no spaces)
/// Medium: "ClassName { field1; field2 }"
/// High:   "class ClassName {\n  field1\n  field2\n}"
pub fn format_class_entry(name: &str, fields: &[String], fidelity: Fidelity) -> String {
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
                format!("class {} {{", name)
            } else {
                format!("class {} {{ {} }}", name, fields.join("; "))
            }
        }
        Fidelity::High => {
            if fields.is_empty() {
                format!("class {} {{", name)
            } else {
                let field_lines = fields
                    .iter()
                    .map(|f| format!("  {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("class {} {{\n{}", name, field_lines)
            }
        }
    }
}

/// Pull the type names that follow a given keyword (`extends` or `implements`)
/// out of a class declaration string.
fn extract_base_types(decl: &str, keyword: &str) -> Vec<String> {
    let Some(after) = decl.split_once(keyword) else {
        return Vec::new();
    };
    let segment = after.1;
    // Everything up to the next keyword or end of string
    let segment = segment
        .split_once("implements")
        .map(|(l, _)| l)
        .unwrap_or(segment);
    let segment = segment
        .split_once("extends")
        .map(|(l, _)| l)
        .unwrap_or(segment);

    segment
        .split(',')
        .map(|s| {
            // Strip generic parameters
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
