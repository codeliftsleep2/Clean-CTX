// Shared typed semantic-identity validation for canonical IR consumers.
//
// `CoreOp` remains the serialized DTO during this migration, so identity
// operands stay strings on the wire. This module is the single checked
// internal boundary used by production validation and hierarchical projection.

use super::CompiledIR;
use std::collections::HashMap;

mod cardinality;
mod contracts;
mod error;
mod pattern;
mod payload;

pub use error::IdentityError;
pub(crate) use pattern::{PatternTarget, pattern_target};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityKind {
    Class,
    Method,
    Field,
    Parameter,
    Interface,
    ImportAlias,
    TypeAlias,
}

impl std::fmt::Display for IdentityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Class => f.write_str("class"),
            Self::Method => f.write_str("method"),
            Self::Field => f.write_str("field"),
            Self::Parameter => f.write_str("parameter"),
            Self::Interface => f.write_str("interface"),
            Self::ImportAlias => f.write_str("import alias"),
            Self::TypeAlias => f.write_str("type alias"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ClassId(String);

impl ClassId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct InterfaceId(String);

impl InterfaceId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MethodId(String);

impl MethodId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct FieldId(String);

impl FieldId {
    pub(crate) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct ParameterId(String);

impl ParameterId {
    pub(super) fn from_serialized(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct IdentityIndex {
    pub(crate) classes: HashMap<ClassId, usize>,
    pub(crate) methods: HashMap<MethodId, usize>,
    pub(crate) fields: HashMap<FieldId, usize>,
    pub(super) method_owners: HashMap<MethodId, ClassId>,
    pub(super) interface_method_owners: HashMap<MethodId, InterfaceId>,
    pub(super) interface_field_owners: HashMap<FieldId, InterfaceId>,
    pub(crate) interfaces: HashMap<InterfaceId, usize>,
}

/// Validate every current identity, ownership, cardinality, pattern-shape,
/// body-span, and controlled-vocabulary contract without using stream position
/// as semantic authority. The exhaustive matches in `contracts` make a new
/// `CoreOp` a compiler-visible validation decision.
pub(crate) fn validate_identity_graph(ir: &CompiledIR) -> Result<IdentityIndex, IdentityError> {
    contracts::validate(ir)
}
