# B8 representation findings and proposed next mission

Agent: **Agent MaxHeadRoom**, 2026-11-12. Investigation and proposal only.
The owner identified this as a primary architectural concern. No representation,
schema, search, traversal or retrieval-interface change is implemented by B8.

## Seven controlling findings

1. **Source sections exist, but the provider has no proposition-boundary contract.**
   `docs/invariants/canonical-ir.md` has separately named WIRE-001, VALID-001,
   ARCH-001, ARCH-002 and PIPELINE-001 sections, source provenance and historical
   section hashes. Those are source-owned human boundaries. The document parser's
   `Declaration` has no proposition selector, section identifier or content interval;
   `scan::scan` attaches the complete source to every declaration. Therefore the
   existing provider cannot establish a current, exact proposition boundary for
   every identity. It would be wrong to say that no source has meaningful sections,
   or that heading adjacency is already an admitted deterministic contract.
2. **SQLite stores the actual words, but does not store them as independent
   propositions.** `document_revision.body` is a complete source BLOB, not merely
   an external pointer. `declaration` stores identity/title/kind/status/track/note
   and typed relationships; `evidence` stores exact source/artifact associations.
   The words of propositions and rationales are in that BLOB. They have no
   independently identified content field, interval, rationale record or obligation
   row. Full-source persistence must not be described as normalized proposition
   storage, nor as a database containing only metadata and pointers.
3. **Whole-source matching produces proposition-level lexical false positives.**
   `document_search` independently indexes title, track and the same full source
   for every node. Four canonical-IR propositions match `PassPipeline` solely
   through the body shared with PIPELINE-001, not their own headed content. This
   establishes a mismatch between the indexed unit and proposition identity.
   Semantic relevance is a separate question: VALID-001's validation invariant
   remains relevant to the pipeline even without the literal word. The eight-node
   assessment below does not equate lexical mismatch with semantic irrelevance.
4. **Rust retrieves exact complete source, not exact bounded proposition content.**
   `store::query` / revision materialization can deterministically return a current
   identity, revision, entire source and evidence. No admitted selector lets it
   retrieve just that identity's complete proposition. The consumer must locate
   and interpret it in the source. That missing deterministic selection capability
   is a representation boundary problem. Engineering judgment about applicability
   and tradeoffs remains appropriate LLM work after selection is made deterministic.
5. **Decision structure is partial.** Kinds, statuses, optional verdict/note and
   typed edges exist. Reasons, alternatives, consequences and outstanding
   verification obligations remain prose. A named constraint node can be found
   and traversed, but its controlling statement is not a separate stored unit.
   PERF-001's rejection rationale, PERF-002's distinct DEFER/REJECT outcomes and
   DOC-206's earlier GREEN / later failure / awaiting-GREEN chronology cannot be
   inferred reliably from lifecycle status. See the decision assessment table.
6. **Exact authority transport can preserve shared artifacts without repeated
   text.** Explicit context `source_refs_v1` already pools exact sources within
   one response while retaining every node/revision, evidence association, typed
   relationship and witness. Independent reconstruction checks byte-equivalent
   inline payloads. Default inline still repeats full sources; search/get/related
   have no pooling format. Pooling resolves representation duplication, not
   proposition selection, applicability completeness or normalized rationale.
   Existing closure is complete only for its declared graph/traversal contract;
   it cannot establish that all semantically applicable undeclared knowledge exists.
7. **GLS shares the pointer/state representation limitation, not the same FTS
   implementation.** Its doctrine explicitly makes reports/evidence authoritative
   and nodes pointer plus state. `scripts/architecture_graph.py::ParsedBlock`
   records origin and declaration line, scalar metadata and edges, not proposition
   content boundaries or normalized rationale/obligation records. Its show/upstream/
   blocker/edge queries operate deterministically over those declarations. It has
   no corresponding SQLite whole-body FTS and per-result full-source materializer;
   therefore this particular repeated-source false-positive mechanism is not
   demonstrated in GLS. Importing its taxonomy did not supply independently
   queryable proposition content. Reference checkout HEAD:
   `491988992586f08984c523034a27f87e1140fec8`; inspected read-only.

## PassPipeline: match origin versus applicable knowledge

Exact identities, source hashes and materialization are retained in
`B8_PASSPIPELINE_MATCHES.csv`. The following assessments refer to the original
source sections, not invented database spans.

