# Resumed #140b: source-bound legacy switch transitions

**#140b LEGACY NGRX SWITCH/CASE TRIGGERSREDUCER SLICE COMPLETE**

Base `3593d2a`, branch `feature/current-cbm-pinned-upgrade`.
The earlier two BLOCKED authority reports remain unchanged historical evidence.
Prereq2 now supplies the class-side source binding; this slice consumes it rather
than retroactively pretending standalone constants had declaration authority.

## 43.1 Proven source form

```ts
import { Action } from '@ngrx/store';
export const SET_AUDIT = '[Audit] Set Audit';
export class AuditAction implements Action {
  readonly type = SET_AUDIT;
}
export function auditReducer(state: any, action: Action) {
  switch (action.type) {
    case SET_AUDIT: return state;
    default: return state;
  }
}
```

Supported owner is a program-level function declaration, exported or not, with
exactly two plain required identifier parameters and no default values. The second
parameter has a plain `Action` annotation from the same NgRx-import authority as
#140a; the first is the state position. The switch must be an immediate statement
of this function body; its expression must be a nonoptional identifier-receiver
`.type` member. That receiver must bind to the second parameter. Cases must be
immediate switch-body clauses with direct identifier operands. Neither function
nor parameter spelling is prescribed. Arbitrary-name/message-parameter producer
control proves that role, not naming convention, owns the fact.

The source form is the established `auditReducer(state: any, action: Action)`
fixture augmented by the explicitly authorized Prereq2 class association. The
original unassociated constant-only switch family remains separate. Optional,
defaulted, destructured, nested, arrow/method owners, different parameter types,
qualified/literal/compound operands and broader lexical resolution are not claimed.
Declared for-in/of bindings anywhere in the source refuse the legacy transition
producer; modern NgRx and independent Action publication are unaffected.

## 43.2 First-loss trace

| Stage | Before | Current | Evidence |
| --- | --- | --- | --- |
| Independent #140a class identity | PRESENT | Preserved | Primary prerequisite assertions; preservation filter |
| Prereq2 Action/type constant association | PRESENT | Consumed privately | Primary prerequisite assertion; seven preservation tests |
| Canonical function declaration/name | PRESENT | Reused by exact span | Existing invocation declaration captures |
| Function parameter/switch/case roles | MISSING | Captured in one structural match | New NgRx-local supplemental query |
| Case identifier declaration resolution | NOT ESTABLISHED for transition | Bounded unambiguous program const identity | Producer scope/value controls and audit RED/GREEN |
| Action/type-binding identity join | MISSING | File/hash/declaration/name spans | Producer exact-span and stale-file/hash tests |
| TriggersReducer projection | MISSING | Existing Action → Reducer relation | Unchanged public primary RED→GREEN |
| Durable semantic publication/index | Existing infrastructure PRESENT | New edge travels that path | Production trace and exact public queries |
| External MCP surface | Edge absent | Exact edge observable | Registered MCP tests; live evidence recorded below |

PROVEN BY CODE INSPECTION for the call path and scope mechanics; PROVEN BY TEST
for executed regressions/controls. Live MCP evidence is classified separately.
No documentation-only assertion is promoted to runtime proof.

## 43.3 Reducer identity authority

The new query associates the switch, typed parameter and case directly with its
actual top-level function node. Canonical declared names come from the existing
`invoke.declaration.callable` / `invoke.declaration_name` match, joined by exact
function span. No nearest function, filename, reducer suffix or marker parsing.
Reducer publication is an endpoint of the proven relation. This slice does not
add independent publication for functions without an admitted transition.

## 43.4 Switch-role authority

The structural query requires the two-parameter function shape and a direct-body
switch. A plain type identifier `Action` is required on parameter two. Imported
Action authority remains #140a's prerequisite. Function-local type parameters
named Action refuse interpretation. Receiver/property captures must correspond to
that parameter and `type`. Duplicate/shadowing declaration spans inside the owner
refuse the parameter association, including conservative nested-scope refusal.
Optional receiver chains and unrelated `state.mode` switches cannot qualify.

This is static source evidence, not compiler type-checking, runtime NgRx registration
or proof that an Action variable has never been reassigned. No general data-flow
or TypeScript compiler resolver was introduced.

## 43.5 Case-reference authority

Prereq2's binding exists only for a program-level const with unambiguous supported
binding evidence. The consumer independently obtains the case identifier's unique
supported declaration-name span and verifies that this is a program const name;
its enclosing function has no admitted competing local declaration of that name.
The direct-body scope is essential: no nested block/callable cases are admitted.
Unsupported source binding patterns already refuse Prereq2 evidence.

