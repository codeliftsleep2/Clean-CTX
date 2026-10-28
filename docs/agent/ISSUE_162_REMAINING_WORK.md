# Issue #162: current position and remaining work

Agent: Agent MaxHeadRoom
Reconciled: 2026-10-27
Status: planning/evidence reconciliation; no production change or runtime run.

## Generation-70 integrated binary acceptance inspected PASS

Inspected config-composition70-binary-20261028-114237: config composition PASS
296 checks, inline-bootstrap preservation PASS 205, native config-provider
preservation PASS 304. Each has zero failed checks/errors, three FINISHED sessions
and fewer than 500 checks. All share binary SHA-256
4db45b11baffbbfae3d0f1cc2af38b7a551bf60bdfc7176b3a160d8bf07db773 and the same
recorded input hash. Syntax preflight passed 37 JavaScript files; the final binary
target compilation Finished in 5.11s without warnings/errors. Full report paths,
counts and hashes: target/issue-162/config-composition70-binary-receipt.json.

This supersedes the failed lifecycle-notification and zero-check freshness runs
for these selected suites; those historical receipts remain preserved. Pair-level
input association, standalone cold acquisition, bootstrap result references,
repeated-support retirement, recreate-and-refresh, stale restore and later API/
package/config repair stages now have bounded binary/stdio acceptance. No production
or harness edit accompanies inspection. Earlier 220 scoped test passes and one
additional recreation pass remain separately recorded; no full repository or
Claude field result is inferred. AQ14/P3/P1/W04/#162 remain open.

## Binary freshness preflight stopped before runtime; handoff corrected

The 2026-10-28T15-39-45-099Z-config-composition-summary-27616 receipt records zero
checks and zero sessions. Error: binary predates Rust/build inputs; rebuild.
Cargo's build log Finished in 1.40s and reused the binary. Static comparison finds
only angular_config_composition_exit_issue_162.rs newer than the executable; the
acceptance guard conservatively includes test sources. This is neither behavioral
FAIL evidence nor a passed binary suite.

The checkpoint now invokes cargo rustc for the all-feature clean-ctx binary with
a unique metadata marker, forcing final-target recompilation while retaining
cached dependencies. The guard and source/binary hashes remain unchanged; no file
mtime is forged, no cargo clean, no production edit and no Rust test repetition.
Corrected binary checkpoint is UNRUN. Existing failed/preflight receipts remain
preserved; no later suite or field result is inferred.

## Recreated-result control inspected GREEN

Inspected config-composition70-recreation-20261028-112745: one targeted test passed,
zero failed/ignored, 15.76s. Recreation with the existing refresh notification
restores both result/input pairs without explicit recapture. Format log empty;
Clippy Finished in 24.38s without warnings/errors for all targets/features and
-D warnings. Hash receipt: target/issue-162/config-composition70-recreation-receipt.json.

The correction changes test/harness lifecycle notification only; production remains
unchanged. Prior 220 scoped passes remain separately recorded, with this one
additional lifecycle execution. The corrected integrated binary checkpoint may
now run; the earlier failed binary receipt is preserved and not superseded yet.
No broad Rust preservation rerun or field switching is requested.

## Binary recreation notification correction, UNRUN

Inspected 2026-10-28T14-59-22-132Z-config-composition-summary-8564: 182 successful
checks followed by one failure (183 total). Cold standalone/warm and earlier
restart/support/input/caller deletion checks pass. The failure follows external
recreation of deleted merge-result-1.ts without refresh: the first composition
answer lacks that result's two input pairs, although the later bootstrap query
reacquires its own result reference. Later stale-restore/API/package/config stages
and preservation suites were not reached.

Generic discovery's documented external-write contract requires refresh for newly
created sources; the suite notified deletion but not recreation. It now refreshes
after recreation before querying. No explicit recapture is added, no expected rows
are removed, and no production/semantic change occurred. The tracked independent-
retirement control now recreates the result with the same refresh notification and
requires both input pairs again. Pre-correction harness/test bytes are preserved
under target/issue-162/config-composition70-recreation-control-correction/.
Only that targeted test and new-test lint need execution before the later binary
checkpoint; no broad preservation rerun is required.

## Generation-70 integrated binary batch authored, UNRUN

Agent MaxHeadRoom added config-composition using the existing binary/stdio runner,
source/input/binary hashes, three sessions and hard 500-check ceiling. Four API
routes, same-named result/leaf declarations, repeated-input pair deduplication and
independent bootstrap references are compared as exact public forward/reverse
source/relation/target multisets. The first graph has no bootstrap caller, proving
standalone query-only acquisition at the real production boundary.

Restart batches one/final repeated-input support removal, input type mutation and
repair with unchanged results, independent caller/result deletion and recreation,
stale restore refusal before query, API/barrel/package/config mutation/repair.
Public checks refuse provider flattening. The planned composition suite is below
400 checks; the existing runner enforces the exact hard ceiling.

Operator handoff run-config-composition70-checkpoint.ps1 runs syntax preflight,
one all-feature binary rebuild, then config-composition plus inline-provider and
ApplicationConfig-provider preservation suites, pinning binary identity between
runs. No focused/preservation Rust checks are repeated. All new Node/build/binary
commands are UNRUN; no agent runtime execution or production edit accompanied
harness authorship. Final repository and Claude field gates remain separate/open.

## Generation-70 supplemental/preservation/lint inspected GREEN

Inspected config-composition70-preserve-20261028-095700: corrected supplemental
refusal test 1, native callable scope 4/type query 5, provider 85/view 6/string 12/
module 14/semantic helper 6, route 11, original bootstrap references 12, inline
bootstrap family 13 and durable semantic restore 5. Total 174 passing executions,
zero failed/ignored. The earlier three supplemental passes remain valid; historical
subject-oracle failure is preserved. Combined supplemental/preservation: 177 passes;
with focused 43, 220 requested passing executions, not unique tests.

Format/fmt logs are empty; all-targets/all-features Clippy Finished in 28.07s with
no warnings/errors for -D warnings. Summary/count/hash receipt:
target/issue-162/config-composition70-preservation-receipt.json.
No production edit accompanies evidence inspection. The deterministic checkpoint
is GREEN; the next gate is one integrated rebuilt-binary checkpoint. Binary,
full repository/final field and broader AQ14/P3/P1/W04/#162 closure remain open.

## Supplemental subject-oracle correction, UNRUN

Inspected config-composition70-preserve-20261028-094926: three supplemental tests
passed; api_type_and_input_refusals failed at reverse-row comparison in mode 9
(3 passed, 1 failed, 112.57s). The separate native typed declaration named nested
legitimately publishes its own base/feature inputs. The reverse helper selected
all config subjects but expected appConfig-only rows. The appConfig result still
correctly refuses nested as an input and preserves Base.

The helper now explicitly filters its intended appConfig subject; mode 9 separately
checks nested's exact two native input files. No production/generation change.
Pre-correction bytes/log: target/issue-162/config-composition70-subject-control-correction/.
Use -ResumeAfterRefusalFailure to rerun only that corrected supplemental test then
the pending preservation/lint filters; the three passing tests are not repeated.
Corrected assertions remain UNRUN. No later preservation/binary result is inferred.

## Generation-70 approval map and supplemental batch authored, UNRUN

