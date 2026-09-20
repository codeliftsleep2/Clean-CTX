/// Return the declaration/signature head, excluding a block body,
/// expression body, or trailing declaration terminator.
///
/// Delimiters inside strings, comments, parameters, attributes, annotations,
/// or generic arguments do not end the head.
pub(super) fn declaration_head(source: &str) -> &str {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut angle_depth = 0usize;
    let mut quote = None;
    let mut line_comment = false;
    let mut block_comment = false;

    while i < bytes.len() {
        let byte = bytes[i];
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            i += 1;
            continue;
        }
        if block_comment {
            if byte == b'*' && bytes.get(i + 1) == Some(&b'/') {
                block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                i += 2;
            } else {
                if byte == delimiter {
                    quote = None;
                }
                i += 1;
            }
            continue;
        }

        if byte == b'/' && bytes.get(i + 1) == Some(&b'/') {
            line_comment = true;
            i += 2;
            continue;
        }
        if byte == b'/' && bytes.get(i + 1) == Some(&b'*') {
            block_comment = true;
            i += 2;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            i += 1;
            continue;
        }

        match byte {
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'<' => angle_depth += 1,
            b'>' if angle_depth > 0 => angle_depth -= 1,
            b'{' if paren_depth == 0 && bracket_depth == 0 && angle_depth == 0 => {
                return &source[..i];
            }
            b';' if paren_depth == 0 && bracket_depth == 0 && angle_depth == 0 => {
                return &source[..i];
            }
            b'=' if paren_depth == 0
                && bracket_depth == 0
                && angle_depth == 0
                && bytes.get(i + 1) == Some(&b'>') =>
            {
                return &source[..i];
            }
            _ => {}
        }
        i += 1;
    }
    source
}

/// Parse the ordered parent list from an interface declaration head.
/// Commas inside generic arguments do not split parent occurrences.
pub(super) fn interface_parents(source: &str, separator: &str) -> Vec<String> {
    let head = declaration_head(source);
    let Some((_, declaration)) = head.split_once("interface") else {
        return Vec::new();
    };
    let Some((_, parents)) = declaration.split_once(separator) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let mut start = 0;
    let mut angle_depth = 0usize;
    for (index, ch) in parents.char_indices() {
        match ch {
            '<' => angle_depth += 1,
            '>' => angle_depth = angle_depth.saturating_sub(1),
            ',' if angle_depth == 0 => {
                let parent = parents[start..index].trim();
                if !parent.is_empty() {
                    result.push(parent.to_string());
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    let parent = parents[start..].trim();
    if !parent.is_empty() {
        result.push(parent.to_string());
    }
    result
}

/// Match a standalone modifier token outside strings and comments.
pub(super) fn has_modifier(head: &str, modifier: &str) -> bool {
    let bytes = head.as_bytes();
    let mut i = 0;
    let mut quote = None;
    let mut line_comment = false;
    let mut block_comment = false;

    while i < bytes.len() {
        let byte = bytes[i];
        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            i += 1;
            continue;
        }
        if block_comment {
            if byte == b'*' && bytes.get(i + 1) == Some(&b'/') {
                block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                i += 2;
            } else {
                if byte == delimiter {
                    quote = None;
                }
                i += 1;
            }
            continue;
        }
        if byte == b'/' && bytes.get(i + 1) == Some(&b'/') {
            line_comment = true;
            i += 2;
            continue;
        }
        if byte == b'/' && bytes.get(i + 1) == Some(&b'*') {
            block_comment = true;
            i += 2;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            i += 1;
            continue;
        }
        if byte.is_ascii_alphabetic() || byte == b'_' || byte == b'@' {
            let start = i;
            i += 1;
            while bytes.get(i).is_some_and(|candidate| {
                candidate.is_ascii_alphanumeric() || matches!(*candidate, b'_' | b'@')
            }) {
                i += 1;
            }
            if &head[start..i] == modifier {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
}
