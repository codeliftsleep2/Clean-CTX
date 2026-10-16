# #140c legacy NgRx constructor dispatch repair

**#140c LEGACY NGRX CONSTRUCTOR DISPATCH ACTION LINKAGE PARTIALLY COMPLETE**

The bounded same-file form is implemented. The historical imported constructor
form needs a separate module/declaration-resolution boundary. Do not close #140
or proceed into that boundary automatically.

## 41.1 Existing dispatch semantic contract

**PROVEN BY CODE INSPECTION:** `NgRxShape::to_semantic_edges` in
`src/angular_meta/ngrx/shape.rs` uses
`angular/Component/<component> → Dispatches → ngrx/Action/<action>`.
Without a component name its existing owner is `angular/DispatchSite/store`.
This repair reuses that projection, source-file attachment, WorkspaceIndex and
forward/reverse query contracts. It adds no public entity or relation kind.
An arbitrary Service is not substituted as owner. Source occurrences distinguish
same-named files; repeated identical edges in one file remain deduplicated.

## 41.2 Proven source form

```typescript
import {Component} from '@angular/core';
import {Action, Store} from '@ngrx/store';
export class AuditAction implements Action {readonly type = 'audit';}
@Component({selector: 'audit-dispatch', template: ''})
export class Dispatcher {
  constructor(private store: Store<unknown>) {}
  run() {this.store.dispatch(new AuditAction());}
}
```

Supported: top-level canonical class, unique unaliased NgRx Store import,
constructor accessibility-modified generic `Store<T>` parameter property,
matching `this.<property>.dispatch`, immediate method-body expression statement,
one direct `new Identifier(...)` argument, uniquely bound same-file independently
admitted Action class. A different receiving property name and constructor
payload expressions are accepted. An instance Store property does not qualify a
static method. Optional calls/receivers are not admitted.

## 41.3 First-loss trace

| Stage | Before | Current evidence |
| --- | --- | --- |
| Ordinary dispatch invocation and argument span | PRESENT | Native invocation evidence retained, one parse |
| Store qualification for constructor path | MISSING | Closed NgRx import/property/type/owner proof |
| Direct argument/new role | MISSING in semantic producer | Domain AST captures joined to existing exact argument span |
| Constructor endpoint | MISSING; raw first token `new` | Direct identifier role captured structurally |
| Same-file declaration resolution | MISSING | Unique binding-name span equals admitted class declaration name span |
| Independent #140a Action identity | PRESENT | Existing class admission unchanged |
| Dispatch projection | PRESENT, wrong lexical endpoint | Existing projection consumes private qualified records |
| WorkspaceIndex/public boundary | PRESENT, delivered `new` | Exact resolved Action and file delivered both ways |
| Imported historical endpoint resolution | UNSUPPORTED | Separate boundary; no basename/suffix/import guesses |

**PROVEN BY TEST:** permanent primary independently checks syntax, class and
Store/property/direct argument structure and #140a admission before testing the
MCP response. RED was one assertion failure, actual `new` versus `AuditAction`;
GREEN passes the identical file, SHA-256
`1a1e35a821bb711c147185c5abd0ea3249ba51abe4220aeaeb242a8c9dae4d5d`.
Regression/registration were stashed during implementation and restored unchanged.

**PROVEN BY CODE INSPECTION:** first lost constructor semantics were in
`extract_selectors::dispatch_action_name`, reducing `new AuditAction()` to `new`.
The source-only raw extraction helper remains a lexical helper; the default
parsed production pipeline supplies the new private side channel. That raw
helper alone is not an independent proof of qualified dispatch semantics.

**PROVEN BY LIVE MCP:** see the execution record below for actual binary results.
No claim about model consumption, real-project recall or cold/warm acceptance.

## 41.4 Dispatch qualification authority

`constructors::collect` requires the NgRx literal module import, unique Store
binding, exact constructor parameter-property span, generic Store type, actual
containing top-level canonical class, matching `this` receiver and own immediate
method-body call. Duplicate/shadowed Store bindings, conflicting member names,
static methods and optional calls fail closed. No suffix, receiving field name,
nearby text or method name alone establishes Store authority.

## 41.5 Constructor structural authority

Domain query captures the direct first new-expression argument. Positive admission
additionally requires exactly one native argument with the identical new span.
Wrappers, conditional/array arguments and descendant constructors are not
promoted. Every direct constructor position is recorded even if resolution fails,
so its exact old lexical site can be suppressed without inventing an Action/new.
Other modern sites are preserved by source offset, never removed by token name.

## 41.6 Declaration-resolution authority

