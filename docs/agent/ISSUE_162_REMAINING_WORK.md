# Issue #162: current position and remaining work

Agent: Agent MaxHeadRoom
Reconciled: 2026-10-25
Status: planning/evidence reconciliation; no production change or runtime run.

## Combined C2/C3 binary acceptance inspected PASS

Inspected the operator receipts: provider-imported-owners PASS, 332 checks;
provider-composition preservation PASS, 190 checks. Each records zero failed
checks/errors, three FINISHED sessions and the 500-check maximum. Both use binary
SHA-256 b5a820fbe59fce1b4b695da0e807f9aaecf5216659cdd3070d780b7c9151236b,
mtime 2026-10-26T19:36:00.297Z, and the same recorded input hash. Summary and
receipt hashes are retained at target/issue-162/composition65-binary-receipt.json.
The full logs are under target/issue-162/composition65-binary-20261026-153511/;
syntax preflight passed 27 JavaScript files and the all-feature binary build
finished without warnings/errors.

This supersedes the binary UNRUN/deferred checkpoints below for these two suites.
It verifies the bounded imported Component and Directive/Module composition
production boundary and affected same-source preservation. No other live suite,
full repository gate or final Claude field result is inferred. No production or
harness change accompanies inspection. Broader P1/W04/#162 closure remains open.

## Where we are

**We have a working, repeatedly verified foundation and several migrated Angular
consumers. We are not near the final gate for the full #162 scope.** The recent
provider work is one part of a substantially larger Angular-wide migration.

The governing scope is the [implementation plan](ISSUE_162_IMPLEMENTATION_PLAN.md),
especially sections 5, 6 and 6A. It contains **26 acceptance questions (AQ01–AQ26)**.
This checklist organizes them into **12 implementation work packages and 3 final
acceptance gates**. These are planning buckets, not 15 remaining fixes, commits,
test runs or equal units of effort. The number of implementation defects and
architectural decisions inside unreviewed packages is not yet known.

Five implementation packages have bounded verified progress; seven still need
their broad contracts reconciled against production. None of the 12 is certified
closed as a whole by this reconciliation. That does not mean nothing is
implemented: it means partial GREEN receipts cannot close larger questions.
There is no defensible percentage-complete or time estimate yet.

**Immediate state:** generation 59 is the last verified production checkpoint.
The bounded `useValue` reference-admission slice has completed V1–V4: 73 passing
deterministic test executions and corrected binary acceptance of 1660/1660 checks.
W04 remains open; its remaining obligations are reconciled below. No next
implementation slice has been selected as part of this checkpoint.

## What the existing GREENs establish

| Verified slice | What it establishes | What it does not establish |
| --- | --- | --- |
| Native retained identity and transport | Exact declaration/reference evidence is consumed by migrated slices through the existing source-owned artifact/session/durable/index path. | Universal TypeScript/project conformance or every Angular consumer. |
| Module and standalone composition slices | Canonical membership/composition cases have recorded focused and binary evidence. | Complete owner-visible declaration sets, all metadata forms or reverse closure. |
| Constructor, field and functional DI | Bounded API/token proof, declaration/string distinctions, same-name discrimination, mutation and restart. | Which scoped provider satisfies a consumer, all DI syntax or complete reverse consumers. |
| Direct member initializers | Exact owner/member identity for the migrated input/output/model initializer forms, with forged-pair and lifecycle controls. | Public aliases, inheritance, generated bindings or template-use joins as a complete contract. |
| Package/public-entry authority | Owning package and consumed public entry are distinct; HTTP role and selected mutation/refusal/lifecycle cases pass. | General package/API closure or every export-map/resolver frontier. |
| Provider registration, class/factory/alias relations | Exact migrated token/pair authority, shared mechanics, adversarial and durable/currentness controls for the approved forms. | Canonical runtime `useValue` identity, all callable forms, all provider owners, multi/precedence or injector closure. |

Latest inspected binary receipt:
`target/live-acceptance/issue-162/2026-10-25T19-21-16-341Z-16680/report.json`:
**1660/1660 checks, zero errors, three FINISHED sessions**. Binary SHA-256:
`aa2a64437fc940fd01f620ada9228018bbdb65745d6d302a10a53e3f127ef685`.
This is bounded operator binary evidence, separate from Rust tests, CI and Claude
field acceptance. Later test-only changes do not extend that harness's scope.

