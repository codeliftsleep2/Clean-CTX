// src/compression/output.rs
//
// Output assembly for the compression pipeline. Turns the sorted tree-sitter
// capture stream into the per-fidelity compacted body lines, with CBM
// filter-first skipping applied inside the walk. Shared by the non-streaming
// orchestrator. Extracted from `pipeline.rs` at a structural boundary
// (active-file size rule).

use std::collections::HashSet;

use crate::compaction::{
    compact_import, format_class_entry, format_java_type_entry, format_rust_type_entry,
    simple_compact,
};
use crate::compression::markers::build_marker;
use crate::compression::skip::should_skip_capture;
use crate::compression::CapEntry;
use crate::compression::Fidelity;

/// Output of [`build_output_lines`]. F-04 (FAANG audit): previously
/// the orchestrator counted classes/methods/imports by
/// `let class_count: usize = 0;` and bound it to `_`, then passed
/// `0, 0, 0` to `format_final_output`. The header always lied.
/// The struct now carries the real counts.
#[derive(Debug, Clone)]
pub struct BuildOutputResult {
    /// The compacted body lines, in document order, with the import
    /// block prepended.
    pub output_lines: Vec<String>,
    /// Number of `class.root` captures that emitted a class entry.
    pub class_count: usize,
    /// Number of `method.root` captures that emitted a method line.
    pub method_count: usize,
    /// Number of `import.root` captures that produced a non-empty
    /// import line.
    pub import_count: usize,
    /// Angular Meta-Layer block (Phase 1: Tier 1 decorator extraction).
    pub meta_block: Option<crate::angular_meta::MetaBlock>,
    /// Spring Boot Meta-Layer block (Phase 1: Tier 1 annotation extraction).
    pub spring_meta_block: Option<crate::spring_meta::MetaBlock>,
    /// .NET / C# Meta-Layer block.
    pub dotnet_meta_block: Option<crate::dotnet_meta::MetaBlock>,
}

