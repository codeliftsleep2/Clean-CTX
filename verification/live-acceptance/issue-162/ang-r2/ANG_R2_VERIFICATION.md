# ANG-R2 Phase A evidence and verification plan

Agent: **Agent MaxHeadRoom**. Current proposal evidence, not ANG-R2 acceptance.

## Current repository checks

Verified clean entry branch and HEAD, origin URL and exact remote branch SHA
`2aa988868202bb39307d2d256bab113c9e61b219`. The ANG-R1 commit contains all eight
required artifacts; their complete contents were read. Semantic generation remains
78, markers 6. No intervening commit/source/authority change was found.

The seven authorized ANG-R1 tracked passes and their hashes/durations are preserved
in ANG_R1_VERIFICATION.md. No rerun was made merely to restate them. Their permission
does not authorize ANG-R2 execution. No new capability test can pass before its
contract and implementation exist; no new test or production code is authored here.

## Actual knowledge workflow

The host still does not advertise pcg_knowledge_v1. Used the existing production
stdio client, actual workspace profile/database and isolated authoring launch
directory, with explicit workspaceRoot on every request. Same binary/profile
hashes as ANG-R1; no new binary build or host integration repair.

| Receipt under target/ang-r2 | Actual outcome |
| --- | --- |
| discovery-01 | `resolver registration`: 34 units, ready/current/complete; `canonical relationship`: 37 units, ready/current/complete. Next `source-owned persistence` request returned -32603 pcg/document_database; fourth planned search was not executed. Driver failed despite child process exit 0. |
| contexts-01 | Corrected FTS `"source-owned" AND persistence`: 19 units; `relationship evidence`: 46 units. Eight exact contexts all ready/current/complete; driver/process exit 0. |
| acquire-01 | Four explicit batches, 94 selected component references from the 30 unique context identities, including qualification delivery; all ready/current/complete, driver/process exit 0. |
| query-boundary-01 | Additional AUTH.TOOL_WORKFLOW closure and resolver context; both ready/current/complete, driver/process exit 0. |
| query-acquire-01 | Explicit returned component acquisition for the tooling closure, including mandatory qualifiers; driver/process exit 0. |

The raw failed request/response/stderr and receipt are preserved. Source inspection
of knowledge_query.rs shows the caller's text is forwarded as an FTS MATCH expression;
store.rs maps SQLite errors to document_database. The unquoted hyphen is consistent
with an FTS syntax failure; the corrected expression succeeds. The receipt does not
expose an underlying SQLite error, so physical database corruption/locking is not
diagnosed. No knowledge implementation repair, SQL edit or concealed fallback ran.

Successful responses retain generation
`4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3`,
catalog coverage 218/218 identities, and no reported missing catalog identities.
That coverage is the admitted catalog, not every potentially applicable repository
statement or a proof that acquired prose was comprehended.

Exact contexts: AUTH.ANGULAR_RESOLVER, ANGULAR_CANONICAL_RELATIONS,
ANGULAR_FOUNDATION, ANGULAR_PUBLIC_ENTRY, TS_RETAINED_EVIDENCE, QUESTION_MATRIX,
FINAL_VERIFICATION, PERF-004-DOCTRINE, plus TOOL_WORKFLOW. Resolver context reaches
retained TS -> foundation/source/currentness constraints and guard preservation.
Canonical relations reach retained TS and WSC-012. Performance reaches bounded-work,
canonical/source/dependency/projection and final verification. Tooling reaches shared
workspace scope, MCP delivery invariants and optional-provider authority separation.

Statements, constraints, scope, rationale, verification obligations and mandatory
qualifications were explicitly acquired from returned revision-pinned references.
Applicable source sections were inspected alongside code. Structural references,
acquisition and comprehension/compliance are separate claims. Reused ANG-R1 source
decisions are checked against current pins; no historical PASS is relabeled current.

The acquired foundation fixes source-owned identity before projection, forbids
invented canonical names and requires persistence/consumer reachability. Canonical
relations forbid silently changing old selectors/responses. WSC-004 requires the
single shared WorkspaceScope and asserting provenance; query-specific root parsing
or a new secondary index is prohibited. These constraints drive the collection,
explicit query and shared-owner design. They do not authorize it automatically.

## Preserved historical collision witness

`issue_162_resolver_semantic_key_only_public_collision` first proves all retained
mapping joins, then edits `data0` to `other` and compares sorted existing ResolvedBy
JSON rows in both directions. It intentionally expects equality. It also explicitly
asserts the source hash changed. The test documents the old public membership
boundary; the proposed separate operation lets this test remain valid unchanged.
Do not turn its equality assertion into a new expected inequality and call that
unchanged RED/GREEN evidence.

## Proposed RED and GREEN sequence — not executed

After exact contract acceptance and required B11 admission, author tracked tests
under src/tests/mcp and register through existing source modules. Invoke the actual
registered workspace_query handler with JSON for the new operation; do not require
unimplemented Rust carrier types merely to compile the baseline witness.

Before asserting missing capability, establish nonempty native route ownership,
retained mapping/key/runtime declaration proof and existing correct ResolvedBy
membership. The new public query must then fail its approved required registrations
assertion on the real baseline, with the actual response/error retained. Unknown
new operation is a behavioral missing-capability boundary, not a compiler RED.
If a prerequisite fails first, classify that failure and do not claim the intended
registration RED was reached.

Preserve the exact valid RED source and historical collision test before repair.
Only the approved implementation may make that same acceptance witness GREEN.
Then add/execute the full contract witnesses and affected preservation, not a
succession of broad speculative campaigns.

## Required tracked acceptance matrix

| Group | Required assertions |
| --- | --- |
| P01 ordinary and repeated target | One mapping; two distinct keys -> one target; two keys -> distinct targets; records exact and old membership correctly deduplicated. |
| P02 occurrence/identity | Same keys on distinct equal-path/empty-path route objects; same-name declarations in different sources/scopes; compare exact route/root/target coordinates rather than names. |
| P03 reference forms | Direct, alias, barrel, namespace and local forms, with independent selected-declaration proof and actual public forward/reverse retrieval. |
| P04 key semantics/provenance | Established identifier/quoted-string spellings and decoded escape equality; retain original source/hash/ranges; no general Unicode/property evaluator claim. |
| P05 cold/warm/query scope | New state's cold forward/reverse reach supported aliases; warm equivalent answers; configured additional roots; asserting withinPath; no unrelated workspace leakage. No manual pre-seeding substituted for cold acquisition. |
| P06 integration | Default combined MetaLayerPass -> artifact -> compatible source-owned publication -> index -> registered query; no helper-only/custom-pipeline substitute. |
| P07 durable/replay | Inspect retained complete records in snapshot/SQLite/index before public reacquisition; restart and warm replay; compare exact records and old public edge JSON. |
| P08 replacement/retirement | Equal-length key-only edit, target-only external change, mapping/field/Object/route/root removal, deletion/recreation/relocation, refresh and repair. Old key cannot survive while membership rendering is unchanged. |
| P09 sibling preservation | Unique mappings amid refused named siblings; all five guard kinds, eager/lazy/children/providers and config/bootstrap/marker behavior independent. |
| N01 ownership/container | Missing/foreign/ambiguous native owner, duplicate resolve fields, unknown computed/spread/shorthand structure, unsupported call/inline/nested value; no registration via legacy membership. |
| N02 duplicate/key/value | Duplicate decoded keys, including unsupported duplicate value; escaped spelling collision; empty/unsupported key bounds; unresolved/ambiguous/type-only runtime targets. |
| N03 proof forgery | Missing carrier/mapping proof; crossed source/hash/route/field/Object/pair/key/value/target joins; valid target elsewhere cannot repair membership; extra/missing stored registration records rejected. |
| N04 currentness/compatibility | Old/missing/unknown-format payload, changed target/barrel/package/config with unchanged owner, stale restore, failed acquisition/publication and final-generation invalidation. No fabricated success/empty answer. |
| N05 API/batch | Invalid direction/selector/types/root, arrays/opposite selector, empty-path forward supported without changing old name validation; batch identity includes all new fields; failure isolation/final generation preserved. |
| N06 meaning | Noncallable runtime written targets retain approved written semantics; no executable eligibility, navigation, DI/provider selection or value-success assertion. |
| W01 bounded work | Shared producer/admission projection, no per-registration full-set replay, source-owner query read once, full cold/warm and serialization/replay costs; existing applicable AF gate retained. |

