# PCG-175-M2A1-ACCEPTANCE: final evidence reconciliation

Agent: **Agent MaxHeadRoom**

Current follow-up: the operator-definition correction and completed single-case
CLI lifecycle are now corroborated in
[PCG_175_M2A1_OPERATOR_RECEIPTS.md](PCG_175_M2A1_OPERATOR_RECEIPTS.md).
**Current owner disposition (2026-11-11): PCG-175-M2A1 ACCEPTANCE READY — M2a-1
closed.** The owner directed closure following the November 11 reconciliation,
without another test campaign, operator run or CI gate. No identified
architectural, persistence or security defect requires another implementation
cycle. The earlier ACCEPTANCE BLOCKED verdict and EVIDENCE PENDING dispositions
are historical and superseded. Evidence categories remain unchanged: focused
tests/lint and build attribution are owner-reported; the successful four-phase
CLI lifecycle is artifact-backed. The original inventory is preserved.

No M2a-2 development begins under this closure. Its separately authorized scope
would consume the existing immutable projection for deterministic applicability,
evidence closure, provenance and completeness semantics, without reopening
enrollment or persistence. No commit, push or publication is authorized.

Date: 2026-11-10 (America/New_York).
Authority: owner-directed static acceptance review only. No implementation,
new tests, executable verification, publication or M2a-2.

Historical disposition at the static review: **M2A1 ACCEPTANCE BLOCKED**, limited to the operator evidence
definition: it cannot currently supply the requested single-case run with
retained explicit phase exit statuses. This is not a demonstrated failure of
the Rust persistence implementation. Final-source lint and corroborated CLI
lifecycle evidence also remain missing. Broad unrun checks are not failures.

## A. Source identity

Current local branch: `(fix)Performance-Regression-Issues`.
HEAD: `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`.
HEAD tree: `10d8ecadd0a151ea0638e2a066bd50b40ad022e0`.
These values were read during this review, not assumed from the predecessor.
No claim is made about the live remote branch tip.

Tracked modifications are exactly `src/lib.rs` and `src/main.rs`: one module
export, explicit PCG CLI dispatch and an argument-free guard on the existing
server fallback (11 additions/1 deletion in the tracked source diff). All PCG
production modules, tests, operator definitions and three predecessor reports
remain untracked. No other existing tracked source is changed. The full evaluated
inventory and exact byte hashes are in the appendix.

Evaluated pre-review worktree fingerprint:
`56fcdb000e9d45cd3c72b47240cf4a9fbed767a4d087ec2c25a4b3059dfd1bc5`.
Definition: sort the 25 modified/untracked paths in the appendix by ordinal path;
encode each as `lowercase-SHA256`, two ASCII spaces, slash-separated path, LF;
SHA-256 the concatenation as UTF-8 without BOM, including the final LF. Hashes
cover actual working bytes, including their actual newline representation.
This new review report is excluded to avoid a self-referential fingerprint.
The existing implementation report was not edited during this mission.

No branch switch/reset/rebase/merge/clean/staging or production edit occurred.

## B. Evidence ledger

Historical source fingerprints were not recorded alongside the owner messages.
The phase descriptions below identify chronological attribution only; they do
not manufacture exact historical content hashes. Current hashes cannot be
retroactively assigned to earlier executions.

