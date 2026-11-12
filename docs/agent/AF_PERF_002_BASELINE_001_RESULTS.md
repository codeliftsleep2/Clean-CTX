# AF-PERF-002: first measured baseline return

Agent: Agent MaxHeadRoom
Status: first Phase 1 packet verified GREEN; investigation incomplete, no repair applied.
Generation 78: implemented, targeted verification GREEN, pushed (owner-reported);
optional final field/repository certification outstanding. AF-PERF-002 is independent.

## Evidence

Run baseline-20261103-170508 passes two new characterization tests (0.40s) and
one existing production lifecycle/query test with new observations (13.02s).
Zero failed/ignored. Format log is empty, active sizes pass, recorded source hashes
match current code. This is instrumentation/fixture acceptance, not an intended
optimization RED or future-budget GREEN. No agent test/build/runtime was started.

Logs/source receipt: target/af-perf-002/baseline-verified-evidence/receipt.json.
Parsed observations: inspection.json in that directory. Timing includes test-only
identity serialization/set maintenance; do not compare instrumented elapsed time
with prior uninstrumented receipts as a speedup/regression claim.

## Actual witness and lifecycle observations

| Phase | Calls | Distinct typed expectations | Content reads / hashes | Canonicalizations | Metadata | Directory enumerations | Witness calls inside snapshot validation lock |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Publication | 26115 | 107 | 486 / 486 | 20043 | 5568 | 18 | 7175 |
| SQLite restore | 22960 | 107 | 432 / 432 | 17664 | 4848 | 16 | 0 |
| Snapshot currentness | 7175 | 107 | 135 / 135 | 5520 | 1515 | 5 | 7175 |
| Cold indexed query | 1435 | 107 | 27 / 27 | 1104 | 303 | 1 | 0 |
| Warm indexed query | 1435 | 107 | 27 / 27 | 1104 | 303 | 1 | 0 |

Distinct identity includes witness variant and every expected field, not just path.
It is phase-wide, not a proof that all 107 expectations share an observation
boundary. The same expectation at entry and commit can require separate checks.
Canonicalizations dominate this witness call mix; syscall latency attribution is
not measured. Directory count measures calls to the directory helper, not every
internal read_dir/entry metadata syscall. Content reads count attempts and hashes
count successful byte-read hash executions. No file reads outside Witness::current
are included in those columns.

Publication/restore visit 198/180 shared route-root facets; snapshot visits 60.
Queries visit zero route-root facets and invoke no retained admission/projection,
but still perform witness IO. Existing parse/discovery/catalog observations stay
separate. Synchronous thread-local counts do not cover un-attributed worker work.

## Index scaling: observed versus inferred

| Files/entities n | Shared-name occurrence comparisons, density 1 | Shared-name occurrence comparisons, density 4 | Single-file unique-key file-map comparisons |
| --- | ---: | ---: | ---: |
| 1 | 0 | 3 | 0 |
| 2 | 1 | 10 | 1 |
| 4 | 6 | 36 | 6 |
| 8 | 28 | 136 | 28 |
| 16 | 120 | 528 | 120 |

Duplicate edge suppression affects actual registration participation. All rows are observations
of this fixture, not a general formula for every edge shape. Density-1 occurrence
and unique-key file-map counts exactly follow n*(n-1)/2 over tested growth: concrete
quadratic membership work. Increasing real distinct occurrences still requires
per-occurrence storage; it does not justify rescanning every prior occurrence.

Root retirement invokes reconciliation n times for n owners, even with zero NgRx
candidates. Insertion also invokes it per file. This confirms invocation amplification
but does not measure expensive populated-candidate reconstruction or establish
that all n mutations can defer visibility safely. No reconciliation policy is chosen.

## Ranked independent repair opportunities

1. Index membership (#174): measured quadratic work, independent of filesystem
   currentness. Inspect all entity/name/file-map mutation paths, then propose
   auxiliary membership ownership while preserving Vec iteration order, multiple
   publishers and occurrence ambiguity. Target expected O(E+U+O) membership work
   with hash-based membership, or explicit logarithmic bounds with ordered sets.
   Exact representation/lifecycle requires owner disposition before production.
2. Witness generation versus replay: canonicalization counts suggest capture may
   retain repeated typed expectations as well as proof replays multiplying sweeps.
   Next distinguish witness-vector length/multiplicity and per-proof/per-owner/commit
   observations before proposing reuse. Potential reduction is from repeated full
   sweeps to exact distinct work per approved boundary, not automatically 7175->107.
3. Root reconciliation (#173): n complete invocations for root removal. Next test
   populated candidates plus survivors and trace publication visibility. Potential
   n->1 batch rebuild applies only to an explicitly valid batch, not arbitrary calls.
4. Snapshot lock: all 7175 checks occur under mutex in this fixture. Moving IO out
   requires pinned revision/retirement/recheck protocol; no contention benefit or
   safe lock-free exposure is inferred. Narrow proof replay improvements may reduce
   lock-held work without changing lock policy, if separately proven equivalent.
5. Structural lookup: count exact mapping/facet/entry scans and conflicting-record
   joins next; 60 facet visits alone do not establish all inner work or redundancy.

## Next Phase 1 packet and coverage-drift design return

Before requesting any optimization approval, extend the baseline with populated
NgRx candidates/export parse/read counts, unique same-name identity buckets,
cross-publisher remove/readd correctness, witness-vector multiplicity and typed
conflicts per named validation scope. Trace every proposed membership index mutation
and persistence/reset owner. Keep actual syscall attempts distinct from logical
observations and parses. No full historical acceptance campaign is needed.

Coverage-drift enforcement remains a design gap: new expensive roots/call-chain
changes must become REVIEW REQUIRED and block architectural acceptance until an
explicit classified function/work-unit/test record exists. Existing AST output
alone does not enforce this. Propose a finite test metadata inventory and AST
comparison of classified function/call boundaries, not whole-file hash approval,
blanket historical allowlists or a runtime proof registry. Each repair's new class
requires deterministic budget tests and real isolated mutation RED/GREEN.

The five requested production slices (witness, membership, reconciliation, locking,
structural lookup) will return with concrete proof-equivalence/currentness/ownership
and concurrency designs after this missing characterization. No production
optimization is authorized or implemented by this measured return. Accepted gen78
semantics remain baseline; instrumentation edits require their own focused lint/
preservation before hardening acceptance, not reopening feature contracts.

```architecture-graph
node: DOC-144
kind: RESEARCH
status: INVESTIGATING
title: First typed witness and index characterization packet
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_BASELINE_001_RESULTS.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
