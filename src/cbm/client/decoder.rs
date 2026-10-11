//! Strict decoding of the pinned provider's tables and grouped identities.
use super::{CbmError, QueryRows, extract_trace_edges};
use serde_json::{Map, Value, json};

fn invalid(message: &str) -> CbmError {
    CbmError::ParseError(message.into())
}
fn string<'a>(value: &'a Value, field: &str) -> Result<&'a str, CbmError> {
    value.as_str().ok_or_else(|| invalid(field))
}
pub(super) fn table(value: &Value, columns: &str) -> Result<QueryRows, CbmError> {
    let columns = value[columns]
        .as_array()
        .ok_or_else(|| invalid("missing table columns"))?
        .iter()
        .map(|v| string(v, "nonstring column").map(str::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    let rows = value["rows"]
        .as_array()
        .ok_or_else(|| invalid("missing table rows"))?
        .iter()
        .map(|v| {
            let row = v.as_array().ok_or_else(|| invalid("nonarray table row"))?;
            if row.len() != columns.len() {
                return Err(invalid("table row width mismatch"));
            }
            Ok(row.clone())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(QueryRows { columns, rows })
}
fn grouped(value: &Value, search: bool) -> Result<Vec<Value>, CbmError> {
    if value["qn_rule"] != "qn = qn_prefix == \"\" ? name : qn_prefix + \".\" + name" {
        return Err(invalid("unsupported qualified-name rule"));
    }
    let groups = value["groups"]
        .as_array()
        .ok_or_else(|| invalid("missing grouped rows"))?;
    // Validate columns even when there are no groups.
    let header = table(&json!({"cols": value["cols"], "rows": []}), "cols")?;
    let required: &[&str] = if search {
        &["name", "label", "lines", "in", "out"]
    } else {
        &["name", "hop"]
    };
    if required
        .iter()
        .any(|key| !header.columns.iter().any(|c| c == key))
    {
        return Err(invalid("missing required grouped column"));
    }
    let mut seen = std::collections::HashSet::new();
    if header.columns.iter().any(|c| !seen.insert(c)) {
        return Err(invalid("duplicate grouped column"));
    }
    let mut result = Vec::new();
    for group in groups {
        let prefix = string(&group["qn_prefix"], "missing qualified-name prefix")?;
        let file = if search {
            Some(string(&group["file"], "missing group file")?)
        } else {
            None
        };
        let rows = table(
            &json!({"cols": value["cols"], "rows": group["rows"]}),
            "cols",
        )?;
        for row in rows.rows {
            let mut object: Map<String, Value> = rows.columns.iter().cloned().zip(row).collect();
            let name = string(&object["name"], "invalid grouped name")?;
            let qn = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}.{name}")
            };
            if search {
                string(&object["label"], "invalid label")?;
                string(&object["lines"], "invalid lines")?;
                for (source, target) in [("in", "in_degree"), ("out", "out_degree")] {
                    if object[source].as_u64().is_none() {
                        return Err(invalid("invalid degree"));
                    }
                    object.insert(target.into(), object[source].clone());
                }
            } else if object["hop"].as_u64().is_none() {
                return Err(invalid("invalid trace hop"));
            }
            object.insert("qualified_name".into(), json!(qn));
            if let Some(file) = file {
                object.insert("file_path".into(), json!(file));
            }
            result.push(Value::Object(object));
        }
    }
    Ok(result)
}
pub(super) fn search(value: &Value) -> Result<Vec<Value>, CbmError> {
    grouped(value, true)
}
pub(super) fn trace(value: &Value, root: &str, direction: &str) -> Result<Vec<Value>, CbmError> {
    if value["function"] != root || value["direction"] != direction {
        return Err(invalid("trace root/direction mismatch"));
    }
    let legs: &[&str] = match direction {
        "inbound" => &["callers"],
        "outbound" => &["callees"],
        "both" => &["callers", "callees"],
        _ => return Err(invalid("unsupported trace direction")),
    };
    let mut normalized = Map::new();
    for leg in legs {
        normalized.insert((*leg).into(), json!(grouped(&value[*leg], false)?));
    }
    Ok(extract_trace_edges(&Value::Object(normalized), root))
}
pub(super) fn architecture(mut value: Value) -> Result<Value, CbmError> {
    let root = value
        .as_object_mut()
        .ok_or_else(|| invalid("architecture is not an object"))?;
    for section in ["packages", "boundaries"] {
        // The pinned producer omits zero-count sections.
        let Some(data) = root.get(section) else {
            root.insert(section.into(), json!([]));
            continue;
        };
        let table = table(data, "cols")?;
        let required: &[&str] = if section == "packages" {
            &["name", "nodes", "fan_in", "fan_out"]
        } else {
            &["from", "to", "calls"]
        };
        let mut seen = std::collections::HashSet::new();
        if table.columns.iter().any(|c| !seen.insert(c))
            || required
                .iter()
                .any(|k| !table.columns.iter().any(|c| c == k))
        {
            return Err(invalid("invalid architecture columns"));
        }
        let mut normalized = Vec::new();
        for row in table.rows {
            let mut object: Map<String, Value> = table.columns.iter().cloned().zip(row).collect();
            for key in required {
                let is_string = matches!(*key, "name" | "from" | "to");
                if (is_string && object[*key].as_str().is_none())
                    || (!is_string && object[*key].as_u64().is_none())
                {
                    return Err(invalid("invalid architecture cell"));
                }
            }
            if section == "packages" {
                object.insert("node_count".into(), object["nodes"].clone());
            }
            normalized.push(Value::Object(object));
        }
        root.insert(section.into(), json!(normalized));
    }
    Ok(value)
}
