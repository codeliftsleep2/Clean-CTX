# PCG-175 REAL-USE PILOT 01: corpus/profile proposal

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: Phase A preparation and owner review only. No enrollment or MCP
execution before approval of the proposed manifest and profile.

Disposition: **READY FOR CORPUS/PROFILE REVIEW — NOT ENROLLED**.
PCG v1 remains accepted and closed. No production code, accepted tests, existing
fixtures, Git index/refs or enrollment was modified. No PCG command, server,
test/build/lint campaign or tokenizer was executed. This report proposes a real
usefulness exercise; it claims no pilot success yet.

## 1. Review artifacts and exact identity

Pinned local repository revision:
`6add8449c1a036d0599283fd6a982009d495e954`.
Sources were acquired as exact local Git blob bytes at that revision, not inferred
from an earlier report or retrieved from a later cloud branch tip.

Review directory: `target/pcg-175/pilot01/review/`.

| Artifact | Review purpose |
| --- | --- |
| [manifest.proposed.json](../../target/pcg-175/pilot01/review/manifest.proposed.json) | Complete closed v1 sidecar: 7 sources, 18 evidence spans, 7 required constraints, 2 selectors, 7 bindings and 8 prerequisite edges. |
| [profile.proposed.json](../../target/pcg-175/pilot01/review/profile.proposed.json) | Exact owner-review digest pin, isolated scope/storage and per-pilot limits. Inert until approved and explicitly used. |
| [proposed-revisions.json](../../target/pcg-175/pilot01/review/proposed-revisions.json) | All source/evidence/record/selector/relationship revisions and a static predicted generation; not importer confirmation. |
| [source-catalog.json](../../target/pcg-175/pilot01/review/source-catalog.json) | Full Git blob OIDs, SHA-256, source sizes and working-file comparison. |
| [evidence-spans.json](../../target/pcg-175/pilot01/review/evidence-spans.json) | Exact byte ranges, excerpt hashes and pinned line orientation. |
| [queries.proposed.json](../../target/pcg-175/pilot01/review/queries.proposed.json) | Four explicit proposed MCP argument objects: constraints, closure, provenance and impact. |
| `review/excerpts/*.txt`, `review/pinned_sources/**` | Exact excerpt bytes and inspection copies of the selected Git sources. Not an enrolled workspace or a new authority owner. |

Manifest raw UTF-8 SHA-256:
`a570c20629da16c422d60698ff4e7aae738d29855033dfa5a447e93f6360870d`.
Profile raw UTF-8 SHA-256:
`4b33647c87fff76759c24bf7b13a2e0c7c437653da83196ef0f25bb7872cc499`.
Static predicted generation:
`9cd113849a41cc4edc60304105b1fffa2fdf55df67365cd25c97f855ea16ee69`.

Those revisions apply the existing published v1 hash rules, including null
optional fields in normalized selector hashes and sorted evidence-ref sets.
They are preparation calculations, not a generation present in SQLite. Phase B
must retain the actual importer result and compare it without silently rewriting
an approved digest or prediction.

## 2. Proposed source set and why each addition is necessary

| Source ID | Repository path | Reason |
| --- | --- | --- |
| invariants | `docs/ARCHITECTURAL_INVARIANTS.md` | Named CBM-ID-001, CBM-LIFE-001, CBM-CACHE-001 and CBM-E-001 contracts: identity, lifecycle qualification, ownership and error/nonempty distinction. |
| migration | `docs/architecture/CBM_PROJECT_CACHE_OWNERSHIP_MIGRATION_PLAN_2026-09-28.md` | Approved decision scope, exact invalidation/refresh ordering, concurrency boundary and approval stops. |
| overview | `docs/ARCHITECTURE_OVERVIEW.md` | CBM discovery versus WorkspaceIndex semantic authority; identity summary and its explicit reference to the named invariant. |
| cache-code | `src/cbm/bridge/cache.rs` | Directly referenced by CBM-CACHE-001. Shows explicit owner-based memory/disk lookup and target-only invalidation; an active A must not supply B's disk root. |
| indexing-code | `src/cbm/bridge/indexing.rs` | Directly referenced by the invariants. Shows non-active target refresh, the success-only generation advance/invalidation, readiness branches and error propagation. |
| query-code | `src/cbm/bridge/query.rs` | Directly referenced by CBM-CACHE-001. Shows scoped owner resolution, cache-hit behavior and recorded query errors. Necessary to locate the actual cache consumer. |
| handler-code | `src/cbm/handlers.rs` | Required by the current public call path: target selection, freshness gate before query/cache access, and consumption of `take_last_error`. Prevents claiming the low-level helper alone enforces the public failure contract. |

