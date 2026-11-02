# Issue #162: exact source-declared loadChildren selection

Agent: Agent MaxHeadRoom
Status: generation-75 scoped Rust/lint GREEN (221 executions); bounded binary PASS (405 Module + 405 RouteConfig + 305 lazy + 305 provider); broader gates and larger-fixture timeout open.
Discovery expansion and runtime routing/provider meaning remain NOT APPROVED.

## Generation-75 bounded same-binary checkpoint inspected PASS

Inspected load-children75-binary-20261102-063755, selection after-modules:
RouteConfig 405/405, lazy-Component 305/305 and route-provider 305/305 checks pass.
Each receipt records zero failed checks/errors, three FINISHED sessions and the
hard 500-check ceiling. Combined with the preserved Module 405/405 receipt, all
four bounded suites complete: twelve FINISHED sessions, no suite errors.

All four share binary SHA-256
78b742b520de07a8c5ae1b894cc1fdc8e6bf87b59fe0df10ebb5740953d5255b
and recorded input hash
df690d8ae1b032e048079568881ddafacb4eb17d122f6c196da2d3bafc66dcc1.
Combined receipt/report hashes: target/issue-162/load-children75-binary-receipt.json.
The Module suite was not repeated by the after-modules selection.

This supersedes corresponding bounded-suite UNRUN/pending statements below.
Generation-75 has 221 scoped Rust passing executions, clean lint and inspected
bounded binary/stdio acceptance as distinct evidence classes. The deterministic
map and bounded binary gate are satisfied within the approved envelope. No
production/test/harness edit accompanies inspection and no agent runtime ran.
The preserved larger 85-TypeScript-file config-transition timeout remains OPEN;
narrowed acceptance does not establish a performance repair or broader scale
acceptance. Full repository/final Claude field and broader AQ12/AQ13/AQ14/W06/
P1/P3/W04/#162 closure remain open.

## Bounded Module binary checkpoint inspected PASS

Inspected load-children75-binary-20261101-233051, selection modules-only:
load-children-modules records 405/405 passing checks, zero errors and all three
sessions FINISHED. The narrowed scenario completes the previously timed-out config
mutation/repair with the unchanged 120-second request policy. This establishes
bounded Module acceptance; it does not resolve the preserved 85-TypeScript-file
config-transition performance finding or certify broader scale behavior.

Binary SHA-256 remains
78b742b520de07a8c5ae1b894cc1fdc8e6bf87b59fe0df10ebb5740953d5255b.
Recorded input hash:
df690d8ae1b032e048079568881ddafacb4eb17d122f6c196da2d3bafc66dcc1.
Receipt/report hashes: target/issue-162/load-children75-modules-binary-receipt.json.
No production/test/harness edit accompanies inspection; no agent runtime ran.

Next run Selection after-modules against this exact binary, preserving the passing
Module receipt. RouteConfig, lazy-Component and route-provider receipts must share
its input/binary hashes before combined checkpoint acceptance. Full repository,
final Claude field, broader issue closure and the larger-fixture timeout stay open.

## Repeated post-config timeout isolated; narrow Module checkpoint authored, UNRUN

Inspected load-children75-binary-20261101-223114 and Module receipt under
2026-11-02T03-31-19-234Z-load-children-modules-summary-18036: again 364/364 recorded
checks pass, first two sessions FINISHED, third FAILED. Timeout is request 64,
the first Route/same query after adding baseUrl to tsconfig. The preceding successful
same queries were in the previous pre-mutation fact batch; the earlier description
must not imply that they proved the changed-config query could complete. Duplicate
query removal reduced calls but did not repair the observed timeout boundary.

The recorded fixture contains 85 TypeScript files: scenario inheritance brought
unrelated bootstrap/legacy/DI/HTTP/provider files into this role-specific check.
Narrowed only the scenario's inherited base to its seven declared core/router/
Component/token dependencies. All Module/RouteConfig target/caller files, native
and foreign roots, callbacks, guard/resolver/provider/eager/lazy oracles and every
mutation/repair remain. Shared runner defaults and preservation suites are unchanged;
no global cap/cache/timeout/retry or production behavior changes. This is a bounded
fixture-scope correction, not a production performance fix. The larger fixture's
config-transition timeout remains an open performance finding; no broader scale
acceptance or full-suite PASS is inferred.

Exact pre-change report/harness/RPC/stderr and hashes:
 target/issue-162/load-children75-config-transition-timeout/receipt.json.
The checkpoint now accepts Selection modules-only for the next focused rerun,
then after-modules for the other three pending suites once Module completes.
Each selection gets its own explicit receipt; overall four-suite acceptance still
requires all four complete receipts with matching input/binary hashes. Reuse the
same hash-verified binary; do not rerun 221 passing Rust tests. New narrower Module
result is UNRUN; no agent Node/build/binary/runtime ran. Repository/field gates stay
open, including this preserved larger-fixture timeout.

## Binary timeout after passing assertions; duplicate query removal authored, UNRUN