| Verification | Source revision | Execution authority | Reported outcome/category | Direct artifact | Confidence |
| --- | --- | --- | --- | --- | --- |
| `projection_corruption_detected` focused test | After fixture-only FK injection correction; before both later production lint edits; historical hash unavailable | Owner execution/report | **OWNER-REPORTED GREEN**; **SUPERSEDED** as proof of final source | Owner message only; no full output | Success attributed, not independently certified |
| `cargo test --all-features --lib pcg::` | Same historical phase; exact hash unavailable | Owner execution/report | **OWNER-REPORTED GREEN**; **SUPERSEDED** for final source | Owner message only | Valid scoped historical report; counts/duration unavailable |
| All-feature binary build | Build/operator handoff phase before later lint edits; exact source binding unavailable | Owner message after combined handoff | **OWNER-REPORTED GREEN** | Current executable exists, but no complete build receipt tying it to source | Reported success; current-source binding unverified |
| Operator CLI lifecycle | Earlier driver with `Git` helper, before rename; binary/source/run binding unavailable | Same owner message after combined handoff | **OWNER-REPORTED GREEN**; **SUPERSEDED** for corrected driver | No complete operator receipt; partial directories below | Uncorroborated; message alone cannot establish four CLI phases |
| Clippy `unused_unit` | `security.rs::no_links` before removal; historical hash unavailable | Owner execution and pasted diagnostic | **OWNER-REPORTED RED**, one lint/setup scope only | Pasted file/line diagnostic | High confidence about reported finding; not repository-wide RED |
| Clippy `nonminimal_bool` | Bearer predicate before De Morgan rewrite; historical hash unavailable | Owner execution and pasted diagnostic | **OWNER-REPORTED RED**, one lint scope only | Pasted file/line diagnostic | Same limitation; not evidence of functional regression |
| Post-fix final-source lint | Current security hash `42d317…3899a2` in appendix | No subsequent execution result supplied | **UNVERIFIED** | None | No cleanliness inference |
| Corrected operator driver execution | Current driver hash `127759…8fef58` in appendix | No subsequent execution result supplied | **UNVERIFIED** | None | AST validity is not execution |
| PowerShell AST review | Exact current driver hash | Agent bounded static inspection | **STATICALLY VERIFIED**, zero parse errors | Parsed current file; no persisted execution receipt | High for syntax and inspected command construction only |
| Source/protected-path review | Exact appendix inventory and fingerprint | Agent bounded static Git inspection | **STATICALLY VERIFIED** | Current status/diff and hashes | Only lib/main integration changes among existing tracked source; no runtime compatibility claim |
| Existing workspace log | Dirty source revision not recorded in log | Owner-created log, inspected read-only | **DIRECTLY VERIFIED** artifact contents, not complete campaign result | `target/pcg-175/workspace-tests-20261110-105724.log`, SHA-256 `eaf078084eef7e7e37b1cc0cd5942bf9fb9c47c9a275bea81db1a75cf68ea66d` | Contains a completed test-profile compilation line (4m 27s), individual non-PCG failure markers and unfinished tests; no final summary, panic details or PCG results |

No later complete verification log was found in the inspected PCG output scope.
The workspace log is not a source-bound PCG receipt. Its unrelated failure causes
were not investigated; no implementation verdict is drawn from those markers.

Present executable: `target/debug/clean-ctx.exe`, 49,946,112 bytes, SHA-256
`557eec1f661937e150e604d6f019cb1180c7fd81f0399951cf57dfa867aa1b66`.
This directly identifies available binary bytes. It does not establish which
uncommitted source produced them; a file timestamp cannot supply that proof.

## C. Clippy reconciliation

Both fixes are present in `src/pcg/security.rs`, SHA-256
`42d3179680d9da0538f7bcfcaf8849571d36ac76e5812b98fc6d9704883899a2`.

| Finding / function | Before | Current | Static assessment / post-fix lint |
| --- | --- | --- | --- |
| `unused_unit`, `no_links` | Permitted missing-leaf match arm `{ () }` | Empty arm `{}` | Both evaluate to unit and continue the same loop without effects. **STATICALLY RESOLVED / LINT UNVERIFIED**. |
| `nonminimal_bool`, `nonsecret` bearer branch | `!(A && !B)` | `!A || B` | A is `after.starts_with(char::is_whitespace)`; B is `after.trim_start_matches(char::is_whitespace).is_empty()`. Boolean De Morgan equivalence preserves the condition passed to `require`, including short-circuit evaluation. Both predicates are side-effect-free. **STATICALLY RESOLVED / LINT UNVERIFIED**. |

No lint suppression was introduced. Neither static finding establishes complete
library or repository-wide Clippy cleanliness. The two behavior-preserving lint
edits alone do not justify rerunning the previously green PCG suite merely for
a newer timestamp; absence of a post-fix test execution is not regression evidence.

## D. Operator-driver inspection and required stop

Current file: `verification/pcg-175/run-cli.ps1`, SHA-256
`1277596c2df471f32e5e24d203abf522a6fc9038db73e16ed25a2da87e8fef58`.

| Boundary | Static observation |
| --- | --- |
| Parameters | Mandatory binary path; optional output root; positive `BaseRecords` in 1..10000, default 8. No single-case mode. |
| Binary | `Resolve-Path -LiteralPath`, invoked through its resolved path. Binary SHA-256 compared before/after the driver. No source-to-binary binding is recorded. |
| Git resolution | Application-only `Get-Command git`, helper renamed `Invoke-FixtureGit`; native invocation is `& $gitBinaryPath -C $Root @Arguments`. Fixture calls all use the renamed helper. Old self-shadowing call is absent. |
| Fixture repository | Creates output/case/repo directories and writes reviewed public fixture bytes, then native init/config/add/commit and exact commit/blob lookup. No network commands. |
| Enrollment | Builds strict source/evidence/record/coverage manifest, hashes its raw JSON bytes into a fixture profile, uses distinct decision subjects and one shared exact quote. Git LF conversion disabled; no arbitrary production evidence enrollment. |
| CLI phases | Four separate native processes: import, identical reimport, `--rebuild`, `--verify-only`. Checks generation equality, reimport operation, current-source work and unchanged main DB bytes over audit. |
| Status handling | Each native helper checks `$LASTEXITCODE` and throws on nonzero. The value is not retained in a receipt. CLI stdout is parsed into an in-memory report; stderr/raw output and phase failure evidence are not durably captured. |
| Receipt | Only after all three populations succeed is one aggregate `operator-receipt.json` written. A prior phase's successful report is lost if a later phase fails. No explicit phase exit-code fields. |
| Population | Hard-coded loop over `BaseRecords`, 2x and 4x: defaults schedule 8/16/32. `-BaseRecords 1` still schedules 1/2/4; it is not a one-case functional run. |
| Cleanup | No cleanup/delete/move operation. Partial directories remain on failure; their existence is not success. |

