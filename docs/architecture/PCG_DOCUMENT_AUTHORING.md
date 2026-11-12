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

## Explicit lossless context transport

Omitting `responseFormat`, or selecting `"inline_v1"`, preserves the original
version-1 response. Existing clients need no change. Explicitly select
`"responseFormat":"source_refs_v1"` only with `mode:"context"` and a consumer
that implements the following version-2 representation. Other modes reject that
selection; unknown values and explicit null are invalid parameters.

The JSON text in the MCP result has `version:2`,
`response_format:"source_refs_v1"`, and a `sources` array. Each source entry has
`source_ref`, `path`, `sha256`, and exact UTF-8 `text`. Identity is the exact
`(path, sha256)` pair, with exact content equality required on reuse. Identical
text at different paths remains distinct. References are deterministic hashes
of the canonical path/hash identity; the array is ordered by path then hash.

Every current document retains its own declaration, revision, previous revision,
path, source hash, evidence associations and other metadata. Its `source_text`
is replaced by `source_ref`. Evidence entries containing string `text` similarly
replace that field with `source_ref`; absent or null text stays unchanged. A
source used both as declaration text and artifact text shares one pool entry
when its exact identity and bytes agree. This is response-local sharing, not a
new storage identity, cache, graph node or authority boundary.

To reconstruct version 1, resolve each document reference and restore
`source_text`; resolve each evidence reference and restore `text`. Check the
reference's path/hash against its owner and verify the source bytes against
SHA-256. Remove those references and the top-level `sources` and
`response_format`, then restore `version:1`. All remaining fields, including
node revisions, relationships, witness paths and evidence metadata, are
unchanged. Consumers must first inspect `result_state`, `catalog_current`, and
`context.complete`; source references do not turn an incomplete result into
complete knowledge.

The entire final serialized MCP **result** is metered after projection, including
reference metadata and JSON escaping. On budget overflow, documents, source
pool, relationships and witness paths are cleared together; no partial
authoritative bundle is returned. History remains the separate, unchanged
version-1 `historical_documents` interface. This format does not change source
scanning, reconciliation, SQLite schema, traversal, freshness observations,
authorization or configured limits. Reference overhead can exceed savings for
contexts without repeated payloads; selecting it does not promise a smaller
response.