The [63-item acceptance map](ISSUE_162_CONFIG_COMPOSITION_ACCEPTANCE.md) separates
43 focused GREEN executions from pending proof/refusal/lifecycle and preservation
obligations. Standalone Clippy after unused import removal is operator-reported
GREEN; no archive is invented. Four new supplemental tracked controls batch
local/type-only/unresolved/ambiguous merge APIs, foreign type/missing initializer,
ambiguous/non-native/nested/aliased inputs with valid sibling preservation;
competing/cross-source proof, repeated queries; API/barrel/package/config stale
restore and unchanged-result repair; independent input/result/caller retirement.
Existing fixture helpers gained only bounded sibling visibility. No production edit.

run-config-composition70-preservation.ps1 runs the four new controls and pending
shared/native/provider/route/bootstrap/durable preservation, then fmt/Clippy for
new tests. It does not repeat the 43 focused checks. All supplemental work remains
UNRUN. Binary/field acceptance remains deferred until this deterministic batch
passes. No agent formatter/Cargo/Node/runtime execution occurred.

## Generation-70 focused tests inspected GREEN; unused import removed

Inspected config-composition70-focused-20261027-230822: composition 13 passed,
existing config providers 10, bootstrap controls 8, template compatibility 1,
NgRx legacy transition 11. Total 43 passing executions, zero failed/ignored.
All nine original REDs and four integrated controls pass; later no-flattening
assertions are now reached. Active-file size check passed 17 active files;
format log is empty. Summary/hash receipt:
target/issue-162/config-composition70-focused-receipt.json.

The compile/test logs retain one unused SemanticRelation import warning in the
new integrated test file. That import is removed without behavioral changes;
43 test passes remain valid. This is test GREEN, not zero-warning GREEN. Only
Clippy is needed to verify this deletion; do not rerun the focused batch solely
for the warning. Broader preservation, 63-item exit reconciliation and integrated
binary acceptance remain pending. No agent runtime execution occurred.

## Named config composition approved; generation 70 authored, UNRUN

Owner approval authorizes the bounded native merge-result role, written input
pairs and exact bootstrap result reference. Nine unchanged REDs were isolated
and restored exactly. Existing retained call/type/argument carriers are reused;
no provider flattening. Four integrated controls and compatibility updates are
authored. Next operator handoff: run-config-composition70-focused.ps1.
Compilation/runtime/preservation/lint/binary acceptance remain unverified.
See [approved checkpoint](ISSUE_162_CONFIG_COMPOSITION_DECISION.md).

## Config-composition behavioral REDs inspected

Nine tests reach their intended composition/bootstrap-reference boundaries after
native prerequisites (0 passed, 9 failed, 113.78s). Bootstrap-demanded acquisition
retains the call initializer; the observed loss is interpretation/target admission.
Exact bytes are archived. The [updated production request](ISSUE_162_CONFIG_COMPOSITION_DECISION.md)
recommends pair-level public input facts with private argument occurrence retention;
production and multiplicity disposition remain pending. No repair has been made.

## Named config-composition investigation tests authored, UNRUN

Owner disposition approves investigation/test authoring only. Nine tracked tests
separate native merge-input association from merged-result bootstrap references,
with exact native prerequisites and a repeated-input multiplicity probe. No
production role, relation, capture or compatibility change occurred.
See [investigation checkpoint](ISSUE_162_CONFIG_COMPOSITION_DECISION.md).
Next operator handoff: run-config-composition-red.ps1. No binary/field run requested.

## Next W06/AQ14 question: named config composition

The inline-provider generation-69 slice now has bounded deterministic/lint and
binary acceptance (205 inline, 201 bootstrap, 304 config checks). It does not
evaluate config merging or establish effective application providers.
The next static trace identifies typed call-initializer acquisition and object-only
config admission boundaries. See [named config composition proposal](ISSUE_162_CONFIG_COMPOSITION_DECISION.md).
It requests investigation/test-authoring disposition for written native config
inputs and exact bootstrap result references; production remains unapproved.
No new tests, production edits or runtime commands accompany this trace.

## Generation-69 integrated binary acceptance inspected PASS

Inspected bootstrap-inline69-binary-20261027-184145: inline bootstrap providers
PASS 205 checks, bootstrap-reference preservation PASS 201, ApplicationConfig
provider preservation PASS 304. Each has zero failed checks/errors, three FINISHED
sessions and fewer than 500 checks. All share rebuilt binary SHA-256
94a12cca11c9399ea7c7ab2af43405f22764ee4aeca4663c99c8500957bcdbeb and the same
recorded input hash. The all-feature build Finished without warnings/errors;
syntax preflight passed 35 JavaScript files. Report paths/count/hash receipt:
target/issue-162/bootstrap-inline69-binary-receipt.json.

This supersedes the inline binary UNRUN checkpoints below for these three suites.
The bounded direct-inline provider question has scoped deterministic/lint and
production binary/stdio acceptance. No production/harness edit accompanies this
inspection. Full repository and final Claude field gates remain open. No named-
config effectiveness, runtime application/injector semantics, merge/legacy
bootstrap support, P1/P3/AQ14/W04 or Issue #162 closure follows.

## Generation-69 integrated binary batch authored, UNRUN

Agent MaxHeadRoom added bootstrap-inline-providers to the existing binary/stdio
runner and explicit suite manifest. Four API routes, two same-named token pairs,
repeated calls and independent named-config calls are checked as exact public
forward/reverse source/relation/target multisets. Three sessions batch cold/warm
query-only acquisition, save/restart, token/contributor mutation and repair,
independent call/field/source retirement and rediscovery, stale contributor restore
refusal before query, and API/barrel/package/config mutation/repair.

The existing runner retains binary/input hashes, RPC/stderr receipts and the hard
500-check ceiling. Planned inline suite is below 300 checks. The handoff rebuilds
once, pins its hash, then runs inline membership, bootstrap-reference preservation
and ApplicationConfig-provider preservation. No focused/preservation Rust tests
are repeated. Node syntax preflight, build and all suites are operator-owned and
UNRUN. No production edit accompanies harness authorship. Final repository/field
and broader issue closure remain open.
Operator: verification/live-acceptance/issue-162/run-bootstrap-inline69-checkpoint.ps1.

## Generation-69 supplemental exit controls and lint inspected GREEN

Inspected bootstrap-inline69-exit-20261027-183652: three tests passed, zero failed/
ignored, 20.76s. Explicit inline foreign/ambiguous/malformed refusals, isolated
object/field/entry proof corruption with independent root preservation and edited-
caller stale restore/repair now pass. Format/fmt logs are empty; Clippy Finished
in 31.23s without warnings/errors for all targets/features and -D warnings.
Hash receipt: target/issue-162/bootstrap-inline69-exit-receipt.json.

This supersedes supplemental UNRUN entries below. Thirty focused, 161 preservation
and three supplemental passing executions are recorded separately (194 requested
executions, not unique tests). No production edit accompanies this inspection.
The bounded deterministic approval gaps are covered; generation-69 rebuilt-binary
inline membership acceptance remains pending. Full repository and final field
acceptance, P1/P3/AQ14/W04/#162 closure are not inferred.

## Remaining deterministic exit cases authored, UNRUN

Three supplemental tracked tests in angular_bootstrap_inline_exit_issue_162.rs
batch explicit foreign/ambiguous API and malformed-entry refusals with an
independent native sibling; nine isolated missing/duplicate/crossed object,
providers-array and imported entry proof mutations while root admission survives;
and edited-caller stale restore refusal before queries, neighbor preservation
and query-driven repair. Existing tracked fixture helpers now have bounded sibling
visibility; the frozen original RED/refusal file is untouched. No production edit.

Handoff: run-bootstrap-inline69-exit.ps1 runs only these three new tests, then
fmt/Clippy for the new code. Earlier 30 focused and 161 preservation executions
are not repeated. The existing standalone Clippy receipt predates these new tests;
new-test lint must pass. All supplemental controls UNRUN. Binary acceptance stays
queued for the next integrated checkpoint after these approval gaps are verified.

