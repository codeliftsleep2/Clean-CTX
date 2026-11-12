# B8 findings and alternatives

Agent: **Agent MaxHeadRoom**. Observed implementation, not proposed architecture.

The owner has identified independent proposition representation and deterministic
retrieval as a primary architectural concern. See
`B8_REPRESENTATION_AND_RETRIEVAL_PROPOSAL.md` for explicit source-owned boundary
findings and the proposed correction. B8's successful removal of a token refusal
cannot by itself satisfy that architectural objective.

1. Document catalog is pointer/state/edge plus exact whole-source history, not a
   normalized decision/rationale/obligation database. This matches explicit GLS
   doctrine and document-graph design. Richer original enrolled records are distinct.
2. Search and related couple identity discovery to complete endpoint source/evidence
   acquisition. PassPipeline duplicates one source five times; whole-source FTS may
   select identities whose own proposition is not shown relevant. This is a demonstrated
   discoverability/interface limitation, not canonical identity corruption.
3. Rust/SQLite already compute declared joins, closure, witnesses, revisions, freshness
   and response composition. No evidence LLM must reproduce SQL or execute passes.
   Semantic applicability, rationale, alternatives and historical verification standing
   still require reading source. Source_refs requires a correct consumer for exact
   legacy reconstruction; host support is not equivalent to semantic reasoning.
4. Decision status alone cannot answer why replay reuse was rejected or whether
   #156's later repair passed. Those facts are preserved in exact source, not normalized
   independent fields. Structured graph answers are bounded by declared edges and
   fields, not all architectural knowledge or inference of missing obligations.

## Proposed options requiring separate authorization

- Clarify existing search/get/related/context responsibilities and consumer behavior.
  Lowest contract cost, but does not remove source-level relevance or full-source
  discovery coupling. Verify realistic discovery and exact evidence reachability.
- Add an explicit identity/edge discovery view while preserving full-source acquisition.
  Justified by discovery failures/coupling, not token savings. Requires API compatibility,
  relevance/completeness semantics, currentness and deterministic registered-path tests.
  Do not silently change existing operation payload or hide complete original evidence.
- Author explicit proposition/decision source boundaries and query their exact spans.
  Could support independent reasoning units without abandoning Markdown authority.
  Existing grammar lacks those spans/rationale fields, so requires source syntax,
  validation, revision/evidence and migration design. Do not infer sections heuristically
  or create a second source of truth. Wrong boundaries can omit decisive caveats.
- Expose existing deterministic edge queries more effectively. Explicit required/
  informative meaning must remain; missing relevance/applicability cannot be solved
  by adding topic-only edges or artificial reverse authority.
- Retain present architecture if complete-document evidence and manual applicability
  suit actual usage. Larger output is not itself a defect; do not pursue a new schema
  absent demonstrated decision/discovery value and proportionate maintenance benefit.

Each option needs owner disposition, compatibility scope and tracked correctness/
freshness/history/atomic-reconciliation tests plus actual production reachability.
None is approved by this investigation. Original B8 document-budget removal is the
only production change. Database/schema/query/extraction/transport/FTS/pass changes,
independent agent evaluation and integration remain excluded.
