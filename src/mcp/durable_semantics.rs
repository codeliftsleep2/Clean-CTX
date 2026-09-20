//! Session and durable ownership for complete semantic-edge snapshots.

use crate::ir::delta::SequenceDelta;
use crate::layers::meta::semantic::{CallEvidence, EntityRef, SemanticEdge, SemanticRelation};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PendingTransitionKey {
    alias: String,
    from: u64,
    to: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingSemanticTransition {
    pub durable_file: String,
    pub from: u64,
    pub to: u64,
    pub target_source_hash: String,
    pub delta_identity: String,
    pub semantic_edges: Vec<SemanticEdge>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct DurableSemanticSnapshot {
    pub file_path: String,
    pub source_hash: String,
    pub version: u64,
    pub edges: Vec<DurableSemanticEdge>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct DurableSemanticEdge {
    relation: SemanticRelation,
    subject: DurableEntityRef,
    object: DurableEntityRef,
    layer: String,
    call_evidence: Option<CallEvidence>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct DurableEntityRef {
    domain: String,
    entity_type: String,
    name: String,
    file: Option<String>,
}

impl DurableSemanticSnapshot {
    pub(crate) fn new(
        file_path: String,
        source_hash: String,
        version: u64,
        edges: &[SemanticEdge],
    ) -> Self {
        Self {
            file_path,
            source_hash,
            version,
            edges: edges.iter().map(DurableSemanticEdge::from).collect(),
        }
    }

    pub(crate) fn restore_edges(&self) -> Result<Vec<SemanticEdge>, String> {
        self.edges.iter().map(SemanticEdge::try_from).collect()
    }
}

impl From<&SemanticEdge> for DurableSemanticEdge {
    fn from(edge: &SemanticEdge) -> Self {
        Self {
            relation: edge.relation,
            subject: DurableEntityRef::from(&edge.subject),
            object: DurableEntityRef::from(&edge.object),
            layer: edge.layer.to_string(),
            call_evidence: edge.call_evidence,
        }
    }
}

impl From<&EntityRef> for DurableEntityRef {
    fn from(entity: &EntityRef) -> Self {
        Self {
            domain: entity.domain.to_string(),
            entity_type: entity.entity_type.to_string(),
            name: entity.name.clone(),
            file: entity.file.clone(),
        }
    }
}

impl TryFrom<&DurableSemanticEdge> for SemanticEdge {
    type Error = String;

    fn try_from(edge: &DurableSemanticEdge) -> Result<Self, Self::Error> {
        Ok(Self {
            relation: edge.relation,
            subject: edge.subject.restore()?,
            object: edge.object.restore()?,
            layer: intern_label(&edge.layer)?,
            call_evidence: edge.call_evidence,
        })
    }
}

impl DurableEntityRef {
    fn restore(&self) -> Result<EntityRef, String> {
        Ok(EntityRef {
            domain: intern_label(&self.domain)?,
            entity_type: intern_label(&self.entity_type)?,
            name: self.name.clone(),
            file: self.file.clone(),
        })
    }
}

fn intern_label(value: &str) -> Result<&'static str, String> {
    static LABELS: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let mut labels = LABELS
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .map_err(|_| "durable semantic label registry is unavailable".to_string())?;
    if let Some(existing) = labels.get(value) {
        return Ok(*existing);
    }
    let value: &'static str = Box::leak(value.to_string().into_boxed_str());
    labels.insert(value);
    Ok(value)
}

pub(crate) fn sequence_delta_identity(delta: &SequenceDelta) -> Result<String, String> {
    let bytes = serde_json::to_vec(delta)
        .map_err(|error| format!("delta identity encoding failed: {error}"))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

impl super::McpState {
    pub(crate) fn remember_semantic_edges(&self, alias: &str, edges: Vec<SemanticEdge>) {
        lock_or_recover!(
            self.semantic_edge_snapshots.lock(),
            "semantic_edge_snapshots"
        )
        .insert(alias.to_string(), edges);
    }

    pub(crate) fn semantic_edges(&self, alias: &str) -> Option<Vec<SemanticEdge>> {
        lock_or_recover!(
            self.semantic_edge_snapshots.lock(),
            "semantic_edge_snapshots"
        )
        .get(alias)
        .cloned()
    }

    pub(crate) fn remember_pending_transition(
        &self,
        alias: &str,
        durable_file: &str,
        delta: &SequenceDelta,
        target_source_hash: String,
        semantic_edges: Vec<SemanticEdge>,
    ) -> Result<(), String> {
        if delta.target_hash.as_deref() != Some(target_source_hash.as_str()) {
            return Err("generated delta target hash does not match semantic state".to_string());
        }
        let key = PendingTransitionKey {
            alias: alias.to_string(),
            from: delta.from,
            to: delta.to,
        };
        let transition = PendingSemanticTransition {
            durable_file: durable_file.to_string(),
            from: delta.from,
            to: delta.to,
            target_source_hash,
            delta_identity: sequence_delta_identity(delta)?,
            semantic_edges,
        };
        lock_or_recover!(
            self.pending_semantic_transitions.lock(),
            "pending_semantic_transitions"
        )
        .insert(key, transition);
        Ok(())
    }

    pub(crate) fn pending_transition(
        &self,
        alias: &str,
        durable_file: &str,
        delta: &SequenceDelta,
    ) -> Result<PendingSemanticTransition, String> {
        let key = PendingTransitionKey {
            alias: alias.to_string(),
            from: delta.from,
            to: delta.to,
        };
        let transition = lock_or_recover!(
            self.pending_semantic_transitions.lock(),
            "pending_semantic_transitions"
        )
        .get(&key)
        .cloned()
        .ok_or_else(|| "missing authoritative semantic-edge snapshot for delta".to_string())?;
        if transition.durable_file != durable_file {
            return Err("pending semantic-edge snapshot has wrong file identity".to_string());
        }
        if delta.target_hash.as_deref() != Some(transition.target_source_hash.as_str()) {
            return Err("pending semantic-edge snapshot has wrong target hash".to_string());
        }
        if transition.delta_identity != sequence_delta_identity(delta)? {
            return Err(
                "pending semantic-edge snapshot does not match delta transition".to_string(),
            );
        }
        Ok(transition)
    }

    pub(crate) fn consume_pending_transition(&self, alias: &str, from: u64, to: u64) {
        lock_or_recover!(
            self.pending_semantic_transitions.lock(),
            "pending_semantic_transitions"
        )
        .remove(&PendingTransitionKey {
            alias: alias.to_string(),
            from,
            to,
        });
    }

    pub(crate) fn forget_semantic_state(&self, alias: &str) {
        lock_or_recover!(
            self.semantic_edge_snapshots.lock(),
            "semantic_edge_snapshots"
        )
        .remove(alias);
        lock_or_recover!(
            self.pending_semantic_transitions.lock(),
            "pending_semantic_transitions"
        )
        .retain(|key, _| key.alias != alias);
    }
}
