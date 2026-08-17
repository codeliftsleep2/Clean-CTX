// src/ir/render_llm.rs
//
// Phase 1: LLM-Optimized Hierarchical IR Renderer.
//
// Converts a `HierarchicalIR` into compact, LLM-friendly text using
// single-character structural markers. This is a STATELESS projection:
// all stateful operations (deltas, persistence, context tracking) operate
// on the flat `CoreOp[]` stream internally. The rendered text is generated
// fresh each time from the latest IR.
//
// Key design decisions:
//   - Uses class/method **names** only, never internal alias IDs (C1, M1)
//   - Overloaded methods disambiguated with `+N` (parameter count)
//   - Fidelity controls field layout (Low = space-separated, Medium/High = one-per-line)
//   - Meta-layer `@` annotations always shown regardless of fidelity
//   - The `// SCHEMA v2` header opens every output with the legend table
//
// Notation reference (also in the SCHEMA v2 header):
//   @=meta  X=extends  I=implements  F=field  M=method
//   $=import  →=scope  fl:=flags  cl:=class-flags  P=pattern  T=type-alias

use std::collections::HashMap;
use crate::compression::Fidelity;
use super::hierarchical::{HierarchicalIR, ClassNode, PatternEntry};

/// Render a `HierarchicalIR` into compact LLM-optimized text.
///
/// The output is a single string with newline-separated lines. The format
/// is designed to be maximally informative per token for LLM consumption.
///
/// # Fidelity behavior
///
/// | Aspect | Low | Medium | High | Edit | Verbatim |
/// |--------|-----|--------|------|------|----------|
/// | Fields | Space-separated, same line | One per line | One per line | One per line | One per line |
/// | Methods | Minimal (name + flags) | Params shown | Params shown | Params shown + verbatim bodies | Full raw source |
/// | Meta `@` | Always | Always | Always | Always | Always |
/// | `X`/`I` | Always | Always | Always | Always | Always |
/// | Patterns | Always | Always | Always | Always | Always |
/// | Imports | Always | Always | Always | Always | Always |
/// | Type aliases | Always | Always | Always | Always | Always |
///
/// # Overloaded method disambiguation
///
/// When a class has multiple methods with the same name, `+N` is appended
/// where N is the parameter count (e.g., `M find(+1)`, `M find(+3)`).
pub fn render_hierarchical_for_llm(hir: &HierarchicalIR, fidelity: Fidelity) -> String {
    let mut output = String::new();

    // ── SCHEMA v2 header ──
    output.push_str("// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias\n");

    // ── Classes ──
    for class in &hir.classes {
        render_class(&mut output, class, fidelity);
    }

    // ── Imports ──
    for imp in &hir.imports {
        if imp.len() >= 3 {
            // Format: $ alias module [named]
            let alias = &imp[0];
            let module = &imp[1];
            let named = &imp[2];
            if named == "*" || named.is_empty() {
                output.push_str(&format!("$ {} {}\n", alias, module));
            } else {
                output.push_str(&format!("$ {} {} [{}]\n", alias, module, named));
            }
        }
    }

    // ── Type aliases ──
    for ta in &hir.type_aliases {
        if ta.len() >= 2 {
            let alias = &ta[0];
            let original = &ta[1];
            output.push_str(&format!("T {} = {}\n", alias, original));
        }
    }

    output
}

/// Render a single class node.
fn render_class(output: &mut String, class: &ClassNode, fidelity: Fidelity) {
    // Class boundary
    output.push_str(&format!("// ── {} ──\n", class.name));

    // Class-level patterns (e.g., EMPTY_CTOR)
    for pat in &class.patterns {
        render_pattern(output, pat);
    }

    // Class-level flags
    if let Some(flags) = &class.class_flags {
        if !flags.is_empty() {
            output.push_str(&format!("cl: {}\n", flags.join(" ")));
        }
    }

    // Extends
    if let Some(parent) = &class.extends {
        output.push_str(&format!("X {}\n", parent));
    }

    // Implements
    if !class.implements.is_empty() {
        output.push_str(&format!("I {}\n", class.implements.join(" ")));
    }

    // Fields — layout depends on fidelity
    render_fields(output, class, fidelity);

    // Methods — with overload disambiguation
    render_methods(output, class, fidelity);
}

/// Render fields for a class.
///
/// Low fidelity: space-separated on one line.
/// Medium/High: one per line.
fn render_fields(output: &mut String, class: &ClassNode, fidelity: Fidelity) {
    if class.fields.is_empty() {
        return;
    }

    match fidelity {
        Fidelity::Low => {
            // Space-separated on one line
            let field_strs: Vec<String> = class.fields.iter().map(|f| {
                if let Some(ft) = &f.field_type {
                    format!("{}:{}", f.name, ft)
                } else {
                    f.name.clone()
                }
            }).collect();
            output.push_str(&format!("F {}\n", field_strs.join(" ")));
        }
        Fidelity::Medium | Fidelity::High | Fidelity::Edit | Fidelity::Verbatim => {
            // One per line
            for field in &class.fields {
                if let Some(ft) = &field.field_type {
                    output.push_str(&format!("F {}:{}\n", field.name, ft));
                } else {
                    output.push_str(&format!("F {}\n", field.name));
                }
            }
        }
    }
}

