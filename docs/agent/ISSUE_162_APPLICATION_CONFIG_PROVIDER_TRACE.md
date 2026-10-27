# Issue #162: source-declared ApplicationConfig providers

Agent: Agent MaxHeadRoom
Status: owner APPROVED; generation-67 scoped Rust checkpoint GREEN (459 passing executions; historical controls corrected); fmt/Clippy GREEN; approval coverage review and binary acceptance pending.

## Generation-67 scoped preservation and lint GREEN

Inspected the final timing controls: post-commit missing source 1 passed (7.65s)
and pre-commit stale-authority refusal 1 passed (4.63s). Final fmt log is empty
and the operator reports GREEN. All-targets/all-features Clippy records Finished
with no warnings/errors for -D warnings.

Across the logged preservation/resume batches, 426 passing executions are retained:
initial Angular/compression/baseline 386; publication/recovery/hydration 13;
durable restore 4 plus corrected rerun 1; restore compatibility 9 plus corrected
reruns 3; delta fidelity 8 plus corrected timing and new refusal controls 2.
The prior focused config/route/compatibility 33 bring the scoped total to 459.
These are executions, not unique tests or a full repository run. Historical five
failed executions and handoff interruptions remain preserved; their corrected
controls now pass, with no unresolved failure in the requested batch. Count/hash
receipt: target/issue-162/application-config67-preservation-receipt.json.

No production behavior changes accompany this evidence update. The test-only
acknowledgement hook does not exist in non-test builds. Broader config approval
coverage review, bounded binary acceptance, final repository/encoding/size and
Claude field gates remain open. No binary run or P1.A1/P1/W04/#162 closure is
inferred from these scoped Rust/lint receipts.

## Delta acknowledgement timing correction, UNRUN

All three corrected generation controls passed. Delta fidelity then recorded
8 passed, 1 failed: committed_delta_acknowledgement_survives_missing_source.
The test deletes its source before apply_delta, so pending_transition rejects
stale native authority before mutation. It never reaches committed acknowledgement
rendering. This is distinct from source becoming unavailable after commit.

The test now removes its source through a thread-local, cfg(test)-only hook after
semantic publication and transition consumption, immediately before response
rendering. Its successful target/version/consumption/skeleton assertions remain.
A separate tracked control requires deletion before acceptance to reject and keep
the baseline IR/version and pending transition unchanged. No currentness check is
weakened; non-test code behavior is unchanged. Original failure/test bytes are
archived at target/issue-162/application-config67-delta-ack-timing-correction/.
Both timing controls remain UNRUN. -ResumeAfterAcknowledgementFailure formats
the affected test/support/hook files and runs these two cases followed by fmt/Clippy;
the eight passing delta controls and earlier preservation passes are not repeated.

## Restore compatibility generation-control correction, UNRUN

The corrected native durable-delta control passed (1 passed, 5.60s). Restore
compatibility then recorded 9 passed, 3 failed (34.25s). The three failures use
obsolete test generation literals: C# semantic projection 4 versus current 5,
and TypeScript canonical 2 versus current 3. The C# mismatch string replacement
was a no-op, so restore correctly accepted the unchanged identity; the two
removal cases failed their old-generation prerequisites before testing restore.

The mismatch now parses stored JSON, verifies the production generation and
changes that exact field to the next generation. Removal controls verify against
existing production constants. Incompatible-restore and clean-state assertions
are unchanged. No production/generation change accompanies this correction.
Pre-correction bytes/log hashes are archived under
target/issue-162/application-config67-restore-generation-control-correction/.
The three corrected controls remain UNRUN. -ResumeAfterCompatibilityFailure
formats that test file, runs only those three cases and the remaining delta
fidelity filter, then fmt/Clippy. Earlier passing controls are not repeated.

## Durable-delta native fixture correction, UNRUN

The resumed preservation batch passed publication race 1, recovery 7, retry 1
and hydration fidelity 4 controls. Durable semantic restore records 4 passed,
1 failed at the baseline app-old selector assertion, before delta application
or restore. The fixture supplies an unresolved @angular/core import but no native
package proof. Nonempty framework/meta edges do not establish its Component role;
the native class-role projection requires exact public API proof.

The affected test now installs the existing angular_authority_fixture and checks
its native selector before editing the source. All delta rejection, pending-state,
application and complete durable restoration assertions remain intact. No production
or semantic generation change accompanies this fixture correction. Original test/log
bytes and hashes are preserved under
target/issue-162/application-config67-durable-native-fixture-correction/.
The corrected control is UNRUN. The handoff's -ResumeAfterDurableFailure runs only
that corrected case, the two remaining preservation filters and fmt/Clippy; prior
passing executions are retained, including the other four durable controls.

## Partial preservation GREEN; handoff filter correction

