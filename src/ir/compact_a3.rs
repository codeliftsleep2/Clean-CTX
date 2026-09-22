//! Research-only COMPACT-A3 Phase 1A declaration/signature codec.
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fmt::Write;
pub mod document;
pub mod facts;
pub const SCHEMA_VERSION: u64 = 3;

fn handle<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    if value.is_empty() || value.contains(['|', '\r', '\n']) {
        return Err(format!("invalid {field}"));
    }
    Ok(value)
}
fn quoted(value: &Value, field: &str) -> Result<String, String> {
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    serde_json::to_string(value).map_err(|error| error.to_string())
}
fn optional_quoted(value: &Value, field: &str) -> Result<String, String> {
    if value.is_null() {
        Ok("-".into())
    } else {
        quoted(value, field)
    }
}
fn string_values(values: &Value, field: &str) -> Result<Vec<String>, String> {
    values
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
        .map(|value| quoted(value, field))
        .collect()
}

fn occurrence_rows(
    output: &mut String,
    tag: &str,
    groups: &Value,
    field: &str,
) -> Result<(), String> {
    for (occurrence, group) in groups
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
        .enumerate()
    {
        let values = string_values(group, field)?;
        write!(output, "{tag}|{occurrence}|{}", values.len()).unwrap();
        for value in values {
            write!(output, "|{value}").unwrap();
        }
        output.push('\n');
    }
    Ok(())
}

fn pattern_rows(output: &mut String, patterns: &Value) -> Result<(), String> {
    for pattern in patterns.as_array().ok_or("patterns must be an array")? {
        let name = quoted(&pattern["name"], "pattern name")?;
        let args = string_values(&pattern["args"], "pattern args")?;
        write!(output, "pt|{name}|{}", args.len()).unwrap();
        for argument in args {
            write!(output, "|{argument}").unwrap();
        }
        output.push('\n');
    }
    Ok(())
}

fn method_rows(output: &mut String, methods: &Value, count: &mut usize) -> Result<(), String> {
    for method in methods.as_array().ok_or("methods must be an array")? {
        let parameters = method["parameters"]
            .as_array()
            .ok_or("parameters must be an array")?;
        writeln!(
            output,
            "M|{}|{}|{}|{}",
            handle(&method["id"], "method id")?,
            quoted(&method["name"], "method name")?,
            parameters.len(),
            optional_quoted(&method["return_type"], "return type")?
        )
        .unwrap();
        *count += 1;
        for parameter in parameters {
            writeln!(
                output,
                "p|{}|{}|{}",
                handle(&parameter["id"], "parameter id")?,
                quoted(&parameter["name"], "parameter name")?,
                optional_quoted(&parameter["type"], "parameter type")?
            )
            .unwrap();
        }
        occurrence_rows(
            output,
            "mo",
            &method["modifier_occurrences"],
            "method modifiers",
        )?;
        occurrence_rows(
            output,
            "cs",
            &method["control_summary_occurrences"],
            "control summaries",
        )?;
        occurrence_rows(
            output,
            "pf",
            &method["pattern_fact_occurrences"],
            "pattern facts",
        )?;
        occurrence_rows(
            output,
            "lf",
            &method["legacy_flag_occurrences"],
            "legacy flags",
        )?;
        pattern_rows(output, &method["patterns"])?;
    }
    Ok(())
}

