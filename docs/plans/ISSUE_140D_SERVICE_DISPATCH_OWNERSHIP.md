# #140d NgRx Service dispatch ownership

**#140d NGRX SERVICE DISPATCH OWNERSHIP PARTIALLY COMPLETE**

The direct-method, same-file constructor path is implemented. Nested/callback
ownership and modern symbolic dispatch qualification remain separate authorities;
no inherited owner is guessed. #140 remains OPEN. Stop after this bounded slice.

## 30.1 Existing ownership contract

**PROVEN BY CODE INSPECTION:** frozen `NgRxShape::to_ngrx_semantic_edges` chooses
`angular/Component/<component> → Dispatches → ngrx/Action/<action>` when its
component scalar is present, otherwise `angular/DispatchSite/store → Dispatches
→ ngrx/Action/<action>`. Service publication is a separate Angular producer.
#140c already resolves qualified constructor endpoints and intentionally preserves
that fallback. Public triples retain file provenance; identical one-file edges
are deduplicated by the existing index, not assigned new per-call identities.

## 30.2 Proven Service source form

The owner approved `Store<unknown>` for this ownership-only primary. The supplied
plain-Store example did not satisfy frozen #140c's generic Store property proof;
plain-Store qualification remains a separate prerequisite, not silently repaired.

```typescript
import {Injectable} from '@angular/core';
import {Action, Store} from '@ngrx/store';
export class AuditAction implements Action {readonly type = 'audit';}
@Injectable()
export class AuditService {
  constructor(private store: Store<unknown>) {}
  run() {this.store.dispatch(new AuditAction());}
}
```

Supported: independently recognized canonical Service, own immediate method-body
expression statement, existing #140c qualified direct constructor dispatch and
same-file admitted Action. The existing generic Store parameter-property, import,
receiver, endpoint, shadowing and static/optional refusal policies remain frozen.

## 30.3 First-loss trace

| Stage | Before | Evidence |
| --- | --- | --- |
| Qualified dispatch and Action endpoint | PRESENT for Store<T> | #140c private record, independent primary checks |
| Exact invocation/declaration context | PRESENT | Native callable declaration and call spans |
| Containing own method/class syntax | PRESENT in AST; missing owner consumer | Domain same-match structural query |
| Canonical class identity | PRESENT | #139 class initializer record, exact class span and canonical consumer |
| Independent Service classification/publication | PRESENT | Existing decorator classification and Defines registration |
| Service-specific dispatch owner selection | MISSING | First loss for the corrected primary |
| Existing Dispatches projection | PRESENT, fallback owner | New per-occurrence owner adapter uses existing relation/kinds |
| WorkspaceIndex/public result | PRESENT, wrong owner | Current public exact forward/reverse Service edge |

**PROVEN BY TEST:** permanent primary proves #140c positive endpoint, direct AST
Store/new syntax, exact callable method context, exact canonical class span and
existing Service classification/publication before asserting owner. RED fails
`DispatchSite` versus `Service`, not endpoint identity. Regression/registration
were stashed while production was implemented, then restored unchanged.
Primary SHA-256: `bf7cb9fb09942ae452b0bae10602ab930ecb51437f8051fce573d2819c3b8c74`.

**PROVEN BY CODE INSPECTION:** the former fallback comes from the frozen shape's
Component-or-DispatchSite selection. Generic Store qualification is required
before this owner boundary; plain Store fails earlier and is UNSUPPORTED.
**NOT ESTABLISHED:** inherited callback/nested owner authority, imports/modules,
real-repository recall or #142 cold/warm acceptance.

## 30.4 Service identity authority

The new collector consumes existing `ClassInitializerEvidence`: canonical
consumer, exact class span and raw class. Existing decorator classification
must return `ClassKind::Service`, the same authority used by independent Angular
Service publication. `class_to_semantic_edges` already publishes Service Defines
independently of dispatch; that producer and #139 identity logic are unchanged.
The Service use does not manufacture a declaration. A helper whose name ends in
Service, a Store property, class proximity or dispatch syntax cannot qualify it.
The source's `Store` DI target is an existing Service reference, not an independent
Service declaration; live declaration-set assertions distinguish that reference
from the canonical owner declarations.

## 30.5 Invocation owner authority

`service_owners::QUERY` captures one structural chain:
class declaration → its own method → immediate body expression statement → call.
Collector joins the exact call span to native invocation evidence and requires
its callable declaration span to equal that captured method. It joins the class
span to the existing canonical class record, also checking the #140c canonical
owner and current file/hash. Neither nearest-owner search nor span containment
alone establishes Service ownership. Nested functions/arrows do not match the
own immediate-method statement chain. Shared core only orchestrates the domain
query/collector and private evidence lifecycle; it does not classify Service
ownership or change framework-neutral invocation contracts.