Name binding captures are reused from the frozen prerequisite; no shared binder
changes. An endpoint must have exactly one binding-name span, equal to the class
name span belonging to an already admitted Action class. Records retain exact
class span, canonical file and source hash. No canonical file means no resolution;
stale/foreign-file records cannot add a resolved reference. Locals, parameters,
imports, class/type bindings and for-in/of declarations conservatively refuse
ambiguous source. Unsupported destructuring/errors/escaped bindings fail closed.

Supported scope is **same-file only**. Direct imports, aliases, qualified names,
barrels and cross-file resolution are not established. The historical
`ClassConsumer` fixture imports `NoSuffixRecord` from `./primary` and uses
non-generic `Store`; it remains outside this closed form. Its earlier live
Action/new observation is historical evidence, not a current accepted link.

## 41.7 Action authority

#140a owns Action admission unchanged. Constructor usage never creates an Action
class. Unused AuditAction/OtherAction declarations remain independently reachable;
NonAction is refused even when directly constructed at a qualified dispatch.
Equal discriminant values do not merge different admitted class identities.

## 41.8 Semantic projection

CoreIRPass collects NgRx-owned records in the existing TypeScript capture parse
and strips all private captures before canonical emission. PassContext owns the
records; MetaLayerContext borrows them. Angular semantic evaluation applies
constructor records to the existing NgRxShape once, and both marker/semantic
projections consume that shape.

**PROVEN BY CODE INSPECTION:** `IRCompiler::compile_inner` creates a fresh
PassContext and runs CoreIRPass before MetaLayerPass, then drains produced
semantic edges. `tool_handlers/core/provide/evaluate.rs` reuses cached IR only
with the corresponding already-produced `state.semantic_edges`; otherwise its
fallback calls `compile_file_ir_focused`, rebuilding private records before
projection. `hydration/publication.rs` either keeps a current semantic projection
or recompiles source through that same compiler; it does not rerun NgRx
interpretation on a bare cached IR lacking private evidence. Durable restore
validates source/config/producer compatibility before publishing stored semantic
products. The new generation excludes prior constructor-unaware products.
No private record is assumed to survive persistence. Same-source/same-fidelity
live repeat checks observe unchanged exact results, not parser-hit telemetry or
#142 acceptance.

Exact raw dispatch starts are internal extraction
provenance, not public schema. Empty shapes after suppression do not manufacture
an NgRx result. No resolver or durable public invocation graph was introduced.

## 41.9 Falsification matrix

Registered tests cover correct payload/receiver rename, NonAction, wrong Store
module/type/receiver, method/local/loop endpoint shadowing, qualified constructor,
duplicate Store properties/member conflicts, optional/static calls, wrapper,
conditional/array/nested new, bare symbol/factory preservation, same-file exact
neighbor identity, foreign-file application refusal, source hash and private
capture stripping/one parse. Imported constructor is explicitly unresolved.
The live harness additionally checks opt-out, repeated edges, same names in two
files, three fidelities and refresh/delete lifecycle. Preserved symbolic modern
endpoints (for example wrapper/flag) are reported as existing relationship
behavior; they are not promoted to constructor-resolved class declarations.

## 41.10 Public MCP evidence

Harness asserts exact `angular/Component/Dispatcher → Dispatches →
ngrx/Action/AuditAction`, both endpoint files, one forward edge and the same
reverse edge. Neighbor uses OtherAction; NonAction/qualified/import/shadow forms
must not substitute AuditAction or `new`. The fallback-owner control explicitly
asserts `angular/DispatchSite/store`, not Service ownership. Independent Action
source occurrences and source-byte preservation are checked separately.

## 41.11 Duplicate/occurrence behavior

Two identical source dispatches retain the existing one-file edge deduplication.
Two source files with the same public owner/action names retain separate
file-provenanced edges and declarations. No public per-call span identity is
claimed. Domain/type/name graph identity is unchanged.

## 41.12 Lifecycle

The harness owns a single session across initial AuditAction, change to
OtherAction, NonAction, call removal, owner removal and source deletion.
Each stage calls `refresh_workspace` and checks current forward/reverse results
and independent declaration sets. These are refresh controls, not #142
query-history or cold/warm acceptance.

## 41.13 Compatibility/generation

Angular marker generation **4 → 5** and semantic generation **7 → 8** invalidate
prior products containing lexical constructor endpoints or their markers.
Canonical pipeline generation remains 1; config and public schema unchanged.
Constructor compatibility test proves both producer signatures differ. Existing
current-cohort Angular-template, selector-occurrence and legacy-transition
fixtures track the new current generation; their behavioral oracles remain.
Existing durable semantic-generation mismatch rejection is tested.

