// src/compression/report.rs
//
// Final optimisation-report formatting. These functions wrap the compacted
// body with the token-optimisation header, savings statistics, and path
// alias. They are the last stage of both the streaming and non-streaming
// compression pipelines.
//
// Phase III (Idea #8 — Progressive Header Elision):
// Low fidelity now uses an ultra-compact header format (~5 tokens) instead
// of the verbose ~30-token header. The compact format is parseable and
// preserves all information. Medium/High fidelities keep the readable header.

use crate::analytics::calculate_savings;
use crate::compression::Fidelity;

/// Wrap the compacted body in the structural layout header.
/// If a symbol footer is present (Low fidelity) it is appended after the body.
pub fn format_compacted_body(
    display_body: &str,
    sym_footer: &str,
    path_alias: &str,
    fidelity: Fidelity,
) -> String {
    let layout_header = match fidelity {
        Fidelity::Low => format!("// --- Compacted Layout (Low Fidelity): {} ---", path_alias),
        Fidelity::Medium => format!("// --- Enhanced Layout (Medium Fidelity): {} ---", path_alias),
        // H-6 (FAANG audit): Edit/Verbatim were mislabeled as "High Fidelity".
        // Edit carries byte-exact method bodies; Verbatim is the full raw source.
        Fidelity::High => format!("// --- Full Layout (High Fidelity): {} ---", path_alias),
        Fidelity::Edit => format!("// --- Edit Layout (Structural + Verbatim Bodies): {} ---", path_alias),
        Fidelity::Verbatim => format!("// --- Verbatim Layout (Full Source): {} ---", path_alias),
    };
    if sym_footer.is_empty() {
        format!("{}\n{}\n", layout_header, display_body)
    } else {
        format!("{}\n{}\n{}", layout_header, display_body, sym_footer)
    }
}

/// Build the complete final output string: savings report + compacted body.
///
/// F-04 (FAANG audit): the previous implementation hard-coded
/// `0, 0, 0` for class/method/import counts and emitted
/// `"{raw}/{raw} raw tokens"` (the denominator was wrong, the
/// numerator and denominator were the same value). The signature
/// now takes the real counts from the orchestrator, and the
/// denominator uses `compressed_tokens`.
///
/// Phase III (Idea #8 — Progressive Header Elision):
/// Low fidelity emits the compact header format (§raw:compressed:savings_pct|fidelity|cls:mth:imp|alias§)
/// instead of the verbose ~30-token header. Medium and High fidelities use the
/// traditional readable format.
pub fn format_final_output(
    source_code: &str,
    compacted_body: &str,
    fidelity: Fidelity,
    class_count: usize,
    method_count: usize,
    import_count: usize,
) -> String {
    if fidelity == Fidelity::Low {
        return format_final_output_compact(
            source_code,
            compacted_body,
            class_count,
            method_count,
            import_count,
        );
    }
    // H-7 (FAANG audit): At Edit/Verbatim the verbose "Token Optimization
    // Report" header would break byte-exactness expectations. Emit the
    // compacted body as-is (the layout header already identifies the mode).
    if fidelity == Fidelity::Edit || fidelity == Fidelity::Verbatim {
        return compacted_body.to_string();
    }
    let meta = calculate_savings(source_code, compacted_body, None);
    let ratio_report = format!(
        "// Structures: {} classes, {} methods, {} imports | {}/{} tokens",
        class_count,
        method_count,
        import_count,
        meta.raw_tokens,
        meta.compressed_tokens,
    );
    format!(
        "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
        meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, compacted_body
    )
}

