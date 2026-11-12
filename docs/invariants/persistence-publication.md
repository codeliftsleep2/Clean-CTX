# Invariant family: persistence-publication

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

### PERSIST-001 Automatic Checkpoints Are Policy; Accepted Mutations Are Durable Authority

| Property | Value |
|----------|-------|
| **Intent** | Operators must be able to keep ordinary context reads session-only without weakening crash recovery, edit safety, or explicitly requested persistence. |
| **Invariant** | `persistence.enabled = false` creates no durable store. With persistence enabled, `auto_save = true` checkpoints canonical read-produced baselines before publishing their live owners; `auto_save = false` leaves ordinary `provide_code_context`, `compress_code_context`, and read-only delta-generation results session-only until `save_context`. Edit-fidelity baselines remain durable because they establish safe edit authority. Accepted `apply_edit` and `apply_delta` transitions, explicit saves, and explicit deletions remain durable regardless of `auto_save`. Verbatim and Angular-template presentations are never checkpointed because they own no canonical IR. Every required checkpoint atomically aligns canonical IR, semantic edges, source hash, version, fidelity, and compatibility identities before live publication. The complete payload is captured from one owner authority epoch while holding that owner's guard; persistence I/O may follow after capture and does not require a global authority lock. |
| **Enforcement** | `src/mcp/tool_handlers/core/provide_persistence.rs` owns read-checkpoint policy; canonical producers consult it before live publication. Mutation handlers retain their independent transactional persistence boundaries. `src/tests/mcp/auto_save_contract.rs` covers disabled persistence, manual session-only reads, every canonical automatic producer, explicit save/restart/restore, and mandatory Edit authority. `src/tests/mcp/delta_fidelity_persistence.rs` covers accepted delta durability with auto-save disabled. Existing baseline-publication and edit-transaction regressions enforce failure atomicity. |
| **Authority** | `src/config.rs` (`PersistenceConfig`), `src/mcp/tool_handlers/core/{provide_persistence,provide,compress,delta,delta_apply}.rs`, `src/mcp/tool_handlers/edit.rs`, `src/mcp/tool_handlers/persistence/mod.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 675-687, original section `PERSIST-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `461e1a6757a02318e8fd0708a32a64a4858a54faa4fa945a45a572fd795a717c`. Original text remains in Git history; this file is the sole current normative owner of this section.

### PERSIST-002 Durable Authority Requires Scoped Compatibility Evidence

