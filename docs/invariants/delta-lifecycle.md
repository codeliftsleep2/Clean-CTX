<!-- pcg:family-standing:begin -->
# Invariant family: delta-lifecycle

Current normative sections relocated from [the invariant catalog](../ARCHITECTURAL_INVARIANTS.md). Source attributes such as ENFORCED, STRUCTURAL, DOCUMENTED, RESOLVED and qualified enforcement retain their original meanings; GLS status is separate. Original section identities and witnesses remain unchanged except the explicitly documented C13 descriptive correction.

Graph status IMPLEMENTED records the source-described existing mechanism or operative governance contract, not a new successful witness execution. The DOCUMENTED mechanism-reuse rule remains INVESTIGATING in GLS: documentary convention is not promoted to proven machine enforcement. Sections without declarations remain current source contracts awaiting their assigned later migration batch.

<!-- pcg:family-standing:end -->
<!-- pcg:DELTA-001:begin -->
### DELTA-001 Delta Correctness

| Property | Value |
|----------|-------|
| **Intent** | Applying a computed delta between two `CompiledIR` states must reproduce the intended semantics of the target state. |
| **Invariant** | `DeltaComputer::compute(baseline, current)` produces a `Some(delta)` when the two IRs differ, and `None` when they are identical. The delta correctly identifies additions, modifications, and deletions. The delta preserves version chain (`from` / `to`). |
| **Enforcement** | Unit tests covering: add detection, removal detection, modification detection (renamed methods, changed types), identical-IR returns None, version chain correctness, JSON serialization with `+`/`~`/`-` keys, and edge cases (empty IRs, different files, duplicate keys). |
| **Authority** | `src/tests/ir/delta.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 62-74, original section `DELTA-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `732b74dc91122b6d2e865633cf8e2f4813aafa37b96af658d8cc659df61632bb`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:DELTA-001:end -->
<!-- pcg:IRDELTA-001:begin -->
### IRDELTA-001 Exact Occurrence-Preserving Delta Replay

| Property | Value |
|----------|-------|
| **Intent** | Incremental transport must reconstruct the canonical stream exactly; compactness never authorizes loss of repeated facts or semantic order. |
| **Invariant** | Corrected delta protocol `dv: 2` satisfies `replay(base, delta(base, target)) == target`. Each positional edit carries typed semantic identity and occurrence ordinal; removals and replacements also carry the expected existing tuple. Replay validates position, tuple, identity, and occurrence before committing transactionally. Production emits only corrected sequence deltas. Legacy `+ / ~ / -` input is compatibility-only and fails when a target is ambiguous. Delta protocol versioning is independent of CoreOp binary serialization. |
| **Enforcement** | `src/tests/ir/delta_sequence.rs` covers exact duplicates, same-target/different-payload facts, arbitrary positions, duplicate replacement, ordering, deterministic generation, compact round trip, conflict rollback, empty/equal algebra, and valid-stream property cases. `src/tests/mcp/delta_sequence.rs` covers real dispatched `delta_code_context` → corrected payload → `apply_delta`. |
| **Authority** | `src/ir/delta/sequence.rs`, `src/ir/replay.rs`, `src/ir/replay/sequence.rs`, `src/mcp/tool_handlers/core/delta.rs` |
| **Type** | STRUCTURAL + ENFORCED (test) |
| **Gate** | `cargo test` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 379-390, original section `IRDELTA-001`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `230076093bf584988b8b9222a0ffed3496c23cba1fa8398b7e330527b9803673`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: IRDELTA-001
kind: INVARIANT
status: IMPLEMENTED
title: IRDELTA-001 Exact Occurrence-Preserving Delta Replay
track: FOUNDATION
artifact: docs/invariants/delta-lifecycle.md
commit: 655f0a75
note: Source enforcement attributes are retained in prose; admission does not execute witnesses or establish independent validation.
knowledge_scope: invariant
knowledge_content:
  - statement self#IRDELTA-001
  - qualification self#family-standing
