# Issue #162: Angular semantic composition and authority scope

Agent: Agent MaxHeadRoom

Date: 2026-10-21

Status: baseline inventory with observed **3 PASS / 9 RED**. The
[foundation design](ISSUE_162_FOUNDATION_DESIGN_FREEZE.md) is **APPROVED AND FROZEN**.
Implementation has begun; phases, current integration gaps and the mandatory
Angular-wide migration matrix live in [the working plan](ISSUE_162_IMPLEMENTATION_PLAN.md).
Descriptions below record the investigated baseline unless explicitly superseded.
Runtime GREEN is NOT ESTABLISHED. No Cargo, binary, MCP/provider process or live
harness was run by the agent.
Filesystem source inspection was used to preserve the operator-owned runtime
boundary. This document implements the issue's required investigation deliverable,
not its completion criterion. The initial tests cover the shared P0 foundation;
they are not a substitute for the full family acceptance matrix below.

## Operator RED receipt

The subsequent operator run in `target/issue-162/red.log` establishes **3 passed,
9 failed**, 4134 filtered out, 14.41 seconds. The agent inspected the completed
summary and all nine behavioral assertion locations. No compiler warning/error
diagnostics precede execution in that capture. Runtime RED is now ESTABLISHED;
earlier NOT RUN statements below describe the initial authoring phase.

The regression SHA-256 remains
`B63883E818FC70DF0990D23E2C31AC9D318FED458DA74B70B693C393DBA86379`,
matching the initial authored snapshot. No oracle was changed.

| Observation | Classification |
|---|---|
| Existing direct control and #157 kind mutation | PASS: existing supported relationship and contributor invalidation preserved |
| Composition cycle with independently proven Leaf | PASS for this fixture, not proof of all cycle shapes |
| Competing star exports | PASS for fail-closed output only; current lack of barrel resolution means this does not prove conflict detection |
| Aliased Angular decorator | Requirement RED: missing Directive membership |
| Barrel chain | Requirement RED at initial positive: missing membership; retarget assertions not reached |
| Imported declaration provenance | Valid behavioral RED: nonempty relationship attributes imported Leaf to app.module.ts rather than leaf.ts |
| Nested imported arrays | Requirement RED: missing membership |
| Package exports and package imports | Two requirement REDs: missing membership with explicit synthetic project/package witnesses |
| Standalone explicit/default | Requirement RED at explicit standalone positive; version-default iteration not reached |
| tsconfig paths | Requirement RED at initial mapping; config-retarget assertions not reached |
| Foreign same-spelling decorator | Valid authority RED: real Directive positive control succeeds, but unrelated decorator still grants Directive membership |

The results justify shared resolution/declaration/composition authority, not nine
consumer-specific fixes. New required capabilities are distinguished from defects
in existing provenance/authority. No live harness or full preservation run is
inferred from this focused evidence. Before production edits, freeze the exact
regression and registration per the required narrow stash procedure.

## Authority and baseline

Baseline HEAD: `83202841`, the committed #157 repair. Working tree was clean at
the start of #162. #157's operator-reported full repository/live success is prior
evidence, not verification of these new tests or future #162 changes.

The supplied issue explicitly requires a capability matrix before implementation.
Repository architecture.md additionally requires observed behavioral RED before
repairing reproducible semantic defects. The initial operator-owned focused RED
is now captured above. Production implementation remains gated on the requested
foundation-design freeze. Missing materially used idioms remain required #162 work.

The governing contract is:

```text
supported Angular claim
  => exact TypeScript reference/declaration identity
   + Angular package/export authority
   + applicable composition scope
   + required closure and exact consumed-source witnesses
   + final surviving publication validation
```

Legacy NgModule and modern standalone forms feed the same authority model.
An AST node proves syntax; it does not prove framework identity. A module path
proves neither the referenced export nor its declaration. Provider search is a
candidate hint only. Lower-bound responses may retain proved facts but cannot
retain disproved positives or stand in for feature completion.

