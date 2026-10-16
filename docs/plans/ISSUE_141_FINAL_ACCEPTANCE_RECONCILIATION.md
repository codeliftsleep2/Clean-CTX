# Issue #141 Final Acceptance Reconciliation

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-15  
**Issue:** GitHub #141, “Angular templates (supersedes the earlier template-only report)”  
**Verdict:** **#141 COMPLETE REPAIR IMPLEMENTED — VERIFICATION REQUIRED**

## 1. Executive verdict

The previously completed Angular template phases satisfy the original
recognition, structural rendering, fidelity, linkage, selector-resolution, and
lifecycle contracts. This reconciliation found one live-gate residual:
`entities_in_file` still rejected linked HTML before consulting Angular
ownership. The bounded repair now makes it use the same ownership authority as
`provide_code_context` and return the existing canonical `angular/Template`
representation. No prohibited architecture expansion was required.

Acceptance is not yet green because this session did not run Cargo, test
binaries, the MCP process, live scripts, or the authoritative final gate.

## 2. Original contract

The authoritative issue reports filename-dependent Angular recognition,
control-flow loss/misattribution, absent Component↔Template and selector-use
answers, and unclear raw-verbatim fallback wording. Its acceptance fixture
requires sequential `@if`, `@else if`, `@for`/`track`/`@empty`,
`@switch`/cases, and `@let`, with every condition/value exactly once and owned
by the correct block. Parent Component queries must report selector-resolved
children and each occurrence's controlling condition.

This campaign preserves #133–#140 and leaves generalized diagnostics,
completeness, admission, refresh, query-history, and coverage work to #142.

## 3. Acceptance matrix

| Area | Criterion | Status | Authority/evidence |
| --- | --- | --- | --- |
| Recognition | Conventional `*.component.html` | SATISFIED | Direct-request hint in `provide_angular::is_angular_template`; production regression extended to `entities_in_file`. |
| Recognition | Arbitrary linked HTML | SATISFIED | Structured `Component.templateUrl` scan and canonical path equality; provide and entity-query regressions. |
| Recognition | Identical-content filename equivalence | SATISFIED | `Invoke-NativeCausalControls.ps1` assertion and shared admission/render path. |
| Recognition | Unowned HTML negative | SATISFIED | Unsupported-file result remains; new entity-query negative guard and existing truthful provide error test. |
| Recognition | `entities_in_file` | SATISFIED | Bounded repair returns canonical `angular/Template`; verification pending. |
| Recognition | LGR-002 | SATISFIED | Production fix plus tracked regression implemented; live rerun pending. |
| Modern flow | Sequential/nested/operator `@if` | SATISFIED | Ordered structural regressions and occurrence ancestry tests. |
| Modern flow | `@else`/`@else if`/order | SATISFIED | Companion-block regression preserves written header and order. |
| Modern flow | `@for`/`track`/`@empty` | SATISFIED | Structure and Angular-version equivalence regressions. |
| Modern flow | `@switch`/case/default | SATISFIED | Version-equivalence matrix retains expression and literal case values. |
| Modern flow | `@let` name/expression | SATISFIED | Companion-block regression retains complete declaration. |
| Modern flow | Baseline `@defer` | SATISFIED | Trigger and placeholder/loading/error headers and children are ordered. |
| Modern flow | Exact content ownership | SATISFIED | Structural event stream plus occurrence byte spans/control ancestry. |
| Legacy | `*ngIf`/`*ngFor` | SATISFIED | Legacy and mixed-template regressions. |
| Legacy | No misleading rewrite | SATISFIED | Written legacy microsyntax is retained even where normalized behavior markers coexist. |
| Content | Text/interpolation/pipes/refs/i18n/comments | SATISFIED | High structural tests; Medium comment omission is disclosed. |
| Content | Bindings/events and owning element | SATISFIED | Complete start-tag events preserve ownership. |
| Fidelity | Low/Medium/High | SATISFIED | Low declares aggregate ordering; Medium is source ordered; High retains more content. |
| Fidelity | Verbatim/Edit | SATISFIED | Verbatim is exact raw document; template Edit does not falsely claim method-body authority. |
| Fidelity | Machine-readable degradation | SATISFIED | Stable `ordering` and `omits` metadata; exact/raw representations use `null`. |
| Linkage | `templateUrl` and arbitrary filename | SATISFIED | `HasTemplate` uses canonical source identity. |
| Linkage | Inline template | SATISFIED | Deterministic Component-scoped inline identity. |
| Linkage | Component→Template | SATISFIED | Existing `HasTemplate`, forward/reverse query coverage. |
| Linkage | Selector resolution | SATISFIED | Published `HasSelector` declaration authority; native tags do not resolve. |
| Linkage | Component→used Component | SATISFIED | Ephemeral additive `UsesComponent` after unique complete resolution. |
| Linkage | Control-flow condition | SATISFIED | Every occurrence carries `template_occurrence.control_ancestry`. |
| Linkage | Ambiguous/unresolved selector | SATISFIED | Neutral `UsesSelector` plus explicit diagnostics; no guessed Component. |
| Lifecycle | Change/delete/restore | SATISFIED | TypeScript owner replacement and refresh regressions; ephemeral occurrences recompute. |
| Errors | Owned/unowned HTML truthfulness | SATISFIED | Owned HTML uses Angular semantics; generic HTML explains raw uncompressed verbatim fallback. |
| Preservation | Non-HTML and #138/#139/#140 | SATISFIED | Repair branches only for proven Angular HTML and does not alter other compilation paths. |