## Runtime execution authorization and production checkpoint

Before **each** long-running invocation, present the exact command, purpose,
selected tests, source/implementation state and RED/GREEN/preservation category.
Await operator authorization for that run or exact batch; do not inherit Phase B
permission or the previous mission's runs. No proposed filter is claimed registered
or nonempty before its tests actually exist.

At a stable approved implementation, request one consolidated production build and
issue-162 binary/MCP scenario using the existing transport/process/shutdown policy.
Record executable/source/config/fixture hashes, all named checks, processes and
errors. Demonstrate actual new query F/R records, two-keys/one-target, exact route
ownership/provenance, old membership, key edit, removal, refusal, replay and siblings.
Report binary observations separately from tracked Rust/CI results. No mocks of
native identity/publication, transport-only substitute, general Angular certification
or full local repository gate. Claude remains the final field gate only if needed.

## Phase A static closeout

Only the six proposal Markdown files are created. Exact byte/source-pin and local
UTF-8/no-BOM/whitespace checks are recorded below; they are not Rust tests, source
encoding CI or production acceptance. No production/test/authority/registry edit,
generation advance, authorization submission, commit or push is claimed in Phase A.

<!-- ANG_R2_PHASE_A_RECEIPTS -->

Static source audit passed for 37 unique returned knowledge identities.
Verified 110 selected-reference acquisitions, 202 delivered component occurrences,
127 qualification occurrences and 25 distinct source files against exact
source hashes, UTF-8 byte intervals, selected-content hashes and returned text.
Counts include repeated qualifier delivery; they are not unique propositions,
comprehension results, tracked tests or certification. Final resolver context
unit-revision and typed-edge sets match the earlier Phase A resolver context.

All six proposal files pass strict UTF-8/no-BOM, local trailing-whitespace and
EOF checks; each is below 800 lines. No repository encoding/size script, Cargo,
formatter, test executable, Clippy, build, AF campaign or Angular production
acceptance driver ran in ANG-R2 Phase A. No code/authority/registry/ANG-R1 file
changed, no staged changes exist and HEAD remains the verified delivered ANG-R1
commit. Only the six local proposal Markdown files are untracked.

| Production receipt | SHA-256 |
| --- | --- |
| target/ang-r2/discovery-01/receipt.json | `41c5e7efd9c67b84e5503242e03341b374912787a9e1336e8b461aecda3ccd49` |
| target/ang-r2/contexts-01/receipt.json | `49daa543f6cad0cd5ead78fc484fb9ca76f501c26e120819367f3d3c045e5c5b` |
| target/ang-r2/acquire-01/receipt.json | `f329fa46da74a1adec1b824d201e3e9f665fa7891f01095d6b05a27f9d26487f` |
| target/ang-r2/query-boundary-01/receipt.json | `1debc0ff329b0b0c231f7f595418dc27a82ca249d4e55183135e31143f553a91` |
| target/ang-r2/query-acquire-01/receipt.json | `c5f8491ad47e541e98bc80c65e0b3619f58a77412deff0fd9340c0a4ffaaaef7` |

The failed discovery receipt remains a failed driver result, with two completed
searches, not a successful four-search campaign. Subsequent corrected searches,
contexts and explicit acquisition are separate successful receipts.

### Relevant current knowledge revision inventory

These exact descriptors accompany the source inspection; they do not make the
Phase A proposal controlling. Registry/protected-source predecessors are preserved.

