# Issue #162 working implementation plan

Agent: Agent MaxHeadRoom

Updated: 2026-10-21

Overall status: **IN PROGRESS — foundation and consumer migration incomplete**.
Foundation design: **APPROVED AND FROZEN by the maintainer**.
Runtime GREEN: **NOT ESTABLISHED**. No agent-started Cargo, binary, MCP/provider
process or live verification is authorized. This plan is the working execution
record; update it as implementation and captured operator evidence advance.

## 1. Authority, scope and document ownership

- [Canonical Question Matrix Standard](../Clean-CTX_Canonical_Question_Matrix_Standard.md):
  **HARD REQUIREMENT for every layer, feature and upgrade**, including every
  material #162 capability. This is an acceptance gate, not optional documentation.
- [Foundation design](ISSUE_162_FOUNDATION_DESIGN_FREEZE.md): approved architecture
  and decision rationale. Do not silently rewrite it to match a weaker solution.
- [Capability inventory](ISSUE_162_ANGULAR_AUTHORITY_SCOPE.md): investigated baseline,
  required legacy/modern envelope and original observed RED.
- This plan: phases, current implementation status, migration matrix, pending
  work, acceptance receipts and next concrete actions.
- [architecture.md](architecture.md): tracked tests, frozen RED procedure and
  production lifecycle requirements.
- [verification.md](verification.md): authoritative final repository gate.

The maintainer explicitly requires an **Angular-wide migration**. Native TS
resolution and NgModule/standalone GREEN alone do not complete #162. Every
applicable producer and consumer must migrate; exceptions require an explicit
source-local proof. Required P0/P1 families remain in scope, including all the
families in section 6. Difficulty is not grounds for an unsupported/deferred label.

```text
ONE TS IDENTITY AUTHORITY
 -> ONE ANGULAR DECLARATION/API AUTHORITY
 -> ONE SHARED COMPOSITION/VISIBILITY FOUNDATION
 -> MANY SPECIALIZED SEMANTIC CONSUMERS
 -> ZERO COMPETING AUTHORITY PATHS
```

Frozen decisions: native in-process Rust; no Node/TypeScript runtime; retained
canonical TS/Angular identity before lossy projection; global EntityRef equality
unchanged; typed witnesses extend existing artifact/snapshot machinery; shared
legacy/standalone composition; explicit legacy fixture enrichment with preserved
behavioral assertions. The frozen #162 test remains byte-identical.

## 2. Current evidence and implementation checkpoint

| Item | Actual state / authority |
|---|---|
| Baseline | HEAD `83202841`, committed #157 repair; clean tree at #162 start |
| Initial tracked foundation suite | 12 tests; operator receipt `target/issue-162/red.log`: 3 PASS / 9 FAIL, 14.41s, 4134 filtered out |
| Frozen regression SHA-256 | `B63883E818FC70DF0990D23E2C31AC9D318FED458DA74B70B693C393DBA86379` |
| Frozen test + registration | Preserved before production edits in stash `6a82efdb5036df0b6d24e40266737d3d16b6885c`, name `issue-162 frozen RED B63883E818FC70DF` |
| Working tree regression state | Original #162 test and registration are currently stashed, not restored; a filtered run now must not be mistaken for the full foundation suite |
| New native source | `src/ir/typescript_resolution/{mod,host,project,syntax,resolver,package,capture}.rs`: initial implementation, not conformance-certified |
| Real compiler entry | ir/mod.rs registers the module; SemanticCompilationArtifact captures neutral TS evidence |
| Scope integration | INCOMPLETE: artifact capture currently sees configured additional roots/source-only authorization, not the request's complete primary-root authorization |
| Retained identity lifecycle | INCOMPLETE: source-owned neutral capture enters artifact, but into_parts currently drops it; canonical Angular facts, durable payload and consumer joins are not implemented |
| Consumer migration | No Angular family is yet certified migrated; old authority paths remain outstanding work |
| Supplemental source tests | Eight tests authored in `src/tests/ir/typescript_resolution_issue_162.rs`; NOT RUN |
| Supplemental test SHA-256 at checkpoint | `01A6CCC19BCA0C294C8E1F5D078C6552783D89AD9B54ED7071BEC60F5530F74A`; may change during ordinary supplemental development |
| Static formatting | Direct rustfmt performed on the initial native files/artifact edits; this is not compilation or test evidence |
| Runtime after implementation | NOT RUN; no compilation, warning-free or behavioral GREEN claim |

The three baseline passes mean only: direct/#157 kind-mutation control works;
the particular cycle fixture retains its independent leaf; conflicting-star
output is empty. The last does **not** establish star-conflict detection because
barrel resolution itself was absent. Positive barrel/path/standalone failures
prevent later lifecycle/default assertions from running; keep them unclassified.

## 3. Status and phase gate rules

### Mandatory canonical question gate for every phase and consumer

Apply the Canonical Question Matrix Standard before designing each material
capability and review its matrix before declaring that capability complete.
The migration matrix in section 6 is a consumer audit; it does not replace the
canonical question matrix. The initial twelve regressions do not replace it either.

For each material question, maintain the standard's section 11 review record:
question ID and real developer/LLM intent; required semantic facts; canonical
identity/authority; acquisition and dependency closure; producer and specialized
owner; retained representation; actual evaluator and public MCP/tool path;
expected answer shape; completeness classification and proof; adversarial and
lifecycle controls; cross-layer dependencies; evidence and remaining gaps.
Link those question IDs from each applicable phase and consumer migration row.

Explicitly assess all ten question classes for every layer: identity, forward,
reverse, why/provenance, scope/visibility, composition, impact/change,
completeness, lifecycle and cross-layer questions. Record architectural reasons
for irrelevant classes or omitted controls; inconvenience is not a reason.

Trace each answer from authorized source through acquisition, canonical identity,
producer, retained fact/proof, publication/persistence and evaluator to the real
public response. A proposed query path must remain marked unimplemented until
that path exists. Internal facts or lossy presentation that cannot express the
proved answer are feature gaps. New public contract decisions still follow the
approved architectural review boundary; do not disguise a gap as completion.

**Current status: the initial #162 canonical question matrix is recorded in
section 6A; implementation and public-path proof remain pending.** Extend its
records as concrete query contracts are traced. No phase or consumer can be certified complete without its applicable
question records and public-path evidence. Source-local exceptions to shared
identity mechanics do not exempt material capabilities from this standard.

Use these statuses precisely:

