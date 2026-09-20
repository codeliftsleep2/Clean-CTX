//! Durable identity normalization for canonical IR persistence.

use crate::ir::compiler::CompiledIR;
use crate::ir::delta::{IRDelta, SequenceDelta};

pub(crate) fn baseline(ir: &CompiledIR, file_path: &str) -> CompiledIR {
    let mut durable = ir.clone();
    durable.file_id = file_path.to_string();
    durable
}

pub(crate) enum PersistedDelta {
    Sequence(SequenceDelta),
    Legacy(IRDelta),
}

impl PersistedDelta {
    pub(crate) fn from_json(payload: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let value: serde_json::Value = serde_json::from_slice(payload)?;
        if value.get("dv").is_some() {
            Ok(Self::Sequence(serde_json::from_value(value)?))
        } else {
            Ok(Self::Legacy(serde_json::from_value(value)?))
        }
    }

    pub(crate) fn normalize_sequence(
        delta: &SequenceDelta,
        file_path: &str,
    ) -> Result<Vec<u8>, serde_json::Error> {
        let mut durable = delta.clone();
        durable.file = file_path.to_string();
        serde_json::to_vec(&durable)
    }

    pub(crate) fn normalize_legacy(
        delta: &IRDelta,
        file_path: &str,
    ) -> Result<Vec<u8>, serde_json::Error> {
        let mut durable = delta.clone();
        durable.file = file_path.to_string();
        serde_json::to_vec(&durable)
    }
}
