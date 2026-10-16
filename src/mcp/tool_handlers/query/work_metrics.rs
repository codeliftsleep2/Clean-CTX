//! Test-only structural work counters for semantic query preparation.
//!
//! These counters observe reconstructable work. They never participate in
//! semantic authority, caching, completion, or production behavior.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct QueryWorkCounts {
    pub(crate) csharp_catalog_builds: usize,
    pub(crate) csharp_catalog_reuses: usize,
    pub(crate) csharp_files_considered: usize,
    pub(crate) csharp_files_parsed: usize,
    pub(crate) derived_catalog_invalidations: usize,
    pub(crate) angular_catalog_builds: usize,
    pub(crate) angular_components_considered: usize,
    pub(crate) angular_template_sources_processed: usize,
    pub(crate) angular_inline_templates_processed: usize,
    pub(crate) angular_external_templates_processed: usize,
    pub(crate) ef_catalog_builds: usize,
    pub(crate) ef_catalog_reuses: usize,
    pub(crate) ef_configuration_declarations_examined: usize,
    pub(crate) ef_configuration_reference_comparisons: usize,
    pub(crate) ef_context_declarations_examined: usize,
    pub(crate) ef_context_reference_comparisons: usize,
    pub(crate) ef_model_occurrences_processed: usize,
    pub(crate) ef_owner_reference_comparisons: usize,
    pub(crate) ef_synthetic_type_parses: usize,
    pub(crate) ef_assembly_configuration_comparisons: usize,
    pub(crate) ef_project_root_lookups: usize,
    pub(crate) ef_project_root_filesystem_resolutions: usize,
    pub(crate) ef_project_root_cache_hits: usize,
    pub(crate) reference_candidate_index_builds: usize,
    pub(crate) reference_candidate_index_keys: usize,
    pub(crate) reference_candidate_indexed_references: usize,
    pub(crate) reference_candidate_max_bucket_size: usize,
    pub(crate) csharp_declaration_resolution_comparisons: usize,
    pub(crate) dependency_graph_edges_considered: usize,
    pub(crate) dependency_graph_reference_comparisons: usize,
    pub(crate) dependency_graph_target_position_comparisons: usize,
    pub(crate) dependency_graph_target_index_entries: usize,
    pub(crate) dependency_graph_target_index_lookups: usize,
    pub(crate) declaration_candidate_index_builds: usize,
    pub(crate) declaration_candidate_index_keys: usize,
    pub(crate) declaration_candidate_indexed_declarations: usize,
    pub(crate) declaration_candidate_max_bucket_size: usize,
}

thread_local! {
    static COUNTS: std::cell::Cell<QueryWorkCounts> =
        const { std::cell::Cell::new(QueryWorkCounts {
            csharp_catalog_builds: 0,
            csharp_catalog_reuses: 0,
            csharp_files_considered: 0,
            csharp_files_parsed: 0,
            derived_catalog_invalidations: 0,
            angular_catalog_builds: 0,
            angular_components_considered: 0,
            angular_template_sources_processed: 0,
            angular_inline_templates_processed: 0,
            angular_external_templates_processed: 0,
            ef_catalog_builds: 0,
            ef_catalog_reuses: 0,
            ef_configuration_declarations_examined: 0,
            ef_configuration_reference_comparisons: 0,
            ef_context_declarations_examined: 0,
            ef_context_reference_comparisons: 0,
            ef_model_occurrences_processed: 0,
            ef_owner_reference_comparisons: 0,
            ef_synthetic_type_parses: 0,
            ef_assembly_configuration_comparisons: 0,
            ef_project_root_lookups: 0,
            ef_project_root_filesystem_resolutions: 0,
            ef_project_root_cache_hits: 0,
            reference_candidate_index_builds: 0,
            reference_candidate_index_keys: 0,
            reference_candidate_indexed_references: 0,
            reference_candidate_max_bucket_size: 0,
            csharp_declaration_resolution_comparisons: 0,
            dependency_graph_edges_considered: 0,
            dependency_graph_reference_comparisons: 0,
            dependency_graph_target_position_comparisons: 0,
            dependency_graph_target_index_entries: 0,
            dependency_graph_target_index_lookups: 0,
            declaration_candidate_index_builds: 0,
            declaration_candidate_index_keys: 0,
            declaration_candidate_indexed_declarations: 0,
            declaration_candidate_max_bucket_size: 0,
        }) };
}

