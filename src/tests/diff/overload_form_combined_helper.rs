//! Call-local signature discrimination; full method equality remains authority.
use crate::diff::snapshot::CapturedMethod;
use std::cmp::Ordering;

#[inline(always)]
fn signature_cmp(a: &str, b: &str) -> Ordering {
    #[cfg(test)]
    {
        crate::diff::differ::overload_alternatives::retrieval::observed_signature_cmp(a, b)
    }
    #[cfg(not(test))]
    {
        a.cmp(b)
    }
}
#[inline(always)]
fn method_equal(a: &CapturedMethod, b: &CapturedMethod) -> bool {
    // Observed test equality is checked against actual derived Eq by a full-field control.
    #[cfg(test)]
    {
        crate::diff::differ::overload_alternatives::retrieval::observed_eq(a, b)
    }
    #[cfg(not(test))]
    {
        a == b
    }
}
#[inline(always)]
fn signature_equal(a: &str, b: &str) -> bool {
    #[cfg(test)]
    {
        crate::diff::differ::overload_alternatives::retrieval::observed_sig_eq(a, b)
    }
    #[cfg(not(test))]
    {
        a == b
    }
}
struct SignatureIndex {
    indices: Vec<usize>,
    cursor: Vec<usize>,
}
pub(super) struct MethodLookup<'a, 'g> {
    current: &'g [&'a CapturedMethod],
    index: Option<SignatureIndex>,
}
impl<'a, 'g> MethodLookup<'a, 'g> {
    pub(super) fn new(current: &'g [&'a CapturedMethod]) -> Self {
        let index = if current.len() == 1 {
            None
        } else {
            let mut indices = Vec::with_capacity(current.len());
            indices.extend(0..current.len());
            indices.sort_unstable_by(|&a, &b| {
                signature_cmp(&current[a].sig, &current[b].sig).then(a.cmp(&b))
            });
            let cursor = vec![0; indices.len()];
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| {
                c.overload.entries[1] += indices.len();
                c.overload.queue_capacity[1] += indices.capacity() + cursor.capacity();
            });
            Some(SignatureIndex { indices, cursor })
        };
        Self { current, index }
    }
    fn range(&self, signature: &str) -> (usize, usize) {
        let index = self.index.as_ref().expect("indexed group");
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.lookups[1] += 1);
        let a = index
            .indices
            .partition_point(|&i| signature_cmp(&self.current[i].sig, signature) == Ordering::Less);
        let b = index.indices.partition_point(|&i| {
            signature_cmp(&self.current[i].sig, signature) != Ordering::Greater
        });
        (a, b)
    }
    pub(super) fn exact(&self, base: &CapturedMethod, used: &[bool]) -> Option<usize> {
        #[cfg(test)]
        crate::work_observations_af_perf_002::update(|c| c.overload.lookups[0] += 1);
        let Some(index) = &self.index else {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| {
                c.overload.selection_visits += 1;
                c.overload.used_flag_visits += 1;
            });
            return (!used[0] && method_equal(self.current[0], base)).then_some(0);
        };
        let (a, b) = self.range(&base.sig);
        index.indices[a..b]
            .iter()
            .find(|&&i| {
                #[cfg(test)]
                crate::work_observations_af_perf_002::update(|c| c.overload.selection_visits += 1);
                !used[i] && method_equal(self.current[i], base)
            })
            .copied()
    }
    pub(super) fn signature(&mut self, signature: &str, used: &[bool]) -> Option<usize> {
        if self.index.is_none() {
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| {
                c.overload.lookups[1] += 1;
                c.overload.selection_visits += 1;
                c.overload.used_flag_visits += 1;
            });
            return (!used[0] && signature_equal(&self.current[0].sig, signature)).then_some(0);
        }
        let (a, b) = self.range(signature);
        if a == b {
            return None;
        }
        let index = self.index.as_mut().expect("indexed group");
        while a + index.cursor[a] < b {
            let i = index.indices[a + index.cursor[a]];
            index.cursor[a] += 1;
            #[cfg(test)]
            crate::work_observations_af_perf_002::update(|c| c.overload.selection_visits += 1);
            if !used[i] {
                return Some(i);
            }
        }
        None
    }
}
