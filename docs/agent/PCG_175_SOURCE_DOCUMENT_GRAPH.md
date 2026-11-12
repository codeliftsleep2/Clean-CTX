# PCG source-owned document graph

B3 source disposition: The exact GLS taxonomy now has one current owner in [document authoring](../architecture/PCG_DOCUMENT_AUTHORING.md#inspected-reference-and-exact-taxonomy). The remaining implementation-contract prose is the historical production implementation explanation; the current query/declaration/transport contract is owned by that authoring document, including B2 negotiation. This report retains its original scoped execution evidence, not independent certification.

Agent: **Agent MaxHeadRoom**. Implementation mission, 2026-11-11.

## Inspected reference and exact taxonomy

Current normative owner: [AUTH.PCG_GLS](../architecture/PCG_DOCUMENT_AUTHORING.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## Gap matrix and reconciliation

| Capability | Existing PCG v1 | New document lifecycle |
| --- | --- | --- |
| Stable scoped IDs, exact bytes, attribution | Supported | Reuse identity, path/secret checks and exact hashes |
| Seven document kinds | Missing; v1 claim/decision/constraint/evidence roles differ | Separate document kind, never replace evidence role |
| Twelve statuses | Different; active/superseded/retracted/falsified lifecycle | Separate source-declared document status |
| 13 graph relations | Subset plus CONSTRAINS | Exact GLS document vocabulary; CONSTRAINS stays v1-only |
| Blockers | Different; explicit outcome/subject/authority gates | GLS status satisfaction for document graph only |
| Supersession | Different; subject/applicability policy | GLS successor consistency; preserve immutable snapshots |
| Source-owned metadata | Missing; reviewed manifests | Exact GLS architecture-graph scalar/list grammar |
| Automatic detection/reconciliation | Missing | Query-boundary scan and atomic publication |
| Controlled tags and full-text discovery | Missing | Source-declared track filter and SQLite FTS5 |
| SQLite history/restart | v1 generation persistence exists | Immutable document revisions and generation membership |
| Approvals, independent reclosure/banking | Source-attributed only | Remain source-attributed, no new authorization principal |
| GLS mission IDs, PEOPLE/FIELD tracks, actor names | GLS-specific | Retain exact GLS identity grammar; tracks stay author-owned |
| Generated Markdown/YAML indexes and Git hooks | GLS-specific transport | SQLite authoritative graph state; no second hand-edited index |

## Implementation contract

Opt-in `CLEAN_CTX_PCG_DOCUMENT_PROFILE` selects one owner-controlled JSON profile.
It contains schema_version=1, workspace_root, database_path, document_roots,
max_files, max_nodes, max_edges, max_bytes and token_budget. All limits are explicit
document-catalog limits, not new global caps. Existing v1 profile/schema/query
bytes and accepted corpora stay unchanged. Because v1 checks its exact schema,
the document catalog uses its own isolated SQLite file under the PCG owner,
not existing source persistence or an alternate graph engine.

Authors query `pcg_documents_v1` before writing. Markdown uses top-level
`architecture-graph` fences with the GLS scalar/block-list subset. Required:
node, kind, status, title, track. Optional: artifact, verdict, commit, note and
thirteen lower_snake_case relationship fields. Multiple nodes per file and nested
inert examples are supported. No JSON metadata fence, independent tag registry,
relationship manifest or metadata inference remains. SQLite records exact source
bytes/hashes and artifact witnesses, preserving source attribution without
inventing verification outcomes or actor identity.

Reconciliation is automatic at explicit document-query boundaries, not a polling
worker. A complete selected-root scan validates metadata, witnesses and the whole
candidate graph; a single SQLite transaction publishes all revisions/membership/
relationships/search state. Invalid declarations yield actionable diagnostics
without source edits or a partial authoritative answer. Previously admitted
declarations cannot silently disappear: disappearance is a missing source witness,
not an inferred lifecycle transition. Explicit status/relationship changes retain
the old SQLite revision. No authentication is inferred from a coherent snapshot.

The implementation reuses the Rust PCG identity/security/tokenizer/SQLite and MCP
boundaries. No Python workflow, per-document manifest, enrollment ceremony,
embedding service, approval writes or automatic context injection is introduced.

## Canonical question matrix and verification boundary

| Question | Public route | Required evidence / negative control |
| --- | --- | --- |
| What related documents exist before I write? | search by ID/track/FTS, related/context | Deterministic stored graph, configured root/profile, source track exact match |
| Is my new declaration valid? | automatic reconcile on next query | Duplicate ID/grammar/kind/status/artifact/endpoint errors prevent publication |
| Can a blocked node advance? | get after source status edit | Only GLS candidate rules; satisfied blockers retained, stale BLOCKED rejected |
| Is a historical failure now a success? | history | Exact old status/bytes retained; no implied authorized revision |
| Does restart preserve updates? | new session, get/history | Same committed generation and revision chain |
| Can an invalid edit leak stale authority? | any query | Blocked diagnostics, no active documents payload |
| Are legacy PCG responses unchanged? | pcg_query_v1 | Separate profile/state/catalog and original exact-byte sender |

## Implemented lifecycle and final-source verification

The initial JSON-fence/tag proposal was corrected during implementation to the
actual GLS architecture-graph grammar. There is no parallel JSON metadata format
or tag authority. track is source-declared; exact track and FTS search discover
candidate nodes, while context supplies bounded transitive typed evidence.
See [the authoring contract](../architecture/PCG_DOCUMENT_AUTHORING.md).

The production route is environment-selected document Session -> conditional MCP
catalog/registry -> pcg_documents_v1 -> bounded source capture -> Rust parser ->
candidate validation -> SQLite immutable revisions/generation/relationships/FTS ->
typed query/context traversal -> exact token packing -> COMPLETION observations ->
transaction commit -> existing exact-byte protocol sender. Restart reads the same
SQLite publication history. Invalid candidate transactions roll back; current
documents are empty, while explicit historical_documents remain clearly separate.
Rename retains node identity; removal cannot silently remove admitted knowledge.

The existing pcg_query_v1 handler also acquires a document-source guard for the
same root when the document profile is enabled. Invalid document state refuses
that response; successful projection keeps its original metered bytes and only
returns after the document COMPLETION observation. With no document profile,
legacy schemas/catalog/behavior remain unchanged.

Focused execution: `cargo test --package clean-ctx --lib --all-features documents::`
passed 17 tests, zero failed/ignored (final pass: 26.79 seconds), including the subprocess worker and actual
fixture crash/rollback/reopen parent. Compilation found two adapter/test issues
(return-value mismatch and missing Debug derive) which were corrected before the
green run; neither was behavioral RED evidence. Scoped library Clippy initially
reported map_entry; the source scanner now uses Entry without suppressions.
`cargo clippy --package clean-ctx --lib --all-features -- -D warnings` passed.
`cargo build --package clean-ctx --bin clean-ctx --all-features` passed.

The tracked `verification/pcg-175/run-documents.ps1` production stdio workflow
passed with retained receipt at
`target/pcg-175/documents-live-20261111/receipt.json`. It exercised an empty initial
catalog, author-created DOC-081/068/042 declarations, two-edge constraint context,
valid status revision, invalid status rejection, separately accessible accepted
history, and a fresh process reopening the exact admitted generation. This is
operator/stdio evidence, not a tracked Rust test or CI result. No Python ran.

Operation scope is one explicit query/reconciliation. Whole selected-root capture
and candidate validation are intentionally complete-set work, O(source bytes plus
nodes/edges and deterministic sorting); each distinct source path is read/hashed
once at START and once at COMPLETION, shared evidence is reused operation-locally.
The tracked shared-source work test checks this count. Supersession counts are
computed once, not rebuilt per node. Context traverses indexed adjacency with
visited nodes/edges, explicit request/profile bounds and one deterministic path
per reached node. It does not enumerate every possible path or infer applicability
from prose. SQLite verification compares the current full candidate membership
and accessed historical revision hashes, including search rows, before absence
answers; this is necessary because this feature already re-admits the complete
source-owned catalog. No validation cache, hidden retries or vector engine exists.

Concrete limits: write-lock contention fails immediately; large documents/context
can block the whole response on token budget; declaration discovery is query-time,
not instantaneous; source observations cannot detect unobserved ABA or establish
an atomic multi-file snapshot. A new file arriving after enumeration may wait for
the next query. Missing/unreadable roots refuse current access. History is retained
under the local owner profile, not an enterprise ACL or lawful-erasure system.
Candidate-state validity does not establish revision authorization: prior bytes
and publication order remain inspectable, but no speculative transition matrix
or authenticated approval system was added. General GLS dependency cycles are
representable; context traversal terminates without treating a cycle as proof.

Broad workspace tests, all-target Clippy, benchmarks and accepted corpus/operator
campaigns were not run. Required scoped checks and normal Git encoding/size hooks
are distinct from repository-wide CI. Earlier untracked corpus reports are
unrelated delivery artifacts and remain unstaged.

Final source/definition fingerprint (22 sorted SHA-256/path lines, UTF-8 LF with
final LF): `17ee763e28154853db686d2ba22c847494a2871a7597afea80d85e5c4f0edf43`.
Inventory: `target/pcg-175/documents-live-20261111/source.sha256`; scope is the
document Rust/SQL modules, focused fixtures/tests, MCP wiring and stdio driver.
Executed binary: `54afcb7843bb42a6c8fe7083c6f9f6d543bfa2bf5c681a67ac25fa5017f0194a`.
The final focused pass followed the lint-only Entry change; all new active code
files remain below 615 lines. Changed-file rustfmt and UTF-8/BOM inspection passed.

Delivery inventory: `src/pcg/documents/{mod,model,parse,profile,scan,store,traversal,validate}.rs`
and schema.sql; PCG module export and test fixture registration; MCP state,
conditional registry/catalog/annotations/dispatch, legacy PCG guard and new
document handler; `src/tests/pcg/{document_fixture,documents}.rs`,
`src/tests/mcp/pcg_documents.rs`; tracked stdio driver; authoring contract and this
reconciliation. Prior corpus reports/data are excluded from this commit.

```architecture-graph
node: PCG-175-DOCUMENTS
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Source-owned document graph implementation and scoped evidence
track: REPOSITORY
artifact: docs/agent/PCG_175_SOURCE_DOCUMENT_GRAPH.md
implements:
  - AUTH.PCG_DOCUMENTS
note: Historical implementation and scoped reported verification; B3 does not rerun or independently certify these witnesses.
```
