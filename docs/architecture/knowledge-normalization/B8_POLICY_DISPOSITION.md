# B8 owner policy disposition

Agent: **Agent MaxHeadRoom**, 2026-11-12.

The owner's consolidated B8 authorization retires token-budget enforcement only
for `pcg_documents_v1`. The controlling source remains
`docs/architecture/PCG_DOCUMENT_AUTHORING.md`; this report records the disposition,
and does not create a second normative contract. Historical B2–B7 reports and the
frozen B7 evaluation are unchanged.

## Compatibility and accounting

Request `tokenBudget` and document profile `token_budget` remain accepted optional
unsigned integer fields. Omission defaults to zero. Zero, small, former-limit and
maximum-u64 hints do not limit retrieval. Closed schemas and invalid-type rejection
remain. Examples and the registered MCP descriptor state that the hints are retired.
The actual installed profile is unchanged, including its legacy value of 65,536.

Existing `projection::Response.used_tokens` remains an exact descriptive count of
the serialized MCP result. The response explicitly reports `budget.limit=null`
and `budget.enforced=false`. Tokenizer selection/validation remains supported; no
token count controls admission, pruning, completeness or publication. The removed
overflow path no longer constructs and counts a second blocked response. There is
no replacement token ceiling, escalation or implicit transport selection.

Consumer model context management is separate. Source/currentness, candidate,
database, security, file/byte/node/edge and traversal checks remain effective and
their legitimate failures remain explicit. Default/explicit inline and explicit
context-only source references retain their existing semantics and exact sources.

## Scope preservation

No schema, FTS predicate, traversal, source extraction, shared-source projection,
enrolled-evidence or pipeline implementation changed. Separate enrolled evidence,
code context, workspace queries and proxy budgets are not retired. Static identity
checks and representative enrolled-budget tests are recorded separately from the
actual document production witnesses. The architectural investigation grants no
additional implementation authority.

Only the changed controlling document's source revisions may advance. Its source
declarations are unchanged; existing identities, typed edges and evidence associations
must survive reconciliation. Exact baseline comparison is recorded with the final
production evidence. A current graph does not certify independent agent reasoning,
host adoption, execution authorization, upstream integration or banking.
