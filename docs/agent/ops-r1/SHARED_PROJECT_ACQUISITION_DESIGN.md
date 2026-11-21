# OPS-R1 disposition 08: shared TypeScript acquisition design

Agent: **Agent MaxHeadRoom**. Date: 2026-11-20.
Standing: proposed design, not implementation approval or verified repair.
Owner Disposition 08 accepts measured amplification and authorizes this design only.

## Evidence and scope

Baseline HEAD: `0dd8d7503af3116241ad7c1e23c293b1d5849cc1`.
Diagnostic source SHA-256: `393f7d7b0ba31dbe0f78cceb43539dccdb120598a154af8c7a7c415b33a661c1`.
Executable SHA-256: `34d2a9ebd67ce949fd27cd373ba013996ba7eb5a9e80573ff5428b995cf89222`.
Exact build and owner-1/4/16 receipts, logs, stages and fixture bytes are retained
in `verification/ops-r1/shared-project-design/pre-repair-evidence.zip`, with member
hashes in `evidence-index.json` beside it. Archive SHA-256:
`a287e34d1551ca7763fc9ddf278f944d794dae40bb0e0f81de85de99aa756674`.
Original capture directories remain unchanged under `target/ops-r1/`.
Executable SHA-256: `34d2a9ebd67ce949fd27cd373ba013996ba7eb5a9e80573ff5428b995cf89222`.

Pre-repair receipts, unchanged under `target/ops-r1/`:

- `p0-owner-1-20261120-130920/receipt.json`
- `p0-owner-4-20261120-131545/receipt.json`
- `p0-owner-16-20261120-131747/receipt.json`

Each selected `mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_absent_route_observation`
exactly once, passed, and returned an empty lower-bound answer. These are successful
observational tests, not executions of a future failing fitness assertion.

| Owners O / distinct project units P | 1 / 1 | 4 / 4 | 16 / 16 |
|---|---:|---:|---:|
| Acquisitions | 1 | 4 | 16 |
| Cumulative unit visits | 1 | 16 | 256 |
| Loop iterations / host misses | 2 / 2 | 20 / 20 | 272 / 272 |
| Witness checks | 408 | 1,980 | 16,188 |
| Owner compilations / publications | 1 / 1 | 4 / 4 | 16 / 16 |
| Query compilations | 1 | 1 | 1 |

Measured acquisition work is O acquisitions and O*P unit visits, with P=O here.
The experiment does not separate owner growth from project growth, prove a leak,
or explain the complete field hang. Fixed-job cells remain unexecuted.

## Preflight and controlling constraints

Reuse the actual admission/acquisition at
`target/ops-r1/admission-20261120-123814/receipt.json`, generation
`abc79b217f2f9fb21a4d6f2cf4dc79ca30cf119f23a3944a85a668111a5eed15`.
That acquisition predates this report and is not fresh admission of it.
Connected `pcg_knowledge_v1` was unavailable during this design turn. No runtime
was started to replace it. Fresh reconciliation/admission remains operator-owned.
Applicable constraints are AGENTS.md AF-PERF-001/002 and architectural approval,
pcg-authoring.md, source-owned authority/currentness, and existing Angular
resolver discovery, refusal and publication contracts. No governing contract is
amended here. Relevant new graph coverage is not claimed complete.

## Actual boundary and critical source-overlay constraint

`hydration/publication.rs::compile_candidates_for` prepares jobs and tickets,
calls `SemanticCompilerExecutor::compile`, then validates results before publishing.
`semantic_compilation/compile.rs::compile_source_artifact` invokes
`TypeScriptAuthority::capture_with_demands` or its capture wrappers for each job.
`typescript_resolution/capture.rs` constructs a fresh Resolver and seeds only
that owner's supplied source. `Host::seed` refuses replacement of consumed bytes.
`program.rs::program_units` caches within that private Resolver only.

Therefore equal config paths are insufficient: one job can supply bytes different
from disk while another job reads that same file from disk. Preseeding all jobs
into one shared resolver would change current authority semantics. Do not do it.
Likewise, sharing a mutable Resolver would mix owner-local project context,
recursion guards, demanded occurrences, and evidence. Do not share that object.

