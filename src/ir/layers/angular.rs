// src/ir/layers/angular.rs
//
// Phase F: Layered Encoding — Angular Meta-Layer (Layer 3)
// Wraps the existing angular_meta module's decorator extraction logic
// and emits CoreOp instructions instead of Φ marker text.
//
// The Angular Meta-Layer is purely additive: it never modifies the
// existing Core IR output. It only appends meta-instructions that
// describe Angular-specific class roles (Component, Injectable, etc.)
// and their metadata (selectors, injects, inputs, outputs).

use super::MetaLayer;
use crate::angular_meta;
use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;

/// Angular meta-layer (Layer 3).
/// Extracts Angular decorators from source and emits CoreOp instructions
/// representing @Component, @Injectable, @NgModule, @Directive, @Pipe,
/// and their associated metadata.
pub struct AngularMetaLayer;

impl AngularMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AngularMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLayer for AngularMetaLayer {
    fn name(&self) -> &str {
        "angular"
    }

    fn extract(
        &mut self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // F-44/F-45: The angular_meta pipeline emits Φ marker *text* via
        // `MetaBlock { lines: Vec<String> }`, which is then re-parsed by
        // `parse_phi_line`. This round-trip through text is a known design
        // debt (two parsers of the same shape). A future refactor should
        // make `run_meta_layer` return structured `Vec<AngularDecorators>`
        // directly. For now, the text round-trip is stable because the
        // Φ marker format is simple and unlikely to change within Phase 3.
        let meta_block = angular_meta::run_meta_layer(source, class_captures, fidelity);

        // If there's no Angular content, return empty
        let block = match meta_block {
            Some(b) => b,
            None => return ops,
        };

        for line in &block.lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(ops_from_line) = parse_phi_line(line) {
                ops.extend(ops_from_line);
            }
        }

        ops
    }
}

/// Parse a single Φ marker line and emit corresponding CoreOp instructions.
fn parse_phi_line(line: &str) -> Option<Vec<CoreOp>> {
    let line = line.trim();

    // Remove leading Φ marker if present
    let content = line.strip_prefix('Φ')?;

    // Determine the type based on the prefix before ':'
    let (prefix, rest) = content.split_once(':')?;

    match prefix {
        "cmp" => {
            // Component: Φcmp:ClassName sel=selector ...injects:SvcA,SvcB
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                format!("NG_COMPONENT_{}", class_name),
                class_name.to_string(),
            ));

            if let Some(sel) = parse_meta_value(metadata, "sel") {
                ops.push(CoreOp::TypeAlias(
                    format!("NG_SEL_{}", class_name),
                    sel.to_string(),
                ));
            }

            if let Some(injects_str) = parse_meta_value(metadata, "injects") {
                let deps: Vec<String> =
                    injects_str.split(',').map(|s| s.trim().to_string()).collect();
                if !deps.is_empty() {
                    ops.push(CoreOp::Injects(class_name.to_string(), deps));
                }
            }

            Some(ops)
        }
        "svc" => {
            // Service: Φsvc:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                format!("NG_SERVICE_{}", class_name),
                class_name.to_string(),
            )])
        }
        "mod" => {
            // Module: Φmod:ClassName decl:[X] imp:[Y] exp:[Z]
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                format!("NG_MODULE_{}", class_name),
                class_name.to_string(),
            ));

            for kind in &["decl", "imp", "exp"] {
                if let Some(values) = parse_meta_value(metadata, kind) {
                    let items: String = values
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .to_string();
                    ops.push(CoreOp::TypeAlias(
                        format!("NG_MODULE_{}_{}", class_name, kind),
                        items,
                    ));
                }
            }

            Some(ops)
        }
        "dir" => {
            // Directive: Φdir:ClassName sel=selector
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                format!("NG_DIRECTIVE_{}", class_name),
                class_name.to_string(),
            ));

            if let Some(sel) = parse_meta_value(metadata, "sel") {
                ops.push(CoreOp::TypeAlias(
                    format!("NG_SEL_{}", class_name),
                    sel.to_string(),
                ));
            }

            Some(ops)
        }
        "pipe" => {
            // Pipe: Φpipe:ClassName name=pipeName
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                format!("NG_PIPE_{}", class_name),
                class_name.to_string(),
            ));

            if let Some(pname) = parse_meta_value(metadata, "name") {
                ops.push(CoreOp::TypeAlias(
                    format!("NG_PIPE_NAME_{}", class_name),
                    pname.to_string(),
                ));
            }

            Some(ops)
        }
        "in" => {
            // Input: Φin:ClassName fieldName alias?
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let field_name = parts[1].trim();
                let alias = parts.get(2).map(|s| s.trim()).unwrap_or("");
                let input_id = format!("NG_INPUT_{}_{}", class_name, field_name);
                let value = if alias.is_empty() {
                    field_name.to_string()
                } else {
                    alias.to_string()
                };
                Some(vec![CoreOp::TypeAlias(input_id, value)])
            } else {
                None
            }
        }
        "out" => {
            // Output: Φout:ClassName fieldName alias?
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let field_name = parts[1].trim();
                let alias = parts.get(2).map(|s| s.trim()).unwrap_or("");
                let output_id = format!("NG_OUTPUT_{}_{}", class_name, field_name);
                let value = if alias.is_empty() {
                    field_name.to_string()
                } else {
                    alias.to_string()
                };
                Some(vec![CoreOp::TypeAlias(output_id, value)])
            } else {
                None
            }
        }
        "injects" => {
            // Constructor injects: Φinjects:TypeA,TypeB
            let types: Vec<String> = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !types.is_empty() {
                Some(vec![CoreOp::Injects("constructor".to_string(), types)])
            } else {
                None
            }
        }
        "model" => {
            // Signal model: Φmodel:ClassName fieldName alias?
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let field_name = parts[1].trim();
                let alias = parts.get(2).map(|s| s.trim()).unwrap_or("");
                let model_id = format!("NG_MODEL_{}_{}", class_name, field_name);
                let value = if alias.is_empty() {
                    field_name.to_string()
                } else {
                    alias.to_string()
                };
                Some(vec![CoreOp::TypeAlias(model_id, value)])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split a Φ line into class name and metadata portion.
/// Format: "ClassName sel=selector ...injects:SvcA,SvcB"
fn split_metadata(input: &str) -> (&str, &str) {
    let input = input.trim();
    if let Some((name, meta)) = input.split_once(' ') {
        (name.trim(), meta.trim())
    } else {
        (input, "")
    }
}

/// Parse a metadata value by key from a metadata string.
/// Metadata format: "key=value ...otherKey:value"
fn parse_meta_value<'a>(metadata: &'a str, key: &str) -> Option<&'a str> {
    for part in metadata.split(' ') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == key {
                return Some(v.trim());
            }
        }
        if let Some((k, v)) = part.split_once(':') {
            if k.trim() == key {
                return Some(v.trim());
            }
        }
    }
    None
}