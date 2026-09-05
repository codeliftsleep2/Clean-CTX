// src/dotnet_meta/semantic.rs
//
// .NET-specific semantic edge construction helpers.
//
// These follow the same scanning patterns as the existing .NET meta-layer
// extractors (aspnet, efcore, signalr, automapper) but produce SemanticEdge
// objects instead of Phi marker strings.
//
// Phase 3 contract: zero duplication of existing Phi output. Semantic edges
// are a separate projection of the same framework information.

use crate::compression::Fidelity;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

/// Extract .NET semantic edges from a single class capture.
/// Reuses the same string-scanning patterns as the existing .NET extractors.
pub fn extract_dotnet_semantic_edges(
    raw_class: &str,
    class_name: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();

    // ── ASP.NET Core: Controller -> ControllerAction, Controller -> HasRoute ──
    let is_controller = raw_class.contains("[ApiController]")
        || raw_class.contains("[Controller]")
        || raw_class.contains(": ControllerBase")
        || raw_class.contains(": Controller");

    if is_controller {
        let controller = EntityRef::new("dotnet", "Controller", class_name);

        // Controller -> HasRoute -> route template
        if let Some(route) = extract_route(raw_class) {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasRoute,
                subject: controller.clone(),
                object: EntityRef::new("dotnet", "Route", &route),
                layer: "dotnet",
            });
        }

        // Controller -> ControllerAction -> action method (at Medium+ fidelity)
        if fidelity != Fidelity::Low {
            let actions = extract_actions(raw_class);
            for (method_name, _params, _return_type) in &actions {
                edges.push(SemanticEdge {
                    relation: SemanticRelation::ControllerAction,
                    subject: controller.clone(),
                    object: EntityRef::new("dotnet", "Action", method_name),
                    layer: "dotnet",
                });
            }
        }
    }

    // ── EF Core: DbContext -> HasEntity (via DbSet<T>) ──
    let is_dbcontext = raw_class.contains(": DbContext");
    if is_dbcontext {
        let dbcontext = EntityRef::new("dotnet", "DbContext", class_name);
        let entities = extract_dbset_entities(raw_class);
        for entity_name in &entities {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HasEntity,
                subject: dbcontext.clone(),
                object: EntityRef::new("dotnet", "Entity", entity_name),
                layer: "dotnet",
            });
        }
    }

    // ── AutoMapper: Profile -> MapsFrom/MapsTo (via CreateMap<TSource, TDest>) ──
    let is_profile = raw_class.contains(": Profile");
    if is_profile {
        let profile = EntityRef::new("dotnet", "MapperProfile", class_name);
        let mappings = extract_create_map_mappings(raw_class);
        for (source, dest) in &mappings {
            edges.push(SemanticEdge {
                relation: SemanticRelation::MapsFrom,
                subject: profile.clone(),
                object: EntityRef::new("dotnet", "Entity", source),
                layer: "dotnet",
            });
            edges.push(SemanticEdge {
                relation: SemanticRelation::MapsTo,
                subject: profile.clone(),
                object: EntityRef::new("dotnet", "Entity", dest),
                layer: "dotnet",
            });
        }
    }

    // ── SignalR: Hub -> HubMethodTargets ──
    let is_hub = raw_class.contains(": Hub<") || raw_class.contains(": Hub");
    if is_hub && fidelity != Fidelity::Low {
        let hub = EntityRef::new("dotnet", "Hub", class_name);
        let methods = extract_hub_methods(raw_class);
        for method_name in &methods {
            edges.push(SemanticEdge {
                relation: SemanticRelation::HubMethodTargets,
                subject: hub.clone(),
                object: EntityRef::new("dotnet", "HubMethod", method_name),
                layer: "dotnet",
            });
        }
    }

    // ── DI Bindings: implementation → abstraction/token ──
    edges.extend(extract_dotnet_di_bindings(raw_class));

    // ── Constructor consumption: declared stored dependencies (Phase 12) ──
    edges.extend(extract_dotnet_ctor_injects(raw_class, class_name, fidelity));

    edges
}

