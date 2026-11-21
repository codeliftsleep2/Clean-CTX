# OPS-R1 Owner Disposition 02 — exact preparation packet

Agent MaxHeadRoom. 2026-11-20. **All implementation diffs remain unapplied.**
Disposition-03 [source review](OWNER_CHECKPOINT_03.md) records subsequent candidate
corrections, including the separate governance diff and stronger capture identity
checks. Use that review and its exact current hashes for application approval;
the earlier unapplied diffs remain preserved in the pre-review archive.
No potentially hanging process, build, new CI policy, grant or publication ran.
The four workstreams are bounded preparation, not cross-cutting implementation approval.

## 1. Tranche-1 implementation manifest

Machine-readable [proposal index](../../../verification/ops-r1/proposals/tranche-1/proposal-index.json)
contains full test identities, baseline byte hashes, candidate LF hashes, file sizes
and exact isolated selections. Package is `clean-ctx`; Cargo target is `--lib`,
registered target `unittests src/lib.rs`. Allfeature identities are historical
run-414 registrations, not guessed namespace-to-file mappings or new execution.

| Golden obligation | Exact tracked source/function anchors required in ordinary allfeature CI |
| --- | --- |
| GARC-001 / AFF-003 | src/tests/ir/ci_runtime_core_query.rs::ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query; src/tests/compression/core_ir_registration.rs::ci_runtime_complete_queries_preserve_captures_and_ir and ::ci_runtime_custom_invalid_and_mismatched_queries_stay_dynamic |
| GARC-002 / AFF-004/010 | src/tests/ir/native_feature_evidence.rs::ci_runtime_native_typescript_evidence_preserves_source_ownership |
| GARC-003 / AFF-011/008 | src/tests/mcp/angular_deleted_owner_membership_issue_162.rs::issue_162_deleted_owner_legacy_membership_is_retired; semantic_retirement_controls_issue_162.rs::issue_162_retirement_acquisition_error_is_not_deletion, ::issue_162_retirement_shared_supports_siblings_and_relationship_families and ::issue_162_retirement_history_and_owner_ordering_survive_deletion |
| GARC-004 / AFF-008 | src/tests/mcp/restore_registration_boundary_issue_162.rs::issue_162_unchanged_bootstrap_durable_registration_boundary; restore_registration_config_matrix_issue_162.rs::issue_162_restore_registration_config_preservation_matrix |
| Existing AFF-012 | Existing ci_verify_r1_required_execution_and_failure_contracts and actual scripts/verify-ci-membership.py; no second equivalent suite |

The primary four historical regressions already executed in retained CI, but none
was explicitly required in the existing 56-group fitness membership. Therefore a
disappearing registration could escape listing-versus-outcome equality. The proposal
requires ten exact allfeature identities (four primary plus preserving controls)
while reusing their existing single ordinary execution. It adds no allfeature
runtime reruns. Source/assertion review remains necessary for semantic test changes.

Exact isolated configuration commands proposed in CI (full profile arguments and
registered names are in the CI diff):

- C# only: `cargo test -p clean-ctx --lib --no-default-features --features csharp`
  with the exact complete-query capture/IR equivalence test; one runtime outcome.
- TypeScript only: corresponding `--features typescript`, exact complete-query
  test plus native source-owned evidence test; two runtime outcomes.
- Both use the existing optimized test profile, debug assertions and overflow
  checks unchanged, `--exact --test-threads=1 --format=pretty`, and retained output.
  Whole lib-test target compilation protects wiring; exact nonempty outcomes
  protect the selected native behavior. No full-suite feature Cartesian product.

Historical counterexamples and authority revisions are in GOLDEN_CORPUS.md,
HISTORICAL_FAILURES.md, ESCAPE_ANALYSIS.md and golden-corpus-design.json. Reuse
the exact query-count RED, isolated compiler failures, deleted-owner RED and
unchanged-source restore RED; their source/revision/feature qualifications remain.
Compiler RED is not behavioral RED, and a current PASS alone is not detection proof.

## 2. Minimal proposed CI/verifier modifications

All files below are concrete review diffs, **not installed behavior**:

- [Membership diff](../../../verification/ops-r1/proposals/tranche-1/membership.patch):
  add one golden membership file and its required count/path to existing config.
  Do not materially edit/decompose the large existing fitness manifest merely to
  insert ten obligations; keep existing 56 groups and 18 measurement pairs intact.
- [Verifier diff](../../../verification/ops-r1/proposals/tranche-1/verifier.patch):
  extend existing prerequisites/reconciliation with required golden membership,
  report GARC IDs, check normalized-LF hashes for the three retained behavioral
  defect witnesses, and verify exact isolated-mode outcomes using the same parser.