| Identity | Path | Revision |
| --- | --- | --- |
| ARCH-001 | docs/invariants/canonical-ir.md | `d14d4a2e520fb6207366dcc394e806c6867cdd66465f7450fe919d81a9b0565b` |
| ARCH-002 | docs/invariants/canonical-ir.md | `d5b405c6abde755867408c2303c531520e6a68345ce06a7cd82d56eea627ab24` |
| ARCH-003 | docs/invariants/projection-fidelity.md | `7c3879d7d747dd90cfe6dfa340d65d633d870991046d9a92954b61ef61385be4` |
| ARCH-004 | docs/invariants/completion-governance.md | `54e7e2960b529fb9e3cda6df3ba7dd28965a9c998627ca59d3b981de5a011081` |
| AUTH-001 | docs/invariants/source-authority.md | `d4ad376f0716c73f30ef39c8758e8683c700e16807e10dd5a3a53997e65ff8d6` |
| AUTH.ANGULAR_CANONICAL_RELATIONS | docs/architecture/contracts/angular-canonical-relations.md | `57d4e32a32f56b9e9fb182afe0821172c5cfc0b498d62d9044636fd4c8689adb` |
| AUTH.ANGULAR_CAN_ACTIVATE | docs/architecture/contracts/angular-can-activate.md | `e7343fb610c74ba58bf12d3ce305d4029206be5248935189c23be0d614e4b391` |
| AUTH.ANGULAR_FOUNDATION | docs/agent/ISSUE_162_FOUNDATION_DESIGN_FREEZE.md | `e39c065d0de8897b36be94fbbbffa5fdb55d749bfb9d0649d2034cf571ee90a6` |
| AUTH.ANGULAR_GUARD_KINDS | docs/architecture/contracts/angular-guard-kinds.md | `1590b858bccc5b7b9ec3b705170ce31451f327f7d97b106c067ad05c80aae6c5` |
| AUTH.ANGULAR_PUBLIC_ENTRY | docs/architecture/contracts/angular-public-entry.md | `007b463769ea454ae3971f36a78e09b56fbfb4bc730f2f5349bf8e3ae154d706` |
| AUTH.ANGULAR_RESOLVER | docs/architecture/contracts/angular-resolver.md | `6c796231ed5b92ed63a6bb4f8ed9f2bb85b2cfb03335d08b7507aaa958af8f1e` |
| AUTH.CANONICAL_IR | docs/architecture/CANONICAL_IR_AUTHORITY.md | `d5c416bcec21967d3c734956d48fd6009a7b70c777aa613e3c2492ea778c592e` |
| AUTH.CORE_OPS | docs/architecture/CORE_OP_CONTRACT_MATRIX.md | `2ef6bc36544649862cba0ac6bf9282c402b2c5091beb0184b6281b3512965ee5` |
| AUTH.FINAL_VERIFICATION | docs/agent/verification.md | `9fb9b7fe4a0bde28f341ef0686d4eb1714cc1fed25b49f5ba16aab9e64188005` |
| AUTH.PRESENTATION_VNEXT | docs/architecture/contracts/presentation-vnext.md | `28117426f160e285e12fe73dafc1a4dc6e7acc776c256ce2b5aacf523cd4a489` |
| AUTH.QUESTION_MATRIX | docs/Clean-CTX_Canonical_Question_Matrix_Standard.md | `91b99cd0d35bbc6191da1eede8e09d3acf6ee23f37769e21c1717cfd0dc8771d` |
| AUTH.TOKEN_ECONOMICS | docs/architecture/contracts/token-economics.md | `6804215b69077e023ed57d4158458024ebccbe7216b409d53c0fa5815ea82811` |
| AUTH.TOOL_WORKFLOW | docs/agent/tooling.md | `9cd976e168473b01c1fe256cf3fb4f86abe475a45889db6e964e8a20b12d8765` |
| AUTH.TS_RETAINED_EVIDENCE | docs/agent/ISSUE_162_RETAINED_AUTHORITY_CONTRACT.md | `f50b18412798d5ed658a5826c70882d6d16dfaa1a5b9de531b50089d9c45d64c` |
| CONTENT-001 | docs/invariants/projection-fidelity.md | `ae5b545690a451864785d7773a7ac405b3cebec6f58b69fd28218297a976725e` |
| CTX-001 | docs/invariants/projection-fidelity.md | `f051789937175961c560fb34fa5ea87694b17d2e2c0d3f964b2955299074cfa7` |
| DOC-228 | docs/agent/ISSUE_162_GENERATION78_BOUNDED_ACCEPTANCE_PROPOSAL.md | `db41d9b923eb561bca505fd01f357fc11a88042d87f3d93be39baaad6ab6813e` |
| INV.CBM_OPTIONAL_001 | docs/invariants/cbm.md | `3164b0c2604baa05bdcc053fcdc4b8812eaf749c01f7a923850e68c77ce4329c` |
| INV.MECHANISM_REUSE_001 | docs/invariants/completion-governance.md | `e60c28da47bec7601764e7f84c346cd752ec22d670f292b0db64c45726a945e5` |
| IRPAT-001 | docs/invariants/identity-occurrences.md | `225c494a89d5a0d59e8905adc419305e031809a720695d391916fba13020b7b6` |
| MCP-001 | docs/invariants/framework-and-tooling.md | `cbb3eb7f8e69c3f5a713d67a2f9457020a2ba78690b677640a1e643393776207` |
| MCP-002 | docs/invariants/framework-and-tooling.md | `b6cd1728a7e478e654514be1ae346c4995451f134724c1be3bde90ce259dbe3a` |
| MCP-003 | docs/invariants/framework-and-tooling.md | `ff9272fa05c2edc1f57a0a12b6a48d04d4e3ca3a5e7c5f633f0dc3c137719dc0` |
| PERF-004-DOCTRINE | docs/architecture/contracts/performance-doctrine.md | `f834f5ffd4456954aa30f07c1d6b96f537dbebaeacc581e26b39edf7d9d5843c` |
| PERF-005-BOUNDED | docs/architecture/contracts/performance-doctrine.md | `2463d292008a05ab6c3989066557b6090a91cb7eea3149cb2ccc041fa6eaf880` |
| PIPELINE-001 | docs/invariants/canonical-ir.md | `b226a521c762f6d63bc83b6659aed1170c59aaf8e048b87d6f7cd6d2a9f739d7` |
| PROJ-001 | docs/invariants/projection-fidelity.md | `ca3fe55ee81b442b2049e062970924f144e95517bd1bfb8e1c99f37ea543bc4a` |
| REFINE-001 | docs/invariants/source-authority.md | `f15543848e9de6369ca426ca73486d8944afc8b04c844c9effdd6b2dd3052ad5` |
| SEMDEP-001 | docs/invariants/source-authority.md | `2aadc421ab225f6ba75e6159cfb0065ab339291376ec7e49679becbc834cf9f7` |
| WSC-004 | docs/invariants/workspace-query.md | `7fe8e8c2e67611a612d50ad035307c8336db4f5a1003b982d600b3b886e31efa` |
| WSC-012 | docs/invariants/identity-occurrences.md | `abdd293159cf98354b65fae2fba04344d92666774ff966cb301d0bf2982d2e47` |
| WSC-014 | docs/invariants/source-authority.md | `57aaa89a4f79c6f76d2ac6fa5f2c789617dc5d484b97faa65a7adfb44dde6e30` |

## Owner disposition 01 continuation — categories remain separate

Architecture accepted in principle; exact omission attribution, cold nomination,
compatibility and three controlling authority candidates are now prepared.
No production/test code or generation changed; no test/build/guard/describe/runtime
was executed for this disposition. The original failed FTS receipt and corrected
success remain separate and unchanged. No database corruption/locking diagnosis,
knowledge repair, host integration repair, CBM retry or P0 optimization occurred.

Additional required witnesses: fresh independent cold forward and reverse direct,
alias, barrel, namespace and local cases without index seeding or warm-up; selected
route versus selected resolver omission proof; reverse unresolved candidate-source
diagnostics with no invented target/relevance; no matching bucket plus unrelated
unsupported source; withinPath filtering for both diagnostic arrays; current empty
versus missing/old/unknown collection and acquisition failure; schema/text/batch parity.
Keep the existing key-only membership collision unchanged and preserve the new
behavioral RED unchanged for GREEN. All remain future work, not executed passes.

The bounded operator descriptor command is handed off in ANG_R2_GOVERNANCE_HANDOFF.md.
No exact Rust candidate digest, B11 grant or publication is claimed. Full gates are
CI-owned. Only fast static source/byte/diff inspections were performed here.
The earlier six-file audit is a dated Phase A receipt; it must not be rerun as if
the subsequently prepared authority changes were absent.

Bounded continuation checks passed: strict UTF-8/no BOM, final newlines/trailing
whitespace, candidate hash pins, HEAD/index preservation, paired unique PCG markers,
addition-only changes in the three governing files, unchanged production/test tree,
and git diff --check. All 37 retained preflight unit source pins were compared:
exactly the three deliberate authority candidates differ. PowerShell's parser
accepted the descriptor-capture script without executing it. This is static syntax
inspection, not a runtime/guard/test pass. The generated audit is
target/ang-r2/owner-disposition-static-audit.json. Static predecessor hashes are Git
blob bytes; candidate hashes are actual working-file bytes, with checkout newline
differences explicitly labeled in the manifest. No Rust governance digest is inferred.

## Transport batch received; current integrated GREEN handed off

Operator capture: target/ang-r2/registration-transport-20261115-005055-616.
Two transport tests passed, exit 0, 136.2871955 seconds enclosing / 24.28 seconds
test time. Six existing resolver78 tests passed, exit 0, 348.1727118 seconds
enclosing / 346.97 seconds test time. No warning/compiler-error marker was found.
These results precede the subsequent admission/index/query/generation integration
and are preserved as stage evidence, not current GREEN or verified full resolution.

The frozen public witness is restored unchanged at SHA-256
8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc.
Original stash a4bd12ca4df42c4bf34063bc5139fa2d463f76fa remains intact. The original
membership collision source remains unmodified. Current generation-79 source,
seventeen focused new tests and old membership preservation remain UNRUN.

The exact next three-command batch is documented in ANG_R2_IMPLEMENTATION.md and
packaged by invoke-registration-green.ps1 (expected selections 17 / 6 / 1).
It captures changed/new source hashes before and after, witness/log hashes, counts,
durations and exits; it stops on failure or warnings. No agent Cargo/test/build/
runtime/format/Clippy/guard invocation was made. Production acceptance and remaining
unestablished lifecycle obligations are separately recorded in their named reports.

Current bounded static inspection passed: strict UTF-8/no BOM, whitespace/newlines,
all active Rust/PowerShell files below the 615-line ceiling, the exact restored
public witness, unchanged original collision source, unchanged protected candidate
pins, and no MCP dependency in workspace code. The new suite enumerates 17 tests.
PowerShell parsed the GREEN wrapper without executing it; git diff --check passed.
Generated detail: target/ang-r2/integration-static-audit.json. These are source
checks only, not compiler, format, Clippy, encoding-guard, CI or current test results.

