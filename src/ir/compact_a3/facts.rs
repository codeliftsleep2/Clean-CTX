//! Research-only A3 sparse fact and exact-body records.

use serde_json::{Value, json};
use std::fmt::Write;

fn quote(value: &Value, field: &str) -> Result<String, String> {
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    if !value.is_empty()
        && value != "-"
        && !value.starts_with('"')
        && !value.contains(['|', '\r', '\n'])
    {
        Ok(value.into())
    } else {
        serde_json::to_string(value).map_err(|error| error.to_string())
    }
}

fn optional(value: &Value, field: &str) -> Result<String, String> {
    if value.is_null() {
        Ok("-".into())
    } else {
        quote(value, field)
    }
}

/// Encode imports and aliases. Record order is authoritative; empty families
/// emit no rows.
pub fn encode(normalized: &Value) -> Result<String, String> {
    let mut output = String::new();
    for import in normalized["imports"]
        .as_array()
        .ok_or("imports must be an array")?
    {
        writeln!(
            output,
            "$|{}|{}|{}",
            optional(&import["alias"], "import alias")?,
            optional(&import["module"], "import module")?,
            optional(&import["named_export"], "named export")?
        )
        .unwrap();
    }
    for alias in normalized["type_aliases"]
        .as_array()
        .ok_or("type aliases must be an array")?
    {
        writeln!(
            output,
            "T|{}|{}",
            optional(&alias["alias"], "type alias")?,
            optional(&alias["original_type"], "original type")?
        )
        .unwrap();
    }
    Ok(output)
}

fn columns(line: &str) -> Result<Vec<&str>, String> {
    let mut output = Vec::new();
    let (mut start, mut quoted, mut escaped) = (0, false, false);
    for (index, character) in line.char_indices() {
        if quoted {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                quoted = false;
            }
        } else if character == '"' {
            quoted = true;
        } else if character == '|' {
            output.push(&line[start..index]);
            start = index + 1;
        }
    }
    if quoted || escaped {
        return Err("unterminated quoted column".into());
    }
    output.push(&line[start..]);
    Ok(output)
}

fn string(column: &str) -> Result<Value, String> {
    if column == "-" {
        return Ok(Value::Null);
    }
    if column.starts_with('"') {
        serde_json::from_str::<String>(column)
            .map(Value::String)
            .map_err(|_| "invalid string column".into())
    } else {
        if column.is_empty() || column.contains(['|', '\r', '\n']) {
            return Err("invalid string column".into());
        }
        Ok(Value::String(column.into()))
    }
}

/// Decode sparse fact records into merge-ready normalized families.
pub fn decode(input: &str) -> Result<Value, String> {
    let mut imports = Vec::new();
    let mut aliases = Vec::new();
    for line in input.lines() {
        let row = columns(line)?;
        match row.first().copied().unwrap_or_default() {
            "$" => {
                if row.len() != 4 {
                    return Err("invalid import row".into());
                }
                imports.push(json!({"occurrence":imports.len(),"alias":string(row[1])?,"module":string(row[2])?,"named_export":string(row[3])?}));
            }
            "T" => {
                if row.len() != 3 {
                    return Err("invalid alias row".into());
                }
                aliases.push(json!({"occurrence":aliases.len(),"alias":string(row[1])?,"original_type":string(row[2])?}));
            }
            _ => return Err("unknown A3 fact record".into()),
        }
    }
    Ok(json!({"imports":imports,"type_aliases":aliases}))
}

#[derive(Debug, PartialEq)]
pub struct BodyFrame {
    pub method_id: String,
    pub start: u64,
    pub end: u64,
    pub body: String,
}

pub fn encode_body(frame: &BodyFrame) -> Vec<u8> {
    let mut output = format!(
        "B|{}|{}|{}|{}\n",
        frame.method_id,
        frame.start,
        frame.end,
        frame.body.len()
    )
    .into_bytes();
    output.extend_from_slice(frame.body.as_bytes());
    output.push(b'\n');
    output
}

pub fn decode_bodies(mut input: &[u8]) -> Result<Vec<BodyFrame>, String> {
    let mut frames = Vec::new();
    while !input.is_empty() {
        let (frame, remaining) = decode_body(input)?;
        frames.push(frame);
        input = remaining;
    }
    Ok(frames)
}

pub fn decode_body(input: &[u8]) -> Result<(BodyFrame, &[u8]), String> {
    let end = input
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or("truncated body header")?;
    let header = std::str::from_utf8(&input[..end]).map_err(|_| "invalid body header")?;
    let row = header.split('|').collect::<Vec<_>>();
    if row.len() != 5 || row[0] != "B" || !row[1].starts_with('M') {
        return Err("invalid body header".into());
    }
    let length = row[4].parse::<usize>().map_err(|_| "invalid body length")?;
    let body_input = &input[end + 1..];
    if body_input.len() <= length || body_input[length] != b'\n' {
        return Err("truncated body frame".into());
    }
    let frame = BodyFrame {
        method_id: row[1].into(),
        start: row[2].parse().map_err(|_| "invalid body start")?,
        end: row[3].parse().map_err(|_| "invalid body end")?,
        body: std::str::from_utf8(&body_input[..length])
            .map_err(|_| "invalid body UTF-8")?
            .into(),
    };
    Ok((frame, &body_input[length + 1..]))
}

#[cfg(test)]
#[path = "../../tests/ir/compact_a3_facts.rs"]
mod tests;
