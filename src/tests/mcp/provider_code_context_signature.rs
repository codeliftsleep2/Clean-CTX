// src/tests/mcp/provider_code_context_signature.rs
//
// End-to-end `provide_code_context` regressions for method-declaration
// identity, through the REAL dispatch path (`dispatch_tools_call` ->
// `handle_provide_code_context` -> `compile_file_ir_focused` -> CoreIRPass ->
// canonical focus resolution -> CONTROL-FULL rendering).
//
// Compression-path proof: every case asserts that the response did NOT come
// back as `raw_passthrough`. A raw passthrough returns the file verbatim, so
// assertions like "the text contains GetPair" would pass while proving nothing
// about the compressed representation. The identity assertions below therefore
// use the COMPRESSED SKELETON form (`M <name>`), which the raw source cannot
// contain.
#![cfg(feature = "csharp")]

use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

/// A synthetic C# fixture large enough that compression is clearly favorable:
/// the three declarations under test, the false-`extends` body shape, and
/// filler members whose bodies make the raw file expensive.
fn csharp_fixture(dir: &tempfile::TempDir) -> String {
    let mut src = String::from(
        r#"using System.Linq;
using System.Linq.Expressions;

namespace Ordering;

public static class QueryablePairExtensions
{
"#,
    );
    // Case A — two method type parameters and a nested generic parameter type.
    src.push_str(
        r#"    public static IOrderedQueryable<TFirst> Pair<TFirst, TSecond>(
        this IQueryable<TFirst> source,
        Expression<Func<TFirst, TSecond>> keySelector,
        ListSortDirection direction)
    {
        return source.OrderByDescending(keySelector);
    }

"#,
    );
    // Case B — a named-tuple return type, and a second, unrelated one.
    src.push_str(
        r#"    public static (int alpha, int beta) GetPair(int[] values)
    {
        return (values[0], values[1]);
    }

    public static (string name, int count) Tenth(string[] names)
    {
        return (names[0], names.Length);
    }

"#,
    );
    // The false-`extends` shape: a member body whose ternary `:` used to be
    // read as the class's base-class separator.
    src.push_str(
        r#"    public static int Pick(int[] values, bool ascending)
    {
        return values.Length == 0 ? 0 : values.OrderByDescending(v => v).First();
    }

"#,
    );
    // Filler: enough declaration bytes that the compressed skeleton is
    // unambiguously cheaper than the raw file.
    for i in 0..30 {
        src.push_str(&format!(
            r#"    public static int Filler{i}(int value, string label)
    {{
        var adjusted = value * {i} + label.Length;
        var bounded = adjusted > 100 ? 100 : adjusted;
        var described = label + ":" + bounded;
        System.Console.WriteLine(described);
        return bounded;
    }}

"#
        ));
    }
    src.push_str("}\n");

    let path = dir.path().join("QueryablePairExtensions.cs");
    std::fs::write(&path, &src).expect("fixture must be writable");
    path.to_string_lossy().into_owned()
}

/// Dispatch `provide_code_context` through the real handler and return the
/// response value.
///
/// The file to read travels inside `args` (`filePath`); the workspace root is a
/// separate argument because it configures the state as well.
///
/// Serialized through `protocol::HANDLER_RESPONSE_SERIAL` because handlers
/// write to the process-wide captured-response sink.
fn request_code_context(root: &str, args: serde_json::Value) -> serde_json::Value {
    let _serial = crate::protocol::handler_response_serial();
    let mut config = crate::tests::test_config();
    config.additional_roots.push(root.to_string());
    let state = crate::mcp::McpState::new(config);
    crate::protocol::captured_responses().clear();
    let id = json!(1);
    dispatch_tools_call(
        &id,
        "provide_code_context",
        &json!({ "arguments": args }),
        &state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("handler must send exactly one response")
}

/// The response's self-reported content kind.
fn content_kind(resp: &serde_json::Value) -> &str {
    resp["result"]["_meta"]["content_kind"]
        .as_str()
        .unwrap_or("missing")
}

/// The response's rendered text.
fn rendered(resp: &serde_json::Value) -> String {
    resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

// ══════════════════════════════════════════════════════════════════════
// Fidelity matrix: the structural fidelities always compress (they are not
// gated by token economics), so each one must return the compressed skeleton
// carrying the STRUCTURAL method identities.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn provide_code_context_returns_structural_method_identity_at_every_structural_fidelity() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path().to_string_lossy().into_owned();
    let path = csharp_fixture(&dir);

    for fidelity in ["low", "medium", "high"] {
        let resp = request_code_context(
            &root,
            json!({
                "filePath": path.as_str(),
                "fidelity": fidelity,
                "workspaceRoot": root.as_str()
            }),
        );
        let kind = content_kind(&resp);
        assert_ne!(
            kind, "raw_passthrough",
            "{fidelity}: the regression must exercise the COMPRESSED path, not a verbatim \
             passthrough, got content_kind={kind}"
        );

        let text = rendered(&resp);
        assert!(text.contains("\"name\": \"GetPair\""), "{fidelity}: {text}");
        assert!(text.contains("\"name\": \"Tenth\""), "{fidelity}: {text}");
        assert!(
            !text.contains("\"name\": \"TSecond>\""),
            "{fidelity}: a type parameter must never be the identity: {text}"
        );
        assert!(
            !text.contains("\"name\": \"static\""),
            "{fidelity}: a modifier must never be the identity: {text}"
        );
        assert!(
            !text.contains("\"name\": \"static(+2)\""),
            "{fidelity}: distinct methods must not group as overloads of a fabricated \
             identity: {text}"
        );
        if fidelity == "low" {
            // Low carries the bare identifier (the established Low contract).
            assert!(text.contains("\"name\": \"Pair\""), "{fidelity}: {text}");
        } else {
            assert!(
                text.contains("\"name\": \"Pair<TFirst, TSecond>\""),
                "{fidelity}: {text}"
            );
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// Edit fidelity + `focusMethods`: symbol targeting resolves against the
// corrected identity. With the defect, `GetPair` and `Tenth` were both named
// `static`, so focusing either by its real name silently matched nothing.
// ══════════════════════════════════════════════════════════════════════

#[test]
fn provide_code_context_edit_focus_methods_targets_corrected_identities() {
    let dir = tempfile::TempDir::new().unwrap();
    let root = dir.path().to_string_lossy().into_owned();
    let path = csharp_fixture(&dir);

    let resp = request_code_context(
        &root,
        json!({
            "filePath": path.as_str(),
            "fidelity": "edit",
            "focusMethods": ["GetPair", "Pair<TFirst, TSecond>"],
            "workspaceRoot": root.as_str()
        }),
    );
    let kind = content_kind(&resp);
    assert_ne!(
        kind, "raw_passthrough",
        "the focused Edit case must render from the IR, got content_kind={kind}"
    );

    let text = rendered(&resp);
    assert!(
        text.contains("return (values[0], values[1]);"),
        "focusing `GetPair` must select its verbatim body: {text}"
    );
    assert!(
        text.contains("return source.OrderByDescending(keySelector);"),
        "focusing the generic method must select its verbatim body: {text}"
    );
    assert!(
        !text.contains("return (names[0], names.Length);"),
        "an unfocused method's body must stay signature-only: {text}"
    );
    assert!(
        !text.contains("\"name\": \"static\""),
        "the focused identities must be the declared names: {text}"
    );
}
