# B11 verification record

Agent: **Agent MaxHeadRoom**. No B11 tests or builds have run yet.

Update: `target/pcg-b11/admission-red-20261113-153244-400` compiled and
recorded exactly two behavioral failures on missing diagnostic location lines
(duplicate status at line 8, invalid status at line 7). Existing rejection,
catalog-currentness and generation assertions passed before those failures.
Frozen test SHA-256:
`227f736d9174bf8b60f3bad20310a1f877f3d840817a84ad76d19a506e0e8dd9`.
Narrow stash `7bd5a0666efb408b91ad11228af38b2619b14730` preserved only the
test and registration before the production changes; it was restored unchanged.
No production fix was made before that observed RED.

The existing Diagnostic now optionally carries an exact line/column/field,
observed value, rejected disposition and false accepted-publication flag for
located parser/identity/kind/status errors. Other diagnostics are not silently
mislabelled as admission rejection. Parsing retains real fence line offsets;
validator field lookup stays inside the exact declaration fence and ignores
nested illustrative fences. Missing locations are not guessed. This is a partial
diagnostic correction: governing references and other violation locations remain
open B11 work, not implicitly verified. The unchanged tests and complete document
suite await operator execution using `invoke-admission-green.ps1`.

GREEN receipt: `target/pcg-b11/admission-green-20261113-153651-659` records
the two unchanged diagnostic regressions passed, and the complete document suite
passed 41 tests. This closes the initial field-location witness only; it does not
establish complete B11 diagnostic coverage.

The next tracked RED file, `document_admission_authority.rs`, requires an exact
accepted governing component reference despite simultaneous candidate edits to
the governing source. It independently acquires the pinned historical component
while the candidate catalog is blocked, checks that currentness stays false, and
restores/corrects the candidate for successful legacy admission. A second case
requires truthful disclosure when no governing authority is in the accepted
catalog rather than fabricating a reference. No production governing-reference
implementation exists yet. Run `invoke-admission-red.ps1 -Authority` for the
expected behavioral RED before that correction.

Four-test RED observed in `target/pcg-b11/admission-red-20261113-163403-371`:
two tests fail on missing governing authority metadata and two on missing
governed membership enforcement. No compile failure substituted for behavioral
RED. Exact frozen authority/membership test hashes are recorded in that receipt.
Narrow stash `d7d5992fb224cbe7b197a8a1e39d2e8620b93149` preserved the two files
and registration before their production correction. Applying it restored the
untracked tests but refused to overwrite the now-modified production module;
the registration-only tail was restored manually, preserving production edits
and exact test file bytes. The original stash remains retained.

The current bounded correction adds accepted-head governing-reference enrichment
to both query entry points and the enrolled-interface same-root document guard.
References select exact accepted statement components with historical pins,
never candidate policy or automatic governing text. Unavailable accepted authority
is explicit. A request-local authority map avoids repeated lookup per diagnostic.

Registry parsing and expected-ID matching run in the shared scanner. CSV bytes
are read through existing observed-source machinery and rechecked at completion.
Canonical safe paths must remain in existing document roots. Duplicate ownership,
missing registered IDs, unexpected IDs and unregistered declared sources reject.
Ordinary unregistered Markdown does not become graph membership. Registry absence
still has legacy compatibility until accepted governance-state activation is
implemented; it is NOT claimed protected against removing an active registry yet.
Accepted snapshot/authorization mechanisms and migration remain required work.

The six unchanged B11 regressions and complete document suite now await focused
GREEN with `invoke-admission-green.ps1`. No B11 build or production verification
of these changes has been run by the agent.

GREEN receipt `target/pcg-b11/admission-green-20261113-163808-027` records all
six unchanged B11 regressions passed and all 45 document tests passed. This
verifies the first governing-reference and explicit membership corrections.

The next tracked bootstrap test establishes an accepted legacy predecessor,
adds structurally valid registry/protected files, and requires blocked admission
with the same accepted generation until explicit bound operator activation. It
also requires the exact change descriptor and procedural trust disclosure. The
current implementation instead silently admits that valid registry; no operator
authorization or governance snapshot implementation exists yet. The intended RED
must precede the production bootstrap correction. Run
`invoke-admission-red.ps1 -Bootstrap`; no full gate is requested.

