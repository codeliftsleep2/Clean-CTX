// src/tests/ir/regression_canonical_identity.rs
//
// RED regression suite proving canonical identity defects at the compiler pipeline boundary:
// 1. DefField carries presentation syntax / modifiers / types instead of bare identifier name.
// 2. DefField name is destroyed (empty string) at Low fidelity.
// 3. DefClass name carries extends/implements clause instead of bare class name.
// 4. Import module string carries trailing quote/semicolon syntax leaks.
// 5. TypeAlias original type carries "type X = " syntax leaks.

use crate::compression::Fidelity;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::opcodes::CoreOp;
use crate::ir::patterns::CompressingPatternRecognizer;

#[cfg(feature = "typescript")]
fn compile_ts_pipeline(source: &str, fidelity: Fidelity) -> CompiledIR {
    use crate::ir::layers::typescript::TypeScriptLayer;
    let language =
        crate::compression::language::safe_typescript_language().expect("typescript grammar");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Identity.ts",
            language,
            crate::queries::TS_QUERY,
            fidelity,
            None,
        )
        .expect("compilation must succeed")
}

#[cfg(feature = "csharp")]
fn compile_cs_pipeline(source: &str, fidelity: Fidelity) -> CompiledIR {
    use crate::ir::layers::csharp::CSharpLayer;
    let language = crate::compression::language::safe_csharp_language().expect("csharp grammar");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Identity.cs",
            language,
            crate::queries::CS_QUERY,
            fidelity,
            None,
        )
        .expect("compilation must succeed")
}

// ── Defect 1: Field name carries presentation/modifiers/type instead of bare identifier ──

#[cfg(feature = "typescript")]
#[test]
fn red_field_name_ts_is_bare_identifier_not_presentation() {
    const SRC: &str = "export class Alpha {\n  private label: string = 'test';\n}";
    let ir = compile_ts_pipeline(SRC, Fidelity::High);

    let field_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefField(_cid, fid, name) => Some((fid.clone(), name.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(field_ops.len(), 1, "must define exactly one field");
    assert_eq!(
        field_ops[0].1, "label",
        "Field name must be bare identifier 'label', not presentation string with modifiers/type"
    );
}

#[cfg(feature = "typescript")]
#[test]
fn red_field_name_ts_is_fidelity_invariant_and_not_empty_at_low() {
    const SRC: &str = "export class Alpha {\n  private label: string = 'test';\n}";
    let ir_low = compile_ts_pipeline(SRC, Fidelity::Low);
    let ir_med = compile_ts_pipeline(SRC, Fidelity::Medium);
    let ir_high = compile_ts_pipeline(SRC, Fidelity::High);

    let get_field_name = |ir: &CompiledIR| {
        ir.instructions
            .iter()
            .find_map(|op| match op {
                CoreOp::DefField(_cid, _fid, name) => Some(name.clone()),
                _ => None,
            })
            .expect("field must exist")
    };

    let name_low = get_field_name(&ir_low);
    let name_med = get_field_name(&ir_med);
    let name_high = get_field_name(&ir_high);

    assert_ne!(
        name_low, "",
        "Field name must not be empty at Low fidelity"
    );
    assert_eq!(
        name_low, "label",
        "Field name at Low fidelity must be 'label'"
    );
    assert_eq!(
        name_med, "label",
        "Field name at Medium fidelity must be 'label'"
    );
    assert_eq!(
        name_high, "label",
        "Field name at High fidelity must be 'label'"
    );
}

#[cfg(feature = "csharp")]
#[test]
fn red_field_name_cs_is_bare_identifier_not_presentation() {
    const SRC: &str = "public class Worker {\n  private readonly IRepository repository;\n}";
    let ir = compile_cs_pipeline(SRC, Fidelity::High);

    let field_ops: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefField(_cid, fid, name) => Some((fid.clone(), name.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(field_ops.len(), 1, "must define exactly one field");
    assert_eq!(
        field_ops[0].1, "repository",
        "C# field name must be bare identifier 'repository', not 'private readonly IRepository repository'"
    );
}

// ── Defect 2: Class name contains extends/implements clause ──

#[cfg(feature = "typescript")]
#[test]
fn red_class_name_ts_is_bare_identifier_not_extends_clause() {
    const SRC: &str = "export class InheritanceProbe extends BaseService implements Runner {\n  run(): void {}\n}";
    let ir = compile_ts_pipeline(SRC, Fidelity::High);

    let class_names: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefClass(_cid, name) => Some(name.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(class_names.len(), 1, "must define one class");
    assert_eq!(
        class_names[0], "InheritanceProbe",
        "Class name must be 'InheritanceProbe', not 'InheritanceProbe:BaseService,Runner'"
    );
}

// ── Defect 3: Import module has trailing quote/semicolon syntax leaks ──

#[cfg(feature = "typescript")]
#[test]
fn red_import_module_has_no_trailing_syntax_characters() {
    const SRC: &str = "import { Injectable } from \"@angular/core\";\nexport class Svc {}";
    let ir = compile_ts_pipeline(SRC, Fidelity::High);

    let import_modules: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Import(_alias, module, _named) => Some(module.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(import_modules.len(), 1, "must have one import");
    assert_eq!(
        import_modules[0], "@angular/core",
        "Import module must be '@angular/core', without trailing quote or semicolon"
    );
}

// ── Defect 4: Type alias original type retains syntax wrapper ──

#[cfg(feature = "typescript")]
#[test]
fn red_type_alias_original_type_has_no_declaration_wrapper() {
    const SRC: &str = "export type Identifier = string;\nexport class Svc {}";
    let ir = compile_ts_pipeline(SRC, Fidelity::High);

    let type_aliases: Vec<_> = ir
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::TypeAlias(_alias, original_type) => Some(original_type.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(type_aliases.len(), 1, "must have one type alias");
    assert_eq!(
        type_aliases[0], "string",
        "Type alias target must be 'string', not 'type Identifier = string;'"
    );
}