The reported Git-resolution correction is **STATICALLY VERIFIED**, and current
AST parsing reports zero syntax errors. No further guaranteed runtime failure
was established by static inspection. However, the existing definition cannot
deliver this mission's requested *single-case, durably corroborated phase-status*
verification. Those are concrete acceptance-harness gaps, not Rust defects.

Exact minimal proposed correction, **NOT IMPLEMENTED**:

1. Add an opt-in `-SingleCase` switch selecting only `BaseRecords`; preserve
   the existing scaling mode for separately authorized performance work.
2. Within the existing native helpers/output directories, retain the stage,
   arguments, stdout/stderr and exit code before throwing on failure. Persist
   each of the four CLI phase reports immediately and retain a final success/
   failure summary. This must distinguish fixture setup failure from CLI failure.
   Do not add a framework, new tests or new verification populations.

Acceptance directive §6 says: “If a defect is found, report the exact minimal
correction required and stop for owner approval before editing.” Therefore the
driver is unchanged in this review; execution and correction are withheld.

## E. Existing operator artifacts

| Run directory under `target/pcg-175/` | Population/files | Git fixture | Profile / approved manifest | Projection | Four phase receipts / exit statuses | Complete summary | Classification |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `m2a1-20261110-153739-5197` | Only `n-8/repo/decision.md`, 201 bytes | Absent, including hidden `.git` | Both absent; no digest approval can be verified | Absent | Absent | Absent | Incomplete / uncorroborated |
| `m2a1-20261110-154641-6334` | Only `n-8/repo/decision.md`, 201 bytes | Absent, including hidden `.git` | Both absent; no digest approval can be verified | Absent | Absent | Absent | Incomplete / uncorroborated |

These directories are consistent with stopping during setup, but do not identify
the historical exit or prove the cause. The Git-shadowing defect is independently
established by earlier source inspection, not proven as the cause of these two
runs. No CLI execution, failure or success is recoverable from these artifacts.
No 16/32 case files or other operator receipts are present in the inspected runs.

## F. Minimal owner verification plan — execution withheld

This is the bounded proposal requested by the owner, not execution authorization.
No executable below was run during this mission. No full suite, all-target lint,
benchmark, new integration test or broader campaign is proposed.

| Command | Contract verified | Why needed | Expected scope / cost | Owner approval |
| --- | --- | --- | --- | --- |
| `cargo clippy --package clean-ctx --lib --all-features -- -D warnings` | Current production library, including the two corrected expressions | No final-source lint outcome is supplied | One package/library and its dependencies; excludes tests, examples, binary and proxy targets. Cargo cannot select the internal PCG module alone. All features preserve the existing development feature mode/cache. Duration unknown; compilation may occur, no promised fast bound. | **NOT EXECUTED — OWNER AUTHORIZATION REQUIRED** |
| Conditional: `cargo build --package clean-ctx --bin clean-ctx --all-features` | Build the actual CLI against identified final source | Existing executable has no recoverable final-source binding, and production changed after the original reported build | One binary plus required library/dependencies, no tests/workspace campaign. Cache-dependent cost unknown. Omit if an existing source-bound final binary receipt is supplied. | **NOT EXECUTED — OWNER AUTHORIZATION REQUIRED** |
| Conditional proposed invocation: `pwsh -NoProfile -ExecutionPolicy Bypass ./verification/pcg-175/run-cli.ps1 -Binary ./target/debug/clean-ctx.exe -BaseRecords 1 -SingleCase` | Exactly one record/source fixture through import, idempotency, rebuild and read-only audit, with retained phase outputs/statuses | Current corrected driver has not executed with corroborated receipts | Four short-lived CLI processes on one tiny fixture; elapsed cost unmeasured. **This flag does not exist yet: do not execute unless the minimal driver correction is separately approved and implemented.** | **NOT EXECUTED — OWNER AUTHORIZATION REQUIRED**, plus prior driver-edit approval |

No focused Rust test rerun is proposed solely for the two equivalent lint edits.
If a later substantive production correction is discovered, it requires its own
scope/authority assessment; it is not authorized here.

