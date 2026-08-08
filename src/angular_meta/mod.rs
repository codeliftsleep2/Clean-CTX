// src/angular_meta/mod.rs
//
// Angular Meta-Layer — Tier 1 (Decorators) + Tier 2 (File-Triplet Bundling).
//
// The Meta-Layer is **purely additive**: it never modifies the existing
// TS compaction output. It only appends a `Φ` block below the existing
// compacted class. Existing users see no change; Angular users get
// enriched output with `@Component` / `@Injectable` / `@Input` /
// `@Output` context.
//
// # Module structure
//
// - `detect`     : Angular detection heuristic
// - `decorators` : `@Component` / `@Injectable` / `@NgModule` /
//                  `@Directive` / `@Pipe` / `@Input` / `@Output` extractor
// - `markers`    : `Φ` marker construction & expansion
// - `bundler`    : file-triplet resolver (*.component.ts → .html + .scss)
// - `template`   : tree-sitter-html Angular-syntax template extractor
// - `style`      : CSS/SCSS class + var extractor
// - `footer`     : `§ΦMAP` workspace footer formatter
// - (this file)  : Public surface, `MetaBlock` struct, `run_meta_layer`

pub(crate) mod decorators;
pub(crate) mod detect;
pub(crate) mod markers;
pub mod bundler;
pub mod template;
pub mod template_compress;
pub mod style;
pub mod footer;
pub mod graph;
pub mod graph_state;

use crate::compression::Fidelity;

/// The Meta-Layer output for a single `.ts` file.
///
/// `None` means "not an Angular file" — the caller should not emit any
/// Φ block at all (zero overhead, byte-identical to non-Angular
/// output).
///
/// `Some(block)` means "Angular file" — the caller should append the
/// Φ block lines below the existing compacted class entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaBlock {
    /// One `Φ` line per Angular-bearing class, in document order.
    /// Each line is the fully-formatted marker line (e.g. `Φcmp:Foo sel=...`).
    /// Already newline-separated; caller is responsible for the
    /// surrounding `// --- Φ Angular Meta ---` header.
    pub lines: Vec<String>,
}

impl MetaBlock {
    /// Returns `true` if there are no Φ lines to emit (i.e. the
    /// caller should skip the entire `// --- Φ Angular Meta ---` block).
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Render the full Φ block, including the header. Returns an
    /// empty string when the block is empty (so callers can `+=`
    /// blindly).
    pub fn render(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();
        s.push_str("// --- Φ Angular Meta ---\n");
        for line in &self.lines {
            s.push_str(line);
            s.push('\n');
        }
        s
    }
}

/// Run the Tier-1 Meta-Layer pass on a single `.ts` file's raw source
/// text. Returns `None` when the file is not an Angular file (no
/// `@Component` / `@Injectable` / etc. decorators present), and
/// `Some(MetaBlock)` when at least one class carries a recognised
/// Angular decorator.
///
/// The function is deliberately **string-based** for Phase 1 — it
/// does not re-parse the AST. The TS capture pipeline has already
/// given us `class.root` captures; we walk the raw text of each
/// class capture looking for decorators that immediately precede
/// it. This is the same strategy used by `compaction/import.rs` and
/// is sufficient for the Tier 1 deliverable (no new dependencies,
/// no new file types, no AST changes).
///
/// # Arguments
///
/// - `source_code`    : the full source text of the file being compressed
/// - `class_captures` : the slice texts of each `class.root` capture
///   (in document order, already sorted by
///   `run_capture_pipeline`)
/// - `fidelity`       : fidelity level (F-ANG-23 — used to drive
///   per-fidelity marker verbosity):
///   - `Fidelity::Low`    → emit only `@Component` / `@Injectable`
///     / `@Directive` / `@Pipe` / `@NgModule` summary markers
///     (no field-level `@Input` / `@Output`).
///   - `Fidelity::Medium` → add field-level `@Input` / `@Output`
///     markers; skip constructor `Φinjects:` (covered by class summary).
///   - `Fidelity::High`   → emit everything including
///     `Φinjects:` and the modern `input()`/`output()`/`model()`
///     signal lines.
pub fn run_meta_layer(
    source_code: &str,
    class_captures: &[String],
    fidelity: Fidelity,
) -> Option<MetaBlock> {
    // Tier 0 (detection): is this an Angular file at all?
    if !detect::is_angular_file(source_code) {
        return None;
    }

    // Tier 1 (extraction): walk each class capture and emit Φ lines.
    // F-ANG-23: fidelity now controls the verbosity of the output.
    let mut block = MetaBlock::default();
    for raw_class in class_captures {
        if let Some(result) = decorators::extract_decorators(raw_class, fidelity) {
            block.lines.extend(result.lines);
            // Tier 2.5: run tree-sitter-html on inline templates.
            // Only for components with `template:` (not `templateUrl:`)
            // since external .html files are handled by the workspace
            // bundle pass.
            // This is only available when the `angular` feature is enabled.
            #[cfg(feature = "angular")]
            if let Some(tpl) = &result.inline_template {
                if !tpl.trim().is_empty() {
                    // Fidelity-gated template rendering (ANGULAR_HTML_COMPRESSION_PLAN).
                    // Low → single-line shape summary; Medium/High → multi-line
                    // structural Angular semantics.
                    let shape = template::extract_template_shape(tpl);
                    for line in shape.to_marker_lines(fidelity) {
                        if line != "Φtpl:empty" {
                            block.lines.push(line);
                        }
                    }
                }
            }
        }
    }

    if block.is_empty() {
        // Angular file but no Angular decorators on any class. Be
        // conservative — do not emit a Φ block header.
        return None;
    }

    Some(block)
}

#[cfg(all(test, feature = "angular"))]
#[path = "../tests/angular_meta/mod.rs"]
mod tests;
