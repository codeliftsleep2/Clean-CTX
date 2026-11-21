# OPS-R1 — Disposition 18 exact test application

Agent MaxHeadRoom — 2026-11-20.

## Applied scope and identities

Applied only the owner-approved `review-17/test-correction.patch`, after verifying
its SHA-256, original test bytes, baseline source-tree identity and successful
`git apply --check`. The corrected raw test hash matches the approved LF hash.
All other files in the pinned source set retain their pre-application raw hashes.

- Patch: `7a4f68c8491e3d4edbf76d99ca50a2e7d18bb230e33c958cbde6f128660129de`.
- Original test: `7767841034a7d890f3ee696df399d5d4b101ca2e8963b0201a0e2a974b388db5`.
- Corrected test: `7f87059494f87b780067875bf71e35de17417df6c988bdceff74764dba8ca582`.
- Baseline tree: `91ff7766ca9f0aaa965734748c6aab8e6b186dd5da8f0d6a36e2dfb16d11efba`.
- Corrected tree: `25d17848f53e607313b877bdb64f5d791d280e20f11dc935c244d880f1b1a8eb`.
- Sole changed source: `src/tests/ir/ops_r1_shared_acquisition_isolation.rs`.
- Application receipt: `target/ops-r1/disposition-18-test-application/application-receipt.json`.

## Build launcher blocker — stopped before compilation

Static inspection establishes that the existing reviewed launcher cannot accept
the new tree. `execution/invoke-build-capture.ps1` calls
`capture.py --verify-sources-only` before creating output or invoking Cargo.
`capture.py:115` selects `reviewed-green-amendment-15-sources.json`; its bytes are
pinned in SOURCE_PINS, and that manifest requires the old baseline tree above.
The adapter has no argument for a new source manifest. OutputDirectory changes
the destination only, not the accepted source identity.

No launcher or old manifest was modified and no obsolete pin was bypassed.
There is no new build receipt, compiler diagnostic set or executable identity.
No Cargo command, test or source-verification adapter execution was started.

Required next decision: approve a separate Disposition 18 source manifest and
the narrow adapter change selecting and hash-pinning it for GREEN, retaining the
old manifest and historical adapter identity. Build command/safety/capture logic
would remain unchanged. Select a fresh output under target/ops-r1 for the single
already-approved compilation-only build. Do not run the old command against the
new tree or attribute its existing executable to this correction.

## Fault-detection strength, separate from passing

The corrected positive control explicitly checks normalized physical/logical
identity and equal grouping basis, then requires unchanged acquisition counts
and one accepted qualification. An implementation that unnecessarily reacquires
that request should fail these assertions. This is a static detection obligation,
not a demonstrated mutation-test result.

The negative control establishes changed loaded strict options, unequal grouping
basis, another independent acquisition and no additional accepted qualification.
Unsafe reuse that actually occurs across that change should fail it. However,
rewriting tsconfig also invalidates prior config-content witnesses: a defective
group key might still avoid unsafe reuse through currentness rejection. Therefore
this control alone does not prove every option-key omission is detected, nor does
it isolate grouping from currentness. A future fault-detection assessment should
identify the exact faulty implementation being challenged and demonstrate its
failure without assuming that every internal safeguard is independently covered.
No mutation, test expansion or execution is authorized or performed here.

## Evidence and governance boundaries

The original case-9 failure, eight preceding passes and sixteen unexecuted cases
retain their original status and evidence. Production implementation, CI,
protected declarations and governance grants remain unchanged. No build, test,
commit, push or publication occurred.

This checkpoint continues the existing OPS-001 mission association; it does not
claim fresh graph reconciliation or admission. The existing OPS-001 source
declaration was not changed. Applicable association/admission remains required
before publication, separately from compilation and eventual bounded execution.