Inspected application-config67-preserve-20261027-091409: provider 85, view 6,
string 12, module 14, provider_semantics 6, compress 261 and baseline publication
2 passed: 386 passing executions, zero test failures. The next filename-derived
provide_publication_race filter selected zero tests, correctly stopping the script.
Remaining filters and fmt/Clippy did not run. This was an agent handoff error,
not production failure. The script now uses actual registered Rust module names
for publication, hydration, restore compatibility and delta fidelity. Its
-ResumeAfterBaseline option skips the seven completed filters. Original logs and
a partial count/hash receipt remain preserved in that timestamped directory.
No production/test assertion changed; remaining verification stays UNRUN.

## Broader compression/provider preservation handoff, UNRUN

The next operator batch selects existing provider/view/string/module/helper
controls and shared compression, baseline/publication race/recovery, hydration
fidelity, durable restore/compatibility and delta-fidelity controls. This scope
covers the shared compression publication repair beyond the config regression.
No production or test assertions change. The preceding 33 config/route/compatibility
passes remain applicable and are not scheduled again. The tracked handoff is
verification/live-acceptance/issue-162/run-application-config67-preservation.ps1;
it stops on failure or an empty filter, retains timestamped full logs, and runs
final fmt/Clippy after preservation. These commands are UNRUN. Config approval
coverage review and bounded binary acceptance remain open; no Node run is included.

## Generation-67 focused config/route/compatibility GREEN

Inspected the completed operator logs: ApplicationConfig 10 passed (98.74s),
route providers 11 passed (95.56s), Angular template compatibility 1 passed
(0.06s), and NgRx legacy transition 11 passed (57.62s). These 33 scoped
executions record zero failed/ignored and no warnings/errors. The compile-only
and repair-format logs are empty; the operator reports the conditional sequence
GREEN. Hashes/counts are retained in
target/issue-162/application-config67-focused-receipt.json.

This verifies the four unchanged original REDs and six integrated config controls,
including compression-to-index authority before queries and subsequent independent
owner retirement. It supersedes the post-correction UNRUN statements below for
these requested checks only. Historical failures remain preserved. No production
change accompanies this evidence update. Broader affected compression/provider
preservation, final fmt/Clippy and binary acceptance remain open. Node acceptance
stays batched at the next major integrated checkpoint; no immediate Node run is
requested. P1.A1, P3, P1/W04 and Issue #162 closure are not inferred.

## Compression-to-index retained-authority correction, UNRUN

The second inspected config batch records 9 passed, 1 failed, 0 ignored,
120.96 seconds. All four original REDs now pass. The remaining lifecycle control
passed field retirement/repair, contributor replacement/deletion and recapture,
then external config-to-class replacement returned the old four registrations
instead of the unaffected neighbor's two. No assertion was weakened.

The located production loss is core/compress::handle_compress_code_context:
authority was retained in session/persistence, but index publication used
remove_file/add_edges, dropping SemanticCoverage and retained TypeScript authority.
After explicit recapture this removed the owner from the existing hydration
retained-owner revalidation inventory. With cached discovery, later owner edits
were not revisited; the captured log shows no fresh config source read at failure.

Compression now uses the same existing fidelity-qualified
replace_semantic_projection_with_retained_authority call as provide/evaluate,
transporting the already-produced source hash, dependencies and TypeScript authority.
The legacy non-semantic fidelity branch and existing C# branch retain their prior
paths. This repairs missing propagation under the existing publication/currentness
policy; no new cache, lifecycle, acquisition owner or global discovery policy is
introduced. The lifecycle test additionally requires index authority immediately
after recapture, before any semantic query can repair it. Its existing replacement/
retirement expectations remain intact.

Full failure log and pre-correction compression/test bytes are archived under
target/issue-162/application-config67-compress-authority-failure/. The four frozen
original REDs remain byte-identical. All post-correction checks are UNRUN, including
broader affected compression/provider/route preservation and zero-warning gates.
The first logged handoff formats only the two repair files with child recursion
skipped, then compiles and runs all config, route and compatibility filters. No
agent-run tests/build/lint/Node verification is claimed. Generation remains 67
within this unfinished approved implementation checkpoint.

## Applicability integration correction, UNRUN

Inspected application-config67-issue_162_application_config_provider.log:
1 passed, 9 failed, 0 ignored, 44.61 seconds. The compile-only log is empty.
The retained proof/admission control passed. The shared-strategy case also reached
its nonempty retained projection/admission checks before later plain-config public
queries failed. The original four controls still reached their intended missing
public registration assertion. Subsequent route/compatibility filters did not run.

