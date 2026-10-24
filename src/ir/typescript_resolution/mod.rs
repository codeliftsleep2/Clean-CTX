//! Native TypeScript authority: snapshots, project policy and exported declarations.
mod automatic_types;

#[cfg(test)]
#[path = "../../tests/ir/typescript_automatic_type_capture_issue_162.rs"]
mod automatic_type_capture_tests;
mod capture;
mod classic;
mod compiler_libraries;
mod constant_strings;
pub(crate) mod constructor_parameters;
pub(crate) mod member_declarations;

#[cfg(test)]
#[path = "../../tests/ir/typescript_library_override_capture_issue_162.rs"]
mod library_override_capture_tests;

pub(crate) mod declaration_facets;
mod declaration_groups;
mod declared_scopes;
mod global_bindings;
mod host;
#[cfg(windows)]
mod host_windows;
#[cfg(test)]
#[path = "../../tests/ir/typescript_library_capture_issue_162.rs"]
mod library_capture_tests;
mod module_references;
mod module_resolution;
mod namespace;
mod package;
mod package_evidence;
mod package_lookup;
mod package_map;
mod parameter_decorators;
mod probe;
mod program;
mod relation_evidence;

#[cfg(test)]
#[path = "../../tests/ir/typescript_global_binding_capture_issue_162.rs"]
mod global_binding_capture_tests;
mod program_directives;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_mode_capture_issue_162.rs"]
mod type_mode_capture_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_directive_capture_issue_162.rs"]
mod directive_capture_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_program_capture_issue_162.rs"]
mod program_capture_tests;
mod project;
mod project_config;
mod project_membership;
mod project_options;
mod project_version;
mod references;
mod relative;
mod resolver;
mod source_classification;
mod string_literal;
mod string_value;

#[cfg(test)]
#[path = "../../tests/ir/typescript_string_literals_issue_162.rs"]
mod string_literal_tests;
mod syntax;
mod syntax_ambient;
mod syntax_frontend;
mod type_packages;
mod type_references;
mod type_versions;
mod version_ranges;
mod version_value;

pub(crate) use capture::TypeScriptAuthority;
pub(crate) use host::{DeclarationId, Host, Namespace, Witness};
pub(crate) use project_config::OptionValue as CompilerOption;
pub(crate) use resolver::{Resolution, Resolver};

#[cfg(test)]
#[path = "../../tests/ir/typescript_resolution_issue_162.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_issue_162.rs"]
mod namespace_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_exports_issue_162.rs"]
mod namespace_export_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_file_resolution_issue_162.rs"]
mod file_resolution_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_virtual_roots_issue_162.rs"]
mod virtual_root_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_resolution_context_issue_162.rs"]
mod project_resolution_context_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_node_resolution_issue_162.rs"]
mod node_resolution_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_package_scope_issue_162.rs"]
mod package_scope_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_export_forms_issue_162.rs"]
mod export_form_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_lexical_sites_issue_162.rs"]
mod lexical_site_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_pattern_bindings_issue_162.rs"]
mod pattern_binding_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_iteration_bindings_issue_162.rs"]
mod iteration_binding_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_callable_scope_issue_162.rs"]
mod callable_scope_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_host_issue_162.rs"]
mod project_host_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_ownership_issue_162.rs"]
mod project_ownership_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_conformance_issue_162.rs"]
mod project_conformance_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_context_issue_162.rs"]
mod project_context_tests;

#[cfg(all(test, windows))]
#[path = "../../tests/ir/typescript_windows_host_issue_162.rs"]
mod windows_host_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_declaration_groups_issue_162.rs"]
mod declaration_group_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_jsx_authority_issue_162.rs"]
mod jsx_authority_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_classic_issue_162.rs"]
mod classic_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_star_issue_162.rs"]
mod type_star_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_occurrences_issue_162.rs"]
mod type_occurrence_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_versions_issue_162.rs"]
mod type_version_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_export_cycles_issue_162.rs"]
mod export_cycle_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_symlink_context_issue_162.rs"]
mod symlink_context_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_request_modes_issue_162.rs"]
mod request_mode_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_merging_issue_162.rs"]
mod namespace_merging_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_declared_scope_capture_issue_162.rs"]
mod declared_scope_capture_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_dotted_namespaces_issue_162.rs"]
mod dotted_namespace_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_lexical_issue_162.rs"]
mod namespace_lexical_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_scoped_uses_issue_162.rs"]
mod namespace_scoped_use_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_namespace_aliases_issue_162.rs"]
mod namespace_alias_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_declaration_aliases_issue_162.rs"]
mod declaration_alias_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_augmentation_issue_162.rs"]
mod project_augmentation_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_program_detection_issue_162.rs"]
mod program_detection_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_detection_defaults_issue_162.rs"]
mod detection_default_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_program_references_issue_162.rs"]
mod program_reference_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_automatic_types_issue_162.rs"]
mod automatic_type_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_compiler_libraries_issue_162.rs"]
mod compiler_library_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_library_overrides_issue_162.rs"]
mod library_override_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_directive_modes_issue_162.rs"]
mod type_directive_mode_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_program_global_bindings_issue_162.rs"]
mod program_global_binding_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_global_merge_groups_issue_162.rs"]
mod global_merge_group_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_enum_merge_parity_issue_162.rs"]
mod enum_merge_parity_tests;

mod import_attributes;

#[cfg(test)]
#[path = "../../tests/ir/typescript_import_type_modes_issue_162.rs"]
mod import_type_mode_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_import_attribute_validation_issue_162.rs"]
mod import_attribute_validation_tests;

mod import_types;

#[cfg(test)]
#[path = "../../tests/ir/typescript_import_type_trace_issue_162.rs"]
mod import_type_trace_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_import_type_boundaries_issue_162.rs"]
mod import_type_boundary_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_import_type_expressions_issue_162.rs"]
mod import_type_expression_tests;

mod type_queries;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_query_boundaries_issue_162.rs"]
mod type_query_boundary_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_type_queries_issue_162.rs"]
mod type_query_tests;

mod ambient_modules;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_scope_evidence_issue_162.rs"]
mod ambient_scope_evidence_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_modules_issue_162.rs"]
mod ambient_module_tests;

mod ambient_groups;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_merge_evidence_issue_162.rs"]
mod ambient_merge_evidence_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_conformance_issue_162.rs"]
mod ambient_conformance_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_acquisition_preservation_issue_162.rs"]
mod ambient_acquisition_preservation_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_ambient_acquisition_issue_162.rs"]
mod ambient_acquisition_tests;

mod reference_outputs;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_output_policy_issue_162.rs"]
mod project_output_policy_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_output_identity_issue_162.rs"]
mod project_output_identity_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_project_output_barriers_issue_162.rs"]
mod project_output_barrier_tests;

mod local_package_inputs;

#[cfg(test)]
#[path = "../../tests/ir/typescript_local_input_witnesses_issue_162.rs"]
mod local_input_witness_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_local_package_inputs_issue_162.rs"]
mod local_package_input_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_local_input_options_issue_162.rs"]
mod local_input_option_tests;

#[cfg(test)]
#[path = "../../tests/ir/typescript_erased_namespace_queries_issue_162.rs"]
mod erased_namespace_query_tests;
