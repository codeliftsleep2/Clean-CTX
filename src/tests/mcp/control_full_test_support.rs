use serde_json::Value;

pub(crate) fn payload(response: &Value) -> Value {
    let text = response
        .pointer("/result/content/0/text")
        .and_then(Value::as_str)
        .expect("CONTROL-FULL text response");
    let document = text
        .split("\n§PATHMAP")
        .next()
        .expect("CONTROL-FULL document");
    let (_, json) = document.split_once('\n').expect("CONTROL-FULL header");
    serde_json::from_str(json).expect("valid CONTROL-FULL JSON")
}

pub(crate) fn payload_without_ir_version(response: &Value) -> Value {
    let mut value = payload(response);
    value["file"]["ir_version"] = Value::Null;
    value
}

pub(crate) fn has_interface(response: &Value, expected: &str) -> bool {
    payload(response)["interfaces"]
        .as_array()
        .is_some_and(|interfaces| interfaces.iter().any(|item| item["name"] == expected))
}

pub(crate) fn has_method_body(response: &Value, method: &str, expected: &str) -> bool {
    payload(response)["classes"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|class| class["methods"].as_array().into_iter().flatten())
        .any(|item| item["name"] == method && item["body"] == expected)
}
