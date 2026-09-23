//! Integrated research-only A3 High/Edit document.

use super::facts::{
    decode as decode_facts, decode_body, encode as encode_facts, encode_body, BodyFrame,
};
use super::{decode_declarations, encode_declarations};
use serde_json::{json, Value};
use std::collections::HashSet;

const SOURCE_REQUIREMENT: &str = "request edit or verbatim when exact source is required";

fn all_methods(value: &Value) -> impl Iterator<Item = &Value> {
    ["classes", "interfaces"]
        .into_iter()
        .flat_map(|family| value[family].as_array().into_iter().flatten())
        .flat_map(|owner| owner["methods"].as_array().into_iter().flatten())
}

fn body_frames(value: &Value) -> Result<Vec<BodyFrame>, String> {
    if value["mode"]["fidelity"] != "edit" {
        return Ok(Vec::new());
    }
    all_methods(value)
        .filter(|method| !method["body"].is_null())
        .map(|method| {
            Ok(BodyFrame {
                method_id: method["id"].as_str().ok_or("missing method ID")?.into(),
                start: method["body_start"].as_u64().ok_or("missing body start")?,
                end: method["body_end"].as_u64().ok_or("missing body end")?,
                body: method["body"]
                    .as_str()
                    .ok_or("body must be a string")?
                    .into(),
            })
        })
        .collect()
}

fn counts(value: &Value) -> Result<(usize, usize, usize), String> {
    let owners = value["classes"]
        .as_array()
        .ok_or("classes must be an array")?
        .len()
        + value["interfaces"]
            .as_array()
            .ok_or("interfaces must be an array")?
            .len();
    let methods = all_methods(value).count();
    let calls = value["calls"]
        .as_array()
        .ok_or("calls must be an array")?
        .len();
    Ok((owners, methods, calls))
}

/// Encode one complete cold A3 document. No production caller uses it.
pub fn encode(normalized: &Value) -> Result<Vec<u8>, String> {
    if !matches!(
        normalized["mode"]["fidelity"].as_str(),
        Some("low" | "medium" | "high" | "edit")
    ) {
        return Err("unsupported A3 fidelity".into());
    }
    let declarations = encode_declarations(normalized)?;
    let (declarations, _) = declarations
        .rsplit_once("\nZ|")
        .ok_or("declaration fragment lacks terminal")?;
    let facts = encode_facts(normalized)?;
    let bodies = body_frames(normalized)?;
    let (owners, methods, calls) = counts(normalized)?;
    let mut output = format!("{declarations}\n{facts}Y|{}\n", bodies.len()).into_bytes();
    for body in &bodies {
        output.extend(encode_body(body));
    }
    output.extend(format!("Z|{owners}|{methods}|{calls}|{}\n", bodies.len()).into_bytes());
    Ok(output)
}

pub fn encode_cold(normalized: &Value) -> Result<Vec<u8>, String> {
    let mut output = format!("{}\n{}\n", super::COLD_PREAMBLE, super::COLD_LEGEND).into_bytes();
    output.extend(encode(normalized)?);
    Ok(output)
}

/// Token-anatomy fragments for the zero-model economics harness. These are not
/// independently decodable A3 documents; they exist so the harness can count the
/// marginal token cost of each family without a model call.
pub struct ColdAnatomy {
    pub legend: Vec<u8>,
    pub declarations: Vec<u8>,
    pub facts: Vec<u8>,
    pub bodies: Vec<u8>,
}

pub fn anatomy(normalized: &Value) -> Result<ColdAnatomy, String> {
    let legend = format!("{}\n{}\n", super::COLD_PREAMBLE, super::COLD_LEGEND).into_bytes();
    let declarations = encode_declarations(normalized)?;
    let (declarations, _) = declarations
        .rsplit_once("\nZ|")
        .ok_or("declaration fragment lacks terminal")?;
    let facts = encode_facts(normalized)?;
    let bodies = body_frames(normalized)?;
    let mut body_bytes = format!("Y|{}\n", bodies.len()).into_bytes();
    for body in &bodies {
        body_bytes.extend(encode_body(body));
    }
    Ok(ColdAnatomy {
        legend,
        declarations: declarations.as_bytes().to_vec(),
        facts: facts.as_bytes().to_vec(),
        bodies: body_bytes,
    })
}

