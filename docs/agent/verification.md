# Verification (`docs/agent/verification.md`)

This is the **single authoritative definition of the final verification
gate**. Do not duplicate these command lists elsewhere; other documents
reference this one.

<!-- pcg:VERIFICATION-current:begin -->
## Current execution responsibility — B11 owner disposition 04

Effective 2026-11-13, the consolidated full repository gate belongs to CI.
Agents must neither execute that gate locally nor hand it to an operator as a
required local step. This changes execution responsibility, not the correctness
standard: the complete gate must cover the reported change set and pass before
final verification or migration completion is claimed. Required architectural
audit, tracked regressions, zero warnings, encoding, active-file guards and
bounded-work obligations remain in force. CI failures remain failures to address.

Local verification consists of proportionate focused tests, applicable scoped
linting, production builds and repository guards. Under Rule #1, agents hand all
tests, Cargo commands and runtimes to the operator unless the owner explicitly
authorizes that particular run. Static, fast bounded inspection remains allowed.
Batch local checks and retain actual receipts; do not repeatedly run an unchanged
checkpoint. Report focused operator results separately from CI and field evidence.
Pending or unavailable CI is not a successful final gate.

The older execution instructions below are retained as historical applicability,
including their command lists. Their local/full-gate responsibility statements
are superseded prospectively by this section. They are not a current local
handoff. Previous results retain the circumstances under which they were run.
<!-- pcg:VERIFICATION-current:end -->

<!-- pcg:VERIFICATION-responsibilities:begin -->
## CI-VERIFY-R1 candidate responsibility amendment

This amendment is pending predecessor-bound B11 owner authorization and actual
admission. The accepted predecessor remains controlling until that checkpoint.
The owner-approved bounded implementation is recorded in [CI-VERIFY-R1](CI_VERIFY_R1.md).

On legitimate admission, the complete mandatory CI entry point is
scripts/check-semantic-work-fitness.ps1 -Consolidated. It preserves the full
workspace/all-target/all-feature correctness scope, lists actual registered tests
without executing them, executes the ordinary suite once with one libtest thread,
then verifies complete registration/outcome membership and exact deterministic
fitness groups. Both frozen oracle environments and optional historical replay
configuration are explicit, pinned and recorded. Missing/duplicate execution,
failed assertions, invalid mandatory prerequisites, incomplete/empty fitness
membership and ignored required tests fail the gate. Semantic fitness outcomes
remain separately attributable in the consolidated receipt.

The original focused fitness command and validated historical local-resume mode
remain supported for their scoped use; they are not an extra duplicate CI test
stage. Its older pre-full-suite execution instructions below remain historical
applicability after admission of this amendment. Existing pipeline status checks,
compilation/lint/format, encoding, active-file, dependency/security checks and
repository governance obligations remain applicable.

Repeated empirical studies are explicitly enumerated in the governed measurement
membership and run only through the manually invoked performance workflow.
Their original fixture axes, warmups, paired samples and reporting remain intact.
All unique semantic, lifecycle, persistence, frozen-equivalence and deterministic
work assertions remain in ordinary mandatory tracked companions. Only measurement
entries may be ignored by ordinary execution, in addition to the nine explicitly
recorded pre-existing ignored tests whose standing is unchanged. This does not
waive an existing empirical acceptance prerequisite for a production change.
Correctness and deterministic fitness results are not performance certification.

No complete local repository run is required or authorized by this amendment.
CI remains owner of final verification; focused local invocations stay operator-owned.
No changed gate is accepted merely because a descriptor or graph query succeeds.
<!-- pcg:VERIFICATION-responsibilities:end -->

## Final Verification Gate

The complete gate must cover the reported change set before declaring any task
complete. Under Rule #1 (§1, long-running process boundary) in the engineering
rules, agents provide these commands for the user to run manually and report
the user's results accurately. Agents may run focused, reasonably short checks
during implementation, but must not start the complete gate unless the user
explicitly authorizes that particular run.

User-run commands:

```bash
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/tests/check-file-sizes.tests.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test --all-features encoding
```

<!-- pcg:VERIFICATION-guards:begin -->
A single warning constitutes a build failure. Do not weaken assertions or
suppress failures merely to obtain green.

The file-size guard enforces the active-file policy from the engineering
rules. Without `-BaseRef`, it checks working-tree, index, and untracked files.
CI passes `-BaseRef` so committed changes relative to the target branch are
also active. Untouched tracked files above 615 lines are reported as legacy
debt and do not fail the gate. Path-scoped exemptions declared inline in the
guard (`$ExemptPaths`, each carrying its justification and the maintainer
decision that added it) are excluded from the active set entirely, so they are
neither failed nor reported as legacy debt.
Activating a legacy oversized file is a correctness decision, not a cost
decision: when the architecturally correct change belongs in an oversized file,
that file is decomposed semantically as part of the change (engineering rules,
§8a) rather than the architecture being weakened to avoid it.
<!-- pcg:VERIFICATION-guards:end -->



## Ordinary-task verification

