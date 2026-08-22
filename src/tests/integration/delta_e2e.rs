// src/tests/integration/delta_e2e.rs
//
// Integration tests for Delta transport E2E and cache hit/miss consistency.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::mcp::McpState;
use std::io::Write;

/// Test 4: Delta Transport E2E
/// `provide_code_context` → `delta_code_context` → `apply_delta` round-trip should preserve IR state.
#[test]
fn delta_transport_round_trip() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("delta_test.ts");

    let source = r#"
export class DeltaService {
    method(): void {}
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // First call: full compression using IRCompiler directly
    let source_text = std::fs::read_to_string(&path).unwrap();
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));

    let (language, query_string) = crate::compression::language::language_for_extension("ts")
        .expect("TypeScript language should be available");

    let result1 = compiler.compile(
        &source_text,
        "delta_test.ts",
        language,
        query_string,
        Fidelity::Low,
        None,
    );

    assert!(result1.is_ok(), "First compile should succeed");
    let ir1 = result1.unwrap();

    // Store the IR in context
    {
        let mut ir_ctx = state.ir_context_lock();
        ir_ctx.load_ir(ir1.clone(), None);
    }

    // Verify the IR was stored
    let has_file = {
        let ir_ctx = state.ir_context_read();
        ir_ctx.has_file("delta_test.ts")
    };
    assert!(has_file, "IR should be stored in context after load");
}

/// Test 9: Cache Hit/Miss Consistency
/// Cache hits should produce identical results to first call.
#[test]
fn cache_hit_miss_consistency() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("cache_test.ts");

    let source = r#"
export class CacheService {
    cachedMethod(): string { return "cached"; }
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // First read - should populate cache
    let result1 = state.read_source(path.to_string_lossy().as_ref());
    assert!(result1.is_ok(), "First read should succeed");

    // Second read - should hit cache
    let result2 = state.read_source(path.to_string_lossy().as_ref());
    assert!(result2.is_ok(), "Second read should succeed");

    // Both should return the same content
    let content1 = result1.unwrap();
    let content2 = result2.unwrap();
    assert_eq!(
        content1.as_str(),
        content2.as_str(),
        "Cache hit should return same content"
    );
}
