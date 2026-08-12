// src/angular_meta/routing.rs
//
// Routing Meta-Layer — Phase 4 of the Angular Ecosystem Deepening.
//
// Detects and compresses Angular Router constructs — `Routes` arrays,
// `RouterModule.forRoot`/`forChild`, lazy `loadComponent`/`loadChildren`,
// route guards, and resolvers — in Angular TypeScript files.
//
// # Purely additive
//
// The Routing meta-layer never modifies existing TS compression output.
// It only appends a `// --- Φ Routing Meta ---` block below the existing
// compacted class. Non-Routing files pay zero overhead (import-gate
// detection via `@angular/router`).

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ---------------------------------------------------------------------------
// RouteKind — single source of truth for Routing marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for Angular Router constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteKind {
    Route,
    Guard,
    Resolver,
}

impl PhiMarker for RouteKind {
    /// The `Φ` marker prefix for this kind.
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Route => "Φroute:",
            Self::Guard => "Φguard:",
            Self::Resolver => "Φresolver:",
        }
    }

    /// The human-readable expansion.
    fn expansion(self) -> &'static str {
        match self {
            Self::Route => "Route",
            Self::Guard => "Guard",
            Self::Resolver => "Resolver",
        }
    }

    /// All variants in a canonical order.
    fn all_in_expand_order() -> &'static [RouteKind] {
        &[
            Self::Resolver, // Φresolver: (10 chars)
            Self::Guard,    // Φguard:    (7 chars)
            Self::Route,    // Φroute:    (7 chars)
        ]
    }

    /// Look up a [`RouteKind`] by its marker token string.
    fn from_token(token: &str) -> Option<RouteKind> {
        match token {
            "Φroute" => Some(Self::Route),
            "Φguard" => Some(Self::Guard),
            "Φresolver" => Some(Self::Resolver),
            _ => None,
        }
    }

    /// Returns the token string.
    fn token(self) -> &'static str {
        match self {
            Self::Route => "Φroute",
            Self::Guard => "Φguard",
            Self::Resolver => "Φresolver",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single route declaration.
#[derive(Debug, Clone)]
pub struct RouteDecl {
    /// The route path (e.g. `"users"`, `""`, `"**"`).
    pub path: String,
    /// The eagerly-loaded component (e.g. `UserListComponent`).
    pub component: Option<String>,
    /// The lazily-loaded component (e.g. `() => import('./user-detail')`).
    pub load_component: Option<String>,
    /// The lazily-loaded children module (e.g. `() => import('./users')`).
    pub load_children: Option<String>,
    /// Guard names applied to this route (e.g. `AuthGuard`).
    pub guards: Vec<String>,
    /// Resolver names applied to this route (e.g. `UserResolver`).
    pub resolvers: Vec<String>,
}

/// A standalone guard declaration (class or function).
#[derive(Debug, Clone)]
pub struct GuardDecl {
    /// The guard name (e.g. `AuthGuard`).
    pub name: String,
    /// The guard kind (e.g. `CanActivate`, `CanLoad`, `CanDeactivate`).
    pub kind: String,
}

/// A standalone resolver declaration.
#[derive(Debug, Clone)]
pub struct ResolverDecl {
    /// The resolver name (e.g. `UserResolver`).
    pub name: String,
}

/// The complete Routing shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct RouteShape {
    pub routes: Vec<RouteDecl>,
    pub guards: Vec<GuardDecl>,
    pub resolvers: Vec<ResolverDecl>,
}