pub(crate) fn reset() {
    COUNTS.with(|counts| counts.set(QueryWorkCounts::default()));
}

pub(crate) fn snapshot() -> QueryWorkCounts {
    COUNTS.with(std::cell::Cell::get)
}

fn update(update: impl FnOnce(&mut QueryWorkCounts)) {
    COUNTS.with(|cell| {
        let mut counts = cell.get();
        update(&mut counts);
        cell.set(counts);
    });
}

pub(crate) fn record_csharp_catalog_build() {
    update(|counts| counts.csharp_catalog_builds += 1);
}

pub(crate) fn record_csharp_catalog_reuse() {
    update(|counts| counts.csharp_catalog_reuses += 1);
}

pub(crate) fn record_csharp_file_considered() {
    update(|counts| counts.csharp_files_considered += 1);
}

pub(crate) fn record_csharp_file_parsed() {
    update(|counts| counts.csharp_files_parsed += 1);
}

pub(crate) fn record_derived_catalog_invalidation() {
    update(|counts| counts.derived_catalog_invalidations += 1);
}

pub(crate) fn record_angular_catalog_build() {
    update(|counts| counts.angular_catalog_builds += 1);
}

pub(crate) fn record_angular_component_considered() {
    update(|counts| counts.angular_components_considered += 1);
}

pub(crate) fn record_angular_template_processed(inline: bool) {
    update(|counts| {
        counts.angular_template_sources_processed += 1;
        if inline {
            counts.angular_inline_templates_processed += 1;
        } else {
            counts.angular_external_templates_processed += 1;
        }
    });
}

macro_rules! counter {
    ($name:ident, $field:ident) => {
        pub(crate) fn $name() {
            update(|counts| counts.$field += 1);
        }
    };
}

counter!(record_ef_catalog_build, ef_catalog_builds);
counter!(record_ef_catalog_reuse, ef_catalog_reuses);
counter!(
    record_ef_configuration_declaration,
    ef_configuration_declarations_examined
);
counter!(
    record_ef_configuration_reference_comparison,
    ef_configuration_reference_comparisons
);
counter!(
    record_ef_context_declaration,
    ef_context_declarations_examined
);
counter!(
    record_ef_context_reference_comparison,
    ef_context_reference_comparisons
);
counter!(record_ef_model_occurrence, ef_model_occurrences_processed);
counter!(
    record_ef_owner_reference_comparison,
    ef_owner_reference_comparisons
);
counter!(record_ef_synthetic_type_parse, ef_synthetic_type_parses);
counter!(
    record_ef_assembly_configuration_comparison,
    ef_assembly_configuration_comparisons
);
counter!(record_ef_project_root_lookup, ef_project_root_lookups);
counter!(
    record_ef_project_root_filesystem_resolution,
    ef_project_root_filesystem_resolutions
);
counter!(record_ef_project_root_cache_hit, ef_project_root_cache_hits);
counter!(
    record_csharp_declaration_resolution_comparison,
    csharp_declaration_resolution_comparisons
);
counter!(
    record_dependency_graph_edge,
    dependency_graph_edges_considered
);
counter!(
    record_dependency_graph_reference_comparison,
    dependency_graph_reference_comparisons
);
counter!(
    record_dependency_graph_target_index_entry,
    dependency_graph_target_index_entries
);
counter!(
    record_dependency_graph_target_index_lookup,
    dependency_graph_target_index_lookups
);

pub(crate) fn record_reference_candidate_index(keys: usize, indexed: usize, maximum: usize) {
    update(|counts| {
        counts.reference_candidate_index_builds += 1;
        counts.reference_candidate_index_keys += keys;
        counts.reference_candidate_indexed_references += indexed;
        counts.reference_candidate_max_bucket_size =
            counts.reference_candidate_max_bucket_size.max(maximum);
    });
}

pub(crate) fn record_declaration_candidate_index(keys: usize, indexed: usize, maximum: usize) {
    update(|counts| {
        counts.declaration_candidate_index_builds += 1;
        counts.declaration_candidate_index_keys += keys;
        counts.declaration_candidate_indexed_declarations += indexed;
        counts.declaration_candidate_max_bucket_size =
            counts.declaration_candidate_max_bucket_size.max(maximum);
    });
}
