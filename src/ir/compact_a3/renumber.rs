//! File-local identity renumbering (Phase 3B).
//!
//! Rewrites canonical global-counter handles (`C487`, `M1204`, `F82`, `P633`) to
//! dense file-local ordinals (`C1`, `M3`, `F2`, `P1`) assigned by first appearance.
//! Presentation-only: applied before `encode` and before building the decode target
//! so both sides carry the same local handles. Local handles never leave the payload.

use serde_json::{json, Value};
use std::collections::HashMap;

pub fn renumber(normalized: &Value) -> Result<Value, String> {
    let classes = normalized["classes"].as_array().ok_or("classes must be an array")?;
    let interfaces = normalized["interfaces"].as_array().ok_or("interfaces must be an array")?;

    let mut class_map = HashMap::new();
    let mut interface_map = HashMap::new();
    let mut field_map = HashMap::new();
    let mut method_map = HashMap::new();

    for (i, owner) in classes.iter().enumerate() {
        let id = owner["id"].as_str().ok_or("class id missing")?;
        class_map.insert(id.to_string(), format!("C{}", i + 1));
    }
    for (i, owner) in interfaces.iter().enumerate() {
        let id = owner["id"].as_str().ok_or("interface id missing")?;
        interface_map.insert(id.to_string(), format!("I{}", i + 1));
    }

    let mut field_ordinal = 0usize;
    for owner in classes.iter().chain(interfaces.iter()) {
        for field in owner["fields"].as_array().ok_or("fields must be an array")? {
            let id = field["id"].as_str().ok_or("field id missing")?;
            field_ordinal += 1;
            field_map.insert(id.to_string(), format!("F{field_ordinal}"));
        }
    }

    let mut method_ordinal = 0usize;
    for owner in classes.iter().chain(interfaces.iter()) {
        for method in owner["methods"].as_array().ok_or("methods must be an array")? {
            let id = method["id"].as_str().ok_or("method id missing")?;
            method_ordinal += 1;
            method_map.insert(id.to_string(), format!("M{method_ordinal}"));
        }
    }

    let mut owner_map = class_map.clone();
    owner_map.extend(interface_map.clone());

    let mut result = normalized.clone();

    if let Some(ids) = result
        .pointer_mut("/mode/exact_body_method_ids")
        .and_then(Value::as_array_mut)
    {
        for id in ids.iter_mut() {
            let s = id.as_str().ok_or("exact body id must be a string")?;
            *id = json!(
                method_map
                    .get(s)
                    .ok_or_else(|| format!("body references unknown method {s}"))?
                    .clone()
            );
        }
    }

    for family in ["classes", "interfaces"] {
        let is_class = family == "classes";
        for owner in result[family].as_array_mut().ok_or("owners must be arrays")? {
            let oid = owner["id"].as_str().ok_or("owner id missing")?.to_string();
            owner["id"] = json!(owner_map.get(&oid).ok_or_else(|| format!("owner {oid} not mapped"))?.clone());

            // extends: class = single class id, interface = array of interface ids.
            if is_class {
                if let Some(ext) = owner.get("extends").and_then(Value::as_str) {
                    if let Some(mapped) = owner_map.get(ext) {
                        owner["extends"] = json!(mapped);
                    }
                }
            } else if let Some(exts) = owner.get_mut("extends").and_then(Value::as_array_mut) {
                for ext in exts.iter_mut() {
                    if let Some(s) = ext.as_str() {
                        if let Some(mapped) = interface_map.get(s) {
                            *ext = json!(mapped);
                        }
                    }
                }
            }

            if let Some(impls) = owner.get_mut("implements").and_then(Value::as_array_mut) {
                for imp in impls.iter_mut() {
                    if let Some(s) = imp.as_str() {
                        if let Some(mapped) = interface_map.get(s) {
                            *imp = json!(mapped);
                        }
                    }
                }
            }

            for field in owner["fields"].as_array_mut().ok_or("fields must be arrays")? {
                let fid = field["id"].as_str().ok_or("field id missing")?.to_string();
                field["id"] = json!(field_map.get(&fid).ok_or_else(|| format!("field {fid} not mapped"))?.clone());
            }

            rewrite_patterns(owner, &owner_map, &method_map, &field_map)?;

            for method in owner["methods"].as_array_mut().ok_or("methods must be arrays")? {
                let mid = method["id"].as_str().ok_or("method id missing")?.to_string();
                method["id"] = json!(method_map.get(&mid).ok_or_else(|| format!("method {mid} not mapped"))?.clone());

                if let Some(params) = method.get_mut("parameters").and_then(Value::as_array_mut) {
                    for (k, param) in params.iter_mut().enumerate() {
                        param["id"] = json!(format!("P{}", k + 1));
                    }
                }

                rewrite_patterns(method, &owner_map, &method_map, &field_map)?;

                for flow_family in ["control_flow", "data_flow"] {
                    if let Some(rows) = method.get_mut(flow_family).and_then(Value::as_array_mut) {
                        for row in rows.iter_mut() {
                            if let Some(pair) = row.as_array_mut() {
                                if pair.len() >= 2 {
                                    if let Some(target) = pair[1].as_str() {
                                        let mapped = owner_map
                                            .get(target)
                                            .or_else(|| method_map.get(target))
                                            .or_else(|| field_map.get(target))
                                            .cloned();
                                        if let Some(mapped) = mapped {
                                            pair[1] = json!(mapped);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for call in result["calls"].as_array_mut().ok_or("calls must be an array")? {
        let caller = call["caller_method_id"].as_str().ok_or("caller method id missing")?;
        call["caller_method_id"] = json!(
            method_map
                .get(caller)
                .ok_or_else(|| format!("call references unknown method {caller}"))?
                .clone()
        );
    }

    Ok(result)
}

fn rewrite_patterns(
    value: &mut Value,
    owner_map: &HashMap<String, String>,
    method_map: &HashMap<String, String>,
    field_map: &HashMap<String, String>,
) -> Result<(), String> {
    if let Some(patterns) = value.get_mut("patterns").and_then(Value::as_array_mut) {
        for pattern in patterns.iter_mut() {
            if let Some(args) = pattern.get_mut("args").and_then(Value::as_array_mut) {
                for arg in args.iter_mut() {
                    if let Some(s) = arg.as_str() {
                        if let Some(mapped) = owner_map
                            .get(s)
                            .or_else(|| method_map.get(s))
                            .or_else(|| field_map.get(s))
                        {
                            *arg = json!(mapped);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