The independent production consumer is
[`compare-document-source-responses.py`](../../verification/pcg-175/compare-document-source-responses.py).
It verifies complete reconstruction and same-generation default compatibility
using captures from the production stdio driver.

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
listed in [the GLS taxonomy below](#inspected-reference-and-exact-taxonomy).
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

## Inspected reference and exact taxonomy

Actual GLS repository: `C:/Users/MNasty/Desktop/GoalLine Stand 2026`, HEAD
`491988992586f08984c523034a27f87e1140fec8`. Its worktree contains unrelated
changes. The governing doctrine, validator and publication contract were read;
no GLS file was modified. This supersedes neither the older PCG discovery pin
nor any domain finding there. Reference files:
`docs/ARCHITECTURE_EVIDENCE_GRAPH.md`, `scripts/architecture_graph.py`,
`docs/DOCUMENT_GRAPH_PUBLICATION_CONTRACT.md`, and P677PR's actual reclosure
declaration (RECLOSES P677P; preserves authorities/invariants; no banking claim).

Seven kinds, exactly: `MISSION`, `IMPLEMENTATION`, `EXPERIMENT`, `RESEARCH`,
`AUTHORITY`, `INVARIANT`, `DEBT`.

Twelve statuses, exactly: `PROPOSED`, `INVESTIGATING`, `FROZEN`,
`IMPLEMENTATION_READY`, `IMPLEMENTED`, `VALIDATED`, `RECLOSED`, `BANKED`,
`BLOCKED`, `FALSIFIED`, `SUPERSEDED`, `DEBT`.

| Relation | Source -> target meaning |
| --- | --- |
| DEPENDS_ON | proposition -> necessary premise |
| USES_EVIDENCE | proposition -> informative evidence, not premise |
| PRESERVES | later -> earlier truth respected |
| IMPLEMENTS | implementation -> realized contract |
| VALIDATES | evidence -> supported proposition |
| RECLOSES | independent reclosure -> closable artifact |
| BANKS | banking decision -> banked target |
| BLOCKED_BY | blocked proposition -> unresolved gate |
| SUPERSEDES | controlling interpretation -> historical proposition |
| FALSIFIES | frozen counterevidence -> refuted proposition |
| DEEPENS | extending proposition -> authority extended without replacement |
| PRODUCES_AUTHORITY | establishing evidence -> AUTHORITY |
| CONSUMES_AUTHORITY | consumer -> AUTHORITY |

GLS has **no pairwise status-transition matrix**. Candidate rules: IDs unique,
valid kinds/statuses, existing required artifacts/endpoints, no self-edge;
DEPENDS_ON cannot target FALSIFIED/SUPERSEDED; IMPLEMENTS targets MISSION,
IMPLEMENTATION or AUTHORITY; RECLOSES targets MISSION/IMPLEMENTATION and a
RECLOSED source requires a RECLOSED/BANKED target; SUPERSEDES targets SUPERSEDED;
authority relations target AUTHORITY. SUPERSEDED requires exactly one incoming
SUPERSEDES. BLOCKED needs at least one unresolved blocker; only RECLOSED/BANKED
satisfy a blocker. IMPLEMENTATION_READY/VALIDATED/RECLOSED/BANKED require all
declared blockers satisfied. No status automatically authenticates approval,
independence, executable success or banking. General dependency cycles are not
prohibited by this GLS validator.

History doctrine retains failed results, superseded propositions and resolved
blocker relationships. It does not supply an executable permission system.
Candidate-state validity and revision authorization remain different questions.
The initial PCG implementation adds no speculative pairwise restrictions.


Source: [docs/agent/PCG_175_SOURCE_DOCUMENT_GRAPH.md](../agent/PCG_175_SOURCE_DOCUMENT_GRAPH.md), commit `187d0096`, original section SHA-256 `6f77a7908d5dd9c979973ad5c82972f1e6db10556823679a716e0a7021d7dc09`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## Identity grammar and bootstrap boundary

The existing `src/pcg/documents/validate.rs::node_id` is the implementation of the imported GLS ID grammar. It accepts `AUTH.`, `INV.` or `DEBT.` followed by an uppercase ASCII letter then uppercase letters/digits/underscores; GLS `P` missions with three or four digits, an uppercase alphanumeric suffix and optional uppercase alphanumeric hyphen components; or a three-to-eight-letter uppercase infrastructure prefix, a three-digit number with optional uppercase suffix, and optional uppercase alphanumeric hyphen components. IDs are case-sensitive. This documents the existing parser, not a new naming system or allocation registry.

The document catalog describes governing source propositions; it does not supply the authority to implement or execute its own interpreter. Production code determines actual behavior. Controlling source contracts, repository instructions, tracked tests, operator receipts and independent certification retain their distinct roles. A successful scan/query establishes only the stated candidate, provenance, completeness and observed-freshness result. It does not prove the interpreter correct, authenticate a revision or approver, execute witnesses, or independently certify reconciliation. `AUTH.PCG_GLS` and `AUTH.PCG_DOCUMENTS` are separate logical propositions sharing this cohesive authoring source; the catalog depends on the imported grammar, not the reverse. Neither status nor a dependency cycle bootstraps execution permission.

The separate enrolled `pcg_query_v1` surface consumes an explicitly owner-enrolled immutable evidence generation. Its four roles, lifecycle, applicability, closure, approval-attribution and integrity assumptions are governed by [enrolled v1](contracts/pcg-enrolled-v1.md), [acquisition](contracts/pcg-acquisition-v1.md), [projection](contracts/pcg-projection-v1.md) and [evidence MCP](contracts/pcg-mcp-v1.md). They are not the catalog's seven kinds/twelve statuses/GLS cycle rules. Catalog reconciliation writes only its isolated derived document state; it does not import, approve or mutate enrolled corpora. The existing same-root document guard on the evidence handler is a refusal/currentness guard, not authority transfer or replacement of metered v1 bytes.

Runtime actors, approval transitions, authorization, retention/erasure and a ledger require a separately reviewed M4b design. Source-attributed status, owner labels, graph admission and successful queries supply none of those contracts. See the preserved [M4b reconciliation](../agent/PCG_175_M4B_SCOPE_RECONCILIATION.md). No blanket supersession joins the two interfaces.

```architecture-graph
node: AUTH.PCG_GLS
kind: AUTHORITY
status: IMPLEMENTED
title: Exact imported GLS grammar and candidate-state lifecycle
track: REPOSITORY
artifact: docs/architecture/PCG_DOCUMENT_AUTHORING.md
note: Operative source-described contract; status does not authenticate approval or certify implementation.
```

```architecture-graph
node: AUTH.PCG_DOCUMENTS
kind: AUTHORITY
status: IMPLEMENTED
title: Source-owned document reconciliation and retrieval
track: REPOSITORY
artifact: docs/architecture/PCG_DOCUMENT_AUTHORING.md
depends_on:
  - AUTH.PCG_GLS
preserves:
  - ARCH-003
note: Operative source-described contract; status does not authenticate approval or certify implementation.
```
