//! Exact demanded value occurrences; owner-only reference_sites stays unchanged.
use super::{DeclarationId, Resolution, TypeScriptAuthority, Witness};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum DemandedValue {
    Reference(Resolution),
    TypeReference(Resolution),
    Declaration {
        id: DeclarationId,
        is_const: bool,
        annotation: Option<(usize, usize)>,
        initializer: Option<(usize, usize)>,
        decorators: Vec<(usize, usize)>,
    },
    Literal {
        kind: String,
        text: String,
    },
    Structure {
        kind: String,
        children: Vec<(String, usize, usize)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct DemandedOccurrence {
    pub source: PathBuf,
    pub source_hash: String,
    pub start: usize,
    pub end: usize,
    pub value: DemandedValue,
}

impl TypeScriptAuthority {
    pub(crate) fn demanded_at(
        &self,
        source: &Path,
        hash: &str,
        start: usize,
        end: usize,
    ) -> Option<&DemandedValue> {
        if start >= end
            || !((source == self.source.as_path() && hash == self.source_hash)
                || self.witnesses.iter().any(|witness| {
                    matches!(witness, Witness::Content { path, hash: actual }
                    if path.as_path() == source && actual == hash)
                }))
        {
            return None;
        }
        let mut sites = self.demanded_occurrences.iter().filter(|site| {
            site.source == source
                && site.source_hash == hash
                && site.start == start
                && site.end == end
        });
        let site = sites.next()?;
        sites.next().is_none().then_some(&site.value)
    }

    pub(crate) fn admits_demanded_structure(
        &self,
        source: &Path,
        hash: &str,
        value: &super::declaration_facets::ExpressionFacet,
    ) -> bool {
        use super::declaration_facets::ExpressionValue;
        let Some(DemandedValue::Structure { kind, children }) =
            self.demanded_at(source, hash, value.start, value.end)
        else {
            return false;
        };
        let actual: Vec<_> = match &value.value {
            ExpressionValue::Spread(child) if kind == "spread" => {
                vec![(String::new(), child.start, child.end)]
            }
            ExpressionValue::Array(values) if kind == "array" => values
                .iter()
                .map(|v| (String::new(), v.start, v.end))
                .collect(),
            ExpressionValue::Object(fields) if kind == "object" => fields
                .iter()
                .map(|(name, v)| (name.clone(), v.start, v.end))
                .collect(),
            _ => return false,
        };
        if &actual != children {
            return false;
        }
        let mut previous = value.start;
        actual.iter().all(|(_, start, end)| {
            let valid = previous <= *start && start < end && *end <= value.end;
            previous = *end;
            valid
        })
    }

    pub(crate) fn demanded_reference_at(
        &self,
        source: &Path,
        hash: &str,
        start: usize,
        end: usize,
    ) -> Option<&DeclarationId> {
        match self.demanded_at(source, hash, start, end)? {
            DemandedValue::Reference(Resolution::Resolved(id)) => Some(id),
            _ => None,
        }
    }
}
