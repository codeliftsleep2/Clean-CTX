# Durable authority compatibility

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

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


Source: [docs/architecture/DURABLE_AUTHORITY_COMPATIBILITY_MIGRATION_PLAN_2026-10-03.md](../DURABLE_AUTHORITY_COMPATIBILITY_MIGRATION_PLAN_2026-10-03.md); original section SHA-256 `a18c6bd0531556c03acbd7ffef3118033c8656c772ae72bb2c9f00844bfd038d`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.DURABLE_COMPATIBILITY
kind: AUTHORITY
status: IMPLEMENTED
title: Durable authority compatibility
track: REPOSITORY
artifact: docs/architecture/contracts/durable-compatibility.md
preserves:
  - PERSIST-002
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