## Generation-69 tests-only preservation inspected GREEN

Inspected bootstrap-inline69-preserve-20261027-173748: provider 85, view-provider 6,
string 12, module 14, provider_semantics 6, config provider 10, route provider 11,
original bootstrap references 12, durable semantic restore 5. Total 161 passing
executions, zero failed/ignored; no compiler warnings/errors in these logs.
Summary/count/hash receipt: target/issue-162/bootstrap-inline69-preservation-receipt.json.

The tests-only batch did not repeat formatting or Clippy. The prior standalone
Clippy GREEN remains operator-reported, and the earlier 30 focused passes remain
recorded separately (191 requested passing executions combined, not unique tests).
No production change accompanies inspection. Complete the remaining explicit
inline refusal/proof/caller-stale controls from the acceptance map before the
major binary checkpoint. Binary, full repository and final field gates remain open.

## Focused GREEN retained; approval exit crosswalk authored

The [60-item acceptance map](ISSUE_162_BOOTSTRAP_INLINE_ACCEPTANCE.md) now separates
focused GREEN from remaining explicit inline refusal/proof/caller-staleness cases,
broader existing-owner preservation and binary acceptance. Clippy cleanup is
operator-reported GREEN; its output was not archived by the one-line handoff.
No focused test repetition is required solely for the removed unused wrapper.
The existing preservation script now supports -TestsOnly to run only the pending
preservation filters, without formatter/fmt/Clippy. This is separate preservation
of the shared terminal-policy changes, not a rerun of the focused 30 tests.
No runtime command or production edit accompanies this review.

## Generation-69 focused tests GREEN; warning cleanup authored

Inspected bootstrap-inline69-focused-20261027-170015: integrated inline controls
5 passed (76.31s), existing bootstrap controls 8 passed (168.36s), template
compatibility 1 passed (0.07s), NgRx transition 11 passed (60.48s), zero failures/
ignored. Together with the preceding unchanged original five passes, the requested
focused batch has 30 passing executions. The historical invalid-oracle failures
remain preserved. These logs contain a dead_code warning for canonical_token;
this checkpoint is test GREEN, not zero-warning GREEN.

The obsolete declaration wrapper canonical_token had no callers after the source/
hash refactor; it is now removed, retaining the active shared canonical_token_at
implementation. No semantic behavior changed. Warning removal is UNRUN. The
next operator batch run-bootstrap-inline69-preservation.ps1 covers affected
provider/view/string/module/config/route/native-bootstrap/durable preservation,
then fmt and all-targets/all-features Clippy with warnings denied. No repeated
focused batch or binary run is requested at this point. Binary/approval exit
reconciliation and broader repository/field gates remain open.

## Integrated test-oracle correction authored, UNRUN

Inspected bootstrap-inline69-focused-20261027-165339: the integrated batch records
3 passed, 2 failed, 73.18s. Composition/strategy assertions passed before the public
helper omitted the independently valid string-token row from its expectation.
The proof-refusal control stopped at a raw count expecting two instead of the
fixture's three retained entries (Left, Right, repeated Left); its subsequent
corruption assertions were not reached.

The helper now explicitly compares declaration-token rows and the composition
control independently requires the exact named string registration in both public
directions. The raw control requires three retained occurrences and two unique
canonical token identities. No production or generation change. Pre-correction
test/log bytes: target/issue-162/bootstrap-inline69-control-correction/.
Use -ResumeAfterInlineControlsFailure to rerun the integrated batch and remaining
bootstrap/compatibility filters without repeating the unchanged original controls.
Corrected tests remain UNRUN; later preservation/lint/binary are not inferred.

## Inline bootstrap production approved; generation 69 authored, UNRUN

Owner approval now authorizes direct inline object provider membership on the
existing bootstrap call occurrence. The original five-test file was isolated
and restored byte-identically. Shared provider producer/admission now consumes
the proved call/argument/object/providers path; no new public identity or retained
payload. Five integrated controls and compatibility updates are authored.
See [approved checkpoint](ISSUE_162_BOOTSTRAP_INLINE_PROVIDER_DECISION.md).
Next operator gate: run-bootstrap-inline69-focused.ps1. No runtime GREEN or
binary acceptance is inferred; broader P1/P3/AQ14/W04/#162 remain open.

## Inline bootstrap RED batch inspected

Four native route cases fail at the intended inline-membership boundary after
all required prerequisites; the refusal/named-config control passes (1 passed,
4 failed, 106.72s). Exact bytes/hashes are archived. The
[bounded production request](ISSUE_162_BOOTSTRAP_INLINE_PROVIDER_DECISION.md)
now records the concrete entry/carrier boundary and integrated exit obligations.
Production semantic/carrier/compatibility extension remains pending approval.

## Inline bootstrap investigation controls authored, UNRUN

Owner disposition authorizes investigation/test authoring only. Five tracked
controls now cover four native API routes plus bounded refusal/named-config
preservation. No production owner meaning, carrier or generation change occurred.
See [inline decision and carrier audit](ISSUE_162_BOOTSTRAP_INLINE_PROVIDER_DECISION.md).
Operator handoff: run-bootstrap-inline-provider-red.ps1. Observe valid behavioral
REDs after prerequisites before production approval. No binary/field run requested.

## Current next question: inline bootstrap provider membership

Generation-68 named bootstrap root/config references and query-bounded refresh
rediscovery have bounded deterministic/lint/binary acceptance. They establish
written associations, not active application or effective provider context.
The next proposed P1/P3 question is direct inline bootstrap providers:
[bounded owner decision](ISSUE_162_BOOTSTRAP_INLINE_PROVIDER_DECISION.md).
Static trace identifies the intentional named-reference-only argument boundary.
No new tests or production edits accompany this investigation. Inline call-owner
membership requires explicit approval; merge/legacy bootstrap/injector semantics
remain unresolved. Earlier "next bootstrap" statements below are historical.

## Generation-68 integrated binary acceptance inspected PASS

Inspected bootstrap68-binary-20261027-152030: bootstrap-references PASS 201 checks,
ApplicationConfig preservation PASS 304, imported-owner preservation PASS 332.
Each receipt has zero failed checks/errors, three FINISHED sessions and fewer
than 500 checks. All share the same rebuilt binary and recorded input hash.
The all-feature build Finished without warnings/errors (22.05s); syntax preflight
passed 33 JavaScript files. Full report paths, hashes and binary identity:
target/issue-162/bootstrap68-binary-receipt.json.

This supersedes the previous bootstrap binary failure for these selected suites.
The repaired query-bounded discovery path now passes the refresh/caller-deletion
stage and the later stale-restore/package/config stages. No production or harness
edit accompanies evidence inspection. This is bounded binary/stdio acceptance,
not a full repository gate or Claude field result. AQ14/P3/P1/W04/#162 remain open.

## Query-bounded discovery preservation/lint inspected GREEN

Inspected bootstrap68-exit-20261027-150605: filesystem discovery cache 9 passed
(8.80s), config provider 10 passed (89.67s), route provider 11 passed (103.76s),
zero failed/ignored. The corrected native-package fixture now passes its original
inbound-result and scan-count assertions. Format/fmt logs are empty; Clippy
Finished in 26.02s without warnings/errors for all targets/features, -D warnings.
Operator reports GREEN. Prior eight bootstrap controls and completed refresh/
hydration-cache passes remain valid; the historical invalid-fixture failure is
preserved. Summary/hash receipt:
target/issue-162/bootstrap68-discovery-preservation-receipt.json.