For ordinary tasks, the gate above is run once, for the changed targets where
possible, before declaring the task complete. Prefer targeted suites for fast
iteration, then run the complete gate at the end.

**Windows performance note:** `cargo test` recompiles even on no-change runs.
On this machine the compile step can exceed the agent's 30-second command
timeout. After the test binary is built, prefer direct invocation (see
`.clinerules/engineering.md` §8b for the exact PowerShell incantation). Use
`cargo test` only when a rebuild is actually required.

## Migration-finalization verification

For an explicitly designated **incremental architectural migration**, do NOT
repeatedly run the full repository-wide gates after every small phase while the
migration is in progress. The branch has an established green baseline before
the migration begins; use targeted inspection or reasoning to verify correctness
per phase.

The **complete Final Verification Gate is mandatory during migration
finalization** (see `incremental-migration.md` for the full procedure):

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --workspace --all-targets --all-features`
4. Run the file-size validator tests and active-file validator.
5. Confirm all checks pass.
6. If anything fails: investigate, determine whether the migration caused it,
   fix migration-caused failures, never dismiss simply because unexpected.
7. Perform the required architectural audit before declaring the migration
   complete.

**An architectural migration is NOT complete until the final verification
gate has been performed.**

## Relationship to CI

The repository's CI (`.github/workflows/ci.yml`) already enforces, on every
push/PR:

- `cargo check --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `scripts/check-tree-sitter-versions.ps1`
- `scripts/check-file-sizes.ps1` with the change's base revision
- `scripts/check-utf8.ps1`
- `cargo test --workspace --all-targets --all-features`
- `cargo audit` (with one documented advisory ignore)

The local gate above mirrors CI; local verification is belt-and-braces, CI is
the authoritative release-time enforcement.

<!-- pcg:VERIFICATION-obligations:begin -->
## Live acceptance harnesses are not tests

Reusable operator harnesses that drive a built binary over MCP stdio are
tracked under `verification/**`. Their fixtures, scenarios, oracles, and driver
scripts are reviewable verification assets. Generated captures, temporary
workspaces, databases, and other runtime state belong under `target/**`.

- A harness result is operator/live-verification evidence. It is not part of
  the Rust/CI gate and never substitutes for a tracked regression test.
- Every required regression MUST be a tracked test under `src/tests/**` (see
  `architecture.md`, Test-file convention) that
  `cargo test --workspace --all-targets --all-features` compiles and runs.
- Report a tracked harness PASS as operator/live verification, never as test
  coverage. Placing a Rust test where the gate does not compile and run it is a
  policy violation (engineering rules, §8d).
- A harness cannot repair a red gate. The failing tracked test stays failing
  until the tracked test itself is green.

## Encoding verification

Text-file encoding is enforced mechanically (no memory needed):

- **Pre-commit** — `.githooks/pre-commit` (wired by
  `scripts/install-git-hooks.ps1`) runs `scripts/check-utf8.ps1` and aborts the
  commit on failure.
- **CI** — `.github/workflows/ci.yml` runs the same guard on every push/PR.
- **`cargo test --all-features encoding`** — a Rust-side twin over the repo tree plus a
  Unicode canary fixture.

All three invoke the same `scripts/check-utf8.ps1`; there is exactly one
implementation of the encoding-detection logic. The authoritative policy
lives in `.clinerules/encoding.md`; rationale is in `docs/ENCODING_POLICY.md`.

## Finalization requirements

A task is complete only when:

- the requested implementation is complete,
- the resulting architecture is coherent,
- the repository is left in a state consistent with the engineering rules,
- every required regression is a tracked test in `src/tests/**` that this gate
  compiles and runs, with applicable reusable live verification tracked under
  `verification/**` and reported separately,
- the final verification gate passes.

The final state must not merely work; it must also be architecturally
coherent, behaviorally preserved, properly tested, zero-warning, documented
accurately, free of obsolete implementation paths, and consistent with
established Clean-CTX architectural principles.

## AF-PERF-001 mandatory bounded-work gate

Run scripts/check-semantic-work-fitness.ps1 before an expensive semantic acceptance
campaign. It runs tracked AST/work-count/lifecycle tests plus resolver P0, with all
features. Compilation errors, empty selections, failures and ignored required tests
fail the gate. CI runs the same command before its full test step. Gate GREEN covers
only the declared inspected admission families; #173/#174, uninstrumented witness
scaling and snapshot lock behavior remain separate open risks. Static unresolved
findings require recorded review; they are not proven absence of amplification.

<!-- pcg:VERIFICATION-obligations:end -->
```architecture-graph
node: AUTH.FINAL_VERIFICATION
kind: AUTHORITY
status: IMPLEMENTED
title: Repository final verification and bounded-work gate
track: REPOSITORY
artifact: docs/agent/verification.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
knowledge_scope: authority
knowledge_content:
  - statement self#VERIFICATION-current
  - statement:responsibilities self#VERIFICATION-responsibilities
  - constraint:guards self#VERIFICATION-guards
  - verification_obligation:completion self#VERIFICATION-obligations
  - historical_applicability:source self#whole
```