## Proposed acquisition equivalence contract

Two jobs may share only if their complete consumed acquisition inputs are equal:

1. Same authoritative physical project/config identity AND route/alias evidence;
   preserve explicit selection versus inferred selection and membership proof.
2. Same effective options and origin-bearing config/extends/reference closure;
   same compiler version, library overrides, automatic types and resolution mode.
3. Same authorized roots and route admission requirements, including symlink
   evidence; initially require exact ordered root inputs rather than broadening them.
4. Same authoritative source snapshot for every consumed unit, including each
   job's supplied source. Sharing eligibility requires proof that the supplied
   owner bytes equal the acquired authoritative bytes; a divergent overlay stays
   isolated. Do not compare source text alone as authority.
5. Same native package/export identity, package content and lookup witnesses,
   ambient/type-root/library inputs, absence and directory inventory evidence.
6. Same acquisition options, source-only/project mode, project ownership and
   relevant demand policy. Owner-specific demands stay owner-specific; initially
   partition by identical declaration-field/API demand sets.

Equality is equality of the complete evidence tuple, not merely a digest or path.
Hashes index a candidate; existing exact identity and currentness predicates
establish eligibility. Unknown equality means no sharing, not inferred equality.
This is a conservative equivalence relation on established immutable acquisition
inputs; discovery of that tuple must not itself repeat complete project traversal.
Use cheap existing project selection for provisional grouping, acquire once, then
qualify every job against the resulting evidence before reuse. No new blanket
source manifest scan per job is permitted.

## Ownership, lifetime and proposed minimal source diff

One stack-owned acquisition context per executor `compile` invocation; drop it
after results are built. No field on McpState, persistent cache, serialization
change, cross-operation reuse or change to worker-count policy.

The following is a source-diff plan, not an applicable patch or compiled API:

| Source | Minimal proposed change |
|---|---|
| `semantic_compilation/executor.rs` | Prepare equivalent TS groups once within compile; pass borrowed immutable acquisition to jobs; retain result order and owner panic/error isolation. |
| `semantic_compilation/compile.rs` | Add optional borrowed TS acquisition input to internal request; owner capture consumes it only after qualification. |
| `typescript_resolution/program.rs` | Extract successful immutable program units plus all acquisition evidence into an internal operation-owned result; retain existing standalone path. |
| `typescript_resolution/capture.rs` | Keep private owner Resolver and source seed; attach qualified acquired program/evidence without reacquiring or cloning the complete project per owner. |
| `typescript_resolution/resolver.rs` and `host.rs` | Minimal borrowed immutable acquisition access; owner mutation and evidence attribution remain local. Exact lifetime changes require review. |
| `mcp/tool_helpers.rs` | Supply no shared input on direct calls; preserve standalone behavior. |
| `src/tests/ir/` and `src/tests/mcp/` | Tracked deterministic work and semantic-preservation regressions, registered through existing path convention. |

Share immutable parsed units/evidence by borrowing or existing Arc ownership;
do not introduce a general cache framework, a global lock, or N full-project
clones. Owner outputs retain applicable project dependencies/witnesses in the
existing representation. Preserve all currentness checks initially; deduplicating
witness validation or durable evidence is a separate proposal.

## Proposed failure and invalidation decisions

| Event | Required proposed behavior |
|---|---|
| Acquisition error | Never install success. Preserve existing disposition/error and owner isolation; no new retries or sticky cross-operation failure. |
| Partial acquisition | Not eligible as successful complete shared acquisition. Preserve qualified partial/refusal behavior through existing standalone path where applicable. |
| Concurrent compile invocations | Independent contexts. No cross-request coordination or authority borrowing. |
| Parallel jobs in one invocation | Immutable successful group input; private owner resolvers. No lock around all semantic compilation. |
| Source changes during acquisition | Existing witness/currentness checks disqualify stale capture; no mixed snapshot admitted as current. |
| Different owner supplied bytes | Isolate that job; do not overwrite shared bytes or seed siblings globally. |
| Config/package/root-route changes | Invalidate qualification; retain applicable failure/currentness outcome. No reuse across changed authority tuple. |
| Owner retirement/replacement | Existing source identity and publication ticket checks remain controlling; no global relationship deletion. |
| Stale acquired snapshot | Refuse current publication using existing validation; sharing is not proof of currentness. |
| Publication | Preserve existing whole-batch prechecks, per-owner tickets, provenance and admission; no streaming-publication change. |
| Cancellation | Existing operation behavior only; drop context after existing work ends. No new cancellation semantics. |
| Retry | Existing caller policy only; a later operation acquires afresh. No retry loop added. |