Final audit found a missing declaration shape: `for(var SET_AUDIT of values)` can
hoist a local token shadow without a `variable_declarator` node. A new permanent
regression proved that the outer class binding remains valid while the case must
not resolve to it. It failed before the refusal (10 pass/1 fail), then passed
unchanged (11/11). The new domain-local query captures declared for-in/of left
patterns and the consumer refuses that source form. It does not modify the frozen
class/type binder or invent broader lexical resolution. This conservative file
refusal also excludes otherwise harmless for-in/of declarations; that limit is
explicit, not a silent promise of generic scope support.

## 43.6 Join authority

After resolving the case, the join key is:

```text
canonical file + exact source hash
+ const declaration span + const name span
```

The consumer builds an identity index of Prereq2 records matching the current file
and source hash, then queries by the resolved declaration/name spans. Identifier
spelling selects a declaration inside the proved scope; it is never the Action
join key. String values never participate. Same-value OTHER_AUDIT resolves to its
own declaration and cannot select AuditAction bound to SET_AUDIT.

A constant shared by multiple admitted Actions has no unique inverse Action
endpoint. Private associations remain intact, but transition projection refuses
that ambiguity instead of picking the first Action or merging identities.

## 43.7 Semantic projection and complete production lifecycle

```text
IRCompiler::compile_inner sets canonical source path
→ default-production CoreIRPass, existing single capture walk
→ unchanged #140a and Prereq2 producers
→ NgRx transitions::collect returns compilation-owned proof records
→ PassContext.ngrx_legacy_transitions
→ MetaLayerPass borrows records into MetaLayerContext
→ Angular semantic evaluate, gated by ngrx.enabled
→ LegacyTransition::edge: Action → TriggersReducer → Reducer
→ inference/CompiledIR semantic result
→ existing atomic canonical/semantic persistence and WorkspaceIndex publication
→ workspace_query forward_edges / reverse_edges
```

Both endpoint files are explicitly the canonical asserting source file. No new
public relation, standalone discriminant entity, call evidence, marker or synthetic
switch-owner identity. HandlesAction remains the unchanged Effect → Action contract.
The shared core only invokes a NgRx-local collector and carries typed records; it
does not interpret switches/cases. WorkspaceIndex code is unchanged.

Private bindings are consumed before their context is dropped. Durable products
contain the resulting existing relation, not the binding record. Cache hits reuse
compatible durable edges; they do not need to reconstruct private evidence. Existing
compatibility validation checks semantic producer identity before accepting a
combined durable context. No discovery, hydration or cache behavior was changed.

## 43.8 Falsification matrix and RED/GREEN

Primary registered file `src/tests/mcp/ngrx_legacy_transition.rs` is unchanged.
Independent source AST, #140a Action, Prereq2 binding, canonical function and direct
case prerequisites all pass before the public edge assertion. The unfixed parent
returns zero TriggersReducer edges: **0 pass/1 fail**. Exact source/oracle was
stashed during implementation and restored byte-for-byte: **1/1 GREEN**.
Final expanded filter selects **11 tests**, all pass.

| Case | Result / oracle |
| --- | --- |
| Primary case SET_AUDIT | AuditAction → TriggersReducer → auditReducer, exact files |
| OTHER_AUDIT without its own Action | No AuditAction edge |
| Equal-valued OTHER_AUDIT | No value-based AuditAction edge |
| Neighbor OtherAction bound to OTHER_AUDIT | Only OtherAction edge |
| Literal equal to Action value | No declaration-identity transition |
| Unbound UNUSED_AUDIT constant | No fabricated Action or edge |
| Wrapper/member/conditional operand | No direct identifier transition |
| Empty/default-only switch | No default-derived Action |
| Unrelated switch/state property/optional chain | No transition |
| Untyped/wrong type/generic Action shadow | No typed Action receiver authority |
| Local token/receiver shadow, nested owner/switch | Refused, no outer binding guess |
| Hoisted for-in/of shadow | Additional unchanged audit RED→GREEN refusal |
| Shared discriminant bound to two Actions | Refused ambiguity |
| Multiple distinct supported cases | Each exact admitted incoming Action |
| Duplicate same Action case | Two private case spans, one indexed file/triple edge |
| Identical names/source in two files | Two file-provenanced edge occurrences; no cross-file join |
| Stale hash/wrong canonical file in private evidence | No edge |
| NgRx disabled | No new semantic contribution |

