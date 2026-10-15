use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};

fn provide(source: &str, fidelity: &str) -> Value {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("Shape.cs");
    let padded = format!(
        "{}{}",
        "// fidelity-corpus-padding-0123456789abcdef\n".repeat(80),
        source
    );
    std::fs::write(&path, padded).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(135),
        "provide_code_context",
        &json!({"arguments": {
            "workspaceRoot": root.path(), "filePath": path, "fidelity": fidelity
        }}),
        &crate::mcp::McpState::new(config),
    );
    crate::protocol::captured_responses().pop().unwrap()
}

const CORPUS: &str = r#"/// <summary>Shape corpus</summary>
[Audit]
public partial record Shape<T>(T Value = default!) where T : class, new()
{
    [Authorize(Roles = "Admin")]
    public required T Field;
    public T Property { get; init; } = Value;
    public Shape(T value, int count = 2) { Value = value; }
    [Route("items/{id}")]
    public TResult Map<TResult>(T input = default!) where TResult : T => default!;
    public interface INested { T Read(T fallback = default!); }
}
public partial record Shape<T> { public T Other => Value; }
"#;

fn omissions(response: &Value) -> Vec<&str> {
    response["result"]["_meta"]["degradation"]["omits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect()
}

#[test]
fn csharp_structural_fidelities_declare_bounded_semantic_loss() {
    for fidelity in ["low", "medium", "high", "edit"] {
        let response = provide(CORPUS, fidelity);
        assert!(response.get("error").is_none(), "{response}");
        let degradation = &response["result"]["_meta"]["degradation"];
        assert_eq!(degradation["language"], "csharp", "{response}");
        assert_eq!(degradation["semantic_completeness"], "declared_loss");
        let omitted = omissions(&response);
        for category in [
            "declaration_attributes",
            "xml_documentation",
            "record_positional_parameters",
            "partial_type_composition",
        ] {
            assert!(omitted.contains(&category), "{fidelity}: {response}");
        }
    }
}

#[test]
fn csharp_fidelity_specific_loss_and_exact_verbatim_are_truthful() {
    let low = provide(CORPUS, "low");
    let medium = provide(CORPUS, "medium");
    let high = provide(CORPUS, "high");
    assert!(omissions(&low).contains(&"parameter_types"));
    assert!(omissions(&medium).contains(&"method_return_types"));
    assert!(!omissions(&high).contains(&"method_return_types"));
    let verbatim = provide(CORPUS, "verbatim");
    assert!(verbatim["result"]["_meta"]["degradation"].is_null());
    assert_eq!(
        verbatim["result"]["_meta"]["byte_exact"],
        json!(["document"])
    );
}

#[test]
fn non_csharp_skeleton_does_not_inherit_csharp_degradation() {
    let value = crate::mcp::tool_handlers::core::common::model_facing_degradation(
        "shape.ts",
        crate::compression::Fidelity::Medium,
        crate::mcp::tool_handlers::core::ContentKind::Skeleton,
    );
    assert!(value.is_null());
}