Bootstrap RED observed in
`target/pcg-b11/admission-red-20261113-164314-010`: the unchanged candidate was
incorrectly ready instead of blocked. Frozen test SHA-256
`341ff962a7b017167517bfbef1aae7c9ae226a1342fe6d23ea3688ff555a143b`.
Narrow stash `548d2ad1b31ae38bcf59d2e921af561b21c03bb1` preserved it and the
registration before implementation. The exact file was restored from the stash's
untracked tree; existing production module changes and previous test registrations
were retained independently. This is behavioral RED, not a compiler failure.

The current implementation adds schema-v3 governance/publication and exact grant
binding rows, recognized v2 migration, and retained v1 migration. Candidate registry
existence does not activate accepted governance. The operator CLI describes without
publishing; explicit authorize verifies the actual descriptor and predecessor and
records only the procedural grant. Query reconciliation recomputes the descriptor,
then commits accepted graph/governance together after COMPLETION. Invalid requests
retain previous accepted graph/state; describe-only migrations roll back.

New isolated tests simulate the explicit owner action, outside agent-facing MCP.
They cover bootstrap, protected punctuation and qualification changes, outside-
component edits/offset changes, substituted digest, workspace mismatch, registry
removal, correction/resubmission and v2 migration rollback/history preservation.
They do not authorize or modify the actual repository's governance. All current
changes remain unverified until the focused GREEN run. Existing v1 migration
success now expects schema version 3 while retaining its original identity/history
assertions. No full gate is requested.

The initial authority discovery/acquisition operator receipt is
`target/pcg-b11/authority-20261113-152531-723` (complete/current/ready).
This is production acquisition evidence, not a Rust regression result.

The first tracked RED candidates are in
`src/tests/pcg/document_admission_diagnostics.rs`, registered through
`src/pcg/documents/mod.rs`. They exercise exact authored status/duplicate-field
locations, truthful blocked currentness and retained accepted generation. The
status case also protects historical membership and correction/resubmission.
Production remains unchanged. Expected RED: missing location metadata at the
exact line assertion, after existing rejection/preservation assertions pass.
Compile errors and unrelated failures are not valid behavioral RED.

Next operator command: `invoke-admission-red.ps1`. It captures exact test and
registration hashes and the complete Cargo output in a fresh target directory.
Full repository gates are CI-only. No independent evaluation is authorized.

Latest checkpoint: `target/pcg-b11/admission-green-20261113-165342-743`
compiled successfully and ran 11 B11 tests: 10 passed, one failed. Its history
assertion incorrectly expected three publications instead of four. Governance
activation creates another publication of the same revision. The corrected test
checks four publications, three distinct revisions, separate activation generations
and exact source preservation. Production code was unchanged for this correction.
Broader document/MCP tests and Clippy were not reached. The failed log remains
preserved. Rerun `invoke-admission-green.ps1` for the corrected focused checkpoint;
this is not a full repository gate and no passing outcome is claimed yet.

The subsequent operator receipt `target/pcg-b11/admission-green-20261113-170058-332`
records exit code 0 for all three tracked test steps: 11 B11 tests, 50 document
tests (including those B11 tests), and 10 registered MCP tests. These counts
overlap and must not be summed as unique coverage. The corrected history
assertion passed. At inspection, scoped Clippy remained active and its completion
was absent from results.json; no Clippy success or complete checkpoint is claimed.
No agent-owned test, Cargo command or runtime was started.

That receipt subsequently completed with scoped Clippy exit code 101:
`result_large_err` at both parser functions returning `Diagnostic` (288 bytes).
The correction boxes only parser error returns and moves the diagnostic back
into the existing capture vector at its consumer. Diagnostic fields, serialized
responses and successful parsing remain unchanged; no lint suppression was added.
Existing tracked parser/admission regressions remain intact. The correction awaits
the focused operator rerun; the earlier test successes precede this code change.

