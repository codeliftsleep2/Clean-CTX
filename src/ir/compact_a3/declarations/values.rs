use serde_json::Value;

pub(super) fn handle<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    let value = value
        .as_str()
        .ok_or_else(|| format!("{field} must be a string"))?;
    if value.is_empty() || value.contains(['|', '\r', '\n']) {
        return Err(format!("invalid {field}"));
    }
    Ok(value)
}

pub(super) fn scoped_handle<'a>(
    value: &'a Value,
    family: char,
    field: &str,
) -> Result<&'a str, String> {
    let value = handle(value, field)?;
    let suffix = value
        .strip_prefix(family)
        .ok_or_else(|| format!("invalid {field} family"))?;
    if suffix.is_empty() || !suffix.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("invalid {field}"));
    }
    Ok(suffix)
}

pub(super) fn quoted(value: &Value, field: &str) -> Result<String, String> {
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

pub(super) fn optional_quoted(value: &Value, field: &str) -> Result<String, String> {
    if value.is_null() {
        Ok("-".into())
    } else {
        quoted(value, field)
    }
}

pub(super) fn string_values(values: &Value, field: &str) -> Result<Vec<String>, String> {
    values
        .as_array()
        .ok_or_else(|| format!("{field} must be an array"))?
        .iter()
        .map(|value| quoted(value, field))
        .collect()
}