Source tracing located the missed default-production contract:
angular_semantic::evaluate_if_applicable admitted decorated/functional native roles,
not native ApplicationConfig object roots. The new producer hook was therefore
unreachable for ordinary undecorated config files despite a working retained
interpreter. Applicability now also nominates exact native ApplicationConfig roots
through the shared typed-root proof; field/entry membership remains independently
validated by the producer/admission path. No textual import/type heuristic or
query-only fallback was added. This is a correction within the approved generation-67
implementation, not a new owner contract or generation advance.

Pre-correction producer/evaluator bytes and failure-log hashes are archived under
target/issue-162/application-config67-applicability-failure/. Original four-test
SHA-256 is unchanged. No test assertion was weakened or removed. The correction
and all post-correction runtime checks are UNRUN; compilation of the preceding
batch does not verify this edit. An unexpectedly stalled formatter process was
stopped; no agent-run tests/builds/lints/Node execution occurred. The same logged
handoff reruns config, route preservation and compatibility after compilation;
archived failure evidence is retained before its active log is replaced.

## Approved generation-67 implementation checkpoint

The supplied Owner Approval authorizes source-owned native ApplicationConfig const
objects as ordinary declaration owners. The four unchanged REDs and isolated
registration were frozen in stash 8d75145f39b1c35629eaad3d402c2985512a99a7,
removed during production repair and restored byte-identically with the recorded
SHA-256 below. HEAD and the normal index are untouched. Six additional controls
are in a separate tracked module; original module registrations were not moved.

The existing Declaration endpoint/key is sufficient: the real config declaration
and its existing exact declaration site identify the subject. No RouteOccurrence,
synthetic object identity, new enum/schema field or global equality change is used.
Native capture now also nominates demanded directly annotated object initializers,
retaining neutral type/const/object/field evidence without assigning Angular roles.
A shared typed-provider-root helper contains the existing native annotation,
package/public-entry, declaration/hash/initializer and independent structure checks;
Routes still separately requires an array and config separately requires an object.
The moved route validation body retains its prior meaning.

Config production additionally requires an unambiguous native Value binding to
that actual declaration, exact unique providers field and container proof. It
passes the original field into the shared provider-entry interpreter with its
explicit strict structure policy. Class callers retain the previous policy, route
callers retain strict replay. Registration and canonical strategy admission replay
the same config producer, preventing membership loss from downgrading to generic
endpoint-only acceptance. View position is not admitted. Existing callable/value,
string-token, composition-cycle and sibling policies are reused.

Existing artifact, retained authority, source witnesses, session/SQLite and index
transport carry the unchanged declaration endpoints and demanded object evidence.
Semantic generation advances to 67 and both compatibility controls are updated.
There is no new durable encoding or payload field in this batch: the existing
neutral type_annotation/Structure evidence is populated for the newly admitted
root shape. Prior Angular projections cannot certify the changed acquisition/
owner meaning. Runtime compatibility remains UNRUN.

The initial integrated filter contains ten tests: four unchanged originals plus
six new controls. It batches direct/aliased/namespace/type-only anchors; fake,
foreign, untyped, mutable, competing and unsupported object-root refusals; mixed
shorthand/string/class/factory/existing/value policies, cyclic/unsupported paths
and sibling preservation; missing/duplicate/shifted root/field/type/package/native
proof and canonical endpoint downgrade refusal; complete SQLite authority and
private endpoint equality and retained-index checks before queries; two same-named
configs with shared contributors; stale contributor/token/barrel/config/package
restore refusal and unchanged-owner repair; independent field/provider/contributor/
config replacement/deletion. Root recreation uses explicit known-source capture.

This is an initial authored checkpoint, not all 51 obligations accepted by source
inspection. Further crossed/forged container and ambiguity/entry variations,
full preservation/lint and later major binary acceptance remain to reconcile.
The first handoff batches compilation, all ten config tests, the existing eleven
route tests and both compatibility filters, with complete logs. All new production
and test behavior is UNRUN; no agent-run Cargo/build/test/lint/Node/server execution
occurred. Static helper/signature/UTF-8/size and frozen-byte checks do not establish
compilation or behavioral GREEN. No P1.A1 or broader application closure is claimed.

## Observed operator batch

The inspected log records 0 passed, 4 failed, 0 ignored, 19.55 seconds. Every
case passed native Component registration and exact native config-type occurrence
prerequisites, then returned zero config registrations instead of two. Later
assertions remain unclassified. Exact bytes/hashes are archived under
target/issue-162/application-config-provider-red-evidence/. See the
[bounded decision](ISSUE_162_APPLICATION_CONFIG_PROVIDER_DECISION.md); no production
repair has been made.

## Question and slice exit

Which canonical provider tokens are declared by this exact native ApplicationConfig
const declaration? The candidate subject is the actual configuration declaration,
not an application, bootstrap invocation, Component or effective injector.

