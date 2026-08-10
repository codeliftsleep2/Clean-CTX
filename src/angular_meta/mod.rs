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
pub mod rx;
pub mod ngrx;
pub mod signals;
pub mod routing;
pub mod phi;
pub mod util;

use crate::compression::Fidelity;

/// A single named section of a meta-layer block.
///
/// Each section has its own header (e.g. `// --- Φ Angular Meta ---`
/// or `// --- Φ RxJS Meta ---`) and a set of `Φ` marker lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaSection {
    /// The header line (e.g. `"// --- Φ Angular Meta ---"`).
    pub header: String,
    /// The `Φ` marker lines for this section.
    pub lines: Vec<String>,
}

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
    ///
    /// **Backward-compat:** this is the Angular decorator section.
    /// New layers (RxJS, NgRx, Signals, Routing) use `sections`.
    pub lines: Vec<String>,
    /// Named sections for additional meta-layers (RxJS, NgRx, etc.).
    /// Each section carries its own header.
    pub sections: Vec<MetaSection>,
}

impl MetaBlock {
    /// Returns `true` if there are no Φ lines to emit (i.e. the
    /// caller should skip the entire `// --- Φ Angular Meta ---` block).
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && self.sections.iter().all(|s| s.lines.is_empty())
    }

    /// Render the full Φ block, including the header. Returns an
    /// empty string when the block is empty (so callers can `+=`
    /// blindly).
    pub fn render(&self) -> String {
        if self.is_empty() {
            return String::new();
        }
        let mut s = String::new();

        // Angular decorator section (backward-compat `lines` field).
        if !self.lines.is_empty() {
            s.push_str("// --- Φ Angular Meta ---\n");
            for line in &self.lines {
                s.push_str(line);
                s.push('\n');
            }
        }

        // Additional named sections (RxJS, NgRx, Signals, Routing).
        for section in &self.sections {
            if section.lines.is_empty() {
                continue;
            }
            s.push_str(&section.header);
            s.push('\n');
            for line in &section.lines {
                s.push_str(line);
                s.push('\n');
            }
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
    run_meta_layer_with_config(source_code, class_captures, fidelity, None)
}

/// Run the meta-layer with an optional [`MetaLayerConfig`] so the
/// per-layer sub-configs (RxJS `min_pipe_operators`, NgRx
/// `include_dispatch_sites`/`include_select_sites`/`entity_selectors`,
/// and the `enabled` master switches) are honored.
///
/// When `config` is `None`, all layers run with their defaults (the
/// same behaviour as [`run_meta_layer`]).
pub fn run_meta_layer_with_config(
    source_code: &str,
    class_captures: &[String],
    fidelity: Fidelity,
    config: Option<&crate::config::MetaLayerConfig>,
) -> Option<MetaBlock> {
    // Tier 0 (detection): is this an Angular file at all?
    let is_angular = detect::is_angular_file(source_code);

    // Resolve per-layer enabled flags from config (defaults: all enabled).
    let rxjs_enabled = config.map(|c| c.rxjs.enabled).unwrap_or(true);
    let ngrx_enabled = config.map(|c| c.ngrx.enabled).unwrap_or(true);
    let signals_enabled = config.map(|c| c.signals.enabled).unwrap_or(true);
    let routing_enabled = config.map(|c| c.routing.enabled).unwrap_or(true);

    // Detect RxJS independently — a file may be RxJS without being
    // Angular (e.g. a standalone RxJS service or utility).
    let rx_shape = if rxjs_enabled {
        rx::extract_rx_shape(source_code, fidelity)
    } else {
        None
    };

    // Detect NgRx independently — NgRx files are typically Angular
    // but may not have decorators (e.g. actions/selectors files).
    let ngrx_shape = if ngrx_enabled {
        ngrx::extract_ngrx_shape(source_code, fidelity)
    } else {
        None
    };

    // Detect Signals independently — modern Angular components use
    // signal()/computed()/effect() without decorators.
    let signal_shape = if signals_enabled {
        signals::extract_signal_shape(source_code, fidelity)
    } else {
        None
    };

    // Detect Routing independently — route config files may not have
    // decorators (e.g. `app.routes.ts`).
    let route_shape = if routing_enabled {
        routing::extract_route_shape(source_code, fidelity)
    } else {
        None
    };

    if !is_angular
        && rx_shape.is_none()
        && ngrx_shape.is_none()
        && signal_shape.is_none()
        && route_shape.is_none()
    {
        // Neither Angular decorators, RxJS, NgRx, Signals, nor Routing —
        // zero overhead.
        return None;
    }

    // Tier 1 (extraction): walk each class capture and emit Φ lines.
    // F-ANG-23: fidelity now controls the verbosity of the output.
    let mut block = MetaBlock::default();

    if is_angular {
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
    }

    // Tier 2 (RxJS): append the RxJS meta block if present.
    if let Some(shape) = rx_shape {
        let rx_block = shape.render_with_config(fidelity, config.map(|c| c.rxjs.min_pipe_operators));
        if !rx_block.is_empty() {
            // The RxJS block has its own `// --- Φ RxJS Meta ---` header.
            // Split the rendered block into header + lines and store as
            // a named section so it renders under the correct header.
            let mut rx_lines: Vec<String> = rx_block.lines()
                .map(|l| l.to_string())
                .collect();
            // First line is the header — pop it off.
            let header = if rx_lines.first().map(|l| l.starts_with("// ---")).unwrap_or(false) {
                rx_lines.remove(0)
            } else {
                "// --- Φ RxJS Meta ---".to_string()
            };
            block.sections.push(MetaSection {
                header,
                lines: rx_lines,
            });
        }
    }

    // Tier 3 (NgRx): append the NgRx meta block if present.
    if let Some(shape) = ngrx_shape {
        let ngrx_block = shape.render_with_config(fidelity, config.map(|c| &c.ngrx));
        if !ngrx_block.is_empty() {
            let mut ngrx_lines: Vec<String> = ngrx_block.lines()
                .map(|l| l.to_string())
                .collect();
            let header = if ngrx_lines.first().map(|l| l.starts_with("// ---")).unwrap_or(false) {
                ngrx_lines.remove(0)
            } else {
                "// --- Φ NgRx Meta ---".to_string()
            };
            block.sections.push(MetaSection {
                header,
                lines: ngrx_lines,
            });
        }
    }

    // Tier 4 (Signals): append the Signals meta block if present.
    if let Some(shape) = signal_shape {
        let signal_block = shape.render(fidelity);
        if !signal_block.is_empty() {
            let mut signal_lines: Vec<String> = signal_block.lines()
                .map(|l| l.to_string())
                .collect();
            let header = if signal_lines.first().map(|l| l.starts_with("// ---")).unwrap_or(false) {
                signal_lines.remove(0)
            } else {
                "// --- Φ Signals Meta ---".to_string()
            };
            block.sections.push(MetaSection {
                header,
                lines: signal_lines,
            });
        }
    }

    // Tier 5 (Routing): append the Routing meta block if present.
    if let Some(shape) = route_shape {
        let route_block = shape.render(fidelity);
        if !route_block.is_empty() {
            let mut route_lines: Vec<String> = route_block.lines()
                .map(|l| l.to_string())
                .collect();
            let header = if route_lines.first().map(|l| l.starts_with("// ---")).unwrap_or(false) {
                route_lines.remove(0)
            } else {
                "// --- Φ Routing Meta ---".to_string()
            };
            block.sections.push(MetaSection {
                header,
                lines: route_lines,
            });
        }
    }

    if block.is_empty() {
        // Neither Angular decorators, RxJS, NgRx, Signals, nor Routing
        // artifacts. Be conservative — do not emit a Φ block header.
        return None;
    }

    Some(block)
}

#[cfg(all(test, feature = "angular"))]
#[path = "../tests/angular_meta/mod.rs"]
mod tests;
