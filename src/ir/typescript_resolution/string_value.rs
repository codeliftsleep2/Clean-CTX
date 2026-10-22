//! Decode syntax-owned string values before module or export-symbol lookup.
pub(super) fn decode(raw: &str) -> Option<String> {
    let quote = raw.chars().next()?;
    if !matches!(quote, '\'' | '"') || !raw.ends_with(quote) || raw.len() < 2 {
        return None;
    }
    let mut chars = raw[1..raw.len() - 1].chars().peekable();
    let mut units = Vec::new();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            units.extend(ch.encode_utf16(&mut [0; 2]).iter().copied());
            continue;
        }
        let escaped = chars.next()?;
        match escaped {
            '\n' | '\u{2028}' | '\u{2029}' => continue,
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                continue;
            }
            'x' => {
                units.push(hex(&mut chars, 2)? as u16);
                continue;
            }
            'u' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let mut digits = String::new();
                    loop {
                        let digit = chars.next()?;
                        if digit == '}' {
                            break;
                        }
                        if !digit.is_ascii_hexdigit() {
                            return None;
                        }
                        digits.push(digit);
                    }
                    let value = u32::from_str_radix(&digits, 16).ok()?;
                    units.extend(
                        char::from_u32(value)?
                            .encode_utf16(&mut [0; 2])
                            .iter()
                            .copied(),
                    );
                } else {
                    units.push(hex(&mut chars, 4)? as u16);
                }
                continue;
            }
            '0' if !chars.peek().is_some_and(char::is_ascii_digit) => units.push(0),
            '0'..='9' => return None, // Legacy octal is not module authority.
            'b' => units.push(8),
            'f' => units.push(12),
            'n' => units.push(10),
            'r' => units.push(13),
            't' => units.push(9),
            'v' => units.push(11),
            other => units.extend(other.encode_utf16(&mut [0; 2]).iter().copied()),
        }
    }
    String::from_utf16(&units).ok()
}

fn hex(chars: &mut impl Iterator<Item = char>, count: usize) -> Option<u32> {
    let mut value = 0;
    for _ in 0..count {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}
