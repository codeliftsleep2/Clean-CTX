//! Agent MaxHeadRoom: test-only named default-operation work, never authority.
use std::cell::{Cell, RefCell};

#[derive(Clone, Debug, Default, serde::Serialize)]
pub(crate) struct Work {
    pub binary_calls: usize,
    pub copied_strings: usize,
    pub copied_string_bytes: usize,
    pub copied_vec_capacity: usize,
    pub reverse_entries: usize,
    pub reverse_key_bytes_offered: usize,
    pub operand_lookups: usize,
    pub operand_key_bytes_offered: usize,
    pub reverse_capacity: usize,
    pub binary_output_bytes: usize,
    pub metadata_queries: usize,
    pub metadata_rows: usize,
    pub discarded_pretty_bytes: usize,
    pub string_table_builds: usize,
    pub tuple_visits: usize,
    pub tuple_strings: usize,
    pub tuple_bytes: usize,
    pub intern_calls: usize,
    pub intern_key_bytes_offered: usize,
    pub new_table_strings: usize,
    pub new_table_string_bytes: usize,
    pub owned_index_capacity_max: usize,
    pub ordered_strings_capacity_max: usize,
}
thread_local! { static WORK: RefCell<Work> = RefCell::new(Work::default()); }
thread_local! { static ENABLED: Cell<bool> = const { Cell::new(true) }; }
fn update(f: impl FnOnce(&mut Work)) {
    if ENABLED.with(Cell::get) {
        WORK.with(|work| f(&mut work.borrow_mut()));
    }
}
pub(crate) fn unobserved<T>(f: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            ENABLED.with(|v| v.set(self.0));
        }
    }
    let previous = ENABLED.with(|v| v.replace(false));
    let _restore = Restore(previous);
    f()
}
pub(crate) fn table_build() {
    update(|w| w.string_table_builds += 1);
}
pub(crate) fn tuple(values: &[String]) {
    update(|w| {
        w.tuple_visits += 1;
        w.tuple_strings += values.len();
        w.tuple_bytes += values.iter().map(String::len).sum::<usize>();
    });
}
pub(crate) fn intern(s: &str) {
    update(|w| {
        w.intern_calls += 1;
        w.intern_key_bytes_offered += s.len();
    });
}
pub(crate) fn table_insert(s: &str, strings_capacity: usize, index_capacity: usize) {
    update(|w| {
        w.new_table_strings += 1;
        w.new_table_string_bytes += s.len();
        w.owned_index_capacity_max = w.owned_index_capacity_max.max(index_capacity);
        w.ordered_strings_capacity_max = w.ordered_strings_capacity_max.max(strings_capacity);
    });
}
pub(crate) fn reset() {
    update(|work| *work = Work::default());
}
pub(crate) fn snapshot() -> Work {
    WORK.with(|work| work.borrow().clone())
}
pub(crate) fn binary_copy(strings: &[String], capacity: usize) {
    update(|work| {
        work.copied_strings += strings.len();
        work.copied_string_bytes += strings.iter().map(String::len).sum::<usize>();
        work.copied_vec_capacity += capacity;
    });
}
pub(crate) fn binary_entry(s: &str) {
    update(|work| {
        work.reverse_entries += 1;
        work.reverse_key_bytes_offered += s.len();
    });
}
pub(crate) fn binary_lookup(s: &str) {
    update(|work| {
        work.operand_lookups += 1;
        work.operand_key_bytes_offered += s.len();
    });
}
pub(crate) fn binary_output(bytes: usize, capacity: usize) {
    update(|work| {
        work.binary_calls += 1;
        work.binary_output_bytes += bytes;
        work.reverse_capacity += capacity;
    });
}
pub(crate) fn metadata_query() {
    update(|work| work.metadata_queries += 1);
}
pub(crate) fn metadata_text(text: Option<&str>) {
    update(|work| {
        work.metadata_rows += 1;
        work.discarded_pretty_bytes += text.map_or(0, str::len);
    });
}

#[cfg(feature = "typescript")]
#[path = "../mcp/default_operation_work_af_perf_002.rs"]
mod public_work;