fn owner_rows(
    output: &mut String,
    owners: &Value,
    interface: bool,
    owner_count: &mut usize,
    method_count: &mut usize,
) -> Result<(), String> {
    for owner in owners.as_array().ok_or("owners must be an array")? {
        if interface {
            writeln!(
                output,
                "I|{}|{}",
                handle(&owner["id"], "interface id")?,
                quoted(&owner["name"], "interface name")?
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "C|{}|{}|{}",
                handle(&owner["id"], "class id")?,
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
            for (occurrence, group) in owner["injection_occurrences"]
                .as_array()
                .ok_or("injections must be an array")?
                .iter()
                .enumerate()
            {
                let dependencies = string_values(group, "injection dependencies")?;
                write!(output, "D|{occurrence}|{}", dependencies.len()).unwrap();
                for dependency in dependencies {
                    write!(output, "|{dependency}").unwrap();
                }
                output.push('\n');
            }
        }
        occurrence_rows(
            output,
            "cm",
            &owner["modifier_occurrences"],
            "owner modifiers",
        )?;
        if !interface {
            pattern_rows(output, &owner["patterns"])?;
        }
        for field in owner["fields"]
            .as_array()
            .ok_or("fields must be an array")?
        {
            writeln!(
                output,
                "F|{}|{}|{}",
                handle(&field["id"], "field id")?,
                quoted(&field["name"], "field name")?,
                optional_quoted(&field["type"], "field type")?
            )
            .unwrap();
        }
        method_rows(output, &owner["methods"], method_count)?;
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
        "A3|3|{fidelity}|{}|{}|{}\n",
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
    )?;
    owner_rows(
        &mut output,
        &normalized["interfaces"],
        true,
        &mut owner_count,
        &mut method_count,
    )?;
    writeln!(output, "Z|{owner_count}|{method_count}|0|0").unwrap();
    Ok(output)
}

fn split_columns(line: &str) -> Result<Vec<&str>, String> {
    let mut columns = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
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
            columns.push(&line[start..index]);
            start = index + 1;
        }
    }
    if quoted || escaped {
        return Err("unterminated quoted column".into());
    }
    columns.push(&line[start..]);
    Ok(columns)
}

fn parsed_string(column: &str) -> Result<Value, String> {
    serde_json::from_str::<String>(column)
        .map(Value::String)
        .map_err(|_| "invalid JSON-string column".into())
}

fn parsed_handle(column: &str, family: Option<char>) -> Result<Value, String> {
    if column.is_empty()
        || column.contains(['|', '\r', '\n'])
        || family.is_some_and(|expected| !column.starts_with(expected))
    {
        return Err("invalid canonical handle".into());
    }
    Ok(Value::String(column.into()))
}

fn parsed_optional(column: &str) -> Result<Value, String> {
    if column == "-" {
        Ok(Value::Null)
    } else {
        parsed_string(column)
    }
}

fn empty_method(id: Value, name: Value, return_type: Value) -> Value {
    json!({
        "id": id, "name": name, "parameters": [], "return_type": return_type,
        "modifier_occurrences": [], "control_summary_occurrences": [],
        "pattern_fact_occurrences": [], "legacy_flag_occurrences": [], "patterns": [],
        "body": null, "body_start": null, "body_end": null,
        "control_flow": [], "data_flow": [], "side_effects": [], "execution_contexts": []
    })
}

fn occurrence(columns: &[&str]) -> Result<(usize, Value), String> {
    if columns.len() < 3 {
        return Err("short occurrence record".into());
    }
    let index = columns[1]
        .parse::<usize>()
        .map_err(|_| "invalid occurrence")?;
    let count = columns[2]
        .parse::<usize>()
        .map_err(|_| "invalid value count")?;
    if columns.len() != count + 3 {
        return Err("occurrence value-count mismatch".into());
    }
    Ok((
        index,
        Value::Array(
            columns[3..]
                .iter()
                .map(|column| parsed_string(column))
                .collect::<Result<_, _>>()?,
        ),
    ))
}