No production edit accompanies this inspection. The post-repair deterministic
checkpoint is GREEN. The existing integrated binary checkpoint may now resume
with one rebuild, bootstrap-references and its config/imported-owner preservation
suites. The previous binary failure is not superseded until new receipts pass.
Final repository/Claude field and broader issue gates remain open.

## Discovery preservation fixture correction, UNRUN

Inspected bootstrap68-exit-20261027-145541: eight integrated bootstrap controls
passed (162.43s). The filesystem discovery-cache filter records eight passes and
one failure: external_refresh_expires_zero_result_inbound_discovery expected a
native Angular constructor edge from a fixture without @angular/core package
proof. Generic inbound discovery did run again; no bootstrap query was involved.
The test now installs the existing canonical Angular fixture before state/source
capture. The original count and two-scan assertions are unchanged. Pre-correction
test/log bytes are under target/issue-162/bootstrap68-filesystem-control-correction/.
No production change accompanies this fixture correction; corrected test UNRUN.

The handoff supports -ResumeAfterFilesystemFailure to rerun the corrected filter,
then config/route preservation and final lint without repeating completed bootstrap,
refresh and hydration-cache batches. Historical failed receipt remains preserved;
no later preservation, lint or binary acceptance is inferred.

## Approved query-bounded discovery implementation authored, UNRUN

The supplied owner approval authorizes conservative TypeScript nomination for the
exact Angular ApplicationBootstrap/bootstrapApplication forward query. The valid
regression and its registration were isolated as raw bytes under
target/issue-162/bootstrap68-refresh-approved-isolation/ before production edits,
then restored byte-identically. HEAD and the normal index were untouched.

A private bootstrap hydration function reuses configured_roots and source_inventory,
selects TypeScript owners without a spelling predicate, and feeds existing High
capture/publication. Only the exact forward identity invokes it. Root exclusions
and source validation remain in existing inventory/publication code; withinPath
is not passed to acquisition. Generic literal discovery, native proof and retained
admission remain unchanged. No new cache, scanner, resolver or lifecycle exists.
Inventory completion does not supply an acquisition satisfaction/completeness proof.
No retained meaning or durable encoding changed; semantic generation remains 68.

One additional tracked control batches cold and post-refresh direct/alias/barrel/
namespace routes, independent caller deletion, generic-query non-expansion and
lower-bound coverage. The original refresh RED is unchanged. Eight integrated
controls plus refresh/discovery-cache/config/route preservation, fmt and Clippy
are now selected by run-bootstrap68-exit-controls.ps1. All post-repair commands
are UNRUN; no agent Cargo, formatter, Node or runtime execution occurred. Binary
acceptance is deferred until this deterministic checkpoint passes.

## Refresh discovery RED and bounded decision

The corrected regression now fails after native prerequisites, matching the
binary omission. Evidence is archived under bootstrap68-refresh-discovery-red-evidence.
See [bounded discovery decision](ISSUE_162_BOOTSTRAP_DISCOVERY_DECISION.md).
No production repair or binary rerun has occurred. The shared literal discovery
predicate cannot establish an alias caller after refresh retracts retained owners.
A bounded candidate policy requires disposition before repair.

## Refresh regression fixture corrected, UNRUN

Inspected bootstrap68-refresh-red-20261027-142557/test.log: one passed in 9.89s.
This did not reproduce the binary failure. The first tracked fixture still wrote
bootstrapApplication as launch in its caller import; the binary caller writes
start as launch. Literal discovery can select the former. The earlier causal
classification remains a hypothesis until the corrected regression is observed.

The corrected fixture exports bootstrapApplication as start from its barrel and
imports start as launch in the unchanged sibling, matching the binary route with
no native API spelling in that caller. Native call/admission prerequisites and
the first-forward-query assertion are unchanged. Pre-correction test bytes are
preserved under target/issue-162/bootstrap68-refresh-control-correction/.
No production repair was made. Corrected regression UNRUN.

## Generation-68 binary refresh discovery failure inspected

The first bootstrap binary receipt records 126 successful checks followed by
q113/exact-public-multiset failure (127 total checks). Cold/warm checkpoint and
restart, root/config/barrel mutation/repair and argument/call replacement checks
passed before failure. After deleting boot-main-0.ts and refresh_workspace, the
first forward query omits the unchanged barrel caller boot-main-2.ts; callers 1
and 3 remain. This is not an all-green binary batch. Later stale-restore/package/
config session and both preservation suites were not reached.

Receipt: target/live-acceptance/issue-162/2026-10-27T18-20-41-970Z-bootstrap-references-summary-11396/.
Binary SHA-256: be0b8ec13efd999077d84b201c2208c2caa785dabf92d97a48667a5883621e39.
The located boundary is refresh clearing indexed/retained owners followed by
name-based forward discovery: the caller imports start as launch from its barrel
and contains no bootstrapApplication spelling. Existing retained-owner replay
cannot recover an owner removed by refresh. Native interpretation had admitted
this exact caller earlier. No production repair has been made.

A seventh tracked integrated control now requires valid native call/admission
prerequisites, deletes one caller, refreshes, and checks the first public forward
answer for its unchanged barrel sibling without explicit recapture or a preceding
reverse query. This regression is UNRUN; isolate a valid behavioral RED before
repair. Existing six-control GREEN and previous lint are historical evidence.
No binary rerun is requested until this divergence is handled deterministically.

## Generation-68 integrated binary batch authored, UNRUN

Agent MaxHeadRoom added bootstrap-references to the existing binary/stdio runner
and explicit suite manifest. Four native API routes, distinct same-named target
pairs and repeated calls use exact public forward/reverse source/relation/target
multisets. Three sessions batch cold/warm queries, restart, root/config/barrel
mutation and repair with unchanged callers, independent argument/call/source
retirement, stale restore refusal before queries, package/config mutation and
repair. A foreign API caller must emit no bootstrap associations.

The handoff rebuilds once, pins the binary hash between suites and runs bootstrap-
references, application-config-providers and provider-imported-owners. Each uses
the existing hard 500-check ceiling, source/input/binary hashes and complete
session logs. The bootstrap scenario has fewer than 300 planned checks. Node
syntax preflight, build and all three live suites are operator-owned and UNRUN.
No production or Rust test change accompanies this harness authorship. Scoped
Rust/lint evidence remains valid; final repository/Claude field gates stay open.
Operator: verification/live-acceptance/issue-162/run-bootstrap68-checkpoint.ps1.

## Generation-68 integrated exit controls inspected GREEN

Inspected bootstrap68-exit-20261027-140450: all six integrated controls passed,
zero failed/ignored, 95.53s. This includes the strengthened exact public forward/
reverse source/relation/target multisets and independent root/config replacement.
Formatting and fmt-check logs are empty; Clippy Finished in 22.60s with no warnings/
errors for all targets/features and -D warnings. Operator reports GREEN.
Hash/count receipt: target/issue-162/bootstrap68-exit-receipt.json.

This supersedes the new public-control UNRUN statements below. No production
change accompanies inspection. The deterministic exit strengthening is GREEN;
rebuilt-binary acceptance remains open, along with final repository/field and
broader AQ14/P3/P1/W04/#162 gates. No binary run is inferred.

## Bootstrap exit review and strengthened public control authored, UNRUN

