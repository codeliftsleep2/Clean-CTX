use serde_json::Value;

pub(crate) fn payload(response: &Value) -> Value {
    let text = response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .expect("model-visible text response");
    if text.starts_with("// COMPACT-A A1") {
        let document = text.split("\n§PATHMAP").next().expect("A1 document");
        let (_, after_header) = document.split_once('\n').expect("A1 header");
        let (_, encoded_and_bodies) = after_header.split_once('\n').expect("A1 legend");
        let (encoded, body_wire) = encoded_and_bodies
            .split_once("\n§BODIES\n")
            .expect("A1 body boundary");
        let envelope = serde_json::from_str(encoded).expect("valid A1 envelope");
        return crate::ir::control_full::compact_a_envelope_tests::decode(
            &envelope,
            body_wire.as_bytes(),
        )
        .expect("decoded normalized CONTROL-FULL");
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
