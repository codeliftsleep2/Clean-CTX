//! Test-scoped observations only. Never consulted by production authority.
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default)]
pub(crate) struct Counts {
    pub publication_copies: std::collections::BTreeMap<&'static str, PublicationCopy>,
    pub publication_sinks: std::collections::BTreeMap<&'static str, [usize; 3]>,
    pub a4: DependencyWork,
    pub witnesses: usize,
    pub stats_initial_text_renders: usize,
    pub stats_initial_text_input_files: usize,
    pub stats_initial_text_bytes: usize,
    pub replay_sequence_calls: usize,
    pub replay_candidate_clones: usize,
    pub replay_candidate_tuples: usize,
    pub replay_candidate_string_bytes: usize,
    pub replay_full_validation_passes: usize,
    pub replay_full_validation_tuple_visits: usize,
    pub replay_tuple_validation_calls: usize,
    pub replay_index_rebuilds: usize,
    pub replay_index_tuple_visits: usize,
    pub replay_occurrence_prefix_visits: usize,
    pub replay_sequence_commits: usize,
    pub sqlite_baseline_queries: usize,
    pub sqlite_owner_queries: usize,
    pub sqlite_decode_bytes: usize,
    pub sqlite_delta_rows: usize,
    pub sqlite_delta_payload_bytes: usize,
    pub sqlite_output_tuple_clones: usize,
    pub spring_detection_parses: usize,
    pub spring_detection_parse_bytes: usize,
    pub spring_method_scan_calls: usize,
    pub spring_method_scan_bytes: usize,
    pub traversal_node_builds: usize,
    pub traversal_source_keys: usize,
    pub traversal_unique_nodes: usize,
    pub traversal_adjacency_edges: usize,
    pub hierarchy_class_comparisons: usize,
    pub hierarchy_placement_map_probes: usize,
    pub token_count_inputs: [std::collections::BTreeMap<String, usize>; 2],
    pub token_count_bytes: [usize; 2],
    pub alias_calls: usize,
    pub alias_input_bytes: usize,
    pub alias_setup_entries: usize,
    pub alias_eligible_entries: usize,
    pub alias_sort_key_calls: usize,
    pub alias_positions: usize,
    pub alias_probes: usize,
    pub alias_key_bytes_considered: usize,
    pub alias_matches: usize,
    pub alias_output_bytes: usize,
    pub alias_pair_capacity: usize,
    pub alias_ir_calls: usize,
    pub alias_ir_instruction_visits: usize,
    pub alias_ir_operands: usize,
    pub identities: BTreeSet<String>,
    pub variants: [usize; 7],
    pub export_reads: usize,
    pub export_parses: usize,
    pub ngrx_binding_collections: usize,
    pub ngrx_binding_demand_checks: usize,
    pub ngrx_binding_edge_visits: usize,
    pub ngrx_binding_records: usize,
    pub ngrx_binding_string_bytes: usize,
    pub ngrx_map_binding_visits: usize,
    pub ngrx_switch_binding_visits: usize,
    pub ngrx_reexport_reads: usize,
    pub ngrx_reexport_parses: usize,
    pub reads: usize,
    pub hashes: usize,
    pub enumerations: usize,
    pub canonicalizations: usize,
    pub metadata: usize,
    pub comparisons: [usize; 3],
    pub declaration_comparisons: [usize; 2],
    pub artifact_publication_clones: [usize; 3],
    pub artifact_projection_owners: usize,
    pub csharp_role_entries: [usize; 6],
    pub csharp_role_class_visits: [usize; 6],
    pub csharp_class_file_visits: [usize; 6],
    pub csharp_method_owner_visits: usize,
    pub csharp_raw_part_visits: [usize; 6],
    pub csharp_context_method_visits: usize,
    pub csharp_aspnet_method_visits: usize,
    pub artifact_ir_instruction_clones: usize,
    pub csharp_resolution_calls: usize,
    pub csharp_catalog_population_max: usize,
    pub csharp_declaration_probes: [usize; 2],
    pub csharp_lookup_builds: usize,
    pub csharp_lookup_build_visits: usize,
    pub csharp_lookup_build_probes: usize,
    pub csharp_lookup_probes: usize,
    pub csharp_lookup_capacities: [usize; 4],
    pub csharp_lookup_payload_bytes: usize,
    pub membership_probes: [usize; 3],
    pub registrations: usize,
    pub reconciliation: usize,
    pub candidates: usize,
    pub retired: usize,
    pub root_facets: usize,
    pub snapshot_validations: usize,
    pub lock_depth: usize,
    pub lock_witnesses: usize,
}
thread_local! { static COUNTS: std::cell::RefCell<Counts> = std::cell::RefCell::new(Counts::default()); }
thread_local! { static ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) }; }
/// Paired helper timing excludes observation bookkeeping, never production work.
pub(crate) fn unobserved<T>(f: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ENABLED.set(self.0);
        }
    }
    let _restore = Restore(ENABLED.replace(false));
    f()
}
pub(crate) fn update(f: impl FnOnce(&mut Counts)) {
    if ENABLED.get() {
        COUNTS.with(|s| f(&mut s.borrow_mut()));
    }
}
pub(crate) fn reset() {
    update(|s| {
        assert_eq!(s.lock_depth, 0);
        *s = Counts::default();
    });
}
pub(crate) fn snapshot() -> Counts {
    COUNTS.with(|s| s.borrow().clone())
}
#[cfg(feature = "typescript")]
pub(crate) fn witness(value: &crate::ir::typescript_resolution::Witness) {
    use crate::ir::typescript_resolution::Witness;
    // Exact serialized enum includes typed expectations, not merely a path.
    let identity = serde_json::to_string(value).expect("witness observation serialization");
    update(|s| {
        s.witnesses += 1;
        s.identities.insert(identity);
        if s.lock_depth > 0 {
            s.lock_witnesses += 1;
        }
        s.variants[match value {
            Witness::Directory { .. } => 0,
            Witness::Content { .. } => 1,
            Witness::Missing { .. } => 2,
            Witness::Kind { .. } => 3,
            Witness::Physical { .. } => 4,
            Witness::Unreadable { .. } => 5,
            Witness::OutsideScope { .. } => 6,
        }] += 1;
    });
}
pub(crate) struct SnapshotScope;
pub(crate) fn snapshot_scope() -> SnapshotScope {
    update(|s| {
        s.snapshot_validations += 1;
        s.lock_depth += 1;
    });
    SnapshotScope
}
impl Drop for SnapshotScope {
    fn drop(&mut self) {
        update(|s| s.lock_depth -= 1);
    }
}

