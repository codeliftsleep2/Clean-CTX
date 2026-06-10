// examples/token_savings_demo.rs
//
// F-FINAL-08 (this PR): end-to-end demo of the token savings
// produced by the new `clean-ctx` compression pipeline. Run with:
//
//     cargo run --example token_savings_demo
//
// Phase I (Ultra-Compact IR): now also compares the new string_table
// wire format and compact delta format alongside the existing named IR.
//
// The example takes the repo's own test files and compresses them:
//   1. Text compression (`compress_file`) — the legacy pipeline
//   2. Compiler IR (`compile_file_ir` → `ir_to_wire`) — named format
//   3. Compiler IR (string_table format) — Phase I compact format
//
// Also demonstrates delta savings by simulating an edit.

use clean_ctx::analytics::bpe_or_init;
use clean_ctx::compression::Fidelity;
use clean_ctx::ir::compiler::IRCompiler;
use clean_ctx::ir::layers::typescript::TypeScriptLayer;
use clean_ctx::ir::layers::angular::AngularMetaLayer;
use clean_ctx::ir::layers::patterns::CodePatternRecognizer;
use clean_ctx::ir::patterns::CompressingPatternRecognizer;
use clean_ctx::ir::wire::ir_to_wire;
use clean_ctx::ir::string_table::ir_to_string_table_wire;
use clean_ctx::ir::string_table::estimate_savings;
use clean_ctx::ir::delta::{DeltaComputer, compact_encode};
use clean_ctx::compression::language::language_for_extension;
use std::path::PathBuf;

/// The repo's own test corpus — TS files only.
fn sample_files() -> Vec<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    vec![
        manifest_dir.join("src/test_files/angular/user.service.ts"),
        manifest_dir.join("src/test_files/angular/user-card.component.ts"),
        manifest_dir.join("src/test_files/angular/user-page.component.ts"),
        manifest_dir.join("src/test_files/angular/non_triplet_file.ts"),
        manifest_dir.join("src/test_files/LargeService.ts"),
        manifest_dir.join("src/test_files/sample_service.ts"),
    ]
}