Receipt `admission-green-20261113-170759-900` completes the parser correction
checkpoint: all three test steps passed (11 B11, 50 document, 10 MCP; overlapping
counts), and scoped all-target/all-feature Clippy exited 0 with warnings denied.
Static `git diff --check` also passed. This is focused verification, not CI or
production-runtime verification.

Remaining lifecycle review identified a protected-component replacement gap:
projection of previously protected selectors requires the old component to exist
in the candidate, so a valid explicitly proposed replacement fails before an
operator descriptor can be produced. A new tracked regression requires refusal
before authorization, an exact reviewable descriptor, authorized replacement and
retained source history. Run `invoke-admission-red.ps1 -Replacement` before any
production correction. Existing protection must not be silently discharged.

Replacement RED receipt `admission-red-20261113-171401-460` compiled and failed
exactly at descriptor creation with `governance_protection_component` (one test,
exit 101). Frozen operator test SHA-256:
`2f0f5f5ea0ca2b187ecc250588475b14be02d09840bd96e95192a46220612589`.
Narrow stash `e06c44eed2b9343d9305816bdc0d2999f7a4df45` preserved that file before
the correction; the exact file was restored from its untracked tree afterward.

Projection now distinguishes strict proposed protections from observation of
previously accepted protections. Missing old components are explicit descriptor
observations, not discarded obligations or operational exceptions. Proposed
selectors must still resolve uniquely; missing candidate identities are rejected.
Replacement changes remain bound to the accepted predecessor, exact proposed state
and previous protection observation, and still require procedural authorization.
The correction awaits the focused GREEN rerun. No agent-owned Cargo command ran.

Replacement GREEN receipt `admission-green-20261113-172257-555` records exit 0
for 12 B11 tests, 51 document tests (overlapping), 10 MCP tests and scoped
all-target/all-feature Clippy with warnings denied. The frozen replacement test
passed, including authorized publication and exact source history preservation.

The next bounded lifecycle batch adds tracked tests for exact authorized corpus
removal and identity-preserving relocation. Inspection found the legacy
`E_DECLARATION_REMOVED` check runs before governance preparation and prevents a
reviewable removal descriptor even when the owner explicitly proposes registry
removal. No production correction has been made. Run
`invoke-admission-red.ps1 -Lifecycle`: expect the removal test to fail at its
reviewable-descriptor assertion, with relocation as a preservation control.
Unapproved disappearance must remain refused; authorization must retain history.

Lifecycle RED receipt `admission-red-20261113-173419-417` compiled both tests:
relocation passed; removal failed at the expected reviewable-descriptor assertion
with `E_DECLARATION_REMOVED`. Frozen lifecycle test SHA-256:
`4f4bdd090e61be292011aa2bb1781dff19a74a950d5bbe3ac6f3f5269c2b8266`.
Narrow stash `af97a7f18b6d260214360c348529d021e0d0ee57` preserved the file before
the correction; exact restoration from its untracked tree was hash-verified.

Shared governance preparation now defers only the legacy disappearance diagnostic
for identities explicitly removed from previously accepted governed membership.
The resulting changed state still requires exact owner authorization. Legacy
catalogs, unchanged membership with missing declarations, unrelated errors and
invalid candidate membership remain blocked. Accepted graph and governance still
publish in the existing transaction; old publications remain historical. The
correction awaits focused GREEN verification; no agent-owned Cargo command ran.

Receipt `admission-green-20261113-174031-975` compiled and ran 14 B11 tests:
13 passed, one failed. Authorized removal and relocation succeeded; the unchanged
removal regression then exposed zero historical entries instead of two. The
legacy history query enumerated current members before looking up publications,
making removed identities inaccessible despite retained database history.
Later document/MCP/Clippy steps were not reached.

History now queries publication membership by the requested identity independently
of current membership, retaining publication sequence, generation, revision
validation and historical freshness. No history rows are deleted or rewritten.
The lifecycle test remains frozen at its recorded SHA-256. This continuation
correction awaits the focused GREEN rerun. Static whitespace checks pass;
no tests, Cargo commands or runtimes were agent-started.

