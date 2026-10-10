//! RED production-path regressions for Angular template ownership.
//!
//! A component's structured templateUrl is authoritative even when the target
//! does not follow the .component.html naming convention.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn state(root: &tempfile::TempDir) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::mcp::McpState::new(config)
}

fn provide(state: &crate::mcp::McpState, root: &tempfile::TempDir, file: &std::path::Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "workspaceRoot": root.path(),
                "filePath": file,
                "fidelity": "medium"
            }
        }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("provide_code_context response")
}

#[test]
fn decorator_linked_nonstandard_html_uses_angular_template_processing() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let component = root.path().join("detail-page.ts");
    let template = root.path().join("detail-page.html");

    std::fs::write(
        &component,
        r#"
import { Component } from '@angular/core';

@Component({
  selector: 'app-detail-page',
  templateUrl: './detail-page.html'
})
export class DetailPage {}
"#,
    )
    .expect("component fixture");

    let template_source = format!(
        "{}\n<app-detail [item]=\"selected\"></app-detail>\n",
        "<!-- compression padding -->\n".repeat(300)
    );
    std::fs::write(&template, template_source).expect("template fixture");

    let response = provide(&state(&root), &root, &template);

    assert!(
        response.get("error").is_none(),
        "a structured templateUrl must establish Angular ownership: {response}"
    );
    assert_eq!(response["result"]["_meta"]["is_angular"], true, "{response}");
    assert_eq!(
        response["result"]["_meta"]["template_compressed"],
        true,
        "{response}"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible template");
    assert!(text.contains("<app-detail"), "{response}");
    assert!(text.contains("[item]=\"selected\""), "{response}");
}

#[test]
fn unrelated_html_error_explains_that_verbatim_is_raw_uncompressed_content() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().expect("workspace");
    let html = root.path().join("standalone.html");
    std::fs::write(&html, "<main>ordinary HTML</main>").expect("HTML fixture");

    let response = provide(&state(&root), &root, &html);
    let message = response["error"]["message"]
        .as_str()
        .expect("unsupported HTML error");

    assert!(
        message.to_lowercase().contains("verbatim"),
        "error must name the available fallback: {response}"
    );
    assert!(
        message.to_lowercase().contains("raw")
            && message.to_lowercase().contains("uncompressed"),
        "error must explain what verbatim returns: {response}"
    );
}

fn provide_linked_template(source: &str) -> Value {
    let root = tempfile::tempdir().expect("workspace");
    let component = root.path().join("migration-host.ts");
    let template = root.path().join("migration-host.html");

    std::fs::write(
        &component,
        r#"
@Component({
  selector: 'app-migration-host',
  templateUrl: './migration-host.html'
})
export class MigrationHost {}
"#,
    )
    .expect("component fixture");
    std::fs::write(
        &template,
        format!(
            "{}
{source}",
            "<!-- production-path compression padding -->
".repeat(300)
        ),
    )
    .expect("template fixture");

    provide(&state(&root), &root, &template)
}

fn model_visible_text(response: &Value) -> &str {
    assert!(
        response.get("error").is_none(),
        "linked Angular template must use the production template path: {response}"
    );
    assert_eq!(response["result"]["_meta"]["is_angular"], true, "{response}");
    response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible template")
}

fn assert_tokens_in_order(rendered: &str, tokens: &[&str]) {
    let mut previous = 0;
    for token in tokens {
        let relative = rendered[previous..]
            .find(token)
            .unwrap_or_else(|| panic!("missing {token:?} in:
{rendered}"));
        previous += relative + token.len();
    }
}

#[test]
fn linked_legacy_and_modern_templates_expose_equivalent_child_capabilities() {
    let cases = [
        (
            "Angular 15-16 legacy",
            r#"
<ng-container *ngIf="ready">
  <child-row
    *ngFor="let item of items; trackBy: trackItem"
    [item]="item">
  </child-row>
</ng-container>
"#,
            ["@if(ready)", "@for(item of items)"],
        ),
        (
            "Angular 17+ modern",
            r#"
@if (ready) {
  @for (item of items; track item.id) {
    <child-row [item]="item"></child-row>
  }
}
"#,
            ["@if(ready)", "@for(item of items)"],
        ),
    ];

    for (generation, source, behaviors) in cases {
        let response = provide_linked_template(source);
        let rendered = model_visible_text(&response);
        for expected in [behaviors[0], behaviors[1], "<child-row", r#"[item]="item""#] {
            assert!(
                rendered.contains(expected),
                "{generation} lost {expected:?}: {response}"
            );
        }
    }
}

#[test]
fn linked_mixed_migration_template_preserves_cross_syntax_source_order() {
    let response = provide_linked_template(
        r#"
<section *ngIf="ready">
  @if (selected) {
    <child-detail [item]="selected"></child-detail>
  }
</section>
@for (item of items; track item.id) {
  <child-row *ngIf="item.visible" [item]="item"></child-row>
}
"#,
    );
    let rendered = model_visible_text(&response);

    assert_tokens_in_order(
        rendered,
        &[
            "@if(ready)",
            "@if(selected)",
            "<child-detail",
            "@for(item of items)",
            "@if(item.visible)",
            "<child-row",
        ],
    );
}