//! Research-only COMPACT-A3 declaration/signature codec.
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
mod parse;
mod values;
use parse::{
    empty_method, is_bare_unsigned, occurrence, parsed_handle, parsed_optional, parsed_string,
    split_columns,
};
use values::{handle, optional_quoted, quoted, scoped_handle, string_values};
pub const SCHEMA_VERSION: u64 = 4;

fn occurrence_rows(
    output: &mut String,
    tag: &str,
    groups: &Value,
    field: &str,
) -> Result<(), String> {
    for group in groups
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
    {
        let values = string_values(group, field)?;
        if values.len() == 1 && !is_bare_unsigned(&values[0]) {
            write!(output, "{tag}|{}", values[0]).unwrap();
        } else {
            write!(output, "{tag}|{}", values.len()).unwrap();
            for value in values {
                write!(output, "|{value}").unwrap();
            }
        }
        output.push('\n');
    }
    Ok(())
}

/// Declaration modifiers dropped at Medium fidelity. This re-expresses the
/// legacy `compaction::modifiers::MODIFIERS_MEDIUM` access/visibility and
/// declaration-kind set in the IR's `DeclarationModifier` vocabulary. Low drops
/// every modifier; High and Edit keep all.
pub(crate) const ACCESS_MODIFIERS: [&str; 5] =
    ["EXPORT", "STATIC", "PRIVATE", "PROTECTED", "ABSTRACT"];