## Classification key

SUPPORTED means authoritative semantics are implemented for a stated form;
SUPPORTED LOWER-BOUND means useful current evidence with incomplete closure or
authority certification. UNSUPPORTED describes a current implementation absence,
not permission to exclude a required feature. DEFERRED requires an intentional
scope decision; none of the issue's P0/P1 families is moved out of #162 here.
The current classifications below are static findings. Runtime preservation and
adversarial behavior still require tracked execution.

## Required capability matrix

L/M denote legacy and modern relevance. A check on rendering does not certify a
semantic graph, and a semantic graph does not alone certify completeness.

| Semantic family | Current support | Legacy | Modern 17+ | Cross-file | Dependency-backed | Completeness proof | Missing forms / proposed action |
|---|---|---|---|---|---|---|---|
| Declarations/decorators | SUPPORTED LOWER-BOUND: five written decorator kinds, graph/markers | yes | yes | imported leaf classification | imported contributors for modules | no common package/declaration proof | P0 bind exact decorator expression through canonical TS/package exports; aliases, shadowing, re-exports, declaration-kind retraction |
| NgModule | SUPPORTED LOWER-BOUND: direct entries, local/imported bounded spreads, declarations/imports/exports | primary | interoperability | yes | #157 path/hash footprint | current recorded footprint, no full composition closure | P0 recursive arrays/spreads, barrels and complete export visibility; cycle-safe partial facts |
| Standalone composition | UNSUPPORTED for imports graph in this path; standalone flag text is not composition | interoperability | primary | required | required | absent | P0 shared composition engine, explicit flag/version default, visible declaration closure |
| TS module resolution | UNSUPPORTED as common project-aware authority; several direct-relative helpers exist | yes | yes | primary | source + resolution/config/package witnesses | absent | P0 one resolver for relative/index/extensions, imports/exports, paths/baseUrl, packages and project mode |
| DI/providers | SUPPORTED LOWER-BOUND: constructors, structured supported inject sites, Component/Module class/factory/value registrations | yes | yes | binding references | uneven; not a full provider neighborhood | no complete injector graph | P0/P1 Directive/route/application/bootstrap owners; useExisting, multi, token identity, provider function obligations |
| Decorated inputs/outputs | SUPPORTED LOWER-BOUND: written decorators and aliases | primary | yes | inherited/exported authority not closed | declaration/member authority | no common owner package proof | preserve exact local facts; strengthen owner/member decorator binding and alias endpoints |
| Signal input/output/model | SUPPORTED LOWER-BOUND: structured invocation-backed input/required/output/model projection | no | primary | imported API bindings | same-file canonical binding; broader resolution absent | no general closed owner/API proof | preserve existing invocation identity; use common resolver for aliases/re-exports and owner provenance |
| View/content queries | UNSUPPORTED for the requested architectural relationship graph; detection names alone do not satisfy it | decorators | signal calls | required for tokens/declarations | required | absent | P1 legacy + signal required/plural queries, locator vs token identity, read option and lifecycle |
| Host composition | UNSUPPORTED for hostDirectives architecture graph | host metadata/decorators | directive composition | required | required | absent | P1 directive/object forms and exposed input/output bindings; audit host events/properties separately |
| Templates/ownership | SUPPORTED LOWER-BOUND: inline/external ownership and ordered rendering | yes | yes | external HTML | owner/template source evidence | not a complete owner/visible universe certificate | preserve exact ownership; bind decorator authority and template snapshots to composition scope |
| Selector resolution | SUPPORTED LOWER-BOUND: occurrence candidates from scoped indexed Components | yes | yes | primary | catalog reads candidates/templates | discovery_complete is not Angular visibility closure | P0/P1 derive visible universe from standalone imports or NgModule declarations/imports/exports; coordinate #159 |
| Control flow | SUPPORTED LOWER-BOUND: legacy/modern occurrence control ancestry and ordered tokens | structural directives | blocks | enclosing scope | template snapshot | structural representation, not branch evaluation | preserve existing frames; audit malformed/nested blocks and all branch kinds rather than rebuilding from text |
| Defer | SUPPORTED LOWER-BOUND for triggers/control frames; UNSUPPORTED for proved defer-loadability edges | no | primary | required | template + declaration + reference-use obligations | absent | P1 distinguish deferred body/placeholder/loading/error; prove Angular eligibility, not tag placement alone |
| Routing | SUPPORTED LOWER-BOUND: route arrays, eager/lazy targets, guards/resolvers | yes | yes | yes | routing-specific dependency list | no shared resolver/complete active-route proof | P0 snapshot/resolution integration; P1 provideRouter features, route providers and bootstrap authority |
| Application bootstrap | UNSUPPORTED for requested root/config/provider graph | bootstrapModule | bootstrapApplication/config | primary | required | absent | P0/P1 actual invocation/package binding and root declaration/configuration obligations |
| Signals/reactivity | SUPPORTED LOWER-BOUND: six marker families | limited | primary | package API authority | not a general reactive dependency graph | marker preservation only | P1 prove binding/owner; version-gate resource/rxResource and preserve existing stable APIs |
| RxJS | SUPPORTED LOWER-BOUND: existing observable/pipe/operator markers | yes | yes | API/operator bindings | no full transitive graph proof | not certified | preserve extraction; audit API binding vs local lookalikes and any cross-file claims |
| NgRx | SUPPORTED LOWER-BOUND: existing selector/actions/reducers/dispatch authority refinements | legacy classes | modern APIs | primary | selected NgRx-specific footprints | full authority neighborhood remains #158 work | preserve #152 exact object/member authority; share TS mechanics, retain NgRx semantics and #158 boundary |
| Reactive Forms | SUPPORTED LOWER-BOUND: forms import aliases/builders/controls/validators markers | yes | yes | imported forms APIs/validators | no general canonical project resolution | not certified | preserve markers; audit local lookalikes, custom validator identity and external dependencies |
| Formly | SUPPORTED LOWER-BOUND: existing field/type/expression markers | yes | yes | API/type references | no general closure proof | not certified | preserve existing field behavior; establish authority only for architectural cross-file claims |
| Testing/TestBed | SUPPORTED LOWER-BOUND: existing Angular testing extraction/edges | yes | yes | tested declaration/provider refs | not a closed TestBed authority universe | not certified | retain test ownership; integrate testing module/import/provider scopes through shared mechanisms |