Lifecycle GREEN receipt `admission-green-20261113-174847-817` records all steps
exit 0: 14 B11 tests, 53 document tests (overlapping), 10 registered MCP tests,
and scoped all-target/all-feature Clippy with warnings denied. The unchanged
membership-removal test now passes its complete sequence, including historical
retrieval after publication. This does not establish actual-corpus governance
activation, production executable verification, or CI completion.

The subsequent source review prepared explicit persistence component boundaries
and updated workflow documentation. Four new tracked association/replay controls
are registered in `document_governance_associations.rs`; these are coverage
extensions to the implemented policy, not claimed RED-to-GREEN demonstrations.
The next focused GREEN run must cover the new tests and scoped Clippy. Actual
source reconciliation, protected bootstrap disposition and production executable
verification remain outstanding. Existing frozen regressions are unchanged.

Receipt `admission-green-20261113-182144-241` records exit 0 for 18 B11 tests,
57 document tests (overlapping), 10 MCP tests and scoped all-target/all-feature
Clippy with warnings denied. All four association/replay extensions passed.
No new production Rust code has changed since that receipt.

Owner disposition 04 authorizes the subsequent documentation boundary work and
prospective CI-responsibility amendments. `invoke-production-capture.ps1` is the
next operator handoff: one all-feature production build, six actual repository
authority contexts, explicit exact source acquisition and read-only before/after
catalog preservation checks. It records executable/profile identity and changed
revisions. It contains no authorization submission and does not activate the
protected bootstrap. This reusable harness is operator/production evidence,
not Rust regression coverage, a CI gate or independent evaluation.