Acquisition failures that currently yield owner-specific qualified authority must
not become new whole-operation hard errors. Exact error propagation is a required
implementation review point, not an approved generic fail-all policy.

## Fitness predicate and historical detection

For a stable operation with K distinct proven-equivalent input groups, successful
shared project acquisitions must equal K, and complete-project traversals must
equal the sum of units acquired for those groups, independent of owner multiplicity.
Owner-specific compilation/publication remains proportional to eligible owners.
Additional legitimate dependency work is counted separately, not hidden in this
predicate. Count group resolution attempts, successful acquisitions and actual
unit traversals separately; the current project counter alone is insufficient.

For the present one-project fixtures expected successful acquisitions are 1/1/1
and complete-project visits 1/4/16, with owner compilations/publications 1/4/16.
The historical counts 1/4/16 and 1/16/256 violate that proposed predicate at 4/16.
That is static counterevidence, NOT an executed RED of a future tracked test.
Future tracked test must run on defective and corrected implementations and
preserve exact public lower-bound semantics, provenance and currentness behavior.

Negative controls must distinguish configs, roots/authorization routes, native
package identities, source revisions and divergent supplied overlays; mutation,
acquisition failure, partial evidence and retirement must retain refusal behavior.
Include multiple jobs sharing one project and multiple genuinely distinct projects.
No time/RSS threshold, constant total compilation budget, or blanket witness-check
reduction is part of the predicate. No new golden runner or duplicate CI pass.

## Cross-consumer impact

These labels describe proposed reach, not post-repair verification:

| Consumer | Classification and source basis |
|---|---|
| Resolver registration queries | Exposed: resolver_registrations.rs calls compile_candidates_for; three pre-repair observations protect only the recorded empty-result fixture. |
| Bootstrap queries | Exposed: hydration/bootstrap.rs uses the shared publication pipeline; no corrected verification yet. |
| Ordinary TS semantic hydration | Exposed: hydration publication invokes executor; owner/currentness preservation required. |
| Executor batch compilation | Exposed: direct boundary; multiple project and non-TS controls required. |
| Explicit-file semantic tools | Unaffected by executor-only reuse: tool_helpers.rs calls compile_source_artifact directly; exposed to signature changes and must retain standalone semantics. |
| C# reconciliation | Unaffected acquisition semantics: reconciliation.rs also calls executor; keep non-TS jobs unchanged. |
| Capture pipeline query reuse | Unaffected: compression/capture_pipeline.rs is not the proposed project-acquisition owner. |
| Other indirect explicit/batch tool flows | Unverified until exact caller chains are reconciled; no resolver-only protection claim. |

## Owner decisions and next gates

Request review of this conservative eligibility policy, immutable operation-local
ownership, failure isolation and unchanged publication boundary before producing
an implementation patch. Non-current supplied overlays remain independent;
support for sharing divergent overlays would require another design, not a fallback.
Borrowed program units must include all consumer-required evidence; if owner-specific
acquisition changes program membership, split groups or return the exact blocker.
Do not promise one acquisition for jobs whose equivalence cannot be established.

Remaining gates: reviewed concrete patch/source identity; separately authorized
build/test invocations; actual historical RED and corrected GREEN; negative controls;
applicable caller checks; fresh graph reconciliation/admission before publication.
No protected contract amendment is proposed. Any necessary protected change follows
B11 owner authorization. No implementation, build, test, CI, grant, commit, push
or publication was performed under disposition 08.