### Integrated operator run and setup repair (2026-11-15)

Agent MaxHeadRoom reviewed the preserved operator run
`target/ang-r2/registration-green-20261115-090936-592`:
17 selected tests, 11 passed, 6 failed, exit 101; enclosing duration
596.0441093 seconds, test duration 363.80 seconds. No compiler warnings were
reported. Both unchanged frozen public witnesses passed, establishing their
public RED-to-GREEN transition at this source state. The wrapper stopped before
the existing six membership controls and original collision test ran.

All six failures assumed cold query hydration created a session snapshot or
durable identity. Production query publication intentionally publishes index
facts only. The lifecycle test setup now explicitly calls the existing
`compress_code_context` operation after the cold query and before snapshot/save
inspection. Performance acquisition occurs after cold/warm measurements and work
counter assertions. Production code, assertions and frozen witnesses are unchanged.
This correction is implemented but its six affected tests remain UNRUN.

Next operator command, from the repository root:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-registration-green.ps1 -RepairOnly
```

This runs the six repaired tests individually (one expected test each), followed
by the unrun membership controls (six) and original collision test (one).
The eleven unchanged passing tests are not repeated. The script retains source
and log hashes, counts, durations and exits, and stops on failure or warnings.
Overall ANG-R2 acceptance, production integration and remaining lifecycle gates
are not established by this partial run or by the static repair checks.

## Verified repair capture (2026-11-15)

Agent MaxHeadRoom inspected `target/ang-r2/registration-green-20261115-100307-289`. All eight invocations exited zero: six repaired tests (one each), existing membership controls (six), original unchanged collision (one). Total: 13 passed, zero failed; no compiler warnings. Receipt log hashes, selected counts and before/after source manifests match. Only the lifecycle test setup file differs from the prior 11-pass/6-fail capture; current source hashes match the repair capture. Thus all 17 new tests have passing evidence across the two captures, with the two unchanged frozen public RED witnesses passing in the first capture. This is focused operator Rust evidence, not a single rerun of all 17, CI, production stdio or field acceptance.

Next operator-owned batch is `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-registration-build.ps1`: scoped all-feature library/binary/test Clippy, all-feature production binary build, UTF-8 guard and active-file guard. It captures source/log/binary hashes and stops on failure. It starts no runtime. Full repository gate remains CI-owned.

## Scoped Clippy failure and correction (2026-11-15)

Agent MaxHeadRoom verified the preserved capture `target/ang-r2/registration-build-20261115-102915-263`: scoped all-feature library/binary/test Clippy exited 101 after 119.5449665 seconds. The batch stopped; production build and repository guards did not run. Receipt stdout/stderr hashes match.

Two diagnostics were corrected without changing runtime behavior: the old membership-only projector wrapper is test-only (all remaining callers are tests), and the existing owned registry result tuple now has the local type alias `OwnedMetaLayerResults`. No lint suppression, assertion change, authority change or lifecycle change was added. The corrected Clippy result remains UNRUN; prior focused test receipts remain historical evidence for their exact source states.

Retry the same operator batch: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-registration-build.ps1`. It reruns the failed Clippy step, then proceeds to the previously unrun production build and guards. No agent Cargo command or runtime was started.

## Verified local build checkpoint (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/registration-build-20261115-103503-868`: scoped all-feature library/binary/test Clippy passed (66.4665054 seconds), all-feature production build passed (90.0229315 seconds), strict UTF-8 guard passed (29.0891902 seconds; 2869 files, zero BOMs), active-file guard passed (4.2010105 seconds; 48 active files, 12 untouched legacy oversized files). Cargo emitted no warnings. The file-size guard's stderr contains existing Git CRLF-to-LF notices, not compiler warnings or file-size violations. All receipt/log hashes, before/after source manifests and current source pins match.

Built executable: `target/debug/clean-ctx.exe`, SHA-256 `5d00fea65d44bf7522d123e10a543b46a92842c77a1e60caee918605ec1549bd`. A successful build establishes no runtime acceptance. The earlier historical knowledge executable identity is superseded only for this build checkpoint.

The new reusable `registration-stdio.mjs` is UNRUN. It reuses the established McpClient transport and synthetic package declarations, checks the exact successful build/source pins before and after, and captures raw RPC/stderr and hashed fixtures under `target/ang-r2/registration-stdio-*`. Three sessions cover independent cold forward/reverse roots plus durable restore, key-only/target replacement, root/source retirement, owner relocation and recreation, exact source/key coordinates, refusal sibling preservation and old unqualified membership/component edges. Runtime results remain pending. This harness is operator production-boundary evidence and cannot replace tracked Rust tests or CI. It does not establish edit/delta/replay/reset or additional-root coverage beyond its explicit scenarios.

Next operator command from repository root:

```powershell
node verification/live-acceptance/issue-162/ang-r2/registration-stdio.mjs target/ang-r2/registration-build-20261115-103503-868
```

This starts the pinned executable for three local synthetic stdio sessions; it performs no build and requests no Claude environment switch. JavaScript syntax and bounded encoding/whitespace inspection passed before handoff; the agent started no runtime. Full CI and remaining acceptance obligations stay separate.

## Production stdio failure retained (2026-11-15)

Agent MaxHeadRoom reviewed `target/ang-r2/registration-stdio-2026-11-15T15-43-41-046Z`: FAIL, 181 passing checks and one failing check (182 total). Independent cold forward/reverse sessions passed. All three server processes exited zero. The restore/mutation session is incomplete because `owner-recreated/unqualified-membership` failed. Current executable and harness-input hashes match the report.

Public registration checks passed cold/warm retrieval, durable restore, key-only replacement, target remap, root retirement, owner relocation, source retirement and recreation, including exact evidence/current target hashes. After deleting moved.ts and recreating one.ts, the separate old forward_edges query returned two ResolvedBy edges plus one RouteMapsTo edge from each owner, including the deleted moved.ts. The preceding registration response correctly returned only three records from one.ts. Do not relabel the overall harness PASS or weaken its old membership assertion.

Source trace: the new operation consumes its explicit current filesystem nominated-owner set; old adjacency queries consume an indexed bucket. Existing candidate eligibility excludes absent files, so acquisition alone does not retire their prior indexed rows. This identifies the differing consumption paths, not a demonstrated pre-ANG-R2 baseline regression or an approved global lifecycle repair. The old edge query source and shared candidate eligibility are unchanged by ANG-R2.

A tracked legacy-only control is now authored in src/tests/mcp/angular_deleted_owner_membership_issue_162.rs, registered in the existing test module. It uses only old forward_edges queries through initial acquisition, relocation, deletion and recreation, never resolver_registrations. It is UNRUN. Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-deleted-owner-regression.ps1`. Preserve either result, including failure; no production code or frozen witness changed. This test-only addition does not require a new production build yet.

CBM remains degraded and this host advertises no pcg_knowledge_v1 tool; architectural graph coverage is unavailable, not repaired or inferred. Any repair that changes shared cross-query deletion/retirement policy needs the applicable owner architectural disposition. Actual production acceptance remains blocked pending this finding's resolution/disposition, with CI and other lifecycle/field obligations separate.

## Legacy-only regression compilation correction (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/deleted-owner-regression-20261115-104819-755`: exit 101, 20.9613521 seconds, zero executed tests. Two E0277 compiler errors came from using Display formatting for Vec<Value> in assertion diagnostic messages. Only those messages now use pretty Debug formatting; assertions, test sequence and production code are unchanged. Log hashes match the receipt. This failed compilation is not behavioral RED evidence and does not establish a legacy-only reproduction. The prior actual stdio failure remains preserved.