The [47-item exit review](ISSUE_162_BOOTSTRAP_REFERENCE_EXIT_REVIEW.md) maps the
approval to concrete controls and receipts, with binary acceptance explicitly open.
Audit found exact bidirectional target-file replacement was weaker than the source/
relation count controls. A sixth integrated test now compares exact public
source/relation/target multisets in both directions, repeated queries, and
independent same-named root/config replacement for one repeated call while its
sibling and neighboring caller survive. This is a test-only change, UNRUN.
Operator handoff: verification/live-acceptance/issue-162/run-bootstrap68-exit-controls.ps1.
The existing 242 preservation passes are retained; this handoff runs the six
integrated controls and lint, not another broad batch. No agent-run runtime.

## Generation-68 scoped preservation and lint GREEN

Inspected the broader preservation logs under bootstrap68-preserve-20261027-130656:
242 passing executions, zero failed/ignored across the 22 selected filters.
The historical Clippy failure is preserved. Following the mechanical type-alias
and test-helper correction, the operator reports the targeted twelve bootstrap
controls GREEN; their console output was not archived by the supplied command.
Do not classify that rerun as an inspected test-log receipt.

Inspected bootstrap68-preserve-20261027-135544: fmt log empty; Clippy Finished in
40.27s with no warnings/errors for all targets/features and -D warnings. The
operator reports the conditional follow-up GREEN. Summary/hash receipt:
target/issue-162/bootstrap68-preservation-receipt.json. No production edit
accompanies this evidence update. The scoped preservation/lint gate is satisfied;
the 47-item exit review, rebuilt-binary acceptance and final repository/field
gates remain open. This is not full-repository or binary acceptance.

## Generation-68 preservation reached lint; mechanical correction UNRUN

The operator batch reached Clippy after the selected preservation filters and
fmt check. Clippy rejected the invocation capture return tuple complexity and
the original bootstrap prerequisite helper's nine arguments. A private return
type alias and a tuple of the same three test expectations address these style
violations without changing assertions or semantic behavior. Original test bytes
are preserved under target/issue-162/bootstrap68-clippy-correction/; the historical
unchanged-RED hash remains evidence of the pre-correction test, not the current
mechanically refactored file. Formatting, the twelve affected bootstrap controls
and Clippy must be rerun. No agent-run Cargo, formatter or runtime occurred.

## Generation-68 broader preservation handoff authored, UNRUN

Agent MaxHeadRoom prepared run-bootstrap68-preservation.ps1 for the operator.
It batches nine existing native lexical/callable/type-occurrence filters, Angular
provider/view/string/module/config/route and injection controls, durable semantic
restore, restore compatibility and delta persistence, then fmt and all-targets/
all-features Clippy with warnings denied. The 30 focused passes are not repeated.
Timestamped logs use Windows-safe filenames; an explicit StartAt filter supports
resuming without rewriting earlier evidence. No production edit accompanies this
handoff. These commands remain UNRUN; binary acceptance and the 47-item exit
review remain pending. This scoped batch does not replace the final repository
or Claude field gates. Node remains deferred to the major integrated checkpoint.

## Combined C2/C3 binary acceptance inspected PASS

Inspected the operator receipts: provider-imported-owners PASS, 332 checks;
provider-composition preservation PASS, 190 checks. Each records zero failed
checks/errors, three FINISHED sessions and the 500-check maximum. Both use binary
SHA-256 b5a820fbe59fce1b4b695da0e807f9aaecf5216659cdd3070d780b7c9151236b,
mtime 2026-10-26T19:36:00.297Z, and the same recorded input hash. Summary and
receipt hashes are retained at target/issue-162/composition65-binary-receipt.json.
The full logs are under target/issue-162/composition65-binary-20261026-153511/;
syntax preflight passed 27 JavaScript files and the all-feature binary build
finished without warnings/errors.

This supersedes the binary UNRUN/deferred checkpoints below for these two suites.
It verifies the bounded imported Component and Directive/Module composition
production boundary and affected same-source preservation. No other live suite,
full repository gate or final Claude field result is inferred. No production or
harness change accompanies inspection. Broader P1/W04/#162 closure remains open.

## Where we are

**We have a working, repeatedly verified foundation and several migrated Angular
consumers. We are not near the final gate for the full #162 scope.** The recent
provider work is one part of a substantially larger Angular-wide migration.

The governing scope is the [implementation plan](ISSUE_162_IMPLEMENTATION_PLAN.md),
especially sections 5, 6 and 6A. It contains **26 acceptance questions (AQ01–AQ26)**.
This checklist organizes them into **12 implementation work packages and 3 final
acceptance gates**. These are planning buckets, not 15 remaining fixes, commits,
test runs or equal units of effort. The number of implementation defects and
architectural decisions inside unreviewed packages is not yet known.

Five implementation packages have bounded verified progress; seven still need
their broad contracts reconciled against production. None of the 12 is certified
closed as a whole by this reconciliation. That does not mean nothing is
implemented: it means partial GREEN receipts cannot close larger questions.
There is no defensible percentage-complete or time estimate yet.

**Immediate state:** generation 67 is the latest bounded verified production
checkpoint. Native declared ApplicationConfig membership has 460 scoped Rust
passing executions, clean fmt/Clippy and inspected binary PASS: config 304,
route preservation 305 and imported-owner preservation 332 checks. Full repository
and final field gates remain open; P1/W04/#162 are not closed.

The next proposed question is [declared bootstrap root/config references](ISSUE_162_BOOTSTRAP_REFERENCE_TRACE.md).
The [bounded decision](ISSUE_162_BOOTSTRAP_REFERENCE_DECISION.md) requires explicit
owner approval for new public relation/occurrence meaning and retained call proof.
Investigation/test authoring approval now has inspected evidence: the native
prerequisite/API control passes and eleven intended root/config association REDs
fail after all prerequisites. Exact test/log bytes are archived; the updated
decision requests bounded production/public vocabulary/retained-carrier approval.
The subsequent owner approval authorizes the two structural relations and bounded
invocation proof. Generation-68 implementation now has inspected focused GREEN: 30 passing
executions across scope-policy, twelve original controls, five integrated
proof/lifecycle controls and compatibility filters. The scoped branch/function
acquisition correction preserves ordinary lookup policy and both frozen test
files remain byte-identical. Broader preservation/lint/binary and final field gates
remain open; no AQ14/P3/P1/W04 closure follows.

## What the existing GREENs establish

| Verified slice | What it establishes | What it does not establish |
| --- | --- | --- |
| Native retained identity and transport | Exact declaration/reference evidence is consumed by migrated slices through the existing source-owned artifact/session/durable/index path. | Universal TypeScript/project conformance or every Angular consumer. |
| Module and standalone composition slices | Canonical membership/composition cases have recorded focused and binary evidence. | Complete owner-visible declaration sets, all metadata forms or reverse closure. |
| Constructor, field and functional DI | Bounded API/token proof, declaration/string distinctions, same-name discrimination, mutation and restart. | Which scoped provider satisfies a consumer, all DI syntax or complete reverse consumers. |
| Direct member initializers | Exact owner/member identity for the migrated input/output/model initializer forms, with forged-pair and lifecycle controls. | Public aliases, inheritance, generated bindings or template-use joins as a complete contract. |
| Package/public-entry authority | Owning package and consumed public entry are distinct; HTTP role and selected mutation/refusal/lifecycle cases pass. | General package/API closure or every export-map/resolver frontier. |
| Provider registration, class/factory/alias relations | Exact migrated token/pair authority, shared mechanics, adversarial and durable/currentness controls for the approved forms. | Canonical runtime `useValue` identity, all callable forms, all provider owners, multi/precedence or injector closure. |

