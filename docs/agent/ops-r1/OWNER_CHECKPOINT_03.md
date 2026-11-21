# OPS-R1 Disposition 03 — exact source review and safe application plan

Agent MaxHeadRoom. 2026-11-20. **Preparation only; implementation remains unapplied.**
No potentially hanging execution, Cargo build, CI mutation, production repair,
owner authorization, commit or push was performed. This is agent source review,
not independent certification or owner approval of the patch bytes.

## 1. Exact Tranche-1 review evidence

[Current patch hashes/review findings](../../../verification/ops-r1/review-03/patch-review.json)
and [per-file baseline/candidate identities](../../../verification/ops-r1/proposals/tranche-1/proposal-index.json)
identify the exact review candidate. The [pre-review index](../../../verification/ops-r1/review-03/pre-review-index.json)
preserves the earlier unapplied diffs byte-for-byte; no earlier execution is
relabeled as verification of these revised proposals.

| Candidate diff | Source review result | Verification still required |
| --- | --- | --- |
| membership.patch | Ten allfeature identities, distinct 1-test C# and 2-test TS selections; three retained behavioral witnesses. Existing 56 groups/18 measurement pairs remain. | Actual post-application exact membership and source-currentness checks. |
| verifier.patch | Source-reviewed mandatory count/ignore/outcome/hash checks; actual Cargo exit is now required in isolated mode. Windows target separators normalized without changing identity. | Adversarial fixture execution; parsing candidate Python is not behavioral proof. |
| fixtures.patch | Existing fixture test extended with missing manifest, changed witness and six isolated-mode valid/negative cases; original cases retained. | Compile/execute selected fixture test. No fixture result is claimed yet. |
| ci.patch | One consolidated allfeature runtime run retained; two distinct feature-mode builds and three selected outcomes, actual verifier calls and retained artifacts. | Approved CI execution and phase-cost/membership comparison. |
| governance.patch | Separate proposed final-verification obligation and protected golden-membership component. Expected data changes require actual B11 review rather than a silent expected-hash update. | Exact authority amendment approval, fresh descriptor/grant/admission after authorized staging; no protected change applied. |

The isolated verifier's missing-exit flaw was corrected before application.
Source hashes prevent unnoticed witness-byte drift against the reviewed manifest;
they do not prove assertions meaningful. Existing historical RED/negative controls
and the exact consumer mapping in checkpoints 01/02 remain part of each obligation.
The proposed B11 binding protects the source-owned membership/configuration data at
document admission/publication. The CI hash check alone does not authenticate owner
approval, and a PASS is not proof of an author performing the governance workflow.

The proposed extra cost remains ten offline membership checks, three source hash
checks, eight additional verifier fixture scenarios inside the existing Rust test,
two feature builds and three isolated runtime outcomes. No extra allfeature test
suite, repeated timing study, generic runner or measured time estimate is claimed.
The governance proposal adds one protected source selection only after approval;
it does not modify the currently accepted 342-selection state in this checkout.

## 2. Exact P0 source/instrumentation review

[P0 source proposal](../../../verification/ops-r1/proposals/p0/proposal-index.json)
pins eleven changed/new files, including actual src/tests paths and normal module
registration. Both [fixture/registration](../../../verification/ops-r1/proposals/p0/registration-and-tests.patch)
and [test-only observation](../../../verification/ops-r1/proposals/p0/test-only-observations.patch)
diffs pass read-only application checks. Rust remains uncompiled/unformatted.

Review confirmed one hook per named stage, except two intentional capture-pipeline
Query::new sites. Fifteen observations span the declared work; compiler workers use
the same isolated-child atomics. Hooks and sampler are gated on test + Angular;
ordinary no-capture execution uses one-unit fixtures and no active sampler/counters.
No product authority, source selection, publication decision, cache, retry or
timeout semantics are changed by these proposed observations. Bounded child
execution, not an ignored test, is the scale-diagnostic path.

Review corrections preserved in the current candidate:

- Pin tracked src/build inputs, including query/fixture assets, not only Rust/TOML.
  Rust/TOML/lock pins normalize LF explicitly; other data is byte-exact.
- Record a complete preapplication baseline digest as well as the after-application
  digest. No unreviewed combined Tranche/P0 source state is accepted.
- Use a separate wall watchdog, independent of memory polling, and retain the
  five-second child wait. Failure to terminate is an error, never a diagnostic PASS.
- Identify the monitored process as the spawned Rust libtest child, not Claude or
  the production server. Windows WorkingSetSize and Linux resident-page readings
  are named; a fast-exit cell without a sample does not establish peak memory.
- Recheck source and executable identities after capture; hash stage output and
  preserve copied fixture inputs plus their content identities outside source roots.

The launcher, monitoring APIs, watchdog, cleanup and new Rust fixture have not been
runtime-exercised. These are implementation proposals with static review, not a
resource safety certification or a behavioral RED. A 60-second/512-MiB stop remains
an external capture outcome, not a product rule or automatic leak/work-budget proof.