These seven sources total **331,332 Git bytes**. Enrolled proposed excerpts total
**31,048 bytes**, shared across the records. No test source is enrolled as proof
of execution. Other cited tests/provider reports are not needed to establish this
bounded normative/source-inspected answer, and their historical PASS claims are
not newly certified. No company corpus, transcript or credential is selected.

## 3. Exact evidence spans

Ranges below are half-open UTF-8 bytes in pinned Git blobs; line numbers are
orientation only. Full excerpt SHA-256 values are in `evidence-spans.json` and
the manifest. Original newlines and Unicode are preserved.

| Evidence | Source | Pinned lines | Byte range |
| --- | --- | --- | --- |
| inv-identity | invariants | 200–210 | [18966,21563) |
| inv-lifecycle | invariants | 211–221 | [21563,23354) |
| inv-cache | invariants | 222–232 | [23354,25704) |
| inv-errors | invariants | 233–242 | [25704,27136) |
| plan-decision | migration | 22–61 | [842,2496) |
| plan-owners | migration | 124–177 | [5090,6788) |
| plan-invalidation | migration | 178–200 | [6788,7515) |
| plan-refresh | migration | 201–235 | [7515,8618) |
| plan-controls | migration | 236–258 | [8618,9768) |
| plan-approval | migration | 712–733 | [31427,32420) |
| overview-authority | overview | 67–78 | [2982,3596) |
| overview-identity | overview | 678–685 | [38515,39808) |
| cache-invalidation | cache-code | 101–116 | [3834,4502) |
| cache-lookup | cache-code | 230–261 | [9477,10765) |
| index-target | indexing-code | 478–580 | [21503,26284) |
| query-owner | query-code | 336–383 | [14429,16610) |
| handler-target | handler-code | 66–137 | [2675,5788) |
| handler-query | handler-code | 219–265 | [8715,10330) |

Complete selected functions/contract sections retain conditions and exceptions.
They are not shortened to manufacture budget success. The plan's pre-migration
Section 2, phase-0 defect inventory and execution receipts are inspected context,
not controlling evidence. Its current completed/approved status is source
attribution; the chosen rule body remains a norm, not a new executed test.

## 4. Record, dependency and binding proposal

All records are required active constraints for the exact proposed subject. The
first six use source-attributed `verified_claim` classification for documented
contracts; the route record is an `attributable_observation` of inspected code.
All use verification execution `not_run`; supported means support for the named
contract/observation, not successful CBM refresh or new authenticated approval.
The classification and mapping are proposed for owner review, not inferred runtime
promotion. There is no duplicated decision body in the manifest.

| Record | Primary | Other mandatory evidence |
| --- | --- | --- |
| cbm-canonical-identity | inv-identity | inv-lifecycle, overview-identity |
| cbm-project-owned-cache | inv-cache | plan-owners, cache-lookup |
| cbm-target-invalidation | plan-invalidation | inv-cache, cache-invalidation |
| cbm-refresh-failure | plan-refresh | plan-decision, plan-controls, plan-approval, index-target |
| cbm-errors-not-empty | inv-errors | handler-query |
| cbm-semantic-authority | overview-authority | None beyond primary |
| cbm-query-route | handler-target | handler-query, query-owner, index-target |

