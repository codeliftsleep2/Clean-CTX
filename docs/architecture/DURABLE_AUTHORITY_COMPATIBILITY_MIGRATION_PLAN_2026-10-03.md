# Durable Authority Compatibility Migration Plan

**Status:** Approved architecture, implementation not started  
**Branch:** `feature/durable-authority-compatibility`  
**Issues:** #117, #119, prerequisite and integration work for #116  
**Out of scope:** #118, Binary `0x04` redesign, relation-family persistence,
whole-configuration or whole-executable fingerprints

**Phase status (2026-10-03):** Phases 0 and 1 complete; Phase 1 focused
structural contract suite reported GREEN by the repository owner

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
