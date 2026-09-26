// src/tests/mcp/rxjs_meta_presentation.rs
//
// Production-path regression for intelligible, method-local RxJS annotations.

#![cfg(all(feature = "typescript", feature = "angular"))]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn fixture() -> String {
    let padding = " rxjs-meta-presentation-economics-padding".repeat(300);
    format!(
        r#"import {{ Injectable }} from '@angular/core';
import {{ Observable, of }} from 'rxjs';
import {{ map, filter }} from 'rxjs/operators';

@Injectable()
export class BigProbe {{
  method1(x: number, y: string, z: boolean = false): Observable<number> {{
    return of(x).pipe(map(v => v + 1), filter(v => v > 0));
  }}

  method2(x: number, y: string, z: boolean = false): Observable<number> {{
    return of(x).pipe(map(v => v + 2), filter(v => v > 1));
  }}
}}
/*{padding} */
"#
    )
}

#[test]
fn provide_code_context_keeps_rxjs_annotations_method_local_and_intelligible() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("temp workspace");
    let path = root.path().join("big-probe.ts");
    std::fs::write(&path, fixture()).expect("fixture");
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);

    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": path.to_string_lossy(),
                "workspaceRoot": root.path().to_string_lossy(),
                "intent": "refactor",
                "fidelity": "high"
            }
        }),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response");
    assert!(response.get("error").is_none(), "{response}");
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text");
    assert!(
        text.starts_with("// SCHEMA v5"),
        "fixture must exercise SCHEMA-v5 rather than raw passthrough:\n{text}"
    );

    let pipe_lines: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("T @pipeRx = "))
        .collect();
    assert_eq!(pipe_lines.len(), 2, "pipe annotations:\n{text}");
    assert!(
        pipe_lines[0].contains("method1"),
        "first pipe must identify method1: {pipe_lines:?}"
    );
    assert!(
        pipe_lines[1].contains("method2"),
        "second pipe must identify method2 instead of method1: {pipe_lines:?}"
    );

    let observable_lines: Vec<&str> = text
        .lines()
        .filter(|line| line.starts_with("T @obs = "))
        .collect();
    assert!(
        observable_lines
            .iter()
            .all(|line| !line.ends_with(')') && !line.contains("false)")),
        "parameter fragments must not render as observable annotations: {observable_lines:?}"
    );
}
