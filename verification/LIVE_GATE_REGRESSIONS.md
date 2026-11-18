# Live gate regression ledger

**Maintainer:** Agent MaxHeadRoom  
**Started:** 2026-10-13  
**Purpose:** Track reproducible failures discovered while running Windows live
verification gates. Repair is intentionally deferred until the current gate
sequence is complete so related production fixes and regression coverage can be
planned together.

This ledger records live/operator evidence. It is not a substitute for tracked
Rust regressions under `src/tests/**`, and a live PASS does not establish CI
coverage.

## Regression records

### LGR-001 — .NET test target is fabricated from the test-class suffix

- **Issue:** #134
- **Gate:** `Invoke-NativeCausalControls.ps1`
- **Case:** `134 / test-target-must-exist`
- **Windows evidence:**
  `target/native-causal-controls/20261013-114555-7b27f06e/ghost-test-target.json`
- **Observed:** `GhostTests` publishes a `Tests` edge targeting `Ghost`, although
  no `Ghost` declaration exists in the fixture.
- **Expected:** A `Tests` target must be justified by a real source declaration;
  a class-name suffix alone must not fabricate an endpoint.
- **First failing boundary:** .NET test subject-under-test selection/verification,
  before query rendering.
- **Repair requirements:** Change production behavior, add a tracked regression
  under `src/tests/**`, rerun the focused native gate, and then rerun applicable
  aggregate verification.
- **Repair evidence:** Commit `1d5f1fce`; tracked declaration-proven subject
  regressions; focused native PASS in
  `target/native-causal-controls/20261015-092905-a3f7a10a/`; complete final
  verification gate reported green on 2026-10-15.
- **Status:** Closed as part of Issue #134 final acceptance reconciliation.

### LGR-002 — Linked HTML `entities_in_file` dispatches to unsupported language IR

- **Issue:** #141
- **Gate:** `Invoke-NativeCausalControls.ps1`
- **Case:** `141 / linked-html-entities-in-file`
- **Windows evidence:**
  `target/native-causal-controls/20261013-114555-7b27f06e/verdicts.json`
- **Observed:** `workspace_query(type="entities_in_file")` for a linked `.html`
  template returns JSON-RPC `-32603` with
  `IR error: Unsupported file extension: .html`.
- **Expected:** An owned linked Angular template must reach the file-entity
  projection boundary without being sent through the language IR compiler.
- **First failing boundary:** `entities_in_file` dispatch for linked HTML.
- **Repair requirements:** Change production behavior, add a tracked regression
  under `src/tests/**`, rerun the focused native gate, and then rerun applicable
  aggregate verification.
- **Historical-evidence correction (2026-10-15):** The former repair entry
  cited Issue #134 commit `d720de41` and ASP.NET ancestry runs. Those artifacts
  do not exercise linked HTML and are not LGR-002 evidence. The original RED
  capture remains authoritative historical evidence.
- **Implemented repair:** `entities_in_file` now shares the same conventional-
  filename/structured-`templateUrl` ownership authority as
  `provide_code_context` and returns the existing canonical
  `angular/Template` representation request-locally. It does not publish fake
  raw entities or move durable assertion ownership away from the declaring
  Component source.
- **Tracked regression:**
  `src/tests/mcp/provide_angular_template_ownership.rs::entities_in_file_uses_the_same_template_authority_for_conventional_and_linked_html`
  covers conventional and arbitrary linked HTML, canonical Template identity,
  and the unrelated-HTML negative guard.
- **Focused live evidence:** User-run native causal controls passed on
  2026-10-15 at
  `target/native-causal-controls/20261015-225232-eb0a7fb0/`. Both #141 cases
  are GREEN: `same-content-filename-and-nested-condition` and
  `linked-html-entities-in-file`. The latter includes canonical identity and
  provenance, conventional/linked equivalence, unowned HTML rejection, and
  refresh-driven owner change/deletion/restoration. Generated captures retain
  the negative and lifecycle evidence without replacing the tracked test.
- **Final verification:** The maintainer reports the tracked Rust gates,
  Angular live acceptance, preservation checks, zero-warning checks, and the
  authoritative final repository gate GREEN on 2026-10-15.
- **Status:** Fixed and verified. Closed as part of Issue #141 final acceptance
  reconciliation; historical RED evidence remains preserved above.

### LGR-003 — Angular entity query leaks the builtin class identity

- **Gate:** `Verify-InvocationEvidenceLive.ps1`
- **Case:** `phase7-same-source-angular-ngrx-rx-composition`
- **Windows evidence:**
  `target/invocation-evidence-verification/20261013-124145-6d1f1e80/captures/cross-same-source-entity.json`