- **PENDING**: required work not implemented.
- **IN PROGRESS**: implementation incomplete; list the remaining boundaries.
- **IMPLEMENTED / UNVERIFIED**: production path exists and static trace is complete,
  but required operator runtime evidence has not passed.
- **VERIFIED**: captured operator evidence covers the stated gate/snapshot.
- **SOURCE-LOCAL EXCEPTION PROVEN**: exact input/output audit establishes that the
  helper supplies no identity/API/resolution/composition/dependency authority.

Never mark a phase or family complete merely because a type/helper exists or
the initial nine failures turn green. Never call an implementation pending gate
green. Record fixture/compile errors separately from behavioral failures. Keep
tracked Rust results separate from verification harness/live/Claude evidence.

For bug repairs, preserve the observed regression unchanged during the required
RED/stash/repair/restore/GREEN sequence. Supplemental tests can evolve before
their own frozen behavioral receipt; never weaken a captured oracle to get GREEN.

## 4. Phased execution

### Phase 0 — Baseline, design freeze and RED preservation

Status: **COMPLETE AND FROZEN for baseline/design/RED preservation**, by maintainer
instruction on 2026-10-21.
This is a historical operator RED receipt plus static preservation verification,
not runtime verification of the in-progress production changes.

Freeze boundary: preserve this baseline, design approval, exact regression hash,
registration snapshot and RED receipt as historical evidence. Append later
corrections or receipts explicitly rather than rewriting the frozen evidence.
This freeze does not certify or freeze the unfinished Phase 1 production work.

- [x] Capability matrix authored before production changes.
- [x] Capture and classify 3 PASS / 9 RED through public dispatch.
- [x] Maintainer approves/freezes the native and retained-identity design.
- [x] Preserve exact #162 regression and registration before production edits.
- [x] Record Angular-wide migration as a mandatory acceptance criterion.
- [x] Record the hard Canonical Question Matrix gate and initial AQ01–AQ26
  requirements, consumer mapping and pending public-query contracts in section 6A.

Restoration is a deferred Phase 7 prerequisite, not an unperformed Phase 0 action:

- [ ] Restore the exact regression and registration after the relevant repair;
  normalize only checkout line endings if necessary to recover the exact hash.

#### Phase 0 receipt — Agent MaxHeadRoom, 2026-10-21

Bounded filesystem/Git inspection reconfirmed:

| Evidence | Verified value |
|---|---|
| Current HEAD and stash base | `8320284150e45b63888f332b4c383279156a8242` |
| Frozen stash object | `6a82efdb5036df0b6d24e40266737d3d16b6885c` |
| Exact regression bytes | Stash third parent, `src/tests/mcp/workspace_query_issue_162.rs`, 12099 bytes |
| Regression SHA-256 | `B63883E818FC70DF0990D23E2C31AC9D318FED458DA74B70B693C393DBA86379` |
| Registration snapshot SHA-256 | Stash `src/mcp/tool_handlers/query.rs`: `AB3D23AAF30E4E85B1C448E74BE9B9C26C15D1A0EE494994A4A1250D6229DBE4` |
| Narrow stash content | Only tracked query.rs registration diff and untracked regression file; no unrelated changes |
| Registration | Angular-gated `tests_issue_162` module with path `../../tests/mcp/workspace_query_issue_162.rs` |
| Operator RED receipt | `target/issue-162/red.log`: 3 passed, 9 failed, 4134 filtered, 14.41 seconds |
| RED receipt SHA-256 | `CE35F3978BBEDE51CD4B02B4FB6C0392716620B2FBA964F9F54E4779AACAF1AC` |

Hashes above were calculated from raw Git blob/file bytes, not text-mode Git
output redirected through PowerShell. The original regression and registration
remain stashed and absent from the working tree. No oracle was modified and no
stash was applied, dropped or recreated. Existing foundation edits remain
in-progress work; Phase 0 did not add production changes or certify them.

Exit: proceed to Phase 1 within the frozen architecture and question contracts.
No Cargo, binary, MCP/provider process or live verification was run. Runtime
GREEN remains **NOT ESTABLISHED**. The saved RED is sufficient for preservation;
no redundant operator RED rerun is requested.

### Phase 1 — Authorized snapshot/project host

Status: **IN PROGRESS**.

Implementation tasks:

- [x] Initial immutable consumed-byte snapshots, source hash and typed witnesses.
- [x] Initial scope rejection and physical-route observations.
- [x] JSONC parsing and initial local project/extends handling.
- [ ] Carry the request's actual authorized roots/project requirement through
  compilation jobs and direct/in-memory candidate compilation. Do not infer a
  workspace from a source parent or promote a path mapping to authorization.
- [ ] Reconcile physical alias, symlink, missing-path, file/directory and access
  error observations against established path authority on all platforms.
- [ ] Handle full project ownership/config inheritance: package extends, multiple
  extends option merging, config-relative origins, include/exclude/references,
  compiler/version/mode defaults and ambiguity.
- [ ] Distinguish source-only capture from closed project authority; no missing
  version/config may silently supply a version-sensitive semantic default.

Gate: explicit-root, same-snapshot, out-of-scope/link, unreadable-vs-absent and
config-appearance/mutation tests. All IO used to prove authority is observable.
No runtime gate has run.

### Phase 2 — Canonical TypeScript lexical/module/export authority

Status: **IN PROGRESS — initial implementation only**.

Implementation tasks:

- [x] Initial source/hash/span declaration IDs and direct named/aliased bindings.
- [x] Initial named/star export traversal, rooted-cycle traversal and contender sets.
- [x] Initial relative/index, paths, package exports/imports and ordered condition capture.
- [ ] Bind actual reference occurrences by exact span and namespace, including
  lexical shadowing; a top-level name map is not the final reference API.
- [ ] Complete namespace/default/re-export forms and legal declaration merging;
  type/value eligibility must be explicit for each declaration kind.
- [ ] Complete mode fidelity: bundler versus node10/node16/nodenext/classic,
  import/require and importing-file format, substitutions and extension/index rules.
- [ ] Complete configuration-dependent lookup precedence, fallback ordering,
  packages/self-reference, exports barriers, imports external targets, type
  routing and supported compiler conditions. Do not treat the current native
  scaffold's unresolved branches as final support exclusions.
- [ ] Validate export SCC/fixed-point behavior, explicit-export precedence,
  conflicting stars versus multiple routes to the same declaration.
- [ ] Capture every negative/search/ambiguity witness that affects the answer.

