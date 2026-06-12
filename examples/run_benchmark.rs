// examples/run_benchmark.rs
//
// Runs single-pass compression on the three canonical test files
// at all three fidelity levels, measuring the text compression
// pipeline (not IR) to match the documentation format.
//
//     cargo run --release --example run_benchmark

use clean_ctx::analytics::{bpe, bpe_or_init};
use clean_ctx::cache::LocalStateCache;
use clean_ctx::compression::Fidelity;
use clean_ctx::compressor::compress_file;
use clean_ctx::dictionary::PathDictionary;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bpe_or_init()?;
    let bpe = bpe();

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let entries: Vec<(&str, PathBuf)> = vec![
        ("sample_service.ts", manifest_dir.join("src/test_files/sample_service.ts")),
        ("LargeService.ts", manifest_dir.join("src/test_files/LargeService.ts")),
        ("UserManagementService.ts", manifest_dir.join("src/test_files/UserManagementService.ts")),
    ];

    println!("{}", "=".repeat(75));
    println!("  Clean-CTX Single-Pass Compression Benchmarks (Text Pipeline)");
    println!("{}", "=".repeat(75));
    println!();
    println!("  Pipeline: source -> tree-sitter AST -> fidelity filter -> opcode encode");
    println!("  Tokenizer: cl100k BPE (tiktoken-rs)");
    println!();

    let mut agg_raw: [usize; 3] = [0; 3];
    let mut agg_compressed: [usize; 3] = [0; 3];
    let fidelities = [Fidelity::Low, Fidelity::Medium, Fidelity::High];

    for (name, file_path) in &entries {
        let source = std::fs::read_to_string(file_path)?;
        let raw_tokens = bpe.encode_with_special_tokens(&source).len();
        let line_count = source.lines().count();

        println!("-- {} ({} lines, {} raw tokens) --", name, line_count, raw_tokens);
        println!();

        for (fi, fidelity) in fidelities.iter().enumerate() {
            let mut dict = PathDictionary::new();
            let mut cache = LocalStateCache::new();
            let compressed_text = compress_file(
                file_path.clone(),
                &mut dict,
                &mut cache,
                *fidelity,
            )?;
            let compressed_tokens = bpe.encode_with_special_tokens(&compressed_text).len();
            let saved = raw_tokens.saturating_sub(compressed_tokens);
            let reduction_pct = if raw_tokens > 0 {
                (saved as f64 / raw_tokens as f64 * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            };

            println!("  {:<8} | raw={:>5} -> compressed={:>5} | saved={:>5} | {:.2}%",
                format!("{:?}", fidelity),
                raw_tokens,
                compressed_tokens,
                saved,
                reduction_pct,
            );

            agg_raw[fi] += raw_tokens;
            agg_compressed[fi] += compressed_tokens;
        }
        println!();
    }

    // Aggregate summary
    println!("====== Aggregated Summary ======");
    println!();

    for (fi, fidelity) in fidelities.iter().enumerate() {
        let total_saved = agg_raw[fi].saturating_sub(agg_compressed[fi]);
        let pct = if agg_raw[fi] > 0 {
            (total_saved as f64 / agg_raw[fi] as f64 * 100.0 * 100.0).round() / 100.0
        } else {
            0.0
        };

        println!("  {:<8} | total_raw={:>6} total_compressed={:>6} | saved={:>6} | {:.2}%",
            format!("{:?}", fidelity),
            agg_raw[fi],
            agg_compressed[fi],
            total_saved,
            pct,
        );
    }

    println!();
    println!("{}", "=".repeat(75));
    Ok(())
}