Historical generation-59 inspected binary receipt:
`target/live-acceptance/issue-162/2026-10-25T19-21-16-341Z-16680/report.json`:
**1660/1660 checks, zero errors, three FINISHED sessions**. Binary SHA-256:
`aa2a64437fc940fd01f620ada9228018bbdb65745d6d302a10a53e3f127ef685`.
This is bounded operator binary evidence, separate from Rust tests, CI and Claude
field acceptance. Later test-only changes do not extend that harness's scope.

The [consolidated audit](ISSUE_162_CONSOLIDATED_AUDIT_2026_10_24.md) and linked
decision records retain chronological receipts and RED/GREEN evidence. This
document is the navigation/status checklist, not another historical receipt log.

## The remaining implementation packages

Status meanings: **PARTIAL** means some bounded migration evidence exists;
**UNRECONCILED** means broad #162 acceptance has not been established here, not
that the repository has no implementation. **CLOSED** requires the exit criteria
below plus concrete evidence links. No package is CLOSED yet.

| ID | Package / acceptance questions | Current position | Remaining work and concrete exit criterion |
| --- | --- | --- | --- |
| W01 | Foundation, applicability and API roles — AQ01; shared AQ25 prerequisites | PARTIAL | Reconcile remaining native/project/mode/API-role frontiers and actual consumer inventory. Resolve the explicitly excluded `erased_namespace_query` frontier for full acceptance. Exit: required roles and language evidence have current, tested authority; no known gap is hidden under a support label. |
| W02 | NgModule and standalone composition — AQ02–AQ03 | PARTIAL | Reconcile remaining expression/metadata forms, declaring lexical context, explicit/version-default eligibility, empty-owner transitions and reverse membership. Exit: required composition questions are production-queryable with canonical targets, paths and lifecycle proof. Closed visibility belongs to W03. |
| W03 | Template ownership, visibility, selectors, pipes and structural context — AQ04–AQ07, AQ18 | UNRECONCILED | Establish exact inline/external template ownership; closed owner-visible declaration/pipe sets; scoped selector/pipe resolution and reverse uses; exact control ancestry. Exit: public answers explain the actual owner and visibility path, exclude out-of-scope lookalikes and survive template/composition changes. |
| W04 | Providers and DI — AQ08–AQ09 | PARTIAL; useValue reference slice verified | Six open obligation groups below cover owners/composition, strategy forms, injector context, multi/precedence, consumer resolution and public closure. Exit: required scoped provider/consumer questions are answerable with reasons and truthful uncertainty, not merely equal token labels or local strategy edges. |
| W05 | Public members and reactive APIs — AQ10–AQ11 | PARTIAL | Complete required member aliases, directionality/required/model meanings, inherited/exposed members and binding joins; reconcile signal/computed/effect/resource/interoperability API authority and versioned dependency forms. Exit: public member/use and admitted reactive-structure questions have exact ownership and lifecycle evidence. No inferred runtime reactive graph. |
| W06 | Routing and application composition — AQ12–AQ14 | UNRECONCILED | Canonical eager/lazy route targets, selected exports and reverse routes; parent/config guard/resolver/provider context; actual bootstrap and merged application config with distinct app boundaries. Exit: public answers trace the correct route/application/config paths through dependency mutation. Functional DI proof alone does not close routing. |
| W07 | Host directives, view/content queries and defer — AQ15–AQ17 | UNRECONCILED | Host composition/member exposure, query locator/token/read/required/plural distinctions, and defer eligibility/eager-use blockers using real visibility and template context. Exit: each required question has exact API/target/owner proof and public explanations without invented runtime query results or bundle behavior. |
| W08 | NgRx and RxJS — AQ19–AQ20 | UNRECONCILED | Reconcile exact registration/action/member/reducer/effect/selector and dispatch/select neighborhoods; prove stream/operator identity and order at admitted use sites. Exit: required forward/reverse architecture questions are canonical/current and preserve prior issue-family behavior. Existing #152 controls are assets, not blanket #162 certification. |
| W09 | Reactive Forms and Formly — AQ21–AQ22 | UNRECONCILED | Control/group/array/validator composition and template binding; Formly field/config/type registration and expression ownership. Exit: required public composition/use questions join exact config/template/registration authority and classify dynamic facts honestly. |
| W10 | Testing and TestBed — AQ23 | UNRECONCILED | Canonical testing API calls, imports/providers/overrides, tested targets, test configuration and reverse uses; preserve test versus production scope. Exit: registered public queries explain the relevant test configuration and target identities through edits and restore. |
| W11 | Cross-layer impact, public joins and completeness — AQ24–AQ25 | PARTIAL | Complete reverse/affected-neighborhood acquisition, reason paths, late publication/currentness and query-specific completeness across required families. Exit: same-named endpoints remain distinct and each answer's claimed closure/currentness is justified through the production query contract. |
| W12 | Rendering and marker consumers — AQ26; AQ18 presentation | UNRECONCILED | Audit every materially affected renderer for lost owner/endpoint/provenance/uncertainty/ordered context; prove any source-local exception. Exit: compressed/public presentation preserves the upstream fact without requiring source reconstruction or inventing authority. |

These packages cover every AQ ID. W11's cross-cutting role does not remove each
family's own reverse/currentness/completeness obligations. W12 does not excuse
weak upstream proof. Existing implementation must be inspected and reused before
assuming a rewrite is needed.

### Complete question crosswalk

Every row remains open at its full question scope; the verified-slice table
above identifies useful completed parts. The implementation plan retains the
full normative question/fact/closure wording.

| Question | Meaning | Main package |
| --- | --- | --- |
| AQ01 | Actual Angular declaration/API role | W01 |
| AQ02 | Module ownership and composed membership | W02 |
| AQ03 | Standalone composition and eligibility | W02 |
| AQ04 | Actual template ownership, forward and reverse | W03 |
| AQ05 | Closed owner-specific visible declarations | W03 |
| AQ06 | Scoped selector occurrence resolution | W03 |
| AQ07 | Scoped pipe resolution and reverse uses | W03 |
| AQ08 | Applicable provider strategy, registration and scope | W04 |
| AQ09 | Scoped DI consumers and removal consequences | W04 |
| AQ10 | Public members, aliases and bindings | W05 |
| AQ11 | Proved reactive API/dependency structure | W05 |
| AQ12 | Actual eager/lazy route targets and reverse uses | W06 |
| AQ13 | Applicable route guards/resolvers/providers/config | W06 |
| AQ14 | Actual bootstrap/application configuration | W06 |
| AQ15 | Host directive composition and exposure | W07 |
| AQ16 | View/content query target and API meaning | W07 |
| AQ17 | Defer eligibility and eager-use blockers | W07 |
| AQ18 | Exact control-flow occurrence context | W03, with W12 presentation |
| AQ19 | NgRx action/state/registration/use neighborhood | W08 |
| AQ20 | RxJS stream/operator/subscription structure | W08 |
| AQ21 | Forms controls/validators/composition/bindings | W09 |
| AQ22 | Formly registration/config/field-use neighborhood | W09 |
| AQ23 | TestBed/test configuration, overrides and targets | W10 |
| AQ24 | Cross-layer semantic impact and reason paths | W11 |
| AQ25 | Exact/current public joins and completeness | W11, with every family |
| AQ26 | Rendering that preserves proved semantics | W12 |

## Completed bounded checkpoint: useValue reference admission

This is a subtask of W04, **not the remaining definition of W04 or #162**.

- [x] **V1 — Owner decision:** owner APPROVED the bounded
  [reference-admission proposal](ISSUE_162_PROVIDER_VALUE_REFERENCE_DECISION.md).
  It repairs registration only; canonical runtime-value identity is not decided.
  The explicit disposition authorizes V2–V4 routine choices within the contract.