Gate: conformance/adversarial tests plus exact-source/hash proof. Source-local
binding helpers must feed this boundary rather than remaining competing answers.
A file path alone never satisfies this phase's declaration identity contract.

### Phase 3 — Retained canonical evidence, snapshot and publication plumbing

Status: **PENDING beyond initial artifact capture**.

- [ ] Preserve neutral evidence and canonical fact identity across the artifact
  return boundary; remove the current drop-only handoff for migrated producers.
- [ ] Retain canonical facts **before** EntityRef/EdgeKey/query-merge dedup.
- [ ] Extend existing semantic snapshot/durable payload and compatibility
  generations coherently; do not encode witnesses in fake dependency paths.
- [ ] Extend currentness validation for typed metadata/negative/error/physical/
  project witnesses in the same lifecycle as content dependencies.
- [ ] Atomically replace canonical facts/proofs/public presentation under the
  existing source owner and publication ordering.
- [ ] Cover hydration, provide, explicit-file queries, compress, edits, delta
  transitions, restore/reopen, reset/deletion and recompilation.
- [ ] Validate surviving relevant publication after completed preparation;
  reprepare superseded work without cache-completion shortcuts or new global caps.

Gate: end-to-end creation -> returned artifact -> persistent owner -> lifecycle ->
consumer -> MCP response. Include same-name facts that old dedup would collapse.
Source cache, canonical source IR and derived authority validity remain distinct.

### Phase 4 — Angular declaration/API authority

Status: **PENDING**.

- [ ] Resolve decorators/APIs through exact TS export/declaration identity and
  the applicable Angular package/version role contract.
- [ ] Support aliases/re-exports, local shadowing, namespace references and
  same-name foreign packages without decorator/API spelling promotion.
- [ ] Include library declaration metadata where decorators are compiled away;
  metadata field/type spelling alone is not Angular authority.
- [ ] Keep source-proven declaration ownership independent of relationship count.
- [ ] Enrich legacy #157 fixture environments with explicit synthetic package/
  project witnesses, preserving assertions and historical receipt provenance.
- [ ] Reject ambiguity and retract all dependent role-specific facts when API/
  decorator/package authority changes.

Gate: canonical package/API positives and foreign/aliased/shadowed/type-only
negatives through production dispatch. The frozen #162 test is not modified.

### Phase 5 — Shared composition and visibility

Status: **PENDING**.

- [ ] One recursive expression/composition authority for NgModule and standalone.
- [ ] Direct/local/nested/imported arrays and spread/barrel/re-export chains,
  preserving each expression's lexical owner and canonical leaf identity.
- [ ] Deterministic cycle-safe traversal that retains independent proved facts
  and structured unresolved obligations, without arbitrary global depth caps.
- [ ] Explicit/version-default standalone eligibility; no current-date inference.
- [ ] NgModule declarations/imports/exports visibility and standalone imports
  produce closed owner-specific visible declaration sets.
- [ ] Membership mutation/retraction/restoration reuses exact consumed snapshots
  and retains owner identity when the relationship set becomes empty.

Gate: initial positive controls + lifecycle/order/batch/config/export retargeting;
same terminal names and selectors outside the visible universe must not leak in.
The 12-test foundation suite is necessary but not sufficient.

### Phase 6 — Angular-wide consumer migration and P0/P1 completion

Status: **PENDING for every family until section 6 records concrete evidence**.

Execute as small production slices after dependencies are available:

1. Declarations/member APIs and metadata; delegate old authority mechanics.
2. Routing, provider/DI and application root/configuration ownership.
3. Templates, actual selector visibility, host composition and view/content queries.
4. Defer eligibility and versioned modern reactive APIs.
5. NgRx/RxJS/Forms/Formly/testing integration and complete ecosystem preservation.

These are implementation groupings, not permission to defer the later groups
outside #162. Shared TS/Angular mechanics answer identity; each consumer keeps
its own semantics. Remove old authoritative paths when each replacement is
integrated; do not union weaker positives back into canonical results.

Gate: every row in section 6 has production call-path/removal/dependency/test
evidence, or a concrete source-local exception proof. No unexplained bypass remains.

### Phase 7 — Batched local acceptance and final architectural audit

Status: **PENDING / operator runtime boundary in force**.

- [ ] Restore frozen #162 test/registration and verify exact SHA-256.
- [ ] Focused foundation and consumer-family test receipts, with zero warnings.
- [ ] All applicable lifecycle/order/identity matrices in section 5.
- [ ] Preserve #152/#155/#156/#157 and coordinate #158/#159/#160/#161 obligations.
- [ ] Source audit: no competing authority mechanics, dead replacement paths,
  prematurely certified partial closure or unconsumed retained evidence.
- [ ] Full authoritative repository gate from verification.md, operator executed.
- [ ] Strict encoding and active-file size guards; no warning suppression to hide gaps.

The initial native supplemental suite is uncompiled. Do not infer GREEN or
test-selection counts from authored registrations alone.

### Phase 8 — Consolidated production-boundary MCP/field verification

Status: **PENDING**.

- [ ] Update/extend reusable verification assets under verification/, generated
  state under target/, with binary/source/input hashes and exact RPC evidence.
- [ ] Cover all applicable migrated families, not just module metadata.
- [ ] Actual binary/stdio lifecycle, batches, concurrency and persistence/reopen
  where claimed, reported separately from tracked Rust/CI results.
- [ ] Only after local questions are resolved, batch any remaining Claude-only
  field questions into the fewest practical sessions.

Exit: operator/live evidence + any necessary field evidence; full #162 completion
requires the entire migration matrix and local gates as well.

## 5. Cross-cutting acceptance matrix

Apply to every dependency-backed family, not only the resolver:

| Dimension | Required cases |
|---|---|
| Order/state | Target first; dependency first; reverse first; unrelated first; cold/warm/repeat; both batch orders; fresh state; caches populated |
| Contributor lifecycle | Initial absence/appearance; mutation; deletion/restoration; retarget/rename; alias/export/barrel changes; membership changes |
| Resolution lifecycle | tsconfig/extends/project ownership; package manifest/exports/imports/conditions; negative candidate appearance; physical route/link changes |
| Declaration/API authority | Same names/files/packages; decorator-kind changes; removed/restored role; aliased and type-only imports; local shadowing; foreign same-spelling APIs |
| Canonical joins | Same-name endpoints from one asserting file; duplicate selector candidates inside/outside owner scope; provider token identity; alias diamonds versus conflicts |
| Publication | Relevant late supersession; stale candidate rejection; final batch truth; deletion without new publication; single-flight followers independently validate |
| Persistence | Retained payload survives return/save/load; old producer evidence invalidates; reopen/edit/delta/reset/delete/restore paths remain consistent |
| Performance | Inventory/config/package probes; snapshot/parse/bind/composition work; retained reuse; warm compatibility checks; no unconditional compile-all repair |

