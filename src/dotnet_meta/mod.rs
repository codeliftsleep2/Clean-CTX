// src/dotnet_meta/mod.rs
//
// .NET / C# Meta-Layer — Tier 1 (Attribute Extraction) + Tier 2 (Layer Bundling).
//
// The Meta-Layer is **purely additive**: it never modifies the existing
// C# compaction output. It only appends a `Φ` block below the existing
// compacted class entry. Existing users see no change; .NET users get
// enriched output with ASP.NET Core, EF Core, SignalR, AutoMapper, and
// other framework context.
//
// # Module structure
//
// - `detect`         : .NET file detection heuristic
// - `markers`        : `Φ` marker construction & expansion
// - `aspnet`         : ASP.NET Core (Controllers, Minimal APIs)
// - `efcore`         : Entity Framework Core
// - `signalr`        : SignalR (hubs, streaming, client interfaces)
// - `automapper`     : AutoMapper profiles
// - `serialization`  : JSON serialization attributes
// - `general`        : Services, DI, validation, caching, identity, logging
// - `graph`          : cross-file dependency graph (DI, endpoints, hubs)
// - `graph_state`    : McpState integration
// - `footer`         : `§ΦMAP` workspace footer formatter
// - (this file)      : Public surface, `MetaBlock` struct, `run_meta_layer`

pub mod aspnet;
pub mod automapper;
pub(crate) mod detect;
pub mod efcore;
pub mod footer;
pub mod general;
pub mod graph;
pub mod graph_state;
pub(crate) mod markers;
pub mod serialization;
pub mod signalr;

use crate::compression::Fidelity;

/// The Meta-Layer output for a single `.cs` file.
///
/// `None` means "not a .NET framework file" — the caller should not emit any
/// Φ block at all (zero overhead, byte-identical to non-.NET output).
///
/// `Some(block)` means ".NET file" — the caller should append the
/// Φ block lines below the existing compacted class entry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MetaBlock {
    /// One `Φ` line per .NET-bearing class, in document order.
    /// Each line is the fully-formatted marker line.
    /// Already newline-separated; caller is responsible for the
    /// surrounding `// --- Φ .NET Meta ---` header.
    pub lines: Vec<String>,
}

impl MetaBlock {
    /// Returns `true` if there are no Φ lines to emit (i.e. the
    /// caller should skip the entire `// --- Φ .NET Meta ---` block).
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
        s.push_str("// --- Φ .NET Meta ---\n");
        for line in &self.lines {
            s.push_str(line);
            s.push('\n');
        }
        s
    }
}

/// Run the .NET Meta-Layer pass on a single `.cs` file's raw source
/// text. Returns `None` when the file is not a .NET framework file
/// (no ASP.NET/EF/SignalR/etc. patterns present), and `Some(MetaBlock)`
/// when at least one class carries a recognised .NET framework pattern.
///
/// # Arguments
///
/// - `source_code`    : the full source text of the file being compressed
/// - `class_captures` : the slice texts of each class capture
///   (in document order, already sorted by `run_capture_pipeline`)
/// - `fidelity`       : fidelity level (controls verbosity):
///   - `Fidelity::Low`    → emit only class-level summaries
///   - `Fidelity::Medium` → add method-level details
///   - `Fidelity::High`   → emit everything including field-level attributes
pub fn run_meta_layer(
    source_code: &str,
    class_captures: &[String],
    fidelity: Fidelity,
) -> Option<MetaBlock> {
    // Tier 0 (detection): is this a .NET framework file at all?
    if !detect::is_dotnet_file(source_code) {
        return None;
    }

    // Tier 1 (extraction): walk each class capture and emit Φ lines.
    let mut block = MetaBlock::default();
    for raw_class in class_captures {
        // ASP.NET Core extraction
        if let Some(result) = aspnet::extract_aspnet(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }

        // EF Core extraction
        if let Some(result) = efcore::extract_efcore(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }

        // SignalR extraction
        if let Some(result) = signalr::extract_signalr(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }

        // AutoMapper extraction
        if let Some(result) = automapper::extract_automapper(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }

        // Serialization extraction
        if let Some(result) = serialization::extract_serialization(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }

        // General DI / validation / identity / caching / logging extraction
        if let Some(result) = general::extract_general(raw_class, fidelity) {
            block.lines.extend(result.lines);
        }
    }

    if block.is_empty() {
        // .NET file but no framework patterns on any class. Be
        // conservative — do not emit a Φ block header.
        return None;
    }

    Some(block)
}

#[cfg(all(test, feature = "dotnet"))]
#[path = "../tests/dotnet_meta/mod.rs"]
mod tests;

// ── Meta-Layer Integration ────────────────────────────────────────────

#[cfg(feature = "dotnet")]
pub struct DotNetMetaLayer;

#[cfg(feature = "dotnet")]
impl DotNetMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "dotnet")]
impl Default for DotNetMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "dotnet")]
impl crate::layers::meta::MetaLayer for DotNetMetaLayer {
    fn name(&self) -> &'static str {
        "dotnet"
    }

    fn is_applicable(
        &self,
        source: &str,
        _path: &std::path::Path,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> bool {
        detect::is_dotnet_file(source)
    }

    fn enrich(
        &self,
        source: &str,
        class_captures: &[String],
        fidelity: crate::compression::Fidelity,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> Option<crate::layers::meta::MetaLayerOutput> {
        // Run the meta-layer pipeline using the real source code and
        // class captures directly — no DefClass round-trip.
        let block = run_meta_layer(source, class_captures, fidelity)?;
        if block.is_empty() {
            return None;
        }
        Some(crate::layers::meta::MetaLayerOutput {
            layer_name: self.name(),
            rendered: block.render(),
            angular_block: None,
            spring_block: None,
            dotnet_block: Some(block),
        })
    }
}

#[cfg(not(feature = "dotnet"))]
pub struct DotNetMetaLayer;

#[cfg(not(feature = "dotnet"))]
impl DotNetMetaLayer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "dotnet"))]
impl Default for DotNetMetaLayer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dotnet"))]
impl crate::layers::meta::MetaLayer for DotNetMetaLayer {
    fn name(&self) -> &'static str {
        "dotnet"
    }

    fn is_applicable(
        &self,
        _source: &str,
        _path: &std::path::Path,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> bool {
        false
    }

    fn enrich(
        &self,
        _source: &str,
        _class_captures: &[String],
        _fidelity: crate::compression::Fidelity,
        _config: Option<&crate::config::CleanCtxConfig>,
    ) -> Option<crate::layers::meta::MetaLayerOutput> {
        // No-op when feature is disabled
        None
    }
}
