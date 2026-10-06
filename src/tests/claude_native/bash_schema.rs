use super::*;
use serde_json::{Value, json};

fn event() -> Value {
    json!({
        "hook_event_name": "PostToolUse", "tool_name": "Bash",
        "tool_input": {"command": "git diff", "description": "private input"},
        "tool_response": {"stdout": "out", "stderr": "err", "interrupted": false,
            "isImage": false, "future": {"array": [3, 2, 2, 1]}}
    })
}

#[test]
fn admits_documented_shape_and_preserves_unknown_members() {
    let value = event();
    let admitted = recognize(&value).unwrap();
    let mut rebuilt = admitted.response.clone();
    rebuilt.insert("stdout".into(), Value::String("changed".into()));
    assert!(validate_reconstruction(admitted.response, &rebuilt));
    assert_eq!(rebuilt["future"], json!({"array": [3, 2, 2, 1]}));
}

#[test]
fn rejects_every_unsupported_admission_boundary() {
    for mutation in [
        "missing_stdout",
        "wrong_stderr",
        "interrupted",
        "image",
        "unknown_tool",
        "failure",
    ] {
        let mut value = event();
        match mutation {
            "missing_stdout" => {
                value["tool_response"]
                    .as_object_mut()
                    .unwrap()
                    .remove("stdout");
            }
            "wrong_stderr" => value["tool_response"]["stderr"] = json!(7),
            "interrupted" => value["tool_response"]["interrupted"] = json!(true),
            "image" => value["tool_response"]["isImage"] = json!(true),
            "unknown_tool" => value["tool_name"] = json!("Read"),
            "failure" => value["hook_event_name"] = json!("PostToolUseFailure"),
            _ => unreachable!(),
        }
        assert!(recognize(&value).is_none(), "{mutation} must be rejected");
    }
    let mut value = event();
    value["tool_response"] = json!("opaque");
    assert!(recognize(&value).is_none());
}
