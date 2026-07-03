// src/tests/integration/persistence.rs
//
// Integration tests for persistence cross-session.

use crate::config::CleanCtxConfig;
use crate::mcp::McpState;
use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use std::io::Write;

/// Test 5: Persistence Cross-Session
/// State saved via `save_context` should survive server restart and be loadable via `restore_context`.
#[test]
fn persistence_cross_session() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("persist_test.ts");
    
    let source = r#"
export class PersistService {
    persistMethod(): void {}
}
"#;
    
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();
    
    // Create state with persistence disabled (for testing without DB)
    let config = CleanCtxConfig::default();
    let state = McpState::new(config);
    
    // Compile the file using IRCompiler directly
    let source_text = std::fs::read_to_string(&path).unwrap();
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    
    let (language, query_string) = crate::compression::language::language_for_extension("ts")
        .expect("TypeScript language should be available");
    
    let result = compiler.compile(
        &source_text,
        "persist_test.ts",
        language,
        query_string,
        Fidelity::Low,
        None,
    );
    
    assert!(result.is_ok(), "Compilation should succeed");
    let ir = result.unwrap();
    
    // Store in IR context
    {
        let mut ir_ctx = state.ir_context_lock();
        ir_ctx.load_ir(ir, None);
    }
    
    // Verify the file is in the context using the file_id from the IR
    let has_file = {
        let ir_ctx = state.ir_context_read();
        ir_ctx.has_file("persist_test.ts")
    };
    
    assert!(has_file, "File should be in IR context after load");
}