- **Observed:** `workspace_query(type="find_entities", domain="angular",
  entity_type="Service", name="LayeredService")` returns both the Angular
  `Service` identity and the builtin `Class` identity for the same declaration,
  producing `count: 2`.
- **Additional evidence:**
  `target/issue-reconciliation/20261013-132835-8d74d247/native-frame-17.json`
  shows `AuditApiService` returned from two dependency-use files as well as its
  declaration file, plus the builtin Class identity. The same query therefore
  reports four occurrences where one declared Angular Service exists.
- **Expected:** The explicit Angular domain and Service-type filters must return
  exactly the matching Angular service declaration identity, never builtin
  identity or dependency-use occurrences.
- **First failing boundary:** Workspace entity filtering/projection after the
  composed TypeScript compilation has published both identities.
- **Repair requirements:** Change production behavior, add a tracked regression
  under `src/tests/**`, rerun the focused invocation-evidence gate, and then
  rerun applicable aggregate verification.
- **Status:** Open; repair deferred until the live-gate sequence is complete.

### LGR-004 — Indirect ASP.NET controller ancestry is not classified

- **Issue:** #134
- **Gate:** `Invoke-NativeIssueReconciliation.ps1`
- **Case:** `134 / controller-two-project-bases-below-framework`
- **Windows evidence:**
  `target/issue-reconciliation/20261013-132835-8d74d247/native-frame-7.json`
- **Observed:** After publishing `OrdersEndpoint -> SecondHttpBase ->
  SharedHttpBase -> ControllerBase`, an exact `.NET Controller/OrdersEndpoint`
  query returns only the builtin Class identity and selects no Controller.
- **Expected:** The published multi-hop base chain must refine
  `OrdersEndpoint` to the `.NET Controller` identity.
- **First failing boundary:** Query-time .NET framework classification over
  transitive base-type ancestry.
- **Repair requirements:** Change production behavior, add a tracked regression
  under `src/tests/**`, rerun the focused aggregate case, and then rerun
  applicable aggregate verification.
- **Status:** Open; repair deferred until the live-gate sequence is complete.

### LGR-005 — Legacy NgRx action declaration is missing from its source file

- **Issue:** #140
- **Gate:** `Invoke-NativeIssueReconciliation.ps1`
- **Case:** `140 / legacy-string-action-has-declaration`
- **Windows evidence:**
  `target/issue-reconciliation/20261013-132835-8d74d247/native-frame-20.json`
- **Observed:** `SET_AUDIT` is published as an NgRx Action occurrence only from
  `dispatch.component.ts`; its `export const SET_AUDIT = '[Audit] Set'`
  declaration in `legacy.actions.ts` is absent.
- **Expected:** The declaration file must own the Action declaration; dispatch
  use may be additional evidence but cannot replace declaration ownership.
- **First failing boundary:** Legacy string-action declaration publication.
- **Repair requirements:** Change production behavior, add a tracked regression
  under `src/tests/**`, rerun the focused legacy-action gates, and then rerun
  applicable aggregate verification.
- **Tracked regressions:**
  `src/tests/angular_meta/ngrx_legacy_discriminant.rs`,
  `src/tests/mcp/ngrx_legacy_transition_matrix.rs`, and
  `src/tests/mcp/ngrx_legacy_object_dispatch.rs` establish declaration
  ownership, exact reducer-transition identity, and exact imported
  discriminant dispatch authority.
- **Repair evidence:** The focused legacy-transition harness passed 80/80 at
  `target/ngrx-legacy-transitions/20261014-113252-c8a075b4/`. The committed
  replacement recall benchmark passed every relation at
  `target/ngrx-recall-benchmark/20261015-040208-69239c88/`: Dispatches 20/20,
  Selects 21/21, HandlesAction 22/22, and TriggersReducer 21/21. The frozen
  oracle checksum remained
  `64f30ff5eafaab5da0a28ba3aeed8d444e5d1e0daf43a1b11b56784061b35171`.
- **Status:** Fixed and verified. The original reconciliation capture remains
  historical RED evidence; tracked regressions, focused operator verification,
  the complete repository gate, and the frozen recall benchmark are green.

## Gate record