The [consolidated audit](ISSUE_162_CONSOLIDATED_AUDIT_2026_10_24.md) and linked
decision records retain chronological receipts and RED/GREEN evidence. This
document is the navigation/status checklist, not another historical receipt log.

## The remaining implementation packages

Status meanings: **PARTIAL** means some bounded migration evidence exists;
**UNRECONCILED** means broad #162 acceptance has not been established here, not
that the repository has no implementation. **CLOSED** requires the exit criteria
below plus concrete evidence links. No package is CLOSED yet.

| ID | Package / acceptance questions | Current position | Remaining work and concrete exit criterion |
| --- | --- | --- | --- |
| W01 | Foundation, applicability and API roles — AQ01; shared AQ25 prerequisites | PARTIAL | Reconcile remaining native/project/mode/API-role frontiers and actual consumer inventory. Resolve the explicitly excluded `erased_namespace_query` frontier for full acceptance. Exit: required roles and language evidence have current, tested authority; no known gap is hidden under a support label. |
| W02 | NgModule and standalone composition — AQ02–AQ03 | PARTIAL | Reconcile remaining expression/metadata forms, declaring lexical context, explicit/version-default eligibility, empty-owner transitions and reverse membership. Exit: required composition questions are production-queryable with canonical targets, paths and lifecycle proof. Closed visibility belongs to W03. |
| W03 | Template ownership, visibility, selectors, pipes and structural context — AQ04–AQ07, AQ18 | UNRECONCILED | Establish exact inline/external template ownership; closed owner-visible declaration/pipe sets; scoped selector/pipe resolution and reverse uses; exact control ancestry. Exit: public answers explain the actual owner and visibility path, exclude out-of-scope lookalikes and survive template/composition changes. |
| W04 | Providers and DI — AQ08–AQ09 | PARTIAL; useValue reference slice verified | Six open obligation groups below cover owners/composition, strategy forms, injector context, multi/precedence, consumer resolution and public closure. Exit: required scoped provider/consumer questions are answerable with reasons and truthful uncertainty, not merely equal token labels or local strategy edges. |
| W05 | Public members and reactive APIs — AQ10–AQ11 | PARTIAL | Complete required member aliases, directionality/required/model meanings, inherited/exposed members and binding joins; reconcile signal/computed/effect/resource/interoperability API authority and versioned dependency forms. Exit: public member/use and admitted reactive-structure questions have exact ownership and lifecycle evidence. No inferred runtime reactive graph. |
| W06 | Routing and application composition — AQ12–AQ14 | UNRECONCILED | Canonical eager/lazy route targets, selected exports and reverse routes; parent/config guard/resolver/provider context; actual bootstrap and merged application config with distinct app boundaries. Exit: public answers trace the correct route/application/config paths through dependency mutation. Functional DI proof alone does not close routing. |
| W07 | Host directives, view/content queries and defer — AQ15–AQ17 | UNRECONCILED | Host composition/member exposure, query locator/token/read/required/plural distinctions, and defer eligibility/eager-use blockers using real visibility and template context. Exit: each required question has exact API/target/owner proof and public explanations without invented runtime query results or bundle behavior. |
| W08 | NgRx and RxJS — AQ19–AQ20 | UNRECONCILED | Reconcile exact registration/action/member/reducer/effect/selector and dispatch/select neighborhoods; prove stream/operator identity and order at admitted use sites. Exit: required forward/reverse architecture questions are canonical/current and preserve prior issue-family behavior. Existing #152 controls are assets, not blanket #162 certification. |
| W09 | Reactive Forms and Formly — AQ21–AQ22 | UNRECONCILED | Control/group/array/validator composition and template binding; Formly field/config/type registration and expression ownership. Exit: required public composition/use questions join exact config/template/registration authority and classify dynamic facts honestly. |
| W10 | Testing and TestBed — AQ23 | UNRECONCILED | Canonical testing API calls, imports/providers/overrides, tested targets, test configuration and reverse uses; preserve test versus production scope. Exit: registered public queries explain the relevant test configuration and target identities through edits and restore. |
| W11 | Cross-layer impact, public joins and completeness — AQ24–AQ25 | PARTIAL | Complete reverse/affected-neighborhood acquisition, reason paths, late publication/currentness and query-specific completeness across required families. Exit: same-named endpoints remain distinct and each answer's claimed closure/currentness is justified through the production query contract. |
| W12 | Rendering and marker consumers — AQ26; AQ18 presentation | UNRECONCILED | Audit every materially affected renderer for lost owner/endpoint/provenance/uncertainty/ordered context; prove any source-local exception. Exit: compressed/public presentation preserves the upstream fact without requiring source reconstruction or inventing authority. |

