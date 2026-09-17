// src/workspace/index/edges.rs
//
// Edge insertion and edge-occurrence identity for WorkspaceIndex.
//
// Identity model:
//   Semantic entity identity: (domain, entity_type, name) - unchanged (Model C).
//   Edge occurrence identity: (asserting source occurrence, relation, subject
//                             semantic identity, object semantic identity).
//
// The asserting source occurrence participates ONLY in edge identity. It is
// deliberately absent from `EntityKey`, so two files may contain occurrences of
// the same semantic entity while asserting distinct edge evidence:
//
//   file A: LoadingComponent --Injects--> Router
//   file B: LoadingComponent --Injects--> Router
//
// are two edge occurrences of one semantic fact. An edge asserted by one source
// occurrence must never suppress an equivalent-looking edge asserted by a
// different source occurrence, because forward/reverse queries, blast-radius
// and consumer counts, file-specific reasoning, and removal/recompilation all
// need the surviving evidence of every asserting occurrence.
//
// `layer` remains provenance/classification metadata (established behaviour) and
// therefore does not participate in the key.
//
// Call evidence: `SemanticRelation::Calls` additionally carries the observed
// explicit argument count AND its spread qualifier as edge evidence, both of
// which participate in the key. Two call facts asserted by one file for the
// same caller/callee pair but a different written arity are two different
// facts and both are retained; so are two facts with the SAME written arity
// where only one of them expands a written argument (`foo(a)` vs
// `foo(...args)`) — the count is not an exact arity in the second, so the two
// are materially different evidence. Identical physical invocations of the
// same shape collapse to one occurrence (this feature never claims physical
// call-site counts). Every other relation carries no evidence, so its identity
// is unchanged.

use super::{EntityKey, WorkspaceIndex, entity_key};
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};

// ── Edge occurrence types ────────────────────────────────────────────

/// Identity of a single edge occurrence: the source occurrence that asserted
/// the edge plus the semantic triple it asserted.
///
/// Same-file duplicates of a triple collapse (one occurrence, one record);
/// the same triple asserted by a different file is a distinct occurrence and is
/// retained as separate evidence.
///
/// Call evidence participates in the key as well: for `Calls`, the observed
/// explicit argument count AND its spread qualifier are part of the FACT a file
/// asserted, so `A --Calls(argc=1)--> Foo`, `A --Calls(argc=2)--> Foo`, and
/// `A --Calls(argc=1, spread)--> Foo` are three distinct occurrences. Every
/// other relation carries no evidence (`None`), so their identity is unchanged.
/// Arity never enters `EntityKey` (Model C).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct EdgeKey {
    /// The source occurrence that asserted the edge (`add_edges` file identity).
    /// Not part of semantic entity identity; not part of the semantic triple.
    asserting_file: String,
    relation: SemanticRelation,
    subject_domain: String,
    subject_type: String,
    subject_name: String,
    object_domain: String,
    object_type: String,
    object_name: String,
    /// Observed explicit argument count, when the relation carries call
    /// evidence. `None` for every evidence-free relation.
    explicit_arg_count: Option<usize>,
    /// Whether a written argument of that call expands at run time, when the
    /// relation carries call evidence. `None` for every evidence-free relation.
    ///
    /// Required for identity: two calls may write the same number of argument
    /// nodes while only one of them expands one of them, and collapsing those
    /// would silently drop the fact that discards exact-arity meaning.
    has_spread: Option<bool>,
}

impl EdgeKey {
    /// Build the occurrence key of `edge` as asserted by `asserting_file`.
    fn new(asserting_file: &str, edge: &SemanticEdge) -> Self {
        Self {
            asserting_file: asserting_file.to_string(),
            relation: edge.relation,
            subject_domain: edge.subject.domain.to_string(),
            subject_type: edge.subject.entity_type.to_string(),
            subject_name: edge.subject.name.clone(),
            object_domain: edge.object.domain.to_string(),
            object_type: edge.object.entity_type.to_string(),
            object_name: edge.object.name.clone(),
            explicit_arg_count: edge
                .call_evidence
                .map(|evidence| evidence.explicit_arg_count),
            has_spread: edge.call_evidence.map(|evidence| evidence.has_spread),
        }
    }

    /// Semantic identity of the asserted subject.
    pub(super) fn subject_key(&self) -> EntityKey {
        (
            self.subject_domain.clone(),
            self.subject_type.clone(),
            self.subject_name.clone(),
        )
    }

    /// Semantic identity of the asserted object.
    pub(super) fn object_key(&self) -> EntityKey {
        (
            self.object_domain.clone(),
            self.object_type.clone(),
            self.object_name.clone(),
        )
    }

    /// Is `edge`, asserted by `asserting_file`, exactly this occurrence?
    ///
    /// Both halves matter: the semantic triple AND the asserting source
    /// occurrence. Two files asserting the same triple are different
    /// occurrences and must never be treated as one.
    pub(super) fn matches(&self, asserting_file: &str, edge: &SemanticEdge) -> bool {
        self.asserting_file == asserting_file && self.same_triple(edge)
    }

