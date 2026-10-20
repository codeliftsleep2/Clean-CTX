use super::requirement::{AcquisitionTarget, EvidenceRequirement, SourceKind};
use super::satisfaction::AcquisitionSatisfaction;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct AcquisitionCacheKey {
    pub(crate) target: AcquisitionTarget,
    pub(crate) roots: Vec<String>,
    pub(crate) admission_fingerprint: String,
    pub(crate) producer_generations: Vec<(SourceKind, String)>,
    pub(crate) capability_version: u32,
    pub(crate) predicate_version: u32,
    pub(crate) proof_contract_version: u32,
    pub(crate) evidence: EvidenceRequirement,
}

#[derive(Clone)]
struct AcquisitionCacheEntry {
    root_generations: Vec<(String, u64)>,
    satisfaction: AcquisitionSatisfaction,
}

#[derive(Default)]
pub(crate) struct AcquisitionCompletionCache {
    root_generations: HashMap<String, u64>,
    completed: HashMap<AcquisitionCacheKey, AcquisitionCacheEntry>,
}

impl AcquisitionCompletionCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn get(&self, key: &AcquisitionCacheKey) -> Option<AcquisitionSatisfaction> {
        let entry = self.completed.get(key)?;
        entry
            .root_generations
            .iter()
            .all(|(root, generation)| self.generation(root) == *generation)
            .then(|| entry.satisfaction.clone())
    }

    pub(crate) fn commit(
        &mut self,
        key: AcquisitionCacheKey,
        satisfaction: AcquisitionSatisfaction,
    ) -> bool {
        if !satisfaction.is_source_complete() {
            return false;
        }
        let root_generations = key
            .roots
            .iter()
            .map(|root| {
                let generation = *self.root_generations.entry(root.clone()).or_insert(0);
                (root.clone(), generation)
            })
            .collect();
        self.completed.insert(
            key,
            AcquisitionCacheEntry {
                root_generations,
                satisfaction,
            },
        );
        true
    }

    pub(crate) fn invalidate_root(&mut self, root: &str) {
        if let Some(generation) = self.root_generations.get_mut(root) {
            *generation = generation.wrapping_add(1);
        }
    }

    pub(crate) fn invalidate_all(&mut self) {
        for generation in self.root_generations.values_mut() {
            *generation = generation.wrapping_add(1);
        }
    }

    fn generation(&self, root: &str) -> u64 {
        self.root_generations.get(root).copied().unwrap_or(0)
    }
}
