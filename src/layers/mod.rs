// src/layers/mod.rs
//
// Modular Language & Meta Layers — trait-based abstraction for optional
// language support and framework meta-layers.
//
// Each language/meta-layer is gated by a Cargo feature:
//   - typescript, csharp, rust, java (language layers)
//   - angular, spring_boot (meta layers)
//
// The LayerRegistry collects all enabled layers at startup and dispatches
// to them at runtime. When a feature is disabled, the corresponding
// tree-sitter grammar is not compiled, linked, or included in the binary.

pub mod language;
pub mod meta;
pub mod registry;

pub use registry::LayerRegistry;
