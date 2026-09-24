use serde_json::Value;

pub(crate) fn payload(response: &Value) -> Value {
    let text = response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .expect("model-visible text response");
    if text.starts_with("// COMPACT-A A1") || text.starts_with("// COMPACT-A A2") {
        let document = text.split("\n§PATHMAP").next().expect("A1 document");
        let (_, after_header) = document.split_once('\n').expect("A1 header");
        let (_, encoded_and_bodies) = after_header.split_once('\n').expect("A1 legend");
        let (encoded, body_wire) = encoded_and_bodies
            .split_once("\n§BODIES\n")
            .expect("A1 body boundary");
        let mut envelope: Value = serde_json::from_str(encoded).expect("valid compact envelope");
        let is_a2 = envelope["A"] == 2;
        if is_a2 {
            envelope["g"]["E"] = serde_json::json!([]);
            envelope["n"]["E"] = serde_json::json!([]);
        }
        let mut decoded = crate::ir::control_full::compact_a_envelope_tests::decode(
            &envelope,
            body_wire.as_bytes(),
        )
        .expect("decoded normalized CONTROL-FULL");
        if is_a2 {
            decoded.as_object_mut().unwrap().remove("navigation");
        }
        return decoded;
    }
    // After the presentation-boundary fix, model-visible content is the
    // SCHEMA-v5 presentation or raw source — never a structured codec JSON.
    // Nothing below decodes to `interfaces`/`classes`; return the text as a
    // plain string so callers' index access is safe (Null) instead of panicking.
    if !text.starts_with("// CONTROL-FULL") {
        return Value::String(text.to_string());
    }
    let document = text
        .split("\n§PATHMAP")
        .next()
        .expect("CONTROL-FULL document");
    let (_, json) = document.split_once('\n').expect("CONTROL-FULL header");
    serde_json::from_str(json).expect("valid CONTROL-FULL JSON")
}

pub(crate) fn has_interface(response: &Value, expected: &str) -> bool {
    if response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .is_some_and(|text| {
            !text.starts_with("// COMPACT-A")
                && !text.starts_with("// CONTROL-FULL")
                && text.contains(expected)
        })
    {
        return true;
    }
    payload(response)["interfaces"]
        .as_array()
        .is_some_and(|interfaces| interfaces.iter().any(|item| item["name"] == expected))
}

pub(crate) fn has_method_body(response: &Value, method: &str, expected: &str) -> bool {
    if response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .is_some_and(|text| {
            !text.starts_with("// COMPACT-A")
                && !text.starts_with("// CONTROL-FULL")
                && text.contains(method)
                && text.contains(expected)
        })
    {
        return true;
    }
    payload(response)["classes"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|class| class["methods"].as_array().into_iter().flatten())
        .any(|item| item["name"] == method && item["body"] == expected)
}
