// src/angular_meta/rx.rs
//
// RxJS Meta-Layer — Phase 1 of the Angular Ecosystem Deepening.
//
// Detects and compresses RxJS observable declarations, subject
// instantiations, pipe chains, and static combinators in Angular
// TypeScript files. Outputs a `// --- Φ RxJS Meta ---` block.
//
// # Purely additive
//
// The RxJS meta-layer never modifies existing TS compression output.
// It only appends a `Φ RxJS Meta` block below the existing compacted
// class. Non-RxJS files pay zero overhead (import-gate detection).
//
// # Marker architecture
//
// This module defines its own `RxJsKind` sub-enum (not added to the
// existing `PhiLineKind` in `markers.rs`) to avoid a 41-variant
// monolith. The `expand_phi_in_line` function is chained into the
// existing Angular expansion via `markers.rs`.

use crate::angular_meta::phi::PhiMarker;
use crate::compression::Fidelity;

// ---------------------------------------------------------------------------
// RxJsKind — single source of truth for RxJS marker vocabulary
// ---------------------------------------------------------------------------

/// Every known `Φ` marker kind for RxJS constructs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RxJsKind {
    Observable,
    Subject,
    PipeRx,
    Map,
    Tap,
    Filter,
    Catch,
    Finalize,
    Delay,
    Combine,
    Share,
    To,
    With,
    Scan,
    Distinct,
    Retry,
}

impl PhiMarker for RxJsKind {
    /// The `Φ` marker prefix for this kind (e.g. `"Φobs:"`).
    fn marker_prefix(self) -> &'static str {
        match self {
            Self::Observable => "Φobs:",
            Self::Subject => "Φsubject:",
            Self::PipeRx => "ΦpipeRx:",
            Self::Map => "Φmap:",
            Self::Tap => "Φtap:",
            Self::Filter => "Φfilter:",
            Self::Catch => "Φcatch:",
            Self::Finalize => "Φfinalize:",
            Self::Delay => "Φdelay:",
            Self::Combine => "Φcombine:",
            Self::Share => "Φshare:",
            Self::To => "Φto:",
            Self::With => "Φwith:",
            Self::Scan => "Φscan:",
            Self::Distinct => "Φdistinct:",
            Self::Retry => "Φretry:",
        }
    }

    /// The human-readable expansion (e.g. `"Observable"`).
    /// Does NOT include the trailing space.
    fn expansion(self) -> &'static str {
        match self {
            Self::Observable => "Observable",
            Self::Subject => "Subject",
            Self::PipeRx => "PipeRx",
            Self::Map => "Map",
            Self::Tap => "Tap",
            Self::Filter => "Filter",
            Self::Catch => "CatchError",
            Self::Finalize => "Finalize",
            Self::Delay => "Delay",
            Self::Combine => "CombineLatest",
            Self::Share => "Share",
            Self::To => "FirstValueFrom",
            Self::With => "WithLatestFrom",
            Self::Scan => "Scan",
            Self::Distinct => "DistinctUntilChanged",
            Self::Retry => "Retry",
        }
    }

    /// All variants in a canonical order (longer prefixes first to
    /// prevent partial-match issues in string replacement).
    fn all_in_expand_order() -> &'static [RxJsKind] {
        &[
            Self::Observable, // Φobs:      (5 chars)
            Self::Subject,    // Φsubject:  (9 chars)
            Self::PipeRx,     // ΦpipeRx:   (8 chars)
            Self::Map,        // Φmap:      (5 chars)
            Self::Tap,        // Φtap:      (5 chars)
            Self::Filter,     // Φfilter:   (8 chars)
            Self::Catch,      // Φcatch:    (7 chars)
            Self::Finalize,   // Φfinalize: (10 chars)
            Self::Delay,      // Φdelay:    (7 chars)
            Self::Combine,    // Φcombine:  (9 chars)
            Self::Share,      // Φshare:    (7 chars)
            Self::To,         // Φto:       (4 chars)
            Self::With,       // Φwith:     (6 chars)
            Self::Scan,       // Φscan:     (6 chars)
            Self::Distinct,   // Φdistinct: (10 chars)
            Self::Retry,      // Φretry:    (7 chars)
        ]
    }

    /// Look up an [`RxJsKind`] by its marker token string (without
    /// the trailing colon). Returns `None` for unknown tokens.
    fn from_token(token: &str) -> Option<RxJsKind> {
        match token {
            "Φobs" => Some(Self::Observable),
            "Φsubject" => Some(Self::Subject),
            "ΦpipeRx" => Some(Self::PipeRx),
            "Φmap" => Some(Self::Map),
            "Φtap" => Some(Self::Tap),
            "Φfilter" => Some(Self::Filter),
            "Φcatch" => Some(Self::Catch),
            "Φfinalize" => Some(Self::Finalize),
            "Φdelay" => Some(Self::Delay),
            "Φcombine" => Some(Self::Combine),
            "Φshare" => Some(Self::Share),
            "Φto" => Some(Self::To),
            "Φwith" => Some(Self::With),
            "Φscan" => Some(Self::Scan),
            "Φdistinct" => Some(Self::Distinct),
            "Φretry" => Some(Self::Retry),
            _ => None,
        }
    }

    /// Returns the token string (without trailing `:`) for a given kind.
    fn token(self) -> &'static str {
        match self {
            Self::Observable => "Φobs",
            Self::Subject => "Φsubject",
            Self::PipeRx => "ΦpipeRx",
            Self::Map => "Φmap",
            Self::Tap => "Φtap",
            Self::Filter => "Φfilter",
            Self::Catch => "Φcatch",
            Self::Finalize => "Φfinalize",
            Self::Delay => "Φdelay",
            Self::Combine => "Φcombine",
            Self::Share => "Φshare",
            Self::To => "Φto",
            Self::With => "Φwith",
            Self::Scan => "Φscan",
            Self::Distinct => "Φdistinct",
            Self::Retry => "Φretry",
        }
    }
}

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// The kind of RxJS subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Subject,
    BehaviorSubject,
    ReplaySubject,
    AsyncSubject,
}

