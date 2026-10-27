//! Internal typed evidence carried beside a projected endpoint.
//! This does not participate in EntityRef equality or public serialization.

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct CanonicalDeclaration {
    pub source: std::path::PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub(crate) enum CanonicalEndpoint {
    InvocationOccurrence {
        invocation_occurrence: InvocationOccurrence,
    },
    RouteOccurrence {
        route_occurrence: RouteOccurrence,
    },
    Declaration {
        declaration: CanonicalDeclaration,
        site: Option<(usize, usize)>,
    },
    StringValue {
        string_value: Vec<u16>,
        proof: LiteralProof,
    },
}

/// A source-owned written call, distinct from every declaration and route.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct InvocationOccurrence {
    pub source: std::path::PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
}

/// A route object is an occurrence under a proved configuration declaration,
/// never a fabricated declaration or a path-string identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct RouteOccurrence {
    pub root: CanonicalDeclaration,
    pub source: std::path::PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub(crate) struct LiteralProof {
    /// Source-owned use occurrence. Its retained expression may reach the value
    /// through a demanded immutable initializer/alias chain in the same capture.
    pub source: std::path::PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
}

/// Deduplication identity is separate from occurrence/provenance evidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum CanonicalKey {
    InvocationOccurrence(InvocationOccurrence),
    RouteOccurrence(RouteOccurrence),
    Declaration(CanonicalDeclaration, Option<(usize, usize)>),
    StringValue(Vec<u16>),
}

impl CanonicalEndpoint {
    pub(crate) fn key(&self) -> CanonicalKey {
        match self {
            Self::InvocationOccurrence {
                invocation_occurrence,
            } => CanonicalKey::InvocationOccurrence(invocation_occurrence.clone()),
            Self::RouteOccurrence { route_occurrence } => {
                CanonicalKey::RouteOccurrence(route_occurrence.clone())
            }
            Self::Declaration { declaration, site } => {
                CanonicalKey::Declaration(declaration.clone(), *site)
            }
            Self::StringValue { string_value, .. } => {
                CanonicalKey::StringValue(string_value.clone())
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn declaration(&self) -> Option<&CanonicalDeclaration> {
        match self {
            Self::Declaration { declaration, .. } => Some(declaration),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) fn declaration_mut(&mut self) -> Option<&mut CanonicalDeclaration> {
        match self {
            Self::Declaration { declaration, .. } => Some(declaration),
            _ => None,
        }
    }
}

/// Stable quoted label without lossy UTF-16 conversion or Unicode normalization.
pub(crate) fn string_label(units: &[u16]) -> String {
    use std::fmt::Write;
    let mut label = String::from("\"");
    for &unit in units {
        match unit {
            0x22 => label.push_str("\\\""),
            0x5c => label.push_str("\\\\"),
            0x20..=0x7e => label.push(char::from_u32(u32::from(unit)).expect("ASCII")),
            _ => write!(label, "\\u{unit:04x}").expect("String formatting"),
        }
    }
    label.push('"');
    label
}
