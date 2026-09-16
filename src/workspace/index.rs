// src/workspace/index.rs
//
// WorkspaceIndex — core index structure, insertion, and retrieval.
//
// Identity model (approved, Phase 4 investigation):
//   Entity identity: (domain, entity_type, name) — file excluded. UNCHANGED.
//   Edge identity:   (asserting source occurrence, relation, subject identity,
//                     object identity) — file excluded from semantic entity
//                     identity, retained in edge OCCURRENCE identity.
//
// Entity ambiguity:  multiple files may contain the same entity identity.
//                     All occurrences are stored; never silently overwritten.
// Occurrence identity: (domain, entity_type, name, file) — within one file an
//                     identity is registered exactly once, no matter how many
//                     edges mention it.
// Edge deduplication: an identical triple (relation + subject identity +
//                     object identity) re-extracted by the SAME asserting file
//                     produces one indexed occurrence. The same triple asserted
//                     by a DIFFERENT file is a distinct occurrence and is
//                     preserved: an edge asserted by one source occurrence must
//                     never suppress an equivalent-looking edge asserted by a
//                     different source occurrence.
// File provenance:   file_id is retained for entity disambiguation.
// Determinism:       HashMap for O(1) lookup; returned collections are
//                    sorted for deterministic ordering.
//
// Module layout:
//   index.rs          — key types, index state, entity registration, queries.
//   index/edges.rs    — edge-occurrence identity and insertion (`add_edges`).
//   index/remove.rs   — file-local, occurrence-exact removal (`remove_file`).
//   index/traversal.rs— graph traversal (cycles, transitive dependencies).
//
// Registration records: a self-referential `Defines` edge (subject identity
//                     == object identity) is an entity-registration carrier,
//                     not a semantic relationship. `add_edges` normalizes it
//                     at the index write boundary — the entity is registered
//                     once with file provenance and the record never enters
//                     edge_set / file_edges / forward / reverse. Real
//                     relationships are unaffected: `Defines(A, B)` stays a
//                     graph edge, and non-Defines self-loops (e.g.
//                     `Injects(A, A)`) remain cycles. Enforced by
//                     tests::self_defines_edge_is_registration_record_only.

use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::{HashMap, HashSet};

mod edges;
mod remove;
mod traversal;

use edges::{EdgeKey, StoredEdge};

// ── Key types ─────────────────────────────────────────────────────────

/// Entity identity key — excludes file (matching EntityRef identity model).
pub type EntityKey = (String, String, String); // (domain, entity_type, name)

// Edge occurrence identity — `(asserting source occurrence, relation, subject
// semantic identity, object semantic identity)` — is defined in `edges.rs` next
// to `add_edges`, together with the stored wrapper (`StoredEdge`) used by the
// forward/reverse indexes. The asserting source occurrence is part of EDGE
// occurrence identity only: `EntityKey` deliberately still excludes the file
// (Model C).

/// Entity identity key for one reference — `(domain, entity_type, name)`, with
/// the file intentionally excluded.
fn entity_key(entity: &EntityRef) -> EntityKey {
    (
        entity.domain.to_string(),
        entity.entity_type.to_string(),
        entity.name.clone(),
    )
}

// ── WorkspaceIndex ────────────────────────────────────────────────────

