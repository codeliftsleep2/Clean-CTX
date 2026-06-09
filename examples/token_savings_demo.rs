// examples/token_savings_demo.rs
//
// F-FINAL-08 (this PR): end-to-end demo of the token savings
// produced by the new `clean-ctx` compression pipeline. Run with:
//
//     cargo run --example token_savings_demo
//
// The example takes the repo's own test files and compresses them
// through both pipelines:
//   1. Text compression (`compress_file`) — the legacy pipeline
//   2. Compiler IR (`compile_file_ir` → `ir_to_wire`) — the new
//      structural representation pipeline
//
// Both pipelines are the same ones exposed via the MCP server's
// `compress_code_context` tool (which returns both outputs).

use clean_ctx::analytics::bpe_or_init;
use clean_ctx::cache::LocalStateCache;
use clean_ctx::compression::Fidelity;
use clean_ctx::compressor::compress_file;
use clean_ctx::dictionary::PathDictionary;
use clean_ctx::ir::compiler::IRCompiler;
use clean_ctx::ir::layers::typescript::TypeScriptLayer;
use clean_ctx::ir::layers::angular::AngularMetaLayer;
use clean_ctx::ir::layers::patterns::CodePatternRecognizer;
use clean_ctx::ir::patterns::CompressingPatternRecognizer;
use clean_ctx::ir::wire::ir_to_wire;
use clean_ctx::compression::language::language_for_extension;
use std::path::PathBuf;

/// The repo's own test corpus — TS files only (the per-file
/// `compress_file` tool only supports .ts and .cs).
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

/// Compile a file to IR (same logic as `compile_file_ir` in mcp/tools.rs).
fn compile_ir(file: &PathBuf, fidelity: Fidelity) -> Result<String, Box<dyn std::error::Error>> {
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

    let ir = compiler.compile(&source, &file_id, language, &query_string, fidelity)?;
    let wire = ir_to_wire(&ir);
    Ok(serde_json::to_string(&wire)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bpe_or_init()?;
    let bpe = clean_ctx::analytics::bpe();

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Clean-CTX Token Savings Demo (text compression vs compiler IR)");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();
    println!("  Pipeline 1 (text):  compress_file → variable mapping + BPE-optimized text");
    println!("  Pipeline 2 (IR):    compile → CoreOp instructions → wire format (JSON)");
    println!("  Both are returned by the MCP `compress_code_context` tool.");
    println!();

    let files = sample_files();

    for fidelity in [Fidelity::Low, Fidelity::Medium, Fidelity::High] {
        println!("── Fidelity: {:?} ──", fidelity);
        println!(
            "  {:<40} {:>6} {:>6} {:>6}  │ {:>6} {:>6} {:>6}",
            "file", "raw", "txt", "txt%", "raw", "IR", "IR%"
        );
        println!("  {}", "─".repeat(90));

        let mut txt_total_raw = 0usize;
        let mut txt_total_comp = 0usize;
        let mut ir_total_raw = 0usize;
        let mut ir_total_comp = 0usize;

        for file in &files {
            let source = std::fs::read_to_string(file)?;
            let raw_tokens = bpe.encode_with_special_tokens(&source).len();

            // Text compression
            let mut dict = PathDictionary::new();
            let mut cache = LocalStateCache::new();
            let compressed = compress_file(file.clone(), &mut dict, &mut cache, fidelity)?;
            let txt_tokens = bpe.encode_with_special_tokens(&compressed).len();

            // IR compression
            let ir_str = compile_ir(file, fidelity)?;
            let ir_tokens = bpe.encode_with_special_tokens(&ir_str).len();

            let name = file
                .strip_prefix(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/test_files/"))
                .unwrap_or(file)
                .display()
                .to_string();

            let txt_pct = if raw_tokens > 0 {
                (raw_tokens.saturating_sub(txt_tokens) * 100) / raw_tokens
            } else { 0 };
            let ir_pct = if raw_tokens > 0 {
                (raw_tokens.saturating_sub(ir_tokens) * 100) / raw_tokens
            } else { 0 };

            println!(
                "  {:<40} {:>6} {:>6} {:>4}%  │ {:>6} {:>6} {:>4}%",
                name, raw_tokens, txt_tokens, txt_pct, raw_tokens, ir_tokens, ir_pct
            );

            txt_total_raw += raw_tokens;
            txt_total_comp += txt_tokens;
            ir_total_raw += raw_tokens;
            ir_total_comp += ir_tokens;
        }

        let txt_pct = if txt_total_raw > 0 {
            (txt_total_raw.saturating_sub(txt_total_comp) * 100) / txt_total_raw
        } else { 0 };
        let ir_pct = if ir_total_raw > 0 {
            (ir_total_raw.saturating_sub(ir_total_comp) * 100) / ir_total_raw
        } else { 0 };

        println!("  {}", "─".repeat(90));
        println!(
            "  {:<40} {:>6} {:>6} {:>4}%  │ {:>6} {:>6} {:>4}%",
            "TOTAL", txt_total_raw, txt_total_comp, txt_pct,
            ir_total_raw, ir_total_comp, ir_pct
        );
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Legend: txt = text compression pipeline, IR = compiler IR wire format");
    println!("  Both pipelines are returned by compress_code_context MCP tool.");
    println!("  IR output includes structural ops (DEF_C, DEF_M, DEF_F, SIG, etc.)");
    println!("  as JSON arrays — enabling delta-based state transport.");
    println!("═══════════════════════════════════════════════════════════════════════");

    Ok(())
}