//! Private operation-local retrieval; map traversal never establishes output order.
use crate::diff::snapshot::{CapturedMethod, DeclarationEvidence};
use std::cmp::Ordering;
use std::collections::{BTreeMap, VecDeque};

fn string_cmp(a: &str, b: &str) -> Ordering {
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|c| {
        c.overload.string_comparisons += 1;
        c.overload.string_bytes_offered += a.len() + b.len();
    });
    a.cmp(b)
}
fn sequence_cmp(a: &[String], b: &[String]) -> Ordering {
    #[cfg(test)]
    crate::work_observations_af_perf_002::update(|c| c.overload.sequence_comparisons += 1);
    for (left, right) in a.iter().zip(b) {
        let order = string_cmp(left, right);
        if order != Ordering::Equal {
            return order;
        }
    }
    a.len().cmp(&b.len())
}
type DeclarationKey<'a> = (&'a str, &'a [String], &'a [String]);
#[derive(Clone, Copy)]
struct ExactKey<'a> {
    sig: &'a str,
    declaration: Option<DeclarationKey<'a>>,
    markers: &'a [String],
    body: Option<&'a str>,
}
impl<'a> ExactKey<'a> {
    fn new(method: &'a CapturedMethod) -> Self {
        // Exhaustive patterns make added equality fields a compile-time review point.
        let CapturedMethod {
            sig,
            declaration,
            markers,
            body,
        } = method;
        let declaration = declaration.as_ref().map(|e| {
            let DeclarationEvidence {
                return_type,
                attributes,
                modifiers,
            } = e;
            (
                return_type.as_str(),
                attributes.as_slice(),
                modifiers.as_slice(),
            )
        });
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.projections += 1);
        Self {
            sig,
            declaration,
            markers,
            body: body.as_deref(),
        }
    }
}
impl Ord for ExactKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.key_comparisons[0] += 1);
        string_cmp(self.sig, other.sig)
            .then_with(|| match (self.declaration, other.declaration) {
                (Some(a), Some(b)) => string_cmp(a.0, b.0)
                    .then_with(|| sequence_cmp(a.1, b.1))
                    .then_with(|| sequence_cmp(a.2, b.2)),
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
            })
            .then_with(|| sequence_cmp(self.markers, other.markers))
            .then_with(|| match (self.body, other.body) {
                (Some(a), Some(b)) => string_cmp(a, b),
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
            })
    }
}
impl PartialOrd for ExactKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for ExactKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for ExactKey<'_> {}

pub(super) struct ExactLookup<'a>(BTreeMap<ExactKey<'a>, VecDeque<usize>>);
impl<'a> ExactLookup<'a> {
    pub(super) fn new(current: &[&'a CapturedMethod]) -> Self {
        let mut map: BTreeMap<_, VecDeque<_>> = BTreeMap::new();
        for (index, method) in current.iter().enumerate() {
            map.entry(ExactKey::new(method))
                .or_default()
                .push_back(index);
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| {
                c.overload.entries[0] += 1;
                c.overload.pushes[0] += 1;
            });
        }
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.overload.retained_keys[0] += map.len();
            c.overload.retained_queues[0] += map.len();
            c.overload.queue_capacity[0] += map.values().map(VecDeque::capacity).sum::<usize>();
        });
        Self(map)
    }
    pub(super) fn take(&mut self, method: &'a CapturedMethod) -> Option<usize> {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.lookups[0] += 1);
        let result = self
            .0
            .get_mut(&ExactKey::new(method))
            .and_then(VecDeque::pop_front);
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.overload.pops[0] += usize::from(result.is_some())
        });
        result
    }
}
struct SignatureKey<'a>(&'a str);
impl Ord for SignatureKey<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.key_comparisons[1] += 1);
        string_cmp(self.0, other.0)
    }
}
impl PartialOrd for SignatureKey<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for SignatureKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for SignatureKey<'_> {}
pub(super) struct SignatureLookup<'a>(BTreeMap<SignatureKey<'a>, VecDeque<usize>>);
impl<'a> SignatureLookup<'a> {
    pub(super) fn new(current: &[&'a CapturedMethod], used: &[bool]) -> Self {
        let mut map: BTreeMap<_, VecDeque<_>> = BTreeMap::new();
        for (index, method) in current.iter().enumerate() {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| c.overload.used_flag_visits += 1);
            if !used[index] {
                map.entry(SignatureKey(&method.sig))
                    .or_default()
                    .push_back(index);
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|c| {
                    c.overload.entries[1] += 1;
                    c.overload.pushes[1] += 1;
                });
            }
        }
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.overload.retained_keys[1] += map.len();
            c.overload.retained_queues[1] += map.len();
            c.overload.queue_capacity[1] += map.values().map(VecDeque::capacity).sum::<usize>();
        });
        Self(map)
    }
    pub(super) fn take(&mut self, signature: &'a str) -> Option<usize> {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.lookups[1] += 1);
        let result = self
            .0
            .get_mut(&SignatureKey(signature))
            .and_then(VecDeque::pop_front);
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| {
            c.overload.pops[1] += usize::from(result.is_some())
        });
        result
    }
}

#[path = "overload_key_acceptance_af_perf_002.rs"]
mod acceptance;