pub fn decode_cold(input: &[u8]) -> Result<Value, String> {
    let prefix = format!("{}\n{}\n", super::COLD_PREAMBLE, super::COLD_LEGEND);
    let document = input
        .strip_prefix(prefix.as_bytes())
        .ok_or("invalid A3 cold legend")?;
    decode(document)
}

fn marker(input: &[u8], needle: &[u8]) -> Option<usize> {
    input
        .windows(needle.len())
        .position(|window| window == needle)
}

fn method_mut<'a>(value: &'a mut Value, id: &str) -> Option<&'a mut Value> {
    for family in ["classes", "interfaces"] {
        let location =
            value[family]
                .as_array()?
                .iter()
                .enumerate()
                .find_map(|(owner_index, owner)| {
                    owner["methods"]
                        .as_array()?
                        .iter()
                        .position(|method| method["id"] == id)
                        .map(|method_index| (owner_index, method_index))
                });
        if let Some((owner, method)) = location {
            return Some(&mut value[family][owner]["methods"][method]);
        }
    }
    None
}

fn declaration_tag(tag: &str) -> bool {
    matches!(
        tag,
        "C" | "I"
            | "X"
            | "J"
            | "cm"
            | "cf"
            | "D"
            | "F"
            | "M"
            | "p"
            | "mo"
            | "cs"
            | "pf"
            | "lf"
            | "pt"
    )
}

fn merge_facts(mut target: Value, facts: Value, bodies: Vec<BodyFrame>) -> Result<Value, String> {
    let method_ids = all_methods(&target)
        .filter_map(|method| method["id"].as_str().map(str::to_string))
        .collect::<HashSet<_>>();
    for call in facts["calls"].as_array().ok_or("decoded calls missing")? {
        let caller = call["caller_method_id"]
            .as_str()
            .ok_or("call caller missing")?;
        if !method_ids.contains(caller) {
            return Err("call references unknown method".into());
        }
    }
    for fact in facts["behavior"]
        .as_array()
        .ok_or("decoded behavior missing")?
    {
        let id = fact["method_id"]
            .as_str()
            .ok_or("behavior method missing")?;
        let method = method_mut(&mut target, id).ok_or("behavior references unknown method")?;
        let (field, scalar) = match fact["family"].as_str() {
            Some("fc") => ("control_flow", false),
            Some("fd") => ("data_flow", false),
            Some("se") => ("side_effects", true),
            Some("ec") => ("execution_contexts", true),
            _ => return Err("unknown behavior family".into()),
        };
        let value = if scalar {
            fact["value"][0].clone()
        } else {
            fact["value"].clone()
        };
        method[field].as_array_mut().unwrap().push(value);
    }
    let mut body_ids = Vec::new();
    for frame in bodies {
        if body_ids.iter().any(|id| id == &frame.method_id) {
            return Err("duplicate body method".into());
        }
        let method =
            method_mut(&mut target, &frame.method_id).ok_or("body references unknown method")?;
        method["body"] = json!(frame.body);
        method["body_start"] = json!(frame.start);
        method["body_end"] = json!(frame.end);
        body_ids.push(frame.method_id);
    }
    target["calls"] = facts["calls"].clone();
    target["imports"] = facts["imports"].clone();
    target["type_aliases"] = facts["type_aliases"].clone();
    target["mode"]["exact_body_method_ids"] = json!(body_ids);
    target["mode"]["source_requirement"] = json!(SOURCE_REQUIREMENT);
    Ok(target)
}

