# OPS-R1 — Disposition 22 fault-detection effectiveness review

Agent MaxHeadRoom — 2026-11-20. Static analysis and unapplied proposals only.

## Findings

Reviewed the selected public fitness test, fourteen native controls and eleven
cross-consumer controls: 26 exact tracked identities, all clean-ctx/lib with
all features. This is the bounded selected corpus, not a manual audit of the
whole repository. No new defect-detection experiment was executed.

1. The public fitness test has historical RED/GREEN evidence for repeated
   complete-project acquisition: [4,4,16] versus [1,1,4], with four nominated
   jobs/publicly reported compiled owners and the same empty lower-bound answer.
   That is demonstrated detection of this historical architectural defect.
2. Corrected case 9 directly compares acquisition bases, in addition to testing
   reuse/refusal outcomes. The prior currentness-masking warning is valid for
   the acquisition outcome, but incomplete as a description of the test's total
   detection strength: its explicit key inequality can detect a lost strict
   option distinction before qualification. This remains a static prediction.
3. Some controls do not prove that their purported shared branch was reached.
   Independent acquisition can satisfy source-role, missing-module or unit-receipt
   output checks. Passing those controls is preservation evidence, not independent
   demonstration that every shared-proof predicate works.
4. Cross-consumer coverage is uneven. The selected C# test checks preparation
   count, not semantic results; the recovery fixture creates no tsconfig; the
   selected retirement test covers reappearance rather than confirmed deletion
   or acquisition denial. Existing sibling tests can fill some gaps without
   adding another framework or indiscriminately increasing test count.
5. No finding here establishes a new production defect, a memory leak, the cause
   of the original field hang, or overall production-scale performance benefit.

## Matrices, anchors and classification

The complete per-control review is split into two linked matrices under
`verification/ops-r1/proposals/shared-acquisition/review-22/`:

- `control-invariant-matrix.csv`: exact identity, package/target/features, source
  anchor, invariant, fixture, observable assertion and production anchor.
- `control-defect-matrix.csv`: plausible defect, independent masking safeguards,
  evidence strength, mechanism-versus-outcome limits and overlapping coverage.
- `review-index.json`: inspected source hashes and review artifact hashes.

Each matrix has exactly 26 matching rows. Test anchors locate the selected
function, not a guessed file or broad filter. Production anchors identify the
implementation read for that claim; entries without line numbers refer to the
named lifecycle module rather than pretending a single predicate proves it all.

Evidence classes are deliberately nonexclusive:

| Code | Meaning |
| --- | --- |
| H | Demonstrated by the named historical RED/GREEN counterexample |
| E | Observable assertions supported by retained executions |
| S | Detection of the proposed fault supported only by static reasoning |
| M | Another safeguard or fallback can mask the named mechanism defect |
| U | Selected coverage does not establish the claimed dimension |

Only the fitness row is H for the historical acquisition defect. E never implies
that a proposed mutation was killed. Overlap is recorded per row; it does not
authorize deleting a test.

## Mandatory configuration-key investigation

Relevant implementation:

- `src/ir/typescript_resolution/shared_acquisition.rs:150`: key construction.
- `src/ir/typescript_resolution/project_config.rs:151`: compiler option value
  and declaring-directory origin acquisition.
- `src/ir/typescript_resolution/project.rs:155`: effective options retained.
- `src/ir/typescript_resolution/program.rs:121`: key/slot/qualification flow.
- `src/ir/typescript_resolution/shared_acquisition.rs:173`: proof sealing.
- `src/ir/typescript_resolution/shared_acquisition.rs:201`: recipient qualification.
- `src/ir/typescript_resolution/host.rs:68`: disk content/hash currentness.
- `src/tests/ir/ops_r1_shared_acquisition_isolation.rs:8`: corrected case 9.

The key includes config identity, project root/base, mode, paths, conditions,
effective options with origins, ownership, compiler version, extensions, request
directory/extension, authorized roots/aliases and demand policy. Between the
second and fifth requests, the fixture changes only strict=true to strict=false
in the same config; roots, source bytes, source path, project selection and option
origins stay equivalent. Strict is carried in options rather than another
derived key field for these explicit bundler/module settings. This is static
source reasoning, not newly captured serialized-key telemetry.

Independently of key construction, config disk reads record Content witnesses.
`qualify` traverses the old proof's inputs and requires witness.current(). The
config is not a supplied source seed, so the own-seed Content exception does not
skip that config witness. Rewriting its bytes therefore rejects an old candidate
if a bad key causes a collision. Missing/Directory/Physical and source-byte checks
provide other independent rejection dimensions, but need not change for strict.

