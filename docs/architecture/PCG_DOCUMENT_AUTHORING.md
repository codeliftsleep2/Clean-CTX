# Source-owned PCG document authoring

**Normal architectural knowledge workflow:** use `pcg_knowledge_v1` as defined
below. `pcg_documents_v1` remains the explicit legacy whole-document interface.

For example, pass `@{workspaceRoot=(Get-Location).Path; mode='get'; id='PIPELINE-001'}`
as the knowledge tool's arguments from this repository root. Use `search` for
identity discovery, `related` for typed relationships, `context` for the complete
declared authority bundle, and `source` for original evidence. See
[the agent workflow](../agent/pcg-authoring.md) for the normal authoring path.

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
  "max_bytes": 33554432
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

## Legacy whole-document retrieval

Call `pcg_documents_v1` with:

```json
{
  "workspaceRoot": "C:\\work\\repository",
  "mode": "context",
  "id": "DOC-081",
  "maxNodes": 64,
  "maxEdges": 256,
  "tokenizer": "o200k"
}
```

Modes: search (optional exact track and SQLite FTS5 text expression), get (ID),
related (one-hop incoming/outgoing), context (transitive pre-authoring context),
history (ID). No vector service is needed; vectors cannot establish status/edges.
ID is mandatory outside search. Track/text apply only to search; maxNodes/maxEdges
apply only to context. Profile node/edge ceilings bound requests. All schemas are closed.

Document responses are not token-budget limited. The legacy request `tokenBudget`
and profile `token_budget` fields are optional, accepted compatibility hints with
no enforcement effect. Exact tokenizer accounting remains diagnostic. Response
`budget.limit` is null and `budget.enforced` is false. This document-only change
does not change the separate enrolled-evidence `pcg_query_v1` budget contract.
Token count is not a document admission, completeness, retrieval or publication
criterion. Consumer-side model context management is separate from acquiring
architectural knowledge. Legitimate source, scope, security, database and structural
resource failures remain explicit; completeness and currentness still control
whether returned knowledge can be used as a complete current context.

Owner policy disposition (B8, 2026-11-12): this source-owned contract replaces the
earlier document-specific token-refusal policy. B2–B7 reports retain their exact
historical budget outcomes. No historical measurement or refusal is rewritten,
and no new graph edge, lifecycle, or competing normative owner is introduced.

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

The entire final serialized MCP **result** is counted after projection, including
reference metadata and JSON escaping. Token counts do not refuse or clear document
results. Finite source/file/byte and context node/edge limits remain enforced;
an incomplete context still returns no partial authoritative bundle.
History remains the separate, unchanged
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
the existing exact-byte protocol seam. Document token-budget refusal is removed;
other resource, source, and candidate failures retain their explicit diagnostics.
SQLite busy fails without retry. Current catalog reconciliation
is a write effect: its MCP annotation is not read-only, despite source documents
never being edited. FTS5 must be supported by SQLite; absence refuses initialization.

<!-- pcg:GLS-grammar-and-bootstrap:begin -->
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

<!-- pcg:GLS-grammar-and-bootstrap:end -->
```architecture-graph
node: AUTH.PCG_GLS
kind: AUTHORITY
status: IMPLEMENTED
title: Exact imported GLS grammar and candidate-state lifecycle
track: REPOSITORY
artifact: docs/architecture/PCG_DOCUMENT_AUTHORING.md
note: Operative source-described contract; status does not authenticate approval or certify implementation.
knowledge_scope: authority
knowledge_content:
  - statement self#GLS-grammar-and-bootstrap
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
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```

## Independently retrievable knowledge (B9)

Owner-authorized B9 corrects retrieval granularity without replacing Markdown
authority, identities, typed relationships, historical evidence or B8's removal
of document token budgets. A source artifact can own several knowledge units.
Rust extracts explicitly owned content; SQLite indexes and selects it. An agent
reasons about that knowledge rather than locating each proposition in a document.

Extend the same flat declaration with `knowledge_scope` and `knowledge_content`.
Both must be present together. Scope is `proposition`, `decision`, `invariant`,
`constraint`, `finding`, `authority`, `evidence`, or `multi_section`. This is content
scope, not a new graph kind or lifecycle state. A complete contract/report may
legitimately use `whole`; a proposition sharing a source requires its own selector.
No implicit whole-source fallback certifies unrepresented knowledge.