## 3. Confirmed source and authority revisions

Checkout HEAD remains 0dd8d7503af3116241ad7c1e23c293b1d5849cc1.
No actual src/, scripts/ or .github/workflows/ implementation diff is applied.
Current source/authority observations and exact helper hashes are in
[review revisions](../../../verification/ops-r1/review-03/observed-revisions.json).
The existing 28-source authority snapshot is retained as its dated observation,
not silently overwritten to imply a different historical revision.

The P0 baseline tree digest was independently checked through the launcher in
`--verify-baseline-only` mode. That mode performs source hashing and launches no
test child. The matching digest and candidate digest are in the P0 index/review
record. A source pin confirms source identity, not successful compilation.

## 4. Fresh actual B11 evidence

Latest completed actual operator capture reviewed:
target/ops-r1/checkpoint-20261120-113420, valid descriptor digest
2eadec758ccfef3eb1751dd4c41a7b8146d07c0a918ca5d6eb2b7e097fe09bac,
actual normal admission REFUSED_OR_INCOMPLETE with only
E_GOVERNANCE_AUTHORIZATION_REQUIRED. No grant or publication occurred.
The [versioned refusal custody index](../../../verification/ops-r1/review-03/b11-refusal-index.json)
pins the exact descriptor, one normal refusal payload, candidate pins and guard
outputs. The other equivalent refusal payloads remain in their original capture.

That actual fresh call returned the same digest as the previous capture. This
is not evidence it failed to run: the descriptor binds protected state/membership,
not every unprotected investigation text byte. Preserve engine-generated digests;
do not invent a new one merely because a candidate report changed. Candidate source
pins still identify which unprotected revision was reviewed.

This Disposition-03 review packet has since changed. Obtain another actual
describe/admission/refusal through the existing operator checkpoint, then review
the descriptor and source pins. Do not infer authorization from the owner message.
Current membership proposal is still OPS-001 only. Future governance.patch, if
authorized/applied, changes the final-verification obligation and protected
selection and therefore needs its own actual reviewed B11 candidate.

## 5. Proposed safe application sequence

**P0 first, separately pinned:**

1. Owner reviews the exact two P0 patch hashes and helper definitions in the review
   record; explicitly authorizes those bytes and the no-run build command.
2. Verify baseline tree and each affected file's baseline identity. Run read-only
   application checks, then apply only those two approved P0 diffs atomically.
   Do not apply Tranche-1 or a production repair alongside them.
3. Verify the resulting source digest with the launcher in --verify-sources-only
   mode; if it differs from the reviewed candidate, stop before building.
4. Operator performs the exact approved build below. Return its receipt and stop.
5. Owner separately approves the first bounded owner-1 execution using that exact
   receipt/binary. Stop on any abnormal result before considering additional cells.
6. Freeze the original diagnostic witness/source/receipt before any production
   correction. A legitimate architectural failure predicate still needs review;
   the current observational tests do not manufacture one from arbitrary budgets.

**Tranche-1 remains independently gated:** review exact five diffs, including the
new governance diff. Stage approved authority/data amendments through actual B11
before declaring their obligations admitted/enforced. Only after exact CI/verifier/
fixture authorization install the implementation and run focused approved checks.
If P0 and Tranche source changes coexist, produce a reviewed combined source pin
and new build receipt; neither old digest can be assumed valid. The complete manual
audit is not a prerequisite for this otherwise justified focused tranche.

## 6. Exact operator build command and expected output — held

After exact P0 source/build approval and application only:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\p0\invoke-build-capture.ps1
```

It verifies the candidate digest then runs exactly:
`cargo test --lib --all-features --no-run --message-format=json`.
It writes target/ops-r1/p0-build-reviewed/{build.jsonl,build-stderr.txt,build-receipt.json},
refuses to overwrite evidence, captures exactly one root lib-test executable/hash,
checks source identity after build and rejects warnings/build failure. No tests
execute. Return that receipt; successful build is not execution permission.

The first bounded command remains the exact owner-1 command in checkpoint 02.
Do not run it yet. It requires separate approval after the actual build receipt,
with 60-second wall / 512-MiB observed working-set / five-second wait safeguards.

## 7. Remaining approvals and risk

| Item | Required next action |
| --- | --- |
| Current design/mission membership | Fresh actual checkpoint; return protected-membership descriptor and stop if authorization required |
| P0 exact test-only patch/helper bytes and no-run build | Owner source/build approval; not implied by Disposition 03 |
| First bounded child | Separate owner approval after reviewing real build receipt, executable and source pins |
| Tranche CI/verifier/fixture/governance bytes | Exact owner review and approval; prospective authority/data binding needs actual B11 admission |
| Shared production correction | Preserve original witness and diagnose; return smallest exact preserving repair for approval |
| Broader A01–A13 exposure | Continue independent high-risk evidence; none is silently waived or universally called the hang cause |

Static review does not close runtime-validation gaps. The packet is ready for
exact owner review, not application, potentially hanging execution or publication.