Inspected load-children75-binary-20261101-212706, Module report under
2026-11-02T02-27-11-051Z-load-children-modules-summary-7528: 364/364 recorded checks
pass, first two sessions FINISHED, third FAILED. Overall FAIL is correct: a 120s
request timeout at stale-role-export-package-config-repair/85/workspace_query
prevents session completion and final binary/input checks. Remaining suites were
not reached. Do not report this as binary acceptance or failed semantic assertion.

The RPC trace records the same outgoing Route/same query immediately preceding
the timeout returning in about 7 seconds three times. The harness repeats that
query separately for provider/guard/resolver assertions after already obtaining
the exact fresh response in the current fact batch. Those three preservation
assertions now consume that same response locally. Every oracle/count, independent
fact batch, mutation/repair, repeated cold/warm query and session remains intact.
There is no cross-batch cache, retry, timeout increase or production change. This
removes redundant harness work; it is not evidence of a production performance
repair, nor a guarantee that the rerun will complete within the existing timeout.

Prior harness/report/RPC/stderr bytes and hashes:
 target/issue-162/load-children75-redundant-query-timeout/receipt.json.
Rerun the four pending suites using the same hash-verified binary. A fresh complete
Module receipt remains required; the previous two finished sessions are preserved
as partial evidence, not relabeled whole-suite PASS. No 221-pass Rust rerun or
rebuild is required for this harness-only change. New binary result is UNRUN;
no agent Node/build/binary/runtime ran. Repository/final field gates remain open.

## Foreign-root legacy binary fixture correction authored, UNRUN

Inspected load-children75-binary-20261101-212056 and its Module suite receipt:
18/19 checks pass, then q12/foreign-legacy-preservation expects one row and gets
zero. The first exact forward/reverse canonical checks, owned refusal and mutable
legacy checks pass, but no binary suite/session acceptance follows from this
partial run. RouteConfig/lazy-Component/route-provider suites were not reached.
Syntax preflight/build completed; runtime receipt records the exact rebuilt binary.

The foreign-root fixture imports Routes only from a local foreign module. The
unchanged routing::extract_route_shape_at_with_regions gate returns None unless
source contains @angular/router, independently of Angular class applicability.
The fixture therefore did not qualify for the old routing contribution it expected.
This is an invalid compatibility control, not evidence of canonical semantic failure.

Added an unrelated native Route type import to that fixture while retaining its
foreign Routes root annotation. Expected legacy row/count and all canonical
assertions are unchanged. No Rust production/test changes. Prior harness, receipt
and log bytes/hashes: target/issue-162/load-children75-foreign-fixture-correction/receipt.json.

The checkpoint accepts ExpectedExistingBinarySha256 to reuse only the exact prior
rebuilt binary after this harness-only correction; runner Rust-input freshness,
binary/input equality, schema and bounded receipts still apply. Rerun all four
pending suites, starting Module from a fresh workspace/session. Do not repeat the
221 passing Rust executions or rebuild unchanged Rust. Corrected binary gates
remain UNRUN; no agent Node/build/binary/runtime ran. Broader gates stay open.

## Generation-75 ecological binary checkpoint authored, UNRUN

Reconciled the 57-item map against passing exact forward/reverse, role/refusal,
retained/lifecycle and preservation assertions plus unchanged emission/encoding
review. No additional deterministic edit was needed. The 221 scoped passing
executions and clean lint remain valid; binary and full repository/field evidence
are separate.

Added load-children-modules and load-children-routes using the existing binary/
stdio runner and one shared scenario factory with distinct target roles. Each
logical suite has three sessions, an existing hard 500-check ceiling and static
budget of 405 checks; actual counts are unobserved. Fixtures cover nested direct/
re-export/alias callers, two same-named targets, repeated/empty/child objects,
provider-bearing/free routes and exact forward/reverse target/source multisets.
RouteConfig targets are native empty provider-free typed const Routes Arrays.

Cold/warm/withinPath, checkpoint/restart, repeated-object/field/child/provider
retirement, independent caller deletion/recreation, stale restore before queries,
selected-role repair, export removal/retarget, package/config mutation and repair
are batched. Owned unsupported/duplicate callbacks have no weak fallback. Eager,
loadComponent, route-provider and existing guard/resolver contributions survive.
Mutable and foreign-root rows are observed only as noncanonical legacy behavior;
this does not certify guard/resolver applicability or AQ13 context.

Handoff: verification/live-acceptance/issue-162/run-load-children75-checkpoint.ps1.
Operator runs syntax preflight, one all-feature final binary rebuild with unique
metadata, then load-children-modules, load-children-routes, lazy-route-components
and route-providers against that binary. The checkpoint validates shared recorded
input hashes as well as binary hash and retains summary paths/hashes. Each suite
keeps its own below-500 receipt. All new binary checks are UNRUN. No Rust production/
test edit accompanies authorship; no agent Node/build/binary/runtime ran. Full
repository/final Claude field and broader issue gates remain open.

## Generation-75 focused Rust/lint inspected GREEN

