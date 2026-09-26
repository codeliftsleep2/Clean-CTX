// RxJS observable and subject declaration extraction.

use super::{ObservableDecl, RxShape, SubjectDecl, SubjectKind, has_rxjs_imports};
use crate::compression::Fidelity;

/// Extract the RxJS shape from a source file.
///
/// Returns `None` when the file has no RxJS imports (zero overhead).
/// Returns `Some(RxShape)` with detected observables, subjects, pipes,
/// and combinators.
pub fn extract_rx_shape(source: &str, _fidelity: Fidelity) -> Option<RxShape> {
    // Import gate: skip non-RxJS files
    if !has_rxjs_imports(source) {
        return None;
    }

    let mut shape = RxShape::default();

    // Extract observable field declarations
    extract_observables(source, &mut shape);

    // Extract subject instantiations
    extract_subjects(source, &mut shape);

    // Extract pipe chains
    super::pipes::extract_pipe_chains(source, &mut shape);

    // Extract static combinator calls
    super::pipes::extract_combinators(source, &mut shape);

    if shape.is_empty() {
        return None;
    }

    Some(shape)
}

/// Extract observable field declarations from the source.
///
/// Detects:
/// - `Observable<T>` type annotations
/// - `$`-suffixed field names
/// - Creation functions: `of(...)`, `from(...)`, `interval(...)`,
///   `timer(...)`, `fromEvent(...)`
/// - Service calls: `http.get(...)`, `this.http.get(...)`
fn extract_observables(source: &str, shape: &mut RxShape) {
    // Scan line-by-line for observable patterns. Track the absolute byte
    // offset of each line so matches inside trailing comments or string
    // literals can be rejected (Round-11 audit: a `// users$ = of(1)`
    // trailing comment or a string containing `Observable<` must not
    // produce a phantom observable).
    let mut line_start = 0usize;
    for line in source.split('\n') {
        let trimmed = line.trim();

        // Skip comment-only lines
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            line_start += line.len() + 1;
            continue;
        }

        // Absolute offset of the trimmed line within `source`.
        let leading = line.len() - line.trim_start().len();
        let trimmed_abs = line_start + leading;

        // Pattern: `name$: Observable<T>` or `name$ = http.get(...)`
        // or `name$ = of(...)`, `name$ = from(...)`, etc.
        if let Some(obs) = extract_observable_from_line(source, trimmed_abs, trimmed) {
            shape.observables.push(obs);
        }
        line_start += line.len() + 1;
    }
}

/// Try to extract an observable declaration from a single line.
///
/// `line_abs` is the absolute byte offset of `line` within `source`, used
/// to reject matches inside comments/strings (Round-11 audit).
fn extract_observable_from_line(
    source: &str,
    line_abs: usize,
    line: &str,
) -> Option<ObservableDecl> {
    // Check for service calls first (e.g. `name$: Observable<T> = this.http.get(...)`)
    // so the source is captured when the type annotation is also present.
    if let Some(obs) = extract_service_call_observable(source, line_abs, line) {
        return Some(obs);
    }

    // Check for `Observable<T>` type annotation
    // Pattern: `name$: Observable<Type>` or `name: Observable<Type>`
    if let Some(idx) = line.find(": Observable<") {
        let before = &line[..idx];
        // A field declaration has no parameter list before its type
        // annotation. Method return annotations and typed parameters do.
        if !before.contains('(')
            && !before.contains(')')
            && !crate::angular_meta::util::is_inside_comment_or_string(source, line_abs + idx)
        {
            let name = extract_field_name(before)?;
            let rest = &line[idx + ": Observable<".len()..];
            let type_param = rest.split('>').next().map(|s| s.trim().to_string());
            return Some(ObservableDecl {
                name,
                source: None,
                type_param,
            });
        }
    }

    // Check for `Observable<Type>` without field prefix (e.g. return type)
    if line.starts_with("Observable<") {
        // Not a field declaration — skip
        return None;
    }

    // Check for creation functions: `name$ = of(...)`, `name$ = from(...)`, etc.
    let creation_funcs = [
        "of(",
        "from(",
        "interval(",
        "timer(",
        "fromEvent(",
        "ajax(",
        "defer(",
        "empty(",
        "never(",
        "throwError(",
        "iif(",
        "merge(",
        "concat(",
        "race(",
        "zip(",
    ];

    for func in &creation_funcs {
        if let Some(idx) = line.find(&format!(" = {}", func)) {
            if !crate::angular_meta::util::is_inside_comment_or_string(source, line_abs + idx) {
                let before = &line[..idx];
                let name = extract_field_name(before)?;
                return Some(ObservableDecl {
                    name,
                    source: Some(func.trim_end_matches('(').to_string()),
                    type_param: None,
                });
            }
        }
    }

    None
}

