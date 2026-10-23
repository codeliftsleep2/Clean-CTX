//! Exact demanded JavaScript string values, including unpaired UTF-16 units.
//! This is literal acquisition, not constant evaluation or Angular admission.

pub(super) fn decode(raw: &str) -> Option<Vec<u16>> {
    decode_units(raw, true)
}

pub(super) fn decode_module(raw: &str) -> Option<String> {
    String::from_utf16(&decode_units(raw, false)?).ok()
}

fn decode_units(raw: &str, literal: bool) -> Option<Vec<u16>> {
    let quote = raw.chars().next()?;
    if !matches!(quote, '\'' | '"') || raw.len() < 2 || !raw.ends_with(quote) {
        return None;
    }
    let mut chars = raw[1..raw.len() - 1].chars().peekable();
    let mut units = Vec::new();
    while let Some(ch) = chars.next() {
        if literal && (ch == quote || matches!(ch, '\r' | '\n')) {
            return None;
        }
        if ch != '\\' {
            units.extend(ch.encode_utf16(&mut [0; 2]).iter().copied());
            continue;
        }
        match chars.next()? {
            '\n' | '\u{2028}' | '\u{2029}' => {}
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
            'x' => units.push(hex(&mut chars, 2)? as u16),
            'u' => {
                if chars.peek() != Some(&'{') {
                    units.push(hex(&mut chars, 4)? as u16);
                    continue;
                }
                chars.next();
                let mut value = 0u32;
                let mut nonempty = false;
                loop {
                    let ch = chars.next()?;
                    if ch == '}' {
                        break;
                    }
                    nonempty = true;
                    value = value.checked_mul(16)?.checked_add(ch.to_digit(16)?)?;
                    if value > 0x10ffff {
                        return None;
                    }
                }
                if !nonempty {
                    return None;
                }
                if !literal && (0xd800..=0xdfff).contains(&value) {
                    return None;
                }
                if value <= 0xffff {
                    units.push(value as u16);
                } else {
                    let scalar = char::from_u32(value)?;
                    units.extend(scalar.encode_utf16(&mut [0; 2]).iter().copied());
                }
            }
            '0' if !chars.peek().is_some_and(char::is_ascii_digit) => units.push(0),
            // Legacy decimal/octal spellings are unsupported, never guessed.
            '0'..='9' => return None,
            'b' => units.push(8),
            'f' => units.push(12),
            'n' => units.push(10),
            'r' => units.push(13),
            't' => units.push(9),
            'v' => units.push(11),
            ch => units.extend(ch.encode_utf16(&mut [0; 2]).iter().copied()),
        }
    }
    Some(units)
}

fn hex(chars: &mut impl Iterator<Item = char>, count: usize) -> Option<u32> {
    let mut value = 0;
    for _ in 0..count {
        value = value * 16 + chars.next()?.to_digit(16)?;
    }
    Some(value)
}
