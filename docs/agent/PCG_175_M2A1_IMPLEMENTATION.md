# PCG-175-M2A1: enrollment and derived persistence

Agent: **Agent MaxHeadRoom**

Authorization: owner-approved corrected [design freeze](PCG_175_DESIGN_FREEZE.md),
M2a-1 only. No staging, commits, pushes, publication or later stage.

Disposition: **PCG-175-M2A1 ACCEPTANCE READY — M2a-1 closed**, per the owner's
2026-11-11 closure direction. Earlier blocked/pending verdicts are historical and
superseded. The focused PCG suite is owner-reported
GREEN. The later one-case CLI lifecycle is directly supported by inspected phase
receipts, and the owner supplied final-source Clippy GREEN and successful build
output in response to the source/binary attribution question. The final evidence
categories and unchanged source/binary hashes are reconciled in
[PCG_175_M2A1_OPERATOR_RECEIPTS.md](PCG_175_M2A1_OPERATOR_RECEIPTS.md).
The later owner rebuild/hash response closes final-source attribution, with
unchanged production fingerprint and an executable matching the operator receipt.
No requested evidence item remains outstanding; the owner directed closure
without another test campaign, operator run or CI gate. Historical observations
below retain their original scope and do not reopen implementation or verification.
No broader verification or owner approval of a later stage is inferred.
The owner determines what runs and when. Local full-suite execution is not an
acceptance blocker; repository-wide CI verification remains separate. No later
stage or publication is authorized by this report.

## A. Source identity and worktree

Starting and ending HEAD: `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`.
Tree: `10d8ecadd0a151ea0638e2a066bd50b40ad022e0`.
Selected branch: `(fix)Performance-Regression-Issues`.
No branch/ref switch, merge, staging, commit or publication occurred. The immutable
approved candidate remains the implementation basis; no later cloud-tip claim is
made. Starting source/proxy diff was empty.

The two predecessor PCG Markdown documents were already untracked at task start
and remain untouched. Changes from this mission are:

- `src/lib.rs`: export PCG maintenance boundary.
- `src/main.rs`: explicit nested CLI dispatch; restrict the existing server
  fallback to genuinely argument-free invocations so malformed `pcg` commands
  cannot accidentally start the server.
- `src/pcg/{mod,model,profile,identity,enrollment,validation,git,security,store,import,cli}.rs`
  and `src/pcg/schema.sql`.
- `src/tests/pcg/{support,enrollment,persistence,security,process}.rs`.
- `verification/pcg-175/{README.md,run-cli.ps1,fixtures/decision.md}`.
- This report.

No other existing source file was edited. All new files remain unstaged. No
runtime database, owner profile or enrolled production manifest was created by
the agent. CBM reported degraded; bounded native reads were used as the documented
fallback. Git object membership was inspected without running application code.

## B. Implementation and ownership

