// Zero-model-call diagnostic: tokenizes the Schema-v5 text pipeline at each
// pass boundary so the marginal cost of each compression primitive is visible.
// Run with `cargo test --all-features print_pass_token_anatomy -- --nocapture`.
use crate::compaction::java::{
    compact_java_package, extract_java_constructor_sig, extract_java_type_name,
};
use crate::compaction::{
    compact_expression, compact_import, extract_class_name, extract_field, extract_method_sig,
    extract_rust_struct_name,
};
use crate::compression::capture_pipeline::run_capture_pipeline;
use crate::compression::language::language_for_extension;
use crate::compression::micro_opcodes::apply_micro_opcodes;
use crate::compression::output::{assemble_body, build_output_lines};
use crate::compression::symbol_compression::apply_symbol_compression;
use crate::compression::type_aliases::apply_type_aliases;
use crate::compression::Fidelity;
use crate::tokenizer::{create_tokenizer, TokenizerKind};
use std::collections::BTreeMap;

#[test]
fn print_pass_token_anatomy() {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/test_files/LargeService.ts"
    ))
    .expect("fixture read");

    let tokenizer = create_tokenizer(TokenizerKind::O200k).expect("o200k tokenizer");
    let count = |s: &str| tokenizer.count_tokens(s);

    println!("== Schema-v5 text pipeline anatomy: LargeService.ts (o200k) ==");
    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        let (language, query) = language_for_extension("ts").expect("ts supported");
        let captures =
            run_capture_pipeline(language, query, &source, fidelity, |name, raw, f| match name {
                "class.root" => Some(extract_class_name(raw)),
                "struct.root" | "trait.root" | "impl.root" => Some(extract_rust_struct_name(raw)),
                "interface.root" | "record.root" => Some(extract_java_type_name(raw, name)),
                "method.root" => Some(extract_method_sig(raw, f)),
                "constructor.root" => Some(extract_java_constructor_sig(raw, f)),
                "field.root" => Some(extract_field(raw, f)),
                "mod.root" => Some(compact_import(raw, f)),
                "package.root" => Some(compact_java_package(raw, f)),
                "type.root" => Some(compact_expression(raw, f)),
                _ => Some(compact_expression(raw, f)),
            })
            .expect("capture pipeline");

        let built = build_output_lines(&captures, &source, fidelity, None, None);
        let body = assemble_body(&built.output_lines, fidelity);

        // Type aliases are config-driven; an empty table exposes the default delta.
        let (ta_body, _ta_footer) = apply_type_aliases(&body, &BTreeMap::new());
        let micro = apply_micro_opcodes(&ta_body, fidelity);
        let (symbol, _sym_footer) = apply_symbol_compression(&micro, fidelity);

        println!(
            "{:?}: raw={} capture+markers={} (+type_aliases={}) (+micro_opcodes={}) (+symbol_dict={})",
            fidelity,
            count(&source),
            count(&body),
            count(&ta_body),
            count(&micro),
            count(&symbol),
        );
    }
}
