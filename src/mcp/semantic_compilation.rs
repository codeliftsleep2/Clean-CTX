pub(crate) mod artifact;
pub(crate) mod compile;
#[cfg(feature = "csharp")]
pub(crate) mod dependencies;
pub(crate) mod executor;

#[cfg(test)]
#[path = "../tests/mcp/semantic_compilation_artifact.rs"]
mod tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/semantic_compilation_scope_issue_162.rs"]
mod scope_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_authority_retention_issue_162.rs"]
mod retention_tests;

pub(crate) mod retained;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_provide_retention_issue_162.rs"]
mod provide_retention_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_restore_retention_issue_162.rs"]
mod restore_retention_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_transition_retention_issue_162.rs"]
mod transition_retention_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_checkpoint_retention_issue_162.rs"]
mod checkpoint_retention_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_retirement_issue_162.rs"]
mod retirement_tests;

#[cfg(all(test, feature = "typescript"))]
#[path = "../tests/mcp/typescript_query_retention_issue_162.rs"]
mod query_retention_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_component_authority_issue_162.rs"]
mod angular_component_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_class_role_authority_issue_162.rs"]
mod angular_class_role_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_module_authority_issue_162.rs"]
mod angular_module_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_module_facets_issue_162.rs"]
mod angular_module_facets_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_module_durable_facets_issue_162.rs"]
mod angular_module_durable_facets_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_module_relations_issue_162.rs"]
mod angular_module_relations_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_standalone_authority_issue_162.rs"]
mod angular_standalone_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_inject_authority_issue_162.rs"]
mod angular_inject_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_inject_trace_issue_162.rs"]
mod angular_inject_trace_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_inject_lifecycle_issue_162.rs"]
mod angular_inject_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_inject_mixed_issue_162.rs"]
mod angular_inject_mixed_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_constructor_eligibility_issue_162.rs"]
mod angular_constructor_eligibility_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_constructor_override_controls_issue_162.rs"]
mod angular_constructor_override_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_string_token_authority_issue_162.rs"]
mod angular_string_token_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_string_literal_capture_issue_162.rs"]
mod angular_string_literal_capture_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_string_token_carrier_issue_162.rs"]
mod angular_string_token_carrier_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_string_alias_issue_162.rs"]
mod angular_string_alias_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_string_lifecycle_issue_162.rs"]
mod angular_string_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_member_authority_issue_162.rs"]
mod angular_member_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_member_lifecycle_issue_162.rs"]
mod angular_member_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_member_identity_admission_issue_162.rs"]
mod angular_member_identity_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_functional_inject_authority_issue_162.rs"]
mod angular_functional_inject_authority_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_functional_native_capture_issue_162.rs"]
mod angular_functional_native_capture_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_functional_identity_issue_162.rs"]
mod angular_functional_identity_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_functional_lifecycle_issue_162.rs"]
mod angular_functional_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_functional_roles_issue_162.rs"]
mod angular_functional_roles_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_public_entry_lifecycle_issue_162.rs"]
mod angular_public_entry_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_declaration_identity_issue_162.rs"]
mod angular_provider_declaration_identity_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_strategy_pair_issue_162.rs"]
mod angular_provider_strategy_pair_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_strategy_currentness_issue_162.rs"]
mod angular_provider_strategy_currentness_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_factory_identity_issue_162.rs"]
mod angular_provider_factory_identity_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_value_identity_issue_162.rs"]
mod angular_provider_value_identity_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_directive_owner_issue_162.rs"]
mod angular_provider_directive_owner_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_view_provider_position_issue_162.rs"]
mod angular_view_provider_position_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_composition_issue_162.rs"]
mod angular_provider_composition_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_imported_composition_issue_162.rs"]
mod angular_provider_imported_composition_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_owner_composition_issue_162.rs"]
mod angular_provider_owner_composition_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_provider_issue_162.rs"]
mod angular_route_provider_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_application_config_provider_issue_162.rs"]
mod angular_application_config_provider_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_application_config_controls_issue_162.rs"]
mod angular_application_config_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_provider_controls_issue_162.rs"]
mod angular_route_provider_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_composition_admission_issue_162.rs"]
mod angular_provider_composition_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_imported_admission_issue_162.rs"]
mod angular_provider_imported_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_view_provider_admission_issue_162.rs"]
mod angular_view_provider_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_directive_admission_issue_162.rs"]
mod angular_provider_directive_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_value_admission_issue_162.rs"]
mod angular_provider_value_admission_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_provider_declaration_lifecycle_issue_162.rs"]
mod angular_provider_declaration_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_reference_issue_162.rs"]
mod angular_bootstrap_reference_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_controls_issue_162.rs"]
mod angular_bootstrap_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_scope_controls_issue_162.rs"]
mod angular_bootstrap_scope_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_inline_provider_issue_162.rs"]
mod angular_bootstrap_inline_provider_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_inline_controls_issue_162.rs"]
mod angular_bootstrap_inline_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_bootstrap_inline_exit_issue_162.rs"]
mod angular_bootstrap_inline_exit_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_config_composition_issue_162.rs"]
mod angular_config_composition_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_config_composition_controls_issue_162.rs"]
mod angular_config_composition_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_config_composition_exit_issue_162.rs"]
mod angular_config_composition_exit_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_nested_config_input_issue_162.rs"]
mod angular_nested_config_input_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_nested_config_cycles_issue_162.rs"]
mod angular_nested_config_cycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_nested_config_controls_issue_162.rs"]
mod angular_nested_config_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_legacy_bootstrap_issue_162.rs"]
mod angular_legacy_bootstrap_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_declared_return_member_issue_162.rs"]
mod typescript_declared_return_member_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_legacy_bootstrap_controls_issue_162.rs"]
mod angular_legacy_bootstrap_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_legacy_bootstrap_exit_issue_162.rs"]
mod angular_legacy_bootstrap_exit_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_component_membership_issue_162.rs"]
mod angular_route_component_membership_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_route_component_capture_issue_162.rs"]
mod typescript_route_component_capture_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_component_controls_issue_162.rs"]
mod angular_route_component_controls_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_component_support_issue_162.rs"]
mod angular_route_component_support;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_component_lifecycle_issue_162.rs"]
mod angular_route_component_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_route_component_exit_issue_162.rs"]
mod angular_route_component_exit_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_lazy_route_component_issue_162.rs"]
mod angular_lazy_route_component_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_lazy_route_field_issue_162.rs"]
mod typescript_lazy_route_field_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_lazy_selection_support_issue_162.rs"]
mod typescript_lazy_selection_support;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_lazy_selection_issue_162.rs"]
mod typescript_lazy_selection_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/typescript_lazy_selection_lifecycle_issue_162.rs"]
mod typescript_lazy_selection_lifecycle_tests;

#[cfg(all(test, feature = "typescript", feature = "angular"))]
#[path = "../tests/mcp/angular_lazy_route_semantic_issue_162.rs"]
mod angular_lazy_route_semantic_tests;