**Two different conclusions are necessary:**

- [4,4,8] and no extra accepted qualification do not identify which guard refused
  the changed request. They alone cannot prove key discrimination.
- The preceding `assert_ne!(basis(...), second_basis)` directly checks key
  discrimination. A mutant collapsing this strict distinction can fail there
  even while production currentness remains fully enforced. The passing accepted
  test establishes inequality for its fixture; it does not establish detection
  of every possible missing option/origin/other key field.

No key values or field-by-field delta were emitted in the retained execution.
For a future experiment, independently verify the fixture preconditions: parse
the two basis JSON objects, confirm all fields except options agree and strict
value is the sole relevant option difference. A separate hypothetical mechanism
probe is not permission to alter the accepted fixture now.

There is a subtle experimental hazard: globally deleting options may also collide
the *first* absent-strict context with the second true context. An old stale slot
can then cause unnecessary reacquisition of the fourth dot-path request. Such a
mutation may be detected at the positive work assertion before the intended
negative-key assertion. That is a killed mutant, but not clean attribution to the
negative control. The selective proposed mutant below avoids that earlier collision.

## Ranked proposed experiments — none applied or executed

All experiments require separate owner approval, separately pinned isolated source
and build identities, unchanged accepted source/evidence, exact test selection,
the existing bounded capture controls, and attributable nonzero assertion failure.
Compiler errors, timeouts, resource stops and missing tests are invalid detection
evidence. Never weaken currentness or source authority to make a key defect visible.

| Rank | Proposal | Expected independent observation | Risk/information |
| --- | --- | --- | --- |
| 1 | M3: collapse only strict=false onto strict=true while constructing the key | Original corrected case reaches fifth request and fails its explicit basis inequality; accepted source retains its known pass | Narrow grouping-only fault; highest information about this negative predicate |
| 2 | M2: disable operation slot selection | Existing positive receipt control should fail [1,1,2]; corrected case should fail equivalent-route [3,3,6]; do not rerun public fitness unless needed | Simple work-regression fault; historical fitness already demonstrates broader no-reuse detection |
| 3 | M1: replace options key value with null | Existing test should fail, possibly earlier at equivalent-route reacquisition because first/second contexts collide | Broader fault with poorer attribution; defer if M3 answers the question |
| 4 | Add reachability assertions to existing sharing-related controls, then challenge one guard at a time | First establish accepted reuse or sealed-proof attachment; require exact observable failure of the chosen mutation | Test-only proposal; no fixture or instrumentation changes authorized now |

Exact **unapplied** patches and their accepted-file/mutant-file hashes are in
`proposed-experiments.json` beside the matrices:

- `M3-collapse-strict-false.patch`: `af2bc315dccfee30a64d193f3d27fa569ffdb8ccfa77d11a887ff84b31eb8d88`.
- `M2-disable-sharing.patch`: `1eaef479f9e772769355c02bc89da6acbe5a3a425763953b1aff674868e7c7f2`.
- `M1-omit-options.patch`: `d0df1db5279252a04cad9b25a33f12b61655466b58f10507c73809dbd42ea995`.

M3 clones the selected Project only for provisional key calculation, changes false
strict to true in that clone and recurses once to the normal key calculation.
It does not change acquired config bytes, currentness predicates or actual project
selection. This models one omitted effective-value distinction, not all possible
key omissions. No accepted source or production safeguard must be weakened for
this experiment: only a separately approved isolated mutant would contain it.

## Cross-consumer review and important gaps

| Boundary | What selected evidence protects | Important gap in this selection |
| --- | --- | --- |
| Resolver registration | Empty absent-route work predicate; positive keys/hashes/spans across four forms and both directions | Positive helper removes second route owner; shared proof and executable eligibility not established by that positive control |
| Bootstrap/workspace discovery | Six dependency mutations, stale restore refusal, recaptured public relation counts | No asserted sharing count, discovery completeness or error-specific restore reason |
| Explicit-file acquisition | Recovery still renders Target and establishes alias | No configured multi-owner sharing fixture; substring does not exclude stale extra facts |
| Ordinary hydration | Restart find_entities yields target count greater than zero | Exact semantic payload/currentness and shared-project branch not asserted |
| TS persistence/prepublication | Durable contributor Content hashes; exact producer/dependency failures; precommit leaf mutation refused | Historical restore and absent routing identity registration configurations not selected here |
| Retirement/reappearance | Commit recheck runs twice and public edges survive reappearance | Confirmed deletion, acquisition denial and sibling supports not executed by selected retirement case |
| Concurrent owner acquisition | Two synchronized owners share once; nested/same-config guard assertions | Not a systematic lock-order/interleaving or real nested-project recursion proof |
| Non-TS ordering/failure | Pools 1/2/4 return identical ordered Rust instructions; injected failure preserves neighbors | No proof workers actually overlap; no panic/cancellation assertion |
| Batch supersession | All candidates failed and first has no current projection | Not a complete durable transaction-state assertion for every owner |
| C# reconciliation | One preparation for two compatible exact query items | Does not assert per-item success, Hit/Go edges or Current authority |
| Ambient dependencies | Shadowed package content excluded; ambient identity retained after corrupting package leaf | Not a shared-operation qualification or ambiguity/lifecycle control |