/// Decode a complete A3 document and reject framing/count/reference corruption.
pub fn decode(input: &[u8]) -> Result<Value, String> {
    let section = marker(input, b"\nY|").ok_or("missing body section")?;
    let text = std::str::from_utf8(&input[..section]).map_err(|_| "invalid A3 text UTF-8")?;
    let mut declaration_lines = Vec::new();
    let mut fact_lines = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if index == 0 || declaration_tag(line.split('|').next().unwrap_or_default()) {
            declaration_lines.push(line);
        } else {
            fact_lines.push(line);
        }
    }
    let owner_count = declaration_lines
        .iter()
        .filter(|line| line.starts_with("C|") || line.starts_with("I|"))
        .count();
    let method_count = declaration_lines
        .iter()
        .filter(|line| line.starts_with("M|"))
        .count();
    let declaration_wire = format!(
        "{}\nZ|{owner_count}|{method_count}|0|0\n",
        declaration_lines.join("\n")
    );
    let declarations = decode_declarations(&declaration_wire)?;
    let facts = decode_facts(&fact_lines.join("\n"))?;
    let after_marker = &input[section + 1..];
    let header_end = after_marker
        .iter()
        .position(|byte| *byte == b'\n')
        .ok_or("truncated body section")?;
    let body_header =
        std::str::from_utf8(&after_marker[..header_end]).map_err(|_| "invalid body section")?;
    let body_count = body_header
        .strip_prefix("Y|")
        .ok_or("invalid body section")?
        .parse::<usize>()
        .map_err(|_| "invalid body count")?;
    let mut remaining = &after_marker[header_end + 1..];
    let mut bodies = Vec::new();
    for _ in 0..body_count {
        let (body, rest) = decode_body(remaining)?;
        bodies.push(body);
        remaining = rest;
    }
    let terminal = std::str::from_utf8(remaining).map_err(|_| "invalid terminal UTF-8")?;
    let row = terminal
        .trim_end_matches('\n')
        .split('|')
        .collect::<Vec<_>>();
    if row.len() != 5 || row[0] != "Z" {
        return Err("invalid terminal record".into());
    }
    let expected = [
        owner_count,
        method_count,
        facts["calls"].as_array().unwrap().len(),
        body_count,
    ];
    for (column, expected) in row[1..].iter().zip(expected) {
        if column.parse::<usize>().ok() != Some(expected) {
            return Err("terminal count mismatch".into());
        }
    }
    merge_facts(declarations, facts, bodies)
}

/// Fidelity-specific normalized target used by deterministic A3 equality tests.
pub fn target(normalized: &Value) -> Result<Value, String> {
    let fidelity = normalized["mode"]["fidelity"]
        .as_str()
        .ok_or("target fidelity missing")?;
    let mut target = json!({
        "schema":"clean-ctx/file-context","schema_version":3,
        "file":normalized["file"],
        "mode":{"fidelity":normalized["mode"]["fidelity"],"exact_body_method_ids":[],"source_requirement":SOURCE_REQUIREMENT},
        "classes":normalized["classes"],"interfaces":normalized["interfaces"],
        "imports":normalized["imports"],"type_aliases":normalized["type_aliases"],"calls":normalized["calls"]
    });
    let body_ids = all_methods(normalized)
        .filter(|method| normalized["mode"]["fidelity"] == "edit" && !method["body"].is_null())
        .map(|method| method["id"].clone())
        .collect::<Vec<_>>();
    target["mode"]["exact_body_method_ids"] = json!(body_ids);
    for family in ["classes", "interfaces"] {
        for owner in target[family]
            .as_array_mut()
            .ok_or("target owners missing")?
        {
            let mut names = std::collections::HashMap::new();
            for method in owner["methods"]
                .as_array()
                .ok_or("target methods missing")?
            {
                *names
                    .entry(
                        method["name"]
                            .as_str()
                            .ok_or("target method name missing")?
                            .to_string(),
                    )
                    .or_insert(0usize) += 1;
            }
            for method in owner["methods"]
                .as_array_mut()
                .ok_or("target methods missing")?
            {
                if fidelity == "low" && names[method["name"].as_str().unwrap()] == 1 {
                    method["parameters"] = json!([]);
                }
                if matches!(fidelity, "low" | "medium") {
                    method["control_flow"] = json!([]);
                    method["data_flow"] = json!([]);
                    method["side_effects"] = json!([]);
                    method["execution_contexts"] = json!([]);
                }
                if fidelity != "edit" {
                    method["body"] = Value::Null;
                    method["body_start"] = Value::Null;
                    method["body_end"] = Value::Null;
                }
            }
        }
    }
    Ok(target)
}

#[cfg(test)]
#[path = "../../tests/ir/compact_a3_document.rs"]
mod tests;