- [x] **V2 — Repair and deterministic evidence:** freeze the unchanged valid RED,
  implement only the approved contract, restore exact bytes, verify focused
  positives/refusals and affected-family preservation in one planned batch.
  Generation-59 provider59-*.log receipts: 73 passed, no failures/ignored or warnings.
- [x] **V3 — Lifecycle/currentness evidence:** verify complete retained proof,
  stale restore refusal, unchanged-consumer dependency/config/package mutation,
  repair and independent retirement; reuse existing fixtures where equivalent.
  Shared durable/currentness variants passed in the inspected generation-59 batch.
- [x] **V4 — Lint and binary acceptance:** finish the selected checks, rebuild
  once for this stable production slice and run the applicable batched driver.
  Corrected generation-59 report: 1660/1660 checks, three FINISHED sessions,
  no errors; fmt/Clippy/syntax GREEN operator-reported. First failed expectation
  remains preserved separately. No Rust repair accompanied its correction.

Original RED: `provider-value-reference-red-rerun.log` — the ordinary baseline
passed, then mixed aliased/namespace value routes returned 0 registrations
instead of 2. The earlier baseline failure was a test-control correction and
is preserved separately. The unchanged valid test and its later assertions now
pass in the generation-59 deterministic batch.

This closes the approved reference-admission slice. It does not authorize a
canonical value evaluator or establish injector closure.

## W04 reconciliation after V4

Six obligation groups remain under the original AQ08/AQ09 questions. These are
requirements to reconcile, not six known bugs or six automatically approved
architectural changes. Existing assets and other work packages may already
answer portions; trace them before implementing replacements.

| ID | Remaining obligation | Current evidence boundary | Exit needed for W04 |
| --- | --- | --- | --- |
| P1 | Provider owners and composed registrations | Direct Component/Directive/Module registration, Component view position and same-source composition have bounded verified checkpoints. Imported Component and Directive/Module composition have scoped Rust/lint GREEN and inspected generation-65 binary PASS (332 imported-owner checks, 190 same-source preservation checks); Native source-owned Routes objects have generation-66 declared membership acceptance and named native ApplicationConfig declarations have generation-67 bounded Rust/lint/binary evidence. Bootstrap linkage, other route roots, application/environment/testing contexts remain unreconciled. | Map required owner/container forms to exact production acquisition and canonical registration answers, including composition/retraction and negative owner controls. Coordinate W02, W06 and W10. |
| P2 | Strategy/form coverage and value meaning | Class shorthand/useClass, function-declaration useFactory and useExisting have bounded canonical pair evidence. Other callable forms retain classified compatibility behavior. useValue reference admission is verified; its strategy presentation remains noncanonical. | Reconcile each required form with its actual question and public fact. Only propose richer target/value identity when a demonstrated question requires it; do not assume a general evaluator. Record genuine unsupported forms and resolve required capabilities rather than silently excluding them. |
| P3 | Injector/application context | Local token/pair identity does not identify the applicable injector or connect its owner to the actual application/configuration hierarchy. | Establish the required owner/application/injector context and visibility proof, with distinct applications/scopes and current dependency paths. This is a prerequisite for effective-provider claims. |
| P4 | Multi, overrides and precedence | Existing deduplication preserves semantic fact identity; it is not an ordered registration/multi/override model. No winning-provider claim follows from existing edges. | Define and verify the required order/multi/override questions against real registration context, with explicit runtime uncertainty. Architectural changes require the actual bounded owner decision. |
| P5 | Scoped consumer satisfaction and reverse impact | Constructor/field/functional consumption facts are bounded GREEN, but they are not joined to an effective scoped registration universe. | Answer which registration applies to a consumer, why, and the effect of removal/override, with complete requested consumer acquisition or correctly classified bounds. Coordinate W11; token equality alone is insufficient. |
| P6 | Public explanation, closure and ecological acceptance | The binary driver exercises selected local identities/lifecycles. It does not certify every provider owner, context or reverse neighborhood. | Expose required scope/strategy/reason paths through registered production queries; verify relevant cross-family order/currentness/negative/durable cases and reconcile all P1–P5 evidence before package closure. Reuse G01–G03 at campaign finalization. |

The next planning task should trace **P1/P3's existing owner/context path** and
produce one bounded question/exit contract for review, rather than automatically
opening another value-identity feature. This is sequencing guidance, not approval
of a new injector model. No implementation for P1–P6 was added during this
reconciliation. W04, AQ08 and AQ09 remain open.

The initial [owner/context trace](ISSUE_162_PROVIDER_OWNER_CONTEXT_TRACE.md) is
now authored. Its concrete next candidate is P1.D1: direct Directive.providers
registration, reusing the existing pipeline. Both current provider-owner gates
rejected Directive despite existing native role proof. The intended public RED
is now observed and the [bounded owner extension](ISSUE_162_PROVIDER_DIRECTIVE_OWNER_DECISION.md)
is approved. Generation-61 provider/durable/currentness and compatibility controls
are operator-reported GREEN; the inspected rebuilt binary report passes all
1844 checks. The bounded P1.D1 direct-registration checkpoint is verified.
P3 injector/application context remains
unresolved; D1 closes only that local owner-admission subquestion.

P1.VP1 [Component viewProviders position](ISSUE_162_VIEW_PROVIDER_POSITION_DECISION.md)
is verified at the bounded generation-62 checkpoint: focused/durable/currentness
and fmt/Clippy operator-reported GREEN; all 16 split binary suites inspected PASS
(2569 checks across receipts, maximum 322 in one suite). RegistersViewProvider
preserves declared position separately from ordinary RegistersProvider. This
does not establish effective visibility or satisfy P3–P6; P1 and W04 remain open.

Next investigation batch: [P1.C1 provider-array composition](ISSUE_162_PROVIDER_COMPOSITION_TRACE.md).
Six tracked controls cover references, spreads and nested arrays in both admitted
Component positions. All six intended REDs are observed and frozen. The
[bounded composition decision](ISSUE_162_PROVIDER_COMPOSITION_DECISION.md) is now
owner-approved with the structural-authority boundary. Generation-63 compilation
and focused composition/preservation controls are operator-reported GREEN;
the additional durable/currentness tracked controls and lint gates are now
operator-reported GREEN. Inspected composition and position binary receipts PASS
with 190 and 160 checks respectively, zero failures and six FINISHED sessions.
P1.C1 bounded local acceptance is satisfied; full repository/final field gates and
remaining owner/composition obligations stay open. No runtime array
evaluation, imported contributor arrays or injector closure is authorized.

Next investigation: [P1.C2 imported provider arrays](ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_TRACE.md).
Six tracked questions cover alias/barrel/namespace contributor routes in both
Component positions, with established direct and same-source prerequisites.
All six intended imported-stage REDs are inspected and byte-preserved (0 passed,
6 failed, 0 ignored). The [bounded source-qualified membership decision](ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_DECISION.md)
is owner-approved. Generation-64 imported focused receipt records 11 passed, zero failures/ignored,
including all six unchanged REDs and five integrated controls. Compilation is
operator-reported GREEN. Seven preservation receipts add 107 passing executions
(zero failed/ignored); fmt/Clippy are GREEN. The combined scoped Rust total is
118 passing executions at that generation. The later generation-65 combined C2/C3
binary checkpoint now supplies bounded acceptance: imported owners 332 PASS and
same-source preservation 190 PASS, with six FINISHED sessions and no errors.

