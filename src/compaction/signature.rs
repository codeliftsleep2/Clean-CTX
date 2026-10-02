// src/compaction/signature.rs
//
// STRUCTURAL declaration-head extraction, shared by the three consumers that
// must agree on a declaration's identity:
//
//   * `compaction::method` — Low/Medium compacted method labels
//   * `ir::pipeline`       — `PassContext::parse_method_sig` (the IR's method
//                            identity, its parameters, and its return type)
//   * `diff::keys`         — `method_key` (diff grouping and change labels)
//
// The rule enforced here: the declared NAME is the identifier that OWNS the
// declaration's parameter list, read from STRUCTURE — the balanced
// angle-bracket groups the grammar permits and the parameter group's own
// position — never from whitespace-token position. `Pair<TFirst, TSecond>`
// contains a `, ` inside its type-parameter list, so "the last whitespace
// token before the `(`" is `TSecond>`; and a tuple return type
// (`public static (int alpha, int beta) GetPair(...)`) puts a modifier in that
// position, so the name became `static`. Token-position inference destroys the
// identity of both shapes. No consumer may re-derive a name by string
// splitting: they use the helpers below.

/// The structural head of a method/function declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HeadParts<'a> {
    /// Identifier as written, including an attached type-parameter list
    /// (`Pair<TFirst, TSecond>`, `ToSet<T>`, `pair<A, B>`).
    ///
    /// When a type-parameter list PRECEDES the return type
    /// (`<A, B> Result<A> pair(...)`) it is not attached to the identifier and
    /// remains in `prefix`.
    pub name: &'a str,
    /// The identifier alone — `Pair`, `ToSet`, `pair`, `GetPair`.
    pub bare_name: &'a str,
    /// Everything before `name`: declaration modifiers, and the return type
    /// when the declaration is return-type-first
    /// (`public static IOrderedQueryable<T> `, `public static (int a, int b) `).
    pub prefix: &'a str,
}

/// Identifier bytes (`@`-prefixed identifiers are legal in C#).
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'@'
}

/// Start of the identifier token ending at `end` (exclusive), or `None` when
/// `end` is not preceded by identifier characters.
///
/// A `.`-qualified chain is ONE declared name — an explicit interface
/// implementation (`IFoo.Bar`) is a single declaration — so the whole chain is
/// returned rather than its last segment.
fn ident_start(sig: &str, end: usize) -> Option<usize> {
    let bytes = sig.as_bytes();
    if end == 0 || !is_ident_byte(bytes[end - 1]) {
        return None;
    }
    let mut start = end;
    loop {
        while start > 0 && is_ident_byte(bytes[start - 1]) {
            start -= 1;
        }
        if start >= 2 && bytes[start - 1] == b'.' && is_ident_byte(bytes[start - 2]) {
            start -= 1;
            continue;
        }
        return Some(start);
    }
}

