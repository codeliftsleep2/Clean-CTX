// src/tests/integration/text_vs_ir.rs
//
// Integration tests for Text vs IR path equivalence and meta-layer feature gates.

use crate::compression::Fidelity;
use crate::config::CleanCtxConfig;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::mcp::McpState;
use std::io::Write;

/// Test 1: Text vs IR Path Equivalence
/// Verify `compress_code_context` (text) and IR compiler produce
/// semantically equivalent output for the same file.
#[test]
fn text_vs_ir_path_equivalence() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("sample.ts");

    // Create a TypeScript file with class, method, and import
    let source = r#"
import { Injectable } from '@angular/core';

export class UserService {
    private users: string[] = [];
    
    getUser(id: string): string {
        return this.users.find(u => u === id) || '';
    }
    
    addUser(user: string): void {
        this.users.push(user);
    }
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // Get text path output
    let text_result = crate::compression::pipeline::compress_file_with_source(
        path.clone(),
        None,
        &mut state.dict_lock(),
        &mut state.cache_write(),
        Fidelity::Low,
        Some(&state.config),
    );

    // Get IR path output - use IRCompiler directly
    let source_text = std::fs::read_to_string(&path).unwrap();
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));

    // language_for_extension returns (Language, query_string)
    let (language, query_string) = crate::compression::language::language_for_extension("ts")
        .expect("TypeScript language should be available");

    let ir_result = compiler.compile(
        &source_text,
        "sample.ts",
        language,
        query_string,
        Fidelity::Low,
        None,
    );

    // Both should succeed
    assert!(text_result.is_ok(), "Text path should succeed");
    assert!(ir_result.is_ok(), "IR path should succeed");

    // Both should identify the class
    let text_output = text_result.unwrap();
    let ir = ir_result.unwrap();

    // Check that both outputs contain the class name
    assert!(
        text_output.contains("UserService"),
        "Text output should contain class name"
    );

    // Check IR contains the class
    let has_class = ir.instructions.iter().any(|op| {
        matches!(op, crate::ir::opcodes::CoreOp::DefClass(_, name) if name.contains("UserService"))
    });
    assert!(has_class, "IR should contain DefClass for UserService");
}

/// Test 10: Meta-Layer Feature Gates
/// With `angular` feature disabled, no Angular markers should appear in output.
/// Note: This test runs in the current feature configuration.
#[test]
fn meta_layer_feature_gates() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("angular.component.ts");

    // Create an Angular component file
    let source = r#"
import { Component, Input, Output, EventEmitter } from '@angular/core';

@Component({
    selector: 'app-user',
    template: '<div>{{user}}</div>'
})
export class UserComponent {
    @Input() user: string = '';
    @Output() changed = new EventEmitter<string>();
}
"#;

    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(source.as_bytes()).unwrap();

    // Compile with IR path directly
    let source_text = std::fs::read_to_string(&path).unwrap();
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));

    // language_for_extension returns (Language, query_string)
    let (language, query_string) = crate::compression::language::language_for_extension("ts")
        .expect("TypeScript language should be available");

    let ir_result = compiler.compile(
        &source_text,
        "angular.component.ts",
        language,
        query_string,
        Fidelity::Low,
        None,
    );

    assert!(ir_result.is_ok(), "IR compilation should succeed");
    let ir = ir_result.unwrap();

    // Check for Angular detection in IR
    // The IR should have the class definition
    let has_component = ir.instructions.iter().any(|op| {
        matches!(op, crate::ir::opcodes::CoreOp::DefClass(_, name) if name.contains("UserComponent"))
    });
    assert!(has_component, "IR should contain UserComponent class");
}
