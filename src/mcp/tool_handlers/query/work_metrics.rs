//! Test-only structural work counters for semantic query preparation.
//!
//! These counters observe reconstructable work. They never participate in
//! semantic authority, caching, completion, or production behavior.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct QueryWorkCounts {
    pub(crate) csharp_catalog_builds: usize,
    pub(crate) csharp_catalog_reuses: usize,
    pub(crate) csharp_files_considered: usize,
    pub(crate) csharp_files_parsed: usize,
    pub(crate) derived_catalog_invalidations: usize,
    pub(crate) angular_catalog_builds: usize,
    pub(crate) angular_components_considered: usize,
    pub(crate) angular_template_sources_processed: usize,
    pub(crate) angular_inline_templates_processed: usize,
    pub(crate) angular_external_templates_processed: usize,
}

thread_local! {
    static COUNTS: std::cell::Cell<QueryWorkCounts> =
        const { std::cell::Cell::new(QueryWorkCounts {
            csharp_catalog_builds: 0,
            csharp_catalog_reuses: 0,
            csharp_files_considered: 0,
            csharp_files_parsed: 0,
            derived_catalog_invalidations: 0,
            angular_catalog_builds: 0,
            angular_components_considered: 0,
            angular_template_sources_processed: 0,
            angular_inline_templates_processed: 0,
            angular_external_templates_processed: 0,
        }) };
}

pub(crate) fn reset() {
    COUNTS.with(|counts| counts.set(QueryWorkCounts::default()));
}

pub(crate) fn snapshot() -> QueryWorkCounts {
    COUNTS.with(std::cell::Cell::get)
}

fn update(update: impl FnOnce(&mut QueryWorkCounts)) {
    COUNTS.with(|cell| {
        let mut counts = cell.get();
        update(&mut counts);
        cell.set(counts);
    });
}

pub(crate) fn record_csharp_catalog_build() {
    update(|counts| counts.csharp_catalog_builds += 1);
}

pub(crate) fn record_csharp_catalog_reuse() {
    update(|counts| counts.csharp_catalog_reuses += 1);
}

pub(crate) fn record_csharp_file_considered() {
    update(|counts| counts.csharp_files_considered += 1);
}

pub(crate) fn record_csharp_file_parsed() {
    update(|counts| counts.csharp_files_parsed += 1);
}

pub(crate) fn record_derived_catalog_invalidation() {
    update(|counts| counts.derived_catalog_invalidations += 1);
}

pub(crate) fn record_angular_catalog_build() {
    update(|counts| counts.angular_catalog_builds += 1);
}

pub(crate) fn record_angular_component_considered() {
    update(|counts| counts.angular_components_considered += 1);
}

pub(crate) fn record_angular_template_processed(inline: bool) {
    update(|counts| {
        counts.angular_template_sources_processed += 1;
        if inline {
            counts.angular_inline_templates_processed += 1;
        } else {
            counts.angular_external_templates_processed += 1;
        }
    });
}
