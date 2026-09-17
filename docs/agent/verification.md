# Verification (`docs/agent/verification.md`)

This is the **single authoritative definition of the final verification
gate**. Do not duplicate these command lists elsewhere; other documents
reference this one.

## Final Verification Gate

The complete gate must cover the reported change set before declaring any task
complete. Under Rule #1 (§1, long-running process boundary) in the engineering
rules, agents provide these commands for the user to run manually and report
the user's results accurately. Agents may run focused, reasonably short checks
during implementation, but must not start the complete gate unless the user
explicitly authorizes that particular run.

User-run commands:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/tests/check-file-sizes.tests.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test --all-features encoding
```

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

## Live acceptance harnesses are not tests

Untracked operator harnesses (for example `target/tmp/*.mjs` or
`target/tmp/*.ps1`) that drive the built binary over MCP stdio are permitted and
often useful: they let the operator observe real output from a real binary. They
are hand-off convenience, and that is their entire role.

- A harness result is NEVER part of this gate, never a substitute for a tracked
  test, and never a completion criterion.
- Every required regression MUST be a tracked test under `src/tests/**` (see
  `architecture.md`, Test-file convention) that
  `cargo test --workspace --all-targets --all-features` compiles and runs.
- Reporting a harness PASS as coverage, or placing a test where the gate does
  not compile and run it, is a policy violation (engineering rules, §8d).
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
  compiles and runs (an untracked live harness outside it is never coverage),
- the final verification gate passes.

The final state must not merely work; it must also be architecturally
coherent, behaviorally preserved, properly tested, zero-warning, documented
accurately, free of obsolete implementation paths, and consistent with
established Clean-CTX architectural principles.
