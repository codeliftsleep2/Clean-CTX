# CI-VERIFY-R1 — Verification responsibility separation

Agent: **Agent MaxHeadRoom**. Candidate dated 2026-11-17, America/New_York.
Owner direction is preserved verbatim in the evidence archive. This candidate is
not B11 protected-authority authorization, admitted authority or successful CI.

<!-- pcg:CI-VERIFY-R1-decision:begin -->
## Decision and constraints

The owner approved eliminating duplicate correctness execution and separating
repeated empirical measurement. Every original semantic, frozen-equivalence,
deterministic-work, lifecycle, ownership and persistence assertion remains
mandatory. Fixture diversity and adversarial cases are retained. No production
code, performance threshold, sampling methodology, branch protection, required
job status, security permission or merge policy changes are proposed.

The single controlling authority remains [verification.md](verification.md).
Its candidate amendment requires actual predecessor-bound B11 owner authorization
before admission/publication. This document does not create that authorization.

Use existing Cargo/libtest mechanisms: list without executing, execute the
ordinary workspace/all-target/all-feature suite once with --no-fail-fast, then reconcile actual
target-qualified outcomes. Python validates evidence; it does not schedule tests.
The existing focused fitness runner and historical local-resume behavior remain
available outside the consolidated CI entry point.

The explicit empirical workflow selects enumerated measurement tests using
--ignored and --exact. Original fixture axes, warmups, alternating orders, sample
counts and reporting remain executable. Ordinary companions remain nonignored
tests. Measurement workflow failure is failure, not an allowed-failure step.
Separating workflow placement never retires an established production performance
acceptance prerequisite. Correctness/counting GREEN is not empirical improvement.
<!-- pcg:CI-VERIFY-R1-decision:end -->

<!-- pcg:CI-VERIFY-R1-baseline:begin -->
## Baseline and historical attribution