/// Walk the captures in document order and build the output lines +
/// the per-fidelity counts. Shared between the streaming and
/// non-streaming variants.
///
/// F-04: the return type is now `BuildOutputResult` (with the real
/// counts) instead of a `(Vec<String>, Vec<String>)` tuple.
///
/// `skip_set`: optional set of symbol names to exclude from the output.
/// When a class, method, or field name matches an entry in this set,
/// the capture is dropped entirely. Used by the CBM filter-first
/// architecture to exclude low-importance symbols.
///
/// `config`: optional project config forwarded to the meta-layer registry
/// so per-framework `enabled` flags and sub-layer settings are honored.
pub fn build_output_lines(
    all_captures: &[CapEntry],
    source_code: &str,
    fidelity: Fidelity,
    skip_set: Option<&HashSet<String>>,
    config: Option<&crate::config::CleanCtxConfig>,
) -> BuildOutputResult {
    let mut output_lines: Vec<String> = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut markers: Vec<String> = Vec::new();
    let mut imports: Vec<String> = Vec::new();
    let mut class_count: usize = 0;
    let mut method_count: usize = 0;
    let mut import_count: usize = 0;
    let mut class_captures: Vec<String> = Vec::new();

    for cap in all_captures {
        if let Some(skip) = skip_set {
            if !skip.is_empty() && should_skip_capture(cap, skip) {
                continue;
            }
        }
        match cap.name.as_str() {
            "import.root" | "mod.root" | "package.root" => {
                let compact = compact_import(&cap.text, fidelity);
                if !compact.is_empty() {
                    imports.push(compact);
                    import_count += 1;
                }
            }
            "class.root" => {
                if !output_lines.is_empty()
                    && (fidelity == Fidelity::High || fidelity == Fidelity::Medium)
                {
                    output_lines.push(String::new());
                }
                output_lines.push(format_class_entry(&cap.text, &fields, fidelity));
                // The meta-layer contract requires the FULL class text
                // (leading decorators + body), NOT the compacted name.
                // Reconstruct the decorator-inclusive span via the CANONICAL
                // shared helper (same one used by
                // `mcp::workspace_util::extract_class_blocks`).
                class_captures.push(
                    crate::meta_util::class_source_from_capture(source_code, cap).to_string(),
                );
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            "struct.root" | "trait.root" | "impl.root" => {
                if !output_lines.is_empty()
                    && (fidelity == Fidelity::High || fidelity == Fidelity::Medium)
                {
                    output_lines.push(String::new());
                }
                output_lines.push(format_rust_type_entry(&cap.text, &fields, fidelity));
                class_captures.push(cap.text.clone());
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            "interface.root" | "enum.root" | "record.root" => {
                if !output_lines.is_empty()
                    && (fidelity == Fidelity::High || fidelity == Fidelity::Medium)
                {
                    output_lines.push(String::new());
                }
                output_lines.push(format_java_type_entry(
                    &cap.text, &cap.name, &fields, fidelity,
                ));
                class_captures.push(cap.text.clone());
                class_count += 1;
                fields.clear();
                markers.clear();
            }
            "method.root" | "constructor.root" => {
                let sig = &cap.text;
                if !markers.is_empty() {
                    let marker_str = markers.join(" ");
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {} {{ {} }}", sig, marker_str));
                    } else if fidelity == Fidelity::Medium {
                        output_lines.push(format!("{} {}", sig, marker_str));
                    } else {
                        output_lines.push(sig.clone());
                    }
                } else {
                    if fidelity == Fidelity::High {
                        output_lines.push(format!("  {}", sig));
                    } else {
                        output_lines.push(sig.clone());
                    }
                }
                method_count += 1;
                markers.clear();
            }
            "field.root" => {
                if !cap.text.is_empty() {
                    fields.push(cap.text.clone());
                }
            }
            _ => {
                if fidelity == Fidelity::Low {
                    continue;
                }
                if let Some(marker) = build_marker(&cap.name, &cap.text)
                    && markers.last().map(|m| m != &marker).unwrap_or(true)
                {
                    markers.push(marker);
                }
            }
        }
    }

    if !fields.is_empty() && output_lines.is_empty() && fidelity != Fidelity::Low {
        output_lines.push(format!("⊕fields {{ {} }}", fields.join("; ")));
    }

    if output_lines.is_empty()
        && let Some(first_line) = source_code.lines().next()
    {
        let trimmed = first_line.trim().to_string();
        if !trimmed.is_empty() {
            // H-5: at Edit/Verbatim the raw fallback must be byte-exact
            // (preserve internal whitespace), not re-compact via simple_compact.
            if fidelity == Fidelity::Edit || fidelity == Fidelity::Verbatim {
                output_lines.push(trimmed);
            } else {
                output_lines.push(simple_compact(&trimmed, fidelity));
            }
        }
    }

    let mut output = output_lines;
    if !imports.is_empty() {
        let import_block = match fidelity {
            Fidelity::Low => imports.join("; "),
            _ => imports.join("\n"),
        };
        output.insert(0, import_block);
    }

    // Phase 4: Dispatch meta-layers through the registry.
    // The `config` is threaded through so per-framework `enabled` flags
    // and sub-layer settings (min_pipe_operators, include_dispatch_sites,
    // etc.) are honored.
    let registry = crate::layers::LayerRegistry::global();
    let meta_results =
        registry.run_meta_layers_pipeline(source_code, &class_captures, fidelity, config);

    // The registry now returns structured `MetaLayerOutput` values. Use the
    // structured blocks directly — no render-then-reparse.
    let meta_block = meta_results
        .iter()
        .find(|o| o.layer_name == "angular")
        .and_then(|o| o.angular_block.clone());

    let spring_meta_block = meta_results
        .iter()
        .find(|o| o.layer_name == "spring_boot")
        .and_then(|o| o.spring_block.clone());

    let dotnet_meta_block = meta_results
        .iter()
        .find(|o| o.layer_name == "dotnet")
        .and_then(|o| o.dotnet_block.clone());

    BuildOutputResult {
        output_lines: output,
        class_count,
        method_count,
        import_count,
        meta_block,
        spring_meta_block,
        dotnet_meta_block,
    }
}

/// Combine the symbol-dictionary footer and the type-alias footer into
/// a single string.
pub(crate) fn combine_footers(sym_footer: &str, ta_footer: &str) -> String {
    if ta_footer.is_empty() {
        sym_footer.to_string()
    } else if sym_footer.is_empty() {
        ta_footer.to_string()
    } else {
        format!("{}\n{}", sym_footer, ta_footer)
    }
}

/// Join the body lines using the per-fidelity separator.
pub fn assemble_body(output_lines: &[String], fidelity: Fidelity) -> String {
    match fidelity {
        Fidelity::Low => output_lines.join(";"),
        Fidelity::Medium => output_lines.join("\n"),
        Fidelity::High => output_lines.join("\n"),
        // Edit/Verbatim preserve structure — newline-joined.
        Fidelity::Edit | Fidelity::Verbatim => output_lines.join("\n"),
    }
}

