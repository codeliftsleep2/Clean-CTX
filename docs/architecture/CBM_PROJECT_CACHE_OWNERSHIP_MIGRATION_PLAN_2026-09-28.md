# CBM project-owned cache migration plan - 2026-09-28

**Status:** Complete (2026-09-28). All six implementation phases, the
architectural audit, and the authoritative Final Verification Gate are
complete.

**Approval state:** The maintainer approved the decisions in Section 9 on
2026-09-28. Any condition listed there as requiring renewed approval remains a
hard stop.

**Scope:** The Clean-CTX `GraphBridge` freshness and graph-query result cache
used by `apply_edit`, structured graph tools, explicitly scoped graph queries,
and the memory/SQLite cache lifecycle.

**Out of scope:** A file-scoped CBM indexing API, dependency-aware invalidation
for arbitrary graph queries, CBM watcher adoption, changes to public MCP
responses, workspace-query hydration caching, and the separate `diff_commits`
Git-subprocess performance finding.

---

## 1. Decision summary

Adopt this invariant:

> Every cached graph-query result is owned by exactly one canonical CBM
> project. A successful graph refresh invalidates all and only that project's
> cached results.

Retain these existing behaviors:

- `apply_edit` marks the containing CBM project dirty without synchronously
  reindexing it.
- Freshness remains a per-project generation pair:
  `dirty_generation` and `indexed_generation`.
- Multiple edits to one project coalesce into one lazy refresh.
- The next graph operation for a dirty project calls CBM's repository-scoped
  `index_repository(repo_path, "fast")` contract.
- The project becomes clean only after the CBM refresh succeeds.
- A failed refresh leaves the project dirty and returns the existing error.
- Every cached query for the refreshed project is invalidated because an edit
  can change global searches, negative results, paths, architecture summaries,
  or arbitrary Cypher answers.

Change these behaviors:

- Memory cache ownership becomes structural instead of being inferred from
  the active project and string conventions.
- Refreshing project A no longer clears memory entries owned by project B.
- Refreshing a non-active project invalidates that project's disk partition,
  not the active project's partition.
- Switching the active project or workspace root no longer clears unrelated
  memory entries merely to prevent cross-project reuse; typed keys prevent the
  reuse by construction.

This is a cache-ownership correction. It is not a second graph dependency
system and does not attempt to predict which individual graph queries depend
on an edited file.

---

## 2. Pre-migration production lifecycle

The edit-to-query path observed before this migration was:

```text
apply_edit
  -> commit source + canonical IR + semantic edges
  -> mark_project_dirty(edited_path)
       -> resolve canonical CBM project
       -> increment project dirty_generation
  -> return without a CBM call

next graph operation for that project
  -> ensure_indexed / ensure_indexed_for
  -> dirty_generation > indexed_generation
  -> index_repository(repo_root, "fast")
  -> set indexed_generation to captured dirty_generation
  -> invalidate_cache()
       -> clear the complete in-memory DashMap
       -> delete the active project_root's SQLite partition
  -> execute graph query
```

The important ownership facts are:

- `project_ids` and `project_paths` are the canonical root/project mapping.
- Freshness is already correctly separated per canonical CBM project.
- The in-memory cache is `DashMap<String, CachedGraphData>` and does not encode
  project ownership in its key type.
- Most active-project query keys omit the project because project switches
  clear the complete memory cache.
- Some explicitly scoped queries embed a project in an ad hoc string key.
- The disk store partitions by `project_root`, while its string `cache_key`
  receives an additional project prefix from `disk_key`.
- `invalidate_cache` derives its disk target from the bridge's active
  `project_root`, even when the completed refresh targeted a non-active
  project.
- Raw `cbm_proxy` calls bypass the structured graph cache and are unaffected by
  this migration except for their existing readiness/freshness gate.

### 2.1 Demonstrated defects

1. A successful refresh clears every in-memory cache entry, including entries
   belonging to unrelated configured projects.
2. A successful non-active-project refresh can delete the active project's
   disk partition while leaving the refreshed project's disk partition stale.
3. Project isolation depends on key formatting and destructive cache clearing,
   not on a type that makes cross-project reuse impossible.
