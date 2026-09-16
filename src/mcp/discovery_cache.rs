// src/mcp/discovery_cache.rs
//
// Session-scoped hydration discovery completion cache.
//
// `workspace_query` semantic hydration is a two-stage operation:
//   1. **discovery**  — ask CBM (or, for roots CBM cannot cover, run a literal
//                       filesystem scan) which candidate files could contribute
//                       semantics for one (entity name, discovery mode) target;
//   2. **compilation** — compile only the discovered files the WorkspaceIndex
//                       does not already hold.
//
// Stage 2 was already deduplicated by `WorkspaceIndex::file_map()`. Stage 1 was
// not: every eligible `workspace_query` call re-ran the CBM project search
// and/or the full filesystem scan even when the workspace had not changed since
// the previous identical query — rediscovering the same candidates and then
// discarding them as already compiled. Repeated identical queries therefore
// paid the full discovery cost every time.
//
// This cache records discovery *completion*. It answers exactly one question:
//
//   "Have we already searched for this semantic target, for this project or
//    root, under the current workspace generation?"
//
// It deliberately stores no candidates, no entities, no edges and no query
// answers. `WorkspaceIndex` stays the single authority for what Clean-CTX
// knows, and every query answer is still evaluated against the live index.
//
// # Validity model
//
// Each discovery **scope** (provider + root identity) owns a monotonically
// increasing generation. A completed entry records the generation it was
// completed at, and is valid only while the scope generation still matches.
// Invalidation is therefore O(1) — bump the generation and every older entry
// stops matching — with no entry scan, eviction, TTL, or size cap.

use std::collections::HashMap;

/// Which provider completed a discovery unit.
///
/// A CBM project search and a filesystem scan are different discovery
/// operations with different (provider-specific) candidate sets, so a result
/// completed by one provider never satisfies the other. Keeping the provider
/// in the scope identity prevents, for example, a CBM-discovered project from
/// suppressing the filesystem fallback scan that the same root would otherwise
/// receive when CBM is unavailable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DiscoveryProvider {
    /// One CBM project (`search_in_project` / `inbound_reference_paths_in_project`).
    Cbm,
    /// One filesystem root scanned by the literal fallback discovery.
    Filesystem,
}

/// The discovery operation a hydration target requires.
///
/// Declaration discovery (name/symbol search) and inbound-reference discovery
/// (`MATCH (caller)-[r]->(target)`) are semantically different operations that
/// select different candidate files, so they never share a cache entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum DiscoveryMode {
    /// Declaration/name discovery (`find_entities`, `forward_edges`,
    /// `transitive_dependencies`).
    Declaration,
    /// Inbound-reference discovery (`reverse_edges`).
    InboundReference,
}

/// Identity of one discovery unit: a provider plus the canonical identity of
/// the project root it covers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DiscoveryScope {
    provider: DiscoveryProvider,
    root: String,
}

impl DiscoveryScope {
    /// Scope for one CBM project, keyed by its canonical root identity.
    pub(crate) fn cbm(root: impl Into<String>) -> Self {
        Self {
            provider: DiscoveryProvider::Cbm,
            root: root.into(),
        }
    }

    /// Scope for one filesystem discovery root, keyed by its canonical root
    /// identity.
    pub(crate) fn filesystem(root: impl Into<String>) -> Self {
        Self {
            provider: DiscoveryProvider::Filesystem,
            root: root.into(),
        }
    }
}

/// One completed discovery: scope + discovery mode + target entity name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DiscoveryKey {
    scope: DiscoveryScope,
    mode: DiscoveryMode,
    name: String,
}

/// Session-scoped discovery completion cache (see module docs).
#[derive(Default)]
pub(crate) struct HydrationDiscoveryCache {
    /// scope → current generation. A scope appears here as soon as it is
    /// marked complete or explicitly invalidated.
    generations: HashMap<DiscoveryScope, u64>,
    /// Completed discovery → the generation it was completed at.
    completed: HashMap<DiscoveryKey, u64>,
}

impl HydrationDiscoveryCache {
    /// Create an empty cache (fresh session).
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Current generation of `scope` (0 when the scope has never completed a
    /// discovery and has never been invalidated).
    fn generation(&self, scope: &DiscoveryScope) -> u64 {
        self.generations.get(scope).copied().unwrap_or(0)
    }

    /// True when discovery for this scope/mode/name already completed in the
    /// scope's current generation. Callers must not repeat that discovery.
    pub(crate) fn is_complete(
        &self,
        scope: &DiscoveryScope,
        mode: DiscoveryMode,
        name: &str,
    ) -> bool {
        let key = DiscoveryKey {
            scope: scope.clone(),
            mode,
            name: name.to_string(),
        };
        self.completed
            .get(&key)
            .is_some_and(|completed_at| *completed_at == self.generation(scope))
    }

    /// Record a *successful* discovery (including a successful discovery that
    /// found zero candidates). Failed or partial discovery must never be
    /// recorded: it stays eligible for retry on the next request.
    pub(crate) fn mark_complete(
        &mut self,
        scope: &DiscoveryScope,
        mode: DiscoveryMode,
        name: &str,
    ) {
        // Track the scope so a later invalidation has a generation to bump.
        let generation = *self.generations.entry(scope.clone()).or_insert(0);
        self.completed.insert(
            DiscoveryKey {
                scope: scope.clone(),
                mode,
                name: name.to_string(),
            },
            generation,
        );
    }

    /// Invalidate every recorded discovery for one scope by advancing its
    /// generation. Unknown scopes are a no-op (nothing recorded, nothing to
    /// invalidate), which keeps caller-supplied root strings that match no
    /// configured root from growing the cache.
    fn invalidate_scope(&mut self, scope: &DiscoveryScope) {
        if let Some(generation) = self.generations.get_mut(scope) {
            *generation = generation.wrapping_add(1);
        }
    }

    /// Invalidate every provider scope derived from one canonical root
    /// identity (used when the root's source may have changed).
    pub(crate) fn invalidate_root(&mut self, root_identity: &str) {
        self.invalidate_scope(&DiscoveryScope::cbm(root_identity));
        self.invalidate_scope(&DiscoveryScope::filesystem(root_identity));
    }

    /// Invalidate every known scope (used when a change cannot be attributed
    /// to a single configured root).
    pub(crate) fn invalidate_all(&mut self) {
        for generation in self.generations.values_mut() {
            *generation = generation.wrapping_add(1);
        }
    }

    /// Test-only: number of scopes with a tracked generation.
    #[cfg(all(test, feature = "rust"))]
    pub(crate) fn tracked_scopes(&self) -> usize {
        self.generations.len()
    }

    /// Test-only: number of completed discovery entries currently tracked.
    #[cfg(all(test, feature = "rust"))]
    pub(crate) fn completed_entries(&self) -> usize {
        self.completed.len()
    }
}

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/mcp/discovery_cache.rs"]
mod tests;
