//! Owned direct evidence views and use-boundary validation; no canonical admission.
use super::{Declaration, Edge, Evidence, Manifest, Obligation};
use crate::ir::source_observations::{
    Span,
    request_budget::{Cost, Permit, Request, Reserved, Stop},
};
use crate::ir::typescript_resolution::Witness;
use std::path::Path;
impl Evidence {
    pub(crate) fn witnesses(&self) -> &[Witness] {
        &self.witnesses
    }
    pub(crate) fn retained_estimate(&self) -> u64 {
        self._bounded_proof
            .as_ref()
            .map_or(0, Permit::reserved_bytes)
    }
    /// Exact authorized root list only; not semantic binding equivalence.
    pub fn same_scope(&self, other: &Self) -> bool {
        self.authorization_roots == other.authorization_roots
    }
    /// Exact consumed source/configuration/package content references; no new I/O.
    pub fn support_content(&self) -> impl Iterator<Item = (&Path, &str)> {
        self.witnesses.iter().filter_map(|w| {
            if let Witness::Content { path, hash } = w {
                Some((path.as_path(), hash.as_str()))
            } else {
                None
            }
        })
    }
    pub fn type_space(&self) -> bool {
        self.type_space
    }
    pub fn annotation(&self) -> Option<Span> {
        self.annotation
    }
    pub fn contextual_modules(&self) -> &[String] {
        &self.contextual_modules
    }
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn source_hash(&self) -> &str {
        &self.source_hash
    }
    pub fn mapping_span(&self) -> Span {
        self.mapping_span
    }
    pub fn direct_target(&self) -> Option<&Declaration> {
        self.target.as_ref()
    }
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }
    pub fn manifests(&self) -> &[Manifest] {
        &self.manifests
    }
    pub fn obligations(&self) -> &[Obligation] {
        &self.obligations
    }
    /// Only module-local lexical declarations receive this classification.
    /// No runtime execution, Angular role or canonical registration is implied.
    pub fn module_local_binding_established(&self) -> bool {
        !self.type_space && self.target.is_some() && self.obligations.is_empty()
    }
    pub fn serialized_witness_bytes(&self) -> usize {
        serde_json::to_vec(&self.witnesses)
            .expect("witness serialization")
            .len()
    }
    pub fn validate(&self, request: &Request) -> Result<Reserved<bool>, Stop> {
        if request.is_integrated() {
            return super::super::bounded::validate(request, &self.witnesses);
        }
        let permit = request.reserve_finalization(Cost {
            validation_bytes: self.validation_bytes,
            validation_checks: self.witnesses.len() as u64,
            ..Cost::default()
        })?;
        // Final validation is an explicitly declared non-interruptible quantum.
        request.finish_validation(permit, || Witness::all_current(&self.witnesses))
    }
}
