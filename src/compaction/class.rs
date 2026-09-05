// src/compaction/class.rs
//
// Class-level extraction and formatting helpers.
// Extended to support Rust structs, enums, and traits.

use crate::compaction::method::find_method_params;
use crate::compaction::modifiers::{
    MODIFIERS_CLASS, MODIFIERS_STRUCT_RS, strip_csharp_attributes, strip_modifiers,
};
use crate::compression::Fidelity;

/// Extract just the class name (and optional base/interface list) from the
/// raw text of a class declaration node.
///
/// Input examples:
///   "export class FooService extends BaseService implements IFoo { ... }"
///   "class Bar { ... }"
///   "abstract class Baz<T> { ... }"
///   "public static abstract class Quux { ... }"   <-- F-07 regression
///
/// Output examples (Low):   "FooService"
/// Output examples (Medium): "FooService:BaseService"
/// Output examples (High):   "FooService:BaseService,IFoo"
pub fn extract_class_name(text: &str) -> String {
    // C# captures may start with attribute lines
    // (`[ApiController]`, `[Route("api/[controller]")]`); strip them so
    // the declaration line is the actual `class` keyword line.
    let stripped = strip_csharp_attributes(text);
    // Take only the declaration line (everything before the first `{`)
    let decl = stripped.lines().next().unwrap_or(stripped);
    let decl = decl.split('{').next().unwrap_or(decl).trim();

    // Strip leading modifiers: export, default, abstract, public, sealed, …
    //
    // F-07: the previous implementation walked the modifier list once
    // and returned the first match, which broke for inputs like
    // "public static abstract class Foo" (it stripped "public " and
    // then returned, leaving "static abstract class Foo" behind).
    // The shared `strip_modifiers` helper loops until stable.
    let rest = strip_modifiers(decl, MODIFIERS_CLASS);
    // Strip "class " / "interface " / "record " keyword (C# interfaces and
    // records are distinct AST nodes but share the same class-like shape).
    // Non-CBM audit 2026-08-25 #1: also strip "enum " / "struct " so the
    // diff snapshot builder can route C#/TS/Java enums and structs through
    // this helper without the type keyword becoming the label.
    let rest = rest
        .strip_prefix("class ")
        .or_else(|| rest.strip_prefix("interface "))
        .or_else(|| rest.strip_prefix("record "))
        .or_else(|| rest.strip_prefix("enum "))
        .or_else(|| rest.strip_prefix("struct "))
        .unwrap_or(rest.as_str())
        .trim();

    // A C# primary-constructor parameter list (`record Example(string Value)`)
    // is part of the declaration header, never part of the class identity —
    // without this strip the whitespace tokenizer produced labels like
    // `Example(string` (regression test
    // `gitdiff_interpolation_does_not_bleed_into_signature_line`).
    let rest = strip_trailing_param_list(rest);

    // Split on whitespace: first token is "Name<T>" or "Name"
    let name_token = rest.split_whitespace().next().unwrap_or(rest);
    // Strip generic parameters for Low fidelity
    let bare_name = name_token.split('<').next().unwrap_or(name_token);

    // F-38: strip characters that would be ambiguous in the output format
    // (`{`, `}`, `:` are structural delimiters in the compact notation).
    let bare_name = bare_name.trim_end_matches(['{', '}', ':']);

    // Collect extends / implements if present
    let extends = extract_base_types(rest, "extends");
    let implements = extract_base_types(rest, "implements");

    match (extends.is_empty(), implements.is_empty()) {
        (true, true) => bare_name.to_string(),
        (false, true) => format!("{}:{}", bare_name, extends.join(",")),
        (true, false) => format!("{}:{}", bare_name, implements.join(",")),
        (false, false) => format!(
            "{}:{},{}",
            bare_name,
            extends.join(","),
            implements.join(",")
        ),
    }
}

