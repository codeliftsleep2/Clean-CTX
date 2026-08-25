// src/angular_meta/phi.rs
//
// Generic `Φ` marker infrastructure shared by all Angular Ecosystem
// Deepening sub-layers (RxJS, NgRx, Signals, Routing).
//
// # Why this module exists
//
// Each sub-layer previously re-implemented the same marker vocabulary
// plumbing (`marker_prefix`, `expansion`, `from_token`, `token`,
// `all_in_expand_order`, `expand_phi_in_line`, `expand_phi`) with
// copy-pasted bodies. Adding a 5th layer meant copying ~120 more lines
// and editing the hardcoded expansion chain in `markers.rs`.
//
// This module extracts the shared shape into a [`PhiMarker`] trait and a
// **registered expander list**. Adding a new layer is now:
//   1. Implement [`PhiMarker`] for the layer's kind enum.
//   2. Register its `expand_phi_in_line` in [`PHI_EXPANDERS`].
//   3. Done — `markers.rs` iterates the list automatically.

/// A `Φ` marker kind that knows its prefix, expansion, and token.
///
/// Implemented by `RxJsKind`, `NgRxKind`, `SignalKind`, and
/// `RouteKind`. The trait provides the single source of truth for the
/// marker vocabulary of a sub-layer.
pub trait PhiMarker: Copy + Sized {
    /// The `Φ` marker prefix for this kind (e.g. `"Φobs:"`).
    fn marker_prefix(self) -> &'static str;

    /// The human-readable expansion (e.g. `"Observable"`).
    /// Does NOT include the trailing space.
    fn expansion(self) -> &'static str;

    /// All variants in a canonical order. Longer prefixes should be placed
    /// before shorter ones to prevent partial-match issues in string
    /// replacement.
    fn all_in_expand_order() -> &'static [Self];

    /// Look up a kind by its marker token string (without the trailing
    /// colon). Returns `None` for unknown tokens.
    fn from_token(token: &str) -> Option<Self>;

    /// The token string (without trailing `:`) for this kind.
    fn token(self) -> &'static str;
}

/// Expand every recognised `Φ` marker of a [`PhiMarker`] kind in a line
/// back to its human-readable form.
///
/// This is the generic implementation that replaces the four copy-pasted
/// `expand_phi_in_line` functions in rx/ngrx/signals/routing.
pub fn expand_phi_in_line<M: PhiMarker + 'static>(line: &str) -> String {
    let mut s = line.to_string();
    for &kind in M::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        if s.contains(prefix) {
            s = s.replace(prefix, &format!("{} ", kind.expansion()));
        }
    }
    s
}

/// Expand a single `Φ` marker token of a [`PhiMarker`] kind.
/// Returns `None` for unknown markers.
pub fn expand_phi<M: PhiMarker>(token: &str) -> Option<&'static str> {
    M::from_token(token).map(|k| k.expansion())
}

/// A registered sub-layer expander: takes a line and expands every `Φ`
/// marker of that sub-layer's vocabulary.
pub type PhiExpander = fn(&str) -> String;

/// The registered list of sub-layer expanders, in expansion order.
///
/// `markers.rs` iterates this list to chain all sub-layer expansions.
/// Adding a new sub-layer (e.g. React) is a single registration here.
pub static PHI_EXPANDERS: &[PhiExpander] = &[
    crate::angular_meta::rx::expand_phi_in_line,
    crate::angular_meta::ngrx::expand_phi_in_line,
    crate::angular_meta::signals::expand_phi_in_line,
    crate::angular_meta::routing::expand_phi_in_line,
];
