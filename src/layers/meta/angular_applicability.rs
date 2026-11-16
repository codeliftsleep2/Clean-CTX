//! Existing Angular applicability, retained separately from owned evaluation.
use crate::compression::Fidelity;
fn applies(context: &super::MetaLayerContext<'_>) -> Option<bool> {
    let meta_config = context
        .config
        .and_then(|config| config.meta_layers.get("angular"));
    if meta_config.is_some_and(|config| !config.enabled) {
        return None;
    }
    let is_angular = crate::angular_meta::detect::is_angular_file_with_regions(
        context.source,
        context.lexical_regions,
    );
    #[cfg(feature = "typescript")]
    let is_angular = is_angular
        || context.typescript_authority.is_some_and(|authority| {
            crate::angular_meta::class_role_authority::has_class_role(authority)
                || crate::angular_meta::functional_authority::has_role(authority)
                || crate::angular_meta::application_config_providers::has_owner(authority)
                || crate::angular_meta::bootstrap_references::has_call(authority)
                || crate::angular_meta::config_composition::has_owner(authority)
        });
    let testing_enabled = meta_config
        .map(|config| config.testing.enabled)
        .unwrap_or(true);
    let reactive_forms_enabled = meta_config
        .map(|config| config.reactive_forms.enabled)
        .unwrap_or(true);
    let formly_enabled = meta_config
        .map(|config| config.formly.enabled)
        .unwrap_or(true);
    let applicable = is_angular
        || crate::angular_meta::rx::has_rxjs_imports(context.source)
        || crate::angular_meta::ngrx::has_ngrx_imports(context.source)
        || crate::angular_meta::ngrx::discriminants::has_switch_action_type_handlers(
            context.source,
        )
        || crate::angular_meta::signals::has_signal_imports(context.source)
        || reactive_forms_enabled
            && crate::angular_meta::reactive_forms::extract_reactive_form_shape_with_regions(
                context.source,
                Fidelity::Low,
                context.lexical_regions,
            )
            .is_some()
        || formly_enabled
            && crate::angular_meta::formly::extract_formly_shape_with_regions(
                context.source,
                Fidelity::Low,
                context.lexical_regions,
            )
            .is_some()
        || crate::angular_meta::routing::has_router_imports(context.source)
        || testing_enabled
            && crate::angular_meta::testing::is_testing_source(context.source, context.path);
    applicable.then_some(is_angular)
}

pub(super) fn evaluate_if_applicable(
    layer_name: &'static str,
    context: &super::MetaLayerContext<'_>,
) -> Option<super::MetaLayerOwnedEvaluation> {
    applies(context).map(|is_angular| {
        super::angular_semantic::evaluate_owned(layer_name, context, Some(is_angular))
    })
}

/// Rebind only producer applicability on decoded source; no semantic evaluation.
#[cfg(feature = "typescript")]
pub(crate) fn registrations_apply(
    source: &str,
    path: &std::path::Path,
    fidelity: Fidelity,
    config: &crate::config::CleanCtxConfig,
    authority: &crate::ir::typescript_resolution::TypeScriptAuthority,
) -> bool {
    if !super::resolver_registrations::enabled(config) {
        return false;
    }
    let regions = crate::meta_util::LexicalRegions::new(source);
    let mut context =
        super::MetaLayerContext::new(source, path, &[], &[], fidelity, Some(config), &regions);
    context.typescript_authority = Some(authority);
    applies(&context).is_some()
}
