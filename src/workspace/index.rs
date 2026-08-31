// src/workspace/index.rs
//
// WorkspaceIndex — core index structure, insertion, and retrieval.
//
// Identity model (approved, Phase 4 investigation):
//   Entity identity: (domain, entity_type, name) — file excluded.
//   Edge identity:   (relation, subject identity, object identity) — file excluded.
//
// Entity ambiguity:  multiple files may contain the same entity identity.
//                     All occurrences are stored; never silently overwritten.
// Edge deduplication: identical edges inserted multiple times produce one
//                     indexed edge. First occurrence wins.
// File provenance:   file_id is retained for entity disambiguation.
// Determinism:       HashMap for O(1) lookup; returned collections are
//                    sorted for deterministic ordering.

use crate::compression::graph_utils;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};
use std::collections::{HashMap, HashSet};

// ── Key types ─────────────────────────────────────────────────────────

/// Entity identity key — excludes file (matching EntityRef identity model).
pub type EntityKey = (String, String, String); // (domain, entity_type, name)

/// Edge identity key — deduplicates by (relation, subject identity, object identity).
/// File is excluded from edge identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EdgeKey {
    relation: SemanticRelation,
    subject_domain: String,
    subject_type: String,
    subject_name: String,
    object_domain: String,
    object_type: String,
    object_name: String,
}