impl std::fmt::Display for SubjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Subject => write!(f, "Subject"),
            Self::BehaviorSubject => write!(f, "BehaviorSubject"),
            Self::ReplaySubject => write!(f, "ReplaySubject"),
            Self::AsyncSubject => write!(f, "AsyncSubject"),
        }
    }
}

/// A single observable field declaration.
#[derive(Debug, Clone)]
pub struct ObservableDecl {
    pub name: String,
    pub source: Option<String>, // "http.get", "of", "from", etc.
    pub type_param: Option<String>,
}

/// A single subject instantiation.
#[derive(Debug, Clone)]
pub struct SubjectDecl {
    pub name: String,
    pub kind: SubjectKind,
    pub initial_value: Option<String>,
    pub type_param: Option<String>,
}

/// A single pipe operator in a chain.
#[derive(Debug, Clone)]
pub struct PipeOperator {
    pub kind: RxJsKind,
    pub operator_name: String,
    pub arg_summary: Option<String>, // High fidelity only
}

/// A pipe chain attached to an observable field.
#[derive(Debug, Clone)]
pub struct PipeChain {
    pub owner: String,
    pub operators: Vec<PipeOperator>,
}

/// A static combinator declaration (combineLatest, forkJoin, etc.).
#[derive(Debug, Clone)]
pub struct CombinatorDecl {
    pub kind: RxJsKind,
    pub name: String,
    pub args: Vec<String>,
}

/// The complete RxJS shape extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct RxShape {
    pub observables: Vec<ObservableDecl>,
    pub subjects: Vec<SubjectDecl>,
    pub pipes: Vec<PipeChain>,
    pub combinators: Vec<CombinatorDecl>,
}

impl RxShape {
    /// Returns `true` if there are no RxJS artifacts to emit.
    pub fn is_empty(&self) -> bool {
        self.observables.is_empty()
            && self.subjects.is_empty()
            && self.pipes.is_empty()
            && self.combinators.is_empty()
    }

    /// Render the full `Φ RxJS Meta` block at the given fidelity.
    pub fn render(&self, fidelity: Fidelity) -> String {
        self.render_with_config(fidelity, None)
    }