Run [44134968130](https://github.com/codeliftsleep2/Clean-CTX/actions/runs/44134968130)
belongs to b94273186a67d5f4ef8d3ec01209e71c69ecfc03, before this redesign.
It was not cancelled or reinterpreted. Elapsed job time through build/lint/format/guards was
approximately 101 seconds; dedicated semantic fitness passed in 545 seconds.
The full library suite then failed: 5,230 passed, 19 failed, 9 ignored;
5,028.75 seconds of execution. Audit was not reached. These 19 baseline failures
are preserved separately and are not resolved by execution separation.

The earlier Windows domain capture passed 159 root and 10 proxy tests, zero
failed/ignored, with all features, one test thread and both frozen oracles.
It is not equivalent to the later Linux full-suite run, which used default
libtest parallelism and lacked the dedicated stage's process-local oracle setup.

[The evidence index](../../verification/ci-verify-r1/evidence-index.json) pins
[the durable ZIP](../../verification/ci-verify-r1/baseline-evidence.zip), its exact
entries, acquired authority contents and first three mixed-companion receipts.
Returned graph content is preserved; acquisition is not proof of inspection of
every unrelated component.
<!-- pcg:CI-VERIFY-R1-baseline:end -->

<!-- pcg:CI-VERIFY-R1-membership:begin -->
## Exact membership and assertion preservation

[The reconciliation](../../verification/ci-verify-r1/membership-reconciliation.csv)
retains all 169 original domain identities, package, library target, feature mode,
source and historical configuration. Unchanged tests remain mandatory, without
reclassification solely from names or duration.

[Fitness membership](../../verification/ci-verify-r1/fitness-membership.json)
records exact identities for the 56 original groups, preserving complete
membership and minimums. One new mandatory enforcement test/group is added.
Each member must execute successfully once in its package/library target.
Fitness group attribution remains distinct within the consolidated result.

[Measurement membership](../../verification/ci-verify-r1/measurement-membership.json)
lists 18 new empirical identities with mandatory companions.
[Mixed dispositions](../../verification/ci-verify-r1/mixed-assertion-dispositions.json)
record each changed source and preservation boundary. Original fixture axes,
helper success checks, complete equality, work counters and failure/currentness
checks remain mandatory. Only repeated measurement and distribution reporting
are conditional. A4/edit companions retain a complete paired semantic evaluation
per original case; empirical paths retain all original rounds. Preserved assertion
site counts are static evidence, not behavioral-equivalence verification.

[Nine legacy ignored tests](../../verification/ci-verify-r1/legacy-ignored.json)
retain historical standing and are not newly promoted to mandatory coverage.
Other ordinary ignored tests are refused outside the exact empirical inventory.
Missing/duplicate execution, failed correctness/fitness, empty/incomplete
membership, missing empirical registration, ordinary execution of measurements
and invalid/missing/changed frozen oracles all fail validation.

The consolidated suite uses one libtest thread to preserve deterministic fitness
configuration. The focused runner retains its original local-resume source
validation. Optional historical replay roots and both mandatory frozen-oracle
variables remain configured and recorded. Different environments/features/oracle
settings are not treated as equivalent execution evidence.
<!-- pcg:CI-VERIFY-R1-membership:end -->

<!-- pcg:CI-VERIFY-R1-enforcement:begin -->
## Implementation and enforcement obligations

The existing check job/status calls the fitness runner with -Consolidated.
The duplicate cargo-test step is removed; that full workspace/all-target/all-feature
scope executes once inside the consolidated entry, with explicit oracle setup and
single-threaded libtest. All compilation, lint, formatting, tree-sitter, active-file,
encoding and audit obligations remain. Immutable action pins and permissions remain.

The validator compares compiler registration with actual outcomes per executable
and fully qualified identity. Cargo success cannot excuse missing/duplicate
outcomes. Required fitness membership is exact. Explicit measurements must be
registered and ignored ordinarily; their mandatory companions must pass. Complete
logs, configuration, source/oracle identities, receipts and scope are retained
under the existing evidence location and uploaded on failure.

The tracked src/tests/architecture/ci_verify_r1.rs exercises the actual validator
with synthetic adversarial evidence. Those fixtures verify enforcement, not the
actual execution of the synthetic regression names.

| Canonical question | Required witness | Current standing |
| --- | --- | --- |
| Can a required test disappear? | Exact membership and missing-registration/execution controls | Pending execution |
| Do correctness/fitness failures propagate? | Distinct failed-assertion controls and real CI receipt | Pending |
| Can absent/invalid mandatory oracles pass? | Exact prerequisite identities and refusal controls | Pending |
| Can ignored/empty/duplicate required selection pass? | Validator adversarial fixtures | Pending |
| Do empirical loops run ordinarily? | Ignored measurement outcomes and successful companions | Pending |
| Does explicit workflow reach preserved assets? | Actual registration plus explicit ignored-test selection | Pending |
| Is source/configuration attribution exact? | Fresh listing/execution and source/oracle inventory | Pending new execution |
| Is baseline evidence recoverable? | ZIP entries/hashes and original CI/source identities | Statically retained |
| Has CI improved? | Comparable post-admission Linux CI observations | Unrun |
<!-- pcg:CI-VERIFY-R1-enforcement:end -->

<!-- pcg:CI-VERIFY-R1-limits:begin -->
## Alternatives, governance and limits

Keeping both stages repeats tests with differing oracle environments. Deleting
the fitness runner loses membership, source attribution, receipts, scope and
resume contracts. Ignoring mixed tests omits mandatory assertions. Generic
sharding or new orchestration infrastructure does not address the responsibility
error and is outside this bounded implementation.

Both authority acquisitions were ready/current/complete at generation
9f54327cb8d494dd88ba525b9c72155eb46bc1ac9998b77593857008f375476e.
Applicable named performance, verification, engineering and completion source
constraints were inspected; broad discovery hits were not all treated as applicable.

The first three ordinary mixed companions passed with zero ignored in 66.76
seconds. Checkpoint-20261117-141734 then recorded 15 additional companion passes
and one enforcement-fixture failure in 144.40 seconds. The fixture wrongly used
a name suffix to mark optional measurements, misclassifying mandatory P0. It now
uses the exact approved empirical inventory; no P0 assertions or implementation
changed. The operator reported completion of the focused enforcement rerun;
its raw output was not supplied, so it remains operator-reported evidence.
These observations are not comparable CI performance evidence. Measurement asset
registration, scoped lint/guards, B11 descriptor, owner-only authorization, actual
admission, graph discoverability and final CI remain pending.
The 19 original full-suite failures remain separately unresolved.

The report governs associated CI-VERIFY-R1 manifests, scripts, tracked test
changes, workflows and evidence listed here. No other mission Markdown is created.
Publication waits for the real governance checkpoint and applicable verification.
No owner authorization is generated or submitted by the agent.
Agent MaxHeadRoom, 2026-11-18: CI run 44303364941 at c3a9cd303ed09ef683c286c8a71354f672538725
reported passing outer test summaries, including 5252 root-library passes and 27
approved ignores, but membership reconciliation failed. PowerShell merged stdout
and stderr out of order: the next Cargo header preceded 211 remaining library
registration lines. An inherited child libtest run split its parent outcome and
made its worker appear duplicated. The bounded parser correction associates ordered
Cargo headers with complete ordered outer libtest blocks, validates block record
counts and handles nested output only within an unfinished parent test. Required
registration, unique execution, ignored-test restrictions and fitness membership
remain enforced. No Clean-CTX server or MCP availability is required in CI.
Tracked enforcement fixtures cover reordered headers and nested workers alongside
existing failure cases. Historical evidence is retained in consolidated-parser-red.zip
with consolidated-parser-red-index.json. Corrected-parser replay and tracked
execution were initially UNRUN. Operator capture parser-repair-20261118-091856
subsequently reconciled all 5629 registered tests with 5629 outcomes and no errors.
The focused all-feature enforcement test passed (one passed, none failed or ignored),
with no compiler warnings reported. Exact logs, source bytes and hashes are retained
in consolidated-parser-green.zip and consolidated-parser-green-index.json. This is
replay of historical CI evidence plus focused current-source verification, not a
new full CI run. Scoped lint, guards and current document admission remain pending. This repairs evidence
attribution within the approved verification contract; it does not change test scope.

Agent MaxHeadRoom, 2026-11-18 runtime continuation: the owner reports numerous
individual tests exceeding 60 seconds. The retained CI library summary establishes
5949.39 seconds in total, but its buffered log does not establish individual costs.
A source-confirmed duplication was found: af002_a4_public_lookup_budget invoked
af002_a4_default_footprint_dimensions while both functions were registered tests.
The latter is now an unregistered private helper, still called by the mandatory
public-budget test. Every original matrix case, semantic assertion, work-count
assertion and final public-budget assertion is retained; the same 19-case setup
is no longer independently scheduled twice. The named public-budget fitness
membership is unchanged. Runtime benefit and patched execution are not yet verified.

The fixed five-test operator batch in verification/ci-verify-r1/invoke-runtime-triage.ps1
records separate compilation and per-test wall time, actual Cargo-selected executable
identity, exact test registration, source hashes and full output. Its 60-second
notices are diagnostic progress, not timeouts, skips, budgets or acceptance criteria.
It selects the A4 public budget plus C# skeleton degradation and delta combined frozen/unused-budget,
integrated and persistent correctness witnesses once each, with all features and one libtest thread. It does
not execute ignored measurement studies, change production behavior, reopen rejected
optimizations or claim these five tests exhaust all slow paths. Run 406 (44134968130, revision b94273186a67d5f4ef8d3ec01209e71c69ecfc03)
was subsequently pulled directly. It contains 37 named over-60-second warnings.
Warning-to-completion elapsed lower bounds include 1441 seconds for C# skeleton
degradation, 894.39 for delta frozen semantics and 789.73 for delta integrated cost.
These include shared-lock waits in its parallel run and are not CPU measurements.
The selection was corrected from these actual logs; it does not rely on
owner recollection or treat old timings as current-source results. Additional
repair selection awaits isolated cost evidence.
Source inspection also established that af002_delta_previous_frozen_semantics and
 af002_delta_previous_unused_budget each regenerated the identical 380-row corpus
under the same serial guard and Observation mode. The surviving frozen-semantics
test now invokes the unchanged unused-budget assertions before that observation
ends. Its original compare(records()) and all original fixture cases remain;
the second registration is removed. The fitness declaration now represents both
obligations in that surviving test, changing 57 groups to 56; the standalone
runner's duplicate selector is removed too. This is one execution of both required
contracts, not ignored coverage or a waiver. No production code changes. Exact
before/after membership is recorded in runtime-deduplication.json. Patched test
execution and elapsed benefit remain unverified.

The batch also verifies that neither duplicate corpus remains independently
registered, then executes the existing membership enforcement test once against
the revised declarations. Capture runtime-triage-20261118-100347 verifies all five selected mandatory tests
and revised membership enforcement. Compilation took 53.52 seconds; isolated
C# skeleton, combined delta frozen/budget, delta integrated, delta persistent and
A4 public budget took 5.77, 232.97, 49.65, 12.78 and 89.40 seconds respectively.
The C# test's run-406 warning is therefore not evidence of 24 minutes of isolated
work. These current-source Windows observations cannot be relabeled as Linux
CI costs or an old/new runtime comparison. Exact receipts and source identities
are retained in runtime-baseline-evidence.zip and runtime-baseline-index.json.

The next bounded trial uses the same two demonstrated expensive witnesses and
root test profile, with only an invocation-local Cargo override optimizing test
dependencies at level 3. It changes no repository profile defaults, production
source, fixture axes, assertions, debug-assertion settings or overflow policy.
Compilation cost and test elapsed time are recorded separately. The dependency trial subsequently passed in runtime-triage-20261118-102252.
Its clean rebuild took 322.98 seconds, and identical delta/A4 witnesses took
75.36 and 24.18 seconds (versus 232.97 and 89.40). Membership enforcement also
passed. Actual Cargo artifact metadata establishes 189 opt3 dependency artifacts
with debug assertions and overflow checks true, plus an opt0 main test crate
with both checks true. This is a material focused runtime reduction, with a
separately measured compilation penalty; it is not a full-suite or CI claim.
Exact evidence is retained in runtime-dependency-trial-evidence.zip and its index.

An invocation-local optimized-root trial additionally requests test opt-level=1,
retaining opt3 dependencies and the same two witnesses plus membership enforcement.
The main crate's Cargo profile metadata will establish whether debug assertions
and overflow checks remain enabled. The root trial passed in runtime-triage-20261118-104106: delta 56.50 seconds, A4 21.07 seconds,
and membership enforcement 5.39 seconds. Root rebuild cost was 339.34 seconds
with already-built optimized dependencies; this is not a cold total. Actual main
crate metadata is opt1 with debug assertions and overflow checks true. Source
manifests establish identical Rust source, Cargo.toml and Cargo.lock across all
three timing captures. These are bounded single-run observations, not a repeated
performance distribution or Linux-wide certification. Evidence is retained in
runtime-root-trial-evidence.zip and runtime-root-trial-index.json.

The owner-directed CI runtime correction adopts these invocation-local test
settings only inside the consolidated correctness runner: root opt1, dependencies
opt3, debug assertions true and overflow checks true. Listing and execution use
the exact same arguments, which are recorded in configuration.json. Local Cargo
profile defaults, production source and explicit empirical-study workflows remain
unchanged. The two observed test runtimes improve about 4.1 and 4.2 times relative
to the initial current-source capture, in addition to eliminating duplicate corpus
execution. Rebuild cost rises materially; the decision trades that measured cost
against repeated expensive execution across the full CI suite. A complete Linux
CI execution must establish the actual overall benefit and reveal any other slow
families. No one-to-two-minute CI promise or full slowdown resolution is claimed.
Scoped lint/guards, final source admission and fresh CI remain pending.

No live Clean-CTX service is a CI prerequisite. Full suite verification remains
CI-owned; the batch is proportionate operator investigation, not a substitute gate.

Agent MaxHeadRoom, 2026-11-18 historical runtime escalation: the owner rejected
70 minutes as an acceptable outcome against the earlier roughly two-minute test
phase. Direct GitHub job metadata establishes a concrete boundary: successful
run 392 at b5a152e8343808ab84c105d77b46458593608d03 spent 105 seconds in cargo test;
run 393 at 37f647dd57d91381b0af43d568ca381d8e3ef24b spent 2404 seconds and failed.
Downloaded console starts report 3433 and 3923 library tests respectively. The
workflow and Cargo profile did not change across these endpoints; Cargo.toml
changed only package version. Thus 14.3 percent more registered library starts
coincided with 22.9 times the test-step duration. That is a regression interval,
not proof of a specific causal change. The interval contains multiple semantic
changes and 142 changed test files with extensive public/lifecycle matrices.
Its older console downloads lack complete final summaries; successful completion
is attributed only to the job metadata, not reconstructed from partial output.

Successful run 410 at 063f672f3460a39621effef179fa5bb6c1f18835 spent 4113 seconds
in consolidated correctness before the optimized-profile/deduplication repair.
Run 411 at 95153d5c86c800b8eab523d9b980a44d44a57301 is the first live CI trial of
that repair and was in progress at inspection. Exact historical logs and step
metadata are preserved in runtime-history-evidence.zip and runtime-history-index.json.
No whole-suite speedup, return to two minutes or full resolution is claimed.
The measured profile improvement is a mitigation; root-cause and full-workload
verification remain open. Additional restructuring must preserve all mandatory
assertions and exact execution evidence, not obtain speed by skipping heavy tests.

Agent MaxHeadRoom, 2026-11-18 bounded query witness preparation: current
CoreIRPass composes capture_query(base) with binary-owned C# semantic or
TypeScript/NgRx query fragments, then invokes run_capture_pipeline_nodes. That
entry point unconditionally selects QuerySelection::Dynamic, whose shared
implementation invokes Query::new for each source. Existing RegisteredQuery
identities are not selected by this path. The historical implementation plan
SEMANTIC_QUERY_PERFORMANCE_ARCHITECTURE_IMPLEMENTATION_2026-10-16 explicitly
classifies core.rs as DYNAMIC_REQUEST and requires retained dynamic compilation.
Consequently a process-lifetime closed registration for complete IR queries would
extend that approved inventory; it is not silently authorized by a CI performance
concern. Generic/custom/edit queries must retain their existing dynamic behavior.

The proposed tracked witness ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query
warms the real CoreIRPass, then repeats the same source and binary query while
checking exact capture/IR equivalence, one fresh source parse, one fresh cursor,
and the proposed zero additional query-compilation budget. Its expected RED is
investigation of a new proposed budget, not proof that the current behavior
violates the old explicit disposition. The operator harness retains exact source
bytes, head, command, output, counts and elapsed observation. Capture core-query-witness-20261118-114634 subsequently establishes the expected
behavioral RED at head 95153d5c86c800b8eab523d9b980a44d44a57301: repeated CoreIRPass
execution took 0.8351601 seconds and performed one parse, one cursor construction
and one query compilation after warming. Capture and IR equality assertions passed
before the proposed zero-compilation budget failed (actual 1, expected 0). This
is not isolated Query::new CPU timing, a C# witness or proof of the full-suite share.
Exact command, source bytes, hashes and output are retained in core-query-red-evidence.zip
and core-query-red-index.json. No compiled-query reuse repair has been implemented. Significance for the full
CI runtime and safe classification of complete query variants remain to establish.

Owner decision required before production implementation: extend the existing
closed RegisteredQuery inventory to two complete binary-owned IR definitions,
TypeScriptIR and CSharpIR. Reuse the current immutable process-lifetime Query
mechanism; retain independent source parsing and mutable cursors, source/hash/
witness checks, producer identities, capture ordering and arbitrary/custom/edit
query behavior. Exact base definition and actual matching grammar must both be
established before selecting a registered identity. A mismatch stays dynamic.
Do not infer an identity from text prefixes or cache caller-supplied query strings.
The complete existing composition remains owned by CoreIRPass and is relocated
verbatim to a small cohesive helper to preserve the active-file ceiling.

The unapplied review patch at target/ci-verify-r1/core-query-proposal/proposal.patch
is also retained in the RED archive. It is UNCOMPILED and is not represented as a
verified resolution. It touches capture_pipeline.rs and core.rs plus one query
composition helper, all below 615 lines. No source artifact, semantic result,
currentness decision, session cache or persistence representation is reused.
Alternatives are continued dynamic compilation (retains current cost), a
caller-string cache (rejected due unbounded identity/policy expansion), or reducing
coverage (not acceptable). Recommendation: approve only the two closed definitions,
then establish exact-source GREEN, cold/warm/concurrent capture equivalence,
dynamic/custom/mismatched-language preservation and focused end-to-end timing.
This is a shared production boundary, so AGENTS.md's Architectural Approval Gate
and the prior explicit dynamic disposition require owner authorization before
applying the proposal. No protected-authority authorization is generated here.

<!-- pcg:CI-VERIFY-R1-core-query-approved:begin -->
## Owner disposition 02 implementation: preparation, not verified completion

Agent MaxHeadRoom. The verbatim owner disposition is retained as evidence. It
approves exactly TypeScriptIR and CSharpIR through the existing OnceLock boundary,
not general query-text caching. The original RED source and execution remain
frozen in core-query-red-evidence.zip; the current witness source is unchanged.

Dependency review rejected the proposed compression-to-CoreIRPass callback as an
unnecessary reversal of producer/primitive ownership. The smallest compliant
arrangement places immutable query constants and exact composition in the existing
queries namespace, consumed by both the IR producer and capture engine. Six
constant bodies are relocated unchanged; the existing IR call-query helpers
delegate to the shared binary definitions. Test-only aliases retain applicable
existing fixture consumers. No compression-to-IR/Angular producer dependency is
introduced, and no generic registry, injection mechanism or new cache architecture
is created. Frozen pre-relocation complete .scm definitions protect composition
bytes and ordering independently of the new builder. Source-independent inputs
are solely compiled constants; runtime config, fidelity, source, workspace and
source evidence are not inputs to the cached Query.

Only exact base text plus the actual matching grammar selects either new
registration. The shared capture walk is unchanged. Other base definitions,
custom/invalid text, whitespace-modified bases and mismatched grammars keep the
existing dynamic composition and error behavior. Existing cached Result<Query,
QueryError> lifecycle is retained; no retry, eviction, global map, tree or cursor
reuse is added. Added tests cover frozen complete definitions, both languages,
existing NgRx and declaration/invocation fixtures, distinct sources, five fidelity
levels, warm canonical IR and fresh parse/cursor work, dynamic/custom/error and
mismatched grammar behavior, disabled-feature refusal and concurrent process-cold
initialization with distinct concurrent source observations. Cached failure
retention remains the existing OnceLock<Result> implementation; valid immutable
complete definitions are not replaced by deliberately invalid production identities.

The prior plan's prospective amendment is appended without rewriting its earlier
inventory. Its governed coverage and the performance doctrine qualification are
explicit; the protected qualification requires a new B11 descriptor and owner-
operated authorization. No agent-generated owner authorization is permitted.
Direct host PCG tools are unavailable in this session; controlling source text was
acquired locally. Runtime graph reconciliation/acquisition remains operator-owned.

At initial preparation patched compilation, tracked test execution, feature-mode
execution and actual admission were UNRUN. Capture core-query-green-20261118-121354
subsequently compiled without reported warnings and ran eight all-feature tests:
seven passed, including the exact original RED witness with warm query compilations
0, one parse, one cursor, unchanged capture/IR and 0.000362-second enclosing time.
Definition equality failed because the newly frozen .scm fixtures had acquired
CRLF checkout spelling. Both LF SHA-256 values still matched their original frozen
identities. The two paths are now pinned eol=lf and restored to the original LF
bytes. No equality assertion, query contribution, order, production source or
original RED source was weakened. Only the definition test must repeat in all-
feature mode; the not-yet-run two restricted-feature selections and actual B11
descriptor remain pending. This capture remains a 7-pass/1-failure result. The focused handoff batches only these
controls plus the existing static-query regression. Additional compile/timing/RSS
attribution remains separate from deterministic correctness, with no extrapolation
of the earlier 0.835-second enclosing observation to total CI savings.

## CI-VERIFY-R1 closure accounting remains open

The owner's three categories are separate: (1) production regressions such as
repeated complete IR query compilation; (2) verification architecture work such
as duplicate selection and repeated Cargo invocation; (3) repeated empirical
campaign execution mixed into correctness. A repair in one category does not
close or conceal an obligation in another. Existing required assertions, fixture
diversity, lifecycle, source/currentness and performance acceptance remain mandatory.

Closure requires a source/configuration-pinned ledger for per-test and per-family
elapsed time; query compilations; repeated fixture/workspace construction;
duplicate executions; historical 24-25-pair campaigns; remaining expensive
production work; compilation and enclosing CI cost. Historical missing timings
must be identified as unavailable, never invented from buffered output. The
existing trials and run histories establish only their recorded scope. Selective
remediation must also apply to the fitness infrastructure itself. A baseline-
comparable corrected full CI run, not merely a fast focused subset or moved
research campaign, is required before overall completion can be claimed.
### Feature-mode blocker discovered, 2026-11-18

Capture core-query-green-20261118-122039 passed the frozen complete-definition
check after the LF fixture correction. The following C#-only build failed before
any selected test could execute: 63 compile errors and 17 warnings. The exact
command and diagnostics are retained with the prior capture in
core-query-verification-evidence.zip and core-query-verification-index.json.
All eight all-feature controls have now passed across the two captures; the
original seven-pass/one-failure run is not relabeled as an eight-pass execution.

Static source comparison against committed 95153d5c establishes that all eleven
diagnosed error-source files are unchanged. core-query-feature-blockers.json
retains their exact errors and normalized path comparison. Examples include
ungated Angular modules importing feature-gated ir::typescript_resolution,
a template module referencing tree_sitter_html without its feature, an ungated
provide_angular re-export, and an existing SQLite test accessing a field absent
without TypeScript. This is source-confirmed pre-existing broken feature wiring,
not a successful baseline-only compilation run; none was performed. The capture
ran against the candidate and remains an actual failure.

Repairing those shared feature boundaries is outside the approved two-query
registration extension and would affect additional Angular, MCP, diff, identifier
binding and test module families. No such repair, feature implication, warning
suppression or gate weakening has been implemented. Required disabled-feature
proof remains blocked; TypeScript-only execution and B11 describe were not reached.
Per Owner Disposition 02 section 9, return this exact scope decision before any
material feature-architecture expansion. Recommended disposition: separately
authorize the smallest correct feature-gating repair, preserving all-feature
behavior, then resume the existing focused modes and B11 checkpoint. The query
repair must not be represented as fully verified or publishable meanwhile.

<!-- pcg:CI-VERIFY-R1-feature-isolation:begin -->
### Owner disposition 03: bounded feature isolation, preparation 2026-11-18

The repeated disposition matches the retained owner authorization exactly. The
verbatim source is feature-isolation-owner-disposition-03.txt. Existing support
is established by Cargo.toml's independent language grammar flags, the existing
meta-layer implications (angular -> typescript; dotnet -> csharp), CONFIGURATION.md's
compile-time feature table, ARCH-002's feature-gated language registration, actual
LayerRegistry guards and safe_*_language returning None when unavailable. This
repair neither defines a new feature matrix nor changes Cargo.toml/Cargo.lock.

The exact historical 63-error C#-only receipt remains in the existing verification
archive. feature-isolation-repair-ledger.json classifies every diagnostic once:
52 from four TS-dependent Angular module owners and their provider-syntax child;
two from HTML-only implementation availability; four from Angular-specific MCP
and diff dispatch; two from a pure syntax-helper registration mismatch; three
from an unconditionally registered TS-specific SQLite witness. These are five
root causes, not 63 independent repairs. The initial 17 warnings identify related
consumer/import applicability; their correction is separate from compiler-error
classification. Source-confirmed baseline defects remain distinct from any future
compiled baseline comparison; no such baseline invocation has been performed.

The selected corrections gate the four authority-dependent Angular modules with
typescript, the template compression module and HTML parsing functions with
angular, and Angular dispatch branches/imports with their producer feature. Shared
HTML data shapes and the existing generic non-HTML dispatch/fallback remain.
The diff's original template body is retained in a local Angular-gated helper;
its source-before-template priority and existing text fallback are preserved.
The pure Node binding-pattern helper is registered with its already unconditional
grammar-optional adapters; it loads/registers no grammar and introduces no TS
feature implication. Only the TS-specific SQLite public witness is gated; generic
SQLite tests remain. C# ancestry forwarding for the .NET meta path now matches
its existing dotnet call-site guard.

Test-only metric exports/accessors and meta-pass fixture helpers are gated with
their actual framework consumers. Cross-language tests retain their original
language guards. No allow attributes, fabricated results, placeholder extractors,
new feature dependencies or producer generations are introduced. All-feature
bodies/assertions remain applicable; historical GREEN is not promoted to current
post-repair verification. A tracked HTML dispatch control checks real template
semantics with Angular and the existing text-only path without it. Existing query
controls now also assert actual language and meta-layer registration against the
compiled feature configuration.

The operator handoff first collects C#-only and TypeScript-only results, then
checks the directly affected existing angular-only and dotnet-only implications
and all-feature controls. It is bounded to these repaired boundaries, not a
combinatorial or full-suite campaign. Tests use exact nonempty selection and
zero-warning checks. All patched compilation and runtime verification is UNRUN
at this preparation checkpoint. The original RED source is byte-preserved.
Protected-authority B11 authorization and actual admission/discoverability remain
pending; no owner authorization is generated or submitted by the agent.

The first feature-isolation capture, `feature-isolation-20261118-125322`,
failed compilation before executing selected tests: C#-only reported one error
and seven warnings; TypeScript-only reported 24 errors and six warnings. The
C# error was introduced by over-guarding the shared Rx regions export and has
been corrected in source. The TypeScript diagnostics exposed C#-specific test
registrations and native EF/alias evidence compiled without C#. The bounded
correction gates native evidence and EF projection while retaining the shared
catalog, graph traversal and existing unavailable-grammar behavior. Observer
and import guards follow their actual feature-specific consumers. Cargo feature
definitions and dependencies are unchanged. These are implemented corrections,
not verified resolutions. Both isolated configurations and the remaining
focused configurations still require operator verification. The exact failed
logs and source-pin receipt are retained in
`verification/ci-verify-r1/feature-isolation-red-evidence.zip`; member hashes and
cause classification are associated by the governed repair ledger. Historical
all-feature results remain attributable only to their original source pins.

The subsequent capture `feature-isolation-20261118-132301` compiled C#-only
and passed all seven selected tracked controls, but emitted 107 warnings; this
is behavioral evidence, not zero-warning acceptance. TypeScript-only failed
compilation with 46 errors and two warnings. Newly exposed causes were the
C# semantic module, a shared roots export incorrectly restricted to C#, the
registration of a pure package fixture, and a second generic candidate-index
witness initializing native alias evidence. Those source corrections are
implemented but unverified. The archive retains both failed captures; the
repair ledger records the exact warning inventory. Remaining feature-specific
unused implementation and test-support ownership must be reconciled before
another acceptance attempt. No warning suppression or acceptance exception is
authorized or introduced.

The follow-up static reconciliation covers the compiler-reported 107 C#-only
warnings by exact captured diagnostic in the repair ledger. Test hooks and
observers now follow the features that register their consumers. Native NgRx
proof production, compiler-owned fields and meta-context propagation follow
TypeScript availability; generic object carriers and acquisition helpers remain
available. Angular template parsing/presentation helpers and feature-owned
producer generations follow their existing registrations. No warning
suppression or Cargo feature/dependency change was introduced. The existing
`PassError` definition moved unchanged from `src/ir/pipeline.rs` to
`src/ir/pipeline/error.rs`, with its original public re-export preserved, to
keep the active pipeline file below 615 lines after the feature guards. This
is a cohesive file-size relocation, not a new error mechanism. The ledger and
this governing statement cover that source correction. Formatting,
post-correction compilation, focused execution and actual document admission
remain pending. The earlier seven C# successes do not verify these later edits.

Capture `feature-isolation-20261118-145353` failed before selected tests:
C#-only reported 20 errors from the omitted producer guard after the NgRx
fields were guarded; TypeScript-only reported one error because the whole
.NET semantic module had been guarded while a grammar-independent testing
helper still consumed its class-name extraction function. Both were introduced
by the attempted feature correction. The native producer now follows the same
TypeScript guard as its owned fields. The semantic module and pure helper retain
their existing paths; only C#-native functions and imports are guarded. Existing
C# identity constants moved unchanged into the existing native `known_types`
helper. The source correction is implemented, not verified. The absence of
warnings before these compilation failures does not establish zero-warning
acceptance. Exact logs, source pins and commands remain in the follow-up evidence
archive identified and hashed by the repair ledger. Earlier successes retain
their historical source identities.

Capture `feature-isolation-20261118-150244` compiled TypeScript-only and
passed all seven selected tracked controls, but reported 117 warnings. C#-only
failed compilation at two shared discovery calls into the over-guarded NgRx
discriminant module. Discovery is restored at its existing grammar-optional
boundary; only native evidence construction is guarded by TypeScript. The
obsolete native identity glob import is removed. Additional Angular-only
projection/admission definitions and C# query instrumentation have been aligned
with their consumers. This remains an implemented, unverified correction.
Native proof-field consumption and feature-specific observation ownership still
require reconciliation; no new operator acceptance handoff is ready. The exact
capture and complete TypeScript warning inventory are retained by the fourth
capture record. The historical seven successes do not verify subsequent edits.

The post-fourth-capture reconciliation preserves native proof production rather
than dropping evidence merely because Angular interpretation is disabled.
`src/tests/ir/native_feature_evidence.rs` checks source-currentness refusal and
exact provenance for native discriminant and service-dispatch records through
the real CoreIRPass. The existing inject initializer matrix now checks its
optional-call qualifier, and the existing EF marker fixture checks authorized
HasEntity projection using the same source. Angular-only metadata fields,
projectors and admission observations follow Angular availability; C#-owned
capability state and observations follow C#. Generic helpers, source acquisition
and native producer behavior remain available at their existing boundaries.
These are implemented corrections and tracked obligations, not passed tests.

The focused handoff selects 8 tests in C#-only and .NET-only, 10 in TypeScript-only
and Angular-only, and 12 with all features. Counts differ because native TS tests
require that grammar and the frozen warm-query witness plus original static
registration control are included in the all-feature phase. Formatting precedes
source pinning; every configuration requires complete nonempty selection, exit
zero and no warnings. The script still collects both isolated modes before
stopping on a failure. No broad suite or measurement campaign is added. The
original RED witness and frozen complete-query definitions remain byte-identical.
Post-reconciliation execution and document admission remain pending.

Capture `feature-isolation-20261118-153849` passed all ten selected
TypeScript-only controls, including native proof ownership, optional-call
provenance and EF relationship preservation. Five unused imports still blocked
acceptance. C#-only failed before tests because the shared
`remember_semantic_projection` method had been over-guarded for Angular. Its
C# production reconciliation consumer requires the shared C#-or-Angular guard.
That correction and the five import guards are implemented but unverified.
The fifth capture archive, source pins and hashes are retained through the
repair ledger. No warning suppression or new failure fallback was introduced.

Capture `feature-isolation-20261118-155508` established TypeScript-only
10/10 selected tracked tests with exit zero and no warnings. C#-only passed
8/8 with exit zero, but three dead-code warnings remained: native initializer
span/collect helpers and the private meta-context initializer field. Those
helpers, associated imports, and the private field/initializer now follow
TypeScript availability. Generic PassContext data is retained. The sixth
archive preserves exact logs and source pins through the repair ledger. This
is a clean historical TypeScript result and an implemented follow-up correction;
post-edit acceptance of all five configurations and actual admission remain
pending.

The source-pinned capture `feature-isolation-20261118-160057` resolves the
focused feature-isolation obligations: C#-only 8/8, TypeScript-only 10/10,
Angular-only 10/10, .NET-only 8/8 and all-features 12/12, all exit zero with
no warnings and complete nonempty selection. Current formatted Rust sources
were compared against every receipt pin before archiving, with no drift. Exact
commands, source hashes, logs and descriptor are retained in the GREEN archive
and its index. The unchanged original warm-query witness measured zero query
compilations, one parse and one cursor (0.0005101 seconds) in its warm operation.
This establishes focused implementation/feature correctness and deterministic
query work, not a new full-CI runtime result or complete performance accounting.

The descriptor reports `E_GOVERNANCE_AUTHORIZATION_REQUIRED`; read-only
inspection is not admission. Recording these results changes the candidate
documentation, so the descriptor must be refreshed after scoped lint and
repository guards before owner review. Reuse
`verification/ci-verify-r1/invoke-checkpoint-continuation.ps1` for the remaining
registration-only check, scoped library/test Clippy, UTF-8/active-size guards
and read-only descriptor. It does not rerun these focused tests or a measurement
campaign. Owner authorization, actual admission/discoverability, guarded branch
delivery, CI and the remaining complete runtime accounting stay open.

Checkpoint `checkpoint-continuation-20261118-161108` passed registration-only
validation of all eighteen measurement assets, then failed scoped Clippy solely
on `vec_init_then_push` in the query test case constructor. The helper now uses
a vector literal with the same feature predicates, tuple expressions and order;
no assertions, source fixtures or required cases changed. The exact pre-style
source and failed Clippy receipts are retained in the style-evidence archive.
The five-mode runtime GREEN remains historical evidence of its exact source
pins; this subsequent style correction has static expression-preservation
evidence and requires the remaining scoped checkpoint. No warning suppression
or test assertion weakening is introduced.

Checkpoint `checkpoint-continuation-20261118-161906` passed scoped library/test
Clippy with all features and warnings denied, UTF-8 validation, active-file
limits and registration of eighteen measurement assets. This includes compilation
of the formatting-only case-constructor correction; prior runtime results retain
their original source hashes. The owner recorded the exact procedural grant for
digest `b6ee555e27f4bdffd456c80b1f64d0664ae9dca9cfc32672b71570fa8957d737`.
The accepted membership and protection sets were unchanged; the protected content
change was the shared performance-standing qualification of PERF-004-DOCTRINE
and PERF-005-BOUNDED.

Actual operator admission `admission-20261118-162951` acquired VERIFY-001,
VERIFY-002, AUTH.FINAL_VERIFICATION and both performance nodes in generation
`d64fb91b0cda1c475a5e58dd8a1157f72a2d2da2d2a173f0a97cee492acbf69f`.
All five contexts and the exact source acquisition were ready, complete and
catalog-current. Exact checkpoint/admission receipts are retained by the admission
index and archive. This final mission-record update is subsequent to that capture;
normal admission must reconcile this unprotected record before delivery. Reuse
the existing admission driver with a fresh source-pin review. No protected content
or membership is amended by this receipt update.

The query registration and bounded feature-isolation correction are ready for
branch delivery after final metadata guards/admission. This is not closure of
CI-VERIFY-R1: full CI and the requested per-test/family runtime accounting,
production work attribution and corrected-CI comparison remain outstanding.

<!-- pcg:CI-VERIFY-R1-feature-isolation:end -->

<!-- pcg:CI-VERIFY-R1-core-query-approved:end -->

<!-- pcg:CI-VERIFY-R1-limits:end -->

```architecture-graph
node: VERIFY-001
kind: IMPLEMENTATION
status: INVESTIGATING
title: CI verification responsibility separation candidate
track: REPOSITORY
artifact: docs/agent/CI_VERIFY_R1.md
depends_on:
  - AUTH.FINAL_VERIFICATION
  - PERF-005-BOUNDED
  - PERF-004-DOCTRINE
preserves:
  - ARCH-004
uses_evidence:
  - VERIFY-002
note: Approved bounded direction; protected authority admission and complete verification remain pending.
knowledge_scope: decision
knowledge_content:
  - statement self#CI-VERIFY-R1-decision
  - observation:baseline self#CI-VERIFY-R1-baseline
  - scope:membership self#CI-VERIFY-R1-membership
  - verification_obligation self#CI-VERIFY-R1-enforcement
  - qualification self#CI-VERIFY-R1-limits
  - evidence:fitness verification/ci-verify-r1/fitness-membership.json#whole
  - evidence:measurement verification/ci-verify-r1/measurement-membership.json#whole
  - evidence:reconciliation verification/ci-verify-r1/membership-reconciliation.csv#whole
  - evidence:preservation verification/ci-verify-r1/mixed-assertion-dispositions.json#whole
  - evidence:configuration verification/ci-verify-r1/membership.json#whole
  - evidence:legacy verification/ci-verify-r1/legacy-ignored.json#whole
  - evidence:runner scripts/invoke-consolidated-verification.ps1#whole
  - evidence:validator scripts/verify-ci-membership.py#whole
  - evidence:empirical_runner verification/ci-verify-r1/invoke-measurements.ps1#whole
  - evidence:enforcement src/tests/architecture/ci_verify_r1.rs#whole
  - evidence:parser_red verification/ci-verify-r1/consolidated-parser-red-index.json#whole
  - evidence:parser_green verification/ci-verify-r1/consolidated-parser-green-index.json#whole
  - statement:core_query_disposition self#CI-VERIFY-R1-core-query-approved
  - evidence:core_query_owner verification/ci-verify-r1/core-query-owner-disposition-02.txt#whole
  - qualification:core_query_history docs/plans/SEMANTIC_QUERY_PERFORMANCE_ARCHITECTURE_IMPLEMENTATION_2026-10-16.md#CI-VERIFY-R1-core-query-amendment
  - evidence:core_query_results verification/ci-verify-r1/core-query-verification-index.json#whole
  - qualification:feature_build_blockers verification/ci-verify-r1/core-query-feature-blockers.json#whole
  - observation:feature_isolation self#CI-VERIFY-R1-feature-isolation
  - statement:feature_isolation_owner verification/ci-verify-r1/feature-isolation-owner-disposition-03.txt#whole
  - evidence:feature_isolation_causes verification/ci-verify-r1/feature-isolation-repair-ledger.json#whole
  - evidence:feature_isolation_warnings verification/ci-verify-r1/feature-isolation-warning-reconciliation.json#whole
  - evidence:feature_isolation_second_capture verification/ci-verify-r1/feature-isolation-second-capture.json#whole
  - evidence:feature_isolation_fourth_capture verification/ci-verify-r1/feature-isolation-fourth-capture.json#whole
  - evidence:feature_isolation_green verification/ci-verify-r1/feature-isolation-green-index.json#whole
  - evidence:query_case_style verification/ci-verify-r1/core-query-clippy-style-index.json#whole
  - evidence:query_actual_admission verification/ci-verify-r1/core-query-admission-index.json#whole
  - verification_obligation:query_publication_guards verification/ci-verify-r1/invoke-core-query-publication-guards.ps1#whole
  - verification_obligation:feature_isolation verification/ci-verify-r1/invoke-feature-isolation-green.ps1#whole
  - evidence:html_feature_dispatch src/tests/gitdiff/feature_isolation.rs#whole
  - evidence:native_feature_ownership src/tests/ir/native_feature_evidence.rs#whole
  - evidence:native_optional_call src/tests/ir/angular_inject_evidence.rs#whole
  - evidence:ef_feature_preservation src/tests/dotnet_meta/efcore.rs#whole
  - evidence:core_query_controls src/tests/compression/core_ir_registration.rs#whole
  - evidence:core_query_ts_definition src/tests/compression/fixtures/core_ir_typescript_before.scm#whole
  - evidence:core_query_cs_definition src/tests/compression/fixtures/core_ir_csharp_before.scm#whole
  - evidence:core_query_red verification/ci-verify-r1/core-query-red-index.json#whole
  - evidence:core_query_witness src/tests/ir/ci_runtime_core_query.rs#whole
  - evidence:core_query_handoff verification/ci-verify-r1/invoke-core-query-witness.ps1#whole
  - verification_obligation:core_query_green verification/ci-verify-r1/invoke-core-query-green.ps1#whole
  - evidence:runtime_history verification/ci-verify-r1/runtime-history-index.json#whole
  - evidence:runtime_root_trial verification/ci-verify-r1/runtime-root-trial-index.json#whole
  - evidence:runtime_dependency_trial verification/ci-verify-r1/runtime-dependency-trial-index.json#whole
  - evidence:runtime_baseline verification/ci-verify-r1/runtime-baseline-index.json#whole
  - evidence:runtime_deduplication verification/ci-verify-r1/runtime-deduplication.json#whole
  - evidence:runtime_triage verification/ci-verify-r1/invoke-runtime-triage.ps1#whole
  - evidence:a4_fixture src/tests/mcp/dependency_footprint_af_perf_002.rs#whole

```

```architecture-graph
node: VERIFY-002
kind: EXPERIMENT
status: FROZEN
title: Baseline CI and authority acquisition evidence before verification redesign
track: REPOSITORY
artifact: verification/ci-verify-r1/baseline-evidence.zip
note: Frozen original failure and scoped operator successes, not candidate CI success or performance certification.
knowledge_scope: evidence
knowledge_content:
  - statement self#CI-VERIFY-R1-baseline
  - qualification self#CI-VERIFY-R1-limits
  - evidence verification/ci-verify-r1/evidence-index.json#whole
```