Inspected load-children75-focused-20261101-210937: corrected all-target/all-feature
Clippy records Finished with no warnings/errors under -D warnings. Format/fmt logs
are empty; active sizes pass. Resume ran no tests, preserving all 221 passing
executions from the preceding focused run. This supersedes the corrected lint/fmt
UNRUN statements below; the older failed Clippy log is preserved as history.
Summary and log hashes: target/issue-162/load-children75-focused-receipt.json.

Original six forward/reverse semantic controls and six acquisition probes now pass,
alongside eight integrated production/lifecycle batches and selected preservation.
This is scoped Rust evidence, not unique tests, full repository or binary/field
acceptance. No production/test/harness edit accompanies inspection; no agent runtime
ran. Next: reconcile the 57-item acceptance map against actual assertions and batch
any uncovered deterministic cases before one major rebuilt-binary checkpoint.
Full repository/final Claude field and broader issue closure remain open.

## Focused tests GREEN; test-only Clippy correction authored, UNRUN

Inspected load-children75-focused-20261101-164910: all selected tracked tests pass,
221 executions total, zero failed/ignored. This includes original six semantic
REDs and six acquisition probes, eight integrated controls, neutral/field and
preservation filters. These are scoped executions, not unique/full repository tests.
Clippy then rejects two useless conversions at control lines 46/48: EntityRef's
entity_type is already &str, so the forged-role control's RouteConfig/Module
literals need no .into(). Both conversions are removed. No production behavior,
assertion meaning or original frozen probe changes.

Prior control/Clippy bytes and test-log hashes are preserved at
 target/issue-162/load-children75-clippy-correction/receipt.json.
Use run-load-children75-focused.ps1 -ResumeAtLint: format/sizes and Clippy/fmt only,
without repeating the passing tests or compile-only step. Corrected lint/fmt gates
remain UNRUN; no agent formatter/Cargo/runtime ran. Binary, acceptance-map review,
full repository/final Claude field and broader issue gates remain open.

## Approved generation-75 production checkpoint authored, UNRUN

The supplied production disposition approves canonical LoadsChildren from existing
RouteOccurrence, complete unique ordinary loadChildren selection, distinct native
Module and typed-const Routes Array target roles, owned selective replacement,
outside-envelope legacy preservation and semantic generation 75. No durable bump,
new public vocabulary/payload/resolver/cache/registry/route tree/discovery change
is introduced. Marker and pipeline generations remain unchanged.

route_load_children consumes shared route_objects membership, exact unique field/
value containment and current arrow_selection_at replay. Module uses the existing
native class/structure proof. RouteConfig uses the existing qualified native typed
const root/initializer proof plus Array requirement, without providers or nonempty
requirements. The role branches are separate and competing proof is refused. Both
produce actual selected declaration names/files and canonical targets. Retained
LoadsChildren admission recomputes this exact projection; no generic endpoint
fallback is added. Existing source-owned snapshot/SQLite/index lifecycle is reused.

The production pipeline emits the new projection while routing is enabled. It
suppresses only the old resolved_load_children branch for actual object spans in
the shared native ownership envelope carrying loadChildren, before weak origin is
lost. Ownership is independent of callback/export/role validity, so refused or
duplicate owned fields never downgrade to legacy labels. Outside ownership remains
legacy. Eager/loadComponent/providers/guards/resolvers/display emission is preserved.

Original semantic REDs, six acquisition probes and registration were raw-byte
isolated under target/issue-162/load-children75-approved-isolation/ before repair,
then restored exactly. This is not a Git stash; HEAD/normal index were untouched.
New integrated registrations were appended after restoration. The historical
field-only public count oracle now requires six same-path/four child canonical
rows for every export form; field/containment/serde assertions remain unchanged,
and prior bytes are preserved. Original genuine semantic REDs are unchanged.

Eight integrated batches cover exact separate native target roles and forged/
missing/crossed canonical/carrier/native/const/annotation/initializer proof;
owned wrong/private/type-only/ambiguous/foreign/competing/unsupported/duplicate
refusal without fallback; empty/provider-free config acceptance; outside legacy
and mixed eager/loadComponent/loadChildren/provider independence; individual
object/child/field/empty-root retirement and repair; complete SQLite snapshot/index
before queries; all-form/both-role cold/warm/refresh/withinPath and independent
caller/target deletion/recreation; unchanged-caller export/barrel/package/config
stale restore/retarget/refusal/repair. These are authored tests, not certification.
The [production acceptance map](ISSUE_162_LOAD_CHILDREN_ACCEPTANCE.md) records each
bounded obligation and later gate without inferring acceptance from authorship.

Handoff: verification/live-acceptance/issue-162/run-load-children75-focused.ps1.
Operator formats only unfrozen files, checks sizes, compiles, runs original exact
semantic/acquisition controls then integrated/neutral/field/eager/lazy/routing/
generation/provider/Module/bootstrap/config preservation, Clippy and fmt. All
post-repair gates UNRUN. No agent formatter/Cargo/build/Node/binary/runtime ran.
After results, reconcile deterministic acceptance before one later consolidated
rebuilt-binary checkpoint, each logical suite below 500. Full repository/final
Claude field and broader AQ12/AQ13/AQ14/W06/P1/P3/W04/#162 closure remain open.

