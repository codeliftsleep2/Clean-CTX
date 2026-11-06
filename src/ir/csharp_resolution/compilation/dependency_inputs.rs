//! Call-local borrowed original inputs; never resolution or currentness authority.
use super::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

// Transparent borrowed values allow test observation of actual BTree comparisons.
// SITE distinguishes family keys, file keys and seed keys; no runtime state.
#[derive(PartialEq, Eq)]
struct Key<'a, T: Ord + ?Sized, const SITE: usize>(&'a T);
impl<T: Ord + ?Sized, const SITE: usize> Ord for Key<'_, T, SITE> {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.a4.lookup_key_comparisons[SITE] += 1);
        self.0.cmp(other.0)
    }
}
impl<T: Ord + ?Sized, const SITE: usize> PartialOrd for Key<'_, T, SITE> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
pub(super) struct OriginalInputs<'a> {
    families: BTreeMap<Key<'a, CSharpSymbolKey, 0>, Vec<&'a ClassInput>>,
    seeds: BTreeMap<Key<'a, str, 1>, BTreeSet<Key<'a, CSharpSymbolKey, 2>>>,
}
impl<'a> OriginalInputs<'a> {
    pub(super) fn insert(&mut self, input: &'a ClassInput) {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.a4.lookup_input_visits += 1;
            for attempts in &mut c.a4.lookup_insert_attempts {
                *attempts += 1;
            }
        });
        self.families
            .entry(Key(&input.part.key))
            .or_default()
            .push(input);
        self.seeds
            .entry(Key(input.part.file.as_str()))
            .or_default()
            .insert(Key(&input.part.key));
    }
    pub(super) fn seeds<'b>(
        &'b self,
        file: &'b str,
    ) -> impl Iterator<Item = &'b CSharpSymbolKey> + use<'a, 'b> {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.a4.lookup_calls[0] += 1);
        self.seeds
            .get(&Key(file))
            .into_iter()
            .flatten()
            .map(|key| key.0)
    }
    pub(super) fn family<'b>(&'b self, key: &'b CSharpSymbolKey) -> &'b [&'b ClassInput] {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.a4.lookup_calls[1] += 1);
        self.families.get(&Key(key)).map_or(&[], Vec::as_slice)
    }
    #[cfg(test)]
    pub(super) fn observe_storage(&self) {
        crate::work_observations_af_perf_002::update(|c| {
            c.a4.retained_input_references += self.families.values().map(Vec::len).sum::<usize>();
            c.a4.retained_seed_references += self.seeds.values().map(BTreeSet::len).sum::<usize>();
            c.a4.retained_family_vectors += self.families.len();
            c.a4.retained_file_sets += self.seeds.len();
            c.a4.family_reference_capacity +=
                self.families.values().map(Vec::capacity).sum::<usize>();
        });
    }
}