| Property | Value |
|----------|-------|
| **Intent** | Structurally decodable durable state must never become live canonical or semantic authority under a runtime that would have produced different facts. |
| **Invariant** | Every new durable baseline owns separate canonical configuration and producer identities; every semantic snapshot owns separate semantic configuration and producer identities. Identities contain only applicable configuration and producer generations, and producer sets compare exactly in both directions. Structural decoding, intrinsic semantic validation (`Calls` iff call evidence exists), current-source/fidelity validation, canonical compatibility, and semantic compatibility precede their respective live adoption/publication boundaries. Canonical IR validity and derived-semantic validity are separate: a semantic snapshot records every external source actually consulted by its producer, and reuse or adoption requires each recorded source identity/hash state to remain current; missing, changed, or restored dependencies force current-source semantic derivation while relationship ownership remains with the asserting source. Ordinary restore, historical replay, delta-chain reuse, and recovered-edit publication validate the relevant identities before mutating live authority. One delta chain retains one canonical epoch. Recovery may finish an older transaction under its stored identities, but incompatibility prevents live adoption and never relabels the recovered artifact. Missing legacy identity is unproven and rejected for adoption; remediation is a current-source compilation that establishes a new compatible baseline, never automatic blessing or deletion. Compatible reuse remains supported. |
| **Publication** | `CompatibleCanonicalState` and `CompatibleSemanticProjection` are the typed evidence boundary. Compatible restore, replay, recovery, and mutating delta publication share `McpState::publish_compatible_semantic_projection`; coverage is recorded only for the projection's own source hash and a certifiable compilation fidelity. WorkspaceIndex stores coverage but owns no compatibility policy. |
| **Producer generations** | Each producing module owns its generation constant and increments it only when identical relevant inputs can yield different durable canonical facts, markers, or semantic edges. Configuration changes are represented by configuration identity instead. Refactors with identical durable output do not bump generations, and unrelated producers are never bumped together. |
| **Diagnostics** | Rejections expose a stable structured `reason` and affected `component`; human-readable details may describe source hashes or fidelity but never substitute for the structured classification. |
| **Enforcement** | Identity determinism and scoping: `src/tests/mcp/durable_compatibility_identity.rs`. Atomic persistence and legacy absence: `durable_compatibility_persistence.rs`. Pure validation: `durable_compatibility_validation.rs`. Production restore/replay/delta/recovery/publication: `durable_restore_compatibility.rs`, `historical_replay_compatibility.rs`, `delta_compatibility_epoch.rs`, `delta_fidelity_persistence.rs`, `semantic_publication_recovery.rs`, `delta_edit_recovery.rs`, and `replay_edit_recovery.rs`. |
| **Authority** | `src/mcp/compatibility/{identity,producer,validator}.rs`; `src/mcp/sqlite_store/{compatibility_schema,semantic_state,edit_intent}.rs`; `src/mcp/durable_semantics.rs`; restore, replay, delta, and edit handlers under `src/mcp/tool_handlers/` |
| **Type** | STRUCTURAL + ENFORCED (test) |
| **Gate** | Focused compatibility authorities plus the final `cargo test --workspace --all-targets --all-features` and zero-warning Clippy gate |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 688-703, original section `PERSIST-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `cbfc877d512d21df475ba3d4142579ca8caf8bde89f5068450c9a1ca8854cacd`. Original text remains in Git history; this file is the sole current normative owner of this section.

### PUB-001 Same-Owner Semantic Publication Is Source-Ordered

| Property | Value |
|----------|-------|
| **Intent** | Parallel compilation, edits, and historical adoption may finish out of order, but completion order must never let an older same-owner authority operation replace authority already committed by a newer operation. Historical snapshot age and operation age remain distinct: a restore/replay genuinely begun later may deliberately establish compatible historical state. |
| **Invariant** | Every authoritative `provide_code_context`, `delta_code_context`, `apply_edit`, hydration, restore, and replay operation receives a generation from its canonical semantic owner before its unlocked candidate-selection work. Compilation candidates validate their compiled source hash again at the commit boundary. An edit additionally compares the physical source bytes with its exact validated precondition while holding the owner commit boundary; a superseded generation or byte mismatch occurs before staging, durable intent, replacement, or live publication. Historical adoption validates durable compatibility under its established rules, while its ticket represents the restore/replay operation's ordering position rather than the selected snapshot's age. Delta generation additionally binds its captured version, canonical instructions, and source hash to that same authority epoch before it may return a transition. Commit is serialized only for that owner. A candidate whose source or captured baseline is no longer current, or whose operation generation is older than an already committed generation, publishes no canonical IR, fidelity, semantic-edge snapshot, WorkspaceIndex projection, rendered cache, pending delta transition, durable checkpoint, or physical edit. Different semantic owners remain independently compilable and publishable. |
| **Enforcement** | `src/mcp/state/publication.rs`; `src/mcp/tool_handlers/edit.rs`; `src/mcp/tool_handlers/core/provide/evaluate.rs`; `src/mcp/tool_handlers/core/{delta,restore}.rs`; `src/mcp/tool_handlers/hydration/publication.rs`; `src/mcp/tool_handlers/persistence/replay.rs`; deterministic controlled-interleaving regressions `src/tests/mcp/provide_publication_race.rs`, `src/tests/mcp/delta_publication_race.rs`, `src/tests/mcp/historical_adoption_publication_race.rs`, and `src/tests/mcp/apply_edit_transaction.rs`; later historical replay authority coverage in `src/tests/mcp/historical_replay_compatibility.rs`. |
| **Authority** | `McpState::begin_semantic_publication`; `SemanticPublicationTicket::commit` |
| **Type** | ENFORCED (file-scoped operation generation, candidate preconditions, and tests) |
| **Gate** | `cargo test --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 921-933, original section `PUB-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `7b66271a98cd1ae8622eea43b200454c9bf198fc180b65e3981fbc54ae1a1192`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: PERSIST-001
kind: INVARIANT
status: IMPLEMENTED
title: PERSIST-001 Automatic Checkpoints Are Policy; Accepted Mutations Are Durable Authority
track: REPOSITORY
artifact: docs/invariants/persistence-publication.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```

```architecture-graph
node: PERSIST-002
kind: INVARIANT
status: IMPLEMENTED
title: PERSIST-002 Durable Authority Requires Scoped Compatibility Evidence
track: REPOSITORY
artifact: docs/invariants/persistence-publication.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```

```architecture-graph
node: PUB-001
kind: INVARIANT
status: IMPLEMENTED
title: PUB-001 Same-Owner Semantic Publication Is Source-Ordered
track: REPOSITORY
artifact: docs/invariants/persistence-publication.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