4. Project switching discards reusable memory entries even though the disk
   cache is intended to retain independently partitioned project results.

### 2.2 Claim explicitly rejected

The migration does not classify the repository-scoped CBM call itself as a
Clean-CTX defect. The verified integration contract accepts a repository path,
and CBM owns incremental/content-hash processing inside that boundary.
Clean-CTX must not invent a file parameter that the provider does not expose.

---

## 3. Target architecture

### 3.1 Memory ownership

Use one private typed key for every structured graph-cache entry:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GraphCacheKey {
    project: String,
    query: String,
}
```

The exact field representation may use an existing canonical project newtype
if one exists by implementation time. Do not introduce a repository-wide
identity framework solely for this migration.

`GraphBridge` then owns:

```rust
cache: DashMap<GraphCacheKey, CachedGraphData>
```

Required construction rules:

- active-project operations construct keys from `project_str()`;
- explicitly scoped operations construct keys from their resolved canonical
  project argument;
- no caller manually prepends or parses a project string;
- mock/test seed helpers use the same constructors as production code;
- unknown projects may be held in memory under their literal identity only if
  the existing CBM call succeeds, but they must not be assigned an invented
  filesystem root for disk persistence.

### 3.2 Disk ownership

Keep the existing SQLite schema unless implementation proves it cannot express
the approved ownership rule. A schema migration is not authorized by this
plan merely for aesthetic normalization.

Disk operations must receive the owner explicitly:

```text
canonical project
  -> project_paths lookup
  -> canonical project root
  -> existing (project_root, project-prefixed query key) partition
```

An explicitly scoped query must never use the active project's root when
reading, writing, or invalidating its disk entry.

If a project has no registered root, structured cache behavior remains
memory-only for that project. Do not guess a root from its slug.

### 3.3 Invalidation boundary

Introduce one internal operation with an explicit owner:

```rust
invalidate_project_cache(project: &str)
```

Its contract is:

1. resolve the supplied identity to the canonical CBM project;
2. remove only memory entries whose key owner equals that project;
3. resolve the registered root through `project_paths`;
4. delete that root's disk-cache partition;
5. never mutate freshness generations;
6. never switch the active project;
7. never contact CBM.

The existing public/internal convenience methods may delegate to the active
project version where compatibility requires them. There must be no ambiguous
invalidation call on a path that has already identified a different target
project.

### 3.4 Refresh ordering

Preserve the current success boundary:

```text
capture dirty generation
  -> call CBM index_repository
  -> on success, advance indexed generation
  -> invalidate that project's Clean-CTX cache
  -> allow query
