# B0 scope and fixture freeze

Phase A A0–A5 is frozen following owner acceptance with declared live limitations.
Phase B continuation was authorized; this slice is B0 from the implementation
plan, not a reopening of Phase A or authorization for Issue #142.

## Current defect and production boundary

GraphBridge cached_value_for reads `(registered root, project:key)` from
GraphCacheStore and accepts any valid JSON payload. Legacy symbol-importance,
search, query and trace payloads therefore share the same adapter-contract
namespace. A project/root ownership boundary already exists and must remain.

B0 will add the planned narrow adapter-wire namespace to GraphBridge memory
and disk keys. The namespace identifies source **contract** pin and wire
revision; it does not prove that cached evidence came from that exact artifact.
Legacy entries must miss without being deleted or hydrated as current evidence.
No SQL migration, provider-store deletion, TTL change, native persistence change,
Binary0x04/producer generation change, public schema or freshness claim.

Existing cache ownership fixtures must use the current disk-key factory when
seeding current payloads, while the new regression deliberately seeds the old
literal key. Existing project invalidation semantics are preserved.

## Fixture authority

- Existing tracked deterministic query-wire fixtures protect old typed projection
  and alias behavior during staged migration; they are not proof of current CBM
  body decoding.
- Existing cache_ownership fixtures protect memory/disk partition isolation.
- Pinned live A5 evidence at `target/cbm-a5/20261011-210512/` captured 17-tool
  registries and distinct grouped structural search results for both TypeScript
  fixture roots. Current required registry facts are recorded in the A5 report.
- Exact current tool-body decoding, soft-error accounting and private result
  limits belong to B1/B2. Do not relabel legacy decoder fixtures as current.
- QueryRows currently contains columns and rows only. B0 does not fabricate
  completeness or admitted-identity metadata; subsequent private metadata work
  must preserve actual missing/unknown observations.

## RED-first verification status

Tracked regression: `src/tests/cbm/wire_epoch.rs`, registered in cbm/tests.rs.
It seeds a valid pre-epoch disk payload, requires a miss/error with no live
provider, and verifies that the old stored payload was not deleted.
Observed RED: one selected test failed at the legacy-payload rejection assertion.
The regression/registration were stashed alone; the production fix was applied
while absent, then the same regression was restored. GREEN: one selected test
passed, including verification that the old disk entry remains readable under
its literal legacy key. Formatting changed layout only, not assertions.

Implementation: GraphCacheKey contains a private wire-contract field for memory
hash/equality; disk keys use the same contract namespace via its key factory.
Namespace: `cbm-wire:82b1ec908f975354ceb4bac8a8bfc594942a9bc4:v1`.
Disk lookup and write-through both use that factory. Project ownership,
registered-root partitions, TTL and invalidation behavior remain unchanged.
Current cache ownership fixtures now seed through the production key factory;
their behavioral assertions are unchanged. No GraphCacheStore SQL migration.

This is the initial migration epoch, not proof that the remaining legacy wire
decoders already implement the current provider contract. B1/B2 must revisit
compatibility when changing persisted payload shapes; no missing completeness
metadata is upgraded to known truth by the epoch.

Requested focused checks: cbm_b0_ RED and GREEN, cache_ownership,
query_wire::deterministic, integration, then package all-feature Clippy.
Authorized results: cbm_b0_ **1 GREEN**, cache_ownership **3 passed**,
query_wire::deterministic **16 passed**, integration **8 passed**, package
all-feature/all-target Clippy **passed with -D warnings**. Commands used the
locked clean-ctx library test target, all features and `-j 4`; exact filters
match those requested. No live provider was launched by these tests.
Full repository suite remains CI-only; no live provider is needed for B0.

B3's aggregate collection deadline/budget remains an explicit architectural
approval point. Generic Phase B continuation does not choose that policy.