## Intended LoadsChildren association RED observed; neutral mechanism GREEN

Inspected shared-written-selection-20261030-144015: second-origin neutral 5,
shared role/origin controls 3, first-origin neutral preservation 5, field 3,
canonical lazy semantics 3, lazy integrated 7, eager Component 21, routing 26,
route provider 11, provider_semantics 6, Module 14, modern bootstrap 8, legacy
bootstrap 14, config composition 17, ApplicationConfig 10 and inline bootstrap 13
pass (166 executions). Original six unchanged acquisition probes pass separately,
80.32 seconds. Total scoped passing executions: 172, zero failed/ignored in those
filters, not unique tests or full repository coverage. Clippy finished without
warnings/errors under all-targets/all-features -D warnings; compile/format/fmt logs
are empty and active sizes pass.

All six additional Module/Routes direct/re-export/alias probes reach line 68,
stage=load-children-association direction=forward: zero passed, six failed,
zero ignored, 143.42 seconds. Exact neutral selection and separate selected Module
or native typed-const Routes facets/package/initializer prerequisites pass for
all fourteen object/target associations before the candidate assertion. This is
the intended association RED, not another callback/target-role prerequisite.
Later reverse comparisons and repeated-query assertions remain unclassified after
the forward panic, although initial public answers are preserved.

Original acquisition probe bytes remain identical to isolation. Tests/registration/
logs and hashes: target/issue-162/load-children-association-red-evidence/receipt.json;
observed-public.json retains all six printed response sets. No production/test edit
accompanies inspection and no agent runtime ran. Neutral carrier reuse requires no
new payload and generation remains 74.

## Concrete LoadsChildren production disposition requested

Approve LoadsChildren from the actual existing RouteOccurrence to the exact
canonical declaration selected by the unique ordinary loadChildren field's complete
arrow_selection_at proof. Keep two explicit framework target branches: Module
requires exact native NgModule class role/structure; RouteConfig requires exact
source-qualified native typed-const Routes Array declaration and initializer proof,
without requiring providers, a nonempty Array or a Component relation. A plain class
or untyped/mutable/foreign Array must not substitute. Shared neutral selection does
not make these roles equivalent. Reject competing/ambiguous target-role proof.

Use existing public LoadsChildren vocabulary with Module and RouteConfig entity
types respectively, retaining actual selected declaration names/files. Canonical
RouteConfig denotes that exact exported Routes declaration, not evaluated routes,
a merged runtime configuration, an active router, module instance, loading success
or provider applicability. No new public field/relation is proposed. Request
explicit approval for this target-role/public identity interpretation.

Observed legacy behavior is the same for both target-role questions:

| Export form | Forward same LoadsChildren | Forward parent/child | Initial reverse target |
| --- | --- | --- | --- |
| Direct | 2 | 2 | 6 |
| Re-export | 0 | 0 | 0 |
| Alias export | 0 | 0 | 0 |

The direct rows attribute targets to asserting caller files; equal path/name/file
facts collapse distinct repeated objects and same-name selected targets. Candidate
proof requires six same-path and four child forward pairs, fourteen total reverse
pairs including empty paths, across two roots with actual target files. Initial
reverse labels are observations, not successful exact-multiset comparisons.

Recommend selective REPLACE of only the old resolved_load_children contribution
for loadChildren-bearing objects in the existing shared native typed-const Routes
ownership envelope. Select by actual object/field source spans before old emission
loses origin; never by path/name, target type or generic LoadsChildren sweep.
Suppress the owned weak branch even when callback/export/role/duplicate proof is
refused, preventing label fallback. Valid siblings survive. Preserve eager and
loadComponent branches, providers, guards/resolvers and display policy independently.

Recommend PRESERVE AS LEGACY outside the shared admitted ownership envelope,
including existing untyped/mutable/foreign/unadmitted/spread-container behavior,
without certifying canonical membership. This compatibility choice requires
explicit disposition. It must not supply fallback for an owned field. Do not
publish old and new facts side-by-side for the same migrated written association.

Producer and retained relation admission must share one exact projection: root/
object/field/value containment, current neutral callback/export selection and the
appropriate independently proved target role. Existing generic endpoint admission
cannot establish route membership. Current relation_evidence accepts (None,None)
compatibility facts but refuses generic RouteOccurrence endpoints; introduce no
generic exception. Reuse source-owned snapshots, witnesses, SQLite/index/currentness,
refresh, deletion/recreation and answers-only withinPath. No resolver/cache/registry,
new route tree, discovery or global EntityRef equality change is proposed.

Request existing Angular semantic compatibility advancement to generation 75 for
the changed public identity/role meaning, retaining marker/pipeline generations.
No durable bump is proposed absent an actual encoding change; no new retained
payload is currently demonstrated necessary. Return before any such expansion.

