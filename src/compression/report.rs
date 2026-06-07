// src/compression/report.rs
//
// Final optimisation-report formatting. These functions wrap the compacted
// body with the token-optimisation header, savings statistics, and path
// alias. They are the last stage of both the streaming and non-streaming
// compression pipelines.

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
        Fidelity::High => format!("// --- Full Layout (High Fidelity): {} ---", path_alias),
    };
    if sym_footer.is_empty() {
        format!("{}\n{}\n", layout_header, display_body)
    } else {
        format!("{}\n{}\n{}", layout_header, display_body, sym_footer)
    }
}

/// Build the complete final output string: savings report + compacted body.
pub fn format_final_output(
    source_code: &str,
    compacted_body: &str,
    fidelity: Fidelity,
    class_count: usize,
    method_count: usize,
    import_count: usize,
) -> String {
    let meta = calculate_savings(source_code, compacted_body);
    let ratio_report = format!(
        "// Structures: {} classes, {} methods, {} imports | {}/{} raw tokens",
        class_count, method_count, import_count, meta.raw_tokens, meta.raw_tokens
    );
    format!(
        "// --- Token Optimization Report --- \n// Raw Tokens: {} | Retained Tokens: {} | Waste Reduced: {:.2}%\n// Fidelity: {:?}\n// {}\n{}",
        meta.raw_tokens, meta.compressed_tokens, meta.savings_percentage, fidelity, ratio_report, compacted_body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_compacted_body_omits_footer_when_empty() {
        let out = format_compacted_body("BODY", "", "a1", Fidelity::Low);
        assert!(out.contains("Compacted Layout (Low Fidelity): a1"));
        assert!(out.contains("BODY"));
        assert!(!out.contains("§SYM"));
    }

    #[test]
    fn format_compacted_body_includes_footer_when_present() {
        let out = format_compacted_body("BODY", "§SYM\n  $1 = Foo", "a1", Fidelity::Low);
        assert!(out.contains("§SYM"));
    }

    #[test]
    fn format_compacted_body_labels_medium() {
        let out = format_compacted_body("BODY", "", "b2", Fidelity::Medium);
        assert!(out.contains("Enhanced Layout (Medium Fidelity): b2"));
    }

    #[test]
    fn format_compacted_body_labels_high() {
        let out = format_compacted_body("BODY", "", "c3", Fidelity::High);
        assert!(out.contains("Full Layout (High Fidelity): c3"));
    }
}