These packages cover every AQ ID. W11's cross-cutting role does not remove each
family's own reverse/currentness/completeness obligations. W12 does not excuse
weak upstream proof. Existing implementation must be inspected and reused before
assuming a rewrite is needed.

### Complete question crosswalk

Every row remains open at its full question scope; the verified-slice table
above identifies useful completed parts. The implementation plan retains the
full normative question/fact/closure wording.

| Question | Meaning | Main package |
| --- | --- | --- |
| AQ01 | Actual Angular declaration/API role | W01 |
| AQ02 | Module ownership and composed membership | W02 |
| AQ03 | Standalone composition and eligibility | W02 |
| AQ04 | Actual template ownership, forward and reverse | W03 |
| AQ05 | Closed owner-specific visible declarations | W03 |
| AQ06 | Scoped selector occurrence resolution | W03 |
| AQ07 | Scoped pipe resolution and reverse uses | W03 |
| AQ08 | Applicable provider strategy, registration and scope | W04 |
| AQ09 | Scoped DI consumers and removal consequences | W04 |
| AQ10 | Public members, aliases and bindings | W05 |
| AQ11 | Proved reactive API/dependency structure | W05 |
| AQ12 | Actual eager/lazy route targets and reverse uses | W06 |
| AQ13 | Applicable route guards/resolvers/providers/config | W06 |
| AQ14 | Actual bootstrap/application configuration | W06 |
| AQ15 | Host directive composition and exposure | W07 |
| AQ16 | View/content query target and API meaning | W07 |
| AQ17 | Defer eligibility and eager-use blockers | W07 |
| AQ18 | Exact control-flow occurrence context | W03, with W12 presentation |
| AQ19 | NgRx action/state/registration/use neighborhood | W08 |
| AQ20 | RxJS stream/operator/subscription structure | W08 |
| AQ21 | Forms controls/validators/composition/bindings | W09 |
| AQ22 | Formly registration/config/field-use neighborhood | W09 |
| AQ23 | TestBed/test configuration, overrides and targets | W10 |
| AQ24 | Cross-layer semantic impact and reason paths | W11 |
| AQ25 | Exact/current public joins and completeness | W11, with every family |
| AQ26 | Rendering that preserves proved semantics | W12 |

## Completed bounded checkpoint: useValue reference admission

This is a subtask of W04, **not the remaining definition of W04 or #162**.

- [x] **V1 — Owner decision:** owner APPROVED the bounded
  [reference-admission proposal](ISSUE_162_PROVIDER_VALUE_REFERENCE_DECISION.md).
  It repairs registration only; canonical runtime-value identity is not decided.
  The explicit disposition authorizes V2–V4 routine choices within the contract.
- [x] **V2 — Repair and deterministic evidence:** freeze the unchanged valid RED,
  implement only the approved contract, restore exact bytes, verify focused
  positives/refusals and affected-family preservation in one planned batch.
  Generation-59 provider59-*.log receipts: 73 passed, no failures/ignored or warnings.
- [x] **V3 — Lifecycle/currentness evidence:** verify complete retained proof,
  stale restore refusal, unchanged-consumer dependency/config/package mutation,
  repair and independent retirement; reuse existing fixtures where equivalent.
  Shared durable/currentness variants passed in the inspected generation-59 batch.
- [x] **V4 — Lint and binary acceptance:** finish the selected checks, rebuild
  once for this stable production slice and run the applicable batched driver.
  Corrected generation-59 report: 1660/1660 checks, three FINISHED sessions,
  no errors; fmt/Clippy/syntax GREEN operator-reported. First failed expectation
  remains preserved separately. No Rust repair accompanied its correction.