Repository `verification.md` lists broad commands, including
`cargo test --workspace --all-targets --all-features` and
`cargo clippy --all-targets --all-features -- -D warnings`. The mission expressly
withholds those executions. The owner reports the suite exceeding five hours;
no defensible all-target Clippy duration is available. Neither is necessary to
close the two lint findings and one CLI lifecycle gap, so neither is in this plan.
Unrun broad checks remain unrun, not RED and not silently GREEN.

## G. Acceptance recommendation

**Historical recommendation: M2A1 ACCEPTANCE BLOCKED — operator evidence definition only.**

The Rust foundation remains a plausible owner-acceptance candidate with scoped,
owner-reported historical green tests. No production atomicity, identity, trust
or rebuild defect was demonstrated in this bounded reconciliation. The current
driver cannot yet provide the mandated one-case/status-retaining proof, so the
mission's explicit driver stop condition applies. Missing receipts and final
lint outcome are additional evidence gaps, not implementation failures.

Needed to close: owner approval of the two minimal driver changes; final-source
library lint evidence; a source-bound binary; one corroborated four-phase CLI
receipt with actual statuses. The owner may instead explicitly accept documented
residual risk; such acceptance must not be labelled an executed GREEN. No M2a-2,
commit, push or publication has been authorized or started.

## Appendix: evaluated inventory

The following is the exact canonical input to the fingerprint in §A:

```text
ece9607661acddcc146ebee09467878535965ef8da0e1e088911a18e5e2028eb  docs/agent/PCG_175_ARCHITECTURE_INVESTIGATION.md
73fd5147ea202906e46eacd7740d719265c407732d01c6a63434e184b4a5ffb4  docs/agent/PCG_175_DESIGN_FREEZE.md
7e5cfecfbf54074aca631c8a8b2dfac10daf1c1f132242b975c3eab268ee8607  docs/agent/PCG_175_M2A1_IMPLEMENTATION.md
cdac6cf4f82d9ee571b9c3d39b8258eb993bbea8d5772b6997f35610f389ff21  src/lib.rs
c7c2a69e83a23b9608080b541e14cc46aadfda7a877209feb427130d2fa8273e  src/main.rs
b43a77f9a0587c55c5a9353948693119079043193e0c300cf56864ede20d175c  src/pcg/cli.rs
1888f81151464171efc437f4c1d9f2eb883a3982f5b62608f05d2a03a772878f  src/pcg/enrollment.rs
b19c530d735eb4d31873dfaf14fc51e20e79090beeada860da3697342ccd7ae4  src/pcg/git.rs
c5ad209db16f8c5350f37a16d4843dfebe5993587b1ad28d5e67634bb4b42c9c  src/pcg/identity.rs
8ea7add87c44fd15612497ad92432623e3440056735935fac725dbbaca0171f7  src/pcg/import.rs
62fe1b4f71c18ecaee92ba7df98abf1c1c6a5e13a97f0c99f4a603e68c293fad  src/pcg/mod.rs
1f191ac1fd72480969cb2bda1fa0489a0a242006159daa9249a3a990f05bff3a  src/pcg/model.rs
05dbda5569bcfd9771045e00ec60c8386ad6fea5550dee5b8525936fcaf51d10  src/pcg/profile.rs
5c1ded99aebfcc1e6a64a3b0eae609a5a59869b167b5f538c06e45bc61852948  src/pcg/schema.sql
42d3179680d9da0538f7bcfcaf8849571d36ac76e5812b98fc6d9704883899a2  src/pcg/security.rs
23093872b300e5b9083e684d1951d991d336fdc88b67df8f3e37429440b75f84  src/pcg/store.rs
b1ba939c03453397b83c705070190df71862004bbd31acb05301db5027a3de21  src/pcg/validation.rs
8e520c1a1c17361be116160d544875315f85606fec40bcb583bdd5f3f55996e9  src/tests/pcg/enrollment.rs
803136228e807fcf61b6e9085a19b7cfd65bedb6ecacefe8acb2634becb082b8  src/tests/pcg/persistence.rs
ee7e56f3c34ef009a043faf12895460ab3d603aa0c27868253dc21ad34380d93  src/tests/pcg/process.rs
702e96f787ca1eaa5830dbc71cd450b634817af5a508f72ce8c44daf06cb7376  src/tests/pcg/security.rs
fe1f7e6836692cccce2dc884b27f23d74ccea9b53f8cda21bdf389cc126a3672  src/tests/pcg/support.rs
4d64799d3d1e030eac33cb86951c06fd814b1a7b2033da68a08752d75cee892c  verification/pcg-175/fixtures/decision.md
9cb958b9877c139b4ed4736fefe11ac8f36730607761d8b7771535bf370827bd  verification/pcg-175/README.md
1277596c2df471f32e5e24d203abf522a6fc9038db73e16ed25a2da87e8fef58  verification/pcg-175/run-cli.ps1
```