    /// Render the full `Φ RxJS Meta` block at the given fidelity,
    /// honoring the `min_pipe_operators` config (when provided).
    ///
    /// `min_pipe_operators` suppresses `ΦpipeRx:` blocks with fewer
    /// than N operators (default 2) to prevent noise from trivial
    /// single-operator chains.
    pub fn render_with_config(
        &self,
        fidelity: Fidelity,
        min_pipe_operators: Option<usize>,
    ) -> String {
        if self.is_empty() {
            return String::new();
        }

        let min_ops = min_pipe_operators.unwrap_or(2);

        let mut s = String::new();
        s.push_str("// --- Φ RxJS Meta ---\n");

        // Observable fields (all fidelities)
        for obs in &self.observables {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φobs:{}\n", obs.name));
                }
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    if let Some(ref src) = obs.source {
                        s.push_str(&format!("  Φobs:{} → {}\n", obs.name, src));
                    } else {
                        s.push_str(&format!("  Φobs:{}\n", obs.name));
                    }
                }
            }
        }

        // Subject declarations (all fidelities)
        for subj in &self.subjects {
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  Φsubject:{}\n", subj.name));
                }
                Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    if let Some(ref iv) = subj.initial_value {
                        s.push_str(&format!(
                            "  Φsubject:{} = new {}({})\n",
                            subj.name, subj.kind, iv
                        ));
                    } else {
                        s.push_str(&format!(
                            "  Φsubject:{} = new {}<{}>()\n",
                            subj.name,
                            subj.kind,
                            subj.type_param.as_deref().unwrap_or("?")
                        ));
                    }
                }
            }
        }

        // Pipe chains (suppressed when below min_pipe_operators).
        for pipe in &self.pipes {
            if pipe.operators.len() < min_ops {
                continue;
            }
            match fidelity {
                Fidelity::Low => {
                    s.push_str(&format!("  ΦpipeRx:{}\n", pipe.owner));
                }
                Fidelity::Medium => {
                    let ops: Vec<&str> = pipe
                        .operators
                        .iter()
                        .map(|op| op.operator_name.as_str())
                        .collect();
                    s.push_str(&format!("  ΦpipeRx:{} = {}\n", pipe.owner, ops.join(" → ")));
                }
                Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
                    let mut pipe_line = format!("  ΦpipeRx:{} = pipe(\n", pipe.owner);
                    for (i, op) in pipe.operators.iter().enumerate() {
                        let comma = if i < pipe.operators.len() - 1 {
                            ","
                        } else {
                            ""
                        };
                        if let Some(ref arg) = op.arg_summary {
                            pipe_line
                                .push_str(&format!("    {}:{}{}\n", op.operator_name, arg, comma));
                        } else {
                            pipe_line.push_str(&format!("    {}{}\n", op.operator_name, comma));
                        }
                    }
                    pipe_line.push_str("  )");
                    s.push_str(&pipe_line);
                    s.push('\n');
                }
            }
        }

        // Static combinators
        for comb in &self.combinators {
            s.push_str(&format!(
                "  Φcombine:{} {}\n",
                comb.name,
                comb.args.join(", ")
            ));
        }

        s
    }
}

// ---------------------------------------------------------------------------
// Detection — import gate
// ---------------------------------------------------------------------------

/// Check whether the source file has RxJS imports.
/// Returns true if the file imports from `rxjs` or `rxjs/operators`.
pub fn has_rxjs_imports(source: &str) -> bool {
    // Simple string-based import scan (consistent with detect.rs approach).
    // We check for the most common RxJS import patterns.
    source.contains("from 'rxjs'")
        || source.contains("from \"rxjs\"")
        || source.contains("from 'rxjs/operators'")
        || source.contains("from \"rxjs/operators\"")
        || source.contains("from 'rxjs/internal'")
        || source.contains("from \"rxjs/internal\"")
}

mod extract;
mod pipes;

pub use extract::extract_rx_shape;

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

/// Expand every recognised RxJS `Φ` marker in a line back to its
/// human-readable form. Used by the decompressor.
///
/// This is chained into the existing Angular `expand_phi_in_line` in
/// `markers.rs` via the [`PHI_EXPANDERS`](crate::angular_meta::phi::PHI_EXPANDERS)
/// registry.
pub fn expand_phi_in_line(line: &str) -> String {
    crate::angular_meta::phi::expand_phi_in_line::<RxJsKind>(line)
}

/// Expand a single RxJS `Φ` marker token to its human-readable form.
/// Returns `None` for unknown markers.
pub fn expand_phi(token: &str) -> Option<&'static str> {
    crate::angular_meta::phi::expand_phi::<RxJsKind>(token)
}

#[cfg(test)]
#[path = "../tests/angular_meta/rx.rs"]
mod tests;