impl RouteShape {
    /// Returns `true` if there are no routing artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty() && self.guards.is_empty() && self.resolvers.is_empty()
    }

    /// Render the full `Φ Routing Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();
        s.push_str("// --- Φ Routing Meta ---\n");

        // Routes
        for route in &self.routes {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φroute:{}\n", route.path));
                }
                Fidelity::Medium | Fidelity::High => {
                    let mut parts: Vec<String> = Vec::new();
                    if let Some(ref c) = route.component {
                        parts.push(format!("component={}", c));
                    }
                    if let Some(ref lc) = route.load_component {
                        parts.push(format!("loadComponent={}", lc));
                    }
                    if let Some(ref lch) = route.load_children {
                        parts.push(format!("loadChildren={}", lch));
                    }
                    if !route.guards.is_empty() {
                        parts.push(format!("guards=[{}]", route.guards.join(",")));
                    }
                    if !route.resolvers.is_empty() {
                        parts.push(format!("resolvers=[{}]", route.resolvers.join(",")));
                    }
                    if parts.is_empty() {
                        s.push_str(&format!("  Φroute:{}\n", route.path));
                    } else {
                        s.push_str(&format!("  Φroute:{} {}\n", route.path, parts.join(" ")));
                    }
                }
            }
        }

        // Guards
        for guard in &self.guards {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φguard:{}\n", guard.name));
                }
                Fidelity::Medium | Fidelity::High => {
                    s.push_str(&format!("  Φguard:{} {}\n", guard.name, guard.kind));
                }
            }
        }

        // Resolvers
        for resolver in &self.resolvers {
            s.push_str(&format!("  Φresolver:{}\n", resolver.name));
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file uses Angular Router.
/// Returns true if the file imports from `@angular/router`.
pub fn has_router_imports(source: &str) -> bool {
    source.contains("@angular/router")
}

// ---------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------

/// Extract the Routing shape from a source file.
pub fn extract_route_shape(source: &str, _fidelity: Fidelity) -> Option<RouteShape> {
    if !has_router_imports(source) {
        return None;
    }
    let mut shape = RouteShape::default();

    // Extract route declarations from `Routes` arrays and
    // `RouterModule.forRoot(...)` / `forChild(...)` calls.
    extract_routes(source, &mut shape);

    // Extract standalone guard declarations.
    extract_guards(source, &mut shape);

    // Extract standalone resolver declarations.
    extract_resolvers(source, &mut shape);

    if shape.is_empty() {
        return None;
    }
    Some(shape)
}

/// Extract route objects from `Routes` arrays and `RouterModule` calls.
fn extract_routes(source: &str, shape: &mut RouteShape) {
    // Strategy: find `{ path: '...', ... }` objects that appear within
    // a `Routes` context. We scan for `path:` keys and parse the
    // enclosing object.
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("path:") {
        let abs_idx = search_from + idx;

        // Find the enclosing `{ ... }` object by scanning backwards
        // for the opening brace. We use the shared string-aware
        // primitive (Round-8 structural audit — same depth/string
        // awareness as every other meta-layer).
        let before = &source[..abs_idx];
        let open_brace = crate::angular_meta::util::find_enclosing_brace(before, before.len());
        if let Some(open) = open_brace {
            // Find the matching closing brace, starting from the
            // opening brace so bracket depth is balanced.
            let after = &source[open..];
            if let Some(close_rel) = crate::angular_meta::util::find_matching_brace(after, '{') {
                let close = open + close_rel;
                let obj = &source[open..=close];

                // Extract the path value.
                let path = crate::angular_meta::util::extract_quoted_value(obj, "path")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "?".to_string());

                // Extract component / loadComponent / loadChildren.
                let component = extract_ident_value(obj, "component");
                let load_component = extract_load_value(obj, "loadComponent");
                let load_children = extract_load_value(obj, "loadChildren");

                // Extract guards.
                let mut guards: Vec<String> = Vec::new();
                for key in ["canActivate", "canLoad", "canDeactivate", "canMatch"] {
                    if let Some(g) = extract_array_idents(obj, key) {
                        guards.extend(g);
                    }
                }

                // Extract resolvers.
                let mut resolvers: Vec<String> = Vec::new();
                if let Some(r) = extract_resolve_idents(obj) {
                    resolvers.extend(r);
                }

                shape.routes.push(RouteDecl {
                    path,
                    component,
                    load_component,
                    load_children,
                    guards,
                    resolvers,
                });

                // Advance past this object.
                search_from = close + 1;
                continue;
            }
        }

        // No enclosing object found — advance past this `path:`.
        search_from = abs_idx + 5;
    }
}

/// Extract standalone guard declarations (classes implementing
/// `CanActivate`/`CanLoad`/`CanDeactivate` or functions typed as such).
fn extract_guards(source: &str, shape: &mut RouteShape) {
    // Class-based guards: `class AuthGuard implements CanActivate {`
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("implements") {
        let abs_idx = search_from + idx;
        let before = &source[..abs_idx];

        // Find the class name: the token immediately after `class`.
        let class_name = class_name_before(before).unwrap_or_else(|| "?".to_string());

        // Extract the interface list after `implements`.
        let after = &source[abs_idx + "implements".len()..];
        let line_end = after.find(['\n', '{']).unwrap_or(after.len());
        let ifaces: Vec<String> = after[..line_end]
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        for iface in ifaces {
            if iface.starts_with("Can") {
                shape.guards.push(GuardDecl {
                    name: class_name.clone(),
                    kind: iface,
                });
            }
        }

        search_from = abs_idx + "implements".len() + 1;
    }

    // Function-based guards: `export const authGuard: CanActivateFn = ...`
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("CanActivateFn") {
        let abs_idx = search_from + idx;
        let before = &source[..abs_idx];

        // Skip matches inside import statements (e.g.
        // `import { CanActivateFn } from ...`).
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..abs_idx + "CanActivateFn".len()];
        if line.contains("import") {
            search_from = abs_idx + "CanActivateFn".len() + 1;
            continue;
        }

        // Find the variable name before the type annotation.
        let name = before.split_whitespace()
            .last()
            .map(|s| s.trim_end_matches(':').trim().to_string())
            .filter(|s| !s.is_empty() && *s != ":")
            .unwrap_or_else(|| "?".to_string());

        shape.guards.push(GuardDecl {
            name,
            kind: "CanActivateFn".to_string(),
        });

        search_from = abs_idx + "CanActivateFn".len() + 1;
    }
}

