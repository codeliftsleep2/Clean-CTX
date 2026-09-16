// src/tests/mcp/discovery_cache.rs
//
// Unit regressions for the session-scoped hydration discovery cache.
//
// These pin the cache's own validity model (generation-based, scope-/mode-/
// name-specific) independently of the hydration handlers that use it.

use super::*;

fn scope(root: &str) -> DiscoveryScope {
    DiscoveryScope::cbm(root)
}

#[test]
fn mark_then_lookup_hits_within_the_same_generation() {
    let mut cache = HydrationDiscoveryCache::new();
    assert!(!cache.is_complete(&scope("root"), DiscoveryMode::Declaration, "FooService"));
    cache.mark_complete(&scope("root"), DiscoveryMode::Declaration, "FooService");
    assert!(cache.is_complete(&scope("root"), DiscoveryMode::Declaration, "FooService"));
    assert_eq!(cache.completed_entries(), 1);
}

#[test]
fn lookup_is_provider_specific() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root"), DiscoveryMode::Declaration, "FooService");
    assert!(scope("root") == DiscoveryScope::cbm("root"));
    assert!(!cache.is_complete(
        &DiscoveryScope::filesystem("root"),
        DiscoveryMode::Declaration,
        "FooService"
    ));
}

#[test]
fn lookup_is_root_specific() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService");
    assert!(cache.is_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService"));
    assert!(!cache.is_complete(&scope("root-b"), DiscoveryMode::Declaration, "FooService"));
}

#[test]
fn lookup_is_discovery_mode_specific() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root"), DiscoveryMode::Declaration, "FooService");
    assert!(
        !cache.is_complete(
            &scope("root"),
            DiscoveryMode::InboundReference,
            "FooService"
        ),
        "declaration discovery must not satisfy inbound-reference discovery"
    );
}

#[test]
fn lookup_is_name_specific() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root"), DiscoveryMode::Declaration, "FooService");
    assert!(!cache.is_complete(&scope("root"), DiscoveryMode::Declaration, "BarService"));
}

#[test]
fn invalidate_root_drops_both_providers_for_that_root_only() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService");
    cache.mark_complete(
        &DiscoveryScope::filesystem("root-a"),
        DiscoveryMode::Declaration,
        "FooService",
    );
    cache.mark_complete(&scope("root-b"), DiscoveryMode::Declaration, "FooService");

    cache.invalidate_root("root-a");

    assert!(!cache.is_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService"));
    assert!(!cache.is_complete(
        &DiscoveryScope::filesystem("root-a"),
        DiscoveryMode::Declaration,
        "FooService"
    ));
    assert!(
        cache.is_complete(&scope("root-b"), DiscoveryMode::Declaration, "FooService"),
        "invalidation is scoped to one root"
    );
}

#[test]
fn invalidate_all_drops_every_scope() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService");
    cache.mark_complete(
        &scope("root-b"),
        DiscoveryMode::InboundReference,
        "BarService",
    );

    cache.invalidate_all();

    assert!(!cache.is_complete(&scope("root-a"), DiscoveryMode::Declaration, "FooService"));
    assert!(!cache.is_complete(
        &scope("root-b"),
        DiscoveryMode::InboundReference,
        "BarService"
    ));
}

#[test]
fn invalidating_an_unknown_root_records_nothing() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.invalidate_root("never-seen");
    cache.invalidate_all();
    assert_eq!(
        cache.tracked_scopes(),
        0,
        "invalidation must not grow the cache for unknown scopes"
    );
}

#[test]
fn invalidation_after_marking_only_is_no_op() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.invalidate_root("never-seen");
    cache.mark_complete(
        &scope("never-seen"),
        DiscoveryMode::Declaration,
        "FooService",
    );
    assert!(
        cache.is_complete(
            &scope("never-seen"),
            DiscoveryMode::Declaration,
            "FooService"
        ),
        "marking after an unknown-scope invalidation must still record completion"
    );
}

#[test]
fn repeated_invalidation_keeps_entries_stale() {
    let mut cache = HydrationDiscoveryCache::new();
    cache.mark_complete(&scope("root"), DiscoveryMode::Declaration, "FooService");
    cache.invalidate_root("root");
    cache.invalidate_root("root");
    cache.mark_complete(
        &scope("root"),
        DiscoveryMode::InboundReference,
        "FooService",
    );
    assert!(cache.is_complete(
        &scope("root"),
        DiscoveryMode::InboundReference,
        "FooService"
    ));
    assert!(!cache.is_complete(&scope("root"), DiscoveryMode::Declaration, "FooService"));
}