pub(crate) fn token_count_input(boundary: usize, text: &str) {
    let hash = crate::cache::content_hash(text.as_bytes());
    update(|s| {
        *s.token_count_inputs[boundary].entry(hash).or_default() += 1;
        s.token_count_bytes[boundary] += text.len();
    });
}

/// Logical payload at actual clone sites; not allocator events or total heap bytes.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub(crate) struct PublicationCopy {
    pub calls: usize,
    pub edges: usize,
    pub dependencies: usize,
    pub endpoint_string_bytes: usize,
    pub canonical_endpoints: usize,
    pub dependency_string_bytes: usize,
    pub hash_string_bytes: usize,
    pub ir_instructions: usize,
    pub ir_file_id_bytes: usize,
}
pub(crate) fn publication_copy(
    site: &'static str,
    edges: &[crate::layers::meta::semantic::SemanticEdge],
    dependencies: &[crate::layers::meta::semantic::SemanticDependency],
    hash: &str,
) {
    update(|counts| {
        let c = counts.publication_copies.entry(site).or_default();
        c.calls += 1;
        c.edges += edges.len();
        c.dependencies += dependencies.len();
        c.hash_string_bytes += hash.len();
        for edge in edges {
            for endpoint in [&edge.subject, &edge.object] {
                c.endpoint_string_bytes +=
                    endpoint.name.len() + endpoint.file.as_ref().map_or(0, String::len);
                c.canonical_endpoints += usize::from(endpoint.canonical.is_some());
            }
        }
        c.dependency_string_bytes += dependencies
            .iter()
            .map(|d| d.file_path.len() + d.source_hash.as_ref().map_or(0, String::len))
            .sum::<usize>();
    });
}
pub(crate) fn publication_ir_copy(site: &'static str, ir: &crate::ir::compiler::CompiledIR) {
    update(|counts| {
        let c = counts.publication_copies.entry(site).or_default();
        c.calls += 1;
        c.ir_instructions += ir.instructions.len();
        c.ir_file_id_bytes += ir.file_id.len();
    });
}

pub(crate) fn publication_sink(site: &'static str, edges: usize, dependencies: usize) {
    update(|counts| {
        let c = counts.publication_sinks.entry(site).or_default();
        c[0] += 1;
        c[1] += edges;
        c[2] += dependencies;
    });
}

/// A4 source-backed dependency work. Arrays: [single-owner, partition-batch].
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub(crate) struct DependencyWork {
    pub lookup_input_visits: usize,
    pub lookup_insert_attempts: [usize; 3],
    pub lookup_calls: [usize; 3],
    pub lookup_key_comparisons: [usize; 3],
    pub matching_family_visits: usize,
    pub base_family_prefix_visits: usize,
    pub retained_input_references: usize,
    pub retained_seed_references: usize,
    pub retained_family_vectors: usize,
    pub retained_file_sets: usize,
    pub family_reference_capacity: usize,
    pub footprint_calls: [usize; 2],
    pub partition_tests: [usize; 2],
    pub unit_clones: [usize; 2],
    pub peer_visits: [usize; 2],
    pub owner_probes: [usize; 2],
    pub term_probes: [usize; 2],
    pub source_bytes_offered: [usize; 2],
    pub term_bytes_offered: [usize; 2],
    pub emitted_dependencies: [usize; 2],
    pub dependency_string_bytes: [usize; 2],
    pub symbol_builds: [usize; 2],
    pub symbol_part_visits: [usize; 2],
    pub collect_calls: usize,
    pub seed_class_visits: usize,
    pub ancestry_visits: usize,
    pub family_class_visits: usize,
    pub base_find_visits: usize,
    pub returned_terms: usize,
    pub returned_requirements: usize,
}