The first compatibility test incorrectly expected a semantic-only generation bump
to change canonical identity. Inspection established the existing separate
identities; the new assertion was corrected to canonical equality and semantic
inequality. That failure was not counted as semantic RED and the public primary
was not changed. An earlier compile error from passing &&str to EntityRef::new was
corrected; it is not RED evidence.

## 43.9 Public MCP evidence and occurrence contract

Registered MCP tests assert exact domain/type/name/file for both endpoints and
relation/direction, and require reducer reverse queries to expose the same edge as
Action forward queries. Independent class declarations remain reachable in all
negative cases. Constants are not published as Actions.

The existing graph base identity is domain/type/name; this task does not introduce
public span identities or module resolution. Source-distinct same names retain
asserting-file edge occurrences. Duplicate cases in one source produce one existing
file/triple occurrence, not one public occurrence per case. No new case-span schema
or multiplicity contract is invented.

Live PowerShell harness uses established `PhaseBSession.ps1`, isolated Git roots,
provider-disabled configurations and owned stdio sessions. Twenty static forms at
High/Medium/Low, three same-name/file groups and six refresh lifecycle stages give
**69/69 operator groups PASS** on the corrected candidate.
No Codex/Claude model-host pilot or UI consumption claim.

## 43.10 Lifecycle

Registered MCP tests replace SET_AUDIT case with OTHER_AUDIT, then literal, then
no case, then remove reducer, checking old edges disappear and class declarations
remain. Repeated High/Medium/Low requests do not strengthen the tested fact.
Producer tests verify fresh compilation and stale private file/hash refusal.

The live harness additionally deletes the entire file. These are native lifecycle
controls, not #142 acceptance or general query-history/discovery completeness.

## 43.11 Compatibility/generation

Angular/NgRx semantic generation **6 → 7** excludes previously missing-transition
semantic products. Marker generation remains **4** because no marker changed.
Canonical pipeline generation remains **1**. Dedicated compatibility regression
proves prior semantic generation differs while canonical producer/config identities
remain equal. Existing durable-restore semantic generation rejection also passes.
No ad hoc cache-dependent truth or new persisted private record.

## 43.12 Preservation

**133 selected preservation tests pass**:

| Filter | Passed |
| --- | ---: |
| ngrx_action_type_binding_ | 7 |
| ngrx_legacy_declaration_ | 9 |
| framework_declaration_publication_ | 10 |
| decorated_owner_identity_ | 9 |
| angular_di_pipe_ | 9 |
| angular_di_inject_token_ | 7 |
| angular_inject_evidence_ | 4 |
| angular_meta::ngrx::tests:: | 25 |
| angular_meta::semantic_tests:: | 18 |
| angular_meta::decorators::tests:: | 28 |
| angular_template_compatibility_tests | 1 |
| workspace_query_angular_selector_occurrence_tests | 5 |
| restore_rejects_semantic_producer_generation_mismatch | 1 |

These ran before the final declared-loop refusal; final 11-case transition matrix,
Clippy and corrected candidate build/live verification follow that audit fix.
No frozen #136/#137/#135 or CBM code was changed; no fresh unrelated test result is
claimed. WorkspaceIndex is untouched; exact duplicate/file tests exercise its
existing behavior. Full suite remains CI-only.

## 43.13 Remaining #140 work

Unsupported/defaulted/nested/arrow legacy reducer forms, general lexical scopes,
standalone discriminant publication, constructor dispatch new, qualified/literal
case authority, imports/aliases/namespaces/barrels/module binding, service ownership,
Selector publication, Effects criteria, representative recall and cold/warm
acceptance remain separate. Original constant-only switch sources without class
association are not repaired by this slice. No #141/#142 work began.

## 43.14 Incidental effects

Additional source-bound relationships make admitted reducer endpoints reachable.
This is not independent publication of every legacy function, improved constructor
dispatch, general declaration/use resolution or representative whole-issue recall.
No incidental outcome is promoted into another acceptance family.

## 43.15 Changed files

- `angular_meta/ngrx/transitions.rs`: domain-local query, scope refusal, declaration
  identity join, private proof and existing relation projection; test registration.
- `angular_meta/ngrx.rs`: module registration only.
- `angular_meta/ngrx/type_binding.rs`: comment aligns with the now-present consumer;
  frozen producer code/schema/contract unchanged.