Production receipt `production-20261113-183428-928` completed successfully:
all-feature binary build exit 0; six actual repository-root authority contexts and
explicit acquisition completed current/ready; 40 distinct acquired spans matched
exact current source bytes and hashes. Before/after checks preserved all 218
identities, 221 typed relationships, ownership and prior revisions. Actual
generation: `faa04cbae0b9bc088f4f57903174271fb761cffc8815b06378c9f943fcf8bfbe`;
336 published components. Executable SHA-256:
`50385c5cf7d1a69bae2cbe4914a2175fdc2f67a944ebdc2037622d5bbdcacf43`.
Profile SHA-256:
`0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.
Six advanced revisions (FINAL_VERIFICATION, PCG_DOCUMENTS, PCG_GLS,
PCG_PREAUTHORING, PERSISTENCE_RECOVERY and TOOL_WORKFLOW) are recorded explicitly
in its `verification.json`; no bootstrap was activated.

Subsequent source-owned corrections are not yet reconciled. The static candidate
selection now has 342 components across the same 137 protected identities; this
is not the published graph's total component count. Static marker, UTF-8,
selection and membership checks are separate from production verification.

Next operator handoff: `invoke-admission-red.ps1 -RuleComponents -ProxyControls`.
It first runs six existing direct cache tests for owner disposition 05, recording
their actual result separately. It then formats and freezes the new tracked
governing-component witness and runs only that witness. Expected RED: blocked
admission and accepted-head preservation already pass, followed by an assertion
that the diagnostic lacks its independently represented constraint reference.
Do not call compilation failure or zero selected tests behavioral RED. The
production enrichment correction has not been applied; previous frozen tests and
all failed receipts remain preserved. No agent-owned tests or Cargo commands ran.

Observed on 2026-11-14: receipt `admission-red-20261114-183914-297` records six
existing direct proxy cache controls passing (exit 0, selected count verified).
The new governing-component witness compiled, ran once and failed at the exact
missing-constraint reference assertion (exit 101). Its preceding blocked/currentness,
accepted-generation and no-current-document assertions passed. This is behavioral
RED, not compilation failure. Test SHA-256:
`42624910d2a95c30f29dfc3f9860491f2c1af7bd7ac55153fc2c55c7cdc56666`.
Stash `50c2c55ae0fdc88f318d221c9fdae3bbb1fec7b9` preserves that test and registration.
Existing production module changes were retained independently; the new registration
was isolated during correction, then restored, and the exact test hash verified.

Enrichment now selects accepted controlling components and qualifications rather
than only summaries. Observation/history components are excluded. Accepted revision
pins, historical acquisition, candidate-policy exclusion and request-local reuse
remain unchanged. The frozen witness is included in the GREEN hash guards.

The same checkpoint adds two deterministic AF-PERF-001 controls: 2 versus 200
diagnostics perform two distinct-authority revision lookups and one available
component load; 1 versus 12 selected components of one unit derive governing
associations once and perform no additional source reads. Internal work counts
measure the actual lookup/load/derivation sites and do not enter response, digest,
accepted-state or authorization data. These are new coverage extensions, not
additional observed behavioral RED claims. They await the focused GREEN run.
The successful production executable receipt predates this Rust correction and
must not be presented as verification of the changed executable.

GREEN receipt `admission-green-20261114-190904-271` records all steps exit 0:
21 B11 tests, 60 document tests (overlapping), 10 registered MCP tests and scoped
all-target/all-feature Clippy with warnings denied. The unchanged governing-rule
witness and both work-count controls passed. No further Rust production edit
followed that receipt.

The next production handoff consolidates one rebuild with isolated CLI/MCP
governance fixtures and final real-root reconciliation/acquisition. The user-owned
fixture harness simulates explicit procedural authorization only for a fresh
synthetic workspace under `target/pcg-b11`; it never submits authorization for
the actual repository. It captures bootstrap refusal, describe/grant nonpublication,
atomic activation, ordinary outside edits, protected-change refusal/admission,
stale/substituted grant refusal, exact historical access, registered declaration
loss and serialized multi-component governing references. These are production
operator receipts, not additional Rust regression or CI results.

The real-root phase retrieves the six authority contexts plus an unfiltered
existing search to enumerate current descriptors. Explicit acquisition is scoped
to every reviewed protected component plus the superseded proxy sentence. It
checks all 342 proposed components against independently prepared source/content
pins, exact returned UTF-8 spans, and the 137-identity scope. Before/after checks
also compare immutable revision row fingerprints, source bodies and publication
history. All real-root grants/governance publications must remain empty. This is
verification of a proposal, not actual protected bootstrap.

Three changed governing files were canonicalized to the repository's expected LF
before final collection, preserving all Unicode/content. Their physical hashes
and candidate pins were refreshed; prior accepted source bytes remain historical.
This avoids making the initial protected state depend on pending Git newline
normalization. All new handoff scripts passed static syntax/whitespace inspection;
the production handoff has not been run by the agent.

Receipt `production-20261114-200614-490` built successfully, executable SHA-256
`4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f`.
The first synthetic legacy request then failed with JSON-RPC -32602
`pcg/invalid_request`: `baseline-rpc/2.request.json` omitted the required
`tokenizer` field. The legacy tool schema and Rust Request both require it.
This is a harness request-contract failure, not a failed governance assertion.
No synthetic authorization or actual-root reconciliation had been reached.

The fixture query helper now supplies `tokenizer=o200k` only for the legacy
document interface. Knowledge requests retain their own separate contract.
Production code and input validation are unchanged. The failed receipt remains.
A fresh capture may reuse that successful executable via `-ReuseBuiltBinaryFrom`;
the script checks exact executable SHA, source HEAD and changed Rust/Cargo input
identities, then records reuse explicitly. It neither reruns Cargo nor silently
claims a new build. The operator must execute the corrected handoff; the agent
ran only bounded source inspection and static syntax/whitespace checks.

## 2026-11-14 final proposal production receipt

Operator receipt `production-20261114-201955-538` passed 47 isolated CLI/MCP
assertions and actual-root source verification. It reused the successful build
from the preserved failed harness receipt after checking executable and Rust input
identities. Executable SHA-256:
`4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f`.
Profile SHA-256: `0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.
Accepted generation:
`9dbaacb94dc0be012a0c23e3d5e0d3d6fc5453c3b56f5cc569df0035dda75703`.

All 218 identities, 221 typed relationships, immutable revision/source rows and
publication history were preserved. The catalog contains 482 components; 342
proposed protected components and 280 distinct exact UTF-8 source spans passed.
The 58 revisions advanced since the captured predecessor are recorded individually
in `B11_PRODUCTION_REVISION_CHANGES.csv`; these are not identity removals.
Actual repository grants and governance publications remained empty.

