# #140b legacy reducer/handler authority gate

**#140b LEGACY NGRX HANDLER-ACTION LINKAGE SLICE BLOCKED**

Investigation base: `feature/current-cbm-pinned-upgrade`, `142b75c`.
This report records current code inspection, not new test or live MCP evidence.
All frozen slices remain untouched. No production repair has been implemented.

## Proven syntax boundary

The repository-established legacy form is:

```ts
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set';
export function auditReducer(state: any, action: Action) {
  switch (action.type) {
    case SET_AUDIT: return state;
    default: return state;
  }
}
```

PROVEN BY CODE INSPECTION: this exact form is in
`verification/issue-reconciliation/Invoke-NativeIssueReconciliation.ps1`.
The #140a `legacy-switch` observation uses the same form with an imported
`SET_AUDIT` constant. The checked-in `src/test_files/angular/ngrx/user.reducer.ts`
and NgRx producer regression fixtures use modern `createReducer` / `on`.
No existing inspected fixture or producer contract establishes a legacy class
handler using `LoadAudit.TYPE`, `LoadAudit.type`, or `instanceof LoadAudit`.
These are illustrative forms in the handoff, not approved repository contracts.

## First-loss trace

| Stage | Classification | Current-code evidence |
| --- | --- | --- |
| Legacy function and switch source | PRESENT | Existing operator fixture above |
| Generic function owner evidence | PRESENT | `TS_QUERY` has `func.root`; `TS_CALL_QUERY` captures function declaration/name |
| Structured switch receiver, case operand and exact owner association | MISSING | Current TS queries do not capture switch/case roles; legacy query only covers imported Action/class declarations |
| Legacy NgRx reducer extraction | MISSING | `ngrx/extract.rs::extract_reducer` recognizes modern `createReducer`/`on` only |
| Structured legacy handler identity | MISSING | No legacy reducer record is produced |
| Action constant declaration authority | UNSUPPORTED | #140a declares classes implementing directly imported Action, not standalone constants |
| Constant-to-class discriminant resolution | NOT ESTABLISHED | #140a records class/import/implements spans, not an authoritative event-value binding |
| Legacy semantic edge | MISSING | `NgRxShape::to_ngrx_semantic_edges` projects modern reducer transitions only |
| WorkspaceIndex/public linkage | MISSING UPSTREAM | Index cannot consume an authoritative legacy edge that is never produced |

All rows are PROVEN BY CODE INSPECTION except explicit NOT ESTABLISHED authority.
The first missing handler evidence is at structured switch-role capture. Fixing
that alone would not establish Action declaration resolution for the proven
legacy form. Its independent constant declaration prerequisite is outside scope.
Parser inability is NOT ESTABLISHED: missing query captures do not imply that
Tree-sitter cannot parse the syntax.

## Handler and Action authority

Generic function declaration/name evidence is reusable; a second owner system
is unnecessary. However, the currently proven discriminant is a constant
reference, not a reference to a #140a class Action. Neither proximity, a reducer
name, an Action suffix, nor equal `type` string values proves a binding.

Adding an unreferenced class implementing Action beside `SET_AUDIT` would make
an Action declaration observable but would not make that class the referenced
Action. Such a fixture would fail the handoff's independent reference oracle.
No invalid primary RED was registered or run.

## Existing relationship contract

PROVEN BY CODE INSPECTION:

- `SemanticRelation::TriggersReducer` means an Action triggers a reducer state
  transition. Current projection direction is **Action → Reducer**.
- `SemanticRelation::HandlesAction` is documented as an **Effect → Action**
  relation and current NgRx projection uses that contract.
- `src/tests/angular_meta/semantic_ngrx.rs` contains a tracked index assertion
  for outgoing `TriggersReducer` on an Action. It was inspected, not rerun.

A truthful reducer association could use existing `TriggersReducer` and be
inspected through reverse edges from the reducer. It must not be mislabeled as
an outgoing handler → Action edge. Reusing `HandlesAction` for a Reducer would
expand its documented public meaning. No relation or schema was changed.

## Declaration independence, linkage and collision controls

#140a production remains untouched: `legacy::collect` supplies independent
class declaration evidence; `legacy::include` adds `class_actions`; self-Defines
projection publishes those declarations through existing index normalization.
This is current-code inspection, not a new preservation run.

No exact handler → Action linkage has been established for this slice.
Neighbor, same-name and equal-discriminant cases must reject inferred bindings;
no new runtime evidence is claimed for them.

| Required matrix row | RED | GREEN | Handler oracle | Action oracle | MCP |
| --- | --- | --- | --- | --- | --- |
| Primary supported legacy handler | Not run | Not run | Function exists in fixture; NgRx handler absent | Referenced constant authority out of scope | Not run |
| Neighbor Action | Not run | Not run | No admissible class binding | Equal value/name insufficient | Not run |
| Unreferenced class Action | Not run | Not run | Must not manufacture linkage | #140a independent contract frozen | Not run |
| Handler without supported reference | Not run | Not run | No linkage admissible | Declaration independence preserved by unchanged code | Not run |
| Same-name/collision control | Not run | Not run | Exact source binding required | No suffix/simple-name resolution accepted | Not run |

## Lifecycle, compatibility and preservation

No lifecycle runtime was executed, no fresh preservation tests were run, and no
GREEN or CI claim is made. Existing #140a evidence remains historical evidence
for that completed slice, not new #140b verification.
Producer generations remain marker **4**, semantic **6**. Canonical generation
is unchanged. No durable meaning changed, so no compatibility bump is needed.

## Owner decision required

There are two bounded ways to establish a valid primary fixture:

1. Provide a repository-supported same-file class-reference handler form with
   its discriminant contract and exact expected Action binding. Structured
   owner/reference evidence can then be added locally using the existing parse.
2. Separately authorize standalone constant/discriminant declaration authority
   for the actually documented `switch(action.type) / case SET_AUDIT` family.
   This is explicitly excluded from the present handoff and requires its own
   declaration-authority design before handler linkage.

Recommendation: supply an actual class-reference reducer fixture if #140b must
remain class-only. Do not invent syntax or a constant-to-class equivalence.
Use existing Action → TriggersReducer → Reducer semantics and reverse-query
acceptance where that faithfully represents the intended result. If an outgoing
handler → Action relation is mandatory, its public contract needs owner review.

## Remaining #140 work and incidental effects

Standalone string/enum/object authority; other legacy reducer forms;
constructor dispatch `new`; qualified/module binding; aliases/namespaces/barrels;
service dispatch/select ownership; Selector publication; Effects criteria;
representative recall and cold/warm behavior remain unresolved separate families.
No discovery, hydration, cache, CBM, LSP, template or query-history code changed.
No incidental functional improvements are claimed.

## Changed files, commands and artifacts

Only this investigation report was added. Inspection used bounded `rg`, `cat`,
`sed`, `git status`, and `git log`, plus reading the uploaded handoff. One initial
search named nonexistent paths; subsequent `rg --files` located the real
`src/queries.rs` and `src/test_files/angular/ngrx` paths. No inference relies on
that incomplete search.

No Cargo tests, Clippy, build, binaries, PowerShell harness, live CBM, or model
host were started. The handoff explicitly disallows carrying runtime permission
forward, and the identified stop boundary does not warrant requesting runs yet.
The full repository suite remains CI-only. This report is not regression
coverage, RED/GREEN evidence, or a reusable verification harness.

Stop at the declaration/discriminant authority boundary pending owner review.