After approval: isolate/restore unchanged valid REDs and original probes; batch exact
forward/reverse Module/RouteConfig same-name/repeated/empty/child/provider-free rows,
complete retained/native-role/crossed/borrowed/duplicate/unsupported refusals,
owned no-fallback versus outside legacy preservation, independent eager/lazy/
loadChildren/provider retirement, selected target/route/API/barrel/package/config
mutation/repair, SQLite/index before queries and cold/warm/refresh/withinPath.
Preserve routing, Module, providers and all bootstrap/config siblings with clean
lint, then one consolidated rebuilt-binary checkpoint with each suite below 500.
Operator owns all execution; full repository/final Claude field stays later.

The supplied neutral disposition explicitly withholds canonical LoadsChildren,
Module/RouteConfig publication and legacy replacement. AGENTS.md Architectural
Approval Gate applies to this externally visible identity/admission migration.
Await production disposition; no semantic repair or unchanged rerun is requested.
Broader AQ12/AQ13/AQ14/W06/P1/P3/W04/#162 closure remains open.

## Shared neutral mechanism approved and authored, UNRUN

The supplied owner disposition approves one explicit shared demand-origin predicate
for loadComponent/loadChildren acquisition and replay. Existing ArrowSelection,
source-qualified spans/joins, binding proof, Import module request, complete unique
Value export resolution and witness/retained lifecycle are reused unchanged. Replay
also refuses competing duplicate mappings of the selected approved field name;
two distinct approved fields in one object retain independent occurrence proofs.
No new payload/discriminator, syntax expansion, role, resolver or durable format is
added. Semantic generation remains 74. No canonical LoadsChildren producer,
admission exception or legacy suppression is introduced.

Original six probes/registration were raw-byte isolated before repair under
 target/issue-162/shared-selection-approved-isolation/, then restored exactly;
new registrations were appended afterwards. This is not a Git stash; HEAD/index
were untouched. Historical field tests now explicitly allow acquired neutral proof
while preserving their Unsupported-without-proof refusal, field/serde/transport
and legacy public oracles. Previous bytes are preserved in the isolation directory.

Eight new neutral batches cover the second origin's exact forms/serde/old payload,
24 forged/crossed/borrowed records, syntax/export refusal and ordinary classes;
SQLite before consumer, dependency/config/package/unrelated and owner repair;
Module/typed Routes/ordinary const selections across all export forms, exact
selected-target facets and separate native role prerequisites; duplicate/unapproved
origins and both approved fields in one object. Existing first-origin neutral and
production controls remain preservation gates. The
[shared neutral acceptance map](ISSUE_162_SHARED_SELECTION_ACCEPTANCE.md) links the
50 owner obligations to these assertions; all post-edit gates remain UNRUN.

Six additional semantic investigation probes require complete neutral selections
and separate selected Module/Routes proof before exact fourteen-pair public
forward/reverse comparisons. Original six acquisition probes remain unchanged.
Their success alone would not certify semantic RED. A selected-target prerequisite
failure must be returned; no role/initializer capture extension is preemptively made.

Handoff: verification/live-acceptance/issue-162/run-shared-written-selection.ps1.
Operator formats only unfrozen files, checks sizes, compiles, runs new neutral and
existing first-origin/eager/lazy/provider/Module/bootstrap/config preservation,
then Clippy/fmt, original six probes and complete selected-role semantic probes.
No agent formatter/Cargo/Node/build/runtime ran. Observe actual next prerequisite
or preserve genuine association RED, then return for production disposition.
Full repository/final Claude field and broader issue gates remain open.

## Post-field callback boundary observed; preservation GREEN

Inspected load-children-field-capture-20261030-133252: neutral fields 3, canonical
lazy semantics 3, lazy integrated controls 7, eager route Component 21, routing 26,
route providers 11, provider_semantics 6, Module 14, modern bootstrap 8, legacy
bootstrap 14, config composition 17, ApplicationConfig 10 and inline bootstrap 13
pass: 153 executions, zero failed/ignored. Clippy records Finished without warnings/
errors under all-targets/all-features -D warnings. Compile/format/fmt logs are empty;
active sizes pass. This is scoped evidence, not unique tests/full repository count.

Unchanged investigation: zero passed, six failed, zero ignored, 67.93 seconds.
All Module/Routes direct/re-export/alias probes pass retained field mapping and
containment, then stop at line 50, stage=callback-structure-prerequisite. Values
are Unsupported("arrow_function") with actual caller spans. The field gap is
resolved. Later import/selector/export/selected-target association remains
unclassified; this is NOT intended LoadsChildren semantic RED. Independent roles
remain separate from selection proof.

Original probe bytes match the approved raw-byte isolation. Exact test/registration/
logs and hashes plus extracted public responses are preserved under
 target/issue-162/load-children-callback-prerequisite-evidence/receipt.json and
observed-public.json. Existing public labels remain compatibility observations;
field-only neutral tests preserve their direct versus re-export/alias contribution.
No production/test edit accompanies inspection and no agent runtime ran.

## Bounded shared neutral written-selection disposition requested

