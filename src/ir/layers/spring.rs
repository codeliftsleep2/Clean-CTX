// src/ir/layers/spring.rs
//
// FAANG audit: NEW — Spring Boot Meta-Layer (Layer 3)
// Wraps the existing spring_meta module's annotation extraction logic
// and emits CoreOp instructions instead of Φ marker text.
//
// The Spring Meta-Layer is purely additive: it never modifies the
// existing Core IR output. It only appends meta-instructions that
// describe Spring-specific class roles (@RestController, @Service,
// @Repository, @Controller, @Configuration, @RequestMapping, etc.)
// and their metadata.
//
// Phase 2: TYPE ops use abbreviated @-prefixed notation instead of
// verbose SP_REST_ format. Each TypeAlias alias matches a
// single-character structural marker (e.g., @rest, @svc, @repo,
// @ctrl, @cfg, @comp, @bean, @wire, @val, @map).

use super::MetaLayer;
use crate::spring_meta;
use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;

/// Spring Boot meta-layer (Layer 3).
/// Extracts Spring annotations from source and emits CoreOp instructions
/// representing @RestController, @Service, @Repository, @Controller,
/// @Configuration, @RequestMapping, and their associated metadata.
pub struct SpringMetaLayer;

impl SpringMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpringMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLayer for SpringMetaLayer {
    fn name(&self) -> &str {
        "spring"
    }

    fn extract(
        &mut self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // Run the existing spring_meta text pipeline
        let meta_block = spring_meta::run_meta_layer(source, class_captures, fidelity);

        // If there's no Spring content, return empty
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
/// All TypeAlias aliases use abbreviated @-prefixed notation (e.g., @rest, @svc).
fn parse_phi_line(line: &str) -> Option<Vec<CoreOp>> {
    let line = line.trim();

    // Remove leading Φ marker if present
    let content = line.strip_prefix('Φ')?;

    // Determine the type based on the prefix before ':'
    let (prefix, rest) = content.split_once(':')?;

    match prefix {
        "rest" => {
            // @RestController: Φrest:ClassName map=[GET /api/users]
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                "@rest".to_string(),
                class_name.to_string(),
            ));

            if let Some(map_str) = parse_meta_value(metadata, "map") {
                let mappings: Vec<String> = map_str
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !mappings.is_empty() {
                    ops.push(CoreOp::TypeAlias(
                        "@map".to_string(),
                        format!("{}={}", class_name, mappings.join(",")),
                    ));
                }
            }

            Some(ops)
        }
        "svc" => {
            // @Service: Φsvc:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@svc".to_string(),
                class_name.to_string(),
            )])
        }
        "repo" => {
            // @Repository: Φrepo:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@repo".to_string(),
                class_name.to_string(),
            )])
        }
        "ctrl" => {
            // @Controller: Φctrl:ClassName map=[GET /api/users]
            let (class_name, metadata) = split_metadata(rest);
            let mut ops = Vec::new();

            ops.push(CoreOp::TypeAlias(
                "@ctrl".to_string(),
                class_name.to_string(),
            ));

            if let Some(map_str) = parse_meta_value(metadata, "map") {
                let mappings: Vec<String> = map_str
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !mappings.is_empty() {
                    ops.push(CoreOp::TypeAlias(
                        "@map".to_string(),
                        format!("{}={}", class_name, mappings.join(",")),
                    ));
                }
            }

            Some(ops)
        }
        "cfg" => {
            // @Configuration: Φcfg:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@cfg".to_string(),
                class_name.to_string(),
            )])
        }
        "comp" => {
            // @Component: Φcomp:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@comp".to_string(),
                class_name.to_string(),
            )])
        }
        "bean" => {
            // @Bean method: Φbean:ClassName methodName
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let method_name = parts[1].trim();
                Some(vec![CoreOp::TypeAlias(
                    "@bean".to_string(),
                    format!("{}.{}", class_name, method_name),
                )])
            } else {
                None
            }
        }
        "autowired" => {
            // @Autowired field: Φautowired:ClassName fieldName
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let field_name = parts[1].trim();
                Some(vec![CoreOp::TypeAlias(
                    "@wire".to_string(),
                    format!("{}.{}", class_name, field_name),
                )])
            } else {
                None
            }
        }
        "value" => {
            // @Value field: Φvalue:ClassName fieldName=expression
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let field_expr = parts[1].trim();
                let field_name = field_expr.split('=').next().unwrap_or(field_expr);
                Some(vec![CoreOp::TypeAlias(
                    "@val".to_string(),
                    format!("{}.{}", class_name, field_name),
                )])
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Split a Φ line into class name and metadata portion.
/// Format: "ClassName map=[GET /api/users]"
fn split_metadata(input: &str) -> (&str, &str) {
    let input = input.trim();
    if let Some((name, meta)) = input.split_once(' ') {
        (name.trim(), meta.trim())
    } else {
        (input, "")
    }
}

/// Parse a metadata value by key from a metadata string.
/// Metadata format: "key=value ...otherKey=value"
fn parse_meta_value<'a>(metadata: &'a str, key: &str) -> Option<&'a str> {
    for part in metadata.split(' ') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim() == key {
                return Some(v.trim());
            }
        }
    }
    None
}