Corrected regression remains UNRUN. Retry the same focused operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-deleted-owner-regression.ps1`. No agent test/build/runtime command was started.

## Confirmed tracked RED and shared-lifecycle decision point (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/deleted-owner-regression-20261115-105355-728`: compilation succeeded without warnings; one tracked test ran and failed at the deletion assertion (line 34), exit 101. Enclosing duration 63.1784525 seconds; test duration 6.86 seconds. Receipt log hashes match. The relocation count assertion passed, but it asserted only count, not relocated provenance; do not claim relocation correctness from it. The recreation assertion was not reached.

After moving one.ts to moved.ts and deleting moved.ts, the old forward_edges query still returned one ResolvedBy edge asserted from now-absent one.ts. No resolver_registrations query occurs in this test. This establishes a current legacy-path behavioral RED independent of invoking the new public operation; it does not establish that the defect predates ANG-R2, since the producer/publication internals are the current implementation. The actual stdio capture separately established stale moved.ts membership after recreation.

Architectural decision requiring owner disposition under AGENTS.md Architectural Approval Gate:

1. Decide whether ANG-R2 includes correcting deleted asserting-source retirement at the existing shared semantic publication/index lifecycle boundary. No implementation choice or permission is inferred from the failure.
2. ANG-R2 preservation verification now fails and its tracked regression is RED. New registration output alone cannot establish mission acceptance while that failure remains unresolved.
3. A shared-boundary repair can affect forward/reverse edges and other indexed semantic consumers across query types. Persistent/session ownership, cache behavior and disappearance versus transient acquisition failure must remain explicit; no new store, global cache or broad policy is proposed.
4. Alternatives: authorize a bounded shared-boundary repair with focused deletion/recreation/sibling tests; authorize a narrower existing boundary if investigation proves equivalent ownership/currentness guarantees; or hold ANG-R2 and handle the defect separately. Holding is not a waiver, green gate or completed delivery. A resolver-only response filter is not recommended because it leaves shared lifecycle authority inconsistent.
5. Recommended: authorize investigation and the smallest correction at the existing shared lifecycle boundary, limited to confirmed deleted asserting owners, preserving source identity, supported in-memory/session sources, failure distinction, unaffected siblings and existing lifecycle semantics. Any new cross-cutting policy discovered during that work must return for a separate disposition. Preserve this tracked RED and rerun it after correction, then batch affected controls and production stdio. Do not reopen accepted P0 batching or unrelated optimizations.

No further test run or production modification is requested before this disposition. The frozen witnesses, failed receipts and admitted authority candidates remain unchanged. Host PCG coverage remains unavailable as previously recorded; no graph or baseline approval is inferred.

## Authority acquired; bounded correction implemented, UNRUN (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/retirement-authority-20261115-110001-740`: eight complete/current/ready contexts, 22 closure units, 158 acquired components, one admitted generation `5c3e27fc7f9b107e4161b25bcb0c704dd1952532dad9dc9b42b5ff1810727174`. Every acquired source hash, byte interval, component hash and decoded text matches current source. `source-review.json` records the bounded verification. These knowledge receipts are authority acquisition, not repair correctness or a protected grant.

Production correction is authored in `src/mcp/state/retirement.rs`. Before indexed query preparation, one request-local reconciliation snapshots existing semantic coverage owners and checks admitted owners for physical existence/read access. Only an explicit NotFound from owner symlink metadata classifies deletion. Wrong-kind paths, dangling links, permissions, acquisition errors and disappearance after the first present observation refuse current consumption without classifying deletion. A missing owner is checked again inside its normal source-ordered publication ticket. Reappearance or supersession refuses/retries. Confirmed deletion retracts owner-local index contributions/coverage, live IR/semantic snapshots, pending transitions and presentation/source caches, invalidates existing discovery completion, and marks applicable C# authority dirty. Durable rows and historical path ownership remain available under unchanged restore/replay compatibility contracts. No response filter, new persistent store, watcher, policy budget or resolver-specific workaround is added.

`PreparationContext.scope` validates scope first, memoizes reconciliation once across all items, and propagates currentness errors as internal failure rather than empty success or invalid scope. Existing `WorkspaceIndex::remove_file` preserves independent source supports and other families. Shared consumers include scoped/unscoped forward/reverse queries, declaration/identity operations, traversal and registration discovery that use the shared preparation context. This is an existing ownership-state repair before consumption, not filtering selected JSON. Currentness observations are boundary observations, not atomic filesystem immutability.

Work scope: one O(U) coverage-owner snapshot per request, one metadata/access probe per admitted owner, and one additional probe only for each missing owner's separate commit. No semantic set is reconstructed for each registration or batch item. Each committed deletion invokes the existing per-owner removal/refinement cleanup against its resulting index state; no cost-free or workspace-independent cleanup claim is made. Existing producer generations and durable formats remain unchanged by this repair because compiled semantic payload meaning is unchanged. All six supplemental tracked controls are UNRUN: sibling/multiple supports and relationship families, injected permission failure without deletion, reappearance before commit, public unknown-kind refusal, SQLite history plus older-ticket rejection, and 2/16/64-owner work counts shared across batch items. The exact original RED source remains byte-identical.

Next operator command:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-green.ps1
```

Seven focused invocations select 37 tests: frozen RED (1), shared retirement controls (6), resolver registration tests (17), old membership controls (6), original collision (1), existing historical replay compatibility (5), and existing failed-publication retry (1). This batch is not a broad/full repository campaign. It records exact source manifests/log hashes/counts/durations and stops on failure or warnings. No tests/builds/runtimes were started by the agent.

After observed focused results, the current build/stdio witness and governed repair record/admission checker remain required. Appropriate governed source findings, owner disposition and exact verification associations must be prepared for actual admission; no publication/push or protected authorization has occurred. The old successful build and failed stdio receipts predate this production correction and cannot verify it.

## Relocation failure and physical-owner binding correction (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-111039-330`: the first frozen witness compiled without warnings, ran and failed at relocation (line 31), exit 101, enclosing 81.5014067 seconds. Its legacy answer was empty rather than one relocated membership. The remaining six invocations did not run. The exact witness source/hash is unchanged; no GREEN is established. Receipt log hashes match.

The first correction retracts absent one.ts as intended, but a rename keeps the OS physical file ID. `PhysicalIdentityRegistry` still resolved discovered moved.ts back to absent one.ts; the existing candidate eligibility check then dropped that path. This is a second stale ownership binding in the approved shared lifecycle, not failed discovery or permission failure. Previous relocation-count success before retirement did not prove relocation provenance.

`PhysicalOwner::reconcile_deleted_path` now updates only the existing physical-file entry when its previous semantic owner path has explicit NotFound and a live path for that physical identity has been resolved. Current hard-link owner paths and unknown/failed metadata observations remain unchanged. An explicitly absent durable path binding is released for the next live spelling; persisted rows and historical dictionary/path evidence are retained under existing compatibility checks. It adds no owner registry, inverse index, alias-guessing or query filter. The physical ID remains lookup-only and public semantic identity remains the canonical source path. Work is a fixed number of path observations on that one existing entry, not a registry scan per candidate.

A separate seventh control checks live hard-link convergence, release of the deleted owner binding, the surviving file's exact bytes and independently recreated source ownership. Three existing platform hard-link controls are added to the batch to preserve state aliasing, provide/hydration deduplication and durable edit-refusal behavior. Both frozen witnesses remain exact.

Retry `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-green.ps1`. The updated batch has eight focused invocations / 41 selected tests (1 frozen RED + 7 shared controls + 3 hard-link controls + 17 registration + 6 old membership + 1 collision + 5 historical replay + 1 publication retry). It reruns the failed first command and then the unrun controls. No agent tests, builds or runtime were started. The corrections remain implemented but UNVERIFIED. Production rebuild/stdio and actual governed document admission remain pending.

