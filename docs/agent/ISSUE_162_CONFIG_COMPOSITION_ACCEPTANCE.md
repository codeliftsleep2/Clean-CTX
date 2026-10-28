# Issue #162: named config composition acceptance map

Agent: Agent MaxHeadRoom
Status: generation-70 focused 43 executions GREEN; supplemental/preservation UNRUN;
binary acceptance pending. Operator-reported Clippy GREEN after import deletion.

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

## Canonical question and consumer review

Question: which exact native named configs are written inputs to this exact native
typed const merge result, and does a native bootstrap occurrence reference it?
Facts: IncludesApplicationConfig(result,input) with explicit public pair-level
deduplication, and independent UsesApplicationConfig(call,result). Exact native
result/type/initializer/API/argument/input proof is required. Existing demanded
facets retain every written argument index/span. Public workspace_query forward
ApplicationConfig/appConfig and reverse ApplicationConfig/leafConfig compare exact
result/input files; bootstrap queries preserve their occurrence subject. Answers
remain structural lower bounds, with no provider union or runtime claims.

Producer/admission: config_composition replays typed const result, native initializer
and leaf input proof. Relation evidence refuses missing retained proof and generic
endpoint downgrade. Bootstrap config admission separately proves result role.
Existing source inventory/capture, source-owned snapshots, SQLite, retained index
and publication carry the proof; no new independent lifecycle. The public binary
path remains unverified until its later integrated checkpoint.

R: nine unchanged angular_config_composition_issue_162.rs controls.
C: four angular_config_composition_controls_issue_162.rs integrated controls.
E: four new angular_config_composition_exit_issue_162.rs supplemental controls.
All paths are tracked under src/tests/mcp/. E is authored, UNRUN.

## Approval crosswalk

| Item | Obligation | Evidence / state |
| --- | --- | --- |
| 1 | direct native merge composition | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 2 | alias native merge composition | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 3 | barrel native merge composition | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 4 | namespace native merge composition | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 5 | direct merged-result bootstrap reference | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 6 | alias merged-result bootstrap reference | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 7 | barrel merged-result bootstrap reference | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 8 | namespace merged-result bootstrap reference | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 9 | repeated identical input | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 10 | two distinct inputs | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 11 | same-named distinct inputs | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 12 | same-named distinct results | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 13 | independent config graphs | R exact two-graph route/reference and repeated-input prerequisites GREEN |
| 14 | standalone cold result discovery without bootstrap demand | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 15 | exact const result requirement | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 16 | exact native `ApplicationConfig` annotation | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 17 | exact initializer association | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 18 | foreign merge API refusal | C foreign API GREEN; E additional local/type-only/ambiguous routes UNRUN |
| 19 | local/shadowed merge API refusal | E explicit API/type/input/refusal matrix UNRUN |
| 20 | type-only/ineligible merge API refusal | E explicit API/type/input/refusal matrix UNRUN |
| 21 | unresolved merge API refusal | E explicit API/type/input/refusal matrix UNRUN |
| 22 | ambiguous merge API refusal | E explicit API/type/input/refusal matrix UNRUN |
| 23 | wrong/foreign `ApplicationConfig` type refusal | E explicit API/type/input/refusal matrix UNRUN |
| 24 | mutable/non-const result refusal under this contract | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 25 | missing initializer refusal | E explicit API/type/input/refusal matrix UNRUN |
| 26 | non-call initializer outside established direct-object behavior | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 27 | spread input refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 28 | inline merge input refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 29 | unresolved input refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 30 | ambiguous input refusal | E explicit API/type/input/refusal matrix UNRUN |
| 31 | non-ApplicationConfig input refusal | E explicit API/type/input/refusal matrix UNRUN |
| 32 | borrowed valid result/input endpoint refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 33 | forged initializer association refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 34 | wrong argument-position refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 35 | shifted/crossed source/hash/span refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 36 | malformed/competing retained proof refusal | C initial proof/serde GREEN; E duplicate/crossed proof and repeated-query expansion UNRUN |
| 37 | valid sibling preservation with unsupported sibling | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 38 | repeated-input one-support removal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 39 | repeated-input final-support removal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 40 | exact pair-level public deduplication | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 41 | private retention of both repeated argument occurrences | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 42 | exact forward/reverse result/input file multisets | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 43 | repeated-query stability | C initial proof/serde GREEN; E duplicate/crossed proof and repeated-query expansion UNRUN |
| 44 | complete retained payload/endpoints | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 45 | SQLite/index restoration before public queries | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 46 | stale restored composition refusal | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 47 | unchanged-result input mutation/repair | C standalone/refusal/proof/pair-support/SQLite controls GREEN |
| 48 | unchanged-result API mutation/repair | E API/barrel/package/config stale restore and unchanged-result repair UNRUN |
| 49 | barrel/package/config mutation/repair | E API/barrel/package/config stale restore and unchanged-result repair UNRUN |
| 50 | independent input retirement | C support retirement GREEN; E independent input/result/caller retirement UNRUN |
| 51 | independent result retirement | C support retirement GREEN; E independent input/result/caller retirement UNRUN |
| 52 | independent bootstrap caller retirement | C support retirement GREEN; E independent input/result/caller retirement UNRUN |
| 53 | direct named config provider preservation | Focused native config provider preservation 10 GREEN |
| 54 | inline bootstrap provider preservation | Pending preservation script selects existing inline owner family; UNRUN at generation 70 |
| 55 | existing `DeclaresBootstrapRoot` preservation | Focused eight bootstrap controls GREEN; original reference preservation queued |
| 56 | existing named `UsesApplicationConfig` preservation | Focused eight bootstrap controls GREEN; original reference preservation queued |
| 57 | Component/Directive/NgModule provider preservation | Existing provider/module/route preservation queued, UNRUN at generation 70 |
| 58 | route-provider preservation | Existing provider/module/route preservation queued, UNRUN at generation 70 |
| 59 | no provider flattening from input to result | Post-boundary original no-flattening and C public checks GREEN |
| 60 | no provider flattening from result to bootstrap | Post-boundary original no-flattening and C public checks GREEN |
| 61 | semantic/durable compatibility as applicable | Focused template/NgRx generation compatibility and C serde/SQLite GREEN |
| 62 | warning-free focused preservation | 43 focused passes; Clippy operator-reported GREEN after unused import deletion; new-test lint/preservation pending |
| 63 | rebuilt-binary acceptance at the planned integrated checkpoint | PASS: 296 composition checks; 205 inline and 304 config preservation checks; same binary/input hashes |

## Next integrated checkpoint

run-config-composition70-preservation.ps1 batches only four new E controls and
pending native/provider/bootstrap/route/durable preservation, then formats/lints
the new test code. It does not rerun the 43 focused checks. Operator owns execution.
After receipts pass, reconcile every row and author one major rebuilt-binary suite
plus affected preservation. Full repository/encoding/size and final Claude field
gates remain separate. AQ14/P3/P1/W04/#162 remain open.