| Responsibility | Actual implementation |
| --- | --- |
| Profile | `profile.rs::Trusted::{load,recheck,manifest}` validates closed v1 JSON, absolute safe paths, exact digest, identity, ceilings, history and policy. Windows profile permission witnesses include owner/group/DACL; Unix witnesses include UID/GID/mode. |
| Manifest | `model.rs` owns closed typed records; `identity.rs::parse` rejects duplicate JSON members/BOM/nonintegral numbers; `enrollment.rs::decode` enforces nullable/optional field distinctions and raw definition/ref ceilings before normalization. |
| Admission semantics | `validation.rs::{validate,relationships,acyclic,conflicts}` checks IDs, selectors/intervals, evidence refs, edge endpoint roles/provenance, gate shape, supersession scope/interval union, cycles and overlapping conflicting controlling records. This is whole-generation admission, not query closure. |
| Source witnesses | `git.rs::{sources,current}` uses literal local Git arguments, exact commit/blob membership and SHA-256, disables lazy fetch/replacement objects, rejects unsafe modes and verifies current files at both admission boundaries. Physical file identity deduplicates hardlinks. No newline conversion. |
| Evidence and revisions | `enrollment.rs::normalize` checks scalar-aligned exact spans/hashes/headings, then constructs source/evidence/record/selector/relationship hashes and deterministic typed rows. `identity.rs::{canonical,hash,set}` owns canonical encoding. |
| SQLite | `schema.sql` is the approved v1 DDL, including required-ref/current-source indexes. `store.rs::{writer,begin,initialize,insert,publish,commit}` owns isolated WAL/FK/FULL-sync transactions with zero busy timeout/retry. |
| Idempotency/CAS | `store.rs::{head,existing,preserve_decisions}` and `import.rs::finish` compare the preparation head, preserve controlling classification/subject/disposition, check all rows on repeated keys and publish only after complete admission. Published rows are never updated. |
| Rebuild | `store.rs::clear` plus insertion/head publication occurs within one transaction. Unknown/nonempty foreign schemas are refused. Explicit first admission can initialize a truly empty file left by an interrupted initial transaction; it never repairs a nonempty schema. |
| Reopen | `store.rs::{schema,reader,expected_head}` checks exact schema/index definitions and header identity. It does not scan constituent rows or hash the corpus at ordinary open. Preparation's old-head lookup is only a CAS witness, not retrieval approval. |
| Full audit | `import.rs::finish(audit=true)` reconstructs the approved generation from authoritative bytes and uses `store.rs::compare` to verify exact full membership/typed rows in one read-only pinned SQLite transaction; no repair/publication. |
| Security | `security.rs` rejects path escapes/reparse points and applies the frozen finite deny signatures to decoded metadata strings and exact quotes before persistence/output. Errors are fixed non-disclosing codes. No source text chooses SQL, commands or roles. |
| CLI | `main.rs::Cli::Pcg` -> `cli.rs::run` -> `import.rs::{prepare,finish}` -> isolated store -> fixed JSON maintenance receipt. `--rebuild` and `--verify-only` conflict. No server-startup PCG call exists. |

Source approval and execution categories remain source-attributed. An importer
does not authenticate an approver, rerun a receipt, inspect prose to infer success
or promote a descriptive USES_EVIDENCE edge. `executed_artifact` remains a reviewed
source classification requiring the actual artifact's enrollment; it is never
inferred from a report naming a receipt. The owner must review that classification.

Set permutation preserves constituent semantic revisions. A changed raw manifest,
including reordered arrays, necessarily changes the approved digest and generation
identity under the frozen formula. The test explicitly distinguishes these facts.
Same-ID nonsemantic witness refresh remains owner-reviewed; prose equivalence is
not guessed. Substantive controlling metadata changes require a successor ID.

## C. Acceptance evidence and execution handoff

Exactly 17 test cases are registered through normal `#[path]` entries in
`src/pcg/mod.rs`: five enrollment/identity, three security, six persistence/work,
and three process entries (one is the subprocess worker). Table-driven checks
cover duplicate keys/IDs, unknown/forged fields, parser/schema/selector errors,
unsafe paths, wrong Git objects/spans/hashes, required refs/provenance, cycles,
supersession and overlapping conflicts.

Storage controls cover atomic admission, exact idempotency/conflicting keys,
read-only access/FKs, late source failure during import and rebuild, preserved
heads, independent writer contention and stale CAS, damaged headers and removed
bindings, rebuild identity, source attribution, shared evidence and bounded work.
Process controls kill only temporary fixture subprocesses before/after the shared
store commit boundary and reopen/audit in a fresh process. They test storage
durability; they do not claim the full public CLI path was executed.

The owner reported the focused PCG suite **GREEN**; detailed execution counts
were not supplied. The agent did not execute the tests.
The normal worker entry exits without a child mode; parent cases exercise its
actual child assertions. There are no ignored acceptance tests. Expected gate
cost is small Git/SQLite fixtures plus two process controls; actual duration is
unmeasured. No new CI job/suite layer is introduced. Larger CLI population runs
are operator-only and are not tracked Rust test results.