## Recreation boundary correction after partial witness progress (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-112237-865`: the frozen witness compiled without warnings, ran and failed only at recreation (line 36), exit 101, 112.8785838 seconds enclosing / 4.98 seconds test time. Raw replies establish initial one.ts count 1, relocated moved.ts count 1, deleted-owner count 0, then recreated-owner count 0. Source and receipt log hashes match. This establishes relocation and deletion progress for that exact source state, not overall GREEN. All later invocations were skipped.

The missing transition was reappearance after a legitimate empty discovery completion. Retirement removes live index ownership but deliberately retains existing dictionary path history. Query preparation now observes those existing unindexed path bindings once per operation alongside active coverage owners. A live known unindexed path invalidates applicable discovery completion, allowing the unchanged discovery/compiler/admission path to reacquire facts. It is never published as a semantic fact merely because its path is remembered. Unavailable historical paths have no current projection to consume and provide neither facts nor deletion evidence. Active owners still refuse unknown currentness and use ordered deletion commits.

This uses the existing path dictionary, not a watcher, tombstone store, new candidate provider or general discovery redesign. `invalidate_discovery_for_edited_paths` batches the existing longest-configured-root/fallback invalidation policy once per affected root; single-path callers delegate to the same policy. The operation scope is O(U+A) for indexed owners U and existing aliases A, with one membership set and at most one source observation per consumed binding, plus deletion commit rechecks. There is no per-registration or per-batch-item complete-set replay. A separate eighth tracked control counts 2/16/64 known unindexed paths across a heterogeneous batch and asserts no semantic ownership is manufactured.

The source edit tool stalled after writing the invalidation helper; the stalled invocation was stopped, all affected files inspected, and the remaining edits completed with exact UTF-8 replacements. A CRLF whitespace finding in the newly touched path dictionary was corrected by normalizing that source file's line endings to LF, preserving Unicode/content. No Cargo command or runtime was started by the agent.

Retry the existing command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-green.ps1`. Eight focused invocations now select 42 tests (1 frozen RED + 8 controls + 3 existing hard-link controls + 17 registration + 6 membership + 1 collision + 5 replay + 1 publication retry). The exact frozen RED source and assertions remain unchanged. Implementation remains UNVERIFIED; production rebuild/stdio and governed document admission remain pending.

## Frozen retirement RED-to-GREEN; wrapper resume correction (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-113110-392`: the exact frozen legacy witness passed (1 passed, 0 failed), exit 0, no compiler warnings, enclosing 146.1044231 seconds / 13.15 seconds test time. Witness hash remains `69c27a304b59029f7aee0ed2bfd511eb47c2702c0a8a43bb22d09653368b797f`. Receipt log hashes and every captured Rust/build source pin match current files. This establishes the corresponding focused RED-to-GREEN including relocation, deletion and recreation under the unchanged assertions. It does not establish supplemental controls, production stdio, CI or full repair acceptance.

The wrapper failed after writing that successful receipt: positional Select-String arguments treated the expected test-result string as a filesystem path. Both checks now use explicit LiteralPath/Pattern arguments. A checked ResumeFrom option compares exact source manifests, HEAD, witness, command, count, exit and log hashes before reusing a prior passing receipt. Reuse is separately recorded and never presented as another execution. Prior failures cannot be reused. No production/test source or assertion was changed for this wrapper correction.

Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-green.ps1 -ResumeFrom target/ang-r2/retirement-green-20261115-113110-392`. It reuses the one passing frozen witness and runs only the seven unrun focused invocations (41 tests). The script fails closed on source drift, warnings or invalid receipts. No agent tests/builds/runtimes were started.

## Shared retirement controls verified; selector expectation corrected (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-115012-609`: the frozen witness was correctly reused from its exact passing receipt; the next invocation exited zero with 9 passed, 0 failed, no compiler warnings, 64.702886 seconds enclosing / 61.69 seconds test time. All eight new retirement controls passed plus the existing `issue_162_retirement_delete_context_retires_only_owned_proof_and_preserves_source` control selected by the same prefix. The wrapper's expected count of eight was wrong and stopped the batch after the valid passing receipt. This is a selection-expectation failure, not a tracked test failure. Receipt hashes, current source pins and the reused witness reference match. The original receipt's expected count remains eight as historical metadata; it is not rewritten.

The wrapper now expects nine and validates actual result counts against that corrected selection while retaining prior expected-count metadata explicitly in reuse records. It follows an existing hash-pinned reuse reference to the original direct receipt, then rechecks source manifest, HEAD, exact command/witness, zero exit, log hashes, warnings and actual count. No production or tracked test source changed.

Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-green.ps1 -ResumeFrom target/ang-r2/retirement-green-20261115-115012-609`. It reuses the frozen witness and all nine shared retirement passes; only the six unrun invocations remain (33 tests). The full focused selection is 43, including the previously overlooked existing control. The shared repair now has focused implementation evidence for the named controls, but affected preservation, current production build/stdio, governed admission and CI remain pending. No agent tests/builds/runtimes were started.

## Focused shared repair selection GREEN; production handoff (reviewed 2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-115618-261`: six new invocations passed, selecting 33 tests (hard links 3, resolver registrations 17, old membership 6, original collision 1, historical replay 5, failed-publication retry 1). The batch correctly retained the exact frozen retirement witness (1) and shared retirement controls (9), giving 43 passing tracked tests across the pinned captures. No compiler warnings were found. All six current log hashes and both prior receipt references match. The captured before/after source manifests are identical. An agent attempt to refresh every current source hash became unexpectedly slow and was stopped; a new current per-file source-hash verification is not claimed from that interrupted command. The next operator build batch captures current source pins separately.

This establishes the named focused repair/preservation results, not current executable behavior, CI, field acceptance or governed admission. The prior production executable/failed stdio captures predate the shared repair. No producer/schema/version or approved resolver contract was changed by this verification checkpoint.

Next operator command:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-build-stdio.ps1
```

The wrapper delegates to the existing scoped all-feature Clippy/build/UTF-8/file-size capture, requires exactly one newly completed build receipt, then runs the existing pinned synthetic three-session registration stdio witness with that build. It performs one production build, preserves raw logs and hashes, and stops on any failure. The stdio witness must resolve the earlier deleted-owner membership failure, not weaken it. PowerShell syntax and bounded UTF-8/whitespace inspection passed before handoff; no agent Cargo, guard or runtime was started.

After actual production results are reviewed, substantive findings, owner disposition and exact verification associations still require the appropriate governed document, actual admission checker and resolution of applicable blockers before push. Protected grants remain owner-only. CI and any applicable final field gate remain separate; no Claude switch is requested now.

## Current production boundary verified; governance candidate prepared (2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/registration-build-20261116-093721-400`: scoped all-feature library/binary/test Clippy passed (200.996687 seconds), production build passed (149.4825576 seconds), UTF-8 guard passed (40.4572944 seconds), active-file guard passed (4.8517157 seconds). No compiler warnings. Build raw log hashes and unchanged captured source manifests match. Source manifest SHA-256 is `25b8e03826e8bab856d21d24a596f9178f83904627aa53091491842e5c088f56`, the same pinned focused-test source.

Current executable SHA-256: `93f1431f8be130ae2f764b31a5d6e0d579955c8274c0d0b207d54ee20bdc0ffa`. Production stdio `target/ang-r2/registration-stdio-2026-11-16T14-44-02-327Z` passed all 185 checks in cold-forward, cold-reverse and restore-and-mutate sessions; all three processes exited zero. Exact trace/log/report/binary identities are retained. The formerly failing legacy membership check after deleted-owner recreation passed. Wrapper `target/ang-r2/retirement-production-20261116-094400-628` exited zero in 31.4023024 seconds; its raw log and binary-manifest hashes match. This establishes those production scenarios, not CI or Claude field certification. Historical failed stdio and regression receipts remain unchanged.

