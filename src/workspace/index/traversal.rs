// Graph traversal queries for WorkspaceIndex.

use super::{EntityKey, WorkspaceIndex, entity_key, entity_occurrence_admitted};
use crate::compression::graph_utils;
use crate::layers::meta::semantic::SemanticRelation;
use crate::workspace::scope::WorkspaceScope;
use std::collections::{HashMap, HashSet};

impl WorkspaceIndex {
    /// The semantic relations treated as dependency relationships.
    ///
    /// `SemanticRelation::Calls` is deliberately NOT here (approved traversal
    /// policy): call relationships are a separate native fact, so
    /// `transitive_dependencies` remains exactly as it was and direct
    /// forward/reverse `Calls` queries are the way to consume them.
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
    fn build_node_index(&self) -> (Vec<EntityKey>, HashMap<EntityKey, usize>) {
        let mut all_keys: HashSet<EntityKey> = HashSet::new();
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
    /// Traverses every semantic relation with three-color DFS over the whole
    /// retained index (unscoped: the caller declared no workspace).
    ///
    /// `SemanticRelation::Calls` is excluded narrowly: native call facts add
    /// many receiver-independent name-level edges, and treating them as cycle
    /// edges would silently change the established generic `has_cycle`
    /// semantics for every workspace that compiles call sites. Cycle semantics
    /// for every other relation are unchanged.
    pub fn has_cycle(&self) -> bool {
        self.has_cycle_with_scope(None)
    }

    /// Workspace-scoped variant of [`WorkspaceIndex::has_cycle`]: only edge
    /// occurrences ASSERTED from inside `scope` are treated as cycle edges.
    ///
    /// This is the whole difference a scope makes. Cycle membership is unchanged
    /// (same relations, `Calls` still excluded) and nothing is deleted from the
    /// index: an edge asserted by another repository is simply not part of THIS
    /// workspace's evidence, so it can neither extend a path nor close a cycle
    /// here. Two repositories that each contribute one half of a cycle
    /// (A: `X -> Y`, B: `Y -> X`) therefore report no cycle when queried for
    /// either workspace, while the unscoped query keeps reporting the cycle the
    /// session's combined evidence contains.
    ///
    /// Node enumeration is the pre-existing [`WorkspaceIndex::build_node_index`]
    /// (unchanged, not enlarged). A node whose occurrences all lie outside the
    /// scope has no admitted outgoing edge and can never participate in a cycle,
    /// so filtering the adjacency is sufficient for correctness.
    pub fn has_cycle_in_scope(&self, scope: &WorkspaceScope) -> bool {
        self.has_cycle_with_scope(Some(scope))
    }

    fn has_cycle_with_scope(&self, scope: Option<&WorkspaceScope>) -> bool {
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
                        // Native call facts are not cycle edges (approved
                        // traversal policy; see the doc comment above).
                        .filter(|e| e.edge.relation != SemanticRelation::Calls)
                        // Occurrence provenance, never semantic identity: an
                        // edge asserted outside the active workspace is not
                        // this workspace's evidence.
                        .filter(|e| scope.is_none_or(|scope| scope.admits(&e.asserting_file)))
                        .filter_map(|e| index_map.get(&entity_key(&e.edge.object)).copied())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        graph_utils::has_cycle(node_list.len(), adj_fn)
    }

    /// Compute dependency-like entities reachable from an identity in BFS order.
    ///
    /// `depth` uses `0` for unlimited traversal; negative values also behave as
    /// unlimited. The starting identity is excluded and every result is unique.
    pub fn transitive_dependencies(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        depth: i32,
    ) -> Vec<EntityKey> {
        self.transitive_dependencies_with_scope(domain, entity_type, name, depth, None)
    }

    /// Workspace-scoped variant of
    /// [`WorkspaceIndex::transitive_dependencies`]: the traversal begins at the
    /// requested identity only when the active workspace contains it, follows
    /// only edge occurrences ASSERTED from inside `scope`, and therefore reports
    /// only what that workspace's own evidence reaches.
    ///
    /// Scope must be applied DURING traversal, never to the final result alone.
    /// A reachable set is not a filterable list: if an unrelated repository
    /// supplied `X -> Y` while this workspace supplied `A -> X`, then filtering
    /// the final output would still have let the walk pass THROUGH `X` and
    /// report `Y` — a dependency this workspace never asserted. Filtering the
    /// adjacency closes that path, so reachability and cycles can only ever be
    /// built from one workspace's evidence.
    ///
    /// The relation set is untouched ([`WorkspaceIndex::DEPENDENCY_RELATIONS`],
    /// `Calls` still excluded) and node enumeration is the pre-existing
    /// [`WorkspaceIndex::build_node_index`].
    ///
    /// Note the asymmetry with an unresolved OBJECT (which stays a valid
    /// dependency of the asserting workspace, exactly as an external callee stays
    /// a valid call target): the boundary is where the fact was ASSERTED, so a
    /// target may be reached even when it is declared elsewhere, while nothing
    /// asserted elsewhere is ever followed.
    pub fn transitive_dependencies_in_scope(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        depth: i32,
        scope: &WorkspaceScope,
    ) -> Vec<EntityKey> {
        self.transitive_dependencies_with_scope(domain, entity_type, name, depth, Some(scope))
    }

    fn transitive_dependencies_with_scope(
        &self,
        domain: &str,
        entity_type: &str,
        name: &str,
        depth: i32,
        scope: Option<&WorkspaceScope>,
    ) -> Vec<EntityKey> {
        let start_key = (
            domain.to_string(),
            entity_type.to_string(),
            name.to_string(),
        );
        if !self.forward.contains_key(&start_key) {
            return Vec::new();
        }
        // A scoped traversal may only start at an entity the active workspace
        // actually contains. Entity-occurrence provenance is the boundary here
        // (`EntityRef.file`): an identity whose only occurrences live in another
        // repository is not this workspace's entity, so the walk does not begin
        // — even if some file inside the scope happened to mention it as an edge
        // endpoint.
        if let Some(scope) = scope {
            let mut start_is_in_scope = false;
            if let Some(occurrences) = self.entities.get(&start_key) {
                for occurrence in occurrences {
                    if entity_occurrence_admitted(occurrence, Some(scope)) {
                        start_is_in_scope = true;
                        break;
                    }
                }
            }
            if !start_is_in_scope {
                return Vec::new();
            }
        }

        let (node_list, index_map) = self.build_node_index();
        let start_idx = match index_map.get(&start_key) {
            Some(&index) => index,
            None => return Vec::new(),
        };

        let adj_fn = |i: usize| {
            let key = &node_list[i];
            self.forward
                .get(key)
                .map(|edges| {
                    edges
                        .iter()
                        .filter(|e| Self::DEPENDENCY_RELATIONS.contains(&e.edge.relation))
                        // Occurrence provenance, never semantic identity.
                        .filter(|e| scope.is_none_or(|scope| scope.admits(&e.asserting_file)))
                        .filter_map(|e| index_map.get(&entity_key(&e.edge.object)).copied())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        };

        let depth = if depth < 0 { 0 } else { depth };
        graph_utils::transitive_dependencies(start_idx, depth, node_list.len(), adj_fn)
            .into_iter()
            .map(|index| node_list[index].clone())
            .collect()
    }
}