/// Compact header format for Low fidelity (~5 tokens vs ~30 tokens).
///
/// Format: `§raw_tokens:compressed_tokens:savings_pct|fidelity_code|classes:methods:imports|file_alias§`
///
/// Fidelity codes: L = Low, M = Medium, H = High, C = Cache hit
///
/// Example: `§245:67:72.6|L|3:12:5|α1§`
fn format_final_output_compact(
    source_code: &str,
    compacted_body: &str,
    class_count: usize,
    method_count: usize,
    import_count: usize,
) -> String {
    let meta = calculate_savings(source_code, compacted_body, None);
    // Compact header: §raw:compressed:savings_pct|L|classes:methods:imports|alias§
    // Round savings to 1 decimal place (max 5 chars e.g. 99.9)
    let savings_str = format_savings_pct(meta.savings_percentage);
    let header = format!(
        "§{}:{}:{}|L|{}:{}:{}§",
        meta.raw_tokens,
        meta.compressed_tokens,
        savings_str,
        class_count,
        method_count,
        import_count,
    );
    format!("{}\n{}", header, compacted_body)
}

/// Format the savings percentage to at most 1 decimal place, with no trailing
/// zeros. E.g. 72.65 → "72.7", 100.0 → "100", 0.0 → "0".
fn format_savings_pct(pct: f64) -> String {
    if pct == pct.trunc() {
        format!("{:.0}", pct)
    } else {
        format!("{:.1}", pct)
    }
}

/// Parse a compact header string and return the extracted fields.
/// Returns `None` if the string does not match the compact header format.
///
/// Used by tests and by the decompression path if needed.
///
/// Note: `§` (U+00A7) is a multi-byte UTF-8 character (2 bytes). All
/// slicing below uses char-boundary-aware logic, not raw byte indices.
#[allow(dead_code)]
pub fn parse_compact_header(
    header: &str,
) -> Option<(usize, usize, f64, usize, usize, usize)> {
    let header = header.trim();
    // Match §raw:compressed:savings_pct|L|classes:methods:imports§
    if !header.starts_with('§') || !header.ends_with('§') || header.len() < 6 {
        return None;
    }
    // Find byte positions of the opening and closing §.
    let start = 0; // header starts with § at byte 0
    let end = header.len(); // header ends with § at the last byte
    // Skip opening § (which is 2 bytes) and closing § (also 2 bytes).
    let inner_start = start + '§'.len_utf8();
    let inner_end = end - '§'.len_utf8();
    if inner_start >= inner_end {
        return None;
    }
    let inner = &header[inner_start..inner_end];
    // Split on | — we expect 3 parts: "raw:compressed:pct", "L", "classes:methods:imports"
    let parts: Vec<&str> = inner.split('|').collect();
    if parts.len() != 3 {
        return None;
    }
    let (stats_str, _fidelity_str, counts_str) = (parts[0], parts[1], parts[2]);
    // Parse "raw:compressed:pct"
    let stats: Vec<&str> = stats_str.split(':').collect();
    if stats.len() != 3 {
        return None;
    }
    let raw_tokens: usize = stats[0].parse().ok()?;
    let compressed_tokens: usize = stats[1].parse().ok()?;
    let savings_pct: f64 = stats[2].parse().ok()?;
    // Parse "classes:methods:imports"
    let counts: Vec<&str> = counts_str.split(':').collect();
    if counts.len() != 3 {
        return None;
    }
    let class_count: usize = counts[0].parse().ok()?;
    let method_count: usize = counts[1].parse().ok()?;
    let import_count: usize = counts[2].parse().ok()?;
    Some((raw_tokens, compressed_tokens, savings_pct, class_count, method_count, import_count))
}

/// Build a compact cache-hit notice (~2-3 tokens vs ~5 tokens for verbose).
///
/// Format: `§245:67:72.6|C|α1§`
/// The `C` designates a cache hit (no counts available).
pub fn format_compact_cache_hit(
    raw_tokens: usize,
    compressed_tokens: usize,
    savings_pct: f64,
    path_alias: &str,
) -> String {
    let savings_str = format_savings_pct(savings_pct);
    format!(
        "§{}:{}:{}|C|{}§\n// [CACHE_HIT] {} unchanged. Use historic memory.\n",
        raw_tokens, compressed_tokens, savings_str, path_alias, path_alias
    )
}

#[cfg(test)]
#[path = "../tests/compression/report.rs"]
mod tests;