The earlier full-suite local handoff was inappropriate and is withdrawn. The
owner controls execution and scheduling; this report issues no further run
instructions. Existing results are evidence only for their stated scope and
source revision. No CI-wide GREEN is inferred from focused results.

Performed bounded static inspection: HEAD/tree/worktree checks, protected-path
diff review, strict byte-level UTF-8/BOM/line inspection, `git diff --check` and
PowerShell AST parsing without executing the operator script. Git reports normal
CRLF-to-LF checkout notices for the two pre-existing integration files; these are
not compiler warnings or executable acceptance. No tests/builds/check scripts,
formatter, benchmark, migrations, application binary or server were launched.

## D. Performance accounting

No timing, DB/WAL size or performance result has been measured by the agent.
Instrumented maintenance reports contain:

- Manifest validation, reopen metadata, source verification, evidence/generation,
  SQLite insertion/comparison or rebuild, commit, full audit and pre-output total
  durations; separate evidence-hash and generation-construction microseconds.
- Actual corpus bytes read (including both sidecar reads), Git bytes, current-file
  bytes over both boundaries, distinct physical current-source counts per boundary,
  normalized definitions/assertions/refs and evidence validation counts.
- Committed database/WAL footprints, sampled with the writer still open; no peak
  usage or crash-durable receipt is implied by these diagnostic measurements.

Tracked N/2N/4N fanout cases assert one shared evidence validation and unchanged
Git-source bytes as record counts grow. The operator driver additionally records
fresh-process CLI durations at 8/16/32 records by default, exact profiles/manifests,
binary digest and import/reimport/rebuild/audit output under `target/**`.

Scope: one complete import generation or one explicit audit. Parsing and source
byte work is bounded by profile ceilings. Graph indexes/maps are operation-local;
there is no persistent validation cache. Cycles are processed in O(V+E); conflict
checking can be quadratic within a same-subject/role group, and supersession scope
union work grows with its declared selectors. These necessary admission comparisons,
row sorting and SQL insertion are included in enclosing operation timings, not
advertised as constant work. Corpus-wide current witnesses are observed twice even
for small selected evidence. Fixture counts are correctness/scaling controls, not
enterprise efficiency or latency evidence.

## E. Compatibility, limitations and consumer audit

Existing MCP dispatch, registry, schemas, annotations, source SQLite, canonical IR,
replay, delta, source caches, publication, tokenizer and compression implementations
are untouched. No new public PCG tool, source hydration, automatic import/injection,
runtime ledger or approval write exists. Runtime compatibility is still unverified;
source nonmutation is not a legacy-response test result.

Boundary observations do not detect unobserved change-and-restore, freeze files
through commit or create an atomic multi-file snapshot. Read-only header verification
does not detect off-path row loss/tampering; explicit audit checks the full admitted
generation. No continuous freshness/storage-integrity or enterprise ACL claim exists.
Trusted local OS account/profile assumptions remain the approved deployment limit.
Windows file identity uses the documented [FileIdInfo API](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex);
the FFI paths still require actual Windows execution in the focused gates.

| Canonical question / consumer | Present route / authority / lifecycle / coverage |
| --- | --- |
| What exact evidence did the owner enroll? | Explicit CLI import; exact raw approval digest, Git/span identities and complete immutable generation. Source-attributed, not authenticated approval. |
| Did that complete generation survive exit/crash? | Store/head metadata and explicit CLI audit; process controls are part of the owner-reported green focused suite. Detailed receipts were not supplied. |
| Does the stored projection equal approved authoritative bytes? | `--verify-only` full pinned membership comparison; modified/absent required rows refuse; no repair fallback. Source observations remain boundary-only. |
| Did replacement fail without changing the admitted head? | Transaction/CAS/rebuild controls belong to the owner-reported green focused suite. Old/new lifecycle assertions remain source-owned. |
| Which constraints apply to an edit, why, and with what completeness? | **Not reachable in M2a-1**. Query applicability/closure/impact/provenance, exact budget projection and MCP exposure remain later owner gates. No complete-context answer is claimed. |

