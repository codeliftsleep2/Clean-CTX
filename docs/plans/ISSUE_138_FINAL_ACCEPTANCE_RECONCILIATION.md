# Issue #138 final acceptance reconciliation and residual repair

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Issue:** [#138 — Angular/TypeScript: dependency injection forms and member decorators](https://github.com/codeliftsleep2/Clean-CTX/issues/138)  
**Verdict:** **#138 COMPLETE REPAIR — ACCEPTANCE GREEN; CI GATE PENDING**

## 1. Executive verdict

Issue #138 now has a bounded implementation for every closure criterion. The completed #138a Pipe constructor-DI and #138b direct property-initializer `inject(TOKEN)` slices remain intact; this repair adds durable provider-registration semantics, structured signal semantics, functional DI ownership, and canonical classic member identity.

The architecture gate was explicitly approved on 2026-10-15. Provider registration now has a distinct durable relation family and does not overload `Injects`; semantic generation was bumped for persistence compatibility. Focused native verification and the consolidated live/MCP acceptance harness are green by user report. The repository-wide gate remains CI-owned and was not run locally.

## 2. Original contract

The authoritative GitHub issue was read directly on 2026-10-15. Its acceptance criteria are:

1. Pipe constructor injection produces an edge.
2. Component, service, and functional guard `inject(Foo)` produce an edge to `Foo`, never a junk `?` identity.
3. `input()`, `input.required()`, `model()`, and `output()` produce `HasInput` / `HasOutput`.
4. A setter input is named by its property.
5. `InputField` names omit definite-assignment `!`.
6. Provider `useClass` produces a relation or is explicitly unsupported.

The issue's Expected paragraph is broader than closure acceptance. `@Inject`, provider variants, standalone imports, host/view decorators, aliases, and required metadata still require truthful classification, but do not independently expand the six closure criteria.

## 3. Acceptance matrix

Only the required status vocabulary is used.

| Criterion / form | Status | Evidence and disposition |
|---|---|---|
| Component/Service/Directive constructor DI | SATISFIED | Established shared typed-constructor producer and Angular projection; historical evidence only. |
| Guard/resolver/interceptor class constructor DI | SATISFIED | Original issue records these as working guards; not rerun. |
| Pipe constructor DI | SATISFIED | #138a commit `b2b84bb8`; tracked regressions; historical live 56/56 and preservation 125. |
| Component/Service direct property `inject(Foo)` | SATISFIED | #138b commit `8520c2ae`; structured exact-initializer evidence; no receiver-name target or `?`. |
| Optional second argument on supported property initializer | SATISFIED | First symbolic argument remains the token; runtime optionality is not inferred. |
| Symbolic bare token on supported property initializer | SATISFIED | Existing symbolic Service endpoint, without manufacturing a declaration. |
| Receiver/wrapper/conditional `inject` negatives | SATISFIED | #138b rejects receiver calls and non-exact initializers. |
| Functional guard `inject(Foo)` | SATISFIED | Exact callable ownership, proven Angular functional role, direct-free invocation, and resolved first argument produce `Injects`. |
| Functional resolver/interceptor `inject(Foo)` | SATISFIED | The same role-owned path supports `ResolveFn` and `HttpInterceptorFn` without adding Angular meaning to generic invocation evidence. |
| Constructor-body `inject(Foo)` | UNSATISFIED | Angular interpretation consumes exact property-initializer evidence only. |
| `@Inject(TOKEN)` / `InjectionToken` | UNSATISFIED | Typed parameter survives decorators, but decorator-argument token identity is not established. |
| Provider shorthand / `useClass` | SATISFIED | Proven Component/NgModule provider contexts publish `RegistersProvider` plus `ProviderUsesClass`; unresolved references publish nothing. |
| `useFactory` / `useValue` | SATISFIED | Representable proven endpoints use distinct `ProviderUsesFactory` / `ProviderUsesValue` relations. |
| Standalone `imports` | OWNED ELSEWHERE | Not closure acceptance; no standalone Component contract was established here. |
| `input()` | SATISFIED | Exact property initializer plus `@angular/core` import authority produces `HasInput`. |
| `input.required()` | SATISFIED | Structured member-call evidence binds the `input` receiver import and exact initializer. |
| `model()` | SATISFIED | Proven exact initializer produces both `HasInput` and `HasOutput`. |
| `output()` | SATISFIED | Proven exact initializer produces `HasOutput`; local/receiver/nested same-spelling forms remain negative. |
| Classic `@Input` family | SATISFIED | Declaration-owned canonical member parsing covers ordinary fields, accessors, modifiers, optional and definite-assignment punctuation. |
| Classic ordinary `@Output` | SATISFIED | Existing property path; weak historical recall fixture was not recovered or rerun. |
| Setter input property name | SATISFIED | Accessor keyword is skipped and the declared property name is retained. |
| Definite-assignment input name | SATISFIED | `!` and `?` declaration punctuation is excluded from canonical identity. |
| Input/output aliases and required metadata | UNSATISFIED | Alias is partial marker metadata; object metadata lacks a semantic contract. Canonical identity must remain the property name. |
| Host/view decorators and `viewChild()` | OWNED ELSEWHERE | Broader Expected capability, not closure acceptance; no canonical relationship currently exists. |
| Historical real-file anomaly | UNSATISFIED | GitHub supplies no file identity and bounded evidence did not recover it. |

## 4. Current Angular semantic authority chain

```text
TypeScript source
  -> tree-sitter canonical captures and invocation evidence
  -> Angular file/declaration classification
  -> exact class/member/initializer occurrence
  -> Angular-local reference/token interpretation
  -> typed SemanticEdge
  -> compiler result semantic edges
  -> WorkspaceIndex occurrence/edge ownership
  -> registered MCP query projection
```

Current authorities are deliberately separated:

- Declaration kind comes from Angular class decorators.
- Class identity comes from canonical class capture and the existing owner pairing, never class proximity.
- Initializer and invocation spans, direct-free-call status, and argument roles come from canonical invocation evidence.
- Constructor type identity comes from `angular_meta::constructor_injects`.
- Direct property-initializer DI interpretation belongs to Angular `inject_evidence`, not generic invocation projection.
- Classic decorator syntax classifies the member; its TypeScript declaration must supply canonical member identity.
- Signal role comes from exact initializer invocation evidence joined to canonical property ownership and an `@angular/core` named-import binding.
- Alias and required data are not semantic endpoint identity.
- Provider and functional-DI endpoints reuse existing import and local declaration-reference authority; unresolved provider endpoints publish no relation.
- WorkspaceIndex remains domain-blind and owns persistence/query lifecycle only after relationship construction.

The established production lifecycle is producer evaluation → compiler result → workspace owner → refresh/lifecycle replacement → query projection. Missing forms fail before canonical relationship construction.

## 5. Constructor DI reconciliation

`ANG-DI-001` controls this path. A typed constructor parameter inside a recognized Angular DI declaration supplies the written dependency identity; modifiers and formatting are irrelevant. Ordinary non-Angular TypeScript classes remain a required negative.

#138a adds Pipe at the Angular projection boundary and deliberately reuses only the constructor channel. This avoids admitting the historically contaminated functional-field receiver evidence. Component, Service, and Directive behavior is unchanged.

Historical evidence retained, not rerun:

- Pipe live acceptance: 56/56.
- Preservation: 125.
- Focused Pipe/compatibility: 9.

## 6. `inject` reconciliation

#138b remains intact. Its supported authority path is:

```text
canonical class
  -> exact property declaration
  -> exact initializer span
  -> direct free inject occurrence
  -> symbolic first argument
  -> Angular-local admitted token
  -> Injects edge
```

It rejects `obj.inject`, `this.inject`, indexed/member calls, and nested wrapper, conditional, or parenthesized initializer forms. An optional second argument does not change the first-argument token. An unresolved bare token remains a symbolic endpoint; it does not create a declaration or select a same-named class.

Constructor-body `inject()` remains outside this slice. Functional guards, resolvers, and interceptors now join exact callable ownership, proven Angular functional-role imports, direct-free invocation evidence, and a resolved first argument; generic invocation core remains Angular-blind.

Historical evidence retained, not rerun:

- Candidate: 131/131.
- Preservation: 180.
- Focused additions: +11.

## 7. `@Inject` / token semantics

Current constructor extraction can see the typed parameter after parameter decorators. This preserves typed constructor DI under the existing invariant; it does not make the `@Inject` decorator argument the target.

A truthful token override requires all of:

- Proven Angular constructor context.
- Exact parameter occurrence.
- `@Inject` decorator identity.
- Decorator-argument reference.
- An authorized target model.

Imported classes, declared `InjectionToken` constants, strings, and complex expressions cannot be collapsed into one Service identity. No such residual was changed.

## 8. Provider semantics

### Approved architectural decision

Approved: represent proven provider registration through a durable Angular relation family, separate from dependency consumption.

### Why completion requires it

Criterion 6 requires a `useClass` relation or explicit unsupported behavior. The current public relation enum contains `Injects`, module import/export/declaration relationships, and member relationships, but no provider-registration relationship. Provider registration is not evidence that the declaring Module or Component injects either the token or implementation.

### Affected systems and contracts

- Angular Module and Component producers.
- `SemanticRelation` serialization and durable semantic generation.
- WorkspaceIndex forward/reverse edge consumers.
- MCP query results and clients matching relation names.
- Persistence compatibility and live provider fixtures.
- `useFactory` and `useValue`, whose endpoints differ from `useClass`.

### Alternatives

1. **Add a durable Angular provider relation family.** Use explicitly typed token and implementation endpoints. This is truthful and queryable, but changes the public/durable graph vocabulary and requires lifecycle, compatibility, and client-impact review.
2. **Add Angular-scoped unsupported/capability diagnostics.** This avoids a false relationship but needs a defined public diagnostic projection. Generalizing it into a global coverage schema is expressly #142 and outside scope.
3. **Map `useClass` to `Injects`.** Rejected: it manufactures consumer-injection semantics and cannot truthfully represent `useFactory` or `useValue`.
4. **Document unsupported only in this report.** Safe as an interim disposition, but it does not make a fixture-facing semantic operation explicitly report unsupported.

### Implemented disposition

Alternative 1 was explicitly approved and implemented. `RegistersProvider` preserves registration scope and token identity; `ProviderUsesClass`, `ProviderUsesFactory`, and `ProviderUsesValue` preserve strategy-specific endpoints. `Injects` remains dependency-consumption evidence only.

## 9. Signal input/output semantics

The former spelling scanner was removed. Signal semantics now consume canonical invocation evidence and declaration ownership, including exported-class decorator ownership, without manufacturing member identity.

The implemented Angular-local projection joins canonical property identity, exact direct initializer invocation, and an `@angular/core` import binding. `input` / `input.required` map to `HasInput`, `output` maps to `HasOutput`, and `model` maps to both. Receiver calls, locally shadowed names, and nested non-exact initializers remain negative.

## 10. Classic Input/Output decorators

Decorator syntax classifies a member, while declaration syntax supplies identity. The repaired scanner skips legal modifiers and accessor keywords, captures the canonical property/accessor name, removes optional/definite-assignment punctuation, and refuses an edge without a valid declaration identity. Alias remains metadata rather than replacement identity, and Output uses the same declaration-name authority.

## 11. Host/view decorators

`@HostBinding`, `@HostListener`, `@ViewChild`, `@ContentChild`, and `viewChild()` are not DI. The detector recognizes several spellings, but the public relation vocabulary has no established host/view relationship. They are broader Expected capabilities rather than one of the six closure criteria. No unrelated fact should be manufactured merely to make the inventory green.

## 12. Unsupported/truthfulness behavior

Workspace queries distinguish established semantic capability from indexed evidence at the query level, but no Angular construct-specific diagnostic says that a recognized provider/host/view form is intentionally unsupported. Absence of an edge therefore cannot be promoted to a complete absence claim.

Creating repository-wide syntax coverage or unsupported policy is #142 and remains prohibited. Provider acceptance is satisfied through durable relations, so no unsupported-diagnostic framework was introduced.

## 13. Historical real-file anomaly

The issue describes one real component with zero extracted inputs and a skeleton leaking `@Input(` into field names, but does not name the repository or file. No recoverable file identity was found in bounded issue, history, or repository evidence. The failure is not guessed, marked repaired, or attributed to #139. The reproducible setter, definite-assignment, and signal-name defects are covered by tracked regressions and the green live harness; the unnamed historical source itself remains unrecoverable.

## 14. Repairs implemented

Implemented the approved bounded repairs: provider relation vocabulary and Angular-owned projection; semantic-generation bump and compatibility regression; structured signal projection; functional guard/resolver/interceptor DI projection; and classic setter/definite-assignment identity normalization. Existing #138a and #138b behavior was preserved.

## 15. Regression inventory

Existing tracked authorities include:

- `src/tests/angular_meta/constructor_injects.rs`.
- `src/tests/angular_meta/constructor_di_edges.rs`.
- `src/tests/ir/angular_inject_evidence.rs` and capture tests.
- `src/tests/mcp/angular_di_pipe.rs` and compatibility.
- `src/tests/mcp/angular_di_inject_token.rs`, matrix, and compatibility.
- General Angular decorator and semantic tests.

New tracked regressions cover provider scope/strategy truthfulness, unresolved-provider negatives, semantic-generation compatibility, signal positives and negative twins, `input.required`, two-sided `model`, functional role ownership/token resolution, setter identity, and definite-assignment normalization. A consolidated production-boundary harness covers these public semantics plus refresh removal/replacement. All focused native changed-test groups are green by user report. The consolidated live harness is also green across all five cases, including refresh replacement/removal.

## 16. Verification evidence

Evidence remains separated:

1. **Tracked regression evidence:** focused provider, member-signal/functional-DI, classic member identity, decorator/marker preservation, import binding, and compatibility groups are green by user report.
2. **Production compiler regressions:** exported-class signal and provider semantics reach the real compilation boundary; both focused tests are green by user report.
3. **Focused live/MCP verification:** all five cases pass in `target/issue-138-live-verification/20261015-175047-f887030d`, including negative twins and refresh replacement/removal.
4. **Repository-wide CI gate:** not run locally; this gate is CI-owned.

Historical #138a/#138b counts remain historical only, not current PASS claims. The provider result is current user-run evidence.

## 17. Cross-issue boundary audit

- #133–#137 and #140 remained frozen.
- No LGR-003/#139 redesign was undertaken.
- No linked-template/#141 behavior was changed.
- No global coverage/query-history/#142 policy was introduced.
- No CBM/provider became required correctness authority.
- No generic TypeScript core acquired Angular semantics.
- No GitHub state was mutated and #139 was not started.

## 18. Remaining residuals

Closure-critical implementation is complete. Broader non-closure residuals remain:

- `@Inject(TOKEN)` decorator-argument identity.
- Constructor-body `inject()` policy outside functional-role ownership.
- Alias and required metadata beyond canonical property identity.
- Host/view decorator relationships, which remain outside this issue.
- The unnamed historical real-file anomaly, whose source identity is still unavailable.

## 19. Closure recommendation

The focused changed-test set and applicable live verification are green. Close #138 after the repository-wide CI gate passes in CI; do not rerun that full gate locally.

## 20. Commands run

Commands/tools and bounded checks used:

- `git status --short`.
- Authoritative `gh issue view 138 --repo codeliftsleep2/Clean-CTX --json ...`.
- Bounded `git log`, `rg`, `Get-Content`, and line-count inspection.
- Clean-CTX `get_cbm_status`, `graph_search`, and batched `provide_code_context`.
- Skeleton-only structured results were followed by exact bounded reads as required by tooling policy.
- `scripts/check-file-sizes.ps1`, strict UTF-8/BOM inspection, `git diff --check`, and targeted module-wiring searches.

Agent-run verification remained read-only and bounded. Cargo builds/tests and the live harness were executed by the user; no agent-started Cargo command, binary, server, benchmark, CBM provider, or repository-wide gate was run. The repository-wide gate remains CI-owned.

No further local test group is required by this reconciliation; leave the repository-wide gate to CI.

**Final verdict: #138 COMPLETE REPAIR — ACCEPTANCE GREEN; CI GATE PENDING**