The existing governed evidence source `docs/agent/ISSUE_162_ROUTE_RESOLVER_ACCEPTANCE.md` (DOC-270) now has a dated continuation for the exact finding/root causes, owner disposition 02, shared implementation/affected consumers, bounded-work declaration, exact RED-to-GREEN and 43-test/185-check receipts, and acceptance limits. Historical generation-78 prose remains intact. Source-owned knowledge components and typed dependencies bind these substantive sections to the existing ownership/publication/persistence constraints. No new governed corpus identity, protected selection or membership was created. This is a candidate, not claimed admitted knowledge.

Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-governance-review.ps1`. It pins the current executable/profile/source/predecessor, runs actual Rust document describe validation and the normal knowledge-context admission checker, and captures complete diagnostics/descriptor. It neither creates nor submits a protected grant. If protected authorization is required, the exact returned descriptor must be reviewed before the owner is asked to authorize its Rust digest. No final publication/push, current governance acceptance, CI or field success is claimed.

## Repair evidence admitted; explicit replay adoption and formatting pending (2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/retirement-governance-review-20261116-095838-027`. Actual Rust descriptor validation returned no diagnostics and no authorization/publication mutation. The normal knowledge admission checker then returned ready/current/complete, no diagnostics, generation `190eab1a00bc0a9e1485ff0fae63887ea2110fd2c8098b97630ab7503799f421`, process exit 0. DOC-270 revision is `6b5ed796b65989c7355e2a3b35e63f7294f0cc68727269635c4e6cc24c05d087`; its six source-owned knowledge components match exact current source SHA, byte spans and component hashes. The context closure has 23 units / 28 relationships. Accepted/proposed governance state digest is unchanged (`3729f869f4c2d4f285fa8df214c82dd0adcae8343455be350d6ec663d58b06d7`), including all 342 protected selections. Existing evidence-source revision required no new protected grant. None was created or submitted. This resolves disposition 02's represented-repair/admission requirement for that candidate, not CI/field or all mission obligations.

The final obligation review distinguishes five passing historical replay compatibility tests from direct verification of the new registration collection's adoption before query hydration. A new tracked test `issue_162_resolver_registration_replay_adopts_exact_keys_before_query` now saves the actual high-fidelity collection, restarts state, invokes replay_history sequence 0, checks exact index collection and admission before any query can reacquire, then checks forward/reverse keys. It is UNRUN. Production code and frozen witnesses are unchanged. The current registration suite has 18 authored tests; earlier 17-pass receipts retain their exact selection.

Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-registration-prepublication.ps1`. It runs only that one new all-feature library test, then performs read-only rustfmt --check over active Rust files with child traversal disabled. It preserves hashes/logs, counts and durations, and mutates no formatter inputs. Scope is local, not the consolidated CI gate. Formatting status was previously unestablished and is not inferred from Clippy. No original frozen source is formatted or weakened. Any formatting conflict with byte-frozen witnesses must be reported with actual evidence before changing them.

No commit or push yet. The 43 earlier focused passes, current production build and 185 stdio checks remain exact-source evidence for their captures; this test-only addition does not establish its own success. Applicable final field verification and CI remain distinct pending obligations. No agent Cargo/formatter/guard/runtime command was started.

## Exact replay PASS; nonfrozen formatting complete; frozen source decision pending (2026-11-16)

Operator prepublication capture `target/ang-r2/prepublication-20261116-101602-601` passed the exact registration replay-adoption test (1/0, no compiler warnings, 272.8515432 seconds), then failed scoped rustfmt check. The report covers 35 files including two exact frozen witnesses. Agent MaxHeadRoom validated the captured diff hunks against their pinned original inputs and applied only the 33 unfrozen formatter outputs directly. No formatter process was started by the agent. The formatting-expanded lifecycle file required semantic decomposition from 858 lines into 8 query tests, 6 persistence/admission/replay tests and shared support. All fourteen formatted test-body hashes match exactly across relocation; all active Rust files are within 615 lines. Post-edit executable verification remains UNRUN.

Both original frozen source hashes remain exact. Their proposed formatting, untouched originals and reviewable diffs are preserved under `target/ang-r2/format-application-20261116`; this is archival evidence, not relocated test execution. `ANG_R2_FROZEN_FORMAT_REVIEW.md` records the concrete formatting-only proposal and why disposition 02 section 4's exact-source requirement needs an explicit preservation decision before those current bytes change. No skip, ignored test, warning suppression or guard bypass is proposed. The governing DOC-270 source and its admitted revision were not changed by this pending decision.

## Disposition 03 applied; post-normalization verification UNRUN (2026-11-16)

The owner explicitly approved only the reviewed wrapping, spacing and trailing-comma replacements for the two primary witnesses, retained source/evidence preservation, and concern-based decomposition of the fourteen lifecycle/persistence tests. `ANG_R2_OWNER_DISPOSITION_03.md` records the attributed decision; it grants no execution permission or protected authority authorization.

Agent MaxHeadRoom applied the exact proposed hashes: retirement witness `1399e46e1b5e2573b1ba70021e9172957d448ceb74c3ffa039204b6a04222cc0`, public witness `9af4593ad4cc62b462dd8d7ca292970018c3dcf3508724ab2ca8ed30c431978b`. Original hashes remain `69c27a304b59029f7aee0ed2bfd511eb47c2702c0a8a43bb22d09653368b797f` and `8b047c6e817adb63aa6ba078375258b68ce2b3677e73b88b15f2d283d78821dc`. The existing target evidence mechanism now consolidates exact originals/proposals, RED/GREEN outputs/receipts, available source manifests and the successful pre-format replay capture under `target/ang-r2/frozen-format-retention-20261116/manifest.json`. Manifest SHA-256 is `7af8f0742cab618c7ecca3c22346758eb1bfd80e6baea88a9165c370ffb695e5`. Every retained entry has an exact source path, byte count and SHA-256. The original public-witness stash remains unchanged. No source archive serves as an alternate test outside src/tests.

Both primary files preserve exact string-literal spellings and assertion macro counts. Fourteen formatted test bodies are retained across eight query tests and six persistence/admission/replay tests, with a shared helper module registered in semantic_compilation. Trailing inter-test separators at new file ends were normalized independently of function bodies. Static UTF-8/no BOM, active-file ceilings and whitespace are checked; compiler/format acceptance and normalized GREEN remain UNRUN. Earlier exact-source RED-to-GREEN, 43 focused passes, successful replay-only capture and 185 stdio checks retain their actual source identities and are not relabeled post-normalization evidence.

`invoke-format-verification.ps1` is the exact operator handoff: active-source read-only rustfmt check; normalized retirement witness (1); all registration tests (18, including two public witnesses, fourteen relocated cases and two transport tests); scoped all-feature library/binary/test Clippy; UTF-8 and size guards; actual normal DOC-270 knowledge admission checker. It captures source/log hashes, exits, counts and times, refuses changed approved identities and stops on failures. It starts no broad/full repository gate, formatter mutation, production build, commit or push. The final local knowledge stdio is explicitly part of the operator command, not an agent invocation.

The governing DOC-270 candidate records disposition 03 and the original/normalized source and evidence associations, and covers all mission documents through ANG_R2_EVIDENCE_INDEX.md and exact document pins. Its new candidate admission remains UNRUN; the earlier admitted revision remains historical, not current approval of this amendment. No protected grant is generated. Applicable CI and final field evidence remain separate.

Static reconciliation under disposition 03 is complete: both active primary files match the approved normalized hashes; 30 retained evidence entries hash-match; the fourteen relocated function bodies match their formatted originals after excluding inter-test separators; the shared helpers and all eighteen registration tests have tracked module registrations. Every active Rust file is at or below 615 lines (maximum 608). No compiler, formatter, guard, admission engine or test was started by Agent MaxHeadRoom. Post-normalization results remain UNRUN. Use ANG_R2_FORMAT_HANDOFF.md and invoke-format-verification.ps1 for the exact current operator packet.

## Post-normalization packet verified; durable evidence association completed (2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/format-verification-20261116-111528-924`: normalized retirement witness 1 PASS (104.9900179 seconds), normalized registration/relocation tests 18 PASS (619.2530992 seconds), scoped format PASS (0.5851703 seconds), scoped all-feature Clippy PASS (41.1512796 seconds), UTF-8 PASS (15.1181733 seconds), active-size PASS (6.0137078 seconds). No compiler warnings. All receipt log hashes, unchanged captured before/after manifests and current active Rust source pins match. The actual document checker returned ready/current/complete, no diagnostics, generation `00ee1003cb3a8c07d2c9f2511cf0bf6bbbc7952798643f888ec08ece7e95bfd4`, process exit 0. This verifies normalized witnesses, decomposed module/helper registration and the tested applicable contracts; it is not full CI or field/delivery certification.