No row is `UNSATISFIED`. “SATISFIED” records implemented contract and tracked
authority; it does not claim the unrun verification gates passed.

## 4. Current template authority chain

`@Component` declaration → structured decorator capture → `templateUrl` or
inline template metadata → lexical path resolution or deterministic inline
identity → ordered Angular template parser → fidelity rendering and selector
occurrences → TypeScript-owned `HasTemplate` publication → WorkspaceIndex →
request-local Angular query catalog → MCP response.

Exact production owners are `angular_meta::decorators`,
`template_ownership`, `template_ordered`, `template_occurrences`,
`angular_meta::semantic`, `provide_angular`, WorkspaceIndex, and
`query::angular_templates`.

## 5. Template recognition and ownership

Filename convention is a hint only. Arbitrary `.html` becomes Angular only
when a current TypeScript Component's structured `templateUrl` resolves to the
same canonical physical identity. The repair exposes this single admission
function to both provide and entity-query surfaces. It does not make all HTML
Angular and does not infer ownership from basename alone.

## 6. LGR-002 reconciliation

Historical RED: `target/native-causal-controls/20261013-114555-7b27f06e/`
records `entities_in_file` dispatching linked HTML to unsupported language IR.
The ledger's former #134/ASP.NET repair citation was unrelated and has been
corrected without deleting history.

The defect was the generic `ensure_supported_file` gate preceding Angular
ownership. The repair checks the shared Angular authority first and returns one
canonical `angular/Template` entity with physical-file provenance. Unowned HTML
still fails `unsupported_file_type`. This is a request-local projection of the
already approved Template vocabulary, not fake raw IR or new durable state.

## 7. Template canonical structure

`OrderedTemplate` is the structural authority. Its source-ordered events retain
control headers, start/end tags, content, and High/Edit comments. Selector
occurrences contain half-open UTF-8 byte spans and structural ancestry. The
renderer is a projection and is never reparsed to recover semantic ownership.

## 8. Modern control-flow semantics

Balanced structural regions distinguish siblings from descendants and retain
written expressions without evaluation. `@else if` remains distinct;
`@for` retains iterable and track clause; companion `@empty` ownership is
preserved; switch cases/default retain values; `@let` retains name/expression;
baseline defer and companion headers retain trigger/timing text.

## 9. Legacy structural directives

