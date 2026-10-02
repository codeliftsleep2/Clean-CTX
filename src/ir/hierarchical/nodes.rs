use crate::ir::opcodes::{
    ControlSummary, DeclarationModifier, ExecutionContextKind, PatternFact, SideEffectKind,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalIR {
    #[serde(rename = "c")]
    pub classes: Vec<ClassNode>,
    #[serde(rename = "if", default, skip_serializing_if = "Vec::is_empty")]
    pub interfaces: Vec<InterfaceNode>,
    #[serde(rename = "i", default, skip_serializing_if = "Vec::is_empty")]
    pub imports: Vec<Vec<String>>,
    #[serde(rename = "t", default, skip_serializing_if = "Vec::is_empty")]
    pub type_aliases: Vec<Vec<String>>,
    #[serde(rename = "ca", default, skip_serializing_if = "Vec::is_empty")]
    pub calls: Vec<HierarchicalCall>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceNode {
    #[serde(rename = "n")]
    pub id: String,
    #[serde(rename = "nm")]
    pub name: String,
    #[serde(rename = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodNode>,
    #[serde(rename = "f", default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldNode>,
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,
    #[serde(rename = "x", default, skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HierarchicalCall {
    #[serde(rename = "c")]
    pub caller: String,
    #[serde(rename = "n")]
    pub callee: String,
    #[serde(rename = "a")]
    pub explicit_arg_count: usize,
    #[serde(rename = "s", default, skip_serializing_if = "spread_is_absent")]
    pub has_spread: bool,
}

fn spread_is_absent(has_spread: &bool) -> bool {
    !has_spread
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClassNode {
    #[serde(rename = "n")]
    pub id: String,
    #[serde(rename = "nm")]
    pub name: String,
    #[serde(rename = "m", default, skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<MethodNode>,
    #[serde(rename = "f", default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldNode>,
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,
    #[serde(rename = "fl", default, skip_serializing_if = "Vec::is_empty")]
    pub class_flags: Vec<Vec<String>>,
    #[serde(rename = "x", default, skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
    /// Written first C# base-list targets whose class/interface kind is unknown.
    #[serde(rename = "br", default, skip_serializing_if = "Vec::is_empty")]
    pub base_type_refs: Vec<String>,
    #[serde(rename = "im", default, skip_serializing_if = "Vec::is_empty")]
    pub implements: Vec<String>,
    #[serde(rename = "ij", default, skip_serializing_if = "Vec::is_empty")]
    pub injects: Vec<Vec<String>>,
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,
    #[serde(rename = "sy", default, skip_serializing_if = "is_false")]
    pub synthetic: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodNode {
    #[serde(rename = "n")]
    pub id: String,
    #[serde(rename = "nm")]
    pub name: String,
    #[serde(rename = "p", default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Vec<String>>,
    #[serde(rename = "r", default, skip_serializing_if = "Option::is_none")]
    pub return_type: Option<String>,
    #[serde(rename = "mo", default, skip_serializing_if = "Vec::is_empty")]
    pub modifiers: Vec<Vec<DeclarationModifier>>,
    #[serde(rename = "cs", default, skip_serializing_if = "Vec::is_empty")]
    pub control_summaries: Vec<Vec<ControlSummary>>,
    #[serde(rename = "pf", default, skip_serializing_if = "Vec::is_empty")]
    pub pattern_facts: Vec<Vec<PatternFact>>,
    #[serde(rename = "fl", default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<Vec<String>>,
    #[serde(rename = "pa", default, skip_serializing_if = "Vec::is_empty")]
    pub patterns: Vec<PatternEntry>,
    #[serde(rename = "b", default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(rename = "bs", default, skip_serializing_if = "Option::is_none")]
    pub body_start: Option<u64>,
    #[serde(rename = "be", default, skip_serializing_if = "Option::is_none")]
    pub body_end: Option<u64>,
    #[serde(rename = "cf", default, skip_serializing_if = "Vec::is_empty")]
    pub control_flow: Vec<Vec<String>>,
    #[serde(rename = "df", default, skip_serializing_if = "Vec::is_empty")]
    pub data_flow: Vec<Vec<String>>,
    #[serde(rename = "se", default, skip_serializing_if = "Vec::is_empty")]
    pub side_effect: Vec<SideEffectKind>,
    #[serde(rename = "ec", default, skip_serializing_if = "Vec::is_empty")]
    pub execution_context: Vec<ExecutionContextKind>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldNode {
    #[serde(rename = "n")]
    pub id: String,
    #[serde(rename = "nm")]
    pub name: String,
    #[serde(rename = "t", default, skip_serializing_if = "Option::is_none")]
    pub field_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatternEntry {
    #[serde(rename = "n")]
    pub name: String,
    #[serde(rename = "a", default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}