Each ordered content entry is `role[:label] path#selector`. `self` means the
declaring source; other paths must be safe repository-relative artifacts. `whole`
selects the complete observed artifact. Other selectors name exact paired marker
lines, outside fenced examples. The selected UTF-8 bytes begin after the begin
line and end before the end line. Marker names are unique per source. Proper
nesting is allowed; missing, duplicate, malformed, unclosed or empty boundaries
refuse admission. Headings, proximity and filenames are not extraction authority.

```markdown
<!-- pcg:EXACT-SUBJECT:begin -->
Exact original proposition, including its necessary conditions and rationale.
<!-- pcg:EXACT-SUBJECT:end -->
```

Source-owned component roles are `statement`, `subject`, `scope`, `outcome`,
`rationale`, `alternative`, `rejected_alternative`, `rejection_reason`, `consequence`,
`constraint`, `verification_obligation`, `observation`, `counterexample`,
`qualification`, `historical_applicability`, and `evidence`. A statement is required.
Labels and selectors use ASCII letters/digits/dot/underscore/hyphen; labels are
explicit semantic annotations, not inferred facts. Multiple intervals can preserve
shared definitions and qualifications. Do not fabricate missing components or
duplicate normative prose. Historical reports and uncertainty remain attributed.

### B9-R1 reference-first responsibility correction

Owner decision, 2026-11-12: this section supersedes B9's automatic component
materialization for knowledge `get` and `context`. Historical B9/B10 reports and
their captured content-bearing responses remain records of that implementation;
they are not rewritten as evidence of this corrected contract. The normal
knowledge response version is 2 under the existing `pcg_knowledge_v1` tool.
The five existing operation names remain, with explicit `acquire` added for one
or multiple selected component references. Legacy `pcg_documents_v1` and the
separate enrolled-evidence interface retain their own behavior and contracts.

Graph retrieval is a map of source-owned knowledge, not source acquisition.
`search`, `get`, `related` and `context` return structural metadata, complete
applicable selected graph information and exact references, never component
text, document bodies or generated excerpts. SQLite continues retaining exact
source bodies, component intervals/hashes and immutable historical revisions.
Authoritative Markdown remains the source of truth; Rust/SQLite perform
deterministic graph/reference/acquisition work. Agents choose necessary explicit
acquisitions and interpret their meaning without reconstructing closure or
guessing source boundaries. Selective acquisition does not authorize selective
compliance with applicable governing authority.

`pcg_knowledge_v1` requires explicit `workspaceRoot` and `mode`:

- `search`: optional FTS `text`, exact `track`, `kind`, `status`, `role`, and `label`.
  Returns identity/title/status/scope/revision, match origins and ready acquisition
  references. It does not materialize source bodies or claim authority closure.
  FTS indexes each admitted component and the composed knowledge content. Identity/
  title matches are distinct. If only a multi-component expression matches,
  `composed_knowledge` does not falsely attribute it to one component.
  Composed matching uses the source-declared component order and may span component
  boundaries; it does not assert a contiguous quotation in one original artifact.
  Role/label
  predicates select the explicitly typed components. Order is deterministic ID,
  not a semantic relevance rank. Shared source ownership creates no sibling match.
- `get`: exact `id`, optionally a declared `role`/`label` selection. Returns ordered
  component descriptors, source/interval/content hashes, evidence references and
  outgoing typed relationships. Full units retain all qualification references.
  Filtered structural selection retains qualification descriptors whenever its
  predicate matches; `selected_by_predicate` distinguishes requested components
  from required interpretation references. `unit_complete:false` describes a
  filtered descriptor set, not acquisition of any component text.
- `related`: exact `id`, optional `edgeKind` and `direction` (`incoming`, `outgoing`,
  `both`). Returns actual typed edges, provenance, meanings and endpoint identities.
  No endpoint source bodies are acquired merely for relationship discovery.
- `context`: exact `id`, optional existing `maxNodes`/`maxEdges`. Rust computes the
  unchanged typed transitive closure and deterministic witnesses, returning every
  selected unit's component/qualification/evidence/obligation references. Query
  roots, generation and graph completeness remain explicit. No source content
  is acquired and no relationship reconstruction is delegated to the LLM. A
  structural overflow returns no partial authority bundle.
