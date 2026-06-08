// src/ir/render.rs
//
// IR → Human-readable text rendering.
//
// Phase A (Phase E in spec): renders IR instructions to backward-compatible
// compressed text output. Fidelity controls what information is included.
// Same IR → different fidelity → different output.
//
// This produces output that is byte-identical (or close) to the existing
// `compress_code_context` tool, enabling backward compatibility.

use crate::compression::Fidelity;

/// Render IR instructions (as positional tuples) to human-readable text.
/// Fidelity controls what information is included, not the compilation.
/// Same IR → different fidelity → different output.
pub fn ir_to_text(instructions: &[Vec<String>], fidelity: Fidelity) -> String {
    let mut output = String::new();
    let mut current_class: bool = false;

    for insn in instructions {
        if insn.is_empty() {
            continue;
        }
        match insn[0].as_str() {
            "DEF_C" => {
                let name = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                match fidelity {
                    Fidelity::Low => {
                        if current_class {
                            output.push(';');
                        }
                        output.push_str(&format!("$c {}", name));
                        current_class = true;
                    }
                    Fidelity::Medium | Fidelity::High => {
                        if current_class {
                            output.push('\n');
                        }
                        output.push_str(&format!("class {} {{\n", name));
                        current_class = true;
                    }
                }
            }
            "DEF_M" => {
                let name = insn.get(3).map(|s| s.as_str()).unwrap_or("?");
                let indent = match fidelity {
                    Fidelity::High => "  ",
                    _ => "",
                };
                if fidelity == Fidelity::Low {
                    output.push_str(&format!("{}();", name));
                } else {
                    output.push_str(&format!("{}{}()", indent, name));
                }
            }
            "SIG" => {
                let param_name = insn.get(4).map(|s| s.as_str()).unwrap_or("?");
                let type_op = insn.get(3).map(|s| s.as_str()).unwrap_or("$v");
                output.push_str(&format!("{}:{},", param_name, type_op));
            }
            "RET" => {
                let type_op = insn.get(2).map(|s| s.as_str()).unwrap_or("$v");
                output.push_str(&format!("):{}", type_op));
                if fidelity != Fidelity::Low {
                    output.push('\n');
                }
            }
            "FLAGS" => {
                let flags: Vec<&str> = insn[2..].iter().map(|s| s.as_str()).collect();
                let markers = flags_to_markers(&flags);
                match fidelity {
                    Fidelity::Low | Fidelity::Medium => {
                        output.push_str(&format!(" {}", markers.join(" ")));
                    }
                    Fidelity::High => {
                        output.push_str(&format!(" {{ {} }}", markers.join(" ")));
                    }
                }
            }
            "IMP" => {
                let module = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                let named = insn.get(3).map(|s| s.as_str()).unwrap_or("*");
                match fidelity {
                    Fidelity::Low => {
                        output.push_str(&format!("$im {}.$fm{};", named, module));
                    }
                    Fidelity::Medium | Fidelity::High => {
                        output.push_str(&format!(
                            "import {{ {} }} from '{}';\n",
                            named, module
                        ));
                    }
                }
            }
            "DEF_F" => {
                let name = insn.get(3).map(|s| s.as_str()).unwrap_or("?");
                match fidelity {
                    Fidelity::Low => {
                        output.push_str(&format!("{};", name));
                    }
                    Fidelity::Medium | Fidelity::High => {
                        output.push_str(&format!("  {};\n", name));
                    }
                }
            }
            "DEF_I" => {
                let name = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                match fidelity {
                    Fidelity::Low => {
                        output.push_str(&format!("$if {};", name));
                    }
                    Fidelity::Medium | Fidelity::High => {
                        output.push_str(&format!("interface {} {{}}\n", name));
                    }
                }
            }
            "EXT" => {
                let parent = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                output.push_str(&format!(" $x {}", parent));
            }
            "IMPL" => {
                let iface = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                output.push_str(&format!(" $m {}", iface));
            }
            "FIELD_T" => {
                let type_op = insn.get(2).map(|s| s.as_str()).unwrap_or("$v");
                output.push_str(&format!(":{}", type_op));
            }
            "INJECTS" => {
                let deps: Vec<&str> = insn[2..].iter().map(|s| s.as_str()).collect();
                output.push_str(&format!(" // injects: {}", deps.join(", ")));
            }
            "TYPE" => {
                let alias = insn.get(1).map(|s| s.as_str()).unwrap_or("?");
                let original = insn.get(2).map(|s| s.as_str()).unwrap_or("?");
                output.push_str(&format!("$ty {}={}", alias, original));
            }
            "FLAGS_C" => {
                let flags: Vec<&str> = insn[2..].iter().map(|s| s.as_str()).collect();
                for flag in &flags {
                    match *flag {
                        "EXPORT" => output.push_str("$e "),
                        "ABSTRACT" => output.push_str("abstract "),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    output
}

/// Convert IR flags back to ⊕ markers for backward-compatible display.
fn flags_to_markers(flags: &[&str]) -> Vec<String> {
    flags
        .iter()
        .map(|f| match *f {
            "IF" => "⊕guard".to_string(),
            "LOOP" => "⊕loop".to_string(),
            "RET" => "⊕⇒".to_string(),
            "THROW" => "⊕!".to_string(),
            "ASYNC" => "$a".to_string(),
            "GEN" => "⊕gen".to_string(),
            other => format!("⊕{}", other),
        })
        .collect()
}

#[cfg(test)]
#[path = "../tests/ir/render.rs"]
mod tests;