use serde_json::Value;

use crate::ir::opcodes::PatternFact;

use super::DecodeError;

/// Revisions 2 through 5 carried pattern facts in `fl`. Move an occurrence
/// only when its complete payload parses as the closed typed vocabulary.
pub(super) fn upgrade_revision_5_pattern_facts(ir: &mut Value) -> Result<(), DecodeError> {
    let Some(classes) = ir.get_mut("c").and_then(Value::as_array_mut) else {
        return Ok(());
    };
    for class in classes {
        let Some(methods) = class.get_mut("m").and_then(Value::as_array_mut) else {
            continue;
        };
        for method in methods {
            let Some(flags) = method.get_mut("fl").and_then(Value::as_array_mut) else {
                continue;
            };
            let mut moved = Vec::new();
            flags.retain(|occurrence| {
                let Some(values) = occurrence.as_array().and_then(|items| {
                    items
                        .iter()
                        .map(|item| item.as_str().map(str::to_owned))
                        .collect::<Option<Vec<_>>>()
                }) else {
                    return true;
                };
                let Some(facts) = PatternFact::parse_all(&values) else {
                    return true;
                };
                moved.push(serde_json::to_value(facts).expect("PatternFact serialization"));
                false
            });
            if !moved.is_empty() {
                method["pf"] = Value::Array(moved);
            }
        }
    }
    Ok(())
}