- `acquire`: explicit nonempty `references` array, with one or multiple returned
  exact component references; no `id` or graph selectors at the request root.
  Each reference pins `id`, `revision`, `ordinal`, `role`, optional/null `label`,
  `path`, `sourceSha256`, `start`, `end`, `contentSha256` and `selector`.
  Rust compares all fields with the stored immutable component descriptor and
  validates exact UTF-8 bytes/content hashes. Every selected component includes
  all qualification components associated with that same unit/revision, even
  when the caller supplied only the non-qualification reference. There is no
  automatic graph expansion or unrelated unit acquisition.
  Input order is preserved, with one disposition per reference. Exact duplicate
  references reuse operation-local resolution but retain a result with
  `duplicate_of` identifying the earlier input position. Invalid/stale references
  explicitly refuse with no content; independent successful references remain,
  and overall `complete:false`/`result_state:incomplete` discloses any refusal.
  Malformed request shapes/unknown fields refuse the operation. Database/security
  failures remain operation errors, not silently downgraded reference failures.
  No arbitrary batch count, token budget or response ceiling is introduced.
- `source`: exact `id`, optional associated `path`. Returns a complete current
  original artifact, UTF-8 text or lossless byte-array data for binary evidence.
  Source access is restricted to that identity's declared/associated artifacts.
  Returned references pin `sourceSha256`; a changed artifact returns explicit
  `source_revision_changed`, never different bytes under the old reference.
  Existing material-denial checks also apply to a binary detection view without
  changing the returned original bytes. Legitimate security failures remain explicit.
  Optional `revision` pins the associated knowledge revision as well. Explicit
  `historical:true` requires `revision` and retrieves the retained associated
  artifact rather than current filesystem content. If original bytes were not
  retained under the existing storage contract, return `document_source_unavailable`;
  never substitute current bytes. Existing legacy history reconstruction remains.

References include workspace, identity and associated path so consumers need no
database knowledge or arbitrary SQL. Required original evidence is exposed by
complete, exact, current artifact references and explicit source retrieval; there
are no optional evidence tiers. Component acquisition is exact text, not a summary. No token count
controls selection, admission, completeness or publication. Whole-source legacy
search remains explicitly document discovery, not proposition discovery.

Read `result_state`, `catalog_current`, `complete`, diagnostics and catalog coverage.
Discovery over unrepresented identities is incomplete; selected missing knowledge
also returns incomplete without an authoritative partial bundle. Complete means
complete against this declared operation/traversal, not all inferred applicability.
Lifecycle labels and parsed obligations do not establish execution or authorization.

Component references are durable revision-bound data, not session-local handles.
Default `historical:false` requires that the pinned revision is still the current
member for that identity. Source change between graph retrieval and acquisition
returns `source_revision_changed`, never a silently substituted new revision.
The caller must explicitly reacquire the current graph to adopt changed authority.
Explicit `historical:true` resolves that retained immutable component revision and
its original qualifications as `historical_snapshot`; it does not certify current
authority. Legacy document history continues exact original-document reconstruction.
Current whole-source `source` references retain their existing source-hash refusal.

Graph selection completeness, complete reference representation, requested content
acquisition completeness, source currentness, host delivery, agent inspection and
comprehension are separate claims. `content_acquired:false` on graph responses
means content has not been retrieved. An acquisition response proves neither
complete authority acquisition nor model delivery/comprehension. Existing B10
delivery-boundary findings remain valid and their further investigation is
suspended under B9-R1; no host limit/presentation policy is changed here.

Catalog schema v2 deduplicates complete `(path,hash)` source storage and adds
revision-bound unit/component projections and attributed FTS. Exact v1 catalogs
migrate transactionally, retaining every revision, predecessor, generation,
publication, identity, edge, evidence association and original body. Source-bound
revision hashing remains unchanged. Explicit external content associations enter
the same evidence/currentness identity. Failed migration/reconciliation rolls back.
Stable knowledge reads verify current members, exact source/revision metadata,
declared edges and the knowledge index, without materializing or certifying the
unconsumed legacy FTS view. Explicit legacy reads retain their existing legacy
index check. Every newly published generation builds and verifies both projections
in the same transaction.
Unsupported/corrupt schemas refuse. Old binaries refuse v2; there is no destructive
downgrade. Legacy response formats and full original history remain supported by
the new executable. The separate enrolled interface and unrelated budgets are unchanged.