```

On failure:

- do not advance the indexed generation;
- do not discard the last cached generation as though refresh succeeded;
- preserve existing error propagation;
- allow the next graph operation to retry through the existing circuit and
  readiness policies.

The implementation must inspect whether any current failure path serves old
cache entries before reaching the freshness gate. This plan does not authorize
stale-success fallback if the current contract returns an error.

### 3.5 Concurrency

Keep the existing `graph_bridge_lock()` serialization for edit dirty-marking,
lazy refresh, and query orchestration. Do not add another mutex, background
worker, invalidation channel, or lock hierarchy.

The existing `DashMap` may remain even if the outer bridge lock serializes
most production access; replacing it is unrelated to this migration.

---

## 4. Non-negotiable invariants

1. Canonical CBM project identity remains defined by `CBM-ID-001`.
2. No cache entry can be read under a different project than its owner.
3. A refresh invalidates all cached graph results for its target project.
4. A refresh does not invalidate another project's memory or disk entries.
5. A non-active-project operation never derives persistence ownership from the
   active project.
6. Dirty edits remain lazy and multiple edits continue to coalesce.
7. Only a successful CBM refresh advances `indexed_generation`.
8. Failure remains distinguishable from a valid empty graph result.
9. `apply_edit` response shape and latency boundary remain unchanged.
10. Public graph tool request/response contracts remain unchanged.
11. Cache TTL semantics remain unchanged.
12. Cache-scope configuration (`Global` versus `PerWorkspace`) remains
    unchanged.
13. No file-scoped reindex API or dependency-provenance registry is invented.
14. No new global limits, timeouts, retries, or truncation behavior are added.
15. Tracked regressions live under `src/tests/**`; scratch harnesses do not
    count as verification.

---

## 5. Phased implementation

Every phase is a reviewable checkpoint. Production behavior remains the
reference except for the explicitly approved ownership corrections above.
Cargo tests, builds, Clippy, and the final gate are operator-run under the
repository's long-running-process policy.

### Phase 0 - Approval and baseline inventory

**Status:** Complete (2026-09-28). The maintainer approved the plan and the
read-only production/test inventory below was completed. No Cargo command or
production edit was performed.

Deliverables:

- approve the decision in Section 1;
- enumerate every `cache.get`, `cache.insert`, `cache.remove`, `cache.clear`,
  disk get/put, project switch, and refresh invalidation call site;
- classify each call as active-project, explicit-project, mock-only, or global
  administrative behavior;
- record the current user-visible behavior and existing test authority;
- confirm modified production files will remain at or below the 615-line
  ceiling after decomposition.

Checkpoint 0 exit criteria:

- the cache call-site inventory is complete;
- no implementation change has started before approval;
- any newly discovered public contract or incompatible persistence requirement
  is returned to the maintainer as a new decision point.

#### Phase 0 checkpoint evidence

The complete production cache surface is:

| Ownership class | Operations | Current key/owner behavior |
|---|---|---|
| Active-project intelligence | symbol importance, blast radius, dead code, call edges, architecture, cross-language endpoint | Plain string keys; memory and disk ownership derived from active bridge state |
| Active-project graph | `query_graph`, `search`, `trace_path` | Plain string keys; project omitted because switching projects clears memory |
| Explicit-project graph | `query_graph_scoped`, `search_scoped` | Project embedded manually in the query string, but shared helpers still derive disk root/project prefix from active state |
| Raw proxy | `proxy_call` | Deliberately uncached; freshness gate only |
| Memory maintenance | expiry removal, insert, `invalidate_symbol`, complete clear | Operates on untyped string keys; `invalidate_symbol` has no production caller found |
| Disk maintenance | get, put, invalidate key, invalidate project | SQLite is keyed by `(project_root, cache_key)`; bridge adds an active-project prefix to `cache_key` |

The complete refresh/invalidation surface is:

1. `reindex_active_project` refreshes the active root and then calls the
   ambiguous complete `invalidate_cache` operation.
2. `reindex_for_file` resolves a target project/root but then discards that
   identity and calls the same active-state invalidation operation.
3. `ensure_indexed_for` refreshes a non-active project/root and then calls the
   same active-state invalidation operation.
4. `set_project` clears the entire memory cache when the active slug changes.
5. `set_workspace_root` clears the entire memory cache when the active root
   changes.
6. `clear_cache` delegates to `invalidate_cache`; existing tests exercise both
   convenience methods.
7. `apply_edit` only marks the containing project dirty. Its source/IR and
   hydration-discovery cache lifecycle is separate and remains out of scope.

Existing focused test authority is registered through `src/cbm/tests.rs` and
the `#[path = "../tests/cbm/..."]` convention. Relevant coverage currently
lives in:

- `src/tests/cbm/regression.rs` for project identity, refresh generations,
  reindex targeting, and complete memory invalidation;
- `src/tests/cbm/graph_intel.rs` for disk-cache workspace isolation;
- `src/tests/cbm/cache_store.rs` for SQLite key/project invalidation;
- `src/tests/cbm/e2e_reindex.rs` and `e2e_multiroot.rs` for live refresh.

Phase 1's intended semantic owner is a focused
`src/tests/cbm/cache_ownership.rs` module because project-owned graph caching is
a coherent test boundary, not because the existing regression file is large.
If implementation evidence shows that the correct change must alter or move
coverage currently owned by `src/tests/cbm/regression.rs`, that 1,633-line
legacy file becomes active and must be decomposed along semantic boundaries so
every resulting new or modified code file is at most 615 lines. The migration
must never choose a weaker test or production boundary to avoid that work.

Production-file size inventory at approval time:

| File | Lines | Phase implication |
|---|---:|---|
| `src/cbm/bridge.rs` | 372 | Safe for the private typed key |
| `src/cbm/bridge/cache.rs` | 274 | Safe for owner-aware helpers |
| `src/cbm/bridge/query.rs` | 579 | Decompose along query-family boundaries if the correct migration would exceed 615 lines; do not constrain the design to avoid decomposition |
| `src/cbm/bridge/indexing.rs` | 498 | Safe for target-specific invalidation calls |
| `src/cbm/bridge/lifecycle.rs` | 263 | Safe for removal of switch-time clears |
| `src/cbm/cache_store.rs` | 185 | No schema change currently required |
| `src/mcp/tool_handlers/edit.rs` | 566 | Inspection only; no edit is currently required |

No persistent schema incompatibility or public MCP contract change was found.
No renewed architectural approval is required to begin Phase 1.

### Phase 1 - Establish unchanged RED contracts

**Status:** Complete (2026-09-28). Three tracked regressions were authored,
registered, and run against the unchanged production implementation. All three
compiled and failed at their intended ownership assertions. The exact tests
and their module registration are stashed for unchanged restoration after the
production implementation.

Add the narrowest tracked regressions before production changes. Required
behaviors:

1. refreshing project A preserves project B's in-memory entry;
2. refreshing non-active project B removes B's disk entries and preserves A's;
3. identical query text in projects A and B cannot collide;
4. switching A -> B -> A can reuse A's valid memory entry without a CBM call;
5. failed refresh leaves the target dirty and does not claim cache freshness;
6. multiple dirty marks still coalesce at the refresh boundary.

Preferred authority is the existing CBM test tree, extending focused files
rather than creating a generic architectural-test framework.

Required RED/GREEN procedure:

- add only the regressions and required test registration;
- have the operator run the focused tests against the unfixed implementation;
- observe failure at the intended ownership assertion;
- stash only the exact regression changes;
- implement production phases with the regression absent;
- restore the unchanged regression;
- have the operator run the same focused tests and observe GREEN.

Checkpoint 1 exit criteria:

- each failure demonstrates one approved defect rather than a compile error;
- no production code is modified in the RED checkpoint;
- the exact test names and operator commands are recorded in the phase handoff.

Authored regression authority:

- `project_switch_preserves_the_original_projects_memory_entry`;
- `explicit_project_disk_hydration_uses_that_projects_registered_root`;
- `active_project_invalidation_deletes_only_that_projects_disk_partition`.

Existing unchanged authority continues to cover refresh failure and dirty
generation coalescing in `src/tests/cbm/regression.rs`; duplicating those tests
would not add a new contract.

Operator RED command:

```powershell
cargo test --all-features cbm::tests::cache_ownership -- --nocapture
```

Expected failures are behavioral assertions or their explicit prerequisite
`expect` messages: project-switch memory retention, project-B disk hydration,
and target-project disk invalidation. A compile error or unrelated failure is
not valid RED evidence.

Observed RED evidence (operator-run on 2026-09-28):

- `project_switch_preserves_the_original_projects_memory_entry` failed because
  switching A -> B -> A discarded A's seeded memory entry and attempted to
  launch unavailable CBM;
- `explicit_project_disk_hydration_uses_that_projects_registered_root` failed
  because the explicitly selected B entry was not hydrated from B's registered
  root and the bridge attempted to launch unavailable CBM;
- `active_project_invalidation_deletes_only_that_projects_disk_partition`
  failed because invalidating B deleted A's disk partition.

This is valid behavioral RED evidence. The earlier invalid `GraphBridge::new_mock`
test-construction attempt was corrected before this run and is not counted as
RED evidence.

### Phase 2 - Introduce typed memory ownership

**Status:** Complete (2026-09-28). The unchanged Phase 1 regressions passed
after the production implementation.

Deliverables:

- add the private typed memory-cache key;
- centralize active-project and explicit-project key construction;
- migrate memory lookup, insertion, expiration, and removal helpers;
- migrate structured search, query, trace, architecture, intelligence, and mock
  seed paths without changing their query payloads;
- decompose any activated oversized file, or any file whose correct change
  would cross 615 lines, along its real semantic boundaries;
- retain compatibility delegation only where needed for an intermediate
  compiling checkpoint.

This phase must be primarily mechanical: existing query strings, result
parsing, TTL calculations, and error behavior remain unchanged.

Checkpoint 2 exit criteria:

- every production memory entry has structural project ownership;
- no production caller manually embeds a project into an untyped key;
- active and explicitly scoped queries with identical query text remain
  distinct;
- no old unscoped insertion path remains reachable.

Implementation evidence:

- `GraphBridge::cache` is keyed by `GraphCacheKey { project, query }`;
- active and explicit-project cache access share owner-aware helpers;
- mock/test seeding uses the same typed key constructor;
- the oversized legacy regression and query-wire suites were decomposed along
  semantic boundaries, with all 97 original tests preserved.

Checkpoint evidence:

- operator-run `cargo test --all-features cbm::tests::cache_ownership --
  --nocapture` completed GREEN on 2026-09-28;
- the restored regression file and module registration matched their RED-stash
  hashes before the GREEN run.

### Phase 3 - Make disk ownership explicit

**Status:** Complete (2026-09-28). Covered by the same unchanged GREEN
regression checkpoint recorded in Phase 2.

Deliverables:

- route disk get/put through the same explicit cache owner as memory;
- resolve registered roots only through `project_paths`;
- keep unknown-project caching memory-only rather than guessing a root;
- preserve the current SQLite schema if it can express the target contract;
- add a migration proposal and stop for approval if a schema change becomes
  necessary.

Checkpoint 3 exit criteria:

- active and non-active scoped operations read and write the correct disk
  partition;
- workspace/project switching cannot cause a scoped result to be persisted
  under another root;
- restart hydration preserves the same project isolation as memory.

Implementation evidence:

- disk get/put receives the explicit canonical project;
- roots resolve only through `project_paths`;
- unknown project identities remain memory-only;
- the existing SQLite schema and key encoding remain unchanged.

### Phase 4 - Replace ambiguous invalidation

**Status:** Complete (2026-09-28). Covered by the same unchanged GREEN
regression checkpoint recorded in Phase 2.

Deliverables:

- add `invalidate_project_cache(project)` or the equivalent explicit boundary;
- use it after active-project, non-active-project, and file-resolved refreshes;
- make active-project convenience invalidation delegate explicitly;
- remove complete memory clears from ordinary project/workspace switches once
  typed ownership is the sole production path;
- preserve any deliberately global administrative clear as a distinctly named
  operation, if one is still required.

Checkpoint 4 exit criteria:

- every refresh invalidates exactly its target project;
- no refresh derives its disk target from unrelated active state;
- no normal project switch destroys another project's reusable entries;
- obsolete string-prefix and clear-on-switch safety mechanisms are removed.

Implementation evidence:

- every refresh path calls `invalidate_project_cache(project)`;
- active invalidation delegates to that explicit boundary;
- project/workspace switches no longer clear unrelated memory entries;
- manually project-prefixed query strings are removed from production.

### Phase 5 - Production lifecycle integration

**Status:** Complete (2026-09-28). The focused live production-lifecycle
regression completed GREEN with CBM available.

Trace and test the complete production lifecycle:

```text
provide_code_context(Edit)
  -> apply_edit source commit
  -> canonical project resolution
  -> dirty generation ownership
  -> next structured graph entry point
  -> successful lazy CBM refresh
  -> target-only memory/disk invalidation
  -> fresh graph answer
  -> later cache hit in the same project
  -> unrelated project cache remains usable
```

Deliverables:

- production-dispatch coverage for the edit-to-query boundary where practical;
- deterministic bridge tests for multi-project ownership;
- applicable live-CBM scenario covering a real edited symbol;
- failure-path coverage proving refresh errors remain errors;
- inspection of delete/reset/workspace-switch lifecycles for obsolete cache
  state.

Checkpoint 5 exit criteria:

- the default production entry points invoke the new owner-aware boundaries;
- a passing isolated cache unit test is not used as integration evidence;
- no old invalidation path can execute alongside the replacement;
- live verification is reported separately from tracked-test results.

Implementation evidence:

- the registered `apply_edit` handler commits source/durable/live semantic
  state before marking the resolved CBM project dirty;
- registered structured graph handlers run `ensure_indexed` before cache/query
  consumption and expose the resulting typed response;
- the live `e2e_apply_edit_triggers_reindex_and_graph_is_fresh` regression now
  drives the real `graph_search` handler after `apply_edit`, asserts the fresh
  result reaches `structuredContent`, repeats the external request through the
  cache-hit path, and proves an unrelated project's sentinel entry survives;
- the focused live checkpoint remains operator-run and pending.

Operator live-checkpoint command:

```powershell
cargo test --all-features cbm::tests::e2e::e2e_reindex::e2e_apply_edit_triggers_reindex_and_graph_is_fresh -- --nocapture
```

A skipped test because CBM is unavailable is not live integration evidence.

Checkpoint evidence:

- operator-run
  `cargo test --all-features cbm::tests::e2e::e2e_reindex::e2e_apply_edit_triggers_reindex_and_graph_is_fresh -- --nocapture`
  completed GREEN on 2026-09-28;
- the test traversed the registered edit and structured graph handlers rather
  than stopping at an isolated bridge API.

### Phase 6 - Documentation, audit, and final gate

**Status:** Complete (2026-09-28). The maintainer reported every command in
the authoritative Final Verification Gate GREEN after the final repair.

Deliverables:

- update `CBM-ID-001` or add a narrowly scoped cache-ownership invariant in
  `docs/ARCHITECTURAL_INVARIANTS.md`;
- update `docs/agent/tooling.md` only where operational behavior changed;
- correct stale comments that imply the complete cache belongs only to the
  active project;
- record any reproducible live discovery in
  `docs/agent/DISCOVERY_REGISTRY.md`;
- perform the post-task architectural audit from
  `docs/agent/architecture.md`;
- run the authoritative final verification gate from
  `docs/agent/verification.md`.

Checkpoint 6 exit criteria:

- implementation, tests, and documentation describe the same ownership model;
- all obsolete paths and transitional adapters are removed;
- modified code files satisfy the active-file ceiling;
- strict UTF-8 checks pass;
- the operator reports the complete final gate results;
- no unrun gate is described as passing.

Final architectural audit (2026-09-28):

- **Correctness and integration:** the production edit producer, freshness
  owner, refresh boundary, memory/disk owners, structured graph consumer, MCP
  response, repeated cache hit, and live reachability were traced and covered;
- **Obsolete paths:** no `DashMap<String, CachedGraphData>`, switch-time
  `cache.clear`, ambiguous refresh invalidation, manually project-prefixed
  structured query key, or active-root disk access remains in production;
- **Failure behavior:** refresh generations still advance only after success;
  existing failure and coalescing regressions remain authoritative;
- **Separation of concerns:** project identity remains in lifecycle/project
  mapping, cache ownership in the cache boundary, refresh orchestration in
  indexing, and query payload behavior in query modules; no new global lock,
  worker, schema, or public contract was introduced;
- **Rust ownership:** `GraphCacheKey` structurally enforces memory ownership;
  explicit registered-root lookup enforces disk ownership without interior
  mutability or a parallel cache;
- **Legacy debt:** activated oversized regression and query-wire test suites
  were decomposed along semantic boundaries; all 77 regression tests and 20
  query-wire tests were preserved, and every modified/new Rust file is at most
  615 lines;
- **Documentation:** `CBM-CACHE-001`, `CBM-VERIFY-002`, the tooling guide, code
  comments, and this migration record now describe the implemented model.

The first complete workspace-test attempt reached 151 passing proxy tests but
was blocked by four pre-existing rate-limiter tests whose real-time sleeps and
immediate-exhaustion assertions became scheduler-dependent under full-gate
load. Production rate-limit behavior was left unchanged; the token-bucket
calculation now has an internal explicit-instant boundary, and the relocated
proxy tests advance that clock deterministically. The maintainer reported the
focused `clean-ctx-proxy` rate-limiter test target GREEN on 2026-09-28. A
subsequent `cargo test --workspace --all-targets --all-features` rerun was also
reported GREEN. The post-repair formatting, Clippy, file-size, and encoding
gates were then reported GREEN.

No critical or high-severity architectural gap remains. The complete
operator-run Final Verification Gate passed, so Phase 6 and the migration are
complete.

---

## 6. Verification matrix

| Contract | Focused authority | Live boundary |
|---|---|---|
| Same query text is project-isolated | CBM bridge cache regression | Query two indexed roots with the same symbol name |
| Project A refresh preserves B memory | Multi-project bridge regression | Warm B, edit/query A, query B |
| Non-active B refresh invalidates B disk | SQLite cache + bridge regression | Restart and query B after an A-active/B-refresh sequence |
| Project switching preserves safe reuse | Lifecycle/cache regression | A -> B -> A repeated query |
| Dirty edits coalesce | Existing freshness-generation regression | Multiple edits followed by one graph query |
| Refresh failure remains dirty | Indexing failure regression | Optional controlled unavailable-CBM scenario |
| Edit reaches fresh graph | Registered MCP production-path test | Real `apply_edit` followed by graph search/trace |

Wall-clock thresholds are not gate authority. If latency measurement is useful,
record it as observational evidence with repository size, CBM version, cold or
warm state, index mode, and whether the provider used an incremental route.

---

## 7. Rollback boundaries

- **After Phase 1:** tests only; remove or retain them according to the exact
  RED/GREEN procedure. No production rollback is needed.
- **After Phase 2:** typed memory keys can be reverted without disk migration
  because no schema change is planned.
- **After Phase 3:** disk helpers can revert to the existing schema paths; no
  stored-data rewrite should be required.
- **After Phase 4:** revert owner-aware invalidation and switch behavior as one
  unit. Do not retain typed keys while restoring assumptions that untyped
  project switches clear all ownership ambiguity.
- **After Phase 5:** production integration and old-path removal are one
  checkpoint. Never ship both invalidation paths concurrently.

Rollback never authorizes deletion of the user's CBM databases or Clean-CTX
cache database. Test databases must remain temporary and isolated.

---

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| A missed cache call site retains an unscoped key | Complete Phase 0 inventory; central constructors; search-based final audit |
| Explicit-project queries use the active root on disk | Require project plus root resolution in disk helpers; non-active regression |
| Old disk rows become unreachable | Preserve schema/key encoding where possible; verify restart hydration |
| Mock availability relies on a sentinel cache entry | Migrate mock seeds through typed constructors and retain availability tests |
| Project switching behavior changes hidden test assumptions | Treat switch-clear removal as Phase 4, after structural ownership is complete |
| Fine-grained invalidation is reintroduced ad hoc | Document all-project-results invalidation as the correctness boundary |
| CBM version behavior differs | Keep Clean-CTX's provider contract repository-scoped; record version in live evidence |
| Migration expands into cache/framework redesign | Stop at the boundaries in Scope and Non-negotiable invariants |

---

## 9. Approval checkpoint

Approval of this plan authorizes the following architectural decisions only:

1. project-level freshness remains the correctness boundary;
2. repository-scoped CBM fast refresh remains the provider boundary;
3. every Clean-CTX graph-cache entry gains canonical project ownership;
4. successful refresh invalidates all and only the target project's cache;
5. ordinary project switching stops clearing unrelated memory entries after
   typed ownership is fully integrated;
6. no public MCP contract, cache TTL, retry, timeout, or query-completeness
   policy changes.

Implementation must stop for renewed approval if it requires:

- a CBM protocol change or minimum-version increase;
- a persistent cache schema migration;
- file-scoped or dependency-aware invalidation;
- stale-data fallback after refresh failure;
- a new global cache, lock, worker, limit, timeout, retry, or public response
  field;
- behavior changes outside the graph cache/freshness lifecycle.

The separate `diff_commits` subprocess optimization requires its own issue and
plan if pursued. It is not authorized by approval of this document.