## 41.14 Preservation

Focused filters: legacy transitions 11, action type binding 7, legacy declarations
9, framework publication 10, decorated owner identity 9, Pipe DI 9, inject-token
7, inject evidence 4, NgRx module 25, Angular semantic 18, decorators 28,
template compatibility 1, selector occurrences 5 and restore generation rejection
1. **144 preservation tests passed**, in addition to six focused constructor tests.
Optional-call and static-owner audit tests observed behavioral failures before
their structural fixes; the final focused set passed unchanged.

## 41.15 Remaining #140 work

Imported/alias/namespace/barrel constructor binding; other Store injection forms;
control-flow/nested statement constructor admission; Service-level ownership;
standalone discriminant publication; unsupported reducers; Selector and Effects
criteria; representative recall and cold/warm acceptance remain separately scoped.
No #140 closure recommendation. #141/#142 and pinned CBM are untouched.

## 41.16 Incidental effects

Fail-closed direct constructors remove the old phantom Action/new. Existing
modern unresolved symbolic endpoints remain as previously projected. Neither
cleanup nor incidental reachability proves module resolution, general functional
DI or a new service dispatch authority.

## 41.17 Changed files

- `ngrx/constructors.rs`: closed query, qualification/resolution and shape adapter.
- `ngrx/extract/orchestration.rs`: relocate existing extraction composition below
  the 615-line policy and retain private call-start provenance.
- `ngrx/extract.rs`, `extract_selectors.rs`, `ngrx.rs`: decomposition/reexports and
  raw dispatch starts; modern extraction algorithms unchanged.
- `ir/pipeline.rs`, `pipeline/core.rs`, `pipeline/meta_layer.rs`: private owned
  records, one parse, stripping and borrowing.
- `layers/meta/context.rs`, `angular_semantic.rs`: borrow and consume records.
- `layers/meta/mod.rs`: marker/semantic generations only.
- Primary, matrix and compatibility test files and their normal registrations:
  immutable RED→GREEN oracle, authority/public/refusal matrix and generations.
- Three existing compatibility fixtures: update current producer cohort only.
- PowerShell harness, verification README and this report: reusable live boundary
  checks, invocation command, observed evidence and declared limitations.

## 41.18 Commands and artifacts

Candidate SHA-256:
`9196b7035bca5dc83cf6e628ccd2a3fa887560e67f2ba7768eacbcabe6f81b56`.
Evidence root:
`target/ngrx-constructor-dispatch/20261012-180108-495cd62a`.
Live verdicts: **88 PASS, 0 FAIL**, PowerShell process exit 0. The 26 static
forms each ran at High/Medium/Low (78), plus three file-occurrence groups and
six lifecycle stages and one repeated same-source/fidelity group. All requests, responses, sources, context projections,
source oracles, binary provenance and stderr are retained there.

Executed:

- `cargo test --locked -p clean-ctx --lib --all-features -j 4 ngrx_constructor_dispatch_`:
  initial primary RED 0/1, final focused GREEN 6/6. One invalid preservation
  oracle used an unsupported quoted object type; corrected to the existing
  supported modern creator form, not treated as production RED evidence.
- Optional-call audit in the six-test filter: RED 5 passed/1 failed; static-owner
  audit using `ngrx_constructor_dispatch_source_authority`: RED 0/1. Both
  unchanged audit cases are GREEN in the final 6/6 set.
- Same test command with the 14 preservation filters listed above: 144 passed;
  NgRx module 25 and Angular semantic 18 rerun after the final static fix, GREEN.
- Focused `rustfmt --edition 2024 --config skip_children=true` on changed Rust
  files except the immutable primary; strict UTF-8/no BOM, active code ≤615
  lines and `git diff --check` passed.
- `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`:
  exit 0 after final production audit changes.
- `cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`:
  one candidate build, exit 0, after final audit.
- Installed PowerShell `-NoProfile -File
  verification/issue-reconciliation/Invoke-NgrxConstructorDispatch.ps1
  -BinaryPath ./target/debug/clean-ctx`: exit 0, 88/88 live groups.

Rust commands used the existing `/workspace/.tooling/cargo` and rustup homes.
PowerShell used the existing installed binary and XDG directories. No dependency
or tooling installation was needed. Focused command logs are retained under the
evidence root's `focused-checks/`, including the valid RED failures.
Full repository suite remains CI-only. No live CBM, daemon termination, module
resolver, issue mutation, push or service ownership expansion was performed.