/// Filter `modifier_occurrences` (occurrence groups) for the declared fidelity.
/// Order, duplicates, and empty groups are preserved; only modifier values are
/// removed.
pub(crate) fn filtered_modifiers(groups: &Value, fidelity: &str) -> Result<Value, String> {
    let groups = groups
        .as_array()
        .ok_or("modifier occurrences must be an array")?;
    let filtered = groups
        .iter()
        .map(|group| {
            let values = group
                .as_array()
                .ok_or("modifier occurrence group must be an array")?;
            let kept = values
                .iter()
                .filter(|value| match fidelity {
                    "low" => false,
                    "medium" => match value.as_str() {
                        Some(text) => !ACCESS_MODIFIERS.contains(&text),
                        None => true,
                    },
                    _ => true,
                })
                .cloned()
                .collect::<Vec<_>>();
            Ok(Value::Array(kept))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Value::Array(filtered))
}

fn method_rows(
    output: &mut String,
    methods: &Value,
    count: &mut usize,
    fidelity: &str,
) -> Result<(), String> {
    let low = fidelity == "low";
    let methods = methods.as_array().ok_or("methods must be an array")?;
    let mut names = HashMap::new();
    for method in methods {
        *names
            .entry(
                method["name"]
                    .as_str()
                    .ok_or("method name must be a string")?,
            )
            .or_insert(0usize) += 1;
    }
    for method in methods {
        let parameters = method["parameters"]
            .as_array()
            .ok_or("parameters must be an array")?;
        writeln!(
            output,
            "M|{}|{}|{}|{}",
            scoped_handle(&method["id"], 'M', "method id")?,
            quoted(&method["name"], "method name")?,
            parameters.len(),
            optional_quoted(&method["return_type"], "return type")?
        )
        .unwrap();
        *count += 1;
        if !parameters.is_empty() && (!low || names[method["name"].as_str().unwrap()] > 1) {
            write!(output, "p|{}", parameters.len()).unwrap();
            for parameter in parameters {
                write!(
                    output,
                    "|{}|{}|{}",
                    scoped_handle(&parameter["id"], 'P', "parameter id")?,
                    quoted(&parameter["name"], "parameter name")?,
                    optional_quoted(&parameter["type"], "parameter type")?
                )
                .unwrap();
            }
            output.push('\n');
        }
        let modifiers = filtered_modifiers(&method["modifier_occurrences"], fidelity)?;
        occurrence_rows(output, "mo", &modifiers, "method modifiers")?;
    }
    Ok(())
}

fn owner_rows(
    output: &mut String,
    owners: &Value,
    interface: bool,
    owner_count: &mut usize,
    method_count: &mut usize,
    fidelity: &str,
) -> Result<(), String> {
    for owner in owners.as_array().ok_or("owners must be an array")? {
        if interface {
            writeln!(
                output,
                "I|{}|{}",
                scoped_handle(&owner["id"], 'I', "interface id")?,
                quoted(&owner["name"], "interface name")?
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "C|{}|{}|{}",
                scoped_handle(&owner["id"], 'C', "class id")?,
                quoted(&owner["name"], "class name")?,
                u8::from(
                    owner["synthetic"]
                        .as_bool()
                        .ok_or("synthetic must be boolean")?
                )
            )
            .unwrap();
        }
        *owner_count += 1;
        if interface {
            for parent in string_values(&owner["extends"], "interface extends")? {
                writeln!(output, "X|{parent}").unwrap();
            }
        } else {
            if !owner["extends"].is_null() {
                writeln!(output, "X|{}", quoted(&owner["extends"], "class extends")?).unwrap();
            }
            for implemented in string_values(&owner["implements"], "implements")? {
                writeln!(output, "J|{implemented}").unwrap();
            }
            occurrence_rows(
                output,
                "cf",
                &owner["class_flag_occurrences"],
                "class flags",
            )?;
        }
        let modifiers = filtered_modifiers(&owner["modifier_occurrences"], fidelity)?;
        occurrence_rows(output, "cm", &modifiers, "owner modifiers")?;
        let fields = owner["fields"]
            .as_array()
            .ok_or("fields must be an array")?;
        if !fields.is_empty() {
            write!(output, "F|{}", fields.len()).unwrap();
            for field in fields {
                write!(
                    output,
                    "|{}|{}|{}",
                    scoped_handle(&field["id"], 'F', "field id")?,
                    quoted(&field["name"], "field name")?,
                    optional_quoted(&field["type"], "field type")?
                )
                .unwrap();
            }
            output.push('\n');
        }
        method_rows(output, &owner["methods"], method_count, fidelity)?;
    }
    Ok(())
}

/// Research-only declaration subset; it omits calls, behavior, imports, and bodies.
pub fn encode_declarations(normalized: &Value) -> Result<String, String> {
    let mode = normalized["mode"]["fidelity"]
        .as_str()
        .ok_or("mode fidelity must be a string")?;
    let fidelity = match mode {
        "low" => "L",
        "medium" => "M",
        "high" => "H",
        "edit" => "E",
        _ => return Err(format!("unsupported A3 fidelity: {mode}")),
    };
    let mut output = format!(
        "A3|4|{fidelity}|{}|{}|{}\n",
        handle(&normalized["file"]["id"], "file id")?,
        normalized["file"]["ir_version"]
            .as_u64()
            .ok_or("IR version must be unsigned")?,
        quoted(&normalized["file"]["source_path"], "source path")?
    );
    let mut owner_count = 0;
    let mut method_count = 0;
    owner_rows(
        &mut output,
        &normalized["classes"],
        false,
        &mut owner_count,
        &mut method_count,
        mode,
    )?;
    owner_rows(
        &mut output,
        &normalized["interfaces"],
        true,
        &mut owner_count,
        &mut method_count,
        mode,
    )?;
    writeln!(output, "Z|{owner_count}|{method_count}|0|0").unwrap();
    Ok(output)
}

/// Decode the declaration subset into its normalized target.
pub fn decode_declarations(input: &str) -> Result<Value, String> {
    let lines = input.lines().collect::<Vec<_>>();
    if lines.len() < 2 {
        return Err("truncated A3 document".into());
    }
    let header = split_columns(lines[0])?;
    if header.len() != 6 || header[0] != "A3" || header[1] != "4" {
        return Err("unsupported A3 header".into());
    }
    let fidelity = match header[2] {
        "L" => "low",
        "M" => "medium",
        "H" => "high",
        "E" => "edit",
        _ => return Err("unknown A3 fidelity".into()),
    };
    let mut classes = Vec::<Value>::new();
    let mut interfaces = Vec::<Value>::new();
    let mut current_owner: Option<(bool, usize)> = None;
    let mut current_method: Option<(bool, usize, usize)> = None;
    let mut ids = HashSet::new();
    let mut param_ids = HashSet::new();
    let mut owner_count = 0;
    let mut method_count = 0;
    let mut terminal = false;

    for line in &lines[1..] {
        if terminal {
            return Err("content after terminal record".into());
        }
        let columns = split_columns(line)?;
        match columns.first().copied().unwrap_or_default() {
            "C" | "I" => {
                let interface = columns[0] == "I";
                if columns.len() != if interface { 3 } else { 4 } {
                    return Err("invalid owner record".into());
                }
                let id =
                    parse::parsed_scoped_handle(columns[1], if interface { 'I' } else { 'C' })?;
                if !ids.insert(id.as_str().unwrap().to_string()) {
                    return Err("duplicate canonical ID".into());
                }
                let owner = if interface {
                    json!({"kind":"interface","id":id,"name":parsed_string(columns[2])?,"methods":[],"fields":[],"modifier_occurrences":[],"extends":[]})
                } else {
                    let synthetic = match columns[3] {
                        "0" => false,
                        "1" => true,
                        _ => return Err("invalid synthetic flag".into()),
                    };
                    json!({"kind":"class","id":id,"name":parsed_string(columns[2])?,"synthetic":synthetic,"methods":[],"fields":[],"modifier_occurrences":[],"class_flag_occurrences":[],"extends":null,"implements":[]})
                };
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                owners.push(owner);
                current_owner = Some((interface, owners.len() - 1));
                current_method = None;
                owner_count += 1;
            }
            "F" => {
                if columns.len() < 2 {
                    return Err("invalid field record".into());
                }
                let (interface, owner) = current_owner.ok_or("field outside owner")?;
                let count = columns[1]
                    .parse::<usize>()
                    .map_err(|_| "invalid field count")?;
                if columns.len() != 2 + count * 3 {
                    return Err("field count mismatch".into());
                }
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                for entry in columns[2..].chunks_exact(3) {
                    let id = parse::parsed_scoped_handle(entry[0], 'F')?;
                    if !ids.insert(id.as_str().unwrap().to_string()) {
                        return Err("duplicate canonical ID".into());
                    }
                    owners[owner]["fields"].as_array_mut().unwrap().push(
                        json!({"id":id,"name":parsed_string(entry[1])?,"type":parsed_optional(entry[2])?}),
                    );
                }
                current_method = None;
            }
            "M" => {
                if columns.len() != 5 {
                    return Err("invalid method record".into());
                }
                let (interface, owner) = current_owner.ok_or("method outside owner")?;
                let id = parse::parsed_scoped_handle(columns[1], 'M')?;
                if !ids.insert(id.as_str().unwrap().to_string()) {
                    return Err("duplicate canonical ID".into());
                }
                let arity = columns[3]
                    .parse::<usize>()
                    .map_err(|_| "invalid method arity")?;
                let mut value =
                    empty_method(id, parsed_string(columns[2])?, parsed_optional(columns[4])?);
                value["declared_arity"] = json!(arity);
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                let methods = owners[owner]["methods"].as_array_mut().unwrap();
                methods.push(value);
                current_method = Some((interface, owner, methods.len() - 1));
                param_ids.clear();
                method_count += 1;
            }
            "p" => {
                if columns.len() < 2 {
                    return Err("invalid parameter record".into());
                }
                let (interface, owner, method) =
                    current_method.ok_or("parameter outside method")?;
                let count = columns[1]
                    .parse::<usize>()
                    .map_err(|_| "invalid parameter count")?;
                if columns.len() != 2 + count * 3 {
                    return Err("parameter count mismatch".into());
                }
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                for entry in columns[2..].chunks_exact(3) {
                    let id = parse::parsed_scoped_handle(entry[0], 'P')?;
                    if !param_ids.insert(id.as_str().unwrap().to_string()) {
                        return Err("duplicate canonical ID".into());
                    }
                    owners[owner]["methods"][method]["parameters"]
                        .as_array_mut()
                        .unwrap()
                        .push(json!({"id":id,"name":parsed_string(entry[1])?,"type":parsed_optional(entry[2])?}));
                }
            }
            "X" | "J" => {
                if columns.len() != 2 {
                    return Err("invalid relation record".into());
                }
                let (interface, owner) = current_owner.ok_or("relation outside owner")?;
                let value = parsed_string(columns[1])?;
                if columns[0] == "J" {
                    if interface {
                        return Err("interface cannot implement".into());
                    }
                    classes[owner]["implements"]
                        .as_array_mut()
                        .unwrap()
                        .push(value);
                } else if interface {
                    interfaces[owner]["extends"]
                        .as_array_mut()
                        .unwrap()
                        .push(value);
                } else if !classes[owner]["extends"].is_null() {
                    return Err("duplicate class extends".into());
                } else {
                    classes[owner]["extends"] = value;
                }
                current_method = None;
            }
            "cm" | "cf" => {
                let (interface, owner) = current_owner.ok_or("owner fact outside owner")?;
                let group = occurrence(&columns)?;
                let field = if columns[0] == "cm" {
                    "modifier_occurrences"
                } else {
                    "class_flag_occurrences"
                };
                if interface && field != "modifier_occurrences" {
                    return Err("class fact under interface".into());
                }
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                let groups = owners[owner][field].as_array_mut().unwrap();
                groups.push(group);
                current_method = None;
            }
            "mo" => {
                let (interface, owner, method) =
                    current_method.ok_or("method fact outside method")?;
                let group = occurrence(&columns)?;
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                let groups = owners[owner]["methods"][method]["modifier_occurrences"]
                    .as_array_mut()
                    .unwrap();
                groups.push(group);
            }
            "Z" => {
                if columns.len() != 5 || columns[3] != "0" || columns[4] != "0" {
                    return Err("invalid Phase 1A terminal".into());
                }
                if columns[1].parse::<usize>().ok() != Some(owner_count)
                    || columns[2].parse::<usize>().ok() != Some(method_count)
                {
                    return Err("terminal count mismatch".into());
                }
                terminal = true;
            }
            _ => return Err("unknown A3 record".into()),
        }
    }
    if !terminal {
        return Err("missing terminal record".into());
    }
    let low = fidelity == "low";
    for owners in [&mut classes, &mut interfaces] {
        for owner in owners {
            let mut names = HashMap::new();
            for method in owner["methods"].as_array().unwrap() {
                *names
                    .entry(method["name"].as_str().unwrap().to_string())
                    .or_insert(0usize) += 1;
            }
            for method in owner["methods"].as_array_mut().unwrap() {
                let expected = method["declared_arity"].as_u64().unwrap() as usize;
                method.as_object_mut().unwrap().remove("declared_arity");
                let actual = method["parameters"].as_array().unwrap().len();
                let omitted_unique =
                    low && names[method["name"].as_str().unwrap()] == 1 && actual == 0;
                if actual != expected && !omitted_unique {
                    return Err("parameter arity mismatch".into());
                }
            }
        }
    }
    Ok(json!({
        "schema":"clean-ctx/file-context","schema_version":4,
        "file":{"id":parsed_handle(header[3],None)?,"ir_version":header[4].parse::<u64>().map_err(|_|"invalid IR version")?,"source_path":parsed_string(header[5])?},
        "mode":{"fidelity":fidelity},"classes":classes,"interfaces":interfaces
    }))
}

#[cfg(test)]
#[path = "../../tests/ir/compact_a3.rs"]
mod tests;