## Concrete repository evidence and per-family obligations

### R1: neutral TypeScript reference and project resolution (P0)

Evidence: ir/typescript_value_import_binding.rs retains source hash, exact
reference span, imported/local names and module specifier. It already rejects
type-only and lexical-shadowing cases for supported forms. Identifier/type/this
binding modules are related same-file mechanics. Cargo.toml has tree-sitter
syntax support, but no inspected canonical TypeScript project/module resolver.
Searches over ir/angular_meta/workspace found no tsconfig/moduleResolution/
baseUrl/package-resolution implementation closing this contract.

Three independent path policies exist in module_metadata/sources.rs,
routing/structured.rs and ngrx/object_actions.rs. Module metadata chooses an
exact relative file or .ts; routing has its own .ts fallback; NgRx has its own
relative canonicalization. None provides a shared export-symbol graph.

Required authority: source identity + span + value/type namespace + lexical
binding + project/compiler options + module mode/conditions + export path +
declaration identity. Distinguish file resolution from export resolution.
Declaration identity must retain physical source and declaration span; canonical
resolution may prove that multiple re-export paths reach the same declaration,
while conflicting star exports remain ambiguous. Do not first-match exports.

Required dependencies: consumed source bytes, tsconfig and extends chain,
package manifests/exports/imports/type routing, negative candidate paths and
failed lookups, package/version witnesses and applicable project boundaries.
Changes to any witness invalidate the affected proof, including previously
missing candidates. A guessed package path or absent installed dependency cannot
stand in for a proven package export.