## 30.6 Endpoint independence

Original #140c records remain unchanged. Service collector accepts only their
already-resolved Action identity with current source hash/file; unknown,
NonAction, qualified, non-NgRx and unsupported constructor cases cannot be
rescued by a known Service. #140a class authority and all binding/transition
producers are untouched. No import, alias or module lookup is introduced.

## 30.7 Fallback behavior

For precise Service occurrences the owner adapter suppresses that occurrence's
fallback projection and emits its Service fact. All other qualified occurrences
still pass through the unchanged #140c/shape projection. Thus a Service and a
non-Service helper dispatching the same Action in the same file retain both
Service and DispatchSite edges. No aggregate edge is rewritten by Action spelling.
Unknown owner plus authoritative endpoint keeps fallback; unknown endpoint adds
no Service edge. Existing modern symbolic dispatch keeps its previous fallback
behavior. Callbacks/top-level forms that do not qualify under #140c are refused,
not claimed to have an authoritative fallback endpoint.

## 30.8 Semantic projection

`PassContext` owns compilation-local Service records, `MetaLayerContext` borrows
them, Angular evaluation computes NgRx semantic edges once via the owner adapter.
The adapter uses a projection-only clone of constructor suppression records to
leave resolved Service occurrences to the precise projection; original records
and endpoint admission are not changed. The original constructor-applied shape
continues to render markers. The existing Angular non-testing producer consumes
the computed NgRx edge set, including all other old NgRx facts.

Exact new fact: `angular/Service/AuditService → Dispatches →
ngrx/Action/AuditAction`, layer `ngrx`, existing file attachment and query contract.
No new relation, entity kind, parallel public graph or per-call schema is added.
Select, Store, reducer and other projections retain their existing owners.

## 30.9 Falsification matrix

Registered checks cover Service whose name lacks a suffix, exact method/context
and class spans, one parse/private capture stripping, helper/Component, logger,
unknown/qualified endpoint, plain Store, static method, callback/nested function,
top-level call before/after Service, multiple Services plus same-target fallback,
Component plus Service, declaration independence and current file/hash refusal.
Live cases additionally cover a second method, repeated calls, neighbor Action,
second Service, opt-out, preserved modern symbolic fallback, three fidelities,
same-named owners in two files, repeated source/fidelity and native lifecycle.
Claims follow executed verdicts below; unsupported forms are not counted as
supported ownership paths.

## 30.10 Public MCP evidence

Harness queries all expected incoming Action Dispatches edges and forward edges
for Service, Component and fallback owner identities. It checks exact domain,
kind, name, direction and both endpoint files, independent declarations, and
absence of ghost unknown/NonAction/new edges. It preserves source bytes in static
cases and retains every request, response, source oracle, context and stderr.
Structured results are authoritative; a marker or nonzero count cannot pass.
No model invocation, host UI rendering or model-consumption claim is made.

## 30.11 Duplicate/occurrence behavior

Repeated calls and two methods to the same Action preserve one file/triple edge
and one canonical Service occurrence. Separate Services remain separate subjects.
Two files with the same Service/Action names retain distinct file-provenanced
edges and entity occurrences. No new general owner/module identity is invented.

## 30.12 Lifecycle and cache

Live lifecycle: Service-owned AuditAction; move to OtherService; remove the first
Service's qualification while keeping dispatch (fallback); restore qualification;
change target to OtherAction; remove dispatch; delete owner; delete source.
Each refresh verifies old owner/target disappearance and independent declarations.

**PROVEN BY CODE INSPECTION:** `IRCompiler::compile_inner` runs CoreIRPass and
MetaLayerPass in a fresh context and drains resulting semantic products. Provide
cache reuse requires corresponding already-produced semantic edges alongside
cached IR; missing edges trigger source compilation. Hydration either retains a
current projection or recompiles through that compiler; it does not invoke owner
classification on bare persisted IR. Durable restore checks source/config/producer
compatibility before reusing semantic products. Private Service records are never
assumed to persist. Repeated same-source/fidelity live requests check unchanged
results, not parser-hit telemetry or #142 truth independence.

## 30.13 Compatibility/generation

