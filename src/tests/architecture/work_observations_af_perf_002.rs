//! Test-scoped observations only. Never consulted by production authority.
use std::collections::BTreeSet;
#[derive(Clone, Debug, Default)]
pub(crate) struct Counts {
    pub witnesses: usize,
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
pub(crate) fn update(f: impl FnOnce(&mut Counts)) {
    COUNTS.with(|s| f(&mut s.borrow_mut()));
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