/// Decode the declaration subset into its normalized target.
pub fn decode_declarations(input: &str) -> Result<Value, String> {
    let lines = input.lines().collect::<Vec<_>>();
    if lines.len() < 2 {
        return Err("truncated A3 document".into());
    }
    let header = split_columns(lines[0])?;
    if header.len() != 6 || header[0] != "A3" || header[1] != "3" {
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
                let id = parsed_handle(columns[1], Some(if interface { 'I' } else { 'C' }))?;
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
                    json!({"kind":"class","id":id,"name":parsed_string(columns[2])?,"synthetic":synthetic,"methods":[],"fields":[],"modifier_occurrences":[],"class_flag_occurrences":[],"extends":null,"implements":[],"injection_occurrences":[],"patterns":[]})
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
                if columns.len() != 4 {
                    return Err("invalid field record".into());
                }
                let (interface, owner) = current_owner.ok_or("field outside owner")?;
                let id = parsed_handle(columns[1], Some('F'))?;
                if !ids.insert(id.as_str().unwrap().to_string()) {
                    return Err("duplicate canonical ID".into());
                }
                let value = json!({"id":id,"name":parsed_string(columns[2])?,"type":parsed_optional(columns[3])?});
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                owners[owner]["fields"].as_array_mut().unwrap().push(value);
                current_method = None;
            }
            "M" => {
                if columns.len() != 5 {
                    return Err("invalid method record".into());
                }
                let (interface, owner) = current_owner.ok_or("method outside owner")?;
                let id = parsed_handle(columns[1], Some('M'))?;
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
                method_count += 1;
            }
            "p" => {
                if columns.len() != 4 {
                    return Err("invalid parameter record".into());
                }
                let (interface, owner, method) =
                    current_method.ok_or("parameter outside method")?;
                let id = parsed_handle(columns[1], Some('P'))?;
                if !ids.insert(id.as_str().unwrap().to_string()) {
                    return Err("duplicate canonical ID".into());
                }
                let value = json!({"id":id,"name":parsed_string(columns[2])?,"type":parsed_optional(columns[3])?});
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                owners[owner]["methods"][method]["parameters"]
                    .as_array_mut()
                    .unwrap()
                    .push(value);
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
            "cm" | "cf" | "D" => {
                let (interface, owner) = current_owner.ok_or("owner fact outside owner")?;
                let (index, group) = occurrence(&columns)?;
                let field = match columns[0] {
                    "cm" => "modifier_occurrences",
                    "cf" => "class_flag_occurrences",
                    _ => "injection_occurrences",
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
                if index != groups.len() {
                    return Err("non-contiguous occurrence".into());
                }
                groups.push(group);
                current_method = None;
            }
            "mo" | "cs" | "pf" | "lf" => {
                let (interface, owner, method) =
                    current_method.ok_or("method fact outside method")?;
                let (index, group) = occurrence(&columns)?;
                let field = match columns[0] {
                    "mo" => "modifier_occurrences",
                    "cs" => "control_summary_occurrences",
                    "pf" => "pattern_fact_occurrences",
                    _ => "legacy_flag_occurrences",
                };
                let owners = if interface {
                    &mut interfaces
                } else {
                    &mut classes
                };
                let groups = owners[owner]["methods"][method][field]
                    .as_array_mut()
                    .unwrap();
                if index != groups.len() {
                    return Err("non-contiguous occurrence".into());
                }
                groups.push(group);
            }
            "pt" => {
                if columns.len() < 3 {
                    return Err("short pattern record".into());
                }
                let count = columns[2]
                    .parse::<usize>()
                    .map_err(|_| "invalid pattern count")?;
                if columns.len() != count + 3 {
                    return Err("pattern count mismatch".into());
                }
                let value = json!({"name":parsed_string(columns[1])?,"args":columns[3..].iter().map(|column|parsed_string(column)).collect::<Result<Vec<_>,_>>()?});
                if let Some((interface, owner, method)) = current_method {
                    let owners = if interface {
                        &mut interfaces
                    } else {
                        &mut classes
                    };
                    owners[owner]["methods"][method]["patterns"]
                        .as_array_mut()
                        .unwrap()
                        .push(value);
                } else {
                    let (interface, owner) = current_owner.ok_or("pattern outside scope")?;
                    if interface {
                        return Err("owner pattern under interface".into());
                    }
                    let owners = if interface {
                        &mut interfaces
                    } else {
                        &mut classes
                    };
                    owners[owner]["patterns"]
                        .as_array_mut()
                        .unwrap()
                        .push(value);
                }
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
    for owners in [&mut classes, &mut interfaces] {
        for owner in owners {
            for method in owner["methods"].as_array_mut().unwrap() {
                let expected = method["declared_arity"].as_u64().unwrap() as usize;
                method.as_object_mut().unwrap().remove("declared_arity");
                if method["parameters"].as_array().unwrap().len() != expected {
                    return Err("parameter arity mismatch".into());
                }
            }
        }
    }
    Ok(json!({
        "schema":"clean-ctx/file-context","schema_version":3,
        "file":{"id":parsed_handle(header[3],None)?,"ir_version":header[4].parse::<u64>().map_err(|_|"invalid IR version")?,"source_path":parsed_string(header[5])?},
        "mode":{"fidelity":fidelity},"classes":classes,"interfaces":interfaces
    }))
}

#[cfg(test)]
#[path = "../tests/ir/compact_a3.rs"]
mod tests;
