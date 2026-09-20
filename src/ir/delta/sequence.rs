//! Corrected, occurrence-aware positional delta protocol.

use super::{SemanticIntent, detect_semantic_intent};
use crate::ir::compiler::CompiledIR;
use crate::ir::wire::op_to_tuple;
use serde::{Deserialize, Serialize};

pub const SEQUENCE_DELTA_VERSION: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaOpcode {
    DefClass,
    DefMethod,
    DefField,
    DefInterface,
    DefInterfaceMethod,
    DefInterfaceField,
    Param,
    Return,
    FieldType,
    MethodModifiers,
    ClassModifiers,
    InterfaceModifiers,
    ControlSummary,
    PatternFacts,
    Flags,
    ClassFlags,
    Extends,
    InterfaceExtends,
    Implements,
    Injects,
    Import,
    TypeAlias,
    Pattern,
    Body,
    DataFlow,
    ControlFlow,
    SideEffect,
    ExecutionContext,
    Call,
}

impl DeltaOpcode {
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "DEF_C" => Some(Self::DefClass),
            "DEF_M" => Some(Self::DefMethod),
            "DEF_F" => Some(Self::DefField),
            "DEF_I" => Some(Self::DefInterface),
            "DEF_IM" => Some(Self::DefInterfaceMethod),
            "DEF_IF" => Some(Self::DefInterfaceField),
            "SIG" => Some(Self::Param),
            "RET" => Some(Self::Return),
            "FIELD_T" => Some(Self::FieldType),
            "MOD_M" => Some(Self::MethodModifiers),
            "MOD_C" => Some(Self::ClassModifiers),
            "MOD_I" => Some(Self::InterfaceModifiers),
            "CTRL_SUM" => Some(Self::ControlSummary),
            "PAT_FACT" => Some(Self::PatternFacts),
            "FLAGS" => Some(Self::Flags),
            "FLAGS_C" => Some(Self::ClassFlags),
            "EXT" => Some(Self::Extends),
            "EXT_I" => Some(Self::InterfaceExtends),
            "IMPL" => Some(Self::Implements),
            "INJECTS" => Some(Self::Injects),
            "IMP" => Some(Self::Import),
            "TYPE" => Some(Self::TypeAlias),
            "PAT" => Some(Self::Pattern),
            "BODY" => Some(Self::Body),
            "DATAFLOW" => Some(Self::DataFlow),
            "CTRL" => Some(Self::ControlFlow),
            "EFFECT" => Some(Self::SideEffect),
            "CTX" => Some(Self::ExecutionContext),
            "CALL" => Some(Self::Call),
            _ => None,
        }
    }

    pub(crate) const fn is_repeatable(self) -> bool {
        !matches!(
            self,
            Self::DefClass
                | Self::DefMethod
                | Self::DefField
                | Self::DefInterface
                | Self::DefInterfaceMethod
                | Self::DefInterfaceField
                | Self::Param
                | Self::Return
                | Self::FieldType
                | Self::Extends
                | Self::Import
                | Self::Body
        )
    }
}

/// Semantic identity before occurrence disambiguation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeltaIdentity {
    pub opcode: DeltaOpcode,
    pub key: Vec<String>,
}

impl DeltaIdentity {
    pub fn from_tuple(tuple: &[String]) -> Option<Self> {
        let opcode = DeltaOpcode::from_wire(tuple.first()?.as_str())?;
        let key_len = match opcode {
            DeltaOpcode::DefMethod
            | DeltaOpcode::DefField
            | DeltaOpcode::DefInterfaceMethod
            | DeltaOpcode::DefInterfaceField
            | DeltaOpcode::Param => 3,
            DeltaOpcode::DefClass
            | DeltaOpcode::DefInterface
            | DeltaOpcode::Return
            | DeltaOpcode::FieldType
            | DeltaOpcode::Extends
            | DeltaOpcode::InterfaceExtends
            | DeltaOpcode::Import
            | DeltaOpcode::Body => 2,
            _ => tuple.len(),
        };
        (tuple.len() >= key_len).then(|| Self {
            opcode,
            key: tuple[..key_len].to_vec(),
        })
    }

    pub(crate) const fn is_repeatable(&self) -> bool {
        self.opcode.is_repeatable()
    }
}

/// Stable ordinal among preceding operations with the same semantic identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OccurrenceKey {
    pub identity: DeltaIdentity,
    pub occurrence: usize,
}

