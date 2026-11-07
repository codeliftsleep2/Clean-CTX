//! Operation-local exact-end retrieval; source captures remain declaration authority.
use super::UnitRecord;
use std::cmp::Ordering;
use std::collections::BTreeMap;

type Declaration = (u64, u64, Option<u64>);

#[derive(Clone, Copy, PartialEq, Eq)]
struct End(u64);
impl Ord for End {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.edit.end_key_comparisons += 1);
        self.0.cmp(&other.0)
    }
}
impl PartialOrd for End {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Default)]
struct EndBucket {
    declarations: Vec<Declaration>,
    max_start: Option<u64>,
}
pub(super) struct EndJoin(BTreeMap<End, EndBucket>);
impl EndJoin {
    pub(super) fn new(declarations: Vec<Declaration>, units: &[UnitRecord]) -> Self {
        let mut buckets = BTreeMap::<End, EndBucket>::new();
        for declaration in declarations {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| c.edit.end_setup_declarations += 1);
            buckets
                .entry(End(declaration.1))
                .or_default()
                .declarations
                .push(declaration);
        }
        for unit in units {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| c.edit.end_setup_units += 1);
            let bucket = buckets.entry(End(unit.end_byte)).or_default();
            bucket.max_start = Some(
                bucket
                    .max_start
                    .map_or(unit.start_byte, |start| start.max(unit.start_byte)),
            );
        }
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.edit.end_buckets += buckets.len();
            c.edit.end_retained_declarations += buckets
                .values()
                .map(|b| b.declarations.len())
                .sum::<usize>();
            c.edit.end_declaration_capacity += buckets
                .values()
                .map(|b| b.declarations.capacity())
                .sum::<usize>();
        });
        Self(buckets)
    }
    pub(super) fn declaration(&self, unit: &UnitRecord) -> Option<Declaration> {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.edit.end_attachment_lookups += 1);
        self.0
            .get(&End(unit.end_byte))
            .into_iter()
            .flat_map(|bucket| &bucket.declarations)
            .copied()
            .filter(|(start, end, _)| {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|c| {
                    c.edit.end_same_end_candidates += 1
                });
                *start <= unit.start_byte && *end == unit.end_byte
            })
            .min_by_key(|(start, end, _)| end - start)
    }
    pub(super) fn editable(&self, start: u64, end: u64) -> bool {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.edit.end_editability_lookups += 1);
        self.0
            .get(&End(end))
            .and_then(|bucket| bucket.max_start)
            .is_some_and(|maximum| maximum >= start)
    }
}