| Identity | Match origin | Applicable knowledge in its own source scope |
|---|---|---|
| PIPELINE-001 | Its own section contains PassPipeline | Direct: required six-pass order, dependency rationale, ordering witness and gate. |
| VALID-001 | Only another section of the shared file contains the term | Applicable validation constraint: E001–E011, validator authority and test requirement. Does not describe pass ordering itself. |
| ARCH-001 | Only another section of the shared file contains the term | Broader canonical-output constraint: inference state remains separate from serialized CompiledIR. No pipeline-specific ordering proposition. |
| ARCH-002 | Only another section of the shared file contains the term | Broader multi-language canonical IR boundary and conformance obligations. No pipeline-specific ordering proposition. |
| WIRE-001 | Only another section of the shared file contains the term | Downstream serialization/round-trip requirement with coverage caveats. No independent pass-order/dependency knowledge. |
| DOC-308 | Its own audit source contains the term | Direct historical production integration path, validation and checked publication boundaries; its historical audit standing is not current executable certification. |
| DOC-634 | Its own evidence source contains the term | Direct historical pattern-stage identity loss, surviving references, fail-closed validation, counterexample and tracked regression obligation. |
| PCG-175-DISCOVERY | Its own investigation contains the term | PCG/source-IR ownership and integration boundary; pipeline cited as source structural authority. Contextual architecture knowledge, not a new pass-order invariant. |

The four inherited matches are lexical proposition false positives. Some are
semantically useful constraints for a broader pipeline change; broad source
relevance does not prove that each proposition answers a specific pipeline query.
No distinct identity is invalidated or collapsed by this finding. The exact original
source and caveats remain necessary evidence.

## Proposed correction for separate authorization

Recommend **source-owned proposition content boundaries with deterministic
materialization**, rather than a response-size objective or LLM-authored summaries.

1. Define an explicit, revision-bound source selector for a proposition and its
   required rationale/caveats. Use source-owned stable delimiters or another
   unambiguous approved syntax; do not infer ownership from nearby headings.
   Permit source-owned supporting intervals where one contiguous section is
   insufficient. Retain full original source/evidence separately and exactly.
2. Specify whether existing identities denote whole documents or bounded
   propositions. Preserve all 218 identities and their history. Classify existing
   declarations explicitly; do not silently reinterpret historical evidence nodes
   or heuristically manufacture missing proposition spans.
3. Derive exact content and hashes in Rust, validate selector uniqueness, UTF-8
   boundaries and source currentness, and publish atomically with the generation.
   A changed/missing/ambiguous selector must fail explicitly, not return neighboring
   text, a stale summary or an apparently complete empty proposition.
4. Index the admitted proposition content independently in SQLite. Keep exact-ID
   lookup distinct from full-text discovery. Define discovery relevance and scope
   honestly; retrieving a source match must not assert an identity-local match.
   Rust must execute deterministic selection and construction, not ask an LLM to
   find each proposition in a large source after the database query.
5. Distinguish decision subject/outcome from lifecycle. Agree source-owned
   representation for rationale, rejected alternatives, constraints, consequences
   and verification obligations where independent queries need those concepts.
   Reference exact records/spans and typed relations. Do not promote source reports
   to executed verification or authenticated approval, and do not create an
   independently edited competing ledger.
6. Keep knowledge composition separate from transport. Resolve declared authority
   closure in Rust/SQLite; preserve all individual identities, typed meanings,
   witnesses and exact supporting source references. Existing response-local exact
   pooling demonstrates lossless artifact sharing. It cannot replace correct
   proposition boundaries or silently omit evidence needed to interpret a result.

An identity-only discovery view could help navigation but would leave the content
boundary problem unresolved. One-file-per-proposition authoring could provide
explicit boundaries using the existing whole-file model, but universal migration
would fragment shared history and still leave decision/obligation semantics
implicit. A new universal normalized ontology would be a much larger commitment
than the demonstrated need. Prefer a bounded source-selector and decision-query
design, with owner-approved contracts before schema or API implementation.

Acceptance for that mission should include the eight actual PassPipeline identities,
independent relevance oracles, shared-source and discontiguous caveat examples,
rejection rationale/counterexample retrieval, exact current source witnesses,
unchanged historical evidence, fail-closed selector drift, complete declared authority
contexts and production MCP reachability. Measure knowledge correctness and
deterministic queryability, not preferred response length or token reduction.

## Source authorities

- Clean-CTX `src/pcg/documents/{model,parse,scan,store,traversal,response}.rs`
  and `schema.sql`: current representation, indexing, selection and exact pooling.
- `docs/invariants/canonical-ir.md`: five independently named source sections and
  their exact controlling statements, provenance and verification caveats.
- `src/ir/pipeline.rs::PassPipeline::{default_production,run}`: actual ordered Rust
  execution, distinct from architectural knowledge retrieval.
- `docs/architecture/IR_PRODUCTION_INTEGRATION_AUDIT.md`,
  `docs/architecture/knowledge-normalization/evidence/consumptive-pattern-orphan.md`,
  `docs/agent/PCG_175_ARCHITECTURE_INVESTIGATION.md`: other three search sources.
- GLS `docs/ARCHITECTURE_EVIDENCE_GRAPH.md` and
  `scripts/architecture_graph.py::{ParsedBlock,Graph,load_graph,query_output}`:
  original pointer/state design and actual query responsibilities.
- `B8_INSPECTED_SOURCES.csv`, `B8_SQLITE_STORAGE_INVENTORY.csv` and
  `B8_QUERY_RECEIPTS.csv`: inspected identities, schema and baseline production traces.