Original RED: `provider-value-reference-red-rerun.log` — the ordinary baseline
passed, then mixed aliased/namespace value routes returned 0 registrations
instead of 2. The earlier baseline failure was a test-control correction and
is preserved separately. The unchanged valid test and its later assertions now
pass in the generation-59 deterministic batch.

This closes the approved reference-admission slice. It does not authorize a
canonical value evaluator or establish injector closure.

## W04 reconciliation after V4

Six obligation groups remain under the original AQ08/AQ09 questions. These are
requirements to reconcile, not six known bugs or six automatically approved
architectural changes. Existing assets and other work packages may already
answer portions; trace them before implementing replacements.

| ID | Remaining obligation | Current evidence boundary | Exit needed for W04 |
| --- | --- | --- | --- |
| P1 | Provider owners and composed registrations | Direct Component/Directive/Module registration, Component view position and same-source composition have bounded verified checkpoints. Imported Component and Directive/Module composition have scoped Rust/lint GREEN and inspected generation-65 binary PASS (332 imported-owner checks, 190 same-source preservation checks); route/application/environment/testing contexts remain unreconciled. | Map required owner/container forms to exact production acquisition and canonical registration answers, including composition/retraction and negative owner controls. Coordinate W02, W06 and W10. |
| P2 | Strategy/form coverage and value meaning | Class shorthand/useClass, function-declaration useFactory and useExisting have bounded canonical pair evidence. Other callable forms retain classified compatibility behavior. useValue reference admission is verified; its strategy presentation remains noncanonical. | Reconcile each required form with its actual question and public fact. Only propose richer target/value identity when a demonstrated question requires it; do not assume a general evaluator. Record genuine unsupported forms and resolve required capabilities rather than silently excluding them. |
| P3 | Injector/application context | Local token/pair identity does not identify the applicable injector or connect its owner to the actual application/configuration hierarchy. | Establish the required owner/application/injector context and visibility proof, with distinct applications/scopes and current dependency paths. This is a prerequisite for effective-provider claims. |
| P4 | Multi, overrides and precedence | Existing deduplication preserves semantic fact identity; it is not an ordered registration/multi/override model. No winning-provider claim follows from existing edges. | Define and verify the required order/multi/override questions against real registration context, with explicit runtime uncertainty. Architectural changes require the actual bounded owner decision. |
| P5 | Scoped consumer satisfaction and reverse impact | Constructor/field/functional consumption facts are bounded GREEN, but they are not joined to an effective scoped registration universe. | Answer which registration applies to a consumer, why, and the effect of removal/override, with complete requested consumer acquisition or correctly classified bounds. Coordinate W11; token equality alone is insufficient. |
| P6 | Public explanation, closure and ecological acceptance | The binary driver exercises selected local identities/lifecycles. It does not certify every provider owner, context or reverse neighborhood. | Expose required scope/strategy/reason paths through registered production queries; verify relevant cross-family order/currentness/negative/durable cases and reconcile all P1–P5 evidence before package closure. Reuse G01–G03 at campaign finalization. |

The next planning task should trace **P1/P3's existing owner/context path** and
produce one bounded question/exit contract for review, rather than automatically
opening another value-identity feature. This is sequencing guidance, not approval
of a new injector model. No implementation for P1–P6 was added during this
reconciliation. W04, AQ08 and AQ09 remain open.

The initial [owner/context trace](ISSUE_162_PROVIDER_OWNER_CONTEXT_TRACE.md) is
now authored. Its concrete next candidate is P1.D1: direct Directive.providers
registration, reusing the existing pipeline. Both current provider-owner gates
rejected Directive despite existing native role proof. The intended public RED
is now observed and the [bounded owner extension](ISSUE_162_PROVIDER_DIRECTIVE_OWNER_DECISION.md)
is approved. Generation-61 provider/durable/currentness and compatibility controls
are operator-reported GREEN; the inspected rebuilt binary report passes all
1844 checks. The bounded P1.D1 direct-registration checkpoint is verified.
P3 injector/application context remains
unresolved; D1 closes only that local owner-admission subquestion.