Every positive family needs a real nonempty control. Negative or unproven results
must not be mistaken for complete support. New unresolved observations are safety
states during implementation, not acceptable permanent omissions.

## 6. Angular Meta-Layer Migration Matrix

All statuses below are the **current checkpoint**, not promised completion.
For each row, replace PENDING with exact file/function/test receipts as the work
lands. A family is migrated only when every column is satisfied.

| Producer / consumer | Current authority path | Foundation capability that feeds/replaces it | Specialized semantics retained | Old authority mechanics to remove/delegate | Dependency/publication integration | Regression / disposition |
|---|---|---|---|---|---|---|
| Detection / applicability | detect.rs decorator/API syntax gate | Canonical declarations/API evidence; syntax may nominate work only | Cheap candidate detection | Any promotion from spelling to framework role or suppression of aliases | Source/API witnesses; cannot block canonical consumers | PENDING; alias/foreign API cases |
| Decorators / declarations | decorator_scan.rs, decorators.rs, semantic.rs | Exact reference -> package export -> Angular declaration role | Class/member metadata interpretation | Name-only decorator classification as semantic authority | Role/declaration payload, version witnesses, owner lifecycle | PENDING; aliased/foreign decorator REDs |
| NgModule metadata | module_metadata.rs and sources.rs | Shared TS/export + canonical declaration + composition | declarations/imports/exports/provider meaning | Relative path helper, independent import/array leaf resolution | Complete contributor/negative closure; empty-owner restoration | PENDING; nested/barrel/provenance and #157 preservation |
| Standalone composition | No complete imports authority in this path | Same composition/visibility as NgModule | Standalone eligibility and Component imports | Any independent modern-only path/alias resolver | Version/default, module/leaf/visibility witnesses | PENDING; explicit/default standalone RED |
| Template ownership | template_ownership.rs, semantic template edge | Canonical Component role and metadata source | Inline/templateUrl ownership and ordered content | Framework role or class ownership recovered from terminal names | Exact owner/template/API bytes and lifecycle | PENDING; same-name owners, missing/restored templates |
| Selector resolution | query/angular_templates.rs indexed candidates | Closed canonical visible-declaration scope | Selector grammar/matching and ambiguity diagnostics | Workspace-wide uniqueness/candidate discovery as visibility proof | Owner/template/scope/selector closure and final preparation | PENDING; coordinate #159, scope counterexamples |
| Routing | routing/structured.rs path classifier and route roots | Exact APIs/targets and shared source/config closure | Route tree, guards/resolvers/lazy semantics | Independent relative path resolution and separate-read certification | Same-snapshot target/config proof; relevant final generation | PENDING; targets, providers, provideRouter/feature APIs |
| Providers / DI | provider_semantics.rs, inject_evidence.rs, constructor_injects.rs | Exact owner/token/API/endpoint symbols and injector scope | Consumption vs registration; useClass/useFactory/useValue/useExisting/multi | Independent token/import/package assumptions | Provider/token/config/strategy witnesses, replacement/retraction | PENDING; all owners and strategies |
| Member semantics | member_semantics.rs, invocation evidence | Exact owner/declaration/API bindings | Inputs/outputs/model aliases and admitted inject sites | Consumer-specific API binding interpretation where shared proof exists | Source/member/API proof and current owner role | PENDING; alias/shadow/owner and #160 preservation |
| Signals | signals.rs import-gated extraction | Versioned Angular API roles and exact references | Signal/computed/effect/interoperability meaning | API spelling/package assumptions as authority | API/version/producer-source evidence | PENDING; foreign APIs, resource/rxResource stability |
| NgRx | ngrx modules and query/index refinement | Shared canonical symbols/resolution and closed authority neighborhood | Qualified actions/members/reducer/selector/dispatch rules | Independent object-action path/import authority | Registration/use/definition neighborhood + surviving generation | PENDING; preserve #152, coordinate #158 |
| RxJS | rx.rs and rx extraction/pipes | Exact RxJS API/operator bindings | Stream/pipe/operator interpretation | Import/string/name shortcuts where they decide identity | External API/operator sources if needed by the claim | PENDING; imported/aliased/foreign operators |
| Reactive Forms | reactive_forms_imports.rs and forms extraction | Canonical forms APIs/builders/custom-validator symbols | Control/group/array/validator semantics | Independent alias/import/API authority | API/validator/owner source witnesses | PENDING; lookalikes and dependency mutation |
| Formly | formly.rs extraction | Exact API/config/type identities as applicable | Fields, types, expressions and specialized metadata | Package/name assumptions that establish architectural identity | Cross-file field/type/config witnesses | PENDING; typed/aliased/foreign/config lifecycle |
| Testing / TestBed | testing.rs and testing graph projection | Canonical testing APIs plus module/provider scope | Test ownership and tested/imported/provider relationships | Textual import/name discovery as final declaration identity | Test module/provider/target/config authority | PENDING; test imports/providers and same-name targets |
| Host composition | Required new hostDirectives authority | Canonical directive/member IDs + visibility/API roles | Composition and input/output exposure, host bindings/listeners | No new parallel resolver or name-based member join | Directive/member/exposure source closure | PENDING; object/direct/alias/mutation cases |
| View/content queries | Existing detection is not a query graph | Canonical owner/query API/locator/token authority | Required/plural/read and string-locator distinctions | Query spelling/terminal target lookup as identity | Owner/API/target/options proof | PENDING; legacy + signal forms |
| Application bootstrap/config | Required root/config/provider graph | Exact bootstrap API/root/config symbols | Application root, provider boundary and active configuration | Proximity or source-text bootstrap detection as authority | Root/config/provider/route/package witnesses | PENDING; modern and legacy bootstrapping |
| Defer semantics | Existing control frames/triggers | Canonical template-visible targets plus eligibility/use proof | Deferred vs placeholder/loading/error and loadability rules | Inferring deferability solely from block/tag placement | Template/target/import/reference-use universe | PENDING; deferable/eager/foreign/scope cases |
| MCP forward/reverse/graph joins | Name buckets, shared EdgeKey and merge helpers | Retained canonical endpoint indexes before public projection | Existing occurrence presentation and asserting-file scope contract | Using lossy public keys/merges as canonical Angular authority | Generation-bound canonical fact owner; relevant validation | PENDING; same-name collision/traversal cases |
| Marker / rendering helpers | markers.rs, phi.rs, ordered presentation helpers | Consume proved inputs; no independent resolver | Rendering/order/format vocabulary | Audit for hidden identity inference | Upstream authority must remain attached; rendering creates none | EXCEPTION CANDIDATE only; source-local proof not yet completed |
| Other discovered Angular consumers | Add each discovered producer/helper here | Identify exact shared capability | Name the actual domain interpretation | Trace/remove every competing authority answer | Identify persistent owner/response boundary | PENDING until inventory closes; no silent omissions |

