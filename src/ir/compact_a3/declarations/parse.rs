use serde_json::{json, Value};

pub(super) fn is_bare_unsigned(column: &str) -> bool {
    !column.is_empty() && column.bytes().all(|byte| byte.is_ascii_digit())
}

pub(super) fn split_columns(line: &str) -> Result<Vec<&str>, String> {
    let mut columns = Vec::new();
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

pub(super) fn parsed_string(column: &str) -> Result<Value, String> {
    if column.starts_with('"') {
        serde_json::from_str::<String>(column)
            .map(Value::String)
            .map_err(|_| "invalid JSON-string column".into())
    } else {
        if column.is_empty() || column == "-" || column.contains(['|', '\r', '\n']) {
            return Err("invalid bare-string column".into());
        }
        Ok(Value::String(column.into()))
    }
}

pub(super) fn parsed_handle(column: &str, family: Option<char>) -> Result<Value, String> {
    if column.is_empty()
        || column.contains(['|', '\r', '\n'])
        || family.is_some_and(|expected| !column.starts_with(expected))
    {
        return Err("invalid canonical handle".into());
    }
    Ok(Value::String(column.into()))
}

pub(super) fn parsed_scoped_handle(column: &str, family: char) -> Result<Value, String> {
    if column.is_empty() || !column.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("invalid scoped handle".into());
    }
    Ok(Value::String(format!("{family}{column}")))
}

pub(super) fn parsed_optional(column: &str) -> Result<Value, String> {
    if column == "-" {
        Ok(Value::Null)
    } else {
        parsed_string(column)
    }
}

pub(super) fn empty_method(id: Value, name: Value, return_type: Value) -> Value {
    json!({
        "id": id, "name": name, "parameters": [], "return_type": return_type,
        "modifier_occurrences": [], "control_summary_occurrences": [],
        "pattern_fact_occurrences": [], "legacy_flag_occurrences": [], "patterns": [],
        "body": null, "body_start": null, "body_end": null,
        "control_flow": [], "data_flow": [], "side_effects": [], "execution_contexts": []
    })
}

pub(super) fn occurrence(columns: &[&str]) -> Result<Value, String> {
    if columns.len() < 2 {
        return Err("short occurrence record".into());
    }
    // A leading bare unsigned integer is the explicit count; any other first
    // column is a single elided value (count implied = 1).
    if is_bare_unsigned(columns[1]) {
        let count = columns[1]
            .parse::<usize>()
            .map_err(|_| "invalid value count")?;
        if columns.len() != count + 2 {
            return Err("occurrence value-count mismatch".into());
        }
        Ok(Value::Array(
            columns[2..]
                .iter()
                .map(|column| parsed_string(column))
                .collect::<Result<_, _>>()?,
        ))
    } else {
        if columns.len() != 2 {
            return Err("occurrence value-count mismatch".into());
        }
        Ok(Value::Array(vec![parsed_string(columns[1])?]))
    }
}

pub(super) fn pattern_fact_occurrence(columns: &[&str]) -> Result<Value, String> {
    if columns.len() < 2 {
        return Err("short pattern-fact record".into());
    }
    // A leading bare unsigned integer is the explicit fact count; any other
    // first column is a single elided fact (count implied = 1).
    let (start, expected) = if is_bare_unsigned(columns[1]) {
        (
            2,
            columns[1]
                .parse::<usize>()
                .map_err(|_| "invalid fact count")?,
        )
    } else {
        (1, 1)
    };
    let mut facts = Vec::new();
    let mut column = start;
    while column < columns.len() {
        let kind = parsed_string(columns[column])?;
        let kind_text = kind.as_str().unwrap();
        column += 1;
        let fact = match kind_text {
            "CTOR" | "OBSERVABLE" | "OVERRIDE" => json!({"k":kind}),
            "GETTER" | "SETTER" => {
                let value = columns.get(column).ok_or("missing pattern-fact value")?;
                column += 1;
                json!({"k":kind,"v":parsed_string(value)?})
            }
            _ => return Err("unknown pattern-fact kind".into()),
        };
        facts.push(fact);
    }
    if facts.len() != expected {
        return Err("pattern-fact count mismatch".into());
    }
    Ok(Value::Array(facts))
}