Approve a minimal generalization of the existing written-selection mechanism for
already-demanded ordinary loadComponent OR loadChildren fields, retaining the same
bounded callback syntax and proof replay. Do not duplicate ArrowSelection or its
resolver. The architectural question is whether the field name is a neutral
language fact or the Angular consumer that demanded it: current payload facts
encode source/hash/spans, parameter binding, import/module/public export and exact
DeclarationId, with no loadComponent field discriminator or Component role. The
field spelling is the demand origin; framework target interpretation is separate.

Static reuse analysis after the observed boundary:

| Boundary | Current implementation | Minimum proposed disposition |
| --- | --- | --- |
| Acquisition trigger | declaration_facets.rs nominates only key == loadComponent | Nominate the same bounded capture for the two explicit already-demanded field names |
| Replay/admission | arrow_selection.rs::field searches only loadComponent mapping | Replay an exact unique ordinary owning mapping for either approved demand origin, preserving source/hash/containment and competing-record refusal |
| Selection identity | Exact outer/returned/import/selector/parameter/member spans and source-qualified target | Reuse unchanged; no property name or independently valid target can replace joins |
| Carrier | ArrowSelection has no Angular field or role discriminator | No new payload or encoding demonstrated necessary; old missing-proof snapshots remain unproved |
| Resolver | acquire.rs uses module_request Import and complete unique export_set Value | Reuse unchanged; no filesystem/name fallback |
| Target role | Neutral selected DeclarationId; existing consumer independently requires Component | Keep neutral value-export selection distinct from later NgModule or typed-const Routes interpretation |

This is a proposed implementation contract, not permission to modify the trigger
or replay now. Prefer one small shared explicit demand-origin predicate reused by
acquisition and replay if needed to prevent drift. It must not become a global
arrow scan or an arbitrary property evaluator. Keep callbacks outside these two
already-demanded field origins unsupported. For two mappings/competing records,
fail closed rather than silently choosing one. Metadata with either field spelling
may retain neutral language proof without gaining Angular routing meaning.

No carrier or resolver limitation has yet been observed after complete callback
acquisition for Routes constants. Test that case explicitly rather than asserting
its target declaration/initializer demands are already certified. No semantic
identity changes, neutral syntax expansion, generic endpoint admission, Angular
role publication, semantic generation or durable format advance is proposed.
If actual representation or selection requirements exceed this contract, return
with the exact new boundary before expanding.

After approval: isolate/restore original probes unchanged; batch both demand
origins across Module, typed Routes and ordinary non-Angular value exports,
direct/re-export/alias resolution, forged/crossed/borrowed field/parameter/module/
export records, duplicate and unsupported callbacks, serde/SQLite before queries,
owner/dependency/config/package mutation/repair, and generation-74 eager/lazy/
provider/Module/bootstrap/config preservation. Rerun the six probes only after
neutral/lint gates. Their field/callback success alone cannot certify selected
export/role or exact public relation identity; extend tracked investigation
controls to require complete joins before declaring genuine association RED.

The field-only owner disposition explicitly requires returning at this boundary
and withholds callback reuse/generalization. AGENTS.md Architectural Approval Gate
therefore requires a separate bounded neutral disposition. Canonical LoadsChildren,
Module-versus-RouteConfig publication, selective replacement, discovery, runtime
router meaning and broader issue closure remain NOT APPROVED. No unchanged rerun
is requested now; operator owns execution and final Claude field remains last.

## Bounded field acquisition approved and authored, UNRUN

The supplied owner disposition authorizes adding loadChildren to the existing
Angular DECLARATION_FIELDS set only. That one field-name addition is the entire
production edit. Native Routes/const/Array/object/children/discovery policy,
callback triggers/replay/identity, target-role policy, semantic producer/admission,
legacy replacement and semantic generation 74 remain unchanged. Existing retained
Object/value/source/hash/witness transport is reused; no payload or durable bump.

Corrected six probes and registration were raw-byte isolated before the field
edit under target/issue-162/load-children-field-approved-isolation/, then restored
exactly. This is not a Git stash; HEAD and normal index were untouched. A neutral
test registration was appended only after restoration.

Three tracked neutral batches cover seven independent fields per root across
Module/Routes and direct/re-export/alias fixtures, repeated/empty/child/provider
objects, source/hash and serde; duplicate fields and non-route metadata neutrality;
SQLite field restoration before queries and actual current-owner-hash retirement/
restoration. Unsupported callbacks remain unproved; no arrow-selection nomination
is added. Legacy public counts use the observed direct versus re-export/alias
contribution as a preservation oracle, not certified selected-export authority.
Independent target-role and full route-owner controls remain in the unchanged
six probes. These are authored tests, not runtime acceptance.

Handoff: verification/live-acceptance/issue-162/run-load-children-field-capture.ps1.
Operator formats only unfrozen files, checks sizes, compiles, requires neutral and
eager/canonical lazy/routing/provider/Module/bootstrap/config preservation GREEN,
then warning-free Clippy/fmt before recording unchanged six probes. Expected
callback failure remains static until observed. No agent formatter/Cargo/Node/
build/runtime ran. All new/post-edit checks UNRUN. No binary checkpoint requested.
Return with the earliest actual post-field prerequisite and explicit trigger versus
replay/identity/carrier/resolver/target-role reuse analysis; do not repair callback
authority or publish canonical LoadsChildren under this field-only disposition.