## F. Next-stage boundary

M2a-2 can inspect the implemented closed manifest types, hash definitions, immutable
generation rows, composite identities, approved schema and read-only metadata owner.
It must not treat an import/audit receipt as query applicability or completeness.
Actual execution evidence and owner acceptance remain prerequisites to advancing.

Evidence limits: no final Clippy outcome after the two lint corrections, detailed
operator receipt or complete CI result has been supplied. Earlier focused/build/
operator greens are retained as user-reported evidence. No real A-E enrollment or
owner acceptance is inferred. No later-stage evaluator, token projection,
canonical-entity adapter, MCP registration or Claude pilot has begun. The owner
decides acceptance and any further execution; the local full suite is not a blocker.

### Verification follow-up

The owner reported `projection_corruption_detected` failing during fixture setup:
the generation-key mutation was rejected by SQLite foreign-key enforcement
(extended code 787), before any corruption-detection assertion. The test now
explicitly disables FKs on its out-of-band tampering connection, verifies that
setting, and restores enforcement after restoring the original key. Production
configuration and detection assertions are unchanged. This setup failure is not
behavioral RED evidence.

The owner subsequently reported **GREEN** for the handed-off focused command:
`cargo test --all-features --lib pcg::persistence_tests::projection_corruption_detected -- --exact`.
This is user-reported tracked-test evidence, not an agent-executed result or a
full-suite/final-gate PASS.

The owner then reported **GREEN** for `cargo test --all-features --lib pcg::`.
The focused PCG tracked suite is therefore user-reported green; no detailed test
count/output was supplied.

Following the build/operator handoff, the owner reported **GREEN** again. Record
the all-feature binary build and operator CLI verification as user-reported green;
no receipt path or detailed output was supplied. Whether this also covers every
command in the authoritative Final Verification Gate remains unconfirmed.
The owner subsequently reported the all-target/all-feature Clippy gate failing
on `clippy::unused_unit` in `security.rs::no_links`. The redundant trailing unit
expression was removed from the permitted-missing-leaf match arm without changing
its behavior or suppressing the lint. Clippy rerun and full final-gate completion
were not subsequently reported. Earlier test/operator greens precede this source
edit and do not constitute a new execution of the revised source. This records
an evidence boundary, not a request to rerun anything.

The next owner-run Clippy attempt reported `clippy::nonminimal_bool` in the bearer
signature predicate. Applied the equivalent De Morgan expression without changing
the denial condition or adding a lint suppression. No subsequent result was
supplied; no new GREEN is claimed and no further execution is prescribed.

### Continued static review

Inspected the two local `target/pcg-175/m2a1-*` directories: each contains only
`n-8/repo/decision.md`, with no Git repository, projection or operator receipt.
These artifacts do not corroborate completed CLI verification. The earlier owner
GREEN remains an attributed report; no performance numbers are reconstructed
from incomplete directories.

Static review identified a driver command-resolution defect: the PowerShell
helper `Git` shadowed the native `git` application in its own body. Renamed it
`Invoke-FixtureGit` and explicitly resolved the executable with application-only
command lookup. All fixture calls use that helper. This changes only the operator
definition; production Rust is untouched. The corrected driver has not been run.

The existing workspace log is incomplete and contains failure markers outside
the PCG module, without a final result or detailed panic summaries. This does not
establish their causes or their relationship to this campaign. No full-suite
completion is needed for continuing M2a-1 review, and no unrelated repair or later
stage is initiated from these markers. The owner controls all execution decisions.

```architecture-graph
node: PCG-175-M2A1
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Owner-enrolled evidence import and derived persistence
track: REPOSITORY
artifact: docs/agent/PCG_175_M2A1_IMPLEMENTATION.md
implements:
  - AUTH.PCG_ENROLLED_V1
note: Historical implementation and scoped reported verification; B3 does not rerun or independently certify these witnesses.
```