- [Adversarial fixture diff](../../../verification/ops-r1/proposals/tranche-1/fixtures.patch):
  include golden identities in existing valid fixtures and reject a missing golden
  manifest or changed frozen witness. No separate framework or execution scheduler.
- [CI diff](../../../verification/ops-r1/proposals/tranche-1/ci.patch):
  add the two isolated-mode commands and actual verifier calls, retain their logs;
  leave the consolidated allfeature execution and security gates intact.

Expected marginal cost: ten extra offline membership checks, three small source
hash checks and two extra adversarial verifier fixture cases; zero extra allfeature
runtime tests, two distinct lib-test feature builds and three selected runtime
outcomes. A feature build can be expensive and does not become free because its
runtime selection is small. No measured duration is claimed for an unrun proposal.
The workflow-key change causes the normal new-key population under existing
approved cache behavior; no new cache design or global time budget is introduced.

The generator is [prepare-tranche.py](../../../verification/ops-r1/prepare-tranche.py).
Its outputs pass `git apply --check` against the current checkout. They are
uncompiled and unexecuted; fixture/verifier changes still need approved focused
validation before being installed. Full repository gates remain CI-owned.
Freezing witness identities is itself proposed enforcement, not an already enacted
policy. Legitimate approved successors must preserve historical evidence and get
the required review; editing both source and expected hash is not prevention proof.
Manifest governance/review must protect this obligation rather than imply a hash
alone establishes the assertion's meaning.

## 3. Angular P0 exact execution packet — return for approval, do not run yet

[P0 proposal index](../../../verification/ops-r1/proposals/p0/proposal-index.json)
pins the baseline bytes and proposed normalized-LF result of eleven source files,
the complete proposed Rust/TOML/lock source tree digest, two test identities and
observation limits. New actual tests, if approved/applied, live under src/tests;
the proposal does not execute test code from an ignored scratch location.

Unapplied changes:
[registration/fixtures](../../../verification/ops-r1/proposals/p0/registration-and-tests.patch)
and [test-only hooks](../../../verification/ops-r1/proposals/p0/test-only-observations.patch).
No actual production source or test registration was changed. Generation is via
[prepare-p0.py](../../../verification/ops-r1/prepare-p0.py).

Fixture: fresh temporary owner with a separate source workspace and external
evidence directory; CBM/persistence disabled, proxy auto-start disabled. Reuse the
existing Angular native package fixture. Explicit tsconfig sets esnext, bundler resolution, ES2022,
legacy module detection, noLib, types empty and `include:["*.ts"]`. Generate N
ordinary script owners, each ProbeN referencing Peer0, with Peer0 declared in the
last owner. There are no resolver mappings. Hash root inputs and native package
declarations. This is a controlled synthetic diagnostic, not enterprise evidence.

Public query is exactly workspace_query with explicit fixture root,
type=resolver_registrations, direction=forward, route_path=__ops_r1_absent__.
The actual production dispatch helper is reused. Require empty lower-bound records
without error; do not assert constant work or a leak. The separate component case
submits one actual CompilationJob for owner0 while total project units grow.
That separates job count from project size at the shared mechanism; the public
query cells must honestly retain the coupling of nominated owners and TS files.

Fifteen process-isolated test observations cover inventory attempts/entries,
nomination/owners, compiler attempts/completions, project acquisitions/unit attempts,
host snapshot read misses, capture-pipeline query compilations, witness checks,
publication attempts/completions, program-loop iterations and returned batch high
water. Atomics observe worker-thread work; a 200ms sampler writes partial snapshots
outside the source inventory. These counters never grant authority, skip work,
cache results or enforce product thresholds. No exact live allocation/retained
authority-byte observation is claimed. Batch result slots are not an allocation
leak counter; program-loop iterations are not all subsystem retry attempts.

Without the explicit isolated capture environment, the proposed tracked tests use
one-unit fixtures, private temporary evidence and no process-wide sampler/counter
activation. Larger diagnostic cells require the external launcher's capture flag.
They are not ignored tests and are designed to exercise their small semantic case
without operator environment during ordinary execution. They remain uncompiled/unrun.

Proposed build command, only after approval and application of the exact P0 diff:

```powershell
pwsh -NoProfile -File .\verification\ops-r1\p0\invoke-build-capture.ps1
```

It runs exactly `cargo test --lib --all-features --no-run --message-format=json`,
with no test execution. Source digest is checked before and after; one matching
root lib-test executable is selected from Cargo JSON and hashed. Logs, command,
features/source identity and launcher identity go to
target/ops-r1/p0-build-reviewed/build-receipt.json. Existing output is never erased.
Return this build receipt for review before any potentially hanging execution.

