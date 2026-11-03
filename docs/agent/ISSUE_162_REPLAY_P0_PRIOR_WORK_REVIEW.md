# Issue #162 P0: prior-work review and admission repair

Agent: Agent MaxHeadRoom
Status: bounded P0 focused GREEN (5 passes); cross-relation performance proposal pending.
Performance blocks acceptance. Generation 78; key/broader work suspended.

## Focused P0 GREEN; cross-relation performance remains blocking

Inspected resolver-replay-p0-after-20261103-124029: five passed, zero failed,
zero ignored, 3.24s. Every source hash in the operator receipt matches the current
working tree. No repeat run was started after the owner disposition; the existing
completed After execution supplies the requested focused verification.

| Fixed-fixture measure | Before | After |
| --- | --- | --- |
| Resolver projections, four candidate edges | 4 | 1 |
| Mapping replays | 40 | 5 |
| Admission wall time | 872742us | 113975us |
| Fixture construction wall time | 288606us | 186896us |

Fixture is unchanged: 236 source bytes, four files, three route objects, five
mapping occurrences, four canonical resolver edges, 325 witness observations.
The valid measurement function and work-count assertions were preserved. Tests
also observe one projection/five replays for 1,2,4,16 requested edges and zero
for empty/unrelated/legacy-only admission. Independent runtime/demanded reference,
forged/crossed-hash/missing authority and corrected consumed-dependency stale
refusal pass. Baseline's invalid seeded-owner control remains separately classified.

Admission is 7.66x faster in this single fixed-fixture observation (86.94% lower
elapsed); it is not a distributional benchmark, enterprise measurement or proof
of the contribution to the earlier 49-minute delay. Total test-run elapsed is not
used as an unchanged-suite timing comparison because the invalid control changed.
Evidence and exact source/test/log hashes:
 target/issue-162/replay-p0-focused-green-evidence/receipt.json.

This proves the bounded P0 work invariant, not full generation-78 correctness or
performance acceptance. Existing producer duplicate/partial/exact-identity policy
is unchanged by P0, but the expensive generation-78 preservation filters have not
been rerun post-fix and are not claimed GREEN. Clippy/full repository/binary gates
remain pending. No cache, currentness/lifecycle contract or lock was changed.

The [cross-relation proposal](ISSUE_162_CROSS_RELATION_REPLAY_PROPOSAL.md) records
remaining guard/Component/LoadsChildren/provider risks and separates call-local
projection batching from broader witness-currentness/lock design. Return for
bounded disposition before implementing sibling batching. No full campaign is
requested now. Generation stays 78; public-key/broader semantic expansion suspended.

## Primary sources and current repository truth

Read all seven GitHub issue bodies/comments/state using authenticated read-only gh.
Browser fetches failed; sandbox networking failed; authorized elevated gh reads
succeeded. Issue JSON/hashes: target/issue-162/replay-p0-prior-work/receipt.json.
CBM is degraded (frontend-retired timeout); known local files/git history supplied
implementation evidence. No server/test/build/profile/acceptance process was started.
Closed issue status is not proof that every suggested optimization exists today.

