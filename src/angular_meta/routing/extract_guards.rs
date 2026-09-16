// src/angular_meta/routing/extract_guards.rs
//
// Standalone Angular Router guard and resolver extraction (classes and
// functions implementing `CanActivate` / `CanLoad` / `CanDeactivate`, and
// `Resolve`-typed resolvers), plus the `class_name_before` helper they share.
//
// Split out of `src/angular_meta/routing.rs` (active-file size policy): the
// module had exceeded the 615-line ceiling. This is a pure relocation -- the
// code below is byte-for-byte the previous implementation.
//
// `extract_guards` and `extract_resolvers` are `pub(super)` because
// `routing::extract_route_shape` calls them; they stay module-internal, so the
// public surface of `routing` is unchanged.

use super::*;

/// Extract standalone guard declarations (classes implementing
/// `CanActivate`/`CanLoad`/`CanDeactivate` or functions typed as such).
pub(super) fn extract_guards(source: &str, shape: &mut RouteShape) {
    // Class-based guards: `class AuthGuard implements CanActivate {`
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("implements") {
        let abs_idx = search_from + idx;

        // Round-10 audit: skip `implements` matches inside comment lines
        // (e.g. `// implements CanActivate` or a doc comment).
        let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_trim = source[line_start..abs_idx].trim_start();
        if line_trim.starts_with("//") || line_trim.starts_with('*') {
            search_from = abs_idx + "implements".len();
            continue;
        }

        // Round-11 audit: reject matches inside trailing comments, block
        // comments, or string literals.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "implements".len();
            continue;
        }

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

        // Round-11 audit: reject matches inside comments or strings.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "CanActivateFn".len() + 1;
            continue;
        }

        // Find the variable name before the type annotation.
        let name = before
            .split_whitespace()
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
pub(super) fn extract_resolvers(source: &str, shape: &mut RouteShape) {
    // Class-based resolvers: `class UserResolver implements Resolve<User> {`
    // Detect before `ResolveFn` so `Resolve<` doesn't match `ResolveFn<`.
    let mut search_from = 0;
    while let Some(idx) = source[search_from..].find("Resolve<") {
        let abs_idx = search_from + idx;

        // Round-10 audit: skip `Resolve<` matches inside comment lines
        // (e.g. `// Resolve<User>` in a doc comment).
        let line_start = source[..abs_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let line_trim = source[line_start..abs_idx].trim_start();
        if line_trim.starts_with("//") || line_trim.starts_with('*') {
            search_from = abs_idx + "Resolve<".len();
            continue;
        }

        // Round-11 audit: reject matches inside trailing comments, block
        // comments, or string literals.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "Resolve<".len();
            continue;
        }

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

        // Round-11 audit: reject matches inside comments or strings.
        if crate::angular_meta::util::is_inside_comment_or_string(source, abs_idx) {
            search_from = abs_idx + "ResolveFn".len() + 1;
            continue;
        }

        // Find the variable name before the type annotation.
        let name = before
            .split_whitespace()
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
///
/// Round-10 audit: the old logic returned the FIRST `class` token in the
/// whole preceding region. For `class Foo {} class AuthGuard implements
/// CanActivate`, that produced `Foo` (the wrong guard/resolver name). We
/// now return the NEAREST preceding `class` (last match), and skip
/// `class` tokens on comment lines (a commented-out class must not
/// shadow the real declaration).
fn class_name_before(before: &str) -> Option<String> {
    let mut last: Option<String> = None;
    for line in before.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            continue;
        }
        let tokens: Vec<&str> = line.split_whitespace().collect();
        for (i, tok) in tokens.iter().enumerate() {
            if *tok == "class" {
                if let Some(next) = tokens.get(i + 1) {
                    // Overwrite on each match so the final value is the
                    // NEAREST preceding class declaration.
                    last = Some(next.trim_end_matches('{').to_string());
                }
            }
        }
    }
    last
}