An exception entry must cite the inspected function, its authoritative inputs,
its output, and why it answers no TS/API/declaration/scope question. Merely being
source-local is insufficient if it classifies a foreign API by spelling.

## 6A. Angular Canonical Question Matrix — hard completion gate

This matrix applies the [Canonical Question Matrix Standard](../Clean-CTX_Canonical_Question_Matrix_Standard.md).
Each ID joins the requirements table, delivery/evidence table and Section 6
consumer row. Questions are architectural requirements, not claims about current
query support. All are in-envelope work; no entry authorizes scope reduction.
The same standard governs every future Clean-CTX layer, feature and upgrade.

### Shared rules recorded for every question

Each row below inherits the following explicit contract; row-specific requirements
add to it. These shared controls are required work, not evidence already obtained.

- **Identity/authority A:** exact canonical source/declaration/reference identity,
  lexical binding, package/export API role and relevant version; internal joins
  precede lossy public projection. No name/proximity fallback.
- **Closure D:** exact consumed snapshots plus applicable transitive import/export,
  project/config/package, negative lookup and ambiguity witnesses; authorized roots;
  retained proof in the actual producer owner; final surviving publication checked.
- **Completeness C:** exact only when the question's required universe and semantic
  obligations are closed and current; proved partial facts are lower-bound with
  missing obligations; missing necessary identity is unresolved; competing proved
  identities are ambiguous. An empty answer proves absence only with closed scope.
  Do not confuse a single resolved endpoint with a complete reverse neighborhood.
- **Adversarial N:** nonempty positive, foreign same-spelling API, same-name distinct
  declarations, shadowing/type-only bindings, alias/barrel conflict and missing
  evidence as applicable. Each delivered question needs explicit test mapping.
- **Lifecycle L:** mutation/retraction/replacement, deletion/restoration, rename,
  alias/export retarget, relevant config/package/version changes; cold/warm/repeat,
  target/dependency/reverse/unrelated-first and both batch orders; durable reopen
  and incompatible-generation rejection for persisted answers. Test surviving
  publication after supersession. Any omitted control needs a recorded architectural
  reason per question; none is waived by this initial matrix.
- **Evidence E:** tracked public-dispatch regression IDs plus distinct operator
  runtime receipts; actual-binary/MCP evidence where applicable. Unit construction
  proves mechanics only. No question is PROVED at this checkpoint.

Question classes: I identity; F forward; R reverse; W why/provenance; S scope;
O composition; X impact; C completeness; L lifecycle; Z cross-layer. C/L apply to
every row. Reverse and impact questions require affected-neighborhood proof;
forward closure alone cannot certify them.

### Questions, intent, required facts and proof