Legacy directives remain tied to their owning element. Output may expose a
normalized behavioral marker for comparability, but it also retains the
written microsyntax (`else`, `trackBy`, variable binding) and never presents a
source rewrite as if modern syntax had been written.

## 10. Element/binding/text ownership

Complete element start tags keep inputs, outputs, references, i18n/static
attributes, and structural directives attached to the element. Source-ordered
content events preserve bounded text and interpolation. Omitting a category at
lower fidelity does not move surviving content into another control block.

## 11. Fidelity and degradation

- Low: aggregate/unordered, with explicit omissions including element
  ownership, control ancestry, visible text, interpolation expressions, and
  comments.
- Medium: source ordered; comments are explicitly omitted.
- High: structural output retains comments and richer visible content.
- Verbatim: byte-exact raw document, uncompressed.
- Edit on HTML: structural template output; it never claims byte-exact method
  bodies.

Raw economic passthrough is exact and therefore carries no false degradation.

## 12. Component-template linkage

Structured Component metadata exclusively authors `HasTemplate`. External
identity is the canonical physical/semantic-owner path. Inline identity is
`inline:<declaring-source>#component=<class>`. The declaring TypeScript file
owns durable publication and retraction; the template does not steal semantic
ownership merely to support invalidation.

## 13. Selector resolution and child use

Template element spelling produces neutral `angular/SelectorRef` occurrence
evidence. Resolution consults exact published `HasSelector` Component
declarations. One candidate under complete discovery adds `UsesComponent`;
zero or multiple candidates remain neutral with unresolved/ambiguous
diagnostics. Selector spelling never becomes Component identity.

## 14. Control-flow-qualified child use

`UsesSelector` and `UsesComponent` are request-local refinements. Each retains
the template occurrence span and outermost-to-innermost legacy/modern control
ancestry. Repeated use of one selector under different conditions therefore
remains distinct without string-encoding conditions into identities or adding
generic durable relationship metadata.

## 15. Inline templates

Inline templates reuse the same parser and occurrence model. Their identity is
tied to the declaring source and Component rather than a fake path. Tracked
tests cover identity, mixed legacy/modern ancestry, external↔inline
transitions, and non-aliasing.

## 16. Cross-file lifecycle validity

Durable `HasTemplate` remains part of the existing Angular semantic projection
and #139 compatibility/dependency validity. Current template content is read
request-locally for occurrence refinement. Component metadata changes replace
the edge; deletion retracts the TypeScript owner; missing template content
retains the written reference with a diagnostic; restoration re-enables current
derivation. No template watcher, second index, or parallel invalidation system
was introduced.

## 17. Error behavior

Proven Angular HTML enters Angular presentation/entity semantics. Unowned HTML
remains unsupported for semantic queries, and the provide error truthfully
describes verbatim as raw, faithful, uncompressed content. Non-HTML behavior is
unchanged.

## 18. Repairs implemented

1. Refactored Angular admission into one shared production helper.
2. Routed proven HTML through that helper before generic language rejection in
   `entities_in_file`.
3. Returned the existing canonical Template identity request-locally.
4. Added conventional/linked equivalence and unowned-negative regression
   coverage.
5. Corrected LGR-002's unrelated #134 repair evidence while retaining RED
   history.
6. Extended the existing native #141 harness with canonical identity,
   conventional/linked equivalence, unowned HTML, and refresh-driven owner
   change/deletion/restoration checks; no additional harness was introduced.

## 19. Regression inventory

Tracked authorities include:

- `template_structure_regressions.rs`: sequential/nested/operator flow,
  legacy/mixed ownership, text/interpolation, companions, defer, let, refs,
  pipes, and i18n.
- `template_version_equivalence.rs`: Angular 15/17 normalized equivalence and
  source-specific syntax retention.
- `provide_angular_template_ownership.rs`: arbitrary filename recognition,
  raw-error wording, linked structure, and repaired `entities_in_file`.
- `workspace_query_angular_templates.rs`: canonical external/inline identity,
  forward/reverse linkage, transitions, deletion/restoration.
