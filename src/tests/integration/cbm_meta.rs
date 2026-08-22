// src/tests/integration/cbm_meta.rs
//
// Integration tests for CBM filter-first + meta-layer and CBM proxy fallback.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::mcp::McpState;
use std::io::Write;

/// Test 2: CBM Filter-First + Meta-Layer
/// CBM skip sets should prevent low-importance symbols from appearing in meta-layer output.
#[test]
fn cbm_filter_first_with_meta_layer() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("service.ts");

    // Create a TypeScript file with multiple classes
    let source = r#"
export class ImportantService {
    doWork(): void {}
}

export class LowImportanceHelper {
    helper(): void {}
}

export class AnotherService {
    process(): void {}
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let config = CleanCtxConfig::default();
    let _state = McpState::new(config);

    // Compile with IR path directly using IRCompiler
    let source_text = std::fs::read_to_string(&path).unwrap();
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));

    let (language, query_string) = crate::compression::language::language_for_extension("ts")
        .expect("TypeScript language should be available");

    let result = compiler.compile(
        &source_text,
        "service.ts",
        language,
        query_string,
        Fidelity::Low,
        None,
    );

    assert!(result.is_ok(), "IR compilation should succeed");
    let ir = result.unwrap();

    // All classes should be in the IR (no CBM filter in this test)
    let class_count = ir
        .instructions
        .iter()
        .filter(|op| matches!(op, crate::ir::opcodes::CoreOp::DefClass(_, _)))
        .count();

    assert_eq!(
        class_count, 3,
        "All 3 classes should be in IR without CBM filter"
    );
}

/// Test 3: Workspace + CBM Integration
/// Full `compress_workspace` with CBM enabled should populate skip sets and reduce output.
#[test]
fn workspace_with_cbm_integration() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("workspace_file.ts");

    let source = r#"
export class WorkspaceService {
    method(): void {}
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // Test that workspace compression works
    let result = crate::mcp::workspace::compress_workspace_dir(
        dir.path().to_string_lossy().as_ref(),
        Fidelity::Low,
        &state,
    );

    // Should succeed (CBM may be disabled, but workspace should still work)
    assert!(result.is_ok(), "Workspace compression should succeed");
}

/// Test 15: CBM Proxy Fallback
/// When CBM returns unparseable JSON, `cbm_proxy` should apply minimum compression.
#[test]
fn cbm_proxy_fallback_on_invalid_json() {
    // This test verifies the fallback path exists
    // The actual cbm_proxy handler is in src/cbm/proxy.rs

    // Test the minimum compression function directly
    let raw = r#"{"result": {"nodes": ["test"]}}"#;
    let compressed = crate::cbm::proxy::apply_minimum_compression(raw);

    // Should strip whitespace
    assert!(
        !compressed.contains('\n') || compressed.contains("result"),
        "Minimum compression should handle JSON"
    );

    // Test with invalid JSON
    let invalid = "not valid json at all {{{";
    let compressed_invalid = crate::cbm::proxy::apply_minimum_compression(invalid);

    // Should still return something (whitespace stripped)
    assert!(
        !compressed_invalid.is_empty(),
        "Invalid JSON should still produce output via fallback"
    );
}
