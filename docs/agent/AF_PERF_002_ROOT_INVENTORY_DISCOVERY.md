# AF-PERF-002: per-root inventory filtering and cloning discovery

Agent: Agent MaxHeadRoom
Status: executed characterization GREEN; no-proof-target manifest design recommended.
Startup projection DEFERRED; delta-count removal REJECTED. Prior corrections stay
CLOSED / FIX / PROTECTED; dense same-signature equality deferred.

## Acquisition and real production path

Known-path reads trace registered workspace_query find_entities -> prepared query/
hydrate_workspace_index_for_acquisition -> provider discovery or filesystem fallback
-> pending_discovery_roots -> filesystem::scan -> enumerate_source_paths ->
build_inventory -> type_ref_candidate_owners -> root_inventories -> DiscoveryOutcome
-> prepare_clean_acquisition -> manifests_by_source_kind -> proof claims/currentness
snapshots -> semantic publication -> acquisition completion cache. The actual tests
force the normal CBM-disabled filesystem fallback, not a custom compiler pipeline.
No server/provider process is launched. Existing degraded frontend_retired admission
from this campaign is acquisition failure, not evidence of missing callers.

configured_roots includes primary plus additional roots; deduplicate_roots canonical-
keys, deduplicates and sorts them. Overlapping ancestors remain distinct obligations.
scan enumerates configured roots, constructs one combined owned manifest, selects
name candidates, then filters the whole combined entry list for EACH pending root.
The filter uses Path::starts_with on owner and canonical root, not a string prefix.
Selected entries are cloned, then InventoryManifest::new sorts, deduplicates owners
and derives a per-root digest while retaining attempted/complete semantics.

Source-kind grouping clones each selected per-root entry again, builds per-kind
vectors, filters producer incapability and constructs new per-unit manifests. These
are proof inputs, not presentation copies. Completion claims retain inventory digest,
root/source-kind/generation/admission identities. Provisional acquisition extracts
owned candidate owners and owner/hash snapshots; publication must still succeed
before completion authority. There is no assumption a filtered representation is
interchangeable across different roots or proof units.

## Ownership finding and observation scope

InventoryEntry owns owner:String and source_hash:String, but source is Arc<String>.
The existing derived Clone copies those TWO metadata strings and increments an Arc
reference. It does NOT clone the source String/body. Source size axes must not be
reported as copied payload work. Long source text still matters for initial read,
hash, candidate search and compilation, outside these named clone counts.

This packet adds cfg(test,rust) hooks at the real root filter/clone and source-kind
clone/group-vector boundaries. No selection, algorithm, lifetime, output or authority
policy is changed. Root manifest construction is named locally to observe its
actual completed vector capacity/digest. Entry fields, source storage and consumers
remain unchanged. Test hooks never feed decisions.

Counters cover full root-list filter rounds (roots per call), input population,
actual predicates/rejections, selected entry clone calls, metadata bytes offered to
String cloning, source lengths shared through Arc (NOT copied), constructed root
manifests and final entry/capacity populations. Grouping reports actual entry clones,
metadata bytes, shared body lengths, admitted group entries/capacity. Group clones
may include subsequently rejected kinds; successful group observations are separate.
Digest/sort CPU comparisons, source hash CPU bytes, Arc atomic costs, allocator events,
peak memory and deallocation events are NOT measured. Root receipt generation and
logging add diagnostic owned data; these are excluded from named production-copy
counters and remain overhead in instrumented timing.

Root manifests preserve owned snapshots beyond combined-manifest lifetime; grouping
retains per-unit ownership during provisional proof construction. Combined manifests
are local and consumers do not require mutable source bodies. This does not prove
borrowing is a local substitution: current struct signatures/lifetimes and multi-
root overlap require owned membership vectors and retained proof identity. A future
borrowing proposal would require separate lifetime/concurrency/design review.

## Characterization and bounded semantic controls

Tracked helper equivalence compares the actual instrumented root constructor to a
frozen uninstrumented copy of its original expression. Complete root order/manifests
(entries/all fields, digest, attempted/complete) must agree. It includes empty roots,
empty entries, duplicate roots, overlapping roots, component-prefix boundaries,
relative/empty malformed owners and duplicate owner entries with different hashes.
These raw malformed inputs are helper controls, not falsely admitted production
sources. Arc::ptr_eq verifies source sharing with original entries.

