# B9 actual engineering-agent workflow

Agent: **Agent MaxHeadRoom**. Use the registered `pcg_knowledge_v1` tool with an
explicit actual workspaceRoot. Normal instructions are in `docs/agent/pcg-authoring.md`.

1. Search with an exact FTS expression and optional kind/status/track/role/label.
   Inspect coverage/currentness/completeness and match origins. A direct lexical
   match is distinct from a related constraint. Discovery returns acquisition refs.
2. Follow the returned get request for exact ordered statement, rationale and
   qualifications. PIPELINE-001 is retrieved independently of sibling invariants.
3. For a specific historical question, get a declared role/label. For example use
   PERF-001-REPLAY role=counterexample label=registered_delete; DOC-206
   role=observation label=live_639_644. Required qualifications accompany selections.
4. Use related with a typed edgeKind/direction to inspect evidence, supersession,
   preserved invariants or incoming dependents. Rust/SQLite resolve endpoints.
5. Use context to retrieve complete declared authority with exact units, all typed
   relationships and witnesses. Do not reconstruct closure manually or drop a premise.
6. Follow a source_request to inspect the complete exact original artifact. The
   returned hash pin refuses on drift; reacquire current knowledge before relying
   on a changed source. Original evidence is not replaced by extracted components.
7. Distinguish requested-component completeness from unit_complete and authority
   closure. Blocked/incomplete results are not current complete knowledge.

This agent reads actual production results and applies engineering judgment to
their content. It does not run an uncontaminated independent A/B evaluation.
The tracked 270-request production set covers every identity and the mandatory
historical witnesses. Follow-on returned-reference acquisition demonstrates the
first-class interface; successful serialization alone does not establish semantics.
The current host does not advertise the conditional knowledge tool. The authorized
stdio wrapper reaches the real production registry against the actual root; it
does not certify host deployment, general adoption or independent decision quality.