impl EdgeKey {
    fn from_edge(edge: &SemanticEdge) -> Self {
        Self {
            relation: edge.relation,
            subject_domain: edge.subject.domain.to_string(),
            subject_type: edge.subject.entity_type.to_string(),
            subject_name: edge.subject.name.clone(),
            object_domain: edge.object.domain.to_string(),
            object_type: edge.object.entity_type.to_string(),
            object_name: edge.object.name.clone(),
        }
    }
}

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
    /// Entity identity → outgoing semantic edges.
    forward: HashMap<EntityKey, Vec<SemanticEdge>>,
    /// Entity identity → incoming semantic edges.
    reverse: HashMap<EntityKey, Vec<SemanticEdge>>,
    /// Dedup set for edges.
    edge_set: HashSet<EdgeKey>,
    /// File → entity keys in that file (for provenance tracking).
    file_map: HashMap<String, Vec<EntityKey>>,
    /// Entity name → entity keys (for name-based lookup across domains/types).
    /// Populated alongside the entities map during registration.
    name_index: HashMap<String, Vec<EntityKey>>,
    /// Total edge count before dedup (for diagnostic purposes).
    total_edges_inserted: usize,
    /// Active edge count after dedup.
    edge_count: usize,
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
            name_index: HashMap::new(),
            total_edges_inserted: 0,
            edge_count: 0,
        }
    }

    /// Insert semantic edges from a single file.
    ///
    /// `file_path` is the canonical file identity (provenance, not identity).
    /// In production this is the canonical physical path; test code may use
    /// any stable identifier. This is NOT the αN session-local alias.
    ///
    /// Edges are deduplicated by identity: (relation, subject identity,
    /// object identity). First occurrence wins.
    ///
    /// Entities are NOT deduplicated by identity — multiple occurrences of
    /// the same (domain, entity_type, name) in different files are all
    /// retained. This is the approved ambiguity model.
    ///
    /// Entity registration happens BEFORE the edge dedup check so that
    /// entity occurrences from all files are tracked even when the edge
    /// itself is a duplicate (architectural review, Phase 4a).
    pub fn add_edges(&mut self, file_path: &str, edges: Vec<SemanticEdge>) {
        let mut file_entity_keys: Vec<EntityKey> = Vec::new();

        for mut edge in edges {
            self.total_edges_inserted += 1;

            // Attach file provenance to subject/object if not already set.
            // This happens BEFORE the edge dedup check so entity occurrences
            // from all files are tracked even when the edge is a duplicate.
            if edge.subject.file.is_none() {
                edge.subject.file = Some(file_path.to_string());
            }
            if edge.object.file.is_none() {
                edge.object.file = Some(file_path.to_string());
            }

            // Register entities (all occurrences retained).
            let subj_key = entity_key(&edge.subject);
            let obj_key = entity_key(&edge.object);
            self.register_entity(&subj_key, edge.subject.clone(), &mut file_entity_keys);
            self.register_entity(&obj_key, edge.object.clone(), &mut file_entity_keys);

            // Dedup: skip if this exact edge identity was already inserted.
            // Forward/reverse indexes are only updated for the first occurrence.
            let key = EdgeKey::from_edge(&edge);
            if !self.edge_set.insert(key) {
                continue;
            }
            self.edge_count += 1;

            // Forward index: subject -> outgoing edge.
            self.forward.entry(subj_key).or_default().push(edge.clone());

            // Reverse index: object -> incoming edge.
            self.reverse.entry(obj_key).or_default().push(edge);
        }

        // Track file -> entity keys for provenance.
        if !file_entity_keys.is_empty() {
            self.file_map
                .entry(file_path.to_string())
                .or_default()
                .extend(file_entity_keys);
        }
    }

    /// Register a single entity occurrence. All occurrences are retained.
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
        self.entities.entry(key.clone()).or_default().push(entity);
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

    /// Get all outgoing semantic edges from the entity matching the given
    /// identity. If multiple entities share the same identity, edges from
    /// all occurrences are returned.
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
            .map(|vec| vec.iter().collect())
            .unwrap_or_default()
    }

    /// Get all incoming semantic edges to the entity matching the given
    /// identity. If multiple entities share the same identity, edges to
    /// all occurrences are returned.
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
            .map(|vec| vec.iter().collect())
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

    /// Get the total number of unique entity identities.
    pub fn entity_identity_count(&self) -> usize {
        self.entities.len()
    }

    /// Get the total number of entity occurrences (including duplicates
    /// across files).
    pub fn entity_occurrence_count(&self) -> usize {
        self.entities.values().map(|v| v.len()).sum()
    }

    /// Get the number of deduplicated edges.
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
                let is_inject_target = incoming.iter().any(|e| {
                    matches!(
                        e.relation,
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
    /// Algorithm: selector → `[selector]` marker entity → incoming
    /// `HasSelector` edges → subject entity occurrences.
    ///
    /// Returns all matching entity occurrences. Preserves ambiguity and
    /// insertion order.
    pub fn resolve_selector(&self, selector: &str) -> Vec<&EntityRef> {
        let marker_name = format!("[{}]", selector);
        let marker_keys = match self.name_index.get(&marker_name) {
            Some(k) => k,
            None => return Vec::new(),
        };
        let mut results: Vec<&EntityRef> = Vec::new();
        for marker_key in marker_keys {
            if let Some(incoming) = self.reverse.get(marker_key) {
                for edge in incoming {
                    if edge.relation == SemanticRelation::HasSelector {
                        let subj_key = entity_key(&edge.subject);
                        if let Some(occurrences) = self.entities.get(&subj_key) {
                            results.extend(occurrences.iter());
                        }
                    }
                }
            }
        }
        results
    }

    // ── Phase 4c: Graph Traversal ──────────────────────────────────

    /// The set of `SemanticRelation` variants that represent dependency
    /// relationships (not structural/metadata). Used by `transitive_dependencies`
    /// to determine which forward edges to traverse.
    const DEPENDENCY_RELATIONS: &'static [SemanticRelation] = &[
        SemanticRelation::Injects,
        SemanticRelation::Autowired,
        SemanticRelation::ImportsModule,
        SemanticRelation::HandlesAction,
        SemanticRelation::CallsService,
        SemanticRelation::HasEntity,
        SemanticRelation::MapsFrom,
        SemanticRelation::ConfigurationProperties,
    ];

    /// Build a deterministic node index from all registered entity identities.
    ///
    /// Returns a sorted `Vec<EntityKey>` (the node list) and a
    /// `HashMap<EntityKey, usize>` mapping each key to its position.
    /// The sorted order ensures deterministic traversal across calls.
    fn build_node_index(&self) -> (Vec<EntityKey>, HashMap<EntityKey, usize>) {
        let mut all_keys: std::collections::HashSet<EntityKey> = std::collections::HashSet::new();
        all_keys.extend(self.entities.keys().cloned());
        all_keys.extend(self.forward.keys().cloned());
        all_keys.extend(self.reverse.keys().cloned());
        let mut keys: Vec<EntityKey> = all_keys.into_iter().collect();
        keys.sort();
        let map: HashMap<EntityKey, usize> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| (k.clone(), i))
            .collect();
        (keys, map)
    }

    /// Check whether the entity graph contains any directed cycles.
    ///
    /// Traverses all currently produced semantic relations (both dependency
    /// and structural). Uses three-color DFS. File provenance is irrelevant
    /// — operates on `EntityKey` identity only.
    ///
    /// Returns `false` for an empty index or a single-node graph with no
    /// self-loop.
    pub fn has_cycle(&self) -> bool {
        let (node_list, index_map) = self.build_node_index();
        if node_list.is_empty() {
            return false;
        }

        let adj_fn = |i: usize| {
            let key = &node_list[i];
            self.forward
                .get(key)
                .map(|edges| {
                    edges
                        .iter()
                        .filter_map(|e| index_map.get(&entity_key(&e.object)).copied())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        graph_utils::has_cycle(node_list.len(), adj_fn)
    }

    /// Compute transitive dependencies of an entity identity.
    ///
    /// Traverses only dependency-like semantic relations:
    /// `Injects`, `Autowired`, `ImportsModule`, `HandlesAction`,
    /// `CallsService`, `HasEntity`, `MapsFrom`, `ConfigurationProperties`.
    ///
    /// Structural/metadata relations (`HasSelector`, `RouteMapsTo`, etc.)
    /// are NOT traversed.
    ///
    /// # Arguments
    /// * `domain` - Entity domain
    /// * `entity_type` - Entity type
    /// * `name` - Entity name
    /// * `depth` - Max traversal depth: `0` = unlimited, `1` = direct,
    ///   `2` = transitive. Negative behaves as `0`.
    ///
    /// # Returns
    /// `Vec<EntityKey>` — reachable entity identities in BFS order
    /// (insertion-order derived). The starting entity is excluded.
    /// Each EntityKey appears at most once.
    pub fn transitive_dependencies(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        depth: i32,
    ) -> Vec<EntityKey> {
        let start_key = (
            domain.to_string(),
            entity_type.to_string(),
            name.to_string(),
        );
        if !self.forward.contains_key(&start_key) {
            return Vec::new();
        }

        let (node_list, index_map) = self.build_node_index();

        let start_idx = match index_map.get(&start_key) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        let adj_fn = |i: usize| {
            let key = &node_list[i];
            self.forward
                .get(key)
                .map(|edges| {
                    edges
                        .iter()
                        .filter(|e| WorkspaceIndex::DEPENDENCY_RELATIONS.contains(&e.relation))
                        .filter_map(|e| index_map.get(&entity_key(&e.object)).copied())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let depth_param = if depth < 0 { 0 } else { depth };
        let indices =
            graph_utils::transitive_dependencies(start_idx, depth_param, node_list.len(), adj_fn);
        indices.into_iter().map(|i| node_list[i].clone()).collect()
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