The final preservation check found that exact original raw bytes still depended on ignored target storage. The existing versioned Git evidence mechanism now retains an ordinary binary archival artifact, `historical-witness-evidence.zip`, alongside the campaign records. SHA-256 `ea328ed56679bf05800658c6577d2a919f3de93fdcdd6a1ca9ba61eddd0b8b53`. This is a byte-preserving ZIP_STORED copy of the existing target retention directory, including its original manifest and all 30 recorded source/proposal/diff/RED/GREEN/replay receipt/log entries. Each archive entry was read back and verified against the source manifest hash. It introduces no evidence service, schema, importer or alternate executable tests. The bundle is a versioned artifact awaiting normal commit, so durable remote availability is pending commit/push rather than already claimed. Original target receipts/stash remain unchanged.

The governing record now associates that artifact and the observed 19-test packet. Because its substantive content and covered document pins changed, the new governance amendment and final documentation guards still require actual execution. The updated existing wrapper supports `-ResumeVerifiedFrom target/ang-r2/format-verification-20261116-111528-924`: it verifies exact unchanged Rust manifests, baseline, passing log hashes/counts/warnings, retains explicit reuse records, then runs only UTF-8/size guards and the actual normal admission checker. It does not repeat tests, format or Clippy. No new long-running invocation is authorized to the agent. Earlier replay-only and pre-normalized GREEN results retain their original source identities.

## Remaining lifecycle checks established; push-only preference (2026-11-16)

The owner clarified that the branch should be pushed without opening a PR. Draft PR 189, opened by the agent beyond that preference, was closed without merge or branch deletion. No new PR will be opened unless requested. Existing remote checkpoint remains 51d2a698d4a3efafcde0b297bcf9badb6eaa3261; this follow-up is not yet committed or pushed. CI remains a separate acceptance requirement and is not inferred from a branch push.

Agent MaxHeadRoom verified the additional-root/refresh test PASS in `target/ang-r2/remaining-lifecycle-20261116-114846-300`: two independent configured roots contribute four registrations, asserting withinPath selects only the additional root, configured_roots refresh clears projections, and forward/reverse reacquisition replaces only the changed key. The same capture's mutation test failed before mutation because the test incorrectly expected Edit fidelity to certify index coverage. Existing SemanticFidelity::from_compilation(Edit)=None was preserved; this was corrected solely in test observation to inspect retained semantic snapshots at mutation boundaries. No production code changed.

Corrected mutation capture `target/ang-r2/remaining-lifecycle-20261116-115908-481` passed one test / zero failed, exit 0, no compiler warnings, enclosing 101.0288367 seconds / test 29.52 seconds. Exact command: `cargo test --lib --all-features --quiet issue_162_resolver_registration_remaining_edit_and_delta_transport -- --test-threads=1 --nocapture`. Formatted test-file SHA-256 is `40f457028bc78375ae8e0940f6513437b242b3f71edf708f82bdd6a3a88ca97d`. stdout SHA-256 `25ca968dd0d59b8a4727909745e94ae6e462e4391bb853222bb241f3e9308c62`; stderr SHA-256 `a36d189f2a941bc250f01651aceed61c488e436e527f8369a1153e899257e043`. The test checks the actual apply_edit snapshot's updated source hash and preserved keys, actual apply_delta snapshot's key replacement/hash, and exact durable restore before a final public query can reacquire facts. This resolves these specific edit/delta/restore witnesses, not unrestricted arbitrary expression/edit support.

Earlier direct replay-adoption, refresh, retirement/relocation/recreation, source/dependency replacement and negative/compatibility witnesses retain their exact-source results. A new standalone reset API is not introduced: existing configured-root refresh and independent state restart are the applicable tested lifecycle boundaries. The current registration selector now contains 20 tests; prior 18-test receipts remain 18. The two new tests are additions in the existing persistence test module, not a new production abstraction or an additional decomposition.

Next exact operator handoff: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1 -VerifiedLifecycleFrom target/ang-r2/remaining-lifecycle-20261116-115908-481`. It validates the exact successful mutation receipt/source/count/log hashes, records reuse, skips tracked test invocations, then runs scoped read-only formatting, scoped all-feature Clippy, UTF-8/file-size guards and actual normal revised-document admission. It reuses known passes instead of rerunning unrelated tests. No agent Cargo/formatter/guard/runtime was started. Commit hooks, push, complete CI and any applicable final field verification remain separate pending obligations.

## Observed CI default-feature compilation corrected (2026-11-16)

Agent MaxHeadRoom read completed CI run 43909906505 on predecessor checkpoint 51d2a698d4a3efafcde0b297bcf9badb6eaa3261. It failed at warning-fatal default-feature `cargo check --all-targets` before reaching tests: an unused TEST_INJECTED_IR_FAILURE reexport, unused CSharpCallAuthorityState::status test accessor, and unread hydration_scan_calls instrumentation. Their existing consumers are gated on feature rust; definitions/reexport had only cfg(test). All-feature local passes did not establish this default-feature case. Current pushed d2eb86dfb53f2b102b461c009fb90495d0181315 has no observed CI run. The closed PR remains closed; CI triggers were not broadened and no replacement PR is authorized.

The smallest correction aligns those three test-support items (including field initialization) to cfg(all(test, feature = "rust")). No production behavior, warning suppression, test assertion, producer generation, schema, runtime eligibility or workflow trigger changes. Feature-guard verification `target/ang-r2/feature-guard-check-20261116-123207-211` passes warning-fatal `cargo check --all-targets` in default mode (166.136998 seconds) and `cargo check --all-targets --all-features` (64.0292848 seconds). Raw receipt/log hashes match. This is compilation evidence, not execution of tracked tests, full CI, field verification or a green report for the failed historical CI run.

Next operator handoff: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-format-verification.ps1 -VerifiedFeatureGuardsFrom target/ang-r2/feature-guard-check-20261116-123207-211`. It checks the exact two successful feature-mode commands, baseline and log hashes, records compilation reuse, skips tracked test runs, then runs read-only scoped formatting, scoped all-feature Clippy, UTF-8/active-size guards and the actual revised DOC-270 admission checker. All prior functional results remain attributed to their original captures; no runtime behavior changed. No agent Cargo/formatter/guard/server/pilot was started. Commit/push, successful full CI and any final applicable field receipt remain pending.
