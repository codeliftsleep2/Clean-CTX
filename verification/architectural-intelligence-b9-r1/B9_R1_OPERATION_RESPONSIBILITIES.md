# Operation responsibilities and bounded work

Agent: **Agent MaxHeadRoom**. Source contract: PCG_DOCUMENT_AUTHORING.md.

| Operation | Deterministic responsibility | Content responsibility |
|---|---|---|
| search | Existing proposition/component FTS, metadata predicates and attributed match references | No component text or excerpts; explicit acquisition requests. |
| get | Exact identity and selected role/label descriptors, required qualification refs and outgoing edges | No component content. |
| related | Existing incident typed edge/direction selection and endpoint descriptors/provenance | No endpoint content. |
| context | Existing bounded typed transitive closure, all members/edges/witnesses and exact references | No automatically materialized authority text. |
| acquire | One or multiple exact component references, revision/boundary/hash validation, required qualifications, ordered dispositions | Explicit selected content only; no graph expansion. |
| source | Explicit complete current or retained associated artifact with exact pin checks | Full source only because expressly requested. |

Every operation retains the existing single source scan, candidate validation,
atomic publication and observed-boundary completion recheck. The new acquisition
batch reconciles once. Within a batch, revision/membership/descriptor sets are
resolved once per identity/revision/historical policy, each shared artifact is
read/hashed once, and each exact source interval is validated/decoded once.
Duplicate input references retain ordered outcomes while reusing exact resolution.
There is no persistent acquisition cache, receipt owner, new lifecycle or budget.

Expected acquisition work scales with distinct referenced revisions, source bytes,
exact requested/required qualification intervals and ordered response membership.
Retaining complete required qualifications is semantic work, not optional pruning.
Tracked tests cover duplicates and 2/8/32 distinct units sharing one source and
qualification interval. Test-only work counters are not production instrumentation.
Existing source-scan bounded-work tests remain, and no new semantic producer is
introduced. Agent reasoning chooses needed acquisition; Rust/SQLite perform all
identity, closure, revision, join, interval and completeness computation.