Angular semantic producer **8 → 9** excludes old Service-as-fallback products.
Angular marker producer remains **5**; the original constructor shape renders
unchanged markers. Canonical pipeline remains **1**. New compatibility test proves
semantic signature changes while canonical product/config signatures stay equal.
Existing current-cohort template/selector/transition fixtures track generation 9;
behavioral oracles and old-cohort restoration rejection are preserved.

## 30.14 Preservation

Executed counts: #136 insert-before 9, #137 signatures 5, #135 DI completeness 6;
constructor dispatch 6, legacy transitions 11, type binding 7, Action declarations
9, framework publication 10, decorated owner 9, Pipe DI 9, inject-token 7, inject
source evidence 4, NgRx module 25, Angular semantic 18, decorators 28, template
compatibility 1, selector occurrences 5 and restore generation rejection 1.
**170 preservation tests passed**, plus six focused Service tests.
Clippy/build and live results are recorded below. No full suite was run.

## 30.15 Remaining #140 work

Nested/arrow owner authority; modern dispatch qualification for precise Service
ownership; plain/other unsupported Store injection forms; imported/alias/
namespace/barrel constructors; select ownership; standalone discriminants;
unsupported reducers; Selector/Effect criteria; representative recall; cold/warm
acceptance remain separate. #141/#142/CBM and query-history mechanisms untouched.

## 30.16 Incidental effects

Known Service precision neither resolves an unknown endpoint nor changes
Component/Select ownership. Same-file mixed Service/Component facts are a local
ownership result, not module resolution. Existing unresolved symbolic behavior
is preserved, not reclassified as constructor or Service authority. Do not close
#140 or promote these probes into other acceptance families.

## 30.17 Changed files

- `ngrx/service_owners.rs`: domain structural query, identity join and owner adapter.
- `ngrx.rs`: module registration only; frozen constructors/binding/legacy/transition
  implementations remain unchanged.
- Pipeline context/core/meta-layer and meta context: owned/borrowed private records,
  one parse, domain orchestration and private capture stripping.
- `layers/meta/angular_semantic.rs`: consume precise semantic projection while
  preserving original marker shape and other NgRx projections.
- `layers/meta/mod.rs`: semantic generation only.
- Primary/matrix/compatibility tests and normal MCP registrations: independent
  immutable RED, ownership/fallback matrix and product signatures.
- Three existing compatibility fixtures: current semantic generation only.
- PowerShell harness/README/report: reusable known stdio pattern, exact oracles,
  evidence, scope correction and declared limits.

## 30.18 Commands and artifacts

Candidate SHA-256:
`f12ca63d8aab5b7eadc7114f096c282cc293c0dd4d79b5c4d7ccd63ca9813ec9`.
Final evidence root:
`target/ngrx-service-dispatch/20261012-184454-580b86e6`.
Live verdicts: **84 PASS, 0 FAIL**, process exit 0. Twenty-four static cases at
High/Medium/Low (72), three source-file occurrence groups, one repeated source/
fidelity group and eight lifecycle stages. The earlier 81/81 run is retained at
`target/ngrx-service-dispatch/20261012-183746-7cb40297`; the final run additionally
checks a genuine `createAction` declaration/call alongside symbolic preservation.

Executed commands:

- `cargo test --locked -p clean-ctx --lib --all-features -j 4 ngrx_service_dispatch_`:
  permanent primary RED 0/1, identical primary GREEN 1/1, final matrix GREEN 6/6.
- Same Cargo test command with the 18 preservation filters enumerated in §30.14:
  170 passed. Final focused Service set adds six, total 176 passed in that run.
- Focused `rustfmt --edition 2024 --config skip_children=true` on changed Rust
  files except the permanent primary. No workspace-wide formatting.
- `cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings`:
  exit 0 after final production changes.
- `cargo build --locked -p clean-ctx --bin clean-ctx --all-features -j 4`:
  one audited candidate build, exit 0.
- Installed PowerShell `-NoProfile -File
  verification/issue-reconciliation/Invoke-NgrxServiceDispatch.ps1
  -BinaryPath ./target/debug/clean-ctx`: exit 0, final 84/84 live groups.

Rust uses existing `/workspace/.tooling/cargo` and rustup homes. PowerShell uses
its existing installation and XDG directories. Logs are retained under final
`focused-checks/`, including the behavioral RED and exact per-filter counts.
The live execution was on Linux; Windows execution of this new harness was not
performed. Existing PowerShell stdio patterns are reused, without asserting
unobserved platform or client-host behavior.
Focused rustfmt excludes immutable primary; changed files pass strict UTF-8/no
BOM, active code ≤615 lines and `git diff --check`. No installation, external
issue write, push, daemon termination or live CBM was performed.