The verified four-column 342-component selection is now staged as the working
`docs/architecture/knowledge-governance/protected.csv` candidate. This does not
activate governance: subsequent admission must refuse bootstrap until the operator
reviews and explicitly authorizes its exact Rust-computed descriptor. The new
`invoke-bootstrap-review.ps1` captures that descriptor without authorization or
publication. Its execution is handed to the operator, not run by the agent.

The rule matrix now records current grouped dispositions rather than provisional
mechanism holds. It remains a bounded mechanical inventory: prose truth, policy
applicability and human authentication are not automatically inferred. The final
bootstrap, focused repository guards and delivery commit/push remain outstanding.
No full repository gate or independent evaluation was run; CBM remains explicitly
failed/degraded. Historical receipts and reports remain preserved.

## Bootstrap review handoff correction

Receipt bootstrap-review-20261114-220625-918 failed before descriptor creation
with document_profile_path. The handoff incorrectly passed a relative profile
path; Trusted::load requires an absolute path. The script now resolves the
existing profile to an absolute path and surfaces retained stderr on failure.
No production code or validation was changed; no authorization was submitted.
The failed receipt is preserved. Only the read-only descriptor capture needs
to be rerun by the operator, with a fresh automatically named receipt.

## Exact operator bootstrap descriptor captured

Read-only operator receipt `bootstrap-review-20261114-220707-840` succeeded.
The sole diagnostic is E_GOVERNANCE_BOOTSTRAP_REQUIRED, with no grant and no
accepted publication. Descriptor membership is exactly 153 paths / 218 identities;
all 342 protection selectors match B11_FINAL_PROTECTED_SCOPE.csv exactly.
The predecessor is 9dbaacb94dc0be012a0c23e3d5e0d3d6fc5453c3b56f5cc569df0035dda75703;
accepted governance state is null, as required for initial bootstrap.
Exact operator-review digest:
4115d4b02491b3259c5165b429ea210fe5e0e3c2e53dadacaab3724a9d13d20e.
The agent has not submitted it. An explicit operator authorization is the next
step; it records only the exact grant, with publication verified separately.
## Final bootstrap publication and bounded closeout status

Agent MaxHeadRoom inspected operator authorization receipt
`bootstrap-authorization-20261114-221135-999.json`: exact reviewed digest granted,
accepted_publication=false. Operator publication receipt
`bootstrap-publication-20261114-221250-812` then passed, activating governance at
`4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3`.
The publication binds exactly to the reviewed state, predecessor and grant.
All 218 identity/revision memberships, 221 relationships, 482 components, prior
revision rows, source bytes and publication history remain intact. Exactly one
new governance publication was added. The protected set has 342 selections across
137 identities; the governed corpus has 153 source owners / 218 identities.

The matrix contains 14 grouped obligations, not 14 individually enumerated GLS
clauses. Ten groups preserve existing mechanical/procedural obligations; four
rows identify three distinct corrected gap families (GOV.AUTH appears in two
related obligations): diagnostic location/accepted authority disclosure, explicit
governed membership, and exact procedural protected-change authorization.
Additional constraint-reference and authorized-removal/history regressions were
corrected inside those families; they are not padded into separate policy counts.
No owner policy decision remains held. Procedural pre-authoring remains procedural;
this is not automated semantic truth validation or authenticated human identity.
Historical matrix observations and failed receipts remain described in the dated
records; the CSV records the final bounded disposition. Structural positive and
negative controls ran in the focused document suite; this is not exhaustive
per-clause transition certification. Source status never proves execution approval.

Final focused evidence: 21 B11 tests, 60 document tests (overlapping selection),
10 MCP tests, scoped all-feature Clippy; six existing proxy unit controls; 47
isolated production CLI/MCP assertions; exact source selection and actual-root
bootstrap preservation. These categories are separate, not additive test counts.
The production executable remains
4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f.
No Rust or protected-source edits followed its verified proposal capture.

Outstanding delivery: user-owned encoding/file-size guards, normal guarded commit,
exact committed-state review and push on (fix)Performance-Regression-Issues.
Full repository gates remain CI-only. Live CBM remains failed/degraded. All sixteen
independent evaluations remain UNRUN; no B10, new mission, or certification claim.