/// Extract just the base class / interface list from a class declaration.
///
/// Input:  "public class FooService : BaseService, IFoo { ... }"
/// Output: ":BaseService,IFoo"
///
/// Input:  "class Bar { ... }"
/// Output: ""
///
/// The result is stored in `CapturedClass::class_meta` so a change to
/// the inheritance list is detected even when the class name is
/// unchanged. F-04 diff audit.
pub fn extract_class_meta(text: &str) -> String {
    let stripped = strip_csharp_attributes(text);
    let decl = stripped.lines().next().unwrap_or(stripped);
    let decl = decl.split('{').next().unwrap_or(decl).trim();
    let rest = strip_modifiers(decl, MODIFIERS_CLASS);
    // Mirror `extract_class_name`'s keyword chain (incl. enum/struct, see
    // Non-CBM audit 2026-08-25 #1) so both derive from the same decl text.
    let rest = rest
        .strip_prefix("class ")
        .or_else(|| rest.strip_prefix("interface "))
        .or_else(|| rest.strip_prefix("record "))
        .or_else(|| rest.strip_prefix("enum "))
        .or_else(|| rest.strip_prefix("struct "))
        .unwrap_or(rest.as_str())
        .trim();

    // Mirror `extract_class_name`'s primary-constructor strip so both derive
    // from the same decl text (see `strip_trailing_param_list`).
    let rest = strip_trailing_param_list(rest);

    // Find the `:` that separates the class name from the base list.
    // C# uses `: Base, IFoo`; TS uses `extends Base implements IFoo`.
    let mut meta = String::new();
    if let Some((_, after)) = rest.split_once(':') {
        // C# base list: everything after the first `:` up to the next
        // keyword or end of string.
        let after = after
            .split_once("where")
            .map(|(l, _)| l)
            .unwrap_or(after)
            .trim();
        if !after.is_empty() {
            meta.push(':');
            meta.push_str(after);
        }
    }
    // TS extends/implements
    let extends = extract_base_types(rest, "extends");
    let implements = extract_base_types(rest, "implements");
    if !extends.is_empty() {
        if !meta.is_empty() {
            meta.push(',');
        }
        meta.push_str(&extends.join(","));
    }
    if !implements.is_empty() {
        if !meta.is_empty() {
            meta.push(',');
        }
        meta.push_str(&implements.join(","));
    }
    meta
}

/// Strip a trailing C# primary-constructor parameter list from a
/// class-like declaration remainder.
///
/// Input is a declaration header after modifier/type-keyword removal:
///   "Example(string Value)"            → "Example"
///   "Range(int lo, int hi)"            → "Range"      (Java records)
///   "FooService : BaseService, IFoo"    → unchanged     (no parens)
///
/// A class/interface/record header may contain a depth-0 paren group only
/// as its primary-constructor parameter list; that group describes the
/// declared NAME, never identity or base metadata. The group is located
/// with the shared name-anchored locator `find_method_params` (first
/// depth-0 group anchored to the declared name — shared with
/// method-signature extraction, no second parser).
///
/// Guard: a group is peeled only when it CLOSES OUT the remainder (empty
/// tail or a lone `;`). A group followed by more text (`: Base(args)`,
/// `where T : Fn(u32)`) is left untouched, so base lists and constraint
/// clauses keep their legacy rendering; for such headers behavior is
/// byte-identical to before this helper existed.
fn strip_trailing_param_list(rest: &str) -> &str {
    let mut cur = rest.trim();
    while let Some((open, close)) = find_method_params(cur) {
        let tail = cur[close + 1..].trim();
        if !(tail.is_empty() || tail == ";") {
            break;
        }
        cur = cur[..open].trim();
    }
    cur
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
        Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
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
pub(crate) fn extract_base_types(decl: &str, keyword: &str) -> Vec<String> {
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

/// Extract name from a Rust struct, enum, trait, or impl declaration.
///
/// Input examples:
///   "pub struct MyStruct<T> where T: Clone { ... }"
///   "enum Status { ... }"
///   "trait Repository { ... }"
///   "pub(crate) struct Service { ... }"
///   "impl MyStruct { ... }"
///   "impl Display for MyStruct { ... }"
///   "impl<T> Repository<T> for PostgresRepo { ... }"
///
/// Output examples:
///   "MyStruct", "Status", "Repository", "MyStruct", "MyStruct:Display"
///   "PostgresRepo:Repository<T>" (Phase E: preserves trait generics)
pub fn extract_rust_struct_name(text: &str) -> String {
    // Take only the declaration line (everything before the first `{`)
    let decl = text.lines().next().unwrap_or(text);
    let decl = decl.split('{').next().unwrap_or(decl).trim();

    // Strip leading modifiers: pub, pub(crate), pub(super)
    let rest = strip_modifiers(decl, MODIFIERS_STRUCT_RS);

    // Trim where clause if present — captures the segment before `where`
    let (rest, _where_clause) = if let Some(where_pos) = rest.find(" where ") {
        (&rest[..where_pos], Some(rest[where_pos + 7..].trim()))
    } else {
        (rest.as_str(), None)
    };

    // Check if this is an impl block — extract meaningful name
    if let Some(impl_rest) = rest.strip_prefix("impl") {
        let impl_rest = impl_rest.trim();
        // "impl Trait for Type" → "Type:Trait"
        // "impl<T> Trait<T> for Type" → "Type:Trait<T>"
        if let Some(for_pos) = impl_rest.find(" for ") {
            let trait_part = impl_rest[..for_pos].trim();
            let type_part = impl_rest[for_pos + 5..].trim();

            // Phase E: extract names preserving generics
            let type_name = extract_name_with_generics(type_part);
            let trait_name = extract_trait_name_from_impl(trait_part);

            if !type_name.is_empty() && !trait_name.is_empty() {
                return format!("{}:{}", type_name, trait_name);
            }
        }
        // Inherent impl "impl Type" → just the type name (with generics)
        return extract_name_with_generics(impl_rest);
    }

    // Strip struct/enum/trait keyword
    let rest = rest
        .strip_prefix("struct ")
        .or_else(|| rest.strip_prefix("enum "))
        .or_else(|| rest.strip_prefix("trait "))
        .unwrap_or(rest)
        .trim();

    // Extract name (up to whitespace, keeping generics intact)
    let name = rest.split_whitespace().next().unwrap_or(rest);
    let name = name.trim_end_matches(['{', '}', ':']);

    name.to_string()
}

/// Extract a name with its generic parameters from a type string.
///
/// Takes the first whitespace-delimited token and preserves its `<...>` generics.
/// For example: "PostgresRepo" → "PostgresRepo", "Vec<T>" → "Vec<T>"
/// Also handles leading generic parameters like "<T> Cache<T>" → "Cache<T>"
fn extract_name_with_generics(text: &str) -> String {
    let text = text.trim();
    // Skip past any leading generic parameters <...>
    let text = if text.starts_with('<') {
        let mut depth = 0i32;
        let mut past_generics = text;
        for (i, ch) in text.char_indices() {
            match ch {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        past_generics = text[i + 1..].trim();
                        break;
                    }
                }
                _ => {}
            }
        }
        past_generics
    } else {
        text
    };
    let name = text.split_whitespace().next().unwrap_or(text);
    name.trim_end_matches(['{', '}', ':']).to_string()
}

