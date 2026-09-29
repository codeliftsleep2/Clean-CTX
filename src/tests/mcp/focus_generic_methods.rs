//! RED production-dispatch regressions for generic `focusMethods` selectors.
//!
//! The public selector grammar is a bare method name or `Owner.method`.
//! Generic type parameters belong to declaration/signature presentation, not
//! to the selector spelling required from a caller.

use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn request(root: &str, path: &str, selector: &str) -> Value {
    let _serial = crate::protocol::handler_response_serial();
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root.to_string());
    let state = crate::mcp::McpState::new(config);
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": path,
                "fidelity": "edit",
                "focusMethods": [selector],
                "workspaceRoot": root,
            }
        }),
        &state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("provide_code_context response")
}

fn assert_focused(response: &Value, body_evidence: &str, selector: &str) {
    assert!(
        response.get("error").is_none(),
        "documented selector {selector:?} must resolve: {response}"
    );
    assert_eq!(
        response["result"]["_meta"]["content_kind"], "skeleton_with_focused_verbatim_bodies",
        "selector {selector:?} must use focused Edit content"
    );
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("focused model content");
    assert!(
        text.contains(body_evidence),
        "selector {selector:?} omitted the generic method body: {text}"
    );
}

fn assert_selectors_focus(root: &str, path: &str, selectors: &[&str], body_evidence: &str) {
    let responses = selectors
        .iter()
        .map(|selector| (*selector, request(root, path, selector)))
        .collect::<Vec<_>>();
    let rejected = responses
        .iter()
        .filter_map(|(selector, response)| {
            response
                .get("error")
                .map(|error| format!("{selector}: {error}"))
        })
        .collect::<Vec<_>>();
    assert!(
        rejected.is_empty(),
        "every documented selector form must resolve; rejected: {}",
        rejected.join(" | ")
    );
    for (selector, response) in responses {
        assert_focused(&response, body_evidence, selector);
    }
}

#[cfg(feature = "csharp")]
#[test]
fn red_csharp_generic_method_accepts_plain_bare_and_owner_qualified_selectors() {
    let dir = tempfile::tempdir().expect("workspace");
    let root = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("QueryableExtensions.cs");
    let mut source = String::from(
        r#"using System;
using System.Linq;
using System.Linq.Expressions;

public static class QueryableExtensions
{
    public static IOrderedQueryable<TSource> SortBy<TSource, TKey>(
        this IQueryable<TSource> source,
        Expression<Func<TSource, TKey>> keySelector,
        bool descending)
    {
        return descending
            ? source.OrderByDescending(keySelector)
            : source.OrderBy(keySelector);
    }

"#,
    );
    for index in 0..24 {
        source.push_str(&format!(
            "    public static int Filler{index}(int value)\n    {{\n        return value + {index};\n    }}\n\n"
        ));
    }
    source.push_str("}\n");
    std::fs::write(&path, source).expect("C# fixture");
    let path = path.to_string_lossy().into_owned();

    assert_selectors_focus(
        &root,
        &path,
        &["SortBy", "QueryableExtensions.SortBy"],
        "source.OrderByDescending(keySelector)",
    );
}

#[cfg(feature = "typescript")]
#[test]
fn red_typescript_generic_method_accepts_plain_bare_and_owner_qualified_selectors() {
    let dir = tempfile::tempdir().expect("workspace");
    let root = dir.path().to_string_lossy().into_owned();
    let path = dir.path().join("RequestClient.ts");
    let mut source = String::from(
        r#"export class RequestClient {
  protected getRequest<T>(data: Partial<T>): Promise<T> {
    return Promise.resolve(data as T);
  }

"#,
    );
    for index in 0..24 {
        source.push_str(&format!(
            "  public filler{index}(value: number): number {{\n    return value + {index};\n  }}\n\n"
        ));
    }
    source.push_str("}\n");
    std::fs::write(&path, source).expect("TypeScript fixture");
    let path = path.to_string_lossy().into_owned();

    assert_selectors_focus(
        &root,
        &path,
        &["getRequest", "RequestClient.getRequest"],
        "return Promise.resolve(data as T);",
    );
}