// ── Constructor consumption (Phase 12) ────────────────────────────────
//
// A C# constructor parameter represents DI consumption when the SAME class
// declares a field/property whose declared type matches the parameter type
// (exact source spelling except one trailing nullable `?` — Phase 11 rule).
// The emitted fact is a *declared dependency*: it does NOT claim container
// resolution, that a DI registration exists, or that the field is assigned
// from the parameter (assignment capture does not exist in this pipeline).
//
// Route: the class capture span (C-22) already contains the declaration,
// constructors, and members — the same projection route the Angular layer
// uses for its constructor injections. The dormant CoreOp::Injects/Param
// machinery is deliberately not activated.

/// Strip ONE trailing nullable annotation (`?`) for service-contract
/// comparison (Phase 11: nullable-insensitive exact text — no generic,
/// namespace, or alias normalization).
fn strip_trailing_nullable(ty: &str) -> &str {
    let trimmed = ty.trim_end();
    match trimmed.strip_suffix('?') {
        Some(stripped) => stripped.trim_end(),
        None => trimmed,
    }
}

/// Locate the class body: the first depth-zero `{` after the declaration
/// keyword (`class`/`struct`/`record`). Returns the inner-body byte range
/// `(open+1, close)`.
fn dotnet_class_body_range(raw_class: &str) -> Option<(usize, usize)> {
    const KEYWORDS: &[&str] = &["class ", "struct ", "record "];
    for kw in KEYWORDS {
        let mut from = 0usize;
        while let Some(rel) = raw_class[from..].find(kw) {
            let pos = from + rel;
            let prev = if pos == 0 {
                b' '
            } else {
                raw_class.as_bytes()[pos - 1]
            };
            if !prev.is_ascii_alphanumeric() && prev != b'_' {
                let open = crate::meta_util::find_first_top_level(raw_class, '{', pos + kw.len())?;
                let close = crate::meta_util::find_matching_brace(&raw_class[open..], '{')? + open;
                return Some((open + 1, close));
            }
            from = pos + kw.len();
        }
    }
    None
}

/// True when `s` ends with `name` as a standalone word.
fn ends_with_word(s: &str, name: &str) -> bool {
    match s.strip_suffix(name) {
        Some(before) => {
            before.is_empty()
                || !before
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_')
        }
        None => false,
    }
}

