# CI-VERIFY-R1 runtime accounting after CI #414

Agent: Agent MaxHeadRoom. Investigation date: 2026-11-19.
Status: CI #414 passed; exact cache-key correction approved and implemented; cache benefit unverified.
Governing source: CI_VERIFY_R1.md, VERIFY-001. This report does not amend a contract.

## Current verified state

GitHub CI #414 passed on `2ada188e1d6fb981d7db8bc605ae1fe9101a3fdb`.
Its job took 818 seconds (13m38s), including 718 seconds (11m58s) in
workspace correctness and deterministic semantic fitness. The retained verifier
reconciled all 5,637 registered outcomes: 5,610 passed, 27 were intentionally
ignored, and no errors remained. All 56 deterministic fitness groups passed.
The ignored set is exactly the 18 enumerated measurement assets and nine
previously declared legacy cases; no required test was newly ignored.

Compared with #411, the same features, serial execution and explicit optimized
test-profile arguments were used. Exact normalized package/target/test identity
comparison finds zero removals, ten additions, and no outcome changes among
existing tests. Executable hash suffixes differ by build and are excluded only
from this cross-revision identity comparison, never from either run's verifier.

## Aggregate accounting

| Receipt | Registration compilation | Library execution | Correctness/fitness step | Outcome |
|---|---:|---:|---:|---|
| #411, `95153d5c` | 7m18s | 26m24.16s | 33m48s | Passed |
| #412, `662c7540` | 7m08s | 4m31.94s | 11m45s | Frozen-source assertion failed |
| #414, `2ada188e` | 5m48s | 6m04.86s | 11m58s | Passed |

For #414, compilation plus the library sum to approximately 712.86 seconds;
the remaining approximately 5.14 seconds of that step covers other test targets,
Cargo startup, prerequisites, membership/source checks and timestamp rounding.
The remaining 100 seconds of the job covers setup, check, Clippy, format, guards,
artifact retention, audit and teardown. These are enclosing-phase totals, not
exclusive compiler CPU profiles. Registration compilation is reported at rounded
whole-second precision. The successful #411-to-#414 library reduction is about
77%; #412's faster RED result is not substituted for accepted correctness.

## Per-test and per-family evidence

The retained Actions console now supplies timestamped individual outcomes.
`analyze-retained-runtime.py` reconciles its complete root-library sequence
against the existing membership verifier's attributed execution records.
All 5,287 library outcomes match, including ignored entries. It emits a CSV
for every library outcome and an exclusive namespace-family partition.

The intervals sum to 364.85794 seconds versus libtest's 364.86 seconds. This is
strong enclosing-time reconciliation, but each value measures elapsed time
between console result arrivals. It includes runner/IO overhead and possible
buffering; it is not an exact per-test native body timer, CPU profile or proof
of which production operation caused that time. Family assignment uses one
namespace partition per test, so cross-cutting fitness groups are not double-counted.

| Exclusive namespace family | Outcomes | Completion-interval total |
|---|---:|---:|
| `mcp::semantic_compilation` | 498 | 251.79 s |
| `mcp::tool_handlers::core::delta` | 47 | 16.86 s |
| `mcp::dispatcher` | 29 | 16.00 s |
| `cbm::client` | 115 | 10.57 s |
| `default_operation_observations_af_perf_002::public_work` | 5 | 6.11 s |
| `compression::capture_pipeline` | 13 | 5.68 s |
| `mcp::tool_handlers::hydration` | 39 | 4.90 s |
| `diff::differ` | 44 | 4.74 s |

The largest result interval is 10.42 seconds for the resolve-mapping forms/serde
case; the dispatcher stuck-worker case is approximately 10 seconds. No result
interval exceeds 60 seconds in this retained serial run. This does not create a
new timeout threshold or authorize dropping slow correctness cases.

## Three distinct causes and their dispositions

1. Production regression: complete TypeScript/C# CoreIR queries were recompiled
   per observation. The exact original warm witness recorded one compilation;
   the repaired witness records zero warm compilations with one parse and cursor.
   Process-cold concurrency and exact frozen-query/capture/IR controls passed.
   Those are scoped deterministic work observations, not an aggregate count of
   every dynamic query compiled throughout CI. Exactly two complete query
   definitions reuse the existing registry; dynamic requests remain dynamic.
2. Verification architecture: dedicated fitness execution duplicated ordinary
   suite execution. The consolidated path now executes each ordinary target once
   and checks complete membership. The previous A4 19-case fixture and 380-row
   delta corpus duplicates were also removed with assertions retained. Their
   original receipts and attribution remain in the governing mission record.
3. Research/correctness separation: 18 explicitly selected measurement assets
   preserve their sampling, warmups and growth dimensions. Their mandatory
   correctness companions remain ordinary tests. #414 confirms the measurement
   entries were ignored while all required fitness groups passed. No historical
   empirical performance acceptance obligation is retired by this separation.

## Remaining repeated fixture/production work

The semantic-compilation family accounts for approximately 251.79 seconds
(about 69% of the library interval partition) across 498 outcomes. This is the
largest remaining runtime family, not evidence that all of its work is wasteful.