P1.VP1 [Component viewProviders position](ISSUE_162_VIEW_PROVIDER_POSITION_DECISION.md)
is verified at the bounded generation-62 checkpoint: focused/durable/currentness
and fmt/Clippy operator-reported GREEN; all 16 split binary suites inspected PASS
(2569 checks across receipts, maximum 322 in one suite). RegistersViewProvider
preserves declared position separately from ordinary RegistersProvider. This
does not establish effective visibility or satisfy P3–P6; P1 and W04 remain open.

Next investigation batch: [P1.C1 provider-array composition](ISSUE_162_PROVIDER_COMPOSITION_TRACE.md).
Six tracked controls cover references, spreads and nested arrays in both admitted
Component positions. All six intended REDs are observed and frozen. The
[bounded composition decision](ISSUE_162_PROVIDER_COMPOSITION_DECISION.md) is now
owner-approved with the structural-authority boundary. Generation-63 compilation
and focused composition/preservation controls are operator-reported GREEN;
the additional durable/currentness tracked controls and lint gates are now
operator-reported GREEN. Inspected composition and position binary receipts PASS
with 190 and 160 checks respectively, zero failures and six FINISHED sessions.
P1.C1 bounded local acceptance is satisfied; full repository/final field gates and
remaining owner/composition obligations stay open. No runtime array
evaluation, imported contributor arrays or injector closure is authorized.

Next investigation: [P1.C2 imported provider arrays](ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_TRACE.md).
Six tracked questions cover alias/barrel/namespace contributor routes in both
Component positions, with established direct and same-source prerequisites.
All six intended imported-stage REDs are inspected and byte-preserved (0 passed,
6 failed, 0 ignored). The [bounded source-qualified membership decision](ISSUE_162_PROVIDER_IMPORTED_COMPOSITION_DECISION.md)
is owner-approved. Generation-64 imported focused receipt records 11 passed, zero failures/ignored,
including all six unchanged REDs and five integrated controls. Compilation is
operator-reported GREEN. Seven preservation receipts add 107 passing executions
(zero failed/ignored); fmt/Clippy are GREEN. The combined scoped Rust total is
118 passing executions at that generation. The later generation-65 combined C2/C3
binary checkpoint now supplies bounded acceptance: imported owners 332 PASS and
same-source preservation 190 PASS, with six FINISHED sessions and no errors.

Next mapped batch: [P1.C3 Directive/NgModule provider composition](ISSUE_162_PROVIDER_OWNER_COMPOSITION_TRACE.md).
Twelve tracked controls cover both ordinary-provider owners across nested/local
reference/spread and imported alias/barrel/namespace forms. Each requires direct
owner registration and matching composed Component controls before its candidate
assertion. All twelve intended REDs are inspected and byte-preserved (0 passed,
12 failed, 0 ignored). The [shared owner/position contract](ISSUE_162_PROVIDER_OWNER_COMPOSITION_DECISION.md)
is owner-approved. Generation-65 focused receipt records 17 passed, zero failures/ignored, including
the twelve unchanged REDs and five integrated tests (both owners per test).
Preservation adds 118 passing executions (67 earlier provider passes, one corrected
control rerun and 50 remaining checks); the historical invalid-control failure is
retained and resolved. Scoped generation-65 total: 135 passing executions;
fmt/Clippy GREEN. Combined generation-65 C2/C3 binary acceptance is inspected PASS
(332 imported-owner and 190 same-source preservation checks). These bounded
subquestions have local acceptance; broader repository/field and P1 gates remain.

Next investigation: [P1.R1 route-declared provider membership](ISSUE_162_ROUTE_PROVIDER_TRACE.md).
Two tracked direct/imported public controls produced the intended omission REDs
(0 passed, 2 failed), after established native Component registrations and existing
route projection prerequisites. Exact evidence is archived; the
[bounded route decision](ISSUE_162_ROUTE_PROVIDER_DECISION.md) is owner-approved
with private route identity and a mandatory public-API guardrail. Generation-66
compilation/focused checkpoint is inspected GREEN: 6 passed, zero failed/ignored,
47.04 seconds, including the two unchanged REDs and four integrated controls.
The additional exit controls and scoped preservation/lint/guard batch are now
GREEN: 161 preservation passes plus six initial, four earlier exit passes and
one corrected acquisition-control rerun yield 172 passing executions. Historical
failure evidence is preserved; automatic removed-root rediscovery is not certified.
The approval evidence map is reconciled and generation-66 binary receipts are
inspected PASS: route-providers 305 checks plus imported-owner preservation 332,
zero errors and six FINISHED sessions. Bounded P1.R1 local acceptance is satisfied
with explicit known-root reacquisition after recreation; automatic rediscovery
of a removed route identity remains uncertified. Broader P1/W04 and repository/
field gates remain open.
Route shape omits providers; native route-root demand and distinct route-object
identity require tracing/approval before production extension. No injector model
or route authority change is authorized by the previous class-owner approvals.