Existing applicable siblings, identified but **not executed or accepted as part of
this selected evidence**, include:

- `src/tests/mcp/semantic_retirement_controls_issue_162.rs:46`: confirmed deletion
  with sibling supports/families; line 96 acquisition-error preservation.
- `src/tests/mcp/csharp_call_authority_lifecycle.rs:66`: warm exact call consumers
  require Current authority; this still does not replace exact edge assertions.
- `src/mcp/compatibility/validator.rs:152`: historical restore is a separate path
  requiring its own applicable existing-test mapping before claiming coverage.

No repository-wide absence claim follows from a gap in this 26-control selection.

## Overlap and recommendations

Low incremental-value assertions are explicit in the matrices. Standalone full
serialized equality subsumes its separate source_hash/current/disposition/reference
equalities, though those smaller checks localize failures. Unit counts alone can
miss wrong declarations; acquisition counters plus returned values are preferable.
Reappearance public queries can reacquire facts after a faulty state change; the
independent two-probe assertion helps, but a direct post-abort state observation
would strengthen it. Source-role and missing-module controls without a reuse
witness should not be advertised as guard-specific mutation coverage.

**Test recommendations only:** prioritize M3, reuse existing tracked controls,
then add minimal reachability or exact semantic-result assertions where the
matrices identify a concrete blind spot. Map existing deletion/error/historical
restore tests before proposing more tests. Keep failures attributable to their
exact faulty mechanism; surviving mutants require analysis, not weaker safeguards.
No test removal or new mandatory CI policy is proposed.

**Production recommendations:** no additional production correction is justified
by this static review. Do not redesign grouping, retirement, persistence or global
resource policy merely because a test is weak. Counterexamples are required first.

## Performance evidence and unresolved questions

Acquisition attempts/completions, complete-project unit visits, qualification
input visits, witness checks, nominated jobs, actual compilation stages and
publication outcomes are different quantities. `executor.rs:57` increments
full_semantic_parses by jobs.len() before worker execution; that counter alone
does not prove four successful full parses. The historical stage and public
response evidence must remain separately attributed.

The preserved fitness comparison records witness checks increasing 1980 to 2671,
elapsed time 2.495 to 4.720 seconds and GREEN qualification work [3,3,76,243,273],
even as acquisition work falls from [4,4,16] to [1,1,4]. These are single-fixture
captures, not a paired performance campaign. Preservation case durations likewise
prove no production-scale latency improvement, leak absence or original-hang cause.

Open questions: Does M3 fail at the intended key assertion? Which sharing guards
have independent detection after reachability is established? Which existing
historical restore/retirement siblings provide attributable coverage? Does total
qualification/currentness cost remain justified at realistic scale? What explains
the original field hang? None is resolved by assuming all passing tests are strong.

## Evidence, authority and stop

All Disposition 22 pinned continuation records, build receipt and executable were
rehashed and matched. The accepted source tree and reviewed source hashes remain
unchanged. Prior execution attribution remains eight passes on preceding GREEN,
corrected case 9 and sixteen passes on amendment 18; the original failed case 9
remains historical failed evidence. This is not a single new 25-test run.

This packet continues OPS-001 without modifying its protected declaration. PCG
preflight coverage remains unavailable through the configured tool catalog; no
new graph acquisition/admission is claimed. Applicable governing association,
graph reconciliation and document admission remain separate before publication.

Only analysis documents, matrices, hashes and unapplied experiment proposals were
authored. No production/test source changes, mutation application, tests,
benchmarks, builds, CI, protected grants, commits, pushes or publication occurred.
Stopped for review; experiments require separate authorization.