Existing architecture can retain artifacts/dependencies, but needs generalized
resolution and retained declaration evidence. A fresh optional provider result
cannot create this authority. Source capture/scope checks must precede reads;
do not execute workspace configuration, install packages or cross authorized
roots merely because a path mapping points there.

### R2: Angular declarations and members (P0)

Evidence: detect.rs recognizes decorator syntax; decorators.rs/decorator_scan.rs
interpret written decorator tokens. module_metadata::declaration_kind uses that
extractor. This is not a proof that a decorator resolves to @angular/core.
semantic.rs produces declaration/member graph facts; #157 preserves NgModule
identity independently of membership. member_semantics.rs uses canonical
invocation evidence and exact TS bindings for signal members and inject.

Required authority: R1 resolves the decorator/API reference to the intended
Angular package export and maps it to a declaration-owned role. Local shadowing,
foreign same-spelling decorators, type-only imports, duplicate declarations and
aliased/re-exported symbols must fail closed or resolve exactly. A marker
detection heuristic can nominate work but cannot admit a semantic role.

Lifecycle: decorator/import/package/version changes retract Component-only
template/provider/member claims, not just the class label. Source-local facts
survive only while their owner/API authority survives. Preserve decorator input/
output aliases and exact invocation-owned signal/DI sites. Do not replace #160's
owner-identity requirement with terminal-name joins.

### R3: NgModule, standalone and template-visible composition (P0)

Evidence: module_metadata.rs expands one level and preserves imported-array
lexical context using BoundLeaf. sources.rs owns the exact byte snapshots and
missing-path witnesses from #157. Angular evaluation carries its dependencies.
There is no standalone imports graph in this projection.

Required authority: R1/R2 + expression AST at its declaring scope. Recursively
compose const references, nested/spread arrays and supported exports, retaining
each leaf's own identity. Use an active traversal set keyed by declaration/
expression identity for cycle detection and a separate completed-result memo.
Cycles terminate by revisiting an active key; no arbitrary depth cap is needed.
Retain independently proven leaves and structured unresolved obligations.

The visible universe differs from raw imports: NgModule declarations, imported
module exports and re-exported visibility must compose according to Angular
rules. Standalone owners import standalone declarations or modules. Directive/
Pipe standalone status is declaration eligibility; they do not gain Component
imports metadata merely for being standalone. Version/default evidence is a
dependency. Do not infer standalone default just from the current wall clock.

Required dependencies: every expression/declaration/export/config/package source
consumed, plus negative/ambiguity witnesses. Mutation, membership removal,
deletion and restoration must keep the source-proven owner so empty results can
later revalidate. Do not compare empty cold/warm results as capability success.

### R4: provider/DI and application roots (P0/P1)

Evidence: provider_semantics.rs has Class/Factory/Value strategies and admits
Component/Module owners. member_semantics.rs and inject_evidence.rs own supported
functional consumption; constructor_injects.rs owns written constructor evidence.
These are separate from provider registration, which must remain distinct.

Required authority: R1/R2/R3 resolve injector owner, token and strategy endpoint.
useExisting is a token alias, not class construction; multi is a registration
property, not a different token identity. Class/string/InjectionToken locators
must be distinguished. Route/ApplicationConfig/bootstrap/env-provider invocations
need actual Angular API binding, not spelling or proximity. Imported provider
arrays/configs create producer-owned obligations through the shared resolver.

Root bootstrap must connect the actual bootstrap invocation with the resolved
root and provider/configuration boundary; legacy bootstrapModule remains first
class. Missing/ambiguous config cannot be filled from another indexed app.
Provider strategy, token target, route configuration and project changes retract
only dependent facts. New relation/schema vocabulary, if unavoidable, requires
review as described below; this initial test set adds none.

### R5: queries and host composition (P1)

Evidence: detector recognizes some legacy query/host decorator names; that alone
does not produce the requested architectural graph. No inspected family projection
implements hostDirectives or signal view/content relationships.

