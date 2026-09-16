// src/angular_meta/constructor_injects.rs
//
// Angular constructor dependency-injection extraction.
//
// # Semantic contract
//
// For an Angular DI-participating class, a *typed* constructor parameter is a
// constructor injection regardless of whether it also declares a TypeScript
// parameter property. `private` / `public` / `protected` / `readonly` control
// TypeScript property generation — they do not decide whether Angular injects
// the parameter. These are equivalent for DI:
//
//     constructor(private foo: FooService) {}
//     constructor(public foo: FooService) {}
//     constructor(protected foo: FooService) {}
//     constructor(readonly foo: FooService) {}
//     constructor(private readonly foo: FooService) {}
//     constructor(foo: FooService) {}
//
// # Applicability gate (not decided here)
//
// Injection identity is the parameter *type*. Callers reach this extractor
// only for class captures already recognized as Angular classes:
// `extract_graph_entries` / `extract_decorators` require a `@Component` /
// `@Injectable` / `@Directive` / `@Pipe` / `@NgModule` decorator before this
// function is called (`src/angular_meta/decorators.rs`). Ordinary TypeScript
// constructors never reach it, so they cannot gain Angular `Injects` edges.
//
// # Type eligibility (unchanged)
//
// Eligibility is the pre-existing rule and is neither tightened nor loosened
// by the modifier fix: a parameter contributes when it declares a type whose
// leading identifier characters are non-empty. Parameter decorators
// (`@Inject`, `@Optional`, `@Attribute`, …) are stripped for the same reason
// modifiers are — they are DI metadata, not the parameter name — so the
// injection *identity* stays the parameter type whether or not they appear.

use crate::angular_meta::decorator_scan::is_word_byte;
use crate::meta_util::{consume_call_expression, split_top_level};

/// TypeScript parameter-property modifiers. They are stripped, never
/// required: a modifier is TypeScript property-generation sugar, not the
/// Angular DI signal.
const PARAMETER_PROPERTY_MODIFIERS: &[&str] = &["public", "private", "protected", "readonly"];

/// Extract the constructor-injected parameter types of an Angular class
/// capture, in declaration order.
///
/// Returns `None` when the class has no constructor, no parameters, or no
/// parameter that contributes an injection type (the pre-existing
/// "no injects" contract).
pub(crate) fn extract_constructor_injects(raw_class: &str) -> Option<Vec<String>> {
    let body_start = raw_class.find('{')?;
    let body = &raw_class[body_start..];

    let open = find_constructor_open_paren(body, body_start)?;
    // F-ANG-09: an unterminated constructor has no params (no
    // injects to extract). Fall back to an empty param list.
    let params = consume_call_expression(raw_class, open)
        .map(|(_, p)| p)
        .unwrap_or_default();

    let types: Vec<String> = split_top_level(&params, ',')
        .iter()
        .filter_map(|param| constructor_param_inject_type(param.as_str()))
        .collect();

    if types.is_empty() { None } else { Some(types) }
}

/// Locate the `(` that opens the constructor parameter list, as an absolute
/// byte offset into `raw_class`.
///
/// `body` is the slice of `raw_class` starting at `body_start`, so the scan
/// stays equivalent to the original implementation (including the
/// word-boundary guard that keeps `constructorLike(...)` from matching).
fn find_constructor_open_paren(body: &str, body_start: usize) -> Option<usize> {
    let mut search_from = 0;
    while let Some(pos) = body[search_from..].find("constructor") {
        let abs = search_from + pos;
        let before_ok = abs == 0 || !is_word_byte(body.as_bytes()[abs - 1]);
        let after_pos = abs + "constructor".len();
        let after_ok = after_pos >= body.len() || !is_word_byte(body.as_bytes()[after_pos]);
        if before_ok && after_ok {
            let rest = &body[after_pos..];
            let trimmed = rest.trim_start();
            if let Some(after_trim) = trimmed.find('(') {
                return Some(body_start + after_pos + (rest.len() - trimmed.len()) + after_trim);
            }
        }
        search_from = abs + 1;
    }
    None
}

/// Resolve the injection type contributed by a single constructor parameter.
///
/// Returns `None` when the parameter declares no usable type.
///
/// The parse is deliberately tolerant of formatting: leading parameter
/// decorators and parameter-property modifiers may be separated from the
/// parameter name by spaces, tabs, or newlines.
fn constructor_param_inject_type(param: &str) -> Option<String> {
    let rest = strip_parameter_modifiers(strip_parameter_decorators(param));
    let colon = rest.find(':')?;
    let type_part = rest[colon + 1..].trim();
    let type_part = type_part.split('=').next().unwrap_or(type_part).trim();
    let type_name: String = type_part
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if type_name.is_empty() {
        None
    } else {
        Some(type_name)
    }
}

/// Remove leading parameter decorators (`@Inject(TOKEN)`, `@Optional()`,
/// `@Self()`, `@Inject(core.APP_ID)`, …) so the parameter name and type
/// behind them stay reachable.
///
/// Decorator arguments are intentionally discarded: the semantic model uses
/// the parameter *type* as the injection identity, and the modifier fix does
/// not redesign injection-token semantics.
fn strip_parameter_decorators(param: &str) -> &str {
    let mut rest = param.trim_start();
    loop {
        let Some(after_at) = rest.strip_prefix('@') else {
            return rest;
        };
        // Decorator name — identifier, optionally dotted/namespaced.
        let name_end = after_at
            .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '$' || c == '.'))
            .unwrap_or(after_at.len());
        let after_name = &after_at[name_end..];
        let after_name_trimmed = after_name.trim_start();
        // Argument group, when present: `@Inject(TOKEN)`.
        if after_name_trimmed.starts_with('(') {
            let open = 1 + name_end + (after_name.len() - after_name_trimmed.len());
            match consume_call_expression(rest, open) {
                Some((consumed, _)) => {
                    rest = &rest[open + consumed..];
                    continue;
                }
                // Unterminated decorator call — nothing left to strip.
                None => return after_name_trimmed,
            }
        }
        rest = after_name_trimmed;
    }
}

/// Remove leading parameter-property modifiers (`public`, `private`,
/// `protected`, `readonly`) in any valid combination.
///
/// A modifier is consumed only as a whole word followed by whitespace, so an
/// identifier that merely starts with a modifier keyword (`readonlyState`,
/// `privateKey`) is never truncated.
fn strip_parameter_modifiers(param: &str) -> &str {
    let mut rest = param.trim_start();
    loop {
        let mut advanced = false;
        for modifier in PARAMETER_PROPERTY_MODIFIERS {
            if let Some(after) = rest.strip_prefix(modifier)
                && after.starts_with(char::is_whitespace)
            {
                rest = after.trim_start();
                advanced = true;
                break;
            }
        }
        if !advanced {
            return rest;
        }
    }
}

#[cfg(all(test, feature = "angular"))]
#[path = "../tests/angular_meta/constructor_injects.rs"]
mod tests;
