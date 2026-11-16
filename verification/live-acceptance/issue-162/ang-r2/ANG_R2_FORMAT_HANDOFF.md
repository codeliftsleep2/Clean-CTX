# ANG-R2 post-normalization operator handoff

Agent: **Agent MaxHeadRoom**. Prepared under owner disposition 03; execution UNRUN.

Applied identities are pinned in ANG_R2_FORMAT_NORMALIZATION.json. It links the
hash-verified raw evidence retention manifest under target/ang-r2. Exact pre-format
sources and recorded RED/GREEN/replay evidence remain historical; no old result is
relabelled post-normalization. The archival sources are not executable test assets.

The active tests remain under src/tests. Fourteen existing test bodies were moved
without changing their formatted bytes: eight query/contract/work tests and six
admission/SQLite/replay tests, using the registered shared support module. Together
with the two normalized public tests and two formatted transport tests, the current
registration filter selects 18. The normalized legacy retirement witness adds one.

From the repository root, the exact operator command is:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1
```

The batch captures these commands in order and stops on failure:

1. `rustfmt --edition 2024 --config skip_children=true --check` with the explicit
   sorted active Rust file list captured in active-rust-files.json. This is a
   read-only scoped check; the consolidated repository format gate stays in CI.
2. `cargo test --lib --all-features --quiet issue_162_deleted_owner_legacy_membership_is_retired -- --test-threads=1 --nocapture` (one).
3. `cargo test --lib --all-features --quiet issue_162_resolver_registration_ -- --test-threads=1 --nocapture` (18, including both normalized public witnesses and all fourteen relocated tests).
4. `cargo clippy --lib --bin clean-ctx --tests --all-features -- -D warnings`.
5. `pwsh -NoProfile -File scripts/check-utf8.ps1`.
6. `pwsh -NoProfile -File scripts/check-file-sizes.ps1`.
7. The established production knowledge harness invokes pcg_knowledge_v1 context
   for DOC-270 using the pinned existing executable/profile. This is the actual
   normal document admission checker. It captures result_state, catalog_current,
   complete, generation and diagnostics. A blocker fails this stage; descriptor
   capture may then identify the precise blocker, but no grant is submitted.

This tests 19 tracked cases, verifies the changed module wiring and formatting,
and retains exact active-source/log hashes, counts and elapsed times. The older
43-test, replay-only and 185-check stdio passes remain separately attributed.
No broad regression campaign, formatter mutation, production rebuild, server pilot,
commit or push is hidden in the batch. The knowledge harness starts only its
established bounded local stdio process for actual admission.

Current governing candidate and scope coverage are pinned separately in
ANG_R2_FORMAT_GOVERNANCE_PINS.json. The operator command is a handoff, not permission
for the agent to execute it. After results, reconcile actual outcomes before any
delivery claim; required CI and applicable field evidence remain distinct.

## Documentation finalization after successful packet

The 19 tests, scoped format/Clippy and initial guards/admission passed in
format-verification-20261116-111528-924. After adding the versioned historical
evidence archive and recording those actual results, reuse that exact source
verification without rerunning it:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1 -ResumeVerifiedFrom target/ang-r2/format-verification-20261116-111528-924
```

Only documentation guards and actual revised-document admission run. Rust source
manifests, prior result logs/counts and baseline must match or the wrapper refuses.
The archive is byte-preserving historical evidence, not executable test coverage.

## Lifecycle follow-up

Two additional tests cover configured additional-root refresh/reacquisition and
actual apply_edit/apply_delta durable transport. Both have passing evidence in
remaining-lifecycle-20261116-114846-300 (root test) and
remaining-lifecycle-20261116-115908-481 (corrected mutation test). No production
code changed. Use the existing packet with the verified lifecycle receipt:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1 -VerifiedLifecycleFrom target/ang-r2/remaining-lifecycle-20261116-115908-481
```

This skips tracked tests and runs only scoped format/Clippy, guards and revised
actual document admission. A full fresh registration selection now expects 20,
not the historical 18. Source hashes and warnings remain fail-closed.

## Feature-guard follow-up

The historical CI run failed default-feature compilation on test-only items whose
consumers require feature rust. Narrow cfg alignment was verified with warnings
fatal in default and all-feature modes. Reuse those compilation receipts while
running only scoped format/Clippy, guards and revised actual admission:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1 -VerifiedFeatureGuardsFrom target/ang-r2/feature-guard-check-20261116-123207-211
```

No tracked test runs, production rebuild, field pilot, PR or CI dispatch is hidden
in this mode. A successful local correction does not turn the failed historical
CI run green or establish CI for a later commit.
