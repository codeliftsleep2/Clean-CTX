# Scoped indexed cycle witnesses

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

## Implementation outcome (2026-09-27)

- `kind` is optional, defaults to `dependency`, and rejects unknown values.
- Eligible relations are exactly `Injects` and `ImportsModule`.
- `WorkspaceIndex` owns one iterative deterministic closed-witness primitive;
  the compatible boolean delegates to it.
- Query-local outgoing-edge ordering makes equivalent insertion histories
  select the same witness. Full cost includes node/edge sorting, not only DFS.
- Witness steps retain asserting-file provenance. Semantic tuple collisions are
  explicitly listed with their admitted occurrence files.
- MCP structured and model-facing responses report
  `indexed_evidence_only` and `source_complete=false`.
- The operation performs no discovery, hydration, source compilation,
  persistence, rendered-context publication, or compression-statistics write.
- Tracked authorities are `src/tests/workspace/index_cycle_policy.rs` and
  `src/tests/mcp/workspace_query_cycle_witness.rs`, with existing WSC-004 suites
  retaining scope-security authority.

Phase 0 findings and the four decisions awaiting approval are recorded in
`HAS_CYCLE_PHASE0_SEMANTIC_AUDIT.md`.


Source: [docs/architecture/HAS_CYCLE_WITNESS_PROPOSAL.md](../HAS_CYCLE_WITNESS_PROPOSAL.md); original section SHA-256 `289e6948c9308f702a66f078e703a4636b988b1f50e7a790296a1af808f6c1a1`. Original wording and tense are retained; admission is limited to the implemented scope described here.


The proposal-era findings below the original outcome are historical pre-migration observations, not the current response contract. No exhaustive source mode is admitted.

```architecture-graph
node: AUTH.CYCLE_WITNESS
kind: AUTHORITY
status: IMPLEMENTED
title: Scoped indexed cycle witnesses
track: REPOSITORY
artifact: docs/architecture/contracts/cycle-witness.md
preserves:
  - WSC-007
  - WSC-004
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