| Gate | Windows result | Evidence | Regression IDs |
| --- | --- | --- | --- |
| Phase B typed/raw MCP acceptance | 18/18 PASS | `target/cbm-phase-b/20261013-111756-b734ca05/` | None |
| A5 provider admission/lifecycle | PASS; matrix captured | `target/cbm-a5/20261013-113211/` | None |
| Duplicate bare-name trace identity | PASS | `target/cbm-duplicate-trace-verification/20261013-113450.json` | None |
| Post-CBM causal investigation | 64/64 PASS | `target/post-cbm-investigation/20261013-113641-426fdf62/` | None |
| Independent native causal controls | 5/7 PASS | `target/native-causal-controls/20261013-114555-7b27f06e/` | LGR-001, LGR-002 |
| Repaired native causal controls | 7/7 PASS | `target/native-causal-controls/20261015-225232-eb0a7fb0/` | LGR-002 fixed; both #141 cases PASS |
| #136 whole-declaration insertion | 23/23 PASS | `target/insert-before-declaration/20261013-114843-33e29bb8/` | None |
| #137 semantic signature evidence | 25/25 PASS | `target/semantic-signature-evidence/20261013-115026-99b7df08/` | None |
| #135 DI retention | 16/16 PASS | `target/di-omission/20261013-115210-850b1e84/` | None |
| #139 decorated owner identity | 32/32 PASS | `target/decorated-owner-identity/20261013-115321-329fc8a7/` | None |
| Framework declaration publication | 57/57 PASS | `target/framework-declaration-publication/20261013-115359-2edf6cb7/` | None |
| #138 Pipe constructor DI | 56/56 PASS | `target/angular-pipe-di/20261013-115442-d4559f8d/` | None |
| #138b structured inject-token | 131/131 PASS | `target/angular-inject-token/20261013-115530-a2bf7d3e/` | None |
| #140 legacy NgRx Action declarations | 78 PASS, 7 OBSERVED, 0 FAIL | `target/ngrx-legacy-declarations/20261013-115638-748c175c/` | None; observations are outside this declaration-only contract |
| #140b-Prereq2 Action/type binding preservation | 27/27 PASS | `target/ngrx-action-type-binding/20261013-115803-cded1271/` | None |
| #140b legacy switch transitions | 69/69 PASS | `target/ngrx-legacy-transitions/20261013-115907-bb17dedd/` | None |
| #140 Slice 1 legacy discriminant authority | 80/80 PASS | `target/ngrx-legacy-transitions/20261014-113252-c8a075b4/` | None |
| #140c constructor dispatch | 88/88 PASS | `target/ngrx-constructor-dispatch/20261013-123224-ed46b446/` | None |
| #140d Service dispatch ownership | 84/84 PASS | `target/ngrx-service-dispatch/20261013-123804-97dfeaa6/` | None |
| #140e Service select ownership | 52/52 PASS | `target/ngrx-service-select/20261014-101053-b72f19dd/` | None |
| #140 Slice 2 direct-relative legacy object dispatch | 11/11 PASS | `target/ngrx-legacy-object-dispatch/20261014-130121-c2046049/` | None |
| #140 frozen replacement recall benchmark | All relations PASS: Dispatches 20/20, Selects 21/21, HandlesAction 22/22, TriggersReducer 21/21 | `target/ngrx-recall-benchmark/20261015-040208-69239c88/` | None; one additional real Selects relationship is outside the frozen oracle and retained for manual review |
| Invocation-evidence live MCP acceptance | 58/59 PASS | `target/invocation-evidence-verification/20261013-124145-6d1f1e80/` | LGR-003 |
| Batched `provide_code_context` live acceptance | PASS | `target/provide-context-batch-verification/captures/verification-report.json` | None |
| Batched `workspace_query` live acceptance | PASS | `target/workspace-query-batch-verification/captures/verification-report.json` | None |
| `entities_in_file` fidelity and hard-link live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| Dependency-cycle semantics and witness live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| Workspace identity resolution and ambiguity live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| C# constructor dependency lifecycle live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| Structured C# base-type live acceptance | PASS | `target/live-acceptance/csharp-base-types/responses.json` | None |
| Angular template ownership/linkage live acceptance | PASS | `target/live-acceptance/angular-template-linkage/responses.json` | None |
| Sparse `workspace_query` discovery diagnostics live acceptance | PASS | Operator-reported harness output; notes were informational and this driver does not persist captures | None |
| Shared signature production live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| Typed method-flags projection live acceptance | PASS | Operator-reported harness output; this driver does not persist captures | None |
| Phase 9 durable-context lifecycle live acceptance | PASS | `target/phase9-verification/state/` | None |
| Consolidated native issue reconciliation | 6/9 PASS | `target/issue-reconciliation/20261013-132835-8d74d247/` | LGR-003, LGR-004, LGR-005 |
| Registered workspace-query capture and deterministic verification | PASS | `target/workspace-query-verification/captures/` | None |

## Completion rule

After the remaining live gates finish:

1. Group open entries by production ownership and dependency order.
2. Confirm the intended behavior and any architectural decision points before
   implementation.
3. Add each required tracked regression under `src/tests/**` before claiming a
   repair.
4. Implement the production fixes without weakening the live assertions.
5. Rerun the focused live gates and report them separately from Rust/CI tests.
6. Run the repository's authoritative final verification gate.