/// Index of the `<` whose `>` group ends at `end` (exclusive).
///
/// Only ever called on a declaration head (the text before the parameter
/// list), where `<` can only open a type-parameter or type-argument list.
fn angle_open(sig: &str, end: usize) -> Option<usize> {
    let bytes = sig.as_bytes();
    if end == 0 || bytes[end - 1] != b'>' {
        return None;
    }
    let mut depth: i32 = 0;
    let mut i = end;
    while i > 0 {
        i -= 1;
        match bytes[i] {
            b'>' => depth += 1,
            b'<' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Structural name/prefix of the declaration whose parameter list opens at
/// `params_open`.
///
/// `None` when the head does not end in an identifier (a malformed or
/// unsupported shape); every caller then keeps its legacy behavior instead of
/// guessing.
pub(crate) fn split_head_parts(sig: &str, params_open: usize) -> Option<HeadParts<'_>> {
    if params_open > sig.len() {
        return None;
    }
    let head = &sig[..params_open];
    let bytes = head.as_bytes();
    let mut end = head.len();
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    if end == 0 {
        return None;
    }
    let (name_start, bare_end) = if bytes[end - 1] == b'>' {
        // Generic declaration: `Pair<TFirst, TSecond>(`, `ToSet<T>(`.
        let open = angle_open(head, end)?;
        (ident_start(head, open)?, open)
    } else {
        (ident_start(head, end)?, end)
    };
    Some(HeadParts {
        name: &head[name_start..end],
        bare_name: &head[name_start..bare_end],
        prefix: &head[..name_start],
    })
}

/// Whether the depth-0 parenthesized group closing at `close` is a
/// PARENTHESIZED RETURN TYPE rather than the declaration's parameter list.
///
/// A tuple return type is a depth-0 `(...)` group that PRECEDES the declared
/// name (`public static (int alpha, int beta) GetPair(...)`), whereas a
/// parameter list is followed by the declaration's body/arrow/`;` boundary, a
/// constructor initializer (`: base(...)`), or a `where` clause. The test is
/// structural: the group is a return type iff the declared name follows it and
/// the declaration's own parameter list follows that name.
pub(crate) fn is_return_type_group(sig: &str, close: usize) -> bool {
    let bytes = sig.as_bytes();
    let mut i = close + 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    // The declared name must follow the group ...
    let name_start = i;
    while i < bytes.len() && is_ident_byte(bytes[i]) {
        i += 1;
    }
    if i == name_start {
        return false;
    }
    // ... optionally with its own type-parameter list ...
    if i < bytes.len() && bytes[i] == b'<' {
        let mut depth: i32 = 0;
        let mut closed = false;
        while i < bytes.len() {
            match bytes[i] {
                b'<' => depth += 1,
                b'>' => {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        closed = true;
                        break;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        if !closed {
            return false;
        }
    }
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    // ... and the declaration's parameter list immediately after the name.
    i < bytes.len() && bytes[i] == b'('
}

/// Index of the opener matching the closing delimiter at `end - 1`.
fn matching_opener(text: &str, end: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if end == 0 {
        return None;
    }
    let close = bytes[end - 1];
    let open = match close {
        b'>' => b'<',
        b')' => b'(',
        b']' => b'[',
        _ => return None,
    };
    let mut depth: i32 = 0;
    let mut i = end;
    while i > 0 {
        i -= 1;
        if bytes[i] == close {
            depth += 1;
        } else if bytes[i] == open {
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

/// Start of the TRAILING TYPE EXPRESSION of a return-type-first prefix.
///
/// The type is identified by its own closing delimiter when it has one
/// (`Task<(A, B)>`, `(int alpha, int beta)`, `ILookup<K, V>[]`, `int[]` —
/// the matching opener, extended left over the type name that owns it) and by
/// its final token when it is a simple type (`IActionResult`, `void`,
/// `string?`). Declaration modifiers therefore never reach the type, because
/// nothing here is derived from a token COUNT or a token POSITION.
fn trailing_type_start(prefix: &str) -> Option<usize> {
    let text = prefix.trim_end();
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut cursor = text.len();
    loop {
        let mut component = if matches!(bytes[cursor - 1], b'>' | b')' | b']') {
            matching_opener(text, cursor)?
        } else {
            let mut token = cursor;
            while token > 0 && !bytes[token - 1].is_ascii_whitespace() {
                token -= 1;
            }
            token
        };
        // A type NAME owns the argument/array group that follows it
        // (`Task<...>`, `ILookup<K, V>[]`), so the expression starts there.
        while component > 0 && is_ident_byte(bytes[component - 1]) {
            component -= 1;
        }
        if component > 0 && matches!(bytes[component - 1], b'>' | b')' | b']') {
            cursor = component;
            continue;
        }
        return Some(component);
    }
}

/// Whether the head text before the declared name declares a RETURN TYPE — the
/// declaration is return-type-first (C#) rather than name-first (TypeScript,
/// Rust).
///
/// This is the Low/Medium compaction DECISION. It deliberately keeps the
/// established classification ("a type-looking final token", shared with
/// `is_csharp_return_type`) so every shape that compacted correctly still
/// compacts identically, and adds the structural tuple case, where the final
/// token is `beta)` and the declaration is nonetheless return-type-first.
///
/// It is NOT the return-type extractor: see [`return_type_from_prefix`].
pub(crate) fn is_return_type_first(prefix: &str) -> bool {
    let trimmed = prefix.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.ends_with(')') {
        return true;
    }
    trimmed
        .split_whitespace()
        .last()
        .is_some_and(super::method::is_csharp_return_type)
}

/// The return type declared by a return-type-first prefix, or `None` when the
/// prefix declares none.
///
/// The type is extracted structurally — the trailing TYPE expression only — so
/// declaration modifiers (`public static async`) and the declaration's own
/// type-parameter list can never be projected into the return-type field.
pub(crate) fn return_type_from_prefix(prefix: &str) -> Option<&str> {
    let text = prefix.trim_end();
    if text.trim_start().starts_with('<') {
        // The declaration's OWN type-parameter list (`<A, B> Result<A> pair`):
        // it belongs to the method, the IR has no field for it, and the
        // established value for such a declaration stays unchanged rather than
        // fabricating `<A, B> Result<A>` as a type.
        return None;
    }
    let start = trailing_type_start(text)?;
    let ty = text[start..].trim();
    if ty.is_empty() {
        return None;
    }
    // A nullable annotation is a type modifier, not part of the type name.
    let bare = ty.strip_suffix('?').unwrap_or(ty);
    if super::method::is_csharp_return_type(bare) || ty.ends_with(')') || ty.ends_with(']') {
        Some(ty)
    } else {
        None
    }
}

/// Split a parameter list into the parameters the declaration actually WROTE.
///
/// A comma nested inside a generic argument list
/// (`Expression<Func<TFirst, TSecond>> keySelector`), a parameter list
/// (`Func<int, int> f`), an array type (`Dictionary<string, int>[] x`), a
/// string/char literal default, or a lambda default
/// (`Action cb = (x) => { }`) does not separate parameters. Splitting on every
/// comma inflates the formal parameter list — the canonical representation
/// must know the parameters, not the punctuation.
pub(crate) fn split_parameters(params: &str) -> Vec<&str> {
    let bytes = params.as_bytes();
    let mut out = Vec::new();
    let mut depth: i32 = 0;
    let mut start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' => {
                i = super::method::skip_quoted_literal(bytes, i);
                continue;
            }
            b'<' | b'(' | b'[' => depth += 1,
            // `=>` must not drive the depth negative; the clamp keeps a
            // lambda default from suppressing every later separator.
            b'>' | b')' | b']' => depth = (depth - 1).max(0),
            b',' if depth == 0 => {
                let piece = params[start..i].trim();
                if !piece.is_empty() {
                    out.push(piece);
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let piece = params[start..].trim();
    if !piece.is_empty() {
        out.push(piece);
    }
    out
}

/// Resolve the declared return type around a method's parameter list.
///
/// Name-first languages put the return annotation after the list (`: T` or
/// `-> T`). Return-type-first languages keep it in `prefix`. Post-parameter
/// constraint/exception clauses are neither form: C# and Rust use `where`,
/// while Java uses `throws`. Those clauses are removed only at top level so a
/// nested generic/function type cannot be split accidentally.
pub(crate) fn declared_return_type<'a>(prefix: &'a str, tail: &'a str) -> Option<&'a str> {
    let tail = tail.trim();
    if let Some(annotation) = tail.strip_prefix(':') {
        let annotation = annotation.trim();
        return (!annotation.is_empty()).then_some(annotation);
    }

    let clause = top_level_keyword(tail, "where")
        .into_iter()
        .chain(top_level_keyword(tail, "throws"))
        .min();
    let annotation = clause.map_or(tail, |index| tail[..index].trim_end());
    if !annotation.is_empty() {
        Some(annotation)
    } else {
        return_type_from_prefix(prefix)
    }
}

fn top_level_keyword(text: &str, keyword: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                index = super::method::skip_quoted_literal(bytes, index);
                continue;
            }
            b'<' | b'(' | b'[' | b'{' => depth += 1,
            b'>' | b')' | b']' | b'}' => depth = (depth - 1).max(0),
            _ => {}
        }
        if depth == 0
            && bytes[index..].starts_with(keyword.as_bytes())
            && (index == 0 || bytes[index - 1].is_ascii_whitespace())
            && bytes
                .get(index + keyword.len())
                .is_none_or(u8::is_ascii_whitespace)
        {
            return Some(index);
        }
        index += 1;
    }
    None
}

/// Split one written parameter into its canonical `(name, type)` fields.
///
/// Name-first declarations (`name: Type`) retain their established handling.
/// Type-first declarations (C#/Java) are split at the last top-level
/// whitespace boundary, so whitespace and commas inside generic/array syntax
/// cannot turn pieces of the type into a parameter name. C# passing/collection
/// modifiers describe the parameter slot rather than the written type identity
/// and are therefore removed from the canonical type.
pub(crate) fn parse_parameter(parameter: &str) -> (String, String) {
    let parameter = strip_leading_parameter_metadata(parameter.trim());
    if let Some(colon_pos) = top_level_separator(parameter, ':') {
        let name = parameter[..colon_pos]
            .trim()
            .trim_start_matches("...")
            .trim_end_matches('?')
            .trim();
        let ty = strip_top_level_default(parameter[colon_pos + 1..].trim());
        return (name.to_string(), ty.to_string());
    }

    let declaration = strip_top_level_default(parameter);
    let declaration = strip_parameter_modifiers(declaration);
    let Some(boundary) = last_top_level_whitespace(declaration) else {
        return (declaration.to_string(), "$v".to_string());
    };

    let name = declaration[boundary..].trim();
    let ty = declaration[..boundary].trim();
    if name.is_empty() || ty.is_empty() {
        (declaration.to_string(), "$v".to_string())
    } else {
        (name.to_string(), ty.to_string())
    }
}

/// Remove balanced declaration metadata without knowing framework names.
/// C# attributes use `[...]`; Java and TypeScript decorators use `@Name` with
/// an optional balanced argument list. Only leading groups are peeled.
fn strip_leading_parameter_metadata(mut parameter: &str) -> &str {
    loop {
        parameter = parameter.trim_start();
        if parameter.starts_with('[') {
            let Some(end) = matching_delimiter_end(parameter, b'[', b']', 0) else {
                return parameter;
            };
            parameter = &parameter[end..];
            continue;
        }
        if parameter.starts_with('@') {
            let bytes = parameter.as_bytes();
            let mut end = 1usize;
            while end < bytes.len()
                && (is_ident_byte(bytes[end]) || bytes[end] == b'.' || bytes[end] == b'$')
            {
                end += 1;
            }
            if end == 1 {
                return parameter;
            }
            if bytes.get(end) == Some(&b'(') {
                let Some(group_end) = matching_delimiter_end(parameter, b'(', b')', end) else {
                    return parameter;
                };
                end = group_end;
            }
            parameter = &parameter[end..];
            continue;
        }
        return parameter;
    }
}

fn matching_delimiter_end(text: &str, open: u8, close: u8, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(start) != Some(&open) {
        return None;
    }
    let mut depth = 0i32;
    let mut index = start;
    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                index = super::method::skip_quoted_literal(bytes, index);
                continue;
            }
            byte if byte == open => depth += 1,
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index + 1);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn strip_parameter_modifiers(mut parameter: &str) -> &str {
    const MODIFIERS: &[&str] = &["ref", "out", "in", "params", "this", "scoped", "readonly"];
    loop {
        let mut stripped = None;
        for modifier in MODIFIERS {
            if let Some(rest) = parameter.strip_prefix(modifier)
                && rest.chars().next().is_some_and(char::is_whitespace)
            {
                stripped = Some(rest.trim_start());
                break;
            }
        }
        match stripped {
            Some(rest) => parameter = rest,
            None => return parameter,
        }
    }
}

fn top_level_separator(parameter: &str, separator: char) -> Option<usize> {
    let mut depth = 0i32;
    for (index, ch) in parameter.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = (depth - 1).max(0),
            '=' if depth == 0 => return None,
            _ if depth == 0 && ch == separator => return Some(index),
            _ => {}
        }
    }
    None
}

fn strip_top_level_default(parameter: &str) -> &str {
    let mut depth = 0i32;
    for (index, ch) in parameter.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = (depth - 1).max(0),
            '=' if depth == 0 => return parameter[..index].trim_end(),
            _ => {}
        }
    }
    parameter
}

fn last_top_level_whitespace(parameter: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut boundary = None;
    for (index, ch) in parameter.char_indices() {
        match ch {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' | ')' | ']' | '}' => depth = (depth - 1).max(0),
            _ if depth == 0 && ch.is_whitespace() => boundary = Some(index),
            _ => {}
        }
    }
    boundary
}

#[cfg(test)]
#[path = "../tests/compaction/signature.rs"]
mod tests;
