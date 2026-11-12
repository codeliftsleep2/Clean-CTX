# B8 FTS semantics

store::publish inserts(id, title, body, track)per node using the complete original
source bytes. schema.sql defines fts5(id UNINDEXED, title, body, track). No explicit
kind/status index, section boundary, snippet, bm25/relevance ordering or relationship-aware
predicate is exposed. Exact get uses stable ID, not this FTS expression.

PassPipeline's five shared-source identities each index the same body. A matching
word in PIPELINE-001's section can select WIRE-001 as well. This is observed shared
source matching, not proof of relevance to each independent proposition. Ordering is
stable ID ORDER BY, not relevance rank. Documentation promises SQLite FTS5 and complete
source retrieval; it does not promise proposition-local semantic search. This is an
intentional complete-document indexing/materialization contract with a demonstrated
proposition-relevance limitation, not evidence that SQLite failed to filter.

Manual inspection of the five complete headed source sections finds PassPipeline
in PIPELINE-001, not in ARCH-001, ARCH-002, VALID-001 or WIRE-001. That is an explicit
source-section relevance witness, not a machine-owned section extraction capability.
All five still match because they each index the complete canonical-ir.md body.

The rebuilt B8 executable makes full material available after token-refusal removal, but
does not change this relevance boundary or discovery/acquisition coupling. A future
identity-level discovery response, explicit source-owned section boundaries or richer
query predicates would be separately authorized contract/schema work. None is
implemented here, and lossy summaries or collapse of shared-source identities are
not valid substitutes for exact original supporting evidence.
