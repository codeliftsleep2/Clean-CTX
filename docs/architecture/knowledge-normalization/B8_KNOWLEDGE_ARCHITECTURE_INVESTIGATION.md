# B8 architectural knowledge storage and execution boundaries

Agent: **Agent MaxHeadRoom**, 2026-11-12, America/New_York.
Baseline dcb60495b80335c33c8d27607ff3ba1886968a9b, PR #186 OPEN; B8 branch
feat/pcg-b8-document-budget. Unrelated untracked reports remain untouched.

## Finding

This is a substantive representation and retrieval limitation for the owner's
objective of independently queryable architectural knowledge. Whole-document FTS
and full-source materialization use a different content boundary from the named
proposition identities. Removing token refusal does not correct that mismatch.
The seven explicit boundary findings, all eight PassPipeline relevance assessments,
GLS comparison and source-supported next-mission proposal are in
`B8_REPRESENTATION_AND_RETRIEVAL_PROPOSAL.md`.

The active pcg_documents_v1 implementation is **Architecture B: a source-owned
document graph and retrieval system**, with structured identity, lifecycle and typed
relationship metadata. In a broad sense it is hybrid metadata plus prose, but it
is not Architecture A's normalized independently queryable rationale/alternatives/
verification-obligation system. The separate pcg_query_v1 owner-enrolled evidence
interface is richer: source/evidence byte spans, claim/decision/constraint/evidence
roles, attribution, execution/verification states, subject/disposition, selectors and
bindings. It is neither the same catalog nor automatic proposition extraction.

SQLite executes FTS, joins, foreign-key consistency and adjacency queries.
Rust parses, validates, reconciles, stores history, hashes, traverses, builds witnesses,
checks source observations and serializes results. There is no finding that the
LLM executes PassPipeline, SQL joins or the provider's deterministic closure algorithm.
The LLM must read source prose to understand rationale and assess applicability.
That semantic reasoning is different from deterministic result reconstruction.

## What is stored and retrieved

document_revision stores revision, id, path, source_sha, declarationJSON, complete body
BLOB, evidenceJSON, and previous revision. Candidate revision hashes path/source/
declaration/evidence, not an extracted proposition section. document_member binds
generation and stable identity to revision. document_edge stores typed endpoints
plus source-path/hash provenance. document_search repeats complete source text
for every declaration, with title and track. Source/title metadata is not a full
decision or complete independently bounded proposition. No rationale/alternatives/
obligation/source-section-span columns exist in this catalog. Exact original source
is stored, not merely an external path. B8_SQLITE_STORAGE_INVENTORY.csv includes
all actual catalog objects, columns/PKs/FKs, indexes and FTS shadow structures, and a
separate in-memory inventory of the enrolled schema without opening its private data.
There are no document schema migrations:store::open compares exact sqlite_master
metadata against version 1 reference schema and refuses incompatibility.

Search/get/related select identities differently but share full revision materialization.
Context adds deterministic graph closure and witnesses, then returns complete sources.
Search has no source_refs mode, no snippets, no ranking, no kind/status filter, no
proposition-local term boundary. Track is filtered by Rust after revision acquisition.
The interface is therefore not a lightweight discover-then-acquire separation.

## Exact PassPipeline witness

B7 search returned 8 identities / 44, 621 tokens. Five nodes ARCH-001, ARCH-002, PIPELINE-001,
VALID-001, WIRE-001 have the same canonical-ir.md path/hash and each return its complete
10, 253-byte source. Four duplicate copies contribute 41, 012 raw UTF-8 bytes before JSON
escaping; distinct identities remain valid and must not be collapsed. The other
sources are the historical production audit, DOC-634, and PCG discovery report. FTS
matches full source bytes for each node; it does not establish that each node's own
proposition contains or is independently relevant to PassPipeline. Exact node-title/
note checks and source hashes are in B8_PASSPIPELINE_MATCHES.csv. No proposition
section boundaries are stored to make that relevance test deterministic today.

## Architectural intent and GLS comparison

The original enrolled design explicitly chose owner-enrolled exact evidence and
attribution, not a universal semantic reasoner. Later source-document graph design
explicitly chose GLS-compatible source declarations, complete-source context and
separate enrolled semantics. The implementation matches that bounded document
lifecycle intent; normalization's practical decision-intelligence goals do not
retroactively create structured rationale fields or automatic applicability.

Actual GLS doctrine docs/ARCHITECTURE_EVIDENCE_GRAPH.md says a node is a pointer plus
state, and reports/production/evidence are authoritative. Its Python Graph/load_graph/
validate/query_output implements Markdown parsing, edges/status/blocker/upstream
queries and derived YAML/Markdown indexes, not this SQLite full-source FTS/materializer.
Clean-CTX inherits taxonomy/source ownership, adapts persistence/query-boundary history/
freshness/transport, and does not authenticate actors or reclosure. GLS does not prove
Architecture A either. Reference paths were read in GoalLine Stand 2026; no file there
was modified. Clean-CTX investigation identity remains Agent MaxHeadRoom.

## Scope and evidence

Production preflight uses the accepted old b8178948... document binary and unchanged
actual-root profile. Historical B7 responses are exact baseline witnesses; new source
changes do not make that executable implement B8. Trace captures label binary and
generation explicitly. Read-only catalog SQL confirms actual stored values and FTS
matches; it is not a fabricated complete production response for refused contexts.
No knowledge schema, FTS, traversal, graph authority or PassPipeline change was made.
Only the separately authorized document token-enforcement removal is implemented.
The all-feature B8 executable was built and verified through 33 actual stdio
requests. All ten exact searches and all six required contexts are now available;
four complete format pairs reconstruct exactly. The separate capture checks verify
current source bytes, graph membership, typed edges and witnesses. See
`B8_VERIFICATION.md` for scoped results and the outstanding full repository gate.

Newly available material confirms the same representation boundary: search
`replay candidate` returns 48 full-source node records, `provider completeness`
67, and `canonical occurrence` 99. These are source-level lexical matches, not
proof of 48, 67 or 99 independently relevant propositions. PassPipeline still returns
the same eight identities and five copies of canonical-ir.md. Context source pooling
preserves all exact evidence without repeating shared source text in the pool,
but does not supply proposition boundaries or normalized rationale/obligations.
Before/after composition is recorded in `B8_PAYLOAD_COMPOSITION.csv` and
`B8_POST_REMOVAL_PAYLOAD_COMPOSITION.csv`; actual selected identities and complete
relationships/witnesses are retained in `B8_POST_REMOVAL_QUERY_RECEIPTS.csv`.
Successful unbudgeted retrieval does not establish that this query architecture
meets the owner's independently queryable architectural knowledge objective.