## Module field boundary confirmed; six prerequisite failures preserved

Inspected load-children-investigation-20261030-132432: corrected Module probes
record zero passed, three failed, zero ignored, 41.49 seconds. Direct/re-export/
alias each pass independent native Module role and eager/provider/canonical
caller-root controls, then stop at line 36, stage=loadChildren-field-prerequisite:
expected one retained field, received zero. Combined with the three preserved
Routes failures, all six questions now reach the same observed field boundary.
No passing tests or semantic association RED is claimed. Later callbacks,
selector/module/export and fourteen-pair semantic assertions remain unclassified.

Compile/format logs are empty and active sizes pass. Exact corrected test,
registration and both runs' logs/hashes are preserved under
 target/issue-162/load-children-field-prerequisite-evidence/receipt.json;
observed-public.json retains the six printed original public response sets.
The prior invalid Module fixture logs remain historical evidence, not semantic RED.
No production/test edit accompanies inspection and no agent runtime ran.

## Bounded loadChildren field-capture disposition requested

Approve adding loadChildren to the existing Angular DECLARATION_FIELDS demand set
only, plus tracked neutral field/containment/serde/SQLite/currentness and affected
preservation controls. Reuse existing source-qualified Object/value/reference
facets and witnesses for already-demanded objects; no root/discovery widening,
new payload, callback/export authority or semantic generation advance is proposed.
As with existing demanded field names, other already-demanded metadata can retain
this field without gaining route roles. Return if scope differs from that demand
mechanism rather than widening it implicitly.

Preserve corrected probes/registration bytes before implementation, restore them
exactly and rerun after neutral/preservation gates. Observe the next prerequisite;
do not substitute old labels, independent native targets or Unsupported arrow
text for complete callback/export proof. Current neutral arrow acquisition and
replay are loadComponent-specific, so a field-only change is not certification
that loadChildren callbacks will be proved. Any reuse/generalization of that
neutral authority needs a later bounded disposition at the actual observed gap.

Production LoadsChildren identity/admission, Module versus RouteConfig target
policy, selective old-row retirement and outside-ownership compatibility remain
unapproved. Preserve all existing eager/lazy/provider/loadChildren behavior in
this acquisition-only slice. No new public vocabulary, resolver/cache/registry,
route tree, runtime router meaning or durable encoding is proposed.

The investigation approval requires returning on missing acquisition before
extending it. AGENTS.md Architectural Approval Gate applies to the new acquired
authority. Owner disposition is requested for this field-only change; no unchanged
rerun is needed now. Full repository/final Claude field and broader issue gates
remain open.

## Routes field boundary observed; Module fixture correction authored, UNRUN

Inspected load-children-investigation-20261030-131316: zero passed, six failed,
zero ignored, 39.93 seconds. All three Routes target probes pass independent
native target, eager/provider and canonical caller-root controls, record public
answers and stop at stage=loadChildren-field-prerequisite, line 36: expected one
field, received zero. Callback/export/association assertions remain unclassified.
This is a prerequisite boundary, not intended semantic RED.

The three Module probes stop earlier at stage=native-Module-control, line 113:
expected Module, got None. The shared angular_authority_fixture installs only
Injectable and Component, so the new fixture's NgModule import has no declaration.
That is invalid fixture evidence. The Module-only fixture now appends the native
NgModule declaration to its existing core package before capture. No production,
capture or expected role assertion changes. Exact prior test/registration/log
bytes and hashes: target/issue-162/load-children-module-fixture-correction/receipt.json.
Compile/format logs are empty; active sizes pass.

Use run-load-children-investigation.ps1 -ResumeModuleFixtureCorrection to run only
the three corrected Module probes. The three observed Routes boundaries remain
preserved. Corrected Module results are UNRUN; no agent formatter/Cargo/runtime
ran. Return with the Module boundary before consolidating a bounded field-capture
disposition. No production semantic repair or unchanged Routes rerun is requested.

## Owner investigation disposition applied; six probes authored, UNRUN

The owner's "ok continue" approves the requested bounded investigation and tracked
tests only. Six independent probes cover Module and typed Routes targets across
direct/re-export/alias forms, two same-named target files and two caller roots.
Each caller has seven loadChildren objects, including repeats, empty/child paths
and provider-bearing/free cases, plus an eager sibling. Independent native target
and canonical provider/root/object controls precede registered public observations.

Forward same/parent-child and reverse target answers are printed before exact
retained field and callback prerequisites. The current loadChildren field omission
is a static expectation, not an observed failure. Full callback/module/export/
role joins, exact fourteen-pair semantic comparisons and refused/forged/lifecycle
cases remain unclassified pending the actual earliest boundary. These six tests
are acquisition probes, not yet valid semantic RED controls. No capture changed;
semantic generation stays 74 and prior acceptance remains historical scoped evidence.

