//! Lightweight value and declaration-name extraction utilities.

/// Extract the first quoted string from a text (single or double quotes).
///
/// Escape-aware: an escaped quote (`\'` / `\"`) does not terminate the
/// string. The returned value excludes the surrounding quotes.
pub fn extract_first_quoted(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    let (value, quote) = if let Some(stripped) = trimmed.strip_prefix('\'') {
        (stripped, '\'')
    } else {
        let stripped = trimmed.strip_prefix('"')?;
        (stripped, '"')
    };
    let mut chars = value.char_indices();
    while let Some((i, c)) = chars.next() {
        if c == '\\' {
            chars.next();
            continue;
        }
        if c == quote {
            return Some(value[..i].to_string());
        }
    }
    // Unterminated — return the rest.
    Some(value.to_string())
}

/// Extract the generic type parameter from the start of `text` up to and
/// including the first top-level `>`.
///
/// Accumulates every character from the beginning — this handles the two
/// common call sites:
/// - `EntityState<User>>({ ... })` → `EntityState<User>` (nested generic)
/// - `User>({...})` → `User` (simple generic)
///
/// String-aware: a `>` inside a string literal or template interpolation
/// does NOT terminate the scan (a type object literal like
/// `EntityState<{ tag: 'x>y' }>` is parsed correctly).
pub fn extract_entity_type(text: &str) -> String {
    let mut depth = 0i32;
    let mut result = String::new();
    let mut chars = text.char_indices().peekable();
    let mut in_string: Option<(char, bool)> = None;
    let mut interp_depth: i32 = 0;

    while let Some((_, c)) = chars.next() {
        if let Some((quote, is_template)) = in_string {
            result.push(c);
            if is_template && interp_depth > 0 {
                match c {
                    '{' => interp_depth += 1,
                    '}' => {
                        interp_depth -= 1;
                        if interp_depth == 0 {
                            in_string = Some(('`', true));
                        }
                    }
                    _ => {}
                }
            } else if c == '\\' {
                if let Some(&(_, next)) = chars.peek() {
                    result.push(next);
                    chars.next();
                }
            } else if c == quote {
                in_string = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                in_string = Some((c, c == '`'));
                result.push(c);
            }
            '<' => {
                depth += 1;
                result.push(c);
            }
            '>' => {
                if depth == 0 {
                    // First top-level `>` — done.
                    break;
                }
                depth -= 1;
                result.push(c);
            }
            _ => {
                result.push(c);
            }
        }
    }
    result.trim().to_string()
}

/// Extract a declarator/assignment name from the text BEFORE the value
/// expression. Handles:
/// - simple identifiers: `count`
/// - member expressions: `this.count`, `obj.x.y` → `y`
/// - optional chaining: `user?.name` → `name`
/// - trailing `=` or `:` before the value: `count =`, `count:`
/// - single-line typed declarations: `private count: number =` → `count`
///
/// Returns `None` when no valid identifier is found.
pub fn extract_decl_name(before: &str) -> Option<String> {
    let trimmed = before.trim_end();
    if let Some(declaration) = trimmed.strip_suffix('=') {
        if let Some(name) = extract_typed_decl_name(declaration) {
            return Some(name);
        }
    }

    extract_last_name(before)
}

/// Recognize the TypeScript `<modifiers> <name>: <type> =` shape. The
/// trailing assignment separator has already been removed by the caller.
fn extract_typed_decl_name(declaration: &str) -> Option<String> {
    let statement = declaration
        .rsplit([';', '\n', '\r', '{', '}'])
        .next()?
        .trim();
    let (target, annotation) = statement.split_once(':')?;
    if annotation.trim().is_empty() {
        return None;
    }

    let mut target_parts = target.split_whitespace();
    let target = target_parts.next_back()?;
    if !target_parts.all(is_declaration_modifier) {
        return None;
    }

    extract_name_token(target, true).map(str::to_owned)
}

fn is_declaration_modifier(token: &str) -> bool {
    matches!(
        token,
        "const"
            | "let"
            | "var"
            | "public"
            | "private"
            | "protected"
            | "readonly"
            | "static"
            | "declare"
            | "abstract"
            | "accessor"
            | "override"
            | "export"
            | "default"
    )
}

fn extract_last_name(before: &str) -> Option<String> {
    let mut tokens: Vec<&str> = before.split_whitespace().collect();
    while let Some(last) = tokens.last() {
        let last = last.trim_end_matches('=').trim();
        if last.is_empty() || last == "=" {
            tokens.pop();
            continue;
        }
        let last = last.trim_end_matches(':').trim();
        if last.is_empty() || last == ":" {
            tokens.pop();
            continue;
        }
        let last = extract_name_token(last, false)?;
        if last.is_empty() {
            tokens.pop();
            continue;
        }
        // Must be a valid identifier-ish token. If not, this is a bare
        // statement (e.g. `{` before a constructor `effect()` call) — NOT
        // an assignment. Do NOT keep walking backward past the invalid
        // punctuation to pick up an enclosing class name; return `None`
        // so the caller renders `?` (matches the original signals layer
        // semantics — Round-8 audit regression guard).
        return Some(last.to_string());
    }
    None
}

fn extract_name_token(token: &str, allow_definite_assignment: bool) -> Option<&str> {
    let token = token.rsplit('.').next().unwrap_or(token);
    let token = if allow_definite_assignment {
        token.trim_end_matches(['?', '!'])
    } else {
        token.trim_end_matches('?')
    };
    token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        .then_some(token)
}
