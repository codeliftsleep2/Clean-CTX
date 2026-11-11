# Source-owned PCG document authoring

Graph declarations belong to their source documents. PCG parses and validates
them; SQLite owns admitted graph revisions/history. No document metadata is
inferred or silently repaired. This interface is distinct from the unchanged
owner-enrolled `pcg_query_v1` evidence-corpus interface.

## Configure once per workspace

Set `CLEAN_CTX_PCG_DOCUMENT_PROFILE` to an absolute owner-controlled JSON file:

```json
{
  "schema_version": 1,
  "workspace_root": "C:\\work\\repository",
  "database_path": "C:\\work\\repository\\.clean-ctx\\documents.sqlite",
  "document_roots": ["docs"],
  "max_files": 2048,
  "max_nodes": 2048,
  "max_edges": 8192,
  "max_bytes": 33554432,
  "token_budget": 32768
}
```

The workspace, document directories and DB parent must already exist. These are
example finite limits, not defaults. max_files debits examined filesystem entries;
max_bytes covers distinct reads at both observation boundaries. Profiles and DBs
must be outside scanned roots. Symlinks/reparse points/traversal are refused.
The session pins profile bytes and permissions; configuration changes require a
fresh session. Changing the physical catalog scope requires a new DB; adjusting
operational ceilings across sessions does not discard history. Profiles are not
per-document enrollment manifests. There is no watcher or background ingestion.

## Query before writing

Call `pcg_documents_v1` with:

```json
{
  "workspaceRoot": "C:\\work\\repository",
  "mode": "context",
  "id": "DOC-081",
  "maxNodes": 64,
  "maxEdges": 256,
  "tokenizer": "o200k",
  "tokenBudget": 32768
}
```

Modes: search (optional exact track and SQLite FTS5 text expression), get (ID),
related (one-hop incoming/outgoing), context (transitive pre-authoring context),
history (ID). No vector service is needed; vectors cannot establish status/edges.
ID is mandatory outside search. Track/text apply only to search; maxNodes/maxEdges
apply only to context. Profile ceilings bound requests. All schemas are closed.

Context follows every outgoing typed relation and incoming SUPERSEDES, FALSIFIES,
VALIDATES, RECLOSES and BANKS, using a deterministic visited traversal. It returns
complete source documents, artifact evidence, typed relationships and one witness
path per discovered node. Thus DOC-081 IMPLEMENTS DOC-068 DEPENDS_ON DOC-042
reveals both the contract and invariant before writing. Incoming falsifiers and
successors remain visible. USES_EVIDENCE/PRESERVES/IMPLEMENTS do not become
prerequisites, VALIDATES does not become reclosure, and reclosure is not banking.
Each relation includes an explicit semantic meaning and source-byte provenance.

Context exhausts its reachable selected graph or reports incomplete with no
partial context when the explicit node/edge limits are reached. No top-k relevance
heuristic or inferred applicability is used. Large connected descriptive graphs
may require a narrower starting point or an explicitly larger permitted request.
This is structural source-backed context, never permission to execute a mission.

## Author the declaration in Markdown

````text
# Proposed architecture work

The report body owns the actual proposition, conditions and evidence.

```architecture-graph
node: DOC-081
kind: MISSION
status: INVESTIGATING
title: Investigate source freshness policy
track: REPOSITORY
artifact: docs/investigations/source-freshness.md
implements:
  - DOC-068
preserves:
  - DOC-042
```
````

Use the exact GLS seven kinds, twelve statuses and thirteen directional relations
listed in [the reconciliation](../agent/PCG_175_SOURCE_DOCUMENT_GRAPH.md).
`node`, `kind`, `status`, `title`, `track` are required. Additional supported scalar
fields are `artifact`, `verdict`, `commit`, `note`. Relationships use established
lower_snake_case keys and two-space `- target` block items. Scalars are literal
strings, without YAML inference. Duplicate/unknown fields, nesting, malformed
indentation and inline-list syntax are errors. Several nodes may inhabit one file;
examples nested in longer fences are inert. No `tags`, generic `relationships`,
JSON declaration or sidecar graph file is accepted. Track is author-owned and
queryable; PCG does not create a second tagging authority.

GLS IDs include DOC-081, P677PR, AUTH.CANONICAL_IR and INV.SOURCE_CURRENTNESS.
An artifact path must be a readable repository file. Standing statuses require
artifact exactly as GLS does; metadata is not a substitute for substantive proof.
Status VALIDATED or RECLOSED remains source-declared and does not execute a test,
authenticate an approver or prove independent closure. Use the report body and
typed evidence relations to retain the actual test subject/outcome/provenance.

## Admission, history and freshness

The next explicit query detects source changes, validates the complete selected
candidate graph and atomically publishes revisions/relationships/FTS state. No
manual import is required. Invalid source receives actionable diagnostics and no
current document payload; the prior admitted graph remains historical. history
returns `historical_documents`, separately labelled snapshots, even while the
candidate is invalid. It never presents them in the current `documents` array.
Source removal is blocked instead of silently erasing a known identity; rename
with the same node ID creates a new revision and retains old paths. Represent
semantic replacement through explicit SUPERSEDES and source status, not deletion.

GLS candidate consistency is enforced without an invented pairwise transition
matrix. Immutable source revisions and publication order make changes observable;
`revision_authorization:not_established` makes clear that a valid snapshot does
not prove permission to change an earlier decision. Broader authorization and
contradiction policy would require concrete separate requirements.

Freshness is exact source/permission observations before interpretation and after
packing, not continuous immutability or an atomic filesystem snapshot. Failure
at COMPLETION discards the response and rolls back publication. With both PCG
profiles enabled for the same root, `pcg_query_v1` also guards against invalid
current document declarations and checks them again before returning its original
metered bytes. Other source-code tools do not consume this document catalog.

The entire MCP result is exactly token-counted with cl100k/o200k and sent through
the existing exact-byte protocol seam. Required overflow returns a bounded blocked
control result with no partial source documents; too-small control budgets are
protocol errors. SQLite busy fails without retry. Current catalog reconciliation
is a write effect: its MCP annotation is not read-only, despite source documents
never being edited. FTS5 must be supported by SQLite; absence refuses initialization.