/// Split a parameter list on commas at top level, respecting nested `()`,
/// `[]`, `{}`, **and `<>`** groups plus string literals — so
/// `IRepository<Customer, Order>` remains one parameter.
fn split_ctor_params(text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut depth = 0i32;
    let mut generic = 0i32;
    let mut start = 0usize;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth > 0 => depth -= 1,
            '<' => generic += 1,
            '>' if generic > 0 => generic -= 1,
            '\'' => {
                crate::meta_util::skip_string(&mut chars, i, '\'');
            }
            '"' => {
                crate::meta_util::skip_string(&mut chars, i, '"');
            }
            c if c == ',' && depth == 0 && generic == 0 => {
                let seg = text[start..i].trim();
                if !seg.is_empty() {
                    segments.push(seg.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = text[start..].trim();
    if !tail.is_empty() {
        segments.push(tail.to_string());
    }
    segments
}

/// The service type declared by a single constructor parameter:
/// `Type name`, `Type? name`, `Type<T> name`, `ref/out/in/params Type name`.
/// Cuts default values at top-level `=`; splits the type at the last
/// top-level whitespace (generic-aware). Fails closed on ambiguous shapes.
fn param_service_type(param: &str) -> Option<String> {
    let mut s = param.trim();
    if s.is_empty() {
        return None;
    }
    // Cut default value (`= expr`) — the first top-level `=`; C# type
    // spellings never contain `=`.
    if let Some(eq) = s.find('=') {
        s = s[..eq].trim();
    }
    // Strip leading parameter modifiers.
    loop {
        let before = s;
        for kw in ["params ", "ref ", "out ", "in ", "this "] {
            if let Some(rest) = s.strip_prefix(kw) {
                s = rest.trim_start();
            }
        }
        if s == before {
            break;
        }
    }
    if s.is_empty() {
        return None;
    }
    // `Type name` — the type is everything before the last top-level
    // whitespace (scanning right-to-left, `>` opens and `<` closes).
    let mut generic = 0i32;
    for (idx, c) in s.char_indices().rev() {
        match c {
            '>' => generic += 1,
            '<' => generic -= 1,
            c if c.is_whitespace() && generic == 0 => {
                let ty = s[..idx].trim_end();
                let name = s[idx..].trim_start();
                if ty.is_empty() || name.is_empty() {
                    return None;
                }
                return Some(ty.to_string());
            }
            _ => {}
        }
    }
    None
}

/// The declared type of a field/property statement (attributes and
/// accessor/constructor bodies already excluded by the scan): cut
/// initializers and expression bodies at `=`, strip modifiers, split
/// `Type Name` at the last top-level whitespace. Fails closed on
/// ambiguous shapes.
fn member_decl_type(stmt: &str) -> Option<String> {
    let s = stmt.trim();
    if s.is_empty() || s.starts_with('[') || s.starts_with(']') {
        return None;
    }
    // Cut initializer / expression body (`= …`, including `=> …`).
    let s = match s.find('=') {
        Some(eq) => s[..eq].trim(),
        None => s,
    };
    if s.is_empty() || s.contains('(') || s.contains(')') {
        return None;
    }
    let stripped = crate::compaction::modifiers::strip_modifiers(
        &crate::compaction::modifiers::strip_modifiers(
            s,
            crate::compaction::modifiers::MODIFIERS_CLASS,
        ),
        crate::compaction::modifiers::MODIFIERS_FIELD,
    )
    .trim()
    .to_string();
    if stripped.is_empty() {
        return None;
    }
    let mut generic = 0i32;
    for (idx, c) in stripped.char_indices().rev() {
        match c {
            '>' => generic += 1,
            '<' => generic -= 1,
            c if c.is_whitespace() && generic == 0 => {
                let ty = stripped[..idx].trim_end();
                let name = stripped[idx..].trim_start();
                if ty.is_empty() || name.is_empty() {
                    return None;
                }
                return Some(ty.to_string());
            }
            _ => {}
        }
    }
    None
}

/// Extract `Binds` semantic edges from DI registrations.
///
/// For two-type registrations like `AddScoped<IService, Service>()`, emits:
/// `Service Binds IService` (implementation → abstraction/token).
///
/// Single-type registrations like `AddDbContext<T>()` are skipped — they have
/// no explicit abstraction/token endpoint.
pub fn extract_dotnet_di_bindings(raw_class: &str) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();

    let registrations = crate::dotnet_meta::general::extract_di_registrations_structured(raw_class);

    for reg in &registrations {
        let impl_type = match &reg.impl_type {
            Some(impl_type) => impl_type,
            None => continue, // Single-type registration: no Binds edge
        };

        // Subject = implementation, Object = abstraction/token
        // Direction: implementation → abstraction/token
        edges.push(SemanticEdge {
            relation: SemanticRelation::Binds,
            subject: EntityRef::new("dotnet", "Implementation", impl_type),
            object: EntityRef::new("dotnet", "Token", &reg.service),
            layer: "dotnet",
        });
    }

    edges
}
// ── Private Helpers (mirroring existing .NET extractor patterns) ──────────

/// Extract route template from [Route("...")] attribute.
fn extract_route(source: &str) -> Option<String> {
    if let Some(pos) = source.find("[Route(") {
        let start = pos + "[Route(".len();
        let rest = &source[start..];
        if let Some(quote_end) = rest.find('"') {
            let end = rest[quote_end + 1..].find('"')?;
            return Some(rest[quote_end + 1..quote_end + 1 + end].to_string());
        }
    }
    None
}

/// Extract action method names with HTTP verb attributes.
fn extract_actions(class_source: &str) -> Vec<(String, String, Option<String>)> {
    let verb_patterns = [
        "[HttpGet",
        "[HttpPost",
        "[HttpPut",
        "[HttpDelete",
        "[HttpPatch",
        "[HttpHead",
        "[HttpOptions",
    ];
    let mut actions = Vec::new();
    for attr in &verb_patterns {
        let mut search_start = 0;
        while let Some(pos) = class_source[search_start..].find(attr) {
            let actual_pos = search_start + pos;
            let rest = &class_source[actual_pos + attr.len()..];
            if let Some(method_info) = extract_method_signature(rest) {
                actions.push(method_info);
            }
            search_start = actual_pos + 1;
        }
    }
    actions
}

/// Extract method name, params, and optional return type.
fn extract_method_signature(source: &str) -> Option<(String, String, Option<String>)> {
    let visibility_patterns = ["public ", "private ", "protected ", "internal "];
    for vis in &visibility_patterns {
        if let Some(pos) = source.find(vis) {
            let start = pos + vis.len();
            let rest = &source[start..];
            if let Some(paren_pos) = rest.find('(') {
                let signature = &rest[..paren_pos];
                let params = &rest[paren_pos + 1..];
                let close_paren = params.find(')')?;
                let params_str = params[..close_paren].to_string();
                let method_name = signature.split_whitespace().last()?.to_string();
                let return_type = if signature.contains(' ') {
                    let parts: Vec<&str> = signature.rsplitn(2, ' ').collect();
                    if parts.len() == 2 {
                        Some(parts[0].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                };
                return Some((method_name, params_str, return_type));
            }
        }
    }
    None
}

/// Extract entity names from DbSet<T> properties.
fn extract_dbset_entities(class_source: &str) -> Vec<String> {
    let mut entities = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("DbSet<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "DbSet<".len()..];
        if let Some(generic_end) = rest.find('>') {
            let entity_name = rest[..generic_end].trim().to_string();
            if !entities.contains(&entity_name) {
                entities.push(entity_name);
            }
        }
        search_start = actual_pos + 1;
    }
    entities
}

/// Extract source/destination type pairs from CreateMap<TSource, TDest>() calls.
fn extract_create_map_mappings(class_source: &str) -> Vec<(String, String)> {
    let mut mappings = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("CreateMap<") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "CreateMap<".len()..];
        if let Some(generic_end) = rest.find('>') {
            let types = rest[..generic_end].trim().to_string();
            if let Some(comma_pos) = types.find(',') {
                let source = types[..comma_pos].trim().to_string();
                let dest = types[comma_pos + 1..].trim().to_string();
                mappings.push((source, dest));
            }
        }
        search_start = actual_pos + 1;
    }
    mappings
}

/// Extract class name from a .NET class declaration.
/// Follows the same pattern as the existing private extract_class_name()
/// functions in aspnet.rs, efcore.rs, etc.
pub fn extract_class_name_from_class(source: &str) -> Option<String> {
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

/// Extract hub method names from a SignalR Hub class.
fn extract_hub_methods(class_source: &str) -> Vec<String> {
    let mut methods = Vec::new();
    let mut search_start = 0;
    while let Some(pos) = class_source[search_start..].find("public ") {
        let actual_pos = search_start + pos;
        let rest = &class_source[actual_pos + "public ".len()..];
        if let Some(paren_pos) = rest.find('(') {
            let signature = &rest[..paren_pos];
            let method_name = signature
                .split_whitespace()
                .last()
                .unwrap_or("")
                .to_string();
            if !method_name.starts_with("On")
                && !method_name.starts_with("Dispose")
                && !method_name.is_empty()
            {
                methods.push(method_name);
            }
        }
        search_start = actual_pos + 1;
    }
    methods
}

/// One string/comment-aware pass over the class body collecting
/// (a) member statements (fields/properties) at member depth and
/// (b) constructor parameter lists (`<ClassName>(…)`, member depth).
/// Method declarations, method bodies, nested types, and attribute groups
/// are skipped; braces inside strings/comments do not affect depth.
fn scan_dotnet_class_body<'a>(body: &'a str, class_name: &str) -> (Vec<&'a str>, Vec<Vec<String>>) {
    let mut members: Vec<&'a str> = Vec::new();
    let mut ctor_param_lists: Vec<Vec<String>> = Vec::new();
    let mut depth = 0i32;
    let mut stmt_start: Option<usize> = None;
    let mut skip_until: Option<usize> = None;
    let mut chars = body.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if let Some(n) = skip_until {
            if i < n {
                continue;
            }
            skip_until = None;
        }
        match c {
            '"' => crate::meta_util::skip_string(&mut chars, i, '"'),
            '\'' => crate::meta_util::skip_string(&mut chars, i, '\''),
            '/' => match chars.peek().map(|(_, c2)| *c2) {
                Some('/') => {
                    for (_, c2) in chars.by_ref() {
                        if c2 == '\n' {
                            break;
                        }
                    }
                }
                Some('*') => {
                    chars.next();
                    let mut prev = '*';
                    for (_, c2) in chars.by_ref() {
                        if prev == '*' && c2 == '/' {
                            break;
                        }
                        prev = c2;
                    }
                }
                _ => {}
            },
            '[' if depth == 0 => {
                stmt_start = None;
                if let Some(close) = crate::meta_util::find_matching_brace(&body[i..], '[') {
                    skip_until = Some(i + close + 1);
                }
            }
            '{' => {
                if depth == 0 {
                    if let Some(st) = stmt_start.take() {
                        members.push(&body[st..i]);
                    }
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth < 0 {
                    depth = 0;
                }
            }
            ';' if depth == 0 => {
                if let Some(st) = stmt_start.take() {
                    members.push(&body[st..i]);
                }
            }
            '(' if depth == 0 => {
                let is_ctor = stmt_start
                    .map(|st| ends_with_word(body[st..i].trim(), class_name))
                    .unwrap_or(false);
                stmt_start = None;
                if is_ctor {
                    if let Some(close) = crate::meta_util::find_matching_brace(&body[i..], '(') {
                        let params_text = &body[i + 1..i + close];
                        ctor_param_lists.push(
                            split_ctor_params(params_text)
                                .iter()
                                .filter_map(|p| param_service_type(p))
                                .collect(),
                        );
                        skip_until = Some(i + close + 1);
                    }
                } else if let Some(close) = crate::meta_util::find_matching_brace(&body[i..], '(') {
                    skip_until = Some(i + close + 1);
                }
            }
            c if c.is_whitespace() && depth == 0 => {}
            _ if depth == 0 && stmt_start.is_none() => {
                stmt_start = Some(i);
            }
            _ => {}
        }
    }
    (members, ctor_param_lists)
}

/// Project constructor DI consumption into the generic substrate:
/// `builtin/Class/<consumer> → Injects → dotnet/Token/<service>` for every
/// constructor parameter whose type matches a same-class field/property
/// declared type (nullable-insensitive exact text).
pub fn extract_dotnet_ctor_injects(
    raw_class: &str,
    class_name: &str,
    fidelity: Fidelity,
) -> Vec<SemanticEdge> {
    let mut edges: Vec<SemanticEdge> = Vec::new();
    // Field/property fidelity must be Medium or High: Low-fidelity captures
    // carry no member type data (Phase 10/11 decision).
    if fidelity == Fidelity::Low || class_name.is_empty() {
        return edges;
    }
    let Some((body_open, body_close)) = dotnet_class_body_range(raw_class) else {
        return edges;
    };
    let body = &raw_class[body_open..body_close];
    let (member_stmts, ctor_param_lists) = scan_dotnet_class_body(body, class_name);
    let member_types: Vec<String> = member_stmts
        .iter()
        .filter_map(|s| member_decl_type(s))
        .collect();
    if member_types.is_empty() {
        return edges;
    }
    let mut emitted: Vec<String> = Vec::new();
    for params in &ctor_param_lists {
        for param_type in params {
            let service = strip_trailing_nullable(param_type);
            if service.is_empty() {
                continue;
            }
            let correlated = member_types
                .iter()
                .any(|m| strip_trailing_nullable(m) == service);
            if !correlated {
                continue;
            }
            if emitted.iter().any(|e| e == service) {
                continue;
            }
            emitted.push(service.to_string());
            edges.push(SemanticEdge {
                relation: SemanticRelation::Injects,
                subject: EntityRef::new("builtin", "Class", class_name),
                object: EntityRef::new("dotnet", "Token", service),
                layer: "dotnet",
            });
        }
    }
    edges
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/semantic.rs"]
mod tests;
