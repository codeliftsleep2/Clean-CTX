// Graph traversal queries for WorkspaceIndex.

use super::{EntityKey, WorkspaceIndex, entity_key};
use crate::compression::graph_utils;
use crate::layers::meta::semantic::SemanticRelation;
use std::collections::{HashMap, HashSet};

impl WorkspaceIndex {
    /// The semantic relations treated as dependency relationships.
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
    /// Traverses every semantic relation with three-color DFS. File provenance
    /// is irrelevant because traversal operates on `EntityKey` identity only.
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
                        .filter_map(|edge| index_map.get(&entity_key(&edge.object)).copied())
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
                        .filter(|edge| Self::DEPENDENCY_RELATIONS.contains(&edge.relation))
                        .filter_map(|edge| index_map.get(&entity_key(&edge.object)).copied())
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