    /// Does `edge` assert this key's relation, semantic triple, and (when the
    /// relation carries evidence) the same call evidence — written argument
    /// count AND spread qualifier?
    fn same_triple(&self, edge: &SemanticEdge) -> bool {
        self.relation == edge.relation
            && self.subject_domain == edge.subject.domain
            && self.subject_type == edge.subject.entity_type
            && self.subject_name == edge.subject.name
            && self.object_domain == edge.object.domain
            && self.object_type == edge.object.entity_type
            && self.object_name == edge.object.name
            && self.explicit_arg_count == edge.call_evidence.map(|e| e.explicit_arg_count)
            && self.has_spread == edge.call_evidence.map(|e| e.has_spread)
    }
}

/// One stored edge occurrence in the forward/reverse adjacency indexes.
///
/// The asserting file is stored explicitly rather than inferred from the
/// edge's entity-reference provenance, so `remove_file` can drop exactly the
/// occurrences one file asserted and can never damage another file's evidence
/// for the same semantic triple.
#[derive(Debug, Clone)]
pub(super) struct StoredEdge {
    /// The source occurrence that asserted this edge.
    pub(super) asserting_file: String,
    /// The semantic edge occurrence.
    pub(super) edge: SemanticEdge,
}

impl StoredEdge {
    fn new(asserting_file: &str, edge: SemanticEdge) -> Self {
        Self {
            asserting_file: asserting_file.to_string(),
            edge,
        }
    }
}

// ── Insertion ────────────────────────────────────────────────────────

impl WorkspaceIndex {
    /// Insert semantic edges from a single file.
    ///
    /// `file_path` is the canonical file identity (provenance, not semantic
    /// identity). In production this is the canonical physical path; test code
    /// may use any stable identifier. This is NOT the αN session-local alias.
    ///
    /// Edges are deduplicated by OCCURRENCE identity: (asserting file,
    /// relation, subject semantic identity, object semantic identity, and the
    /// relation's own edge evidence when it carries any — currently the call
    /// argument count of `Calls`). A repeated extraction within one file
    /// produces one occurrence; the same triple asserted by a different file is
    /// a distinct occurrence and is preserved as its own evidence record.
    ///
    /// Entity occurrences are deduplicated by occurrence identity:
    /// (domain, entity_type, name, file). An entity that participates in many
    /// edges within one file is registered exactly once for that file; the
    /// same (domain, entity_type, name) in a different file remains a
    /// distinct occurrence. This is the approved ambiguity model.
    ///
    /// Entity registration happens BEFORE the edge dedup check so that
    /// entity occurrences from all files are tracked even when the edge
    /// itself is a duplicate (architectural review, Phase 4a).
    ///
    /// Registration-record normalization (index write boundary): a
    /// self-referential `Defines` edge (subject identity == object identity)
    /// is an entity-registration carrier, not a relationship. It registers the
    /// entity once with file provenance and is never inserted into `edge_set`,
    /// `file_edges`, `forward`, or `reverse`.
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

            let subj_key = entity_key(&edge.subject);
            let obj_key = entity_key(&edge.object);

            // Registration-record normalization (index write boundary):
            // a self-referential `Defines` edge (subject identity == object
            // identity) is an entity-registration carrier (e.g.
            // BuiltinMetaLayer declaring an ordinary type), NOT a semantic
            // relationship. It registers the entity once with file provenance
            // and never enters the relationship graph (edge_set / file_edges /
            // forward / reverse), so graph queries (`has_cycle`, forward and
            // reverse edges) only ever see real relationships. Real
            // relationships are unaffected: `Defines(A, B)` remains a graph
            // edge, and a non-Defines self-loop (e.g. `Injects(A, A)`) remains
            // a detected cycle (tests::has_cycle_self_loop).
            if edge.relation == SemanticRelation::Defines && subj_key == obj_key {
                self.register_entity(&subj_key, edge.subject, &mut file_entity_keys);
                continue;
            }

            // Register entities (all occurrences retained).
            self.register_entity(&subj_key, edge.subject.clone(), &mut file_entity_keys);
            self.register_entity(&obj_key, edge.object.clone(), &mut file_entity_keys);

            // Dedup at the OCCURRENCE level: only a repeat of the same triple
            // from the same asserting file is a duplicate. Both adjacency
            // indexes receive one entry per asserting occurrence.
            let key = EdgeKey::new(file_path, &edge);
            if !self.edge_set.insert(key.clone()) {
                continue;
            }
            self.edge_count += 1;

            // Track this edge occurrence under the asserting file for precise
            // removal on recompilation or file deletion.
            self.file_edges
                .entry(file_path.to_string())
                .or_default()
                .push(key);

            // Forward index: subject -> outgoing edge occurrence.
            self.forward
                .entry(subj_key)
                .or_default()
                .push(StoredEdge::new(file_path, edge.clone()));

            // Reverse index: object -> incoming edge occurrence.
            self.reverse
                .entry(obj_key)
                .or_default()
                .push(StoredEdge::new(file_path, edge));
        }

        // Track file -> entity keys for provenance. One entry per unique
        // (entity identity, file) occurrence (the same occurrence identity
        // rule applied during registration) so `entities_in_file` can never
        // report the same occurrence twice even if a file is ingested twice
        // without an intervening `remove_file`.
        if !file_entity_keys.is_empty() {
            let entry = self.file_map.entry(file_path.to_string()).or_default();
            for key in file_entity_keys.iter() {
                if !entry.contains(key) {
                    entry.push(key.clone());
                }
            }
        }
    }
}