/// Extract an observable from a service-call assignment pattern.
/// Handles lines like `name$: Observable<T> = this.http.get(...)` or
/// `name$ = this.http.get(...)`.
///
/// `line_abs` is the absolute byte offset of `line` within `source`, used
/// to reject matches inside comments/strings (Round-11 audit).
fn extract_service_call_observable(
    source: &str,
    line_abs: usize,
    line: &str,
) -> Option<ObservableDecl> {
    // Check for service calls: `name$ = this.http.get(...)` or `name$ = http.get(...)`
    // or `name$: Observable<T> = this.http.get(...)`
    let eq_idx = line.find(" = ")?;
    // Reject when the ` = ` itself is inside a comment/string (e.g. a
    // trailing `// users$ = http.get(...)` comment).
    if crate::angular_meta::util::is_inside_comment_or_string(source, line_abs + eq_idx) {
        return None;
    }
    let before = &line[..eq_idx];
    // Round-9 audit: strip the type annotation BEFORE extracting the field
    // name. For `users$: Observable<User[]> = this.http.get(...)`, the last
    // whitespace token of the full LHS is `Observable<User[]>` (the type),
    // not the field name — the old code emitted `Φobs:Observable<User[]>`.
    // Split on `:` first so only the declarator part (`private users$`) is
    // passed to `extract_field_name`.
    let name_part = before.split(':').next().unwrap_or(before).trim();
    let name = extract_field_name(name_part)?;

    // Check various HTTP/service patterns.
    // Note: the call may include a generic type param, e.g.
    // `http.get<User[]>(...)` — so we match `http.get` followed by
    // either `(` or `<`.
    let after = &line[eq_idx + 3..];
    let service_patterns = [
        "http.get",
        "http.post",
        "http.put",
        "http.delete",
        "http.patch",
        "this.http.get",
        "this.http.post",
        "this.http.put",
        "this.http.delete",
        "this.http.patch",
    ];

    for pat in &service_patterns {
        if let Some(pat_idx) = after.find(pat) {
            // Reject when the service pattern is inside a comment/string.
            if crate::angular_meta::util::is_inside_comment_or_string(
                source,
                line_abs + eq_idx + 3 + pat_idx,
            ) {
                continue;
            }
            // Extract the URL or first argument.
            // The pattern may be followed by `<Type>` (generic) then `(`.
            let rest = after.split(pat).nth(1).unwrap_or("");
            // Skip generic type params: `<...>`
            let rest = if let Some(gt_idx) = rest.find('>') {
                &rest[gt_idx + 1..]
            } else {
                rest
            };
            // Now find the first `(` and extract the URL argument.
            let url = if let Some(open_idx) = rest.find('(') {
                let after_open = &rest[open_idx + 1..];
                after_open
                    .split(')')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string()
            } else {
                String::new()
            };
            return Some(ObservableDecl {
                name,
                source: Some(format!("http.get({})", url)),
                type_param: None,
            });
        }
    }

    // Check for `this.service.method()` patterns
    let method_patterns = [".get(", ".post(", ".put(", ".delete(", ".patch("];
    let mut earliest: Option<usize> = None;
    for mp in &method_patterns {
        if let Some(mp_idx) = after.find(mp) {
            if earliest.is_none_or(|e| mp_idx < e) {
                earliest = Some(mp_idx);
            }
        }
    }
    if let Some(mp_idx) = earliest {
        // Reject when the method pattern is inside a comment/string.
        if !crate::angular_meta::util::is_inside_comment_or_string(
            source,
            line_abs + eq_idx + 3 + mp_idx,
        ) {
            // Extract up to the first '('
            let method_call = after.split('(').next().unwrap_or("").trim();
            return Some(ObservableDecl {
                name,
                source: Some(method_call.to_string()),
                type_param: None,
            });
        }
    }

    None
}

/// Extract field name from before a type annotation or assignment.
/// Handles: `name$`, `private name$`, `readonly name$`, `public name$`,
/// `protected name$`, `static name$`.
fn extract_field_name(before: &str) -> Option<String> {
    let trimmed = before.trim();
    // Split by whitespace and take the last word (the field name)
    let name = trimmed.split_whitespace().last()?;
    if name.is_empty() {
        return None;
    }
    // Filter out keywords and type names
    if name == "private"
        || name == "public"
        || name == "protected"
        || name == "readonly"
        || name == "static"
        || name == "const"
        || name == "let"
        || name == "var"
    {
        return None;
    }
    Some(name.to_string())
}

