# IR Architecture Certification

**Status:** Phase 9 certified; Phase 10 release-candidate documentation.

**Release:** `0.8.0-rc`

**Certified:** 2026-09-21

Final `0.8.0` remains withheld until live field testing is complete. This
record certifies repository and registered production-path architecture; it
does not relabel a release candidate as field-proven.

## 1. Certified production lifecycle

Every registered IR/MCP semantic path was traced through:

```text
registered dispatch
  -> default compilation or checked canonical input
  -> validation and transformations
  -> canonical session ownership
  -> file-scoped durable ownership
  -> replacement, replay, edit, deletion, and recovery
  -> checked consumer projection
  -> registered response
```

The Phase 9 audit records findings P9-01 through P9-27 and their tracked
production evidence. The user reported the repository gate and operator
production scenarios green before Phase 10 began.

## 2. Current semantic boundaries

- Canonical IR uses typed class, interface, method, field, and parameter
  identity. Class and interface member ownership are distinct.
- Validation is exhaustive over current `CoreOp` variants. Checked projection
  reports unresolved and wrong-kind identity rather than dropping facts.
- Hierarchical structure protects ownership, cardinality, and ordering. The
  compact LLM renderer is a separate minimized projection.
- Full `Body` text and absolute byte spans remain authoritative edit-fidelity
  artifacts. Unsafe pattern compression declines rather than orphaning them.
- Physical binary `0x04` round-trips complete canonical semantics exactly.
  Lossy `0x01`-`0x03` inputs do not fabricate missing identity.
- `dv:2` is the production occurrence-aware positional delta protocol.
  Durable application requires the server-owned target semantic-edge snapshot.

## 3. Durable ownership and transactions

Canonical `0x04`, source hash, fidelity, semantic version, and the complete
semantic-edge snapshot form one logical durable state. Durable commit precedes
live publication for persistence-enabled baseline and delta operations.

Pending edit-intent recovery is a shared file-scoped preflight for every path
that can load, compile, publish, checkpoint, or hydrate semantic state.
`apply_edit` is byte-exact and crash-recoverable: it validates exact prior
source identity, compiles the exact candidate bytes, stages recovery, replaces
the source atomically, commits aligned semantics, then publishes live state.

Reads (`context_stats`, `context_history`, persisted listings, and storage read
helpers) are observational. Purge and deletion mutate only their explicit
file/history ownership. No registered semantic lifecycle relies on a global
buffer flush or opportunistic fallback import.

## 4. Compatibility and quarantine

- Older supported hierarchical revisions remain decode-only compatibility
  inputs and are never reinterpreted as newer semantic kinds without evidence.
- Legacy delta input remains compatibility-only where structurally
  unambiguous; production emits `dv:2`.
- Incomplete legacy fallback artifacts are quarantine/inspection-only.
  `inspect_legacy_fallbacks` reports them through registered MCP without
  importing, deleting, rewriting, or mutating semantic state.

## 5. Explicitly deferred decisions

The following are not hidden gaps in the certified lifecycle:

- a future class-scoped family for injectable/DI participation metadata;
- a future portable delta contract carrying externally supplied authoritative
  edge snapshots;
- a future complete versioned fallback protocol, if production need is proven;
- later token-efficiency review of full-body LLM output, without weakening
  byte-exact edit fidelity or full-body fallback.

Each requires a separate architecture decision. None may be inferred from the
`0.8.0-rc` certification.

## 6. Evidence and authority

- [`IR_PRODUCTION_INTEGRATION_AUDIT.md`](IR_PRODUCTION_INTEGRATION_AUDIT.md)
- [`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION.md)
- [`IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION_2.md`](IR_PRODUCTION_INTEGRATION_AUDIT_CONTINUATION_2.md)
- [`CORE_OP_CONTRACT_MATRIX.md`](CORE_OP_CONTRACT_MATRIX.md)
- [`BINARY_V04_CONTRACT.md`](BINARY_V04_CONTRACT.md)
- [`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md)

The original reconnaissance report remains a historical baseline rather than
a current-state dashboard.

## 7. Legacy documentation classification

`docs/DEVELOPER_DOCUMENTATION.md` remains an oversized legacy contributor
guide whose persistence and Compiler IR sections describe the pre-`0.8.0-rc`
architecture (buffered fire-and-forget writes, reset/recompile restore, legacy
delta, and incomplete fallback recovery). Those sections are historical and
non-authoritative for semantic lifecycle behavior. This certification, the
operation matrix, binary contract, living architecture overview, and tooling
guide are the current authorities. A future rewrite of the contributor guide
must decompose it under the active-file ceiling rather than patching isolated
claims into an internally inconsistent document.