Tracked actual registered workspace_query runs12 populations varying1/4/8 configured
roots,0/1/32 total-files axes, disjoint versus nested roots,0/4096 Unicode payload
repetitions, missing roots and an invalid-UTF8 source case. Each issues cold, identical
warm and different-name operations:36 complete response/work rows. Root population
reported is fixture count; actual effective roots/filter invocations are in work,
including deduplication and cache effects. Cold must actually reach root construction;
zero reachability is fixture failure, not zero-work success. Warm reuse is observed,
not assumed. No forced cache invalidation or production policy change.

Complete registered responses, ordered per-root owner/hash/kind/digest/flag receipts,
source byte identity and optional live alias/hash currentness are retained. Hydration
may legitimately publish semantic/cache state; tests assert any live source hash
matches the independent fixture source rather than forbid required admission.
Persistence is disabled, so no durable identity is invented. Missing/malformed cases
retain actual partial/failure responses; no success/completeness shortcut is asserted.
Existing incomplete-authorized-root completeness regression runs independently.

Actual synchronous dispatch timing includes acquisition/filter/manifests/proof/output
construction and internal teardown up to response return. Final response destruction
is separately timed after an evidence copy is made outside both clocks. No paired
replacement is implemented; observation/receipt overhead limits causal attribution.
No claim of optimization benefit, machine CPU work, allocator saving or public
latency improvement. Original-helper equivalence is untimed, avoiding an asymmetric
unobserved-vs-observed benchmark. Symmetric candidate timing is only warranted if
this discovery identifies a plausible separately authorized design.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-root-inventory-discovery.ps1
```

Expected three tracked passes: helper original-equivalence, actual public12x3
characterization and existing incomplete-root protection. Require36 complete work/
response rows, warning-fatal all-target/all-feature Clippy, format/size/strict encoding
and exact source receipts. Protected startup/SQLite text/encoder/matcher/edit files
must remain unchanged. Only packet/hook files are formatted. No optimization RED,
real mutation or composed gate requested. All runtime UNRUN; agent performed bounded
source/parser/UTF-8/diff checks only. No stage/commit/push.

Return after execution: rank root filtering, metadata/Arc cloning and source-kind
regrouping by observed work and actual exposure/reuse; distinguish required root/
proof snapshots from proven avoidable work. At most one bounded next design slice,
or DEFER if no worthwhile safe alternative is established. Do not relabel shared
source bytes as copies or assume cached operations rescan. Startup and all protected
repairs stay closed; inventory sourcebody/hash/lifecycle authority unchanged. Census
remains13/9763 scoped,9750 unresolved plus96 supplements.
## Executed discovery and conditional consumer correction

Agent MaxHeadRoom inspected root-inventory-discovery-20261108-221307: three tracked
passes, zero failed/ignored. Original/helper and public tests7.70s; existing incomplete-
root proof regression0.01s. All36 rows complete, warning-fatal all-target/all-feature
Clippy42.54s, both format logs empty, size26 active files and strict encoding2268
text files GREEN. All1392 recorded source hashes match at inspection before this
report update, including all protected production identities. inspection.json retains
complete rows. No optimization, mutation or composed gate executed.

The initial lifecycle trace above needs an explicit CONDITIONAL consumer boundary:
ordinary find_entities follows CleanAuthoritative, whose proof_target() is None.
In execution.rs, outcome.inventories is consulted only under a present proof target:
extra proof scan when required, then prepare_clean_acquisition in and_then(target).
Ordinary query discovery builds per-root inventories, returns them in the outcome,
and ultimately drops them without those consumers. All executed grouping clone
counters are zero. This is confirmed by source trace, not inferred solely from zero
counters. Proof-backed acquisition DOES consume root inventories and is not shown
to have redundant ownership. Its positive grouping cost remains unmeasured here.
The independent incomplete-root regression protects refusal but is not a substitute
for a measured proof-backed public population.

## Actual named work and reuse

Representative COLD real registered request counts:

| Roots / files per root / overlap | Combined entries | Predicates / rejected | Entry clones | Metadata bytes | Output entries / capacity |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1 / 1 / no | 1 | 1 / 0 | 1 | 120 | 1 / retained in receipt |
| 1 / 32 / no | 32 | 32 / 0 | 32 | 3862 | 32 / 32 |
| 4 / 8 / no | 32 | 128 / 96 | 32 | 3840 | 32 / 32 |
| 8 / 4 / no | 32 | 256 / 224 | 32 | 3840 | 32 / retained in receipt |
| 4 / 8 / yes | 32 | 128 / 48 | 80 | 11040 | 80 / retained in receipt |
| 8 / 4 / yes | 32 | 256 / 112 | 144 | 22560 | 144 / 172 |
| 4 / 0 / no | 0 | 0 / 0 | 0 | 0 | 0 / 0 |
| 4 plus missing / 8 / no | 32 | 160 / 128 | 32 | 3840 | 32 / retained in receipt |
| 4 plus missing / 8 / yes | 32 | 160 / 80 | 80 | 11040 | 80 / retained in receipt |

Capacities are summed across root vectors, not peak or allocated bytes. Counts name
the actual filtering/derived-clone sites, not extra diagnostic receipt copies. Each
clone copies owner/hash strings and shares source Arc. Long-body disjoint populations
retain the SAME metadata clone counts/bytes as their unpadded counterpart.32 entries
with4096 Unicode repetitions share787254 source bytes at one root, or787232 across
four disjoint roots; these body bytes are NOT copied by entry cloning. The lengths
differ slightly due fixture identifiers, not a normalization or allocation effect.

For complete roots, identical warm requests have ZERO root construction calls,
predicates/clones/manifests. Different-name operations repeat the cold named work.
With a missing root (including the malformed-source overlapping case), identical
warm requests repeat all5-root/160-predicate work because discovery remains incomplete.
That retry is necessary for currentness/failure recovery; it is not evidence for
caching incomplete results. No new cache or invalidation exception is proposed.

Actual whole-operation times are descriptive single observations: complete one-root/
32-file cold17.53ms, disjoint8-root/32-file41.69ms, overlapping8-root/32-file59.76ms;
the matching identical warm operations1.89/8.44/8.81ms. Different-name overlap8-root
operation61.44ms. Missing-root retries remain roughly25..34ms in these fixtures.
These include filesystem acquisition, hashes, diagnostics and response/proof work;
they do NOT attribute differences to256 predicates or22KB copying. Final response
drop times are retained separately; evidence-copy construction is outside clocks.
No paired removal or public/release latency benefit has been measured. Do not use
the root/time relationship as causal CPU or allocator proof.

## Ranked avoidable versus necessary work

1. **No-proof-target root manifest construction: one bounded DESIGN recommendation.**
   On actual ordinary queries, complete per-root filtering, metadata/Arc cloning,
   vector construction, manifest sort/digest work and teardown have no consumer.
   Avoidability is established for this conditional branch by actual counts plus
   exhaustive outcome.inventories use tracing in execution.rs. The combined source
   inventory, name candidate search, traversal/read/hash completion and candidates
   remain necessary. A design could thread explicit existing proof demand to the
   private scan boundary and omit ONLY per-root manifests when proof_target is None.
   No new matching/cache/global policy is needed. This is a hypothesis for a small
   local demand boundary, not an implemented or measured replacement. Consequential
   benefit remains unestablished; metadata volumes here are bounded/modest.
2. **Proof-backed per-root/group ownership: retain; positive cost unmeasured.**
   Different roots/kinds require distinct completion/digest/owner obligations.
   Overlapping membership is not automatically redundant. Borrowing across current
   owned proof structs would add lifetime/interface complexity; Arc already avoids
   body copies. Do not recommend a general borrowed manifest redesign from this run.
3. **Repeated acquisition after incomplete roots: necessary retry, not a correction.**
   Failure recovery/currentness prevents treating the partial scan as completion.
   Same-name successful warm reuse already eliminates the named work. Keep per-name
   discovery and authority distinctions; no broader persistent reuse proposal.

## Next owner decision and precise design boundary

Recommend at most ONE next bounded design comparison: demand-driven construction
of per-root manifests for existing proof consumers. Compare original versus test-only
omission on ordinary no-proof requests while retaining complete source inventory/
candidates/completion flags, root ordering, public response, publication/cache/source
hash identities and errors. Include proof-backed positive controls demonstrating
unchanged complete manifests and incomplete-root refusal; they are currently a
coverage limit. Measure symmetric full scan/setup/query/result/drop plus named
work and clone/digest construction. Avoid long body-copy claims or inferring public
latency from diagnostics. If saved complete-operation cost is negligible, DEFER
rather than authorize another correction.

No production implementation is recommended for authorization yet. Current return
is a specific source-supported avoidable-work candidate with a bounded semantic
alternative to TEST, not FIX PROPOSED or optimization RED. Any later implementation
requires separate owner approval, genuine original budget RED, frozen ordinary and
proof-backed/public/currentness oracles, integrated cost review, real restoration
mutation and composed acceptance. No new scope cache, producer acquisition policy,
source ownership or proof contract change.

All completed repairs remain CLOSED / FIX / PROTECTED. Startup projection remains
DEFERRED; count removal REJECTED; dense equality deferred. No stage/commit/push,
no further runtime merely to repeat this characterization. Census unchanged.