- `workspace_query_angular_selector_occurrences.rs`: spans, ancestry, unique,
  unresolved, ambiguous, refresh, and selector lifecycle.
- `angular_template_phase6_diagnostics.rs`: fidelity degradation, missing
  template, selector diagnostics, and exact/lower-bound coverage.

## 20. Preservation audit

The repair changes no parser, renderer, selector registry, durable relation,
producer generation, cache policy, global admission rule, or non-HTML path.
Existing #138 DI/signals/providers/Input/Output, #139 declaration and semantic
dependency behavior, and #140 NgRx behavior remain outside the branch. The
negative guard protects unrelated HTML.

## 21. Verification evidence

Historical evidence remains historical: earlier Phase 0–6 focused gates,
authoritative gate, and consolidated Angular live MCP pilot were maintainer-
reported green before this residual repair. They do not verify the current
working tree.

Fresh focused live evidence is GREEN. The user ran the native causal-control
harness against the rebuilt binary on 2026-10-15; all seven cases passed at
`target/native-causal-controls/20261015-225232-eb0a7fb0/`, including both #141
cases. The linked-HTML case verified canonical Template identity/provenance,
conventional and arbitrary linked recognition, unowned HTML rejection, and
owner change/deletion/restoration after refresh. This is operator/live evidence,
not a Rust test or authoritative final-gate result.

This session ran only read-only inspection, `git diff --check`, diff/status,
history, GitHub issue read, and static file-size inspection. `git diff --check`
reported no whitespace errors. No tracked test, binary, build, check, Clippy,
formatter, live harness, or final gate ran.

## 22. Cross-issue boundary audit

#133–#140 were not reopened. #139's canonical-versus-derived validity boundary
is reused. #142 retains generalized completeness, admission, refresh,
query-history, coverage, diagnostics, and dependency-capture work. This repair
does not alter global HTML admission or generic query semantics.

## 23. Remaining residuals

There is no known implementation or harness-definition residual inside #141
acceptance. Remaining work is executing the tracked and live verification.
Broader defer trigger variants, exhaustive i18n,
general CSS selectors, expression evaluation, singular `includeTemplate`, and
global diagnostics/coverage are non-goals, not hidden completeness claims.

If a gate exposes a new behavior defect, this verdict must return to
`UNSATISFIED` for the affected row rather than treating the historical pass as
current evidence.

## 24. Closure recommendation

Do not close #141 yet. The focused native LGR-002 case is GREEN. After the
focused tracked test, Angular live MCP acceptance, preservation tests, and
authoritative final gate pass against the current tree/binary, change the
verdict to `#141 COMPLETE REPAIR — ACCEPTANCE GREEN`.

No architecture approval is required for the implemented repair: it reuses the
already approved Template kind, source identity, and request-local projection.

## 25. Commands run

Read-only commands: authoritative `gh issue view 141`; bounded `git status`,
`git log`, `git show`, `git diff`, `git diff --check`; `rg`; `Get-Content`;
line counts; Clean-CTX CBM status, graph search, and structured context reads.
Edits were applied with `apply_patch`.

Not run: Cargo tests/check/build/Clippy/fmt, direct test binaries, Clean-CTX
runtime, MCP/live scripts, model pilots, benchmarks, authoritative final gate,
GitHub mutation, commit, or push.

Recommended user-run verification, in order:

```powershell
cargo test --all-features provide_angular_template_ownership
cargo build --all-features --bin clean-ctx
powershell -NoProfile -ExecutionPolicy Bypass -File verification/issue-reconciliation/Invoke-NativeCausalControls.ps1 -BinaryPath ./target/debug/clean-ctx.exe
node verification/live-acceptance/angular_template_linkage_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

Then run every user-run command in the single authoritative Final Verification
Gate at `docs/agent/verification.md` § “Final Verification Gate”. Its list is
the command authority and is intentionally not duplicated here.

**Final verdict: #141 COMPLETE REPAIR IMPLEMENTED — VERIFICATION REQUIRED**
