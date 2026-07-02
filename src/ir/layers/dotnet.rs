// src/ir/layers/dotnet.rs
//
// .NET / C# Meta-Layer (Layer 3) for the IR compiler.
// Wraps the existing dotnet_meta module's attribute extraction logic
// and emits CoreOp instructions instead of Φ marker text.
//
// The .NET Meta-Layer is purely additive: it never modifies the
// existing Core IR output. It only appends meta-instructions that
// describe .NET-specific class roles (Controller, DbContext, Hub, etc.)
// and their metadata.
//
// Phase 2: TYPE ops use abbreviated @-prefixed notation (e.g., @ctrl,
// @ef, @hub, @map, @svc, @di).

use super::MetaLayer;
use crate::dotnet_meta;
use crate::compression::Fidelity;
use crate::ir::opcodes::CoreOp;

/// .NET / C# meta-layer (Layer 3).
/// Extracts .NET framework attributes from source and emits CoreOp
/// instructions representing ASP.NET Core, EF Core, SignalR, AutoMapper,
/// and general .NET patterns.
pub struct DotNetMetaLayer;

impl DotNetMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DotNetMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaLayer for DotNetMetaLayer {
    fn name(&self) -> &str {
        "dotnet"
    }

    fn extract(
        &mut self,
        source: &str,
        class_captures: &[String],
        fidelity: Fidelity,
    ) -> Vec<CoreOp> {
        let mut ops = Vec::new();

        // Run the existing dotnet_meta text pipeline
        let meta_block = dotnet_meta::run_meta_layer(source, class_captures, fidelity);

        // If there's no .NET content, return empty
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
/// All TypeAlias aliases use abbreviated @-prefixed notation.
fn parse_phi_line(line: &str) -> Option<Vec<CoreOp>> {
    let line = line.trim();

    // Remove leading Φ marker if present
    let content = line.strip_prefix('Φ')?;

    // Determine the type based on the prefix before ':'
    let (prefix, rest) = content.split_once(':')?;

    match prefix {
        "ctrl" => {
            // Controller: Φctrl:ClassName [route]
            let (class_name, _metadata) = split_metadata(rest);
            Some(vec![CoreOp::TypeAlias(
                "@ctrl".to_string(),
                class_name.to_string(),
            )])
        }
        "api" => {
            // ApiController: Φapi:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@api".to_string(),
                class_name.to_string(),
            )])
        }
        "action" => {
            // Action: Φaction:Verb Name(params) → ReturnType
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                let class_name = parts[0].trim();
                let action_sig = parts[1].trim();
                Some(vec![CoreOp::TypeAlias(
                    "@action".to_string(),
                    format!("{}.{}", class_name, action_sig),
                )])
            } else {
                None
            }
        }
        "model" => {
            // Model: Φmodel:ModelName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@model".to_string(),
                class_name.to_string(),
            )])
        }
        "auth" => {
            // Authorize: Φauth:Policy
            let policy = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@auth".to_string(),
                policy.to_string(),
            )])
        }
        "ef" => {
            // DbContext: Φef:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@ef".to_string(),
                class_name.to_string(),
            )])
        }
        "dbset" => {
            // DbSet: Φdbset:Name
            let name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@dbset".to_string(),
                name.to_string(),
            )])
        }
        "entity" => {
            // Entity: Φentity:Name { fields }
            let (name, _metadata) = split_metadata(rest);
            Some(vec![CoreOp::TypeAlias(
                "@entity".to_string(),
                name.to_string(),
            )])
        }
        "rel" => {
            // Relationship: Φrel:Name → Target
            let rel = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@rel".to_string(),
                rel.to_string(),
            )])
        }
        "cfg" => {
            // Config: Φcfg:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@cfg".to_string(),
                class_name.to_string(),
            )])
        }
        "map" => {
            // Mapper: Φmap:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@map".to_string(),
                class_name.to_string(),
            )])
        }
        "mapfrom" => {
            // MapFrom: Φmapfrom:Source → Dest
            let mapping = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@mapfrom".to_string(),
                mapping.to_string(),
            )])
        }
        "hub" => {
            // Hub: Φhub:ClassName [ClientInterface]
            let (class_name, _metadata) = split_metadata(rest);
            Some(vec![CoreOp::TypeAlias(
                "@hub".to_string(),
                class_name.to_string(),
            )])
        }
        "method" => {
            // Hub method: Φmethod:Name(params) → target
            let method = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@method".to_string(),
                method.to_string(),
            )])
        }
        "svc" => {
            // Service: Φsvc:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@svc".to_string(),
                class_name.to_string(),
            )])
        }
        "di" => {
            // DI registration: Φdi:Service → Registration
            let di = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@di".to_string(),
                di.to_string(),
            )])
        }
        "valid" => {
            // Validator: Φvalid:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@valid".to_string(),
                class_name.to_string(),
            )])
        }
        "identity" => {
            // Identity: Φidentity:ClassName
            let class_name = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@identity".to_string(),
                class_name.to_string(),
            )])
        }
        "jwt" => {
            // JWT: Φjwt:Config
            let config = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@jwt".to_string(),
                config.to_string(),
            )])
        }
        "cache" => {
            // Cache: Φcache:Type
            let cache_type = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@cache".to_string(),
                cache_type.to_string(),
            )])
        }
        "log" => {
            // Logging: Φlog:Pattern
            let pattern = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@log".to_string(),
                pattern.to_string(),
            )])
        }
        "metric" => {
            // Metric: Φmetric:Provider
            let provider = rest.trim();
            Some(vec![CoreOp::TypeAlias(
                "@metric".to_string(),
                provider.to_string(),
            )])
        }
        _ => None,
    }
}

/// Split a Φ line into class name and metadata portion.
/// Format: "ClassName key=value"
fn split_metadata(input: &str) -> (&str, &str) {
    let input = input.trim();
    if let Some((name, meta)) = input.split_once(' ') {
        (name.trim(), meta.trim())
    } else {
        (input, "")
    }
}