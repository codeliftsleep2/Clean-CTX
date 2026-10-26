//! Exact demanded value occurrences; owner-only reference_sites stays unchanged.
use super::{DeclarationId, Resolution, TypeScriptAuthority, Witness};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) enum DemandedValue {
    Reference(Resolution),
    Literal { kind: String, text: String },
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