Handoff: verification/live-acceptance/issue-162/run-load-children-investigation.ps1.
Operator formats only new test/registration, checks sizes, compiles and records all
six together. All new checks are UNRUN; no agent formatter/Cargo/runtime ran.
Return with actual earliest prerequisite and public contributions before further
capture or semantic disposition. No binary run requested; final field stays last.

## Canonical questions and bounded exit

For this exact object in an admitted native typed const Routes root, which exact
native NgModule declaration is selected by its ordinary loadChildren callback?
Separately, which exact native typed const Routes declaration is selected when
the public export is a route configuration? Start with
() => import('./feature').then(m => m.FeatureModule) and the same callback shape
selecting an exported Routes const. These are two distinct target-role questions.
Neither implies loaded code, active routes, module instantiation, navigation,
provider applicability or application context.

Generation-74 eager/lazy Component acceptance remains valid. Its disposition
explicitly preserved loadChildren rather than migrating it. AQ12/W06 target
identity remains open for this branch; AQ13 context and broader AQ14 obligations
remain separate. Hypothesis: reuse LoadsChildren, the actual RouteOccurrence and
exact canonical Module or RouteConfig declaration, subject to explicit vocabulary/
target-role review. No new public vocabulary is proposed at investigation time.

## Located paths and earliest static boundaries

routing/structured.rs reads loadChildren through lazy_evidence and resolve_lazy.
The latter resolves a direct relative file, checks declaration_kind for Module
or has_routes_const for RouteConfig, and records a file dependency. The retained
LazyRouteTarget has only entity_type/name. This does not itself demonstrate the
exact callback parameter/public-export join or canonical route-object identity.
The current production branch remains separate from eager/loadComponent
suppression. No observed failure is inferred from this static description.

module_metadata::DECLARATION_FIELDS contains loadComponent but omits loadChildren.
declaration_facets currently nominates neutral arrow-selection acquisition only
for loadComponent. Existing arrow_selection_at therefore is a reuse candidate,
not proof that this new field is captured or replayable. It must not be called
with unrelated proof or granted authority by property spelling alone.
Current relation_evidence has relation-specific canonical RouteMapsTo admission;
that does not authorize canonical LoadsChildren through generic endpoints.

Inspected known paths: src/angular_meta/module_metadata.rs;
src/angular_meta/routing.rs; src/angular_meta/routing/structured.rs;
src/ir/typescript_resolution/declaration_facets.rs;
src/ir/typescript_resolution/relation_evidence.rs.

## Recommended investigation contract

Approve tracked investigation/test authoring only for the existing native const
Routes Array/Object/children envelope and the already bounded ordinary zero-arg,
non-async expression arrow, relative literal dynamic import, one ordinary then
selector and exact simple parameter/member binding. Investigate direct public
exports first, then resolver-proved re-export/alias forms. Keep Module and typed
Routes target controls independent; a plain class or untyped Array is not enough.

Require native eager/lazy Component and route-provider siblings before candidate
assertions. Use two same-named target declarations in independent files, repeated
same-path objects, empty paths, children and provider-free objects. Record existing
registered forward/reverse answers before requiring exact retained field/value,
callback/binding/module/export and separate target-role prerequisites. Observe
actual public entity types and source attribution; do not presume the candidate
vocabulary already has exact declaration semantics.

Missing field/callback/export/target proof is a prerequisite boundary, not semantic
RED. Extend no capture preemptively. Return the actual earliest missing join and
minimum bounded reuse proposal, including whether field-specific replay currently
prevents loadChildren reuse. Any acquisition extension needs its own disposition.
No second resolver, generic callback evaluator, new discovery, route tree, cache,
registry or lifecycle is authorized.

Batch unresolved/private/type-only/ambiguous exports, foreign Module/Routes roles,
wrong selector receiver, unsupported/computed/async/duplicate fields and valid
siblings. Broad forged/durable/mutation gates follow only after prerequisites can
be established. Do not classify initial old labels as certified association.

If genuine post-prerequisite association RED is reached, preserve exact tests/logs
and return with old/new carrier comparison and explicit ownership-based replacement
versus outside-ownership legacy policy before production changes. Preserve
loadComponent, eager routing, provider facts, guard/resolver behavior and marker
presentation. A previous loadChildren preservation pass does not authorize
retiring or strengthening that behavior.

## Alternatives and return point

Recommended: investigate Module and Routes selection as separate controls sharing
one exact callback syntax and current root envelope. This exposes role-specific
boundaries without assuming their semantics are interchangeable. A Module-only
first slice is smaller but leaves RouteConfig identity outstanding. A general
lazy/router evaluator is outside scope; target name plus file shortcuts do not
establish this callback's public-export selection.

Owner disposition requested for investigation and tracked tests only. Generation-74
approval explicitly excluded loadChildren migration. AGENTS.md Architectural
Approval Gate says: "Do not make any architectural decision with global,
cross-cutting, repository-wide, or externally observable effects unless that
decision was explicitly authorized by the task prompt or approved during the
current work." New capture, target authority, canonical publication/admission and
legacy replacement remain unapproved. No runtime handoff is needed yet. Final
repository/Claude field and broader AQ12/AQ13/AQ14/W06/P1/P3/W04/#162 remain open.
