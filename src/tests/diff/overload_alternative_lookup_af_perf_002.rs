//! Test-only candidate retrieval. No production matcher delegates here.
use crate::diff::snapshot::{CapturedMethod, DeclarationEvidence};
use crate::work_observations_af_perf_002 as work;
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub(crate) enum Design {
    Partition,
    Refined,
    Capacity,
    Sorted,
    Hash,
    Collisions,
}
thread_local! {static DESIGN:Cell<Design>=const {Cell::new(Design::Partition)};}
pub(crate) fn with_design<T>(design: Design, f: impl FnOnce() -> T) -> T {
    struct Restore(Design);
    impl Drop for Restore {
        fn drop(&mut self) {
            DESIGN.with(|d| d.set(self.0));
        }
    }
    let _restore = Restore(DESIGN.with(|d| d.replace(design)));
    f()
}
fn string_cmp(a: &str, b: &str, payload: bool) -> Ordering {
    work::update(|c| {
        c.overload.string_comparisons += 1;
        c.overload.string_bytes_offered += a.len() + b.len();
        if payload {
            c.overload.payload_comparisons += 1;
            c.overload.payload_bytes_offered += a.len() + b.len();
        } else {
            c.overload.discrimination_comparisons += 1;
            c.overload.discrimination_bytes_offered += a.len() + b.len();
        }
    });
    a.cmp(b)
}
fn sequence_cmp(a: &[String], b: &[String]) -> Ordering {
    work::update(|c| c.overload.sequence_comparisons += 1);
    for (a, b) in a.iter().zip(b) {
        let o = string_cmp(a, b, true);
        if o != Ordering::Equal {
            return o;
        }
    }
    a.len().cmp(&b.len())
}
fn declaration_cmp(a: &Option<DeclarationEvidence>, b: &Option<DeclarationEvidence>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => string_cmp(&a.return_type, &b.return_type, true)
            .then_with(|| sequence_cmp(&a.attributes, &b.attributes))
            .then_with(|| sequence_cmp(&a.modifiers, &b.modifiers)),
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
    }
}
fn payload_cmp(a: &CapturedMethod, b: &CapturedMethod) -> Ordering {
    work::update(|c| c.overload.key_comparisons[1] += 1);
    string_cmp(&a.sig, &b.sig, false)
        .then_with(|| declaration_cmp(&a.declaration, &b.declaration))
        .then_with(|| sequence_cmp(&a.markers, &b.markers))
        .then_with(|| match (&a.body, &b.body) {
            (Some(a), Some(b)) => string_cmp(a, b, true),
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
        })
}
// Equality uses equality operations, not lexicographic ordering. Each compared
// field is observed before the same short circuit; actual inspected bytes unknown.
fn string_eq(a: &str, b: &str, payload: bool) -> bool {
    work::update(|c| {
        c.overload.string_comparisons += 1;
        c.overload.string_bytes_offered += a.len() + b.len();
        if payload {
            c.overload.payload_comparisons += 1;
            c.overload.payload_bytes_offered += a.len() + b.len();
        } else {
            c.overload.discrimination_comparisons += 1;
            c.overload.discrimination_bytes_offered += a.len() + b.len();
        }
    });
    a == b
}
fn seq_eq(a: &[String], b: &[String]) -> bool {
    work::update(|c| c.overload.sequence_comparisons += 1);
    a.len() == b.len() && a.iter().zip(b).all(|(a, b)| string_eq(a, b, true))
}
pub(crate) fn observed_eq(a: &CapturedMethod, b: &CapturedMethod) -> bool {
    let CapturedMethod {
        sig,
        declaration,
        markers,
        body,
    } = a;

    work::update(|c| {
        c.overload.key_comparisons[1] += 1;
        c.overload.full_equalities += 1;
    });
    string_eq(sig, &b.sig, false)
        && match (declaration, &b.declaration) {
            (None, None) => true,
            (Some(a), Some(b)) => {
                let DeclarationEvidence {
                    return_type,
                    attributes,
                    modifiers,
                } = a;
                string_eq(return_type, &b.return_type, true)
                    && seq_eq(attributes, &b.attributes)
                    && seq_eq(modifiers, &b.modifiers)
            }
            _ => false,
        }
        && seq_eq(markers, &b.markers)
        && match (body, &b.body) {
            (None, None) => true,
            (Some(a), Some(b)) => string_eq(a, b, true),
            _ => false,
        }
}
pub(crate) fn observed_sig_eq(a: &str, b: &str) -> bool {
    string_eq(a, b, false)
}
pub(crate) fn observed_signature_cmp(a: &str, b: &str) -> Ordering {
    string_cmp(a, b, false)
}
struct SignatureIndex {
    indices: Vec<usize>,
    cursor: Vec<usize>,
}
impl SignatureIndex {
    fn exact_capacity(current: &[&CapturedMethod]) -> Self {
        let mut indices = Vec::with_capacity(current.len());
        indices.extend(0..current.len());
        indices.sort_unstable_by(|&a, &b| {
            string_cmp(&current[a].sig, &current[b].sig, false).then(a.cmp(&b))
        });
        let cursor = vec![0; indices.len()];
        work::update(|c| {
            c.overload.entries[1] += indices.len();
            c.overload.queue_capacity[1] += indices.capacity() + cursor.capacity();
        });
        Self { indices, cursor }
    }
    fn new(current: &[&CapturedMethod], used: Option<&[bool]>) -> Self {
        let mut indices: Vec<_> = (0..current.len())
            .filter(|&i| {
                work::update(|c| c.overload.used_flag_visits += usize::from(used.is_some()));
                used.is_none_or(|u| !u[i])
            })
            .collect();
        indices.sort_unstable_by(|&a, &b| {
            string_cmp(&current[a].sig, &current[b].sig, false).then(a.cmp(&b))
        });
        let cursor = vec![0; indices.len()];
        work::update(|c| {
            c.overload.entries[1] += indices.len();
            c.overload.queue_capacity[1] += indices.capacity() + cursor.capacity();
        });
        Self { indices, cursor }
    }
    fn range(&self, current: &[&CapturedMethod], sig: &str) -> (usize, usize) {
        work::update(|c| c.overload.lookups[1] += 1);
        let a = self
            .indices
            .partition_point(|&i| string_cmp(&current[i].sig, sig, false) == Ordering::Less);
        let b = self
            .indices
            .partition_point(|&i| string_cmp(&current[i].sig, sig, false) != Ordering::Greater);
        (a, b)
    }
    fn signature(
        &mut self,
        current: &[&CapturedMethod],
        used: &[bool],
        sig: &str,
    ) -> Option<usize> {
        let (a, b) = self.range(current, sig);
        if a == b {
            return None;
        }
        while a + self.cursor[a] < b {
            let i = self.indices[a + self.cursor[a]];
            self.cursor[a] += 1;
            work::update(|c| c.overload.selection_visits += 1);
            if !used[i] {
                return Some(i);
            }
        }
        None
    }
}
#[derive(Clone, Copy)]
struct Key<'a>(&'a CapturedMethod);
impl PartialEq for Key<'_> {
    fn eq(&self, other: &Self) -> bool {
        observed_eq(self.0, other.0)
    }
}
impl Eq for Key<'_> {}
impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let CapturedMethod {
            sig,
            declaration,
            markers,
            body,
        } = self.0;
        sig.hash(state);
        declaration
            .as_ref()
            .map(|d| {
                let DeclarationEvidence {
                    return_type,
                    attributes,
                    modifiers,
                } = d;
                (return_type, attributes, modifiers)
            })
            .hash(state);
        markers.hash(state);
        body.hash(state);
    }
}
#[derive(Default)]
struct MeasuredHasher(std::collections::hash_map::DefaultHasher);
impl Hasher for MeasuredHasher {
    fn write(&mut self, bytes: &[u8]) {
        work::update(|c| c.overload.hash_input_bytes += bytes.len());
        self.0.write(bytes);
    }
    fn finish(&self) -> u64 {
        if DESIGN.with(|d| matches!(d.get(), Design::Collisions)) {
            0
        } else {
            self.0.finish()
        }
    }
}
type HashIndex<'a> = HashMap<Key<'a>, (Vec<usize>, usize), BuildHasherDefault<MeasuredHasher>>;
enum Exact<'a> {
    Partition(SignatureIndex),
    One,
    Sorted {
        indices: Vec<usize>,
        cursor: Vec<usize>,
    },
    Hash(HashIndex<'a>),
    Empty,
}
pub(crate) struct Lookup<'a, 'g> {
    current: &'g [&'a CapturedMethod],
    exact: Exact<'a>,
    signature: Option<SignatureIndex>,
}
impl<'a, 'g> Lookup<'a, 'g> {
    pub(crate) fn new(current: &'g [&'a CapturedMethod]) -> Self {
        let design = DESIGN.with(Cell::get);
        let exact = match design {
            Design::Partition => Exact::Partition(SignatureIndex::new(current, None)),
            Design::Refined => {
                if current.len() == 1 {
                    Exact::One
                } else {
                    Exact::Partition(SignatureIndex::exact_capacity(current))
                }
            }
            Design::Capacity => Exact::Partition(SignatureIndex::exact_capacity(current)),
            Design::Sorted => {
                let mut indices: Vec<_> = (0..current.len()).collect();
                indices
                    .sort_unstable_by(|&a, &b| payload_cmp(current[a], current[b]).then(a.cmp(&b)));
                let cursor = vec![0; indices.len()];
                work::update(|c| {
                    c.overload.entries[0] += indices.len();
                    c.overload.queue_capacity[0] += indices.capacity() + cursor.capacity();
                });
                Exact::Sorted { indices, cursor }
            }
            Design::Hash | Design::Collisions => {
                let mut map: HashIndex<'_> = HashMap::default();
                for (i, &m) in current.iter().enumerate() {
                    map.entry(Key(m)).or_default().0.push(i);
                    work::update(|c| {
                        c.overload.entries[0] += 1;
                        c.overload.projections += 1;
                    });
                }
                work::update(|c| {
                    c.overload.hash_table_capacity += map.capacity();
                    c.overload.retained_keys[0] += map.len();
                    c.overload.retained_queues[0] += map.len();
                    c.overload.queue_capacity[0] +=
                        map.values().map(|(v, _)| v.capacity()).sum::<usize>();
                });
                Exact::Hash(map)
            }
        };
        Self {
            current,
            exact,
            signature: None,
        }
    }
    pub(crate) fn exact(&mut self, base: &'a CapturedMethod, used: &[bool]) -> Option<usize> {
        work::update(|c| c.overload.lookups[0] += 1);
        match &mut self.exact {
            Exact::One => {
                work::update(|c| {
                    c.overload.selection_visits += 1;
                    c.overload.used_flag_visits += 1;
                });
                (!used[0] && observed_eq(self.current[0], base)).then_some(0)
            }
            Exact::Partition(index) => {
                let (a, b) = index.range(self.current, &base.sig);
                for &i in &index.indices[a..b] {
                    work::update(|c| c.overload.selection_visits += 1);
                    if !used[i] && observed_eq(self.current[i], base) {
                        return Some(i);
                    }
                }
                None
            }
            Exact::Sorted { indices, cursor } => {
                let a = indices
                    .partition_point(|&i| payload_cmp(self.current[i], base) == Ordering::Less);
                let b = indices
                    .partition_point(|&i| payload_cmp(self.current[i], base) != Ordering::Greater);
                if a == b || a + cursor[a] >= b {
                    None
                } else {
                    let i = indices[a + cursor[a]];
                    cursor[a] += 1;
                    work::update(|c| c.overload.selection_visits += 1);
                    Some(i)
                }
            }
            Exact::Hash(map) => {
                work::update(|c| c.overload.projections += 1);
                map.get_mut(&Key(base)).and_then(|(indices, cursor)| {
                    let i = indices.get(*cursor).copied();
                    *cursor += usize::from(i.is_some());
                    work::update(|c| c.overload.selection_visits += usize::from(i.is_some()));
                    i
                })
            }
            Exact::Empty => unreachable!(),
        }
    }
    pub(crate) fn finish_exact(&mut self, used: &[bool]) {
        // A structural cardinality case; global phase sequencing remains in caller.
        if matches!(self.exact, Exact::One) {
            return;
        }
        let old = std::mem::replace(&mut self.exact, Exact::Empty);
        self.signature = Some(match old {
            Exact::Partition(index) => index,
            other => {
                drop(other);
                SignatureIndex::new(self.current, Some(used))
            }
        });
    }
    pub(crate) fn signature(&mut self, sig: &str, used: &[bool]) -> Option<usize> {
        if matches!(self.exact, Exact::One) {
            work::update(|c| {
                c.overload.lookups[1] += 1;
                c.overload.selection_visits += 1;
                c.overload.used_flag_visits += 1;
            });
            return (!used[0] && observed_sig_eq(&self.current[0].sig, sig)).then_some(0);
        }
        self.signature
            .as_mut()
            .unwrap()
            .signature(self.current, used, sig)
    }
}
pub(crate) fn layout() -> serde_json::Value {
    serde_json::json!({"index":size_of::<usize>(),"signature_index_header":size_of::<SignatureIndex>(),
        "hash_key":size_of::<Key<'_>>(),"hash_value":size_of::<(Vec<usize>,usize)>(),
        "hash_header":size_of::<HashIndex<'_>>(),"lookup_header":size_of::<Lookup<'_, '_>>(),
        "scope":"type sizes only; reported capacities omit allocator/node/control-byte overhead"})
}
