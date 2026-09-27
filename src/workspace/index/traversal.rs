// Graph traversal queries for WorkspaceIndex.

use super::{EntityKey, WorkspaceIndex, entity_key, entity_occurrence_admitted};
use crate::layers::meta::semantic::SemanticRelation;
use crate::workspace::scope::WorkspaceScope;
use std::collections::{HashMap, HashSet};

fn cycle_relation_sort_key(relation: SemanticRelation) -> u8 {
    match relation {
        SemanticRelation::Injects => 0,
        SemanticRelation::ImportsModule => 1,
        _ => 2,
    }
}

/// One semantic identity participating in a dependency-cycle witness.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
pub(crate) struct CycleEntity {
    pub domain: String,
    pub entity_type: String,
    pub name: String,
}

impl From<&EntityKey> for CycleEntity {
    fn from((domain, entity_type, name): &EntityKey) -> Self {
        Self {
            domain: domain.clone(),
            entity_type: entity_type.clone(),
            name: name.clone(),
        }
    }
}

/// One directed edge in an ordered, closed dependency-cycle witness.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CycleWitnessStep {
    pub subject: CycleEntity,
    pub relation: SemanticRelation,
    pub object: CycleEntity,
    pub asserting_file: String,
}

/// One witness identity that has multiple admitted physical occurrences.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub(crate) struct CycleIdentityAmbiguity {
    pub identity: CycleEntity,
    pub occurrence_files: Vec<String>,
}

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

    /// Relations with an approved closed-path meaning for the initial
    /// dependency-cycle contract.
    ///
    /// This is deliberately narrower than `DEPENDENCY_RELATIONS`, whose
    /// members support reachability but do not all describe a meaningful
    /// architectural cycle. `Autowired` remains excluded until its producer
    /// emits reliable target identity. Native `Calls` remains a separate
    /// call-graph concern.
    const DEPENDENCY_CYCLE_RELATIONS: &'static [SemanticRelation] =
        &[SemanticRelation::Injects, SemanticRelation::ImportsModule];

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
    /// Traverses the approved dependency-cycle relations with three-color DFS
    /// over the whole retained index (unscoped: the caller declared no
    /// workspace). This is intentionally narrower than arbitrary semantic
    /// graph cyclicity: containment, routing, event-flow, mapping, testing,
    /// native call, and unreliable producer relations cannot close a
    /// dependency cycle.
    pub fn has_cycle(&self) -> bool {
        self.dependency_cycle_witness().is_some()
    }

    /// Workspace-scoped variant of [`WorkspaceIndex::has_cycle`]: only edge
    /// occurrences ASSERTED from inside `scope` are treated as cycle edges.
    ///
    /// This is the whole difference a scope makes. Cycle membership is unchanged
    /// (the approved dependency-cycle relation set is unchanged) and nothing
    /// is deleted from the index: an edge asserted by another repository is
    /// simply not part of THIS workspace's evidence, so it can neither extend
    /// a path nor close a cycle here. Two repositories that each contribute one
    /// half of a cycle (A: `X -> Y`, B: `Y -> X`) therefore report no cycle when
    /// queried for either workspace, while the unscoped query keeps reporting
    /// the cycle the session's combined evidence contains.
    ///
    /// Node enumeration is the pre-existing [`WorkspaceIndex::build_node_index`]
    /// (unchanged, not enlarged). A node whose occurrences all lie outside the
    /// scope has no admitted outgoing edge and can never participate in a cycle,
    /// so filtering the adjacency is sufficient for correctness.
    pub fn has_cycle_in_scope(&self, scope: &WorkspaceScope) -> bool {
        self.dependency_cycle_witness_in_scope(scope).is_some()
    }

    /// Return one deterministic closed dependency-cycle witness, if present.
    pub(crate) fn dependency_cycle_witness(&self) -> Option<Vec<CycleWitnessStep>> {
        self.dependency_cycle_witness_with_scope(None)
    }

    /// Workspace-scoped dependency-cycle witness.
    pub(crate) fn dependency_cycle_witness_in_scope(
        &self,
        scope: &WorkspaceScope,
    ) -> Option<Vec<CycleWitnessStep>> {
        self.dependency_cycle_witness_with_scope(Some(scope))
    }

    /// Report physical-occurrence ambiguity for identities in `witness`.
    pub(crate) fn cycle_identity_ambiguities(
        &self,
        witness: &[CycleWitnessStep],
    ) -> Vec<CycleIdentityAmbiguity> {
        self.cycle_identity_ambiguities_with_scope(witness, None)
    }

    /// Workspace-scoped physical-occurrence ambiguity for witness identities.
    pub(crate) fn cycle_identity_ambiguities_in_scope(
        &self,
        witness: &[CycleWitnessStep],
        scope: &WorkspaceScope,
    ) -> Vec<CycleIdentityAmbiguity> {
        self.cycle_identity_ambiguities_with_scope(witness, Some(scope))
    }

    fn cycle_identity_ambiguities_with_scope(
        &self,
        witness: &[CycleWitnessStep],
        scope: Option<&WorkspaceScope>,
    ) -> Vec<CycleIdentityAmbiguity> {
        let identities: std::collections::BTreeSet<CycleEntity> = witness
            .iter()
            .flat_map(|step| [&step.subject, &step.object])
            .cloned()
            .collect();
        identities
            .into_iter()
            .filter_map(|identity| {
                let key = (
                    identity.domain.clone(),
                    identity.entity_type.clone(),
                    identity.name.clone(),
                );
                let mut occurrence_files: Vec<String> = self
                    .entities
                    .get(&key)?
                    .iter()
                    .filter(|occurrence| entity_occurrence_admitted(occurrence, scope))
                    .filter_map(|occurrence| occurrence.file.clone())
                    .collect();
                occurrence_files.sort();
                occurrence_files.dedup();
                (occurrence_files.len() > 1).then_some(CycleIdentityAmbiguity {
                    identity,
                    occurrence_files,
                })
            })
            .collect()
    }

    fn dependency_cycle_witness_with_scope(
        &self,
        scope: Option<&WorkspaceScope>,
    ) -> Option<Vec<CycleWitnessStep>> {
        let (node_list, index_map) = self.build_node_index();
        if node_list.is_empty() {
            return None;
        }

        let mut adjacency = Vec::with_capacity(node_list.len());
        for key in &node_list {
            let mut outgoing = self
                .forward
                .get(key)
                .map(|edges| {
                    edges
                        .iter()
                        // Only relations with an approved dependency-cycle
                        // meaning may extend or close this traversal.
                        .filter(|e| Self::DEPENDENCY_CYCLE_RELATIONS.contains(&e.edge.relation))
                        // Occurrence provenance, never semantic identity: an
                        // edge asserted outside the active workspace is not
                        // this workspace's evidence.
                        .filter(|e| scope.is_none_or(|scope| scope.admits(&e.asserting_file)))
                        .filter_map(|e| {
                            let object_key = entity_key(&e.edge.object);
                            let object_index = index_map.get(&object_key).copied()?;
                            Some((
                                object_index,
                                CycleWitnessStep {
                                    subject: CycleEntity::from(key),
                                    relation: e.edge.relation,
                                    object: CycleEntity::from(&object_key),
                                    asserting_file: e.asserting_file.clone(),
                                },
                            ))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            // Hash-backed index storage has no semantic iteration order. Sort
            // once per query so identical index state selects the same witness.
            // Together with sorted node construction, this makes the full query
            // O(V log V + sum(d(v) log d(v)) + V + E), not plain O(V+E).
            outgoing.sort_by(|(_, left), (_, right)| {
                (
                    left.object.domain.as_str(),
                    left.object.entity_type.as_str(),
                    left.object.name.as_str(),
                    cycle_relation_sort_key(left.relation),
                    left.asserting_file.as_str(),
                )
                    .cmp(&(
                        right.object.domain.as_str(),
                        right.object.entity_type.as_str(),
                        right.object.name.as_str(),
                        cycle_relation_sort_key(right.relation),
                        right.asserting_file.as_str(),
                    ))
            });
            adjacency.push(outgoing);
        }

        let mut color = vec![0_u8; node_list.len()];
        let mut parent_node = vec![None; node_list.len()];
        let mut parent_edge: Vec<Option<CycleWitnessStep>> = vec![None; node_list.len()];

        for start in 0..node_list.len() {
            if color[start] != 0 {
                continue;
            }
            color[start] = 1;
            let mut stack = vec![(start, 0_usize)];
            while let Some((node, next_edge)) = stack.last_mut() {
                if *next_edge == adjacency[*node].len() {
                    color[*node] = 2;
                    stack.pop();
                    continue;
                }

                let (next, edge) = adjacency[*node][*next_edge].clone();
                *next_edge += 1;
                match color[next] {
                    0 => {
                        parent_node[next] = Some(*node);
                        parent_edge[next] = Some(edge);
                        color[next] = 1;
                        stack.push((next, 0));
                    }
                    1 => {
                        let mut path = Vec::new();
                        let mut cursor = *node;
                        while cursor != next {
                            path.push(
                                parent_edge[cursor]
                                    .clone()
                                    .expect("a gray descendant has a parent edge"),
                            );
                            cursor =
                                parent_node[cursor].expect("a gray descendant has a parent node");
                        }
                        path.reverse();
                        path.push(edge);
                        return Some(path);
                    }
                    _ => {}
                }
            }
        }

        None
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
        crate::compression::graph_utils::transitive_dependencies(
            start_idx,
            depth,
            node_list.len(),
            adj_fn,
        )
        .into_iter()
        .map(|index| node_list[index].clone())
        .collect()
    }
}
