//! Structural TypeScript export-wrapper ownership through production capture.

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::opcodes::{CoreOp, DeclarationModifier};
use crate::ir::patterns::CompressingPatternRecognizer;

fn compile(source: &str) -> Vec<CoreOp> {
    let language = crate::compression::language::safe_typescript_language()
        .expect("typescript grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "exports.ts",
            language,
            crate::queries::TS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production TypeScript compilation")
        .instructions
}

fn exported_class(ops: &[CoreOp], name: &str) -> bool {
    let Some(id) = ops.iter().find_map(|op| match op {
        CoreOp::DefClass(id, found) if found == name => Some(id),
        _ => None,
    }) else {
        return false;
    };
    ops.iter().any(|op| {
        matches!(op, CoreOp::ClassModifiers(owner, modifiers)
            if owner == id && modifiers.contains(&DeclarationModifier::Export))
    })
}

fn exported_interface(ops: &[CoreOp], name: &str) -> bool {
    let Some(id) = ops.iter().find_map(|op| match op {
        CoreOp::DefInterface(id, found) if found == name => Some(id),
        _ => None,
    }) else {
        return false;
    };
    ops.iter().any(|op| {
        matches!(op, CoreOp::InterfaceModifiers(owner, modifiers)
            if owner == id && modifiers.contains(&DeclarationModifier::Export))
    })
}

#[test]
fn export_wrappers_attach_to_their_exact_class_and_interface_declarations() {
    let ops = compile(
        "export class Named {}\n\
         export default class Defaulted {}\n\
         export interface PublicApi { run(): void; }\n\
         class PrivateClass { text(): string { return 'export default class Fake {}'; } }\n\
         interface PrivateApi { stop(): void; }\n\
         const exportNoise = 'export class AlsoFake {}';",
    );

    assert!(exported_class(&ops, "Named"));
    assert!(exported_class(&ops, "Defaulted"));
    assert!(exported_interface(&ops, "PublicApi"));
    assert!(!exported_class(&ops, "PrivateClass"));
    assert!(!exported_interface(&ops, "PrivateApi"));
}