```

<!-- pcg:IRDELTA-001:end -->
<!-- pcg:IRDELTA-002:begin -->
### IRDELTA-002 Explicit Code-Side Delta Acknowledgement

| Property | Value |
|----------|-------|
| **Intent** | Delta generation must never silently mutate the canonical baseline, and acknowledgement must install exactly the server-authorized target state or install nothing. |
| **Invariant** | `delta_code_context` leaves live canonical IR and indexed semantics at the acknowledged baseline while retaining one server-owned pending transition for the exact file, version pair, target hash, delta identity, and complete target edge set. Repeating generation against the same source is deterministic and replaces that same pending key rather than accumulating transitions. `apply_delta` requires the expected live version and exact pending authority; when persistence is enabled, it durably commits the canonical delta plus semantic snapshot before replacing live IR and indexed edges, then consumes only the matching transition. A rejected delta is never partially installed: state and pending authority remain coherent, while preflight may first complete a separately durable edit intent. Delta operations remain code-side auxiliary data; the repository ships the explicit server tools but no automatic production host consumer. |
| **Enforcement** | `src/tests/mcp/durable_semantic_restore.rs` crosses dispatched generation, deterministic repeat generation, forged-transition rejection, mutation coherence, exact acknowledgement, durable restart/restore, index replacement, pending consumption, and replay rejection. `src/tests/mcp/delta_edit_recovery.rs` covers recovery ordering and recovery-failure atomicity. `src/tests/mcp/content_kind_lifecycle.rs` enforces the model-visible boundary. |
| **Authority** | `src/mcp/durable_semantics.rs`, `src/mcp/tool_handlers/core/delta.rs`, `src/mcp/tool_handlers/core/delta_apply.rs`, `src/mcp/tool_handlers/core/delta/persistence.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 391-402, original section `IRDELTA-002`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `27d1bf8277509ae778143e894334c69aaff2cdc93c8d4f50306309b022019e0d`. Original text remains in Git history; this file is the sole current normative owner of this section.

<!-- pcg:IRDELTA-002:end -->
<!-- pcg:IRDELTA-003:begin -->
### IRDELTA-003 Delta Statistics Mirror the Successful Response Lifecycle

| Property | Value |
|----------|-------|
| **Intent** | The read-only statistics dashboard must describe what a successful delta-capable handler actually returned; observability must not invent, omit, or double-count transport events. |
| **Invariant** | Every successful `delta_code_context` response records exactly one session event using its resolved fidelity, Angular classification, selected token counts, and actual strategy. Initial, cached, and no-difference complete responses are `full`; a non-empty generated sequence delta is `delta` and retains the prior full-compression token baseline for efficiency accounting. Every `provide_code_context` response is independently recorded once as `full`. `context_stats` remains a read-only projection of this already-recorded state. |
| **Enforcement** | `src/tests/mcp/delta_stats_lifecycle.rs` crosses registered dispatch for the dedicated full-baseline → external-change → generated-delta lifecycle and repeated complete provider reads. The regressions assert strategy, fidelity, Angular status, non-zero tokens, and per-file/session delta counts. |
| **Authority** | `src/mcp/tool_handlers/core/delta.rs`, `src/mcp/tool_handlers/core/provide/evaluate.rs`, `src/mcp/session_stats.rs`, `src/mcp/tool_handlers/stats/mod.rs` |
| **Type** | ENFORCED (test) |
| **Gate** | `cargo test --all-features` |
| **Relationship to IRDELTA-002** | IRDELTA-002 governs pending-transition authority and acknowledgement. IRDELTA-003 governs truthful observation of generation and complete-response events; recording statistics never applies or acknowledges a delta. |

---

Source provenance: commit `655f0a75`, `docs/ARCHITECTURAL_INVARIANTS.md` lines 403-415, original section `IRDELTA-003`; source SHA-256 `3af6ad63ff76c246119799832820ca60002153172c317be0b3072519f8fa9d9e`; section SHA-256 `489cc197e50ae85f7774df927fe98c73d360dee6de3a0cfee22629ebe77770b7`. Original text remains in Git history; this file is the sole current normative owner of this section.

```architecture-graph
node: DELTA-001
kind: INVARIANT
status: IMPLEMENTED
title: DELTA-001 Delta Correctness
track: REPOSITORY
artifact: docs/invariants/delta-lifecycle.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#DELTA-001
  - qualification self#family-standing
```

```architecture-graph
node: IRDELTA-002
kind: INVARIANT
status: IMPLEMENTED
title: IRDELTA-002 Explicit Code-Side Delta Acknowledgement
track: REPOSITORY
artifact: docs/invariants/delta-lifecycle.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#IRDELTA-002
  - qualification self#family-standing
```

```architecture-graph
node: IRDELTA-003
kind: INVARIANT
status: IMPLEMENTED
title: IRDELTA-003 Delta Statistics Mirror the Successful Response Lifecycle
track: REPOSITORY
artifact: docs/invariants/delta-lifecycle.md
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: invariant
knowledge_content:
  - statement self#IRDELTA-003
  - qualification self#family-standing
```
<!-- pcg:IRDELTA-003:end -->