/// Extract standalone resolver declarations.
fn extract_resolvers(source: &str, shape: &mut RouteShape) {
    // Class-based resolvers: `class UserResolver implements Resolve<User> {`
    // Detect before `ResolveFn` so `Resolve<` doesn't match `ResolveFn<`.
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("Resolve<") {
        let abs_idx = search_from + idx;
        let before = &source[..abs_idx];

        // Find the class name: the token immediately after `class`.
        let class_name = class_name_before(before).unwrap_or_else(|| "?".to_string());

        shape.resolvers.push(ResolverDecl { name: class_name });

        search_from = abs_idx + "Resolve<".len() + 1;
    }

    // Function-based resolvers: `export const userResolver: ResolveFn<User> = ...`
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("ResolveFn") {
        let abs_idx = search_from + idx;
        let before = &source[..abs_idx];

        // Skip matches inside import statements (e.g.
        // `import { ResolveFn } from ...`).
        let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line = &source[line_start..abs_idx + "ResolveFn".len()];
        if line.contains("import") {
            search_from = abs_idx + "ResolveFn".len() + 1;
            continue;
        }

        // Find the variable name before the type annotation.
        let name = before.split_whitespace()
            .last()
            .map(|s| s.trim_end_matches(':').trim().to_string())
            .filter(|s| !s.is_empty() && *s != ":")
            .unwrap_or_else(|| "?".to_string());

        shape.resolvers.push(ResolverDecl { name });

        search_from = abs_idx + "ResolveFn".len() + 1;
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Find the class name in source text preceding an `implements` or
/// `Resolve<` keyword. The class name is the token immediately after
/// the `class` keyword.
fn class_name_before(before: &str) -> Option<String> {
    // Find the `class` keyword and take the next token.
    let tokens: Vec<&str> = before.split_whitespace().collect();
    for (i, tok) in tokens.iter().enumerate() {
        if *tok == "class" {
            if let Some(next) = tokens.get(i + 1) {
                return Some(next.trim_end_matches('{').to_string());
            }
        }
    }
    None
}

/// Extract an identifier value for a key in an object literal
/// (e.g. `component: UserListComponent`).
fn extract_ident_value(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("{}:", key);
    let idx = obj.find(&pattern)?;
    let after = &obj[idx + pattern.len()..];
    let after = after.trim_start();
    let end = after.find([',', '}', '\n']).unwrap_or(after.len());
    let ident = after[..end].trim();
    if ident.is_empty() || ident.starts_with('(') {
        None
    } else {
        Some(ident.to_string())
    }
}

/// Extract a lazy-load value for a key (e.g. `loadComponent: () => import('./x')`).
fn extract_load_value(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("{}:", key);
    let idx = obj.find(&pattern)?;
    let after = &obj[idx + pattern.len()..];
    let after = after.trim_start();
    // Find the import path in `() => import('./x')`.
    let import_idx = after.find("import(")?;
    let import_after = &after[import_idx + "import(".len()..];
    let import_after = import_after.trim_start();
    // Escape-aware first-quoted extraction (Round-8 audit): the shared
    // `extract_first_quoted` handles escaped quotes (`user\'s`), unlike
    // the old naive `rest.find(quote)` which truncated on the first
    // backslash-escaped quote.
    crate::angular_meta::util::extract_first_quoted(import_after)
}

/// Extract identifier names from an array value for a key
/// (e.g. `canActivate: [AuthGuard, AdminGuard]`).
fn extract_array_idents(obj: &str, key: &str) -> Option<Vec<String>> {
    let pattern = format!("{}:", key);
    let idx = obj.find(&pattern)?;
    let after = &obj[idx + pattern.len()..];
    let after = after.trim_start();
    if !after.starts_with('[') {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find(']')?;
    let idents: Vec<String> = rest[..end]
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if idents.is_empty() {
        None
    } else {
        Some(idents)
    }
}

/// Extract resolver identifiers from a `resolve: { key: Resolver }` object.
fn extract_resolve_idents(obj: &str) -> Option<Vec<String>> {
    let idx = obj.find("resolve:")?;
    let after = &obj[idx + "resolve:".len()..];
    let after = after.trim_start();
    if !after.starts_with('{') {
        return None;
    }
    let rest = &after[1..];
    let end = rest.find('}')?;
    let idents: Vec<String> = rest[..end]
        .split(',')
        .filter_map(|pair| {
            let pair = pair.trim();
            if pair.is_empty() {
                return None;
            }
            let colon = pair.find(':')?;
            let value = pair[colon + 1..].trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            }
        })
        .collect();
    if idents.is_empty() {
        None
    } else {
        Some(idents)
    }
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised Routing `Φ` marker in a line back to its
/// human-readable form.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<RouteKind>(line)
}

/// Expand a single Routing `Φ` marker token.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<RouteKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/routing.rs"]
mod tests;