Exact proposed first cell, requiring separate final operator execution approval:

```powershell
python .\verification\ops-r1\p0\run_bounded.py --build-receipt .\target\ops-r1\p0-build-reviewed\build-receipt.json --case owner-1 --wall-seconds 60 --memory-mib 512 --stop-wait 5
```

Other individually approved cells use the identical command with `--case owner-4`,
`owner-16`, `project-4` or `project-16`. No automatic campaign loop is provided.
Stop and review the first abnormal result rather than launch later cells blindly.

Exact tracked identities (package clean-ctx, root lib, allfeatures) are:

The P0 and Tranche-1 proposals are separately pinned to the same original checkout.
Applying both requires a reviewed combined source pin before build/execution,
because Tranche-1 also changes a tracked Rust verifier test. The P0 launcher must
refuse such unreviewed source drift rather than silently use a mismatched binary.

- mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_absent_route_observation
- mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_fixed_job_project_observation

The [external launcher](../../../verification/ops-r1/p0/run_bounded.py) verifies
the applied source-tree digest, build/launcher/log identity and executable hash.
It runs one exact test with one test thread and requires `1 passed; 0 failed;
0 ignored;` to establish nonempty successful selection. Direct stdout/stderr go
to files; OS working-set samples identify the exact spawned PID. Windows uses
GetProcessMemoryInfo on that process handle; Linux uses that PID's resident pages.

At a 60-second wall deadline or observed working set above 512 MiB, kill only the
spawned test process and wait at most five seconds. Record stop/error/exit and retain
partial logs, memory/fixture/query responses and source/binary identities. Unsupported
monitoring or capture failure stops the child, not an unrelated server. A baseline
already over the threshold invalidates the cell; a launch baseline is not a
pre-query allocation baseline. The selected test starts no child server process.
These safeguards are diagnostic controls, not product budgets or semantic fitness
pass/fail conditions. A stopped child is not automatically a behavioral RED.

The proposed tests initially establish semantic behavior and attributable work.
After observing the legitimate relationship, an exact architectural regression
predicate still needs review; preserve the original witness and baseline before
repair. This packet does not manufacture a RED by assuming all discovery is constant.

## 4. Governance state and fresh descriptor

Last valid actual descriptor before this preparation:
target/ops-r1/checkpoint-20261120-105530/descriptor.json, digest
2eadec758ccfef3eb1751dd4c41a7b8146d07c0a918ca5d6eb2b7e097fe09bac.
Actual normal context admission was blocked with
E_GOVERNANCE_AUTHORIZATION_REQUIRED; census reused, guards passed, all 28 candidate
file hashes matched. The proposal added only OPS-001 source membership; all 342
protection selections and protected authority content matched the accepted state.

This new evidence/owner checkpoint changes source components, so the older digest
must not be reused or described as admitting this packet. Obtain a fresh actual
describe and normal admission/refusal with the existing operator checkpoint. Review
any new diagnostics and return the exact descriptor before any owner-operated
authorization. No grant is inferred from Disposition 02 or generated by the agent.

## 5. Remaining risk and approval matrix

| Boundary | Current standing | Exact next approval/evidence |
| --- | --- | --- |
| First enforcement candidates | Accepted for detailed preparation; concrete unapplied diffs supplied | Owner approves exact CI/verifier/fixture diff and applicable feature obligations before application/execution |
| P0 observation source | Concrete test-only proposal supplied, not applied/compiled | Owner approves exact fixture/hooks/registration and no-run build command |
| P0 potentially hanging execution | External safeguard basis accepted, no execution authorization yet | Review build receipt; explicit approval of exact selected child command(s) |
| P0 causal repair | Broad preparation confirmed; actual stage/retention causality not measured | Preserve witness; approve exact source/project ownership repair after diagnosis |
| Publication/concurrency/retry changes | Existing contracts preserved; no blanket optimization approved | Separate disposition if the demonstrated repair requires these boundaries |
| Governance | Previous actual refusal preserved; new descriptor pending | Existing operator checkpoint, review exact new B11 descriptor and stop if protected grant required |
| A01–A13 systemic follow-up | All remain mapped; not all proven causes/defects | Prioritized consumer/counterexample evidence, independent dispositions rather than resolver-only closure |
| Deep test audit | Incremental, high-risk families first | Not a prerequisite for the otherwise approved focused tranche; no test removal or invented coverage percentage |

No production repair, installed CI policy, protected publication, commit/push or
potentially hanging execution is authorized or performed by this preparation.
