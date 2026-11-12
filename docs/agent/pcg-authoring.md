# PCG pre-authoring context

Before a significant architectural decision, investigation, implementation plan
or architectural report, query the configured PCG document graph. Read the source
constraints before proposing work that may contradict earlier decisions.
This is an authoring practice, not a new approval or verification gate.
Ordinary coding edits without a meaningful architectural decision do not require
this preflight. Reuse the current task's applicable, still-current context rather
than querying before every edit.

## Retrieve first

1. Use `pcg_documents_v1` with the explicit workspaceRoot. If the task names a
   node, request `mode:"context"` for that exact ID. Otherwise use `mode:"search"`
   with source-declared track and/or a deterministic FTS text expression, then
   request context for relevant returned IDs. Never invent document IDs.
2. Inspect result_state, catalog_current, diagnostics and context.complete.
   Retain the returned generation and relevant node revisions as the preflight
   identity. A successful RPC or nonempty search is not complete context.
3. Follow the returned typed paths: controlling contracts/invariants, DEPENDS_ON
   prerequisites, BLOCKED_BY obligations, incoming SUPERSEDES successors and
   FALSIFIES counterevidence. Inspect cited source text and artifact witnesses.
   USES_EVIDENCE, PRESERVES and IMPLEMENTS do not establish prerequisite truth;
   VALIDATES is neither independent reclosure nor banking.
4. Carry the relevant constraints and unresolved obligations into the work.
   For example, DOC-081 IMPLEMENTS DOC-068 DEPENDS_ON DOC-042 means the author
   must inspect DOC-068 and DOC-042 before proposing a contradictory change.
   Traversal returns structural relationships, not an inferred applicability or
   conflict verdict. The author assesses the actual proposition and conditions.

Use only profile-permitted token/node/edge budgets. If context is incomplete or
blocked, do not claim the missing chain was checked. Refine an unrelated broad
search or inspect explicit returned references; never drop an applicable
prerequisite merely to obtain a smaller response. An empty catalog/search proves
only that no admitted declaration matched that registered query, not that no
architectural constraint exists elsewhere in the repository.

## Author and reconcile

Prepare the report and its GLS `architecture-graph` declaration together. Stable
node identity, kind, status, track, artifact and typed relationships belong to
the author. Do not infer independent approval from graph status or rewrite a
historical failure to pass validation. Use the source grammar in
[PCG document authoring](../architecture/PCG_DOCUMENT_AUTHORING.md).

After writing or modifying a declaration, issue a get/context query for the node.
This automatically validates and reconciles the candidate graph. Correct actual
source declaration defects using the actionable diagnostics; do not silently
repair metadata in a separate manifest or edit SQLite by hand. Invalid source
must not be reported as current authority. Explicit historical_documents remain
historical, including when the current candidate is invalid.

Compare the final reconciliation generation with the pre-authoring generation.
A different generation is a change signal, not an instruction to restart:

- Compare relevant returned node revisions and typed relationship endpoints/kinds,
  including incoming successors, blockers and counterevidence. A changed source
  revision may be a prose-only edit; inspect the actual changed constraints.
- If a controlling document or a relationship relevant to this work changed,
  retrieve its current context and revisit only the affected conclusions before
  claiming consistency. A newly relevant successor or falsifier also counts,
  even if the original node's own revision did not change.
- If only unrelated documents changed, retain the original reasoning and record
  that the relevant constraint chain is unchanged. Do not repeat the investigation
  or seek another approval merely because the global generation advanced.

For context comparisons, retain the relevant edge set as well as returned node
revisions: one witness path per node is not an inventory of all possible paths.
If the final context is incomplete/invalid, report the unestablished portion;
do not infer that an unseen controlling change was harmless. This is source
comparison by the author, not automatic semantic conflict detection.

In the work report, briefly retain:

- Query starting ID/search, returned generation and relevant revisions.
- Applicable constraint/dependency paths and unresolved blockers or counterevidence.
- Final reconciliation result, or the exact missing/unavailable coverage.
- Any relevant generation/revision/relationship changes and the affected
  conclusions revisited; unrelated generation changes need no new investigation.

No separate review ceremony, semantic conflict engine, watcher, vector service
or corpus manifest is required. Existing owner authorization still determines
what actions may proceed; a graph query does not grant that authorization.

## Unavailable tool or profile

For this repository's configured production workspace, the
[B0/B1 authoring route](../architecture/knowledge-normalization/B0_B1_PROGRESS.md#using-this-production-authoring-route)
documents the existing ignored owner profile and a bounded production stdio
query client. It is an actual-workspace route when the host cannot advertise
the conditional document tool directly. Use the existing profile and explicit
workspace; do not substitute a prior controlled corpus. Starting that client
remains subject to the existing runtime-execution rule and current owner
authorization. Direct host exposure requires a host session launched with the
profile; it is not established by writing configuration or by a stdio receipt.

If pcg_documents_v1 is not advertised, the current host/session cannot expose the
document capability. State that limitation and consult the known authoritative repository
documents through the normal tooling policy; do not pretend a PCG preflight ran.
Do not create profiles, start servers or enroll production data without task
authority. If a concrete controlling prerequisite cannot be established, report
that specific unresolved condition rather than infer permission from tool absence.

This instruction makes use of the graph explicit and reviewable. It is not
runtime enforcement that every agent obeys it; real authoring sessions must
supply the query/reconciliation evidence before adoption can be called observed.

```architecture-graph
node: AUTH.PCG_PREAUTHORING
kind: AUTHORITY
status: IMPLEMENTED
title: Retrieve constraints before authoring and reconcile source changes
track: REPOSITORY
artifact: docs/agent/pcg-authoring.md
consumes_authority:
  - AUTH.PCG_DOCUMENTS
note: Operative source-described contract; status does not authenticate approval or certify implementation.
```