Required authority: R1/R2 resolve each query API/decorator, owner and declaration/
token locator. String template references remain locators unless another proof
identifies a declaration. Account for read/required/plural semantics without
pretending runtime query results are statically known. hostDirectives object
bindings need exact directive identity and exposed input/output alias validation.
Cross-file member/exposure contributors remain obligations. Host expressions and
events need structured binding evidence, not a marker for every API name.

### R6: templates, selector scope, control flow and defer (P0/P1)

Evidence: template_ownership.rs uses structured templateUrl/class capture
evidence. template_ordered.rs retains ordered syntax. template_occurrences.rs
already has If/ElseIf/Else/For/Empty/Switch/Case/Default and Defer/Placeholder/
Loading/Error control frames with syntax and spans. This is more than raw text
preservation and must not be erased by a replacement parser.

query/angular_templates.rs constructs an ephemeral catalog from indexed Component
keys/HasTemplate links and selector candidates. Discovery completion is not the
NgModule/standalone visible-universe proof required by #162/#159.

Required authority: R1-R3 + exact template ownership/source + closed visible
candidate universe. Same selectors in unrelated modules do not create ambiguity
inside a proven owner scope, and global uniqueness never establishes visibility.
Control ancestry must survive ordered compression. Defer-loadability additionally
needs standalone/import/reference-use evidence prescribed by Angular; placement
inside a Defer frame alone is not a loading/bundle proof. Missing templates,
changed exports, changed selectors and scope retargeting invalidate dependents.

### R7: routing and reactive/ecosystem families (P0/P1)

Evidence: routing/structured.rs parses route arrays/eager/lazy targets and guard/
resolver names. Its target classifier records SemanticDependency::current before
separately reading target text. That is a different read from the interpreted
snapshot and must be reconciled with #157's same-snapshot invariant. Its separate
resolver must be replaced by shared R1 mechanics without rewriting route meaning.
provideRouter needs actual invocation/config linkage, not the current route-root
text test alone. Existing guard/resolver/route syntax must stay available.

signals.rs has six SignalKind variants; it does not establish resource/rxResource
support. Preserve existing marker/API distinctions and strengthen binding before
claiming a cross-file reactive graph. RxJS/formly/reactive_forms/testing modules
provide existing specialized extraction; these remain semantic owners rather
than being replaced by one generalized framework interpreter. NgRx object action
dependencies and exact invocation/member evidence remain #152 authority assets.

Required dependencies for every new cross-file claim are R1/R2 snapshots plus
family-owned registration/use/route/test configuration obligations. A same-name
signal/inject/provideRouter from another package is a negative control, not a
hint to keep the Angular edge. Completeness for NgRx neighborhood and template
candidate universes must coordinate with #158/#159, not silently close them.

### R8: publication and persisted owner lifecycle (P0)

Current trace: public dispatch -> request/base preparation -> scoped discovery ->
candidate publication/compiler executor -> IR meta pass -> Angular evaluation ->
artifact edges/dependencies -> semantic index/durable context -> query evaluator.
#157 adds module footprint validation/revalidation, retaining empty owner identity.
The current acquisition capability catalog has source declarations, builtin type
reference incoming and C#-oriented dependency closure; none is automatically an
Angular composition certificate.

Required integration: producer obligations derive the narrow required sources;
shared acquisition enforces authorized scope and publication; Angular consumers
resolve meaning; final evidence validates the surviving publication plus all
resolution/identity/scope witnesses. A completed flight or cached discovery never
stands in for producer proof. Requests can reprepare superseded work through the
established coordinator; do not invent a new global retry/timeout budget.

Retain closure evidence in the actual persistent owner that consumes it. Prove
creation, propagation, returned artifact, ownership, reset/deletion/recompile,
consumer, tool exposure and live reachability. Do not complete an authority type
that only custom-pipeline unit tests populate. Durable incompatible producer
generations must invalidate semantic reuse without discarding valid canonical
source unnecessarily.