pub(crate) fn occurrence_key_at(
    instructions: &[Vec<String>],
    position: usize,
) -> Option<OccurrenceKey> {
    let identity = DeltaIdentity::from_tuple(instructions.get(position)?)?;
    let occurrence = instructions[..position]
        .iter()
        .filter(|tuple| DeltaIdentity::from_tuple(tuple).as_ref() == Some(&identity))
        .count();
    Some(OccurrenceKey {
        identity,
        occurrence,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SequenceEdit {
    Insert {
        at: usize,
        value: OccurrenceKey,
        instruction: Vec<String>,
    },
    Remove {
        at: usize,
        target: OccurrenceKey,
        expected: Vec<String>,
    },
    Replace {
        at: usize,
        target: OccurrenceKey,
        expected: Vec<String>,
        value: OccurrenceKey,
        replacement: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceDelta {
    #[serde(rename = "dv")]
    pub delta_version: u8,
    pub file: String,
    pub from: u64,
    pub to: u64,
    pub edits: Vec<SequenceEdit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<SemanticIntent>,
}

impl SequenceDelta {
    pub fn empty(file: impl Into<String>, from: u64, to: u64) -> Self {
        Self {
            delta_version: SEQUENCE_DELTA_VERSION,
            file: file.into(),
            from,
            to,
            edits: Vec::new(),
            intent: None,
        }
    }
}

pub struct SequenceDeltaComputer;

impl SequenceDeltaComputer {
    pub const fn new() -> Self {
        Self
    }

    pub fn compute(&self, baseline: &CompiledIR, current: &CompiledIR) -> Option<SequenceDelta> {
        let intent = detect_semantic_intent(baseline, current);
        let baseline_version = baseline.version;
        let baseline_tuples = baseline
            .instructions
            .iter()
            .map(op_to_tuple)
            .collect::<Vec<_>>();
        let target = current
            .instructions
            .iter()
            .map(op_to_tuple)
            .collect::<Vec<_>>();
        if baseline_tuples == target {
            return None;
        }

        let mut prefix = 0;
        while prefix < baseline_tuples.len().min(target.len())
            && baseline_tuples[prefix] == target[prefix]
        {
            prefix += 1;
        }
        let mut suffix = 0;
        while suffix < baseline_tuples.len().saturating_sub(prefix)
            && suffix < target.len().saturating_sub(prefix)
            && baseline_tuples[baseline_tuples.len() - 1 - suffix]
                == target[target.len() - 1 - suffix]
        {
            suffix += 1;
        }

        let base_middle = baseline_tuples.len() - prefix - suffix;
        let target_middle = target.len() - prefix - suffix;
        let replacements = base_middle.min(target_middle);
        let mut work = baseline_tuples;
        let mut edits = Vec::new();

        for offset in 0..replacements {
            let at = prefix + offset;
            let expected = work[at].clone();
            let target_key = occurrence_key_at(&work, at).expect("canonical tuple identity");
            let replacement = target[at].clone();
            work[at] = replacement.clone();
            let value = occurrence_key_at(&work, at).expect("replacement tuple identity");
            edits.push(SequenceEdit::Replace {
                at,
                target: target_key,
                expected,
                value,
                replacement,
            });
        }

        for _ in replacements..base_middle {
            let at = prefix + replacements;
            let expected = work[at].clone();
            let target_key = occurrence_key_at(&work, at).expect("canonical tuple identity");
            work.remove(at);
            edits.push(SequenceEdit::Remove {
                at,
                target: target_key,
                expected,
            });
        }

        for offset in replacements..target_middle {
            let at = prefix + offset;
            let instruction = target[at].clone();
            work.insert(at, instruction.clone());
            let value = occurrence_key_at(&work, at).expect("canonical tuple identity");
            edits.push(SequenceEdit::Insert {
                at,
                value,
                instruction,
            });
        }

        debug_assert_eq!(work, target);
        Some(SequenceDelta {
            delta_version: SEQUENCE_DELTA_VERSION,
            file: current.file_id.clone(),
            from: baseline_version,
            to: current.version,
            edits,
            intent,
        })
    }
}

impl Default for SequenceDeltaComputer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactSequenceDelta {
    #[serde(rename = "d")]
    pub delta_version: u8,
    #[serde(rename = "f")]
    pub file: String,
    #[serde(rename = "v")]
    pub versions: [u64; 2],
    #[serde(rename = "e")]
    pub edits: Vec<SequenceEdit>,
    #[serde(rename = "i", skip_serializing_if = "Option::is_none")]
    pub intent: Option<SemanticIntent>,
}

pub fn compact_sequence_encode(delta: &SequenceDelta) -> CompactSequenceDelta {
    CompactSequenceDelta {
        delta_version: delta.delta_version,
        file: delta.file.clone(),
        versions: [delta.from, delta.to],
        edits: delta.edits.clone(),
        intent: delta.intent.clone(),
    }
}

pub fn compact_sequence_decode(compact: &CompactSequenceDelta) -> SequenceDelta {
    SequenceDelta {
        delta_version: compact.delta_version,
        file: compact.file.clone(),
        from: compact.versions[0],
        to: compact.versions[1],
        edits: compact.edits.clone(),
        intent: compact.intent.clone(),
    }
}