| ID / Section 6 feature / classes | Canonical question and developer/LLM intent | Required semantic facts | Required authority / acquisition closure beyond A/D |
|---|---|---|---|
| AQ01 Detection + decorators/declarations / I,W | What Angular role does this declaration actually have, and why? Distinguish framework declarations from lookalikes without reading source. | Reference-to-export-to-declaration role, owner/span, metadata, version; candidate detection is not semantic admission | Decorator/API bindings, package exports, shadowing and complete role eligibility; aliases must reach the same authority |
| AQ02 NgModule / F,R,W,O | What does this NgModule declare/import/export, and which modules declare or export this declaration? Explain ownership and membership paths. | Typed membership, canonical endpoints, recursively composed arrays, declaring lexical context, provenance paths | Module/declaration roles, supported expression closure, cycle obligations; reverse module inventory for reverse answer |
| AQ03 Standalone / F,W,O,S | What does this standalone component import, and why are those imports eligible? Explain actual composition. | Explicit/default standalone status, version evidence, component imports and module/standalone eligibility | Version/package defaults, recursive metadata and referenced declaration/module closure; Directive/Pipe do not gain Component imports |
| AQ04 Template ownership / I,F,R,W | Which template belongs to this component, and which component owns this template? Find the actual owner and source. | Canonical owner, inline/external template identity and source span, ownership relation | Component role, metadata expression and exact HTML/source snapshots; reverse owner inventory for shared/multiple owners |
| AQ05 Visibility + selectors / S,O,W,Z | What declarations are visible to this component, and which imports/exports make each visible? Explain availability instead of global candidate matches. | Owner composition, module export closure, standalone eligibility, visible canonical declaration set and witness paths | Closed owner-specific visible universe including re-exports/cycles; distinguish raw imports from Angular visibility |
| AQ06 Selectors / I,F,W,S,Z | What does this selector occurrence resolve to here, and why can that component/directive be used here? Resolve a concrete occurrence. | Exact template occurrence/owner, selector grammar, visible candidates and canonical match identities | AQ04/AQ05 plus complete scoped selector catalog; duplicates outside scope cannot create ambiguity |
| AQ07 Pipes / I,F,R,W,S,Z | Which pipe does this template expression use, why is it visible, and where is this pipe used? Explain pipe dependencies. | Pipe name/API role, template expression occurrence, scoped pipe candidates, usage relation | Canonical Pipe authority and visible pipe universe; reverse template-use acquisition, not component-tag matching |
| AQ08 Providers / F,W,S,O | Where is this token provided and what registration satisfies it in this injector scope? Explain strategy and applicable scope. | Token identity, registration owner, hierarchy, useClass/useFactory/useValue/useExisting, multi, endpoint and precedence | Full applicable component/directive/module/route/application/environment provider closure and token namespace; distinguish alias from construction |
| AQ09 DI consumption / R,F,W,X,Z | What consumes this service/token, and what breaks if its registration is removed? Assess scoped dependency impact. | Constructor/inject sites, canonical token, injector resolution paths, dependent consumers and unsatisfied obligations | AQ08 plus closed affected consumer/injector neighborhood; runtime-conditional resolution must retain uncertainty |
| AQ10 Members / I,F,R,W | What inputs, outputs and model bindings does this owner expose, under which aliases, and who binds them? Explain public member use. | Owner/member IDs, decorated/signal API roles, aliases, required/model directionality, binding occurrences | Exact owner/member APIs, inherited/exposed members where applicable, templates and visibility; preserve #160 owner identity |
| AQ11 Signals / I,F,W,X | Which signal/computed/effect/resource APIs and dependencies are proved here, and what changes if this source changes? Explain admitted reactive structure. | Exact API/use identities, owner, explicit dependency evidence, version/stability and lifecycle | Canonical core/interoperability APIs; prove dependency expressions rather than infer a runtime reactive graph; version-gate resource/rxResource |
| AQ12 Routing targets / F,R,W,O,Z | What component/module does this eager/lazy route actually load, and which routes load this declaration? Explain runtime architecture. | Route owner/tree, eager/lazy expression, selected export, target role and reverse route uses | Canonical router API, lazy import/export/config resolution, route tree plus reverse route inventory |
| AQ13 Routing neighborhood / F,W,S,O | What guards, resolvers and providers apply to this route, and through which configuration? Explain effective route scope. | Parent/child route context, guard/resolver targets, route providers, provideRouter features and active config | Route/config/bootstrap linkage, exact API/token identities, applicable inherited configuration and provider closure |
| AQ14 Bootstrap/configuration / I,F,W,O,Z | What is the application's actual root/bootstrap composition and active configuration? Distinguish applications sharing a workspace. | Actual bootstrap invocation, resolved root module/component, merged config/providers/routes and application boundary | Canonical bootstrapApplication/bootstrapModule APIs and config/import/merge witnesses; never choose another app by proximity |
| AQ15 Host composition / F,R,W,O | What host directives compose this component/directive, what members are exposed, and where is this directive composed? Explain host architecture. | Canonical host directive IDs, composition paths, input/output exposure mappings, host property/event ownership | Host metadata and directive/member eligibility/aliases; full host composition and reverse owner neighborhood |
| AQ16 View/content queries / F,W,S | What do these view/content queries target and what do read/required/plural mean here? Explain static query intent without inventing runtime results. | Owner/query site/API, locator vs declaration/token, read token, required/plural/view/content flags | Legacy/signal API authority, token/locator distinctions and relevant template/composition proof; runtime result set may remain unresolved |
| AQ17 Defer / F,W,O,X,Z | What is deferred versus eagerly required here, and why? Explain eligibility and blockers, not just block placement. | Body/placeholder/loading/error frames, triggers, referenced targets, standalone status and eager uses | Owner/template scope and reference-use closure sufficient for Angular defer eligibility; no bundle/runtime loading claim from syntax alone |
| AQ18 Control flow / F,W,O | Under which if/for/switch/defer branches does this occurrence appear? Preserve structural context in answers. | Ordered occurrence spans and complete control ancestry, branches, triggers and template owner | Exact template snapshot and parser structure; malformed syntax obligations; no inferred branch evaluation |
| AQ19 NgRx / F,R,W,O,Z | Which actions trigger which reducers/effects, which selectors derive from which state, and why? Trace state architecture and consumers. | Qualified action/member IDs, registration, reducer/effect/selector composition, dispatch/select sites | Shared TS identity plus specialized #152 exact object/member and registration authority; #158 neighborhood closure remains required coordination |
| AQ20 RxJS / I,F,R,W | Which observable/operator chains and subscriptions are proved, and where are they consumed? Explain stream structure without claiming runtime emissions. | Exact RxJS API/operator references, pipe order, admitted producer/consumer sites | Canonical RxJS exports and explicit expression/use evidence; do not infer dependencies from equal operator names |
| AQ21 Reactive Forms / F,R,W,O | Which controls/groups/validators compose this form and where are they bound or consumed? Explain form architecture. | Control/group/array hierarchy, builder/API roles, validator endpoints and template bindings | Canonical forms/custom-validator IDs, config and template-owner closure; distinguish runtime-created controls from proved static facts |
| AQ22 Formly / F,R,W,O | Which field types/configurations/expressions compose this form and which registrations satisfy them? Explain dynamic-form configuration. | Field/config source, type registrations, expression ownership and uses | Canonical Formly APIs/type/config references and registration neighborhood; arbitrary runtime expressions stay explicitly unresolved |
| AQ23 Testing/TestBed / F,R,W,S,O | What declarations/providers/imports does this test configure or override, and which tests exercise this declaration/token? Explain test scope. | Test owner, TestBed calls, imports/providers/overrides, tested targets and effective configuration | Canonical testing APIs plus shared module/provider authority and reverse test inventory; production and test injector scopes remain distinct |
| AQ24 Cross-layer impact / R,W,X,Z | What breaks semantically if this import/export/provider/configuration/declaration is removed? Trace affected facts and explain each dependency. | Canonical changed fact, incoming proof dependencies, affected owners/consumers, reason paths and invalidation boundaries | Closed requested affected neighborhood across layers; negative resolution and visibility witnesses; explicitly lower-bound when dependents are undiscovered |
| AQ25 Public joins/completeness / I,R,W,C,Z | Are these same-named endpoints actually the same declaration, and is this answer exact/current? Let the caller assess the answer safely. | Canonical endpoint discrimination, retained witness paths, scope/generation and unresolved/ambiguity obligations | Pre-projection canonical indexes, query-specific closure, publication/currentness and response preserving distinctions |
| AQ26 Marker/rendering helpers / W,C,Z | Does this compressed/rendered answer preserve the proved relationship, owner, provenance and uncertainty? Consume facts without reconstructing source. | Ordered semantic presentation, stable endpoint association, proof/classification carried through rendering | Upstream canonical facts plus loss audit; source-local rendering exemption does not waive answer correctness |

### Producers, query contracts, answer shape and evidence ledger

Existing operations were checked in `query/request.rs`: `workspace_query` exposes
`find_entities`, `entities_in_file`, `declarations_in_file`, `forward_edges`,
`reverse_edges`, `transitive_dependencies`, `has_cycle` and `calls_in_file`.
Their existence does **not** prove they express the answers below. F/R denotes
existing forward/reverse operations as candidate carriers only. Each row records
the evaluator that must be traced or implemented, not a completed integration.
No new operation names or public schema are approved by this document.

**GAP** means the required answer contract is absent or not yet demonstrated;
implementation must distinguish absence from inadequate presentation and close
the gap. **PARTIAL** means baseline behavior exists but required authority or
answer semantics are incomplete. Both prevent completion. All rows require
completeness C and controls N/L/E above, plus their specific controls below.

