//! Existing NgRx extraction composition with private call-site provenance.
use super::*;

/// Extract the NgRx shape from a source file.
///
/// Returns `None` when the file has no NgRx imports (zero overhead).
/// Returns `Some(NgRxShape)` with detected actions, reducers, effects,
/// selectors, entity adapters, and store usage.
pub fn extract_ngrx_shape(source: &str, _fidelity: Fidelity) -> Option<NgRxShape> {
    let lexical_regions = crate::meta_util::LexicalRegions::new(source);
    extract_ngrx_shape_with_regions(source, _fidelity, &lexical_regions)
}

pub(crate) fn extract_ngrx_shape_with_regions(
    source: &str,
    fidelity: Fidelity,
    lexical_regions: &crate::meta_util::LexicalRegions,
) -> Option<NgRxShape> {
    extract_ngrx_shape_with_sites(source, fidelity, lexical_regions).map(|(shape, _)| shape)
}

pub(crate) fn extract_ngrx_shape_with_sites(
    source: &str,
    _fidelity: Fidelity,
    lexical_regions: &crate::meta_util::LexicalRegions,
) -> Option<(NgRxShape, Vec<usize>)> {
    #[cfg(test)]
    crate::angular_meta::ngrx::extraction_metrics::record_extraction();

    // Import gate: skip non-NgRx files
    if !has_ngrx_imports(source) {
        return None;
    }

    let mut shape = NgRxShape::default();

    // Extract feature name from createFeature or StoreModule.forFeature
    extract_feature_name(source, &mut shape, lexical_regions);

    // Extract action creators
    extract_actions(source, &mut shape, lexical_regions);

    // Extract reducer
    extract_reducer(source, &mut shape, lexical_regions);

    // Extract effects
    extract_effects(source, &mut shape, lexical_regions);

    // Extract selectors
    extract_selectors(source, &mut shape, lexical_regions);

    // Extract entity adapter
    extract_entity_adapter(source, &mut shape, lexical_regions);

    // Extract the enclosing component class name (for Component -> Store
    // graph edges). Must run before `extract_store_injections` so the
    // component name is available when wiring the edge.
    extract_component_name(source, &mut shape, lexical_regions);

    // Extract store injections
    extract_store_injections(source, &mut shape, lexical_regions);

    // Extract dispatch/select call sites
    let starts = extract_call_sites(source, &mut shape, lexical_regions);

    if shape.is_empty() {
        return None;
    }

    Some((shape, starts))
}
