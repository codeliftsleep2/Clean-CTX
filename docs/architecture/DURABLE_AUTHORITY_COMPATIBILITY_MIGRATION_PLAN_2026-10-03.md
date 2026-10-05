# Durable Authority Compatibility Migration Plan

**Status:** Implementation in progress; Phases 0-9 complete
**Branch:** `feature/durable-authority-compatibility`  
**Issues:** #117, #119, prerequisite and integration work for #116  
**Out of scope:** #118, Binary `0x04` redesign, relation-family persistence,
whole-configuration or whole-executable fingerprints

**Phase status (2026-10-04):** Phases 0-8 complete; Phase 8 typed semantic
publication, truthful coverage, and affected restore/replay/delta/recovery
authorities reported GREEN by the repository owner

## 1. Objective

Implement the frozen durable compatibility envelope so that decodable durable
state becomes live authority only when it is compatible with the running
canonical and semantic configuration and producer generations.

The migration must establish four separate identities:

```text
CanonicalConfigIdentity
CanonicalProducerIdentity
SemanticConfigIdentity
SemanticProducerIdentity
```

It must then validate those identities before ordinary restore, replay, delta
reuse, semantic publication, or coverage certification mutates live authority.
Pending-edit recovery may complete an older durable transaction, but may not
publish or relabel the recovered artifact unless it is compatible with the
current runtime.

## 2. Frozen invariants

1. Physical decoding and structural coherence do not establish semantic
   authority.
2. Canonical and semantic authority have independent configuration and
   producer identities.
3. Only producers relevant to an artifact participate in its identity.
4. Persisted and currently relevant producer sets compare bidirectionally.
5. One durable delta chain belongs to one canonical compatibility epoch.
6. Compatibility validation precedes every ordinary live-state mutation.
7. Crash recovery preserves the recovered transaction's original identities.
8. Missing legacy identity is not evidence of compatibility.
9. Coverage certifies only an already-compatible semantic projection.
10. Compatible persistence reuse remains supported.

## 3. Scope boundaries

This migration owns:

- identity derivation and scoped producer generations;
- atomic persistence of compatibility metadata;
- structural-load versus authoritative-adoption separation;
- compatibility validation for restore, replay, delta chains, and edit
  recovery;
- typed compatibility evidence for semantic publication;
- completion of #116 only after compatibility is enforced;
- structured diagnostics for every rejection class;
- tracked production-path regressions.

This migration does not own:

- reconciliation of an already-live owner with the current source universe
  (#118);
- arbitrary same-session configuration mutation;
- a generic cache-key audit;
- Binary `0x04`, WorkspaceIndex, or semantic-store redesign;
- partial adoption of relation-family subsets;
- deleting or automatically blessing legacy persistence.

## 4. Current production boundaries to preserve

The implementation must trace and preserve these existing paths before each
phase edits them:

```text
source
  -> feature-aware language producer
  -> canonical compiler pipeline
  -> framework marker and semantic producers
  -> post-compilation type-alias transform
  -> canonical IR + semantic edges
  -> contexts baseline + delta chain
  -> semantic_edge_snapshots
  -> restore/replay/recovery
  -> IRContext + semantic cache + WorkspaceIndex + coverage
```

Existing behavior remains the reference except where the approved
compatibility rejection deliberately changes adoption semantics.

## 5. Required RED/GREEN procedure

Every reproducible defect boundary uses the repository's authoritative
RED/GREEN process from `docs/agent/architecture.md`.

For each behavioral slice:

1. Add the narrowest tracked regression under `src/tests/**` and its normal
   `#[path = "..."]` registration. Test-only deterministic generation seams
   may be added first when alternate builds would otherwise be required; they
   must not change production behavior.
2. Hand the exact focused `cargo test --all-features <test-name> -- --exact`
   command to the repository owner.
3. The owner runs it against the unfixed implementation and reports RED from
   the intended assertion. Compilation failure or an unrelated failure is not
   valid RED evidence.
4. Stash only that regression and any required test-module registration:

   ```powershell
   git stash push -u -m "red durable compatibility <slice>" -- <test-path> <registration-path>
   ```

5. Implement the production change while the regression is absent.
6. Restore the exact regression without editing it:

   ```powershell
   git stash apply stash@{0}
   ```

7. Hand the same focused command to the owner and record GREEN.
8. Keep RED and GREEN results categorized separately from static inspection
   and any operator harness.

If the test itself is invalid, correct it and restart the slice from RED. Never
weaken an observed assertion to obtain GREEN.

No agent starts Cargo commands. Phase checkpoints are not claimed green until
the owner supplies the corresponding results.

## 6. Phased implementation

### Phase 0 - Baseline inventory and change map

**Purpose:** Freeze the exact edit surface before production changes.

Tasks:

- record all callers of baseline save/load, semantic snapshot write/load,
  delta append, edit-intent establishment/recovery, restore, replay, and
  semantic publication;
- record active-file line counts before modifying Rust files;
- identify legacy files over 615 lines that require semantic decomposition if
  activated;
- identify existing persistence/restart tests and operator harnesses to extend;
- map each configuration field to its actual canonical or semantic consumer;
- map generation ownership to the producing module rather than persistence.

Exit criteria:

- every durable authority transition has one named owner in the plan;
- no production edit has begun;
- any file-size decomposition required by later phases is planned by semantic
  boundary, not used to weaken the architecture.

#### Phase 0 completion record (2026-10-03)

Phase 0 completed by static repository inspection. No production file and no
test file was changed, and no Cargo command was run.

##### Durable transition ownership

| Transition | Current production owner | Current authority effect | Planned migration owner |
|---|---|---|---|
| Automatic provide baseline save | `tool_handlers/core/provide_persistence.rs` | Writes baseline IR and semantic snapshot | Compatibility-aware persistence API |
| Explicit `save_context` | `tool_handlers/persistence/mod.rs` | Writes current baseline IR and semantic snapshot | Compatibility-aware persistence API |
| Delta baseline establishment | `tool_handlers/core/delta/persistence.rs` | Creates or reuses durable baseline | Canonical epoch validator plus persistence API |
| Delta append | `tool_handlers/core/delta_apply.rs` and `sqlite_store/semantic_state.rs` | Appends delta, updates hash/text, writes semantic snapshot | Canonical epoch validator and atomic identity-bearing append |
| Edit-intent establishment | `tool_handlers/edit.rs` and `sqlite_store/edit_intent.rs` | Persists target source, IR, edges, fidelity, and stage ownership | Identity-bearing edit-intent transaction |
| Edit-intent durable recovery | `sqlite_store/edit_intent.rs` | Reconciles bytes and commits recovered target baseline | Durable recovery only; authority validation follows separately |
| Pending-edit live recovery | `mcp/durable_semantics.rs` | Loads recovered IR and publishes edges directly | Recovery result plus shared compatibility/adoption boundary |
| `restore_context` load/adoption | `tool_handlers/core/restore.rs` | Loads IR/cache/index and currently reconstructs coverage | All-or-nothing compatible durable adoption |
| `replay_history` load/adoption | `tool_handlers/persistence/mod.rs` | Loads historical IR/cache/index without coverage | Historical compatibility validator and typed adoption |
| Delta baseline validation | `tool_handlers/core/delta/persistence.rs` | Compares file/version/hash only | Canonical chain-epoch validator |
| Edit durable precondition | `tool_handlers/edit.rs` | Loads durable state and compares version/hash | Canonical compatibility validator before reuse |
| Mutating `apply_delta` publication | `tool_handlers/core/delta_apply.rs` | Publishes target edges without coverage | Compatible semantic publication primitive |
| Fresh provide publication | `tool_handlers/core/provide/evaluate.rs` | Publishes newly compiled IR, edges, and coverage | Direct construction of current compatibility evidence |
| Fresh hydration publication | `tool_handlers/hydration/publication.rs` | Recompiles current source and publishes coverage | Direct construction of current compatibility evidence |
| Workspace entity preparation | `tool_handlers/query/entities.rs` | Recompiles and replaces current semantic projection | Remains fresh-production authority |
| WorkspaceIndex coverage | `workspace/index/coverage.rs` | Stores fidelity and source-hash evidence | Policy-free consumer of compatible projection evidence |

`sqlite_store/semantic_state.rs` remains the atomic baseline/snapshot and
delta/snapshot transaction owner. It must receive identities from the producer
boundary; it must not derive current compatibility from persistence data.

##### Configuration-to-artifact map

Canonical configuration consumers:

- `type_aliases` is applied after compilation in `mcp/tool_helpers.rs` and
  changes returned canonical instructions;
- Angular master/sub-layer marker settings flow through
  `layers/meta/mod.rs` and `angular_meta/mod.rs`; rendered Phi lines become
  canonical `CoreOp::TypeAlias` records in `ir/pipeline/meta_layer.rs`;
- Angular `rxjs.min_pipe_operators` and NgRx dispatch/select/entity rendering
  options alter marker output;
- .NET `enabled` and `testing.enabled` alter marker output;
- Spring currently consumes no project configuration.

Semantic configuration consumers:

- Angular `enabled`, `ngrx.enabled`, `routing.enabled`, and
  `testing.enabled` alter semantic-edge families in
  `layers/meta/angular_semantic.rs`;
- .NET `enabled` and `testing.enabled` alter .NET/testing edges in
  `dotnet_meta/mod.rs`;
- type aliases run after semantic projection and therefore do not enter
  semantic configuration identity;
- RxJS marker threshold, NgRx marker-detail options, signals, reactive forms,
  Formly, tokenizer, cache, proxy, persistence, observability, discovery roots,
  exclusions, and resource limits do not currently alter persisted semantic
  edges;
- `ngrx.cross_layer_cbm` has no current durable compilation consumer and is
  excluded from both identities.

Fidelity selection configuration is excluded from both config identities
because the resolved effective fidelity is already a separate durable
authority dimension.

##### Existing regression authorities

The migration will extend, rather than duplicate:

- `src/tests/mcp/durable_semantic_restore.rs` for restore, compatible restart,
  transactional rejection, and coverage;
- `src/tests/mcp/persistence_lifecycle_semantics.rs` for historical replay;
- `src/tests/mcp/delta_fidelity_persistence.rs` and delta-sequence coverage for
  chain epochs;
- `src/tests/mcp/delta_edit_recovery.rs`,
  `src/tests/mcp/replay_edit_recovery.rs`, and
  `src/tests/mcp/semantic_publication_recovery.rs` for recovery;
- existing SQLite store tests for schema migration, atomic writes, and legacy
  rows.

The tracked `verification/live-acceptance` scripts exercise unrelated
workspace-query or signature behavior. No existing persistence/restart
operator harness was found. Phase 0 therefore freezes **no new harness** as the
default. A later harness addition requires a real production-binary boundary
that cannot be adequately represented by the required tracked regressions.

##### Active-file size inventory and decomposition plan

No identified edit target currently exceeds the 615-line ceiling. Files close
enough to the ceiling to constrain implementation are:

| File | Baseline lines | Constraint |
|---|---:|---|
| `src/mcp/sqlite_store.rs` | 578 | Extract schema migration ownership before material schema growth |
| `src/mcp/tool_handlers/edit.rs` | 574 | Extract durable edit transaction helpers before material compatibility work |
| `src/mcp/tool_helpers.rs` | 570 | Keep identity derivation in a dedicated compatibility module; do not grow this helper boundary materially |
| `src/mcp/tool_handlers/persistence/mod.rs` | 533 | Extract replay handling before adding substantial validation/adoption logic |

Planned semantic decompositions:

1. Move SQLite migration steps into a dedicated
   `mcp/sqlite_store/migrations.rs`, leaving `SqliteStore::migrate` as narrow
   orchestration.
2. Move replay-history handling into a dedicated persistence handler module
   before Phase 5 changes its authority lifecycle.
3. Keep compatibility identity derivation and comparison in new focused
   modules rather than expanding `tool_helpers.rs`.
4. If Phase 7 requires material changes in `edit.rs`, extract durable
   intent/commit orchestration into an edit submodule before adding behavior.

Other primary production targets begin below 500 lines, including
`durable_semantics.rs` (354), `core/delta_apply.rs` (322),
`sqlite_store/edit_intent.rs` (244), `sqlite_store/semantic_state.rs` (238),
and `core/restore.rs` (143). Every modified or new Rust file remains subject to
the 615-line absolute ceiling and 600-line target.

##### Phase 0 exit decision

All durable authority transitions now have a named current and planned owner.
The configuration consumers, regression authorities, harness decision, and
file-size decomposition boundaries are frozen. Phase 1 may begin without an
additional architectural decision.

### Phase 1 - Compatibility value model and producer registry

**Purpose:** Introduce repository-native identity values without changing
adoption behavior.

Deliverables:

- `CanonicalConfigIdentity`;
- `CanonicalProducerIdentity`;
- `SemanticConfigIdentity`;
- `SemanticProducerIdentity`;
- versioned, deterministic structured serialization;
- component-level hashes where inputs are large;
- explicit producer keys and scoped generation constants owned beside:
  - the shared canonical pipeline;
  - TypeScript, C#, Rust, and Java canonical producers;
  - the type-alias post-transform;
  - Angular, .NET, and Spring marker producers;
  - builtin and generic semantic projection;
  - Angular, .NET, and Spring semantic producers;
- deterministic test seams that can override producer generations or relevant
  producer sets without alternate Cargo builds.

Identity inputs:

- canonical configuration includes deterministic `type_aliases` and only
  applicable marker-producing framework settings;
- semantic configuration includes only settings that alter emitted edges;
- effective fidelity remains a separate authority field;
- type aliases do not enter semantic configuration under current ordering;
- producer-set comparison is exact and bidirectional.

Tracked tests:

- deterministic serialization independent of map insertion order;
- unrelated language/framework generation changes do not alter an artifact's
  identity;
- marker-only configuration changes canonical but not semantic identity;
- semantic-producing configuration changes semantic identity;
- producer-set addition and removal are distinguishable.

These are structural contract tests. Behavioral RED/GREEN begins in the
production adoption phases below.

Exit criteria:

- no anonymous generation integers exist in persistence code;
- identity derivation is centralized and deterministic;
- no whole-config, crate-version, commit, executable, or global-feature hash
  is introduced.

#### Phase 1 implementation record (2026-10-03)

Implementation is complete and its focused structural contract suite was
reported GREEN by the repository owner on 2026-10-03.

- focused value modules live under `src/mcp/compatibility/`;
- deterministic identities use versioned structs, `BTreeMap` producer sets,
  and a component-level SHA-256 digest for normalized `type_aliases`;
- producer generations are owned beside the shared pipeline, language IR
  layers, type-alias transform, builtin/generic projections, and framework
  marker/semantic producers;
- `SemanticInputGeneration` is language-scoped and versions only canonical
  fact meanings consumed by semantic projection;
- framework applicability uses the active `LayerRegistry` with current source,
  path, and configuration and does not adopt durable IR;
- test-only generation replacement/removal seams are confined to the producer
  catalog under `cfg(test)`;
- tracked structural tests live in
  `src/tests/mcp/durable_compatibility_identity.rs`;
- no persistence, restore, replay, delta, recovery, WorkspaceIndex, coverage,
  Binary `0x04`, or SQLite behavior changed.

Owner verification command:

```powershell
cargo test --all-features mcp::compatibility::tests -- --nocapture
```

Owner-reported result: 5 passed, 0 failed. This completes Phase 1 and permits
Phase 2 to begin.

### Phase 2 - Durable ownership and SQLite migration

**Purpose:** Persist identity evidence atomically with the artifact it
certifies.

Schema changes:

- nullable canonical config and producer identity columns on `contexts`;
- nullable semantic config and producer identity columns on every
  `semantic_edge_snapshots` row;
- all four target identity columns on `edit_intents`;
- a new idempotent SQLite schema migration;
- no backfill of legacy rows.

Production changes:

- baseline save writes canonical identity in the baseline transaction;
- semantic snapshot save writes semantic identity in the snapshot transaction;
- delta append keeps the baseline's canonical epoch and writes the current
  semantic snapshot identities atomically;
- edit-intent establishment stores target identities with target source, IR,
  edges, hash, fidelity, and version;
- raw load returns identities but does not label them compatible.

Tracked regressions and RED/GREEN slices:

1. Newly persisted baselines and snapshots round-trip all four identities.
2. Legacy rows remain readable structurally but carry missing identity.
3. Failed transactional writes cannot pair new artifacts with old identities.
4. Edit intents round-trip the producer epoch that created the target.

Exit criteria:

- every new artifact/identity pair has one atomic owner;
- legacy rows remain preserved and explicitly unproven;
- Binary `0x04` is unchanged.

#### Phase 2 implementation record (2026-10-03)

Implemented schema version 6 with resumable, idempotent nullable identity
columns and no legacy backfill. Canonical baselines, semantic snapshots,
delta snapshots, and production edit intents now persist their scoped
identity evidence atomically. Structural loads preserve each identity as
optional evidence without granting compatibility.

Tracked evidence:

- schema-column regression observed RED before migration and GREEN after;
- four-identity round-trip regression observed RED with four NULL values and
  GREEN after the atomic writer was installed;
- legacy NULL evidence, mid-transaction rollback, edit-intent target epoch,
  and interrupted-migration resumption are covered as tracked contracts;
- owner-reported focused result:

  ```powershell
  cargo test --all-features mcp::sqlite_store::compatibility_persistence_tests -- --nocapture
  ```

  Six tests passed with no reported warnings.

### Phase 3 - Structural load and authoritative validation boundary

**Purpose:** Make unchecked publication difficult by construction.

Deliverables:

- structurally validated but untrusted loaded durable types;
- separate canonical and semantic compatibility validators;
- an all-or-nothing compatible durable context for ordinary restore;
- `CompatibleCanonicalState` and `CompatibleSemanticProjection`, or equally
  narrow repository-native types;
- centralized failure taxonomy:
  - physical/schema incompatibility;
  - structural snapshot incoherence;
  - source mismatch;
  - insufficient fidelity;
  - canonical configuration incompatibility;
  - semantic configuration incompatibility;
  - canonical producer incompatibility;
  - semantic producer incompatibility;
  - missing legacy identity;
  - historical source not current.

The validator recomputes relevant producer applicability from current source,
path, configuration, and the active registry. It does not adopt persisted IR
to decide applicability.

Tracked contract tests:

- each mismatch produces a distinct diagnostic;
- exact relevant-producer set equality is bidirectional;
- validation itself does not mutate IRContext, semantic caches, WorkspaceIndex,
  fidelity ownership, or presentation caches.

Exit criteria:

- raw durable values cannot reach the compatibility-aware publication API;
- compatibility policy has one implementation rather than handler copies.

#### Phase 3 implementation record (2026-10-03)

Structural SQLite loads now return `UntrustedDurableContext`. Physical/schema
decoding failures and cross-artifact structural incoherence enter the shared
`CompatibilityFailure` taxonomy before leaving persistence. Pure validators
derive current applicability from source, path, configuration, and the active
producer registry, then return separate typed canonical and semantic evidence
only after the complete context succeeds.

The validator has no access to `McpState`, IRContext, semantic caches,
WorkspaceIndex, fidelity ownership, or presentation caches. Its immutable
component validators and consuming all-or-nothing entry point therefore cannot
mutate live authority.

Owner-reported focused result:

```powershell
cargo test --all-features mcp::compatibility::validator::tests -- --nocapture
```

Nine tests passed with no reported warnings. They cover successful typed
evidence, source and fidelity rejection, scoped configuration mismatches,
bidirectional producer-set mismatch, each missing legacy component, taxonomy
separation, immutable validation, malformed identity encoding, and structural
snapshot incoherence.

### Phase 4 - Restore adoption gate

**Purpose:** Close the primary #117/#119 externally observable restore defect.

RED/GREEN slices:

1. Canonical config mismatch using `type_aliases`.
2. Canonical marker-config mismatch.
3. Semantic config mismatch, enabled-to-disabled.
4. Semantic config mismatch, disabled-to-enabled.
5. Canonical producer-generation mismatch.
6. Semantic producer-generation mismatch.
7. Persisted producer present/current absent.
8. Persisted producer absent/current applicable.
9. Legacy identity missing.
10. Positive compatible restart reuse.

Required assertions for every rejection:

- restored IR is not loaded;
- semantic session cache is unchanged;
- WorkspaceIndex facts and coverage are unchanged;
- fidelity and persisted-path ownership are unchanged;
- LLM/presentation caches are unchanged;
- an exact structured reason is returned.

Positive assertion:

- a compatible restart retains durable reuse and publishes coherent canonical
  and semantic state.

Exit criteria:

- ordinary restore validates the complete context before any live mutation;
- the current restore-only coverage reconstruction is reachable only through
  a compatible semantic projection.

#### Phase 4 implementation record (2026-10-04)

`restore_context` now loads structurally untrusted durable state, reads the
current source, and obtains all-or-nothing typed canonical and semantic
compatibility evidence before alias creation, hierarchy publication, IRContext
loading, semantic caching, WorkspaceIndex/coverage replacement, fidelity and
persisted-path ownership, or presentation caching.

Compatibility rejection returns a stable structured `reason`. The tracked
production-path matrix covers type-alias configuration, marker-only
configuration, semantic enablement in both directions, canonical and semantic
producer generations, both producer-set directions, missing legacy identity,
and compatible restart reuse. Every rejection asserts that live authority
remains untouched.

Owner-reported focused results:

```powershell
cargo test --all-features mcp::tool_handlers::core::restore::compatibility_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::persistence::durable_semantic_restore_tests -- --nocapture
```

The compatibility matrix passed ten tests with no reported warnings. The
existing durable restore authority passed four tests after its intentionally
source-divergent H1/H2 case was updated to require `source_mismatch` and zero
publication.

### Phase 5 - Historical replay gate

**Purpose:** Apply the same authority rules without erasing historical source
identity.

RED/GREEN slices:

- compatible historical replay adopts H1 with H1 identities;
- H1 coverage never satisfies current disk H2;
- incompatible config or producer identity rejects before live mutation;
- legacy historical snapshots remain durable but cannot become live authority;
- positive replay behavior remains unchanged when all identities match.

Exit criteria:

- replay uses the shared validator;
- replay does not manufacture current-source authority from historical state.

#### Phase 5 implementation record (2026-10-04)

`replay_history` now loads structurally untrusted durable state and obtains
typed canonical and semantic compatibility evidence through the shared
validator before alias creation or any live-state mutation. Identity
applicability is derived from the current source and configuration, while an
accepted historical artifact retains its persisted H1 source hash. Replay
therefore cannot manufacture current H2 authority, and raw passthrough remains
available only when the current bytes match H1.

The tracked regressions cover incompatible canonical configuration, producer
generation mismatch, missing legacy identity, compatible historical H1 replay
without H2 coverage, and compatible current-source replay. The first
configuration regression was preserved unchanged through the required
RED/GREEN procedure.

Owner-reported results:

```powershell
cargo test --all-features mcp::tool_handlers::persistence::lifecycle_tests::semantics::historical_replay_rejects_incompatible_config_before_live_mutation -- --exact
cargo test --all-features historical_replay_ -- --nocapture
cargo test --all-features mcp::tool_handlers::persistence::lifecycle_tests::semantics -- --nocapture
```

The exact regression first failed because replay adopted incompatible state,
then passed unchanged after the production gate was implemented. The full
focused Phase 5 replay suite and the existing replay lifecycle suite were also
reported GREEN, with no reported warnings.

### Phase 6 - Delta-chain compatibility epoch

**Purpose:** Prevent a durable delta chain from crossing a canonical
compatibility boundary.

RED/GREEN slices:

- baseline under epoch A accepts a delta under A;
- canonical config change A-to-B rejects reuse of the A chain;
- canonical producer change A-to-B rejects reuse of the A chain;
- rejected reuse leaves the chain and live authority unchanged;
- a new compatible baseline can be established for B;
- semantic snapshot identities remain snapshot-local and atomic with each
  accepted delta.

Exit criteria:

- no supported delta operation changes the chain epoch in place;
- sequence-delta version remains a structural check only.

#### Phase 6 implementation record (2026-10-04)

Existing durable-chain reuse now validates the baseline's canonical
configuration and producer identities before delta persistence or live IR
mutation. A mismatch leaves both the durable sequence and the session version
unchanged. Matching epochs continue to append, while an explicitly persisted
compatible baseline establishes a replacement epoch rather than relabeling the
old chain.

Generated pending transitions now own the target compatibility identities
alongside their target hash and semantic edges. Apply-time validation,
missing-baseline materialization, and atomic semantic-snapshot persistence all
consume that captured evidence. This preserves acknowledgements after the
source file disappears and prevents different reads from assigning different
epochs to one accepted target. Canonical identities remain baseline-owned;
semantic identities remain version-local on each snapshot.

The canonical configuration and canonical producer rejection regressions each
failed against the ungated implementation by committing version 2, then passed
unchanged after the shared canonical validator was installed. An initially
invalid configuration fixture was discarded and restarted from RED after it
was found not to create a durable baseline.

Owner-reported results:

```powershell
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests::apply_delta_rejects_canonical_config_epoch_mismatch_without_advancing_chain -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests::apply_delta_rejects_canonical_producer_epoch_mismatch_without_advancing_chain -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::compatibility_epoch_tests -- --nocapture
cargo test --all-features mcp::sqlite_store::compatibility_persistence_tests::accepted_delta_keeps_canonical_epoch_and_writes_snapshot_local_semantic_identity -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests -- --nocapture
cargo test --all-features mcp::sqlite_store::compatibility_persistence_tests -- --nocapture
```

The positive epoch module, snapshot-local identity contract, complete delta
fidelity persistence authority, and complete compatibility persistence
authority were reported GREEN with no reported warnings. The existing
missing-source acknowledgement regression initially caught a post-generation
disk dependency and passed after identity ownership moved to the pending
transition.

### Phase 7 - Pending-edit recovery separation

**Purpose:** Preserve crash consistency without granting stale authority.

RED/GREEN slices:

- compatible target recovery commits and publishes as before;
- incompatible runtime B completes or rolls back the A durable transaction;
- A's stored identities remain attached after recovery;
- incompatible recovered state is not loaded into IRContext, semantic caches,
  WorkspaceIndex, or coverage;
- byte-exact existing recovery behavior remains protected.

Implementation boundary:

```text
filesystem reconciliation
  -> durable transaction completion under stored identity
  -> coherent durable result
  -> current compatibility validation
  -> optional live adoption
```

Exit criteria:

- recovery never relabels an old target with current identities;
- incompatibility cannot make durable recovery itself impossible.

#### Phase 7 implementation record (2026-10-04)

Pending-edit recovery continues to reconcile filesystem bytes and complete or
roll back the durable transaction using the identities stored on the intent.
Recovered state now passes through `validate_current_context` before any alias,
IRContext, semantic cache, WorkspaceIndex, fidelity, persisted-path, or
presentation-cache mutation. Incompatibility therefore leaves a coherent
durable artifact under its original epoch while skipping live adoption;
compatible targets retain the previous publication behavior.

The tracked RED regression recovered an A-epoch target under a B runtime. It
proved durable completion and preservation of all four A identities, then
failed because the old path created a live alias. The unchanged regression
passed after compatibility-gated hydration was installed. A complementary
positive test confirms compatible target hash, IR, semantic edges, fidelity,
and persisted-path publication.

Existing delta, replay, and semantic recovery fixtures were migrated from the
obsolete identity-less test helper to `establish_compatible_edit_intent`,
matching the Phase 2 production path. Their byte-level success, rollback, and
irreconcilable-recovery assertions were not weakened. Missing legacy identity
remains unproven and cannot be adopted.

Owner-reported results:

```powershell
cargo test --all-features mcp::tool_handlers::semantic_publication_recovery_tests::incompatible_recovered_target_completes_durably_without_live_publication -- --exact
cargo test --all-features mcp::tool_handlers::semantic_publication_recovery_tests::compatible_recovered_target_commits_and_publishes_with_stored_identity -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::edit_recovery_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::semantic_publication_recovery_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::persistence::replay_edit_recovery_tests -- --nocapture
```

All focused and existing recovery authorities were reported GREEN with no
reported warnings.

### Phase 8 - Compatibility-aware semantic publication and #116 completion

**Purpose:** Make coverage publication uniform only after compatibility is
established.

Deliverables:

- one publication primitive accepting `CompatibleSemanticProjection`;
- atomic replacement of semantic facts plus truthful source-hash/fidelity
  coverage;
- migration of:
  - `restore_context`;
  - `replay_history`;
  - pending-edit recovery when compatible;
  - mutating `apply_delta`;
- removal of direct uncovered `remove_file` plus `add_edges` adoption code;
- no compatibility policy inside WorkspaceIndex.

RED/GREEN slices:

- incompatible restored projection cannot establish coverage;
- compatible restore establishes coverage;
- compatible replay establishes only historical-hash coverage;
- compatible pending recovery establishes coverage;
- compatible mutating delta immediately establishes target coverage;
- the next equal-or-lower fidelity hydration does not redundantly compile;
- current-source divergence still forces transition to the current hash.

Exit criteria:

- every authoritative adoption path uses the same typed publication boundary;
- #116's contract gap is closed without weakening #117/#119.

#### Phase 8 implementation record (2026-10-04)

`McpState::publish_compatible_semantic_projection` is now the sole publication
boundary for compatible durable restore, historical replay, compatible
pending-edit recovery, and mutating delta acknowledgement. It accepts
`CompatibleSemanticProjection`, replaces the session semantic snapshot and
WorkspaceIndex facts together, and records source-hash/fidelity coverage only
when `SemanticFidelity::from_compilation` can certify it. Edit and Verbatim
remain deliberately uncertified, preserving the existing non-monotonicity
invariant.

Pending delta transitions now own a typed semantic projection captured from
the current compilation. Delta persistence, acknowledgement, semantic cache,
index facts, and coverage therefore consume one target hash, fidelity, and edge
set. Restore, replay, and recovery no longer contain their own direct
`remove_file` plus `add_edges` adoption sequences.

The tracked RED regression showed that a successful mutating delta installed
target edges without target coverage. It passed unchanged after the typed
publication boundary was integrated. Additional contracts prove that the next
equal-fidelity hydration compiles zero candidates, historical replay certifies
H1 but never disk H2, and a current-source query transitions coverage from H1
to H2.

Owner-reported results:

```powershell
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests::mutating_apply_delta_establishes_target_semantic_coverage_immediately -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests::covered_delta_target_skips_redundant_equal_fidelity_hydration -- --exact
cargo test --all-features mcp::tool_handlers::persistence::historical_replay_compatibility_tests::compatible_historical_replay_keeps_h1_identity_and_never_covers_h2 -- --exact
cargo test --all-features mcp::tool_handlers::core::delta::fidelity_persistence_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::persistence::historical_replay_compatibility_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::persistence::durable_semantic_restore_tests -- --nocapture
cargo test --all-features mcp::tool_handlers::semantic_publication_recovery_tests -- --nocapture
```

All focused regressions and affected publication authorities were reported
GREEN with no reported warnings. #116's implementation boundary is closed;
issue closure remains contingent on final migration verification.

### Phase 9 - Diagnostics, documentation, and harness decision

**Purpose:** Make the new rejection behavior operable and document durable
facts only after production integration exists.

Tasks:

- expose actionable structured mismatch components without leaking opaque
  implementation data;
- update durable architecture and invariants documentation;
- document producer-generation bump rules beside their ownership;
- inspect existing persistence/restart operator harnesses;
- extend one only if it can exercise a real production compatibility boundary
  without test-only seams;
- do not create a redundant harness for generation mismatch;
- keep any harness result separate from tracked test results.

Exit criteria:

- documentation matches the production lifecycle;
- legacy remediation is explicit: compile current source to establish a new
  baseline rather than blessing or deleting old history.

**Implementation record (2026-10-04):** Complete. Restore and historical
replay compatibility failures now expose a stable mismatch `reason` and a
coarser actionable `component` without exposing serialized identity payloads.
`PERSIST-002` records the durable compatibility lifecycle, legacy remediation,
typed publication boundary, and ownership model in
`docs/ARCHITECTURAL_INVARIANTS.md`. Producer-generation bump rules are
documented beside each producer owner.

No new operator harness was added. Existing tracked restart, replay, delta,
and recovery tests already exercise the production compatibility boundaries.
A generation-mismatch harness would require the same test-only generation
seams as those regressions and would not add live-system evidence, so it would
be redundant under the harness policy.

The repository owner reported these focused tracked tests GREEN with no
reported warnings:

```powershell
cargo test --all-features mcp::tool_handlers::core::restore::compatibility_tests::restore_rejects_canonical_type_alias_config_before_live_mutation -- --exact
cargo test --all-features mcp::tool_handlers::persistence::lifecycle_tests::semantics::historical_replay_rejects_incompatible_config_before_live_mutation -- --exact
```

### Phase 10 - Final integration audit and verification

Audit the complete lifecycle:

```text
producer
  -> identity derivation
  -> atomic persistence
  -> structural load
  -> compatibility validation
  -> live canonical adoption
  -> semantic publication
  -> coverage
  -> external MCP response/query
```

Confirm:

- no bypass path publishes unchecked durable state;
- no old and new adoption implementations coexist;
- unrelated producers do not invalidate compatible artifacts;
- compatible persistence reuse remains reachable;
- all modified/new Rust files satisfy the 615-line ceiling;
- obsolete comments, imports, helpers, and debt records are removed or updated;
- no critical/high finding remains in the final architectural audit.

The repository owner then runs the authoritative Final Verification Gate from
`docs/agent/verification.md`. This plan intentionally does not duplicate that
command list.

Exit criteria:

- all focused RED/GREEN results are recorded;
- the owner reports the complete final gate green;
- the production integration lifecycle is traced end to end;
- #117 and #119 can be closed;
- #116 can be closed only if Phase 8 is included and green;
- #118 remains independently open unless separately completed.

## 7. Planned regression placement

Prefer extending existing test authorities:

- `src/tests/mcp/durable_semantic_restore.rs` for restart, restore,
  compatibility, live-state immutability, and coverage;
- `src/tests/mcp/persistence_lifecycle_semantics.rs` for historical replay;
- `src/tests/mcp/delta_fidelity_persistence.rs` and existing delta sequence
  tests for chain epochs and apply behavior;
- `src/tests/mcp/delta_edit_recovery.rs` and
  `src/tests/mcp/replay_edit_recovery.rs` for recovery;
- SQLite-specific persistence tests for migrations and legacy rows;
- focused identity tests under the owning production module's existing
  external `src/tests/**` convention.

Create a new test file only when none of these authorities can express the
invariant cleanly. A new tracked file must still be registered through the
normal `#[path = "..."]` convention.

## 8. Phase checkpoint discipline

Each phase should end with:

1. targeted diff inspection;
2. active-file size inspection;
3. confirmation that only the intended authority boundary changed;
4. focused user-run tests for that phase;
5. a small checkpoint commit after reported GREEN;
6. an updated phase status in this document or the implementation work log.

Do not run or request the full repository gate after every phase. The full gate
is reserved for migration finalization.

## 9. Stop and approval boundaries

Stop for explicit architectural approval if implementation appears to require:

- changing the four-identity model;
- adding a fifth compatibility dimension;
- making identity executable-global;
- partial semantic snapshot reuse;
- changing query completeness or #118 lifecycle behavior;
- modifying Binary `0x04`;
- changing WorkspaceIndex's public contract beyond accepting compatible
  projection publication;
- deleting legacy persistence automatically;
- recompiling every restore;
- a delta chain that spans more than one canonical epoch.

Small implementation choices within this frozen architecture do not require a
new approval gate.

## 10. Definition of done

The migration is complete only when:

- all four identities exist with the frozen scope;
- identity/artifact writes are atomic;
- legacy absence is rejected, not guessed;
- ordinary adoption mutates nothing before validation;
- recovery preserves the original transaction epoch;
- every durable adoption path is gated;
- compatible reuse remains operational;
- #116 publication is unified behind compatibility evidence;
- all required regressions are tracked under `src/tests/**` with preserved
  RED/GREEN evidence;
- documentation reflects the actual production architecture;
- the final architectural audit finds no unresolved critical/high issue;
- the repository owner reports the complete Final Verification Gate green.