Typed prerequisites (each with exact provenance in the manifest): cache ownership
depends on canonical identity; target invalidation depends on cache ownership;
refresh correctness depends on identity, ownership, invalidation and honest error
handling; the public query route depends on identity and honest error handling.
There are eight prerequisite edges and no cycles. Semantic authority is independently
applicable required context, not falsely declared a premise of cache invalidation.
The route is a consumer of the refresh boundary, not its prerequisite. There are
no fabricated VALIDATES, FALSIFIES, approval or supersession acts.

One coverage selector and one applicability selector use the same exact tuple:

```json
{"workspace":"cleanctx-cbm-cache-pilot01","operation":"graph_query","task":"explain_project_cache","from":null,"until":null}
```

Each of seven constraints has a CONSTRAINS binding to that applicability selector.
The requested project A/B scenario is the engineering question, not an invented
entity or a new project selector field. The workspace ID is a PCG corpus scope,
never a CBM slug. Explicit selectors are fully representable in existing v1.
Proposed `enrollment.complete:true` means the owner-reviewed registered universe
for this question; it is inert without approval and is not a claim about every
document, provider version, repository or architecture question.

## 5. Source-identity limitation and isolated profile

Four original working files differ from their pinned Git bytes: invariants,
migration, cache-code and handler-code. Independent byte comparison confirms
their working bytes become equal after CRLF-to-LF conversion. The other three
are byte-identical. **No conversion was applied to the checkout or evidence.**
Existing PCG v1 intentionally does not normalize away this difference, so an
all-current manifest against the original root would fail exact freshness checks.

Proposed solution, contingent on approval: materialize the seven files from local
Git objects in an isolated minimal checkout at the same immutable commit, with
exact blob bytes, before import. No network or new source commit is needed. All
sources remain `check=current` against that snapshot. We do not label current
documents historical merely to bypass source verification.

The isolated root will establish currentness of that pinned snapshot only. It
will not prove that the original working tree remains current after future edits.
The snapshot repository/root and storage do not yet exist; the review profile
therefore is not ready to load until approved preparation creates them.

Proposed paths, relative to the original repository:

- Workspace root: `target/pcg-175/pilot01/workspace`.
- Isolated DB: `target/pcg-175/pilot01/storage/projection.sqlite`.
- Manifest: `target/pcg-175/pilot01/review/manifest.proposed.json`.
- Profile: `target/pcg-175/pilot01/review/profile.proposed.json`.
- Partition: `local-pilot01`; policy: `pcg-pilot01-source-attribution-v1`.
- Owner attribution: `MNasty-pilot01-corpus-review`, proposed local review label.
- History permission: false. Current reports describing earlier events do not
  thereby become historical source revisions.
- Per-pilot ceilings: 512 records/candidates, 1,024 edges/refs, 4,194,304 evidence
  bytes, 32,768 whole-result o200k tokens. These are explicit pilot profile values,
  not product/global defaults. Local profile permissions must remain owner-controlled.

The expected unchanged-source Git plus two working-file observations and excerpts
fit inside the proposed byte ceiling by static size inspection. Whole-MCP-result
token fit has **not** been measured; overflow must block rather than omit mandatory
conditions. No tokenizer/binary was run to predetermine a successful outcome.

## 6. Interpretation issues preserved for review

1. The overview says directory basenames are not canonical project identities;
   CBM-ID-001 additionally permits an exact configured-root basename as an alias
   that must resolve to the canonical slug. Both spans are mandatory. The overview
   explicitly names CBM-ID-001 as its normative detail; this does not authorize
   treating an alias as the canonical ID or inventing a slug. No new automatic
   document-precedence engine is proposed.
2. Older construction/indexing statements in CBM-ID-001 must not be confused with
   MCP state construction. CBM-LIFE-001 is retained. The pinned non-active gate
   also has an additional-root-not-started path that consults primary readiness
   before starting that root. The answer must retain that branch rather than
   promise an unconditional absence of any primary-root startup dependency.
3. The public `graph_query(project=B)` handler selects B before gating/querying.
   It is not a promise that A remains the active project after the handler runs.
   The internal `query_graph_scoped` path resolves B without selecting it, but its
   cache lookup alone is not a freshness gate. The public handler and target gate
   quotes are necessary to avoid asserting that an arbitrary direct helper call
   cannot serve cached data without caller orchestration.