/// Extract subject instantiations from the source.
///
/// Detects:
/// - `new Subject<T>()`
/// - `new BehaviorSubject<T>(initialValue)`
/// - `new ReplaySubject<T>(n)`
/// - `new AsyncSubject<T>()`
fn extract_subjects(source: &str, shape: &mut RxShape) {
    let subject_patterns = [
        ("new Subject<", SubjectKind::Subject),
        ("new BehaviorSubject<", SubjectKind::BehaviorSubject),
        ("new ReplaySubject<", SubjectKind::ReplaySubject),
        ("new AsyncSubject<", SubjectKind::AsyncSubject),
    ];

    // Track the absolute byte offset of each line so matches inside
    // trailing comments or string literals can be rejected (Round-11
    // audit: a `// selectedUser$ = new Subject()` trailing comment must
    // not produce a phantom subject).
    let mut line_start = 0usize;
    for line in source.split('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with('*') {
            line_start += line.len() + 1;
            continue;
        }

        // Absolute offset of the trimmed line within `source`.
        let leading = line.len() - line.trim_start().len();
        let trimmed_abs = line_start + leading;

        for (pattern, kind) in &subject_patterns {
            if let Some(idx) = trimmed.find(pattern) {
                // Round-11 audit: reject when the pattern match is inside a
                // comment or string literal (e.g. a trailing comment).
                if crate::angular_meta::util::is_inside_comment_or_string(source, trimmed_abs + idx)
                {
                    continue;
                }
                // Extract field name before the `new` keyword.
                // The line looks like: `name = new Subject<T>()` or
                // `private name = new Subject<T>()`.
                let before = &trimmed[..idx].trim();
                // Round-9 audit: strip the type annotation before extracting
                // the field name. For
                // `selectedUser$: BehaviorSubject<User | null> = new ...`,
                // the text before `new` is
                // `selectedUser$: BehaviorSubject<User | null> = ` — the old
                // last-token logic landed on `|` (from the type), not the field
                // name. Split on `=` to keep only the declarator, then on `:`
                // to drop the type annotation → `selectedUser$`.
                let before = before
                    .split('=')
                    .next()
                    .unwrap_or(before)
                    .split(':')
                    .next()
                    .unwrap_or(before)
                    .trim();
                // Take the last non-empty token, stripping trailing `=`.
                let name = before
                    .split_whitespace()
                    .last()
                    .map(|s| s.trim_end_matches('=').trim().to_string())
                    .filter(|s| {
                        !s.is_empty()
                            && *s != "="
                            && *s != ":"
                            && *s != "private"
                            && *s != "public"
                            && *s != "protected"
                            && *s != "readonly"
                            && *s != "static"
                            && *s != "const"
                            && *s != "let"
                            && *s != "var"
                            && *s != "new"
                    });
                // If the last token was `=` (e.g. `name = new Subject`),
                // the actual field name is the second-to-last token.
                let name = name.or_else(|| {
                    let tokens: Vec<&str> = before.split_whitespace().collect();
                    if tokens.len() >= 2 {
                        let candidate = tokens[tokens.len() - 2].trim_end_matches('=').trim();
                        if !candidate.is_empty()
                            && candidate != "="
                            && candidate != "new"
                            && candidate != "private"
                            && candidate != "public"
                            && candidate != "protected"
                            && candidate != "readonly"
                            && candidate != "static"
                            && candidate != "const"
                            && candidate != "let"
                            && candidate != "var"
                        {
                            Some(candidate.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });

                let rest = &trimmed[idx + pattern.len()..];

                // Extract type parameter
                let type_param = rest.split('>').next().map(|s| s.trim().to_string());

                // Extract initial value (for BehaviorSubject) or buffer size (for ReplaySubject)
                let rest_after_type = rest.split('>').nth(1).unwrap_or("");
                let initial_value = if rest_after_type.starts_with('(') {
                    // Extract the constructor argument
                    let args = rest_after_type.trim_start_matches('(');
                    let arg = args.split(')').next().map(|s| s.trim().to_string());
                    arg.filter(|s| !s.is_empty() && *s != ";" && *s != ",")
                } else {
                    None
                };

                shape.subjects.push(SubjectDecl {
                    name: name.unwrap_or_else(|| "?".to_string()),
                    kind: *kind,
                    initial_value,
                    type_param,
                });
                break;
            }
        }
        line_start += line.len() + 1;
    }
}