The slowest mapping test enumerates five source forms and two owners, proves
seven mappings per owner, independently captures each selected declaration, and
checks both serde round-trip and older payload refusal. The passing source
therefore entails five setup calls, five McpState constructions, ten owner
capture calls and seventy independent-target capture calls. These are static
call counts for that test, not measured query-compilation counts, distinct
workspace counts for the whole suite or permission to reuse stale authority.
Any local reuse candidate must preserve every source form and independent exact
identity/currentness assertion and demonstrate enclosing benefit first.

The ten-second dispatcher test contains an actual `sleep(10)` but its intended
shutdown timeout call and assertions remain commented out. It currently asserts
only successful spawn through `unwrap`, so it is not proof of the stated timeout
contract. Record this as verification debt; no production timeout/lifecycle change,
assertion weakening, test removal or new acceptance policy is made here. It is a
small contributor compared with compilation and the semantic family.

## Demonstrated cache-profile problem

#414 restored `v0-rust-check-Linux-x64-88921d97-9e387348` as a full match
(about 207 MB). Nevertheless registration compiled 220 distinct crate entries,
including workspace packages, and took 5m48s. At teardown the cache action
reported `Cache up-to-date` rather than saving these new artifacts.

The pinned cache action retains save-state only for missing/non-exact cache
restores, and its save action returns when that state is absent. Its automatic
key includes Rust/environment and manifest/lock/config inputs, but not the
PowerShell source defining the CLI-only optimized test profile. An exact old-key
hit therefore prevents the newly built profile artifacts from refreshing that key.
This explains a verification build-reuse defect; it is not a new production cache.

Primary sources at the exact pinned revisions:

- [Setup action inputs](https://raw.githubusercontent.com/actions-rust-lang/setup-rust-toolchain/166cdcfd11aee3cb47222f9ddb555ce30ddb9659/action.yml).
- [Cache restore path](https://raw.githubusercontent.com/Swatinem/rust-cache/c19371144df3bb44fab255c43d04cbc2ab54d1c4/src/restore.ts).
- [Cache save path](https://raw.githubusercontent.com/Swatinem/rust-cache/c19371144df3bb44fab255c43d04cbc2ab54d1c4/src/save.ts).
- [Automatic key inputs](https://raw.githubusercontent.com/Swatinem/rust-cache/c19371144df3bb44fab255c43d04cbc2ab54d1c4/src/config.ts).

## Concrete owner decision: existing CI cache-key input

Approved decision: add only the existing setup action's cache-key input:

```yaml
cache-key: consolidated-${{ hashFiles('.github/workflows/ci.yml', 'scripts/invoke-consolidated-verification.ps1') }}
```

The exact originally proposed patch, candidate workflow and base/candidate SHA-256 identities
are retained in the accounting archive under cache-profile-proposal. The reviewable
local patch is target/ci-verify-r1/cache-profile-proposal/proposal.patch.
The owner approved this exact patch on 2026-11-19 with the response "yes".
It is now applied with the approved resulting workflow hash. The tracked owner
decision record preserves the response, patch hash and implementation scope.
The Architectural Approval Gate required this decision before implementation.

Affected system: only the existing CI dependency cache selection/population. Keep
all action pins, cache options (including workspace-crate exclusion), profiles,
feature selection, correctness/fitness gates, security and branch policy unchanged.
The key is stable across ordinary Rust source edits but changes when the workflow
or runner containing CLI profile/feature arguments changes. Non-profile workflow
or runner edits will also invalidate it; that bounded extra miss is a tradeoff of avoiding a new profile
configuration abstraction.

Alternatives: retain the stale key and repeated rebuilding; manually bump a key
without future profile invalidation; move CI profiles into global Cargo settings
with broader local effects. Recommend the existing input plus workflow/runner hashes: smallest
scope, no new cache manager or production cache, and a direct relation to the
missing cache input. Implementation is owner-approved; measured benefit and
final performance disposition remain pending warm-CI evidence.

Validation after approval: a normal CI run populates the new key. A subsequent
warm CI run must demonstrate cache reuse, fewer dependency compilations and lower
enclosing build cost while complete correctness still passes. A cache-key change
cannot make the first cold run free, and workspace code still rebuilds under the
existing defaults. No promised return to two minutes or speedup is established.
Any deliberate extra full-CI comparison remains operator-owned and requires its
execution authorization; no agent-run benchmark or local full suite is proposed.

## Remaining evidence limits and continuity

The older approximately 108-minute concern cannot be retrospectively partitioned
into exact per-test, fixture-construction and empirical-campaign seconds from
logs that did not retain those measurements. Earlier historical step durations,
long-test lower bounds and candidate experiment receipts remain discoverable;
they are not relabeled as complete CPU/causal profiles. Current aggregate phase
accounting and timestamp intervals do not fill those historical gaps.

This report uses retained, ready/complete source acquisition from
admission-20261118-173649. VERIFY-001, AUTH.FINAL_VERIFICATION,
PERF-004-DOCTRINE and PERF-005-BOUNDED component source hashes were compared with
current files before authoring. No live agent-started runtime or graph admission
is claimed. New/modified mission artifacts are covered by VERIFY-001; actual
admission and repository guards remain required before publishing this update.
The cache-key correction is implemented under owner approval; other candidates
remain observations. The broader mission is not closed merely because #414 passed
or because the cache key changed. No extra local Cargo campaign is required for
this workflow input and evidence-only change.