4. Low-level graph queries record failure through `last_error` and may return an
   empty carrier. The MCP handler consumes that error and returns an error result.
   Do not quote an empty carrier as successful empty graph evidence. No production
   repair is authorized by this pilot.

These issues limit simple summaries. PCG v1 can carry their exact evidence and
the reviewer-selected records, but does not itself adjudicate prose, execute code
or prove that source-attributed constraints are obeyed in a live CBM deployment.

## 7. Independent answer/evaluation oracle, frozen before execution

For the registered question, independently assess whether the later answer conveys:

- Canonical target identity is B from the root/project map, not a guessed label or
  active A. Explain the public-handler versus internal-scoped route distinction.
- B owns its memory and disk results; its registered root supplies its disk
  partition. Unknown projects do not acquire invented disk roots. A's entries are
  not invalidated merely because B refreshes or selection changes.
- Dirty generations are per project; edits remain lazy/coalesced. The relevant
  readiness/freshness gate precedes the public graph query's cache access, with
  the pinned startup condition above retained.
- Success captures/advances the relevant indexed generation and invalidates all
  and only B's graph cache before query proceeds. Invalidation itself does not
  change generations, switch active project or contact CBM.
- Refresh failure leaves B dirty, does not advance indexed generation or invalidate
  as though fresh, and propagates error. Retained old cache bytes are not permission
  for stale-success fallback. Existing circuit/readiness policies govern retries;
  no new retry, timeout, file-reindex API, TTL or schema policy is invented.
- Valid empty data is distinct from failure; low-level error carriers require the
  actual public error handling. CBM discovery does not replace WorkspaceIndex/live
  canonical semantic authority.
- Cite the exact revisions/spans supporting these claims, including conditions;
  distinguish norm, historical observations and source inspection from execution.

Expected required record set is exactly the seven IDs in §4, with their eight
prerequisite and seven binding relationships, shared citations for all 18 evidence
IDs, and honest scope/currentness limitations. No hidden optional history is
needed. Completeness is conditional on the registered universe and all actual
v1 checks; a successful tool call alone is insufficient.

Phase C will retain the actual result bytes, generation, required IDs, paths,
citations/hashes, state/coverage, omissions, independently counted whole-result
tokens and elapsed observations. Compare each claim to the source spans without
using the returned summary as its own oracle. Missing/incorrect conditions or
false-complete output fail the usefulness assessment even if transport succeeds.

Assess engineering effort separately: could a reader recover the answer from the
returned cited bundle without searching the three documents and call-path sources
again? Record remaining manual interpretation, especially §6. Do not call the
difference between 331 KB of entire files and the returned envelope token savings;
compare equivalent complete evidence, including metadata. No benchmark/statistical
efficiency claim is planned for one question.

## 8. Preparation results and approval gate

Performed bounded exact Git reads, UTF-8/BOM/hash/span calculations, JSON drafting,
reference and dependency acyclicity checks, and a finite deny-signature inspection
of proposed persisted metadata/excerpts. No signature matched; this is not proof
against every possible secret. Code context was requested through Clean-CTX and
returned skeletons; known bodies and exact Git bytes were then inspected natively.
No PCG import/verify-only, SQL validation, database creation, server, MCP query,
compiler, test, formatter, lint or accepted verification campaign was executed.
The preparation helper under `target/pcg-175/pilot01/` only assembles review data;
it is not a new product ingestion adapter, test or verification framework.

Only this report is an untracked repository deliverable. Drafts and inspection
copies are local ignored pilot artifacts under `target/**`. No existing source or
fixture changed; no Git staging, commit, push, PR or publication occurred.

**Owner review needed:** approve the exact manifest/profile digests, source and
classification mapping, narrow completeness scope, isolated exact-byte snapshot,
storage paths and limits. Approval authorizes preparing that snapshot and Phase B
explicit import, followed by the proposed fresh-session Phase C queries using the
existing PCG tool and binary. Until then, stop here: **NOT ENROLLED / NOT EXECUTED**.