## Architectural decisions and recommended direction

The prompt authorizes the shared-authority objective, but explicitly says its
conceptual components are not authorization for those exact abstractions. The
following decisions must be made from the production evidence before committing
to cross-cutting implementation:

1. Canonical resolution engine: a native in-process Rust implementation/library
   integrated with current exact TS bindings, or a TypeScript compiler API host.
   A compiler host offers direct compiler module/export semantics but introduces
   an executable runtime/version/deployment and lifecycle contract. Native Rust
   retains the current product shape but must implement/validate the declared
   project modes, package conditions and export identity; a path resolver alone
   is insufficient. Recommend retaining in-process product mechanics if full
   conformance is attainable; do not add a Node/TypeScript runtime dependency or
   a resolver crate merely on this document's recommendation. The exact backend
   remains a reviewed design decision, not an implemented policy.
2. Identity exposure: preserve file+span declaration identity through Angular
   projection and joins. Existing EntityRef equality excludes file; merely
   filling file fields cannot fix every global same-name bucket. Decide how
   canonical declaration identity participates in existing occurrence/query
   semantics before changing shared equality or response contracts. Alternatives
   are a producer-specific retained identity consumed by existing occurrence
   projection, or a broader semantic identity migration. Prefer the narrower
   authority-preserving integration if it meets every required consumer.
3. New architectural relations: reuse applicable existing relations (e.g.
   ImportsModule for typed component composition) where their contract allows;
   do not mislabel provider aliases as classes or view queries as DI to avoid a
   vocabulary decision. Any genuinely new public relation/schema changes need
   an explicit reviewable mapping and approval before implementation.

No such global change is implemented in this phase. Existing behavior is the
migration reference, not proof of correct authority. Unsafe legacy positives
must be corrected through observed adversarial regressions, not preserved as
intended semantics. No unnecessary global architecture approval is requested
for ordinary test authoring or inspection.

## Initial tracked RED contracts: 12 tests

`src/tests/mcp/workspace_query_issue_162.rs`, registered in query.rs, uses public
tools dispatch with explicit workspaceRoot and CBM disabled. Synthetic project/
Angular package files are fixture evidence, not npm/network dependencies.
NgModule declarations explicitly use standalone:false; standalone tests use
standalone:true or the package-version-backed default. This avoids invalid
standalone declarations in an NgModule fixture.

| Test | Contract and expected static classification |
|---|---|
| existing_direct_control_and_157_lifecycle | Existing direct relation plus kind replacement; preservation control expected PASS |
| nested_imported_arrays_keep_their_lexical_source | Required #162 positive; no bare-name recovery when nested arrays cross files |
| composition_cycle_retains_independently_proven_leaf | Finite cycle with independent Leaf; cannot succeed by empty output or infinite recursion |
| barrel_chain_retargets_without_target_edits | index/star/named export chain; unchanged module observes changed exporting source |
| competing_star_exports_do_not_choose_first | Direct positive control, then conflicting Leaf exports; ambiguity fails closed |
| tsconfig_paths_retarget_invalidates_composition | Current project mapping supplies identity; changing config retargets unchanged target |
| package_exports_establish_source_identity | Package export conditions lead to a real declaration; excluded discovery directory does not erase required authorized proof |
| package_imports_establish_source_identity | Nearest package #imports mapping supplies source identity under bundler mode |
| standalone_explicit_and_version_default_share_composition | Both forms produce real Directive import relationship; cannot compare two empties |
| aliased_angular_decorator_preserves_declaration_kind | Canonical package export binding preserves Angular role through local alias |
| unrelated_same_spelling_decorator_cannot_grant_authority | Warm real Directive control then fresh foreign decorator: no Angular membership |
| imported_declaration_provenance_is_not_asserting_owner | Nonempty relation's object source must be the imported declaration, not AppModule's asserting source |

