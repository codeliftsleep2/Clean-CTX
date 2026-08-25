// src/decompression/markers.rs
//
// Backward-compatible re-export shim. Marker expansion used to live in
// this file (`expand_marker`, `expand_markers_in_line`); in Phase 2
// those functions move to `crate::compression::markers` so that the
// decompressor and the construction side (`markers::build_marker`)
// share one source of truth.
//
// This shim re-exports `expand_markers_in_line` (the only function the
// `Decompressor` actually used) so the existing call site keeps
// compiling without modification.
//
// Phase 1 of the Angular Meta-Layer also re-exports
// `expand_phi_in_line` so the `Decompressor` can expand the `Φ…`
// framework markers (`Φcmp:` → `@Component`, `Φsvc:` → `@Injectable`,
// etc.) alongside the existing `⊕…` behavior markers.
//
// Phase 3 (Meta-Layer Integration Audit): `expand_phi_in_line` now
// chains all enabled meta-layer expanders so .NET and Spring Boot
// markers are also expanded in decompressed output.

pub(crate) use crate::compression::markers::expand_markers_in_line;

/// Expand all recognised `Φ…` markers from every enabled meta-layer.
///
/// Chains the Angular, Spring Boot, and .NET expanders in order.
/// Each meta-layer's expander is gated by its Cargo feature so
/// disabled features contribute zero overhead.
pub(crate) fn expand_phi_in_line(line: &str) -> String {
    let s = crate::angular_meta::markers::expand_phi_in_line(line);
    #[cfg(feature = "spring_boot")]
    let s = crate::spring_meta::markers::expand_phi_in_line(&s);
    #[cfg(feature = "dotnet")]
    let s = crate::dotnet_meta::markers::expand_phi_in_line(&s);
    s
}