| Precedent | Inspected implementation/history | Reusable invariant and limit |
| --- | --- | --- |
| [#173 global reconciliation](https://github.com/codeliftsleep2/Clean-CTX/issues/173), OPEN | index/edges.rs still calls rebuild_ngrx_import_refinements after add_edges; index/remove.rs calls it after remove_file; ngrx_refinement.rs retracts/rebuilds accumulated candidates with source/export checks | Batch expensive complete-set work at the operation boundary. This separate NgRx repair is NOT already present in these paths and P0 does not implement it. |
| [#174 index complexity](https://github.com/codeliftsleep2/Clean-CTX/issues/174), OPEN | register_entity still uses name_entries.contains and occurrence iter().any, pushes file_entity_keys before occurrence dedup; edges.rs dedups the growing Vec using entry.contains | Work should scale with unique affected facts. These index concerns remain independently of resolver replay. |
| [#55 file removal](https://github.com/codeliftsleep2/Clean-CTX/issues/55), CLOSED | commit 364273de; remove.rs Phase 3 uses affected_names and targeted name buckets; index_performance.rs counts touched buckets/unrelated-growth invariance | File-local cleanup survives; this does not imply #174 insertion or later #173 refinement is fixed. |
| [#70 caller verification](https://github.com/codeliftsleep2/Clean-CTX/issues/70), CLOSED | cda734b2; caller verification retains candidate identity and per-batch ParseMemo; RED-CV10 in caller_verify.rs/caller_verify_proxy.rs pins parse reuse | Preserve already-derived exact identity; share work within a batch. A caller parse memo is not retained TypeScript admission authority. |
| [#71 discovery](https://github.com/codeliftsleep2/Clean-CTX/issues/71), CLOSED | f792a3c7; discovery_cache.rs, hydration/cache.rs/invalidation.rs and workspace_query_8/9 tests retain provider/root/mode/name/generation validity | Reuse the right unit with correct invalidation. Discovery completion cannot certify relation proof/currentness. |
| [#73 edit reindex](https://github.com/codeliftsleep2/Clean-CTX/issues/73), CLOSED | bridge/indexing.rs still calls client.index_project(repo_path,mode,project); cache invalidation is now project-scoped, preserving unrelated projects; e2e_reindex.rs tests this; d3b85b6f separately repairs Clean refresh lifecycle | Do not infer file-scoped CBM indexing from closed status: current indexing remains project-level. Preserve explicit scope/freshness boundaries. |
| [#75 framework parsing](https://github.com/codeliftsleep2/Clean-CTX/issues/75), CLOSED | a08c2325,6d498399,0d63c352,3ede31f4; combined MetaLayerPass evaluation shares compile evidence and extraction; pipeline_meta_layer.rs counts work | Extract once per compilation and share results; these counters do not cover later admission projections/witness IO. |

Additional closest precedent: 6e75e571 and query/prepare.rs::PreparationContext
lazily own compatible C#/Angular/EF catalogs for one request/coordinator, invalidate
derived state after publication, and separate completeness interpretations.
workspace_query_performance_work_counts.rs asserts request-local build/reuse and
invalidation. Resolver admission does NOT bypass an existing retained-authority
validation context: none exists in the inspected entry point. PreparationContext
lives above query evaluation; admission also runs during publication, restore,
pending transitions and snapshots outside that coordinator. Reusing its catalog
cache directly would cross lifecycle/authority boundaries. P0 adopts call-local
ownership, rather than making query caches grant admission authority.

## Why previous tests missed this defect

Existing work-count tests measure parses, discovery scans, catalog builds or
file-removal buckets. They did not count complete relation projections or witness
replays per candidate at retained admission. Semantic tests proved identity/refusal
without work scaling. Thus compile-stage single-pass tests could remain GREEN
while later admission reconstructed everything per edge. This is an architectural
enforcement gap at a production lifecycle boundary, not permission to weaken proof.

The new mandatory retained-admission regression now measures zero projections for
unrelated/empty/legacy-only admission; one projection for 1/2/4/16 canonical edges;
one replay per mapping. Keep compile/discovery/catalog counters separate so zero
compile work cannot hide repeated admission IO.

## Observed RED and small fixed baseline

resolver-replay-p0-before-20261103-122658: 2 passed, 3 failed, 5.55s.
Two valid architecture REDs: two edges cause two projections; five mappings replay
ten times per projection. Fixed measurement: 4 files, 3 objects, 5 mappings,
4 edges, 325 witnesses; fixture 288606us, admission 872742us, 4 projections,
40 mapping replays. This is fixed-fixture evidence, not enterprise performance
or attribution of every second in the slow six-test filters.

The third failure is an invalid fixture assumption. Host::seed owns supplied
immutable bytes without rereading/re-witnessing the owner. Owner-currentness uses
the outer caller's actual current hash; changing only the disk file while reusing
direct immutable authority does not exercise that caller. The corrected control
changes a consumed router declaration dependency. No production currentness policy
was weakened or repaired to satisfy this control. Prior bytes/logs/hashes are in
 target/issue-162/replay-p0-observed-red/receipt.json. Four valid work-count/timing/
unrelated functions are unchanged; hashes: unchanged-valid-assertions.json.
The corrected dependency control is UNRUN.

## P0 correction authored

admits_relation_evidence owns a lazy Option<Vec<SemanticEdge>> for canonical resolver
projection in that single synchronous call. Only canonical ResolvedBy initializes
it. All candidates retain exact relation/layer/call-evidence/domain/type/name/file/
canonical comparisons. Nothing survives the call or authorizes later requests.
No cache, synchronization, completeness/currentness contract or generation changes.

project calls resolve_mapping_at once per candidate mapping. That replay already
proves exact unique demanded reference, owner/field/Object/pair/key/value and target
hash/span/witness joins. Remove the second demanded_reference_at replay but retain
a unique source/value-record check for its extra crossed-hash competing-record
refusal. Independent value_reference_at remains; source currentness, selected-id
verification and duplicate/partial/ownership checks remain. A crossed-hash borrowed
same-value record is covered by the new forged test.

All post-fix counts/timings/tests remain UNRUN. Next operator command is only
run-resolver-replay-p0.ps1 -Phase After. Do not run the full campaign before this
proves the repeated-work correction. Full generation-78 and binary gates remain
pending; public key-specific design remains suspended.

## Existing regression extensions identified

- workspace_query_performance_work_counts.rs: add retained resolver projection/
  replay counts around actual canonical warm/cold queries, snapshot reads, restore
  and publication. Catalog-build counts alone miss these non-query entry points.
- pipeline_meta_layer.rs: keep combined-registry/shared-shape/detection assertions;
  pair new producer coverage with downstream admission work tests rather than
  treating one parse as proof of one lifetime validation.
- workspace/index_performance.rs: extend after #173/#174 design approval with batch
  refinement counts and semantic-density/unique-fact insertion work. Preserve its
  current file-local removal regressions.
- Keep RED-CV10 batch parse, discovery-cache validity, and issue_144 warm-work tests;
  they protect different work units and cannot substitute for admission coverage.

These larger integration extensions are identified, not implemented or reported
GREEN in P0. The new direct authority tests immediately cover the missing boundary.

## Sibling audit, not changed by P0

| Consumer | Current static pattern |
| --- | --- |
| canActivate/four guard relations | route_guards::admits reconstructs memberships per edge |
| eager Component | route_components::admits calls complete project per edge |
| lazy Component | route_lazy_components::admits calls project per edge; arrow replay checks witnesses |
| LoadsChildren | route_load_children::admits calls project per edge |
| route providers | route_providers::admits calls project per edge |
| witness validation | mapping/arrow/native proof repeatedly validate filesystem observations |
| snapshots | semantic_snapshot holds semantic_edge_snapshots mutex during is_current/admission |

These are inspected patterns, not measured wall-time attribution. No duplicate
process cause is asserted. #173/#174 remain independent possible additional costs.

## Shared validation mechanism: warranted design, separate disposition

A shared operation-scoped authority-validation context warrants review because
publication, restore, snapshots and relation consumers repeat the same immutable
authority/witness set. Bind it to exact authority/source hash, typed witness
expectations, dependencies/config/package identity and a named stable consumer
operation. Contradictory observations must refuse, not deduplicate merely by path.
Do not reuse it after publication/mutation/hash change or across later calls.
A request containing hydration/publication is not automatically one stable operation.

Before broader implementation, decide validation entry/exit points, generation/
revision checks, failure behavior under concurrent external edits and context
ownership. The filesystem is not an atomic snapshot; reuse must not claim stronger
currentness than observed. No TTL or cross-call cache. Moving IO out of the snapshot
mutex separately requires an explicit revision/publication race protocol. These
shared currentness/lifecycle/lock decisions return for architectural disposition.
P0 makes none of those changes. No new GitHub issue is created.