/// Compile a file to IR and return the CompiledIR.
fn compile_file_ir(file: &PathBuf, fidelity: Fidelity) -> Result<clean_ctx::ir::compiler::CompiledIR, Box<dyn std::error::Error>> {
    let source = std::fs::read_to_string(file)?;
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (language, query_string) = language_for_extension(ext)
        .ok_or_else(|| format!("Unsupported extension: .{}", ext))?;

    let file_id = file.file_stem().unwrap().to_string_lossy().to_string();

    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(TypeScriptLayer::new()));
    compiler.add_meta_layer(Box::new(AngularMetaLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));

    let ir = compiler.compile(&source, &file_id, language, query_string, fidelity)?;
    Ok(ir)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bpe_or_init()?;
    let bpe = clean_ctx::analytics::bpe();

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Clean-CTX Token Savings Demo (Phase I: Ultra-Compact IR)");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("  Format 1 (named):   JSON arrays with opcode strings       ");
    println!("  Format 2 (table):   String table + integer index arrays   ");
    println!("  Format 3 (delta):   Compact field-patch delta encoding    ");
    println!();

    let files = sample_files();

    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        println!("── Fidelity: {:?} ──", fidelity);
        println!(
            "  {:<40} {:>6} {:>6} {:>6} {:>6}   │ {:>16}",
            "file", "raw", "named", "table", "sav%", "delta_c/named_c"
        );
        println!("  {}", "─".repeat(110));

        let mut total_raw = 0usize;
        let mut total_named = 0usize;
        let mut total_table = 0usize;

        for file in &files {
            let source = std::fs::read_to_string(file)?;
            let raw_tokens = bpe.encode_with_special_tokens(&source).len();

            let ir = compile_file_ir(file, fidelity)?;

            // Named IR format
            let named_str = serde_json::to_string(&ir_to_wire(&ir))?;
            let named_tokens = bpe.encode_with_special_tokens(&named_str).len();

            // String-table IR format
            let table_str = serde_json::to_string(&ir_to_string_table_wire(&ir))?;
            let table_tokens = bpe.encode_with_special_tokens(&table_str).len();

            // Delta savings: rename a class to generate a delta
            let mut modified_ir = ir.clone();
            modified_ir.version = ir.version + 1;
            if let Some(clean_ctx::ir::CoreOp::DefClass(_, name)) = modified_ir.instructions.first_mut() {
                name.push_str("V2");
            }

            let delta_comp = DeltaComputer::new();
            let delta_info = if let Some(d) = delta_comp.compute(&ir, &modified_ir) {
                let compact = compact_encode(&d);
                let compact_str = serde_json::to_string(&compact)?;
                let named_delta_str = serde_json::to_string(&d)?;
                format!("{} / {}b", compact_str.len(), named_delta_str.len())
            } else {
                "N/A".to_string()
            };

            let name = file
                .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test_files/"))
                .unwrap_or(file)
                .display()
                .to_string();

            let tbl_pct = named_tokens
                .checked_sub(table_tokens)
                .and_then(|diff| diff.checked_mul(100))
                .and_then(|num| num.checked_div(named_tokens))
                .unwrap_or(0);

            println!(
                "  {:<40} {:>6} {:>6} {:>6} {:>4}%   │ {:>16}",
                name, raw_tokens, named_tokens, table_tokens, tbl_pct, delta_info
            );

            total_raw += raw_tokens;
            total_named += named_tokens;
            total_table += table_tokens;
        }

        let overall_pct = total_named
            .checked_sub(total_table)
            .and_then(|diff| diff.checked_mul(100))
            .and_then(|num| num.checked_div(total_named))
            .unwrap_or(0);

        println!("  {}", "─".repeat(110));
        println!(
            "  {:<40} {:>6} {:>6} {:>6} {:>4}%   │",
            "TOTAL", total_raw, total_named, total_table, overall_pct
        );
        println!();
    }

    // ── String table char-level savings breakdown ─────────────────
    println!("───────────────────────────────────────────────────────────────────────");
    println!("  String Table Savings Breakdown (Low fidelity, raw chars)");
    println!("───────────────────────────────────────────────────────────────────────");
    println!("  {:<40} {:>10} {:>10} {:>8}", "file", "named(ch)", "table(ch)", "sav%");

    for file in &sample_files() {
        let ir = compile_file_ir(file, Fidelity::Low)?;
        let (named_chars, table_chars) = estimate_savings(&ir);
        let pct = named_chars
            .checked_sub(table_chars)
            .and_then(|diff| diff.checked_mul(100))
            .and_then(|num| num.checked_div(named_chars))
            .unwrap_or(0);
        let name = file
            .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test_files/"))
            .unwrap_or(file)
            .display()
            .to_string();
        println!("  {:<40} {:>10} {:>10} {:>6}%", name, named_chars, table_chars, pct);
    }

    // ── Compact delta JSON example ────────────────────────────────
    println!();
    println!("───────────────────────────────────────────────────────────────────────");
    println!("  Compact Delta Example (rename a method, Low fidelity)");
    println!("───────────────────────────────────────────────────────────────────────");
    if let Some(first_file) = sample_files().first() {
        let ir = compile_file_ir(first_file, Fidelity::Low)?;
        let mut modified = ir.clone();
        modified.version = ir.version + 1;
        // Rename first method
        for insn in &mut modified.instructions {
            if let clean_ctx::ir::CoreOp::DefMethod(_, _, name) = insn {
                *name = format!("renamed_{}", name);
                break;
            }
        }

        let delta_comp = DeltaComputer::new();
        if let Some(d) = delta_comp.compute(&ir, &modified) {
            // Show size comparison
            let named_full = serde_json::to_string(&ir_to_wire(&ir))?;
            let named_delta = serde_json::to_string(&d)?;
            let compact_delta = compact_encode(&d);
            let compact_str = serde_json::to_string_pretty(&compact_delta)?;

            println!("  Named full IR size:  {} chars / {} tokens",
                named_full.len(),
                bpe.encode_with_special_tokens(&named_full).len());

            println!("  Named delta size:    {} chars / {} tokens",
                named_delta.len(),
                bpe.encode_with_special_tokens(&named_delta).len());

            println!("  Compact delta size:  {} chars / {} tokens",
                compact_str.len(),
                bpe.encode_with_special_tokens(&compact_str).len());

            println!();
            println!("  Compact delta JSON:");
            for line in compact_str.lines() {
                println!("    {}", line);
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Legend:");
    println!("    named = original IR with opcode strings");
    println!("    table = string table + integer index arrays (Phase I)");
    println!("    delta = compact delta with field patches + abbrev opcodes (Phase I)");
    println!("═══════════════════════════════════════════════════════════════════════");

    Ok(())
}