| ID | Producer(s) -> consumer/evaluator | Public MCP/tool path and contract disposition | Expected answer shape | Specific adversarial/lifecycle obligations; current regression/public-path evidence |
|---|---|---|---|---|
| AQ01 | detection/decorators + canonical API authority -> entity/edge evaluators | workspace_query entity operations + F/R; PARTIAL role/provenance contract | Declaration ID -> Angular role -> API/export witness, classification | Alias vs foreign decorator, role change; frozen alias/foreign/provenance tests are RED, not migrated proof |
| AQ02 | shared composition/module metadata -> edges/graph | workspace_query F/R/transitive_dependencies; PARTIAL membership, GAP complete ownership/why contract | Owner -> typed membership -> declaration + composition path | Nested/cyclic arrays, barrel retarget, empty-owner restoration; frozen nested/barrel RED and limited cycle PASS; #157 baseline only |
| AQ03 | standalone shared composition -> edges/visibility evaluator | workspace_query F/R candidate; GAP eligible composition/default explanation | Component -> import -> canonical target -> eligibility/version reason | Explicit/default/version mutation; frozen standalone RED, default iteration not reached |
| AQ04 | template ownership -> angular_templates + reverse ownership consumer | workspace_query F/R candidate; PARTIAL ownership, GAP canonical reverse/proof certification | Owner ID <-> template ID/source span with ownership reason | Shared HTML/same-named owners, deleted/restored template; no #162 acceptance evidence |
| AQ05 | shared visibility -> angular_templates/scoped catalog | workspace_query F/R candidate; GAP closed visible-set/path answer | Owner -> visible declarations -> import/export witness paths + closure | Unrelated modules, re-export removal, standalone/module retarget; no #162 acceptance evidence; coordinate #159 |
| AQ06 | selector matching + AQ04/05 -> angular_templates | workspace_query existing template integration requires exposure trace; GAP scope-authoritative answer | Occurrence -> scoped matches -> canonical declarations + why/status | Duplicate selector inside/outside scope, changed selector/owner; no #162 acceptance evidence |
| AQ07 | pipe/template expression semantics -> scoped pipe-use evaluator | workspace_query F/R candidate; GAP scoped pipe answer/reverse use | Pipe expression -> canonical Pipe -> visibility path; incoming uses | Duplicate pipe names, foreign Pipe, export removal; tests/public path pending |
| AQ08 | provider semantics + composition -> injector resolution evaluator | workspace_query F/R candidate; GAP effective-provider/strategy/scope answer | Consumer scope -> token -> registrations -> strategy/endpoint + precedence | useExisting cycles, multi, same-name tokens, route/app boundaries; tests/public path pending |
| AQ09 | constructor/inject member sites + AQ08 -> DI reverse/impact evaluator | workspace_query reverse_edges/transitive_dependencies candidate; GAP scoped consumers/impact proof | Token/registration -> scoped consuming sites -> consequences/reasons | Removed registration, overridden token, unindexed consumers; tests/public path pending |
| AQ10 | member semantics + templates/host -> member-use evaluator | workspace_query F/R candidate; PARTIAL extraction, GAP canonical alias/binding answer | Owner -> member ID -> public alias/direction/required -> uses | Shadowed APIs, inherited/exposed aliases, #160 owner collisions; preservation and public tests pending |
| AQ11 | signals/member semantics -> reactive facts evaluator | workspace_query F/R candidate; PARTIAL markers, GAP proved reactive dependency answer | API/use -> owner -> admitted dependency paths + version/limits | Foreign computed/effect, resource version change, dynamic dependency uncertainty; tests/public path pending |
| AQ12 | structured routing + canonical resolver -> route edge/reverse evaluator | workspace_query F/R candidate; PARTIAL routes, GAP canonical lazy/why closure | Route path/owner -> lazy/eager reference -> selected canonical target | Barrel/package retarget, lazy export ambiguity, deletion; routing same-snapshot/public tests pending |
| AQ13 | routing/provider/bootstrap semantics -> effective route evaluator | workspace_query F/R candidate; GAP active route neighborhood | Route -> ancestor/config path -> guards/resolvers/providers/features | Parent route change, provideRouter lookalike, config replacement; tests/public path pending |
| AQ14 | bootstrap/config producer -> application composition evaluator | workspace_query F/R candidate; GAP root/config answer | Bootstrap site -> application root -> config/provider/route composition | Two apps, merged config mutation, foreign bootstrap API; tests/public path pending |
| AQ15 | host composition/member producer -> host forward/reverse evaluator | workspace_query F/R candidate; GAP host/exposure answer | Owner -> directive -> exposed source/public members + composition path | Duplicate aliases, foreign directive, member/export mutation; tests/public path pending |
| AQ16 | query semantics/template scope -> query intent evaluator | workspace_query F/R candidate; GAP typed locator/read/query answer | Query site -> kind/locator -> proved target/read + static limits | String locator vs token, plural/required, read retarget; tests/public path pending |
| AQ17 | defer + reference-use/visibility -> defer eligibility evaluator | workspace_query F/R candidate; PARTIAL frames, GAP eligibility/blocker answer | Block/occurrence -> deferred/eager target -> eligibility/blocking proofs | Eager use outside block, placeholder use, nonstandalone target, version changes; tests/public path pending |
| AQ18 | template_ordered/template_occurrences -> template rendering/evaluator | Existing template presentation requires public-tool trace; PARTIAL structured ancestry | Occurrence -> ordered control-frame ancestry with spans/unknowns | Nested/malformed blocks, else/empty/case/loading/error; existing structure is baseline, #162 public evidence pending |
| AQ19 | NgRx specialized producers + canonical authority -> NgRx edge/graph evaluators | workspace_query F/R/transitive_dependencies candidate; PARTIAL relations, GAP full canonical why/neighborhood | Dispatch/action -> reducer/effect -> state/selector -> consumers + registration proof | Same literal/different action, shorthand/member, changed registration; preserve #152, coordinate #158; no #162 acceptance evidence |
| AQ20 | rx extraction -> stream-use evaluator | workspace_query F/R candidate; PARTIAL markers, GAP architectural chain/consumer answer | Canonical stream/site -> ordered operators -> admitted subscriptions/uses | Aliased/barrel operators, foreign pipe, dynamic chain, removal; tests/public path pending |
| AQ21 | forms producers + template binding -> form composition/use evaluator | workspace_query F/R candidate; PARTIAL markers, GAP control/validator/use answer | Form -> controls -> validators -> template/consumer bindings | Same-name validators, changed builders/config, dynamic controls; tests/public path pending |
| AQ22 | Formly specialized producer -> field registration/use evaluator | workspace_query F/R candidate; PARTIAL markers, GAP field/type resolution answer | Field/config -> registered type/expressions -> owner/use path | Competing type registrations, imported config mutation, foreign API; tests/public path pending |
| AQ23 | testing/TestBed + composition/providers -> test-scope/reverse evaluator | workspace_query F/R candidate; PARTIAL testing extraction, GAP effective test scope | Test -> configured/overridden declarations/providers -> canonical tested targets | Override/removal, same-name production vs test targets, imported config; tests/public path pending |
| AQ24 | cross-layer retained facts/dependency owner -> reverse/impact evaluator | workspace_query reverse_edges/transitive_dependencies candidate; GAP closed semantic impact/reason answer | Changed authority -> affected facts/consumers -> reason paths + scope/classification | Missing dependent inventory, new negative candidate, warm/durable stale reverse results; tests/public path pending |
| AQ25 | canonical fact owner/publication -> entity/edges/edge_merge/graph | workspace_query existing operations; GAP canonical discrimination and per-answer closure retention | Distinct endpoint identities + evidence/currentness + exact/partial/unknown status | Same-file/name collisions, superseded publication, durable producer generation; frozen provenance RED; no complete answer proof |
| AQ26 | markers/ordered rendering/compression -> public presentation consumer | Public response route must be traced per renderer; GAP preservation audit | Same proved answer with owner/identity, control order, provenance and uncertainty preserved | Compression merges same names, drops unknowns/why, reordered controls; tests/public path pending |

