use serde_json::{json, Value};

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
    serde_json::from_str::<String>(column)
        .map(Value::String)
        .map_err(|_| "invalid JSON-string column".into())
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

pub(super) fn occurrence(columns: &[&str]) -> Result<(usize, Value), String> {
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

pub(super) fn pattern_fact_occurrence(columns: &[&str]) -> Result<(usize, Value), String> {
    if columns.len() < 3 {
        return Err("short pattern-fact record".into());
    }
    let occurrence = columns[1]
        .parse::<usize>()
        .map_err(|_| "invalid occurrence")?;
    let expected = columns[2]
        .parse::<usize>()
        .map_err(|_| "invalid fact count")?;
    let mut facts = Vec::new();
    let mut column = 3;
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
    Ok((occurrence, Value::Array(facts)))
}