New-capability failures are requirement RED candidates, not claims those forms
already worked. The two likely passing controls do not certify unrelated gaps.
Compile failures/fixture defects are not behavioral RED. The package fixture
uses real installed-source-shaped files under its authorized root; the future
resolver must distinguish discovery exclusions from explicitly required source
evidence under the repository's approved scope model.

These tests deliberately do not invent final public schemas for host/query/
bootstrap/provider-alias relations. Those contracts follow the reviewed relation
mapping and shared foundation, with the full next-stage matrices below. No
production code should be edited merely to make these twelve examples pass.

## Full implementation sequence and acceptance obligations

1. Capture and classify initial public-path RED; freeze exact test/registration.
2. Resolve the backend/identity decision points; establish TS project/module/
   export declaration authority with tests for bundler vs node modes, JSONC/
   extends, paths precedence/fallbacks, package conditional exports/imports,
   declaration type/value spaces, symbol aliases and conflicting star cycles.
3. Bind Angular declarations, NgModule recursive composition and standalone
   composition to that authority; retain identity independent of membership.
4. Derive actual visible template scope, application root and providers. Complete
   the provider strategies/owners, modern routing linkage, host and view/content
   queries using shared mechanics and specialized semantics.
5. Preserve/audit control flow and implement proved defer semantics; version-gate
   modern reactive APIs and preserve all existing ecosystem layers.
6. Integrate every producer through artifact ownership, shared acquisition,
   dependency validity, final publication, persistence and response contracts.
7. Run the batched local gates and controlled production-boundary regressions;
   author one consolidated actual-binary MCP/live campaign, leaving any Claude
   field session until local questions are resolved and batched.

For each dependency-backed family: target/dependency/reverse/unrelated first;
cold/warm/repeat; both batch orders; fresh state and durable reopen; mutation,
deletion, restoration, retarget, rename, alias/export/barrel/config/package/project
and declaration-kind changes. Include same terminal names in separate files,
duplicate/conflicting candidates, type-only imports, local shadowing, unrelated
framework-spelled APIs and absent source. Compare against nonempty supported
controls, retain truthful partial evidence for unresolved obligations, and never
promote completeness merely because synchronization/discovery finished.

Track inventory, parsing, config/package resolution, composition, projection and
warm reuse work separately. Reuse compatible snapshots/ASTs/export results within
their actual owner rather than parsing all TS files per query. Required closure
determines work; caching removes repetition without changing truth. Full #162
completion remains pending every material P0/P1 family, not just this foundation.

## Official semantic references checked for this investigation

- [TypeScript module reference](https://www.typescriptlang.org/docs/handbook/modules/reference): project mode governs file resolution; package exports/imports and paths are not interchangeable heuristics.
- [TypeScript moduleResolution](https://www.typescriptlang.org/tsconfig/moduleResolution): bundler and Node modes differ; one unconditional extension/index policy is insufficient.
- [Angular component imports](https://angular.dev/guide/components/importing): standalone defaults to false before Angular 19; tests must carry version evidence rather than guessing from Angular 17+ terminology.
- [Angular NgModules](https://angular.dev/guide/ngmodules/faq): declarations intended for NgModules explicitly opt out of standalone in modern Angular.
- [resource](https://angular.dev/api/core/resource) and [rxResource](https://angular.dev/api/core/rxjs-interop/rxResource): current API pages mark stability since v22. Version/stability witnesses are required; this is not authority to apply those APIs to earlier projects.

## First operator runtime command

```powershell
New-Item -ItemType Directory -Force target/issue-162 | Out-Null
cargo test --quiet --all-features issue_162_ -- --test-threads=1 *> target/issue-162/red.log
$LASTEXITCODE
Get-Content target/issue-162/red.log -Tail 140
```

Expected authored selection: 12 tests. **NOT RUN; runtime RED NOT ESTABLISHED.**
No production implementation or full #162 completion is claimed. After observed
valid RED, preserve the exact regression/registration per architecture.md before
repair. No build, provider or Claude environment switch is needed for this step.