Every material question includes its own forward/reverse/why subquestions above;
test receipts must identify which subquestion they prove. Split IDs when distinct
query contracts or completeness proofs emerge. In particular, do not certify an
entire row from a forward-only test while its reverse question is unanswered.

### Section 6 migration-to-question index and phase gates

| Section 6 migration row | Required question IDs |
|---|---|
| Detection / applicability; decorators / declarations | AQ01, AQ25 |
| NgModule metadata | AQ02, AQ05, AQ24 |
| Standalone composition | AQ03, AQ05, AQ24 |
| Template ownership | AQ04, AQ06, AQ18 |
| Selector resolution | AQ05, AQ06, AQ07, AQ24 |
| Routing | AQ12, AQ13, AQ14, AQ24 |
| Providers / DI | AQ08, AQ09, AQ24 |
| Member semantics | AQ09, AQ10, AQ16 |
| Signals | AQ10, AQ11 |
| NgRx | AQ19, AQ24 |
| RxJS | AQ20 |
| Reactive Forms | AQ21 |
| Formly | AQ22 |
| Testing / TestBed | AQ23 |
| Host composition | AQ10, AQ15 |
| View/content queries | AQ16 |
| Application bootstrap/config | AQ08, AQ13, AQ14 |
| Defer semantics | AQ05, AQ17, AQ18 |
| MCP forward/reverse/graph joins | AQ24, AQ25 and every family answer they expose |
| Marker / rendering helpers | AQ18, AQ26 and every upstream question rendered |
| Other discovered Angular consumers | Assign/extend IDs before implementation; inventory remains open, no exemption |

Phases 1–3 must preserve authority/closure/retained representation for all IDs,
especially AQ01/AQ25. Phase 4 gates AQ01 and family API roles. Phase 5 gates
AQ02–AQ07. Phase 6 gates all remaining families and cross-layer questions.
Phases 7–8 require per-question regression/public-path receipts and applicable
operator/live evidence. A phase cannot certify a question assigned to a later
phase merely because its underlying mechanics exist.

Acceptance for each migration row is **ANSWERABLE + AUTHORITATIVE + QUERYABLE
THROUGH PRODUCTION + CURRENT + CORRECTLY CLASSIFIED FOR COMPLETENESS**.
Implemented but not queryable is incomplete; queryable without authority is
incorrect; authoritative but not current is stale truth. Provable semantics
without an expressive public contract are a query-contract implementation gap.
If the LLM must reconstruct claimed semantics from raw source, the feature is
incomplete. Truthful degradation never substitutes for an in-envelope capability.

## 7. Immediate next actions

0. Refine section 6A's initial question contracts against actual producers and
   evaluators; replace candidate query carriers with traced production paths,
   map exact regressions to each subquestion, and close query-contract gaps.
   Keep the question index synchronized with every Section 6 migration receipt.
1. Complete primary-root/project authorization plumbing through all compilation
   entry points before any consumer trusts the new resolver.
2. Finish source/reference/project/mode conformance gaps and add targeted tests;
   no known incomplete branch may be hidden behind a support label.
3. Transport retained identity and typed witnesses across actual publication and
   persistence boundaries, preserving canonical facts before lossy presentation.
4. Wire Angular API roles and shared composition, then migrate consumers using
   section 6 as an executable work ledger.
5. Restore/hash the exact #162 regression when ready for operator verification.
   Do not mistake the current stashed test selection for foundation acceptance.

There is no immediate runtime command to run for the current incomplete checkpoint.
At the next handoff, supply the exact selected command and expected test inventory
for the restored snapshot. The normal focused command will be:

```powershell
New-Item -ItemType Directory -Force target/issue-162 | Out-Null
cargo test --quiet --all-features issue_162_ -- --test-threads=1 *> target/issue-162/green.log
$LASTEXITCODE
Get-Content target/issue-162/green.log -Tail 140
```

This is a **future operator handoff**, not authorization to execute it now and
not evidence of GREEN. The eight authored supplemental tests would add to the
twelve frozen tests after restoration; further authoring may change that total.

## 8. Update and completion discipline

For each implementation slice, append a short receipt: date, exact files,
production call path reached, duplicate authority removed, retained owner,
witness/publication behavior, tracked test names, static checks and operator
results by category. Update phase status and the corresponding migration rows
together. Preserve historical failed/blocked receipts rather than overwriting
them with a green label.

Completion requires all applicable rows migrated or source-local exception
proven, the entire material P0/P1 envelope implemented, no competing authority
paths, full production lifecycle integration, captured local gates, and applicable
consolidated live/field acceptance. Any open required row means **#162 INCOMPLETE**.
It also requires reviewed canonical question coverage: authoritative, current,
queryable answers with correct completeness classification and public-path
evidence for every material required question. An unanswered in-envelope question
is unfinished work, even if its internal semantic implementation passes tests.