- `ir/pipeline.rs`: compilation-local transition vector/default ownership.
- `ir/pipeline/core.rs`: append query, invoke collector, strip private captures.
- `ir/pipeline/meta_layer.rs`, `layers/meta/context.rs`: borrowed proof propagation.
- `layers/meta/angular_semantic.rs`: gated semantic projection consumption.
- `layers/meta/mod.rs`: existing semantic generation bump only.
- `src/tests/mcp/ngrx_legacy_transition.rs`: unchanged public primary regression.
- `src/tests/angular_meta/ngrx_legacy_transition.rs`: source identity/receiver/case
  matrix, stale evidence and additional loop-shadow RED/GREEN.
- `src/tests/mcp/ngrx_legacy_transition_matrix.rs`: exact reverse/forward identity,
  feature-off, refresh, duplicate and same-name/file occurrence tests.
- `src/tests/mcp/ngrx_legacy_transition_compatibility.rs`: generation compatibility.
- `mcp/mod.rs`, `mcp/compatibility/mod.rs`: normal test registrations.
- Existing template/selector compatibility fixtures: current semantic generation
  expectation 7; behavior assertions retained.
- `verification/issue-reconciliation/Invoke-NgrxLegacyTransitions.ps1`: reusable
  exact-edge/falsification/lifecycle driver using established helper pattern.
- `verification/issue-reconciliation/README.md`: command and scope/observability.
- This report: exact contract, evidence, limitations and stop boundary.

## 43.16 Commands and artifacts

Fresh current-task authorization received for focused filters, preservation,
focused rustfmt, package Clippy, one build and provider-disabled live MCP harness.
An additional corrected-binary build was separately authorized after the scope audit fix.

```bash
cargo test --locked -p clean-ctx --lib --all-features -j 4 ngrx_legacy_transition_
cargo test --locked -p clean-ctx --lib --all-features -j 4 <preservation filters above>
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4
```

Rust environment uses `/workspace/.tooling/cargo/bin`, RUSTUP_HOME
`/workspace/.tooling/rustup`, CARGO_HOME `/workspace/.tooling/cargo`. Focused rustfmt
used edition 2024/skip_children=true; the observed public RED was not reformatted.
The additional scope regression/matrix was stashed intact while its guard was
implemented and restored unchanged. The initial all-feature build passed before
that guard, so it is not the final candidate evidence.

Final audit matrix: **11 passed, 0 failed**; final Clippy: exit **0**, warnings
denied. Authorized corrected candidate rebuild: exit **0**. Actual binary/stdio
live MCP harness: **69/69 PASS**, exit **0**. The full suite was not run.

Artifact root: `target/ngrx-legacy-transitions/20261012-171221-4979d80b`. It contains source oracles, binary provenance, raw protocol
captures, exact identities/relations in named forward/reverse captures, all verdicts,
and copied primary RED/GREEN, matrix, loop-shadow RED/GREEN, fourteen focused
preservation logs/results, initial/final lint/build and corrected live logs.

Candidate binary SHA256: `c9429e453150f5cd3832ae84f8e4ace2ac22c518061b323d35f3808707d17f8c`.
Unchanged public primary SHA256: `8322adffaec9bbf5339f9cb10220ff156d6b021bc9d304682ef32ae78270ba2a`.
Unchanged scope-regression/matrix SHA256: `2347614bdfbc7a7502b9395164ea8d08788b6557adc5eb1d0166db99837fc5b9`.

PowerShell command executed:

```bash
XDG_CACHE_HOME=/workspace/.tooling/powershell-cache XDG_CONFIG_HOME=/workspace/.tooling/powershell-config XDG_DATA_HOME=/workspace/.tooling/powershell-data /workspace/.tooling/powershell/opt/microsoft/powershell/7/pwsh -NoProfile -File verification/issue-reconciliation/Invoke-NgrxLegacyTransitions.ps1 -BinaryPath /workspace/Clean-CTX/target/debug/clean-ctx
```

Twenty static forms × three fidelities = 60 live groups; same-name file provenance
adds 3; six lifecycle stages add 6. Verdict JSON confirms 69 unique groups, all PASS.
The loop audit regression and original public RED are verified byte-for-byte against
their stashed blobs. Strict UTF-8/no BOM, active-file ceiling and git diff checks
pass. All owned harness sessions were closed; no owned stash remains.
No full suite, live CBM, model host or unrelated production repair was run.

STOP after owner-reviewed completion of this direct-identifier transition path.
No automatic follow-on production slice.