## What counts as closing an implementation package

Each package must meet the implementation plan's existing acceptance rule:
**answerable, authoritative, production-queryable, current and correctly
classified for completeness**. In concrete terms:

- [ ] Map required AQ subquestions and admitted forms to actual production
  producers, consumers and public answers; record known exceptions/gaps.
- [ ] Reuse shared authority/lifecycle mechanics and keep specialized semantics
  with their owner. Remove the replaced weaker authority path where applicable.
- [ ] Make required positive questions answerable; refusal/uncertainty must not
  silently remove an in-scope capability or stand in for implementation.
- [ ] Retain and validate exact identities, occurrences and consumed witnesses.
- [ ] Supply applicable tracked nonempty/adversarial/currentness/durable/public
  regression evidence, including valid RED before repair for demonstrated bugs.
- [ ] Verify the default production path and applicable binary behavior; record
  tests, operator binary receipts and later field results as distinct categories.
- [ ] Link the evidence and state exactly which subquestions remain open.

A new architectural decision is requested only when implementation actually
crosses an unapproved contract. Existing approvals and equivalent-mechanism reuse
remain valid; routine implementation choices do not create another approval gate.

## Three final acceptance gates

| ID | Gate | Exit criterion / current status |
| --- | --- | --- |
| G01 | Final source audit and repository gate | All W01–W12 exit criteria reconciled; obsolete competing authority removed; known frontier/exclusions resolved for the claimed scope; operator runs the single [authoritative Final Verification Gate](verification.md#final-verification-gate). OPEN. Incremental fmt/Clippy receipts do not equal this gate. |
| G02 | Consolidated production-boundary acceptance | Reusable binary assets cover all applicable migrated families and cross-family/ecological cases, with rebuilt binary/input hashes, real public answers, persistence and lifecycle. OPEN. Current 1660-check driver is strong bounded evidence, not whole-migration coverage. |
| G03 | Batched final Claude field acceptance | Local questions resolved first; consolidate remaining real-workspace/client-only questions into the fewest practical pilots, record results and any discoveries, then obtain owner acceptance. OPEN; do not request field switching while local implementation gaps remain. |

Do not duplicate or improvise the full repository command list here; verification.md
is its sole authority. All Cargo/build/lint/binary/field execution remains
operator-owned under AGENTS.md.

## How we keep this from becoming an endless sequence

This is a sequencing/checklist proposal within the existing scope, not a new
repository policy or permission to narrow #162.

1. Select one named package/subquestion and write its concrete slice exit before
   authoring another regression. Show what finishing the slice will close.
2. List the known positive, refusal, lifecycle and public obligations together.
   Author/batch independent controls where the contract permits, instead of
   handing off one new case after every GREEN.
3. Repair causal gaps under existing approvals. When a new decision is required,
   explain its bounded effect and whether it is part of the current package.
4. At a stable GREEN, update this checklist with closed subquestions, remaining
   obligations and evidence links. A commit checkpoint is not package closure.
5. Keep discoveries under their AQ/package. Do not expand into compiler parity,
   arbitrary runtime evaluation or another strategy solely because syntax is
   nearby. Any genuine scope change needs an explicit owner decision; required
   existing questions cannot be silently dropped either.
6. Re-estimate only after broad UNRECONCILED packages have been inspected. Until
   then, report remaining packages/questions and concrete blockers, not a guessed
   percentage or a promise that the next GREEN finishes the campaign.

The bounded V1–V4 slice is verified and W04 has been reconciled above. The next
planning action is a production trace of the existing provider owner/context
path (P1/P3) to select a concrete scope contract before further implementation.
No production changes or new tests were made for this reconciliation. No Cargo,
binary, server or field harness was run. Static link/UTF-8/whitespace validation
does not certify completion of any implementation package.