/// Extract the trait name (with its own generics) from an impl block's trait part.
///
/// The trait part may be:
///   - "Repository<T>" → "Repository<T>"
///   - "impl<T> Repository<T>" → "Repository<T>"
///   - "Display" → "Display"
///
/// We skip the impl-level generic parameters `<T>` (which come after "impl")
/// and extract the trait name + its own generics.
fn extract_trait_name_from_impl(trait_part: &str) -> String {
    // Skip past "impl" keyword and any impl-level generics
    let after_impl = trait_part.strip_prefix("impl").unwrap_or(trait_part).trim();

    // Skip past any impl-level generic parameters <...>
    let mut depth = 0i32;
    let mut past_generics = after_impl;
    for (i, ch) in after_impl.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    past_generics = after_impl[i + 1..].trim();
                    break;
                }
            }
            _ => {}
        }
    }

    // If depth > 0, we never closed all generics — just use the whole thing
    if depth > 0 {
        past_generics = after_impl;
    }

    // Extract the trait name (preserving its own generics)
    // "Repository<T>" → "Repository<T>" (split_whitespace gives the whole token)
    let trait_name = past_generics
        .split_whitespace()
        .next()
        .unwrap_or(past_generics)
        .trim()
        .trim_end_matches(['{', '}', ':']);

    trait_name.to_string()
}

/// Format a Rust type entry line, embedding any accumulated field signatures.
/// Unlike `format_class_entry`, this does NOT prepend "class".
///
/// Low:    "MyStruct{field1;field2}"
/// Medium: "MyStruct { field1; field2 }"
/// High:   "MyStruct {\n  field1\n  field2\n}"
pub fn format_rust_type_entry(name: &str, fields: &[String], fidelity: Fidelity) -> String {
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
                format!("{} {{", name)
            } else {
                format!("{} {{ {} }}", name, fields.join("; "))
            }
        }
        Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
            if fields.is_empty() {
                format!("{} {{", name)
            } else {
                let field_lines = fields
                    .iter()
                    .map(|f| format!("  {}", f))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{} {{\n{}", name, field_lines)
            }
        }
    }
}

#[cfg(test)]
#[path = "../tests/compaction/class.rs"]
mod tests;