/// Render methods for a class with overload disambiguation.
///
/// First pass: count occurrences of each method name.
/// Second pass: emit with `+N` for duplicates.
fn render_methods(output: &mut String, class: &ClassNode, fidelity: Fidelity) {
    if class.methods.is_empty() {
        return;
    }

    // First pass: count method name occurrences
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for method in &class.methods {
        *name_counts.entry(&method.name).or_insert(0) += 1;
    }

    // Second pass: emit methods
    let mut name_indices: HashMap<&str, usize> = HashMap::new();
    for method in &class.methods {
        let count = name_counts[&method.name.as_str()];
        let idx = name_indices.entry(&method.name).or_insert(0);
        *idx += 1;

        // Method-level patterns first
        for pat in &method.patterns {
            render_pattern(output, pat);
        }

        // Method declaration
        if count > 1 {
            // Overloaded: disambiguate with +N (parameter count)
            let param_count = method.params.len();
            output.push_str(&format!("M {}(+{})", method.name, param_count));
        } else {
            output.push_str(&format!("M {}", method.name));
        }

        // Method body (params, return type, flags)
        let has_params = !method.params.is_empty();
        let has_return = method.return_type.is_some();
        let has_flags = method.flags.as_ref().is_some_and(|f| !f.is_empty());

        if has_params || has_return || has_flags {
            output.push_str("  →");

            // Params (shown in Medium/High, hidden in Low unless overloaded)
            if has_params && (fidelity != Fidelity::Low || count > 1) {
                let param_strs: Vec<String> = method.params.iter().map(|p| {
                    if p.len() >= 3 {
                        format!("{}:{}", p[2], p[1])
                    } else if p.len() >= 2 {
                        format!("{}:{}", p[0], p[1])
                    } else {
                        p[0].clone()
                    }
                }).collect();
                output.push_str(&format!(" p:{}", param_strs.join(" ")));
            }

            // Return type
            if let Some(rt) = &method.return_type {
                output.push_str(&format!(" → {}", rt));
            }

            // Flags
            if let Some(flags) = &method.flags {
                if !flags.is_empty() {
                    output.push_str(&format!(" fl:{}", flags.join(",")));
                }
            }
        }

        // Control-flow metadata at High fidelity (Gap 1 fix)
        if fidelity == Fidelity::High && !method.control_flow.is_empty() {
            let cf_strs: Vec<String> = method.control_flow.iter()
                .map(|cf| {
                    if cf.len() >= 2 {
                        format!("{}:{}", cf[0], cf[1])
                    } else {
                        cf.join(":")
                    }
                })
                .collect();
            output.push_str(&format!(" cf:{}", cf_strs.join(",")));
        }

        // Data-flow metadata at High fidelity (Gap 1 fix).
        // Rendered as `df:reads:config,writes:users` — same inline pattern
        // as control-flow so the LLM sees semantic read/write pairs.
        if fidelity == Fidelity::High && !method.data_flow.is_empty() {
            let df_strs: Vec<String> = method.data_flow.iter()
                .map(|df| {
                    if df.len() >= 2 {
                        format!("{}:{}", df[0], df[1])
                    } else {
                        df.join(":")
                    }
                })
                .collect();
            output.push_str(&format!(" df:{}", df_strs.join(",")));
        }

        // Side-effect annotation at High fidelity (Gap 1 fix).
        // e.g. `se:mutation` — quickly tells the LLM whether a method is
        // pure, performs I/O, mutates state, is async, or is transactional.
        if fidelity == Fidelity::High {
            if let Some(se) = &method.side_effect {
                output.push_str(&format!(" se:{}", se));
            }
        }

        // Execution-context annotation at High fidelity (Gap 1 fix).
        // e.g. `ec:async` — tells the agent the runtime context without
        // a full body read.
        if fidelity == Fidelity::High {
            if let Some(ec) = &method.execution_context {
                output.push_str(&format!(" ec:{}", ec));
            }
        }

        // Verbatim method body at Edit fidelity (byte-exact for replace_in_file)
        if fidelity == Fidelity::Edit {
            if let Some(body) = &method.body {
                output.push('\n');
                output.push_str(body);
                // Ensure trailing newline before next method
                if !body.ends_with('\n') {
                    output.push('\n');
                }
            }
        }

        output.push('\n');
    }
}

/// Render a pattern entry.
fn render_pattern(output: &mut String, pat: &PatternEntry) {
    if pat.args.is_empty() {
        output.push_str(&format!("P {}\n", pat.name));
    } else {
        let args_str = pat.args.join(" ");
        output.push_str(&format!("P {} {}\n", pat.name, args_str));
    }
}

#[cfg(test)]
#[path = "../tests/ir/render_llm.rs"]
mod tests;