Next mapped batch: [P1.C3 Directive/NgModule provider composition](ISSUE_162_PROVIDER_OWNER_COMPOSITION_TRACE.md).
Twelve tracked controls cover both ordinary-provider owners across nested/local
reference/spread and imported alias/barrel/namespace forms. Each requires direct
owner registration and matching composed Component controls before its candidate
assertion. All twelve intended REDs are inspected and byte-preserved (0 passed,
12 failed, 0 ignored). The [shared owner/position contract](ISSUE_162_PROVIDER_OWNER_COMPOSITION_DECISION.md)
is owner-approved. Generation-65 focused receipt records 17 passed, zero failures/ignored, including
the twelve unchanged REDs and five integrated tests (both owners per test).
Preservation adds 118 passing executions (67 earlier provider passes, one corrected
control rerun and 50 remaining checks); the historical invalid-control failure is
retained and resolved. Scoped generation-65 total: 135 passing executions;
fmt/Clippy GREEN. Combined generation-65 C2/C3 binary acceptance is inspected PASS
(332 imported-owner and 190 same-source preservation checks). These bounded
subquestions have local acceptance; broader repository/field and P1 gates remain.

Next investigation: [P1.R1 route-declared provider membership](ISSUE_162_ROUTE_PROVIDER_TRACE.md).
Two tracked direct/imported public controls produced the intended omission REDs
(0 passed, 2 failed), after established native Component registrations and existing
route projection prerequisites. Exact evidence is archived; the
[bounded route decision](ISSUE_162_ROUTE_PROVIDER_DECISION.md) is owner-approved
with private route identity and a mandatory public-API guardrail. Generation-66
compilation/focused checkpoint is inspected GREEN: 6 passed, zero failed/ignored,
47.04 seconds, including the two unchanged REDs and four integrated controls.
The additional exit controls and scoped preservation/lint/guard batch are now
GREEN: 161 preservation passes plus six initial, four earlier exit passes and
one corrected acquisition-control rerun yield 172 passing executions. Historical
failure evidence is preserved; automatic removed-root rediscovery is not certified.
The approval evidence map is reconciled and generation-66 binary receipts are
inspected PASS: route-providers 305 checks plus imported-owner preservation 332,
zero errors and six FINISHED sessions. Bounded P1.R1 local acceptance is satisfied
with explicit known-root reacquisition after recreation; automatic rediscovery
of a removed route identity remains uncertified. Broader P1/W04 and repository/
field gates remain open.
The earlier route acquisition/object-identity omission is repaired under the
separate route approval. No activation or effective injector model follows.

Next investigation: [P1.A1 native ApplicationConfig declarations](ISSUE_162_APPLICATION_CONFIG_PROVIDER_TRACE.md).
All four direct/local/imported controls reached their intended omission REDs
(0 passed, 4 failed, 19.55 seconds), after Component registration and native
config-type occurrence prerequisites passed. Evidence is byte-preserved and the
[bounded config-owner decision](ISSUE_162_APPLICATION_CONFIG_PROVIDER_DECISION.md)
is now owner-approved. Generation-67 typed-object acquisition/owner membership
and the initial ten-test integrated checkpoint now pass, reusing the shared
provider mechanisms and real declaration identity. Inspected focused receipts
record 33 passing executions: config 10, route 11 and compatibility 12. The
compression-to-index retained-authority repair passes its before-query lifecycle
control. Inspected resumed compression/provider/publication/durable preservation records
426 passing executions; with the focused 33, the scoped total is 459. Historical
fixture/generation/timing failures remain recorded with corrected GREEN controls.
Final fmt/Clippy are GREEN. Approval coverage review, later major-checkpoint binary
acceptance and final repository/field gates remain open.
The [51-item acceptance map](ISSUE_162_APPLICATION_CONFIG_ACCEPTANCE.md) now records
the expanded refusal control GREEN (460 scoped Rust passing executions total) and
one-build binary PASS: config 304, route 305, imported owners 332 checks, each in
three FINISHED sessions below 500 checks. Final repository/encoding/size and field
gates remain open. Actual bootstrap/config selection remains a separate P3/AQ14 integration gap.

## What counts as closing an implementation package

Each package must meet the implementation plan's existing acceptance rule:
**answerable, authoritative, production-queryable, current and correctly
classified for completeness**. In concrete terms:

- [ ] Map required AQ subquestions and admitted forms to actual production
  producers, consumers and public answers; record known exceptions/gaps.
- [ ] Reuse shared authority/lifecycle mechanics and keep specialized semantics
  with their owner. Remove the replaced weaker authority path where applicable.
- [ ] Make required positive questions answerable; refusal/uncertainty must not
  silently remove an in-scope capability or stand in for implementation.
- [ ] Retain and validate exact identities, occurrences and consumed witnesses.
- [ ] Supply applicable tracked nonempty/adversarial/currentness/durable/public
  regression evidence, including valid RED before repair for demonstrated bugs.
- [ ] Verify the default production path and applicable binary behavior; record
  tests, operator binary receipts and later field results as distinct categories.
- [ ] Link the evidence and state exactly which subquestions remain open.

A new architectural decision is requested only when implementation actually
crosses an unapproved contract. Existing approvals and equivalent-mechanism reuse
remain valid; routine implementation choices do not create another approval gate.

## Three final acceptance gates

| ID | Gate | Exit criterion / current status |
| --- | --- | --- |
| G01 | Final source audit and repository gate | All W01–W12 exit criteria reconciled; obsolete competing authority removed; known frontier/exclusions resolved for the claimed scope; operator runs the single [authoritative Final Verification Gate](verification.md#final-verification-gate). OPEN. Incremental fmt/Clippy receipts do not equal this gate. |
| G02 | Consolidated production-boundary acceptance | Reusable binary assets cover all applicable migrated families and cross-family/ecological cases, with rebuilt binary/input hashes, real public answers, persistence and lifecycle. OPEN. Current split binary assets supply bounded family-specific receipts, not whole-migration coverage. |
| G03 | Batched final Claude field acceptance | Local questions resolved first; consolidate remaining real-workspace/client-only questions into the fewest practical pilots, record results and any discoveries, then obtain owner acceptance. OPEN; do not request field switching while local implementation gaps remain. |

Do not duplicate or improvise the full repository command list here; verification.md
is its sole authority. All Cargo/build/lint/binary/field execution remains
operator-owned under AGENTS.md.

## How we keep this from becoming an endless sequence

This is a sequencing/checklist proposal within the existing scope, not a new
repository policy or permission to narrow #162.

1. Select one named package/subquestion and write its concrete slice exit before
   authoring another regression. Show what finishing the slice will close.
2. List the known positive, refusal, lifecycle and public obligations together.
   Author/batch independent controls where the contract permits, instead of
   handing off one new case after every GREEN.
3. Repair causal gaps under existing approvals. When a new decision is required,
   explain its bounded effect and whether it is part of the current package.
4. At a stable GREEN, update this checklist with closed subquestions, remaining
   obligations and evidence links. A commit checkpoint is not package closure.
5. Keep discoveries under their AQ/package. Do not expand into compiler parity,
   arbitrary runtime evaluation or another strategy solely because syntax is
   nearby. Any genuine scope change needs an explicit owner decision; required
   existing questions cannot be silently dropped either.
6. Re-estimate only after broad UNRECONCILED packages have been inspected. Until
   then, report remaining packages/questions and concrete blockers, not a guessed
   percentage or a promise that the next GREEN finishes the campaign.

The bounded V1–V4 slice is verified and W04 has been reconciled above. The next
planning action is a production trace of the existing provider owner/context
path (P1/P3) to select a concrete scope contract before further implementation.
No production changes or new tests were made for this reconciliation. No Cargo,
binary, server or field harness was run. Static link/UTF-8/whitespace validation
does not certify completion of any implementation package.