This is the next bounded P1 owner question after verified route membership. P3's
actual bootstrap/config selection, distinct applications and injector hierarchy
remain unresolved. Recording a configuration does not prove that an application
uses it. This step does not redefine application-wide closure as a local answer.

The intended exit would admit named source-owned const objects with an explicit
native @angular/core ApplicationConfig annotation, exact providers-field and
terminal membership through the existing interpreter, canonical declaration/token
identity, durable/currentness and independent replacement/retirement. Any extension
requires explicit owner approval after the behavioral evidence is classified.

## Existing production boundary

| Boundary | Located source | Finding |
| --- | --- | --- |
| Native type evidence | typescript_resolution/capture.rs type_reference_sites and package_exports; class_role_authority::public_entry_api | Existing Type namespace/package/public-entry proof can establish a config type without making it a runtime token. Test this exact occurrence within the actual config declaration. |
| Demanded root nomination | typescript_resolution/declaration_facets.rs capture | Decorated classes, typed callable declarations and typed array initializers are roots; typed object initializers are not nominated by that source-owned root loop. A config reference demanded elsewhere is a different acquisition path, not proof of general configuration acquisition. |
| Neutral initializer capture | DeclarationFacet initializer/type_annotation and Demand::expression | Existing const initializer, object field, source/hash, demanded structure and native reference machinery can carry the bounded object once actually demanded. Reuse is a candidate, not accepted config authority. |
| Provider owners | provider_position::Position::owner_type | Existing policy proves decorated Component/Directive/Module class owners. A config variable cannot pass this class-role gate and must not be relabeled as a class. |
| Additional route owner | route_providers::root/project | Generation 66 admits native Router Routes const arrays and exact route-object occurrences. It deliberately cannot authorize an ApplicationConfig object. |
| Default producer | layers/meta/angular_semantic::extract_non_testing_edges | Current class/route provider projections do not nominate configuration declaration owners. Adding a query-only answer would bypass missing acquisition/production proof. |
| Candidate identity | layers/meta/canonical_endpoint.rs Declaration | Unlike route objects, a named config already has a real DeclarationId. Audit exact site/field evidence and existing endpoint keys before considering any new carrier. A fabricated object declaration or route-occurrence carrier is not appropriate. |
| Public answer | workspace_query forward_edges with typed entity name and canonical edge transport | Candidate lower-bound ApplicationConfig registration can reuse RegistersProvider if approved; the additional public owner kind/meaning still needs an explicit decision. No active application/config join follows. |
| Bootstrap/application integration | Targeted Rust search across src for bootstrapApplication/bootstrapModule/ApplicationConfig/mergeApplicationConfig/EnvironmentProviders | No corresponding producer/reference matched before these investigation tests were authored. This is a bounded source-search result, not evidence of complete absence in other languages/assets or an executed bootstrap oracle. |

Clean-CTX CBM reports degraded; requested context items succeeded as skeletons.
Targeted native reads supplied the bodies used for this trace. No alternative
resolver, registry, evaluator or independent lifecycle is justified.

## Four tracked controls and evidence discipline

src/tests/mcp/angular_application_config_provider_issue_162.rs covers direct arrays,
local const/alias nested spread, imported barrel and imported namespace composition.
Each requires an exact native Component role and two existing provider-token facts
with distinct same-named token files first. It then requires exact native
ApplicationConfig Type/public-entry authority at an occurrence within the actual
source config variable declaration before the intended two-registration assertion.

The tests ask forward_edges for ApplicationConfig/appConfig. Later provenance,
endpoint identity and repeated-query assertions remain unclassified until reached.
These tests describe a proposed answer; they do not implement or approve the new
public owner/semantic authority. Compilation failure or an earlier prerequisite
failure is not valid omission RED evidence. All four are UNRUN.

## Proposed integrated obligations after observation/approval

Batch native aliased/type-only/namespace type anchors; fake/foreign/ambiguous and
untyped/mutable objects; direct/local/imported provider paths and shared strategies;
multiple same-named configs, empty config, duplicate/competing fields and unsupported
object/config merges; forged/crossed/missing root/field/source/hash/reference proof;
complete SQLite and retained-index restoration before queries; contributor/token/
config/package mutation and unchanged-owner repair; independent config/provider/
contributor/root retirement; all admitted decorated/route owner preservation and
compatibility. Recreated external roots use the existing explicit acquisition
boundary unless a separate discovery contract is demonstrated and approved.

Keep bootstrapApplication/bootstrapModule, inline bootstrap options,
mergeApplicationConfig, imported config selection, EnvironmentProviders-producing
calls, application activation, effective injector scope and TestBed separate.
These required broader P1/P3 questions stay open; they are not exclusions from
#162. No Node run is requested for this investigation. Operator owns runtime;
binary work is batched at a later major checkpoint and final field remains last.
