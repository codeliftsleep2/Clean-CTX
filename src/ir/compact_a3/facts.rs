//! Research-only A3 sparse fact and exact-body records.

use serde_json::{Value, json};
use std::collections::HashMap;
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

fn handle(value: &Value, family: char) -> Result<&str, String> {
    let value = value.as_str().ok_or("handle must be a string")?;
    if !value.starts_with(family) || value.contains(['|', '\r', '\n']) {
        return Err("invalid typed handle".into());
    }
    let suffix = value.strip_prefix(family).ok_or("invalid typed handle")?;
    if suffix.is_empty() || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("invalid typed handle".into());
    }
    Ok(suffix)
}

fn methods(normalized: &Value) -> Result<Vec<&Value>, String> {
    let mut methods = Vec::new();
    for family in ["classes", "interfaces"] {
        for owner in normalized[family]
            .as_array()
            .ok_or_else(|| format!("{family} must be an array"))?
        {
            methods.extend(
                owner["methods"]
                    .as_array()
                    .ok_or("methods must be an array")?,
            );
        }
    }
    Ok(methods)
}

fn pair_rows(output: &mut String, tag: &str, values: &Value, field: &str) -> Result<(), String> {
    for value in values
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
    {
        let columns = value
            .as_array()
            .ok_or_else(|| format!("{field} row must be an array"))?;
        if columns.len() != 2 {
            return Err(format!("{field} row must contain two values"));
        }
        writeln!(
            output,
            "{tag}|{}|{}",
            quote(&columns[0], field)?,
            quote(&columns[1], field)?
        )
        .unwrap();
    }
    Ok(())
}

fn scalar_rows(output: &mut String, tag: &str, values: &Value, field: &str) -> Result<(), String> {
    for value in values
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
    {
        writeln!(output, "{tag}|{}", quote(value, field)?).unwrap();
    }
    Ok(())
}

/// Encode sparse behavior, local calls, imports, and aliases. Record order is
/// authoritative; empty families emit no rows.
pub fn encode(normalized: &Value) -> Result<String, String> {
    let fidelity = normalized["mode"]["fidelity"]
        .as_str()
        .ok_or("missing fidelity")?;
    let detailed = matches!(fidelity, "high" | "edit");
    let mut output = String::new();
    if detailed {
        for method in methods(normalized)? {
            let mut rows = String::new();
            pair_rows(&mut rows, "fc", &method["control_flow"], "control flow")?;
            pair_rows(&mut rows, "fd", &method["data_flow"], "data flow")?;
            scalar_rows(&mut rows, "se", &method["side_effects"], "side effects")?;
            scalar_rows(
                &mut rows,
                "ec",
                &method["execution_contexts"],
                "execution contexts",
            )?;
            if !rows.is_empty() {
                writeln!(output, "V|{}", handle(&method["id"], 'M')?).unwrap();
                output.push_str(&rows);
            }
        }
    }
    let calls = normalized["calls"]
        .as_array()
        .ok_or("calls must be an array")?;
    let mut start = 0;
    while start < calls.len() {
        let caller_id = calls[start]["caller_method_id"]
            .as_str()
            .ok_or("call caller must be a string")?;
        let caller = handle(&calls[start]["caller_method_id"], 'M')?;
        let mut end = start + 1;
        while end < calls.len() && calls[end]["caller_method_id"] == caller_id {
            end += 1;
        }
        write!(output, "K|{caller}|{}", end - start).unwrap();
        for call in &calls[start..end] {
            if call["callee_resolution"] != "unresolved" {
                return Err("A3 cannot fabricate resolved callees".into());
            }
            write!(
                output,
                "|{}|{}",
                quote(&call["callee_written_name"], "callee")?,
                call["explicit_argument_count"]
                    .as_u64()
                    .ok_or("call arity must be unsigned")?
            )
            .unwrap();
            if call["has_spread"]
                .as_bool()
                .ok_or("spread must be boolean")?
            {
                output.push_str("|*");
            }
        }
        output.push('\n');
        start = end;
    }
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
    let mut behavior = Vec::new();
    let mut calls = Vec::new();
    let mut imports = Vec::new();
    let mut aliases = Vec::new();
    let mut method = None::<String>;
    let mut ordinals = HashMap::<(String, String), u64>::new();
    for line in input.lines() {
        let row = columns(line)?;
        match row.first().copied().unwrap_or_default() {
            "V" => {
                if row.len() != 2 || !row[1].bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("invalid method scope".into());
                }
                method = Some(format!("M{}", row[1]));
            }
            "fc" | "fd" | "se" | "ec" => {
                let owner = method.clone().ok_or("behavior outside method")?;
                let pair = matches!(row[0], "fc" | "fd");
                if row.len() != if pair { 3 } else { 2 } {
                    return Err("invalid behavior row".into());
                }
                let next = ordinals.entry((owner.clone(), row[0].into())).or_default();
                let ordinal = *next;
                *next += 1;
                let mut value = vec![string(row[1])?];
                if pair {
                    value.push(string(row[2])?);
                }
                behavior.push(
                    json!({"method_id":owner,"family":row[0],"ordinal":ordinal,"value":value}),
                );
            }
            "K" => {
                if row.len() < 3 || !row[1].bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err("invalid call run".into());
                }
                let count = row[2].parse::<usize>().map_err(|_| "invalid call count")?;
                let mut column = 3;
                for _ in 0..count {
                    let callee = row.get(column).ok_or("truncated call run")?;
                    let arity = row.get(column + 1).ok_or("truncated call run")?;
                    column += 2;
                    let spread = row.get(column) == Some(&"*");
                    column += usize::from(spread);
                    calls.push(json!({
                        "occurrence":calls.len(),"caller_method_id":format!("M{}", row[1]),
                        "callee_written_name":string(callee)?,
                        "explicit_argument_count":arity.parse::<u64>().map_err(|_|"invalid call arity")?,
                        "has_spread":spread,"callee_resolution":"unresolved"
                    }));
                }
                if column != row.len() {
                    return Err("call count mismatch".into());
                }
            }
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
    Ok(json!({"behavior":behavior,"calls":calls,"imports":imports,"type_aliases":aliases}))
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