/// Framework-agnostic cross-file semantic index.
///
/// # Ownership
///
/// The WorkspaceIndex owns its data. It is populated by inserting semantic
/// edges (typically from a batch of InferenceLayer instances) and queried
/// by the public API. It is independent of the per-file pipeline.
///
/// # Lifecycle
///
/// ```text
/// WorkspaceIndex::new()
///     → index.add_edges(file_id, edges)
///     → queries
///     → drop
/// ```
///
/// # Thread safety
///
/// Not thread-safe by default. Callers must synchronize externally.
#[derive(Debug, Clone)]
pub struct WorkspaceIndex {
    /// Entity identity → all entity occurrences (with file context).
    entities: HashMap<EntityKey, Vec<EntityRef>>,
    /// Entity identity → outgoing edge occurrences (one entry per asserting
    /// source occurrence).
    forward: HashMap<EntityKey, Vec<StoredEdge>>,
    /// Entity identity → incoming edge occurrences (one entry per asserting
    /// source occurrence).
    reverse: HashMap<EntityKey, Vec<StoredEdge>>,
    /// Dedup set for edges.
    edge_set: HashSet<EdgeKey>,
    /// File → entity keys in that file (for provenance tracking).
    file_map: HashMap<String, Vec<EntityKey>>,
    /// File → edge keys originating from that file (for precise edge cleanup
    /// on file recompilation or deletion). Populated alongside edge_set.
    file_edges: HashMap<String, Vec<EdgeKey>>,
    /// Entity name → entity keys (for name-based lookup across domains/types).
    /// Populated alongside the entities map during registration.
    name_index: HashMap<String, Vec<EntityKey>>,
    /// Total edge count before dedup (for diagnostic purposes).
    total_edges_inserted: usize,
    /// Active edge count after dedup.
    edge_count: usize,
    /// Name buckets examined by the most recent file removal.
    #[cfg(test)]
    name_cleanup_buckets_examined: Vec<String>,
}
impl WorkspaceIndex {
    /// Create an empty WorkspaceIndex.
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            forward: HashMap::new(),
            reverse: HashMap::new(),
            edge_set: HashSet::new(),
            file_map: HashMap::new(),
            file_edges: HashMap::new(),
            name_index: HashMap::new(),
            total_edges_inserted: 0,
            edge_count: 0,
            #[cfg(test)]
            name_cleanup_buckets_examined: Vec::new(),
        }
    }

    // `add_edges` lives in `index/edges.rs` together with the edge-occurrence
    // identity types it maintains (`EdgeKey`, `StoredEdge`).

    /// Register a single entity occurrence.
    ///
    /// Occurrence identity is (domain, entity_type, name, file): registering
    /// the same occurrence again is a no-op. An entity that participates in
    /// many edges within one file is registered exactly once for that file;
    /// the same identity in a different file remains a distinct occurrence
    /// (cross-file ambiguity preserved).
    fn register_entity(
        &mut self,
        key: &EntityKey,
        entity: EntityRef,
        file_entity_keys: &mut Vec<EntityKey>,
    ) {
        // Only push unique entity keys into the name index to avoid
        // repeated lookups for the same (name, identity) pair.
        let name_entries = self.name_index.entry(entity.name.clone()).or_default();
        if !name_entries.contains(key) {
            name_entries.push(key.clone());
        }
        // Idempotent occurrence registration: edge participation must not
        // multiply occurrences. An entity mentioned by N edges in one file
        // is registered once for that file (occurrence identity:
        // (domain, entity_type, name, file)).
        let occurrences = self.entities.entry(key.clone()).or_default();
        if occurrences.iter().any(|e| e.file == entity.file) {
            return;
        }
        occurrences.push(entity);
        file_entity_keys.push(key.clone());
    }
    // ── Core queries (Phase 4a) ──────────────────────────────────────

    /// Get all entity occurrences matching the given identity.
    /// Returns an empty vec if no entities match.
    pub fn entities_by_identity(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<&EntityRef> {
        let key = (
            domain.to_string(),
            entity_type.to_string(),
            name.to_string(),
        );
        self.entities
            .get(&key)
            .map(|vec| vec.iter().collect())
            .unwrap_or_default()
    }

    /// Get all outgoing edge occurrences from the entity matching the given
    /// identity. If multiple source occurrences share the same identity, the
    /// complete evidence of every occurrence is returned: an edge asserted by
    /// one occurrence is never collapsed into an equivalent-looking edge
    /// asserted by another.
    pub fn forward_edges_by_identity(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<&SemanticEdge> {
        let key = (
            domain.to_string(),
            entity_type.to_string(),
            name.to_string(),
        );
        self.forward
            .get(&key)
            .map(|vec| vec.iter().map(|stored| &stored.edge).collect())
            .unwrap_or_default()
    }

    /// Get all incoming edge occurrences to the entity matching the given
    /// identity. If multiple source occurrences share the same identity, every
    /// occurrence's evidence is returned and counted separately, so consumer
    /// counts and blast-radius queries see all real consumers.
    pub fn reverse_edges_by_identity(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
    ) -> Vec<&SemanticEdge> {
        let key = (
            domain.to_string(),
            entity_type.to_string(),
            name.to_string(),
        );
        self.reverse
            .get(&key)
            .map(|vec| vec.iter().map(|stored| &stored.edge).collect())
            .unwrap_or_default()
    }

    /// Get all entities in a specific file.
    pub fn entities_in_file(&self, file_path: &str) -> Vec<&EntityRef> {
        let keys = match self.file_map.get(file_path) {
            Some(k) => k,
            None => return Vec::new(),
        };
        let mut results: Vec<&EntityRef> = Vec::new();
        for key in keys {
            if let Some(occurrences) = self.entities.get(key) {
                for entity in occurrences {
                    if entity.file.as_deref() == Some(file_path) {
                        results.push(entity);
                    }
                }
            }
        }
        results
    }

    /// Get the number of files tracked in the index.
    pub fn file_count(&self) -> usize {
        self.file_map.len()
    }

    /// Access the file map (file path → entity keys) for hydration dedup.
    pub fn file_map(&self) -> &std::collections::HashMap<String, Vec<EntityKey>> {
        &self.file_map
    }

    /// Get the total number of unique entity identities.
    pub fn entity_identity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the total number of entity occurrences (including duplicates
    /// across files).
    pub fn entity_occurrence_count(&self) -> usize {
        self.entities.values().map(|v| v.len()).sum()
    }

    /// Get the number of indexed edge occurrences.
    pub fn edge_count(&self) -> usize {
        self.edge_count
    }

    /// Get the total number of edges inserted before dedup.
    pub fn total_edges_inserted(&self) -> usize {
        self.total_edges_inserted
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.edge_count == 0
    }

    // `remove_file` lives in `index/remove.rs`: file-local, occurrence-exact
    // cleanup driven by `file_edges` (plus the test-only observation accessor).
    // ── Phase 4b queries ─────────────────────────────────────────────

    /// Find all entity occurrences with the given name across all domains
    /// and entity types.
    ///
    /// This is a name-based lookup (not identity-based). Multiple identical
    /// entity identities at different file locations all match.
    ///
    /// Returns an empty vec when no entity with that name exists.
    pub fn find_entities_by_name(&self, name: &str) -> Vec<&EntityRef> {
        let keys = match self.name_index.get(name) {
            Some(k) => k,
            None => return Vec::new(),
        };
        let mut results: Vec<&EntityRef> = Vec::new();
        for key in keys {
            if let Some(occurrences) = self.entities.get(key) {
                results.extend(occurrences.iter());
            }
        }
        results
    }

    /// Resolve an injection reference target by bare type name.
    ///
    /// Returns all entity occurrences that are referenced as injection
    /// targets by an `Injects` or `Autowired` edge whose object has the
    /// requested type/name.
    ///
    /// Semantics: the returned EntityRef.file indicates the extraction
    /// provenance (the file where the injection reference occurred), NOT
    /// necessarily the definition file of the injected target.
    ///
    /// Preserves ambiguity: if the same target name is referenced from
    /// multiple files, all occurrences are returned.
    pub fn resolve_inject_type(&self, type_name: &str) -> Vec<&EntityRef> {
        let keys = match self.name_index.get(type_name) {
            Some(k) => k,
            None => return Vec::new(),
        };
        let mut results: Vec<&EntityRef> = Vec::new();
        for key in keys {
            // Only include entities that are the target of an Injects or
            // Autowired edge (incoming edge on the object side).
            if let Some(incoming) = self.reverse.get(key) {
                let is_inject_target = incoming.iter().any(|stored| {
                    matches!(
                        stored.edge.relation,
                        SemanticRelation::Injects | SemanticRelation::Autowired
                    )
                });
                if is_inject_target {
                    if let Some(occurrences) = self.entities.get(key) {
                        results.extend(occurrences.iter());
                    }
                }
            }
        }
        results
    }

    /// Resolve a CSS selector string to component/directive entity
    /// occurrences that expose that selector.
    ///
    /// Algorithm: selector → selector entity (by literal name) → incoming
    /// `HasSelector` edges → subject entity occurrences.
    ///
    /// The selector is stored verbatim in the `HasSelector` object's
    /// `EntityRef.name` (Selector-Value Invariant); no lookup encoding is
    /// applied here.
    ///
    /// Returns all matching entity occurrences. Preserves ambiguity and
    /// insertion order. Each occurrence is returned exactly once, no matter how
    /// many files asserted the selector.
    pub fn resolve_selector(&self, selector: &str) -> Vec<&EntityRef> {
        let marker_keys = match self.name_index.get(selector) {
            Some(k) => k,
            None => return Vec::new(),
        };
        // Occurrence-aware storage may hold several occurrences of the same
        // `HasSelector` triple (one per asserting file). This resolver answers
        // with ENTITY occurrences, so each matching subject identity is visited
        // once regardless of how many files asserted the selector.
        let mut subject_keys: Vec<EntityKey> = Vec::new();
        for marker_key in marker_keys {
            if let Some(incoming) = self.reverse.get(marker_key) {
                for stored in incoming {
                    if stored.edge.relation == SemanticRelation::HasSelector {
                        let subj_key = entity_key(&stored.edge.subject);
                        if !subject_keys.contains(&subj_key) {
                            subject_keys.push(subj_key);
                        }
                    }
                }
            }
        }
        let mut results: Vec<&EntityRef> = Vec::new();
        for subject_key in subject_keys {
            if let Some(occurrences) = self.entities.get(&subject_key) {
                results.extend(occurrences.iter());
            }
        }
        results
    }
}

impl Default for WorkspaceIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/workspace/index.rs"]
mod tests;

#[cfg(test)]
#[path = "../tests/workspace/index_support.rs"]
mod index_support;

#[cfg(test)]
#[path = "../tests/workspace/index_queries.rs"]
mod query_tests;

#[cfg(test)]
#[path = "../tests/workspace/index_graph.rs"]
mod graph_tests;

#[cfg(test)]
#[path = "../tests/workspace/index_edge_occurrence.rs"]
mod edge_occurrence_tests;

#[cfg(test)]
#[path = "../tests/workspace/index_edge_lifecycle.rs"]
mod edge_lifecycle_tests;

#[cfg(test)]
#[path = "../tests/workspace/index_performance.rs"]
mod performance_tests;
