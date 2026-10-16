# Angular Template Correctness and Linkage Plan

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-10  
**Status:** Complete — Phases 1–6, final verification, and live MCP pilot GREEN
**Source:** Live MCP investigation plus production-code trace  
**Compatibility floor:** Angular 15 and later

## Purpose

Correct Angular template handling so Clean-CTX recognizes templates by Angular ownership rather than filename convention, never emits structurally misleading control-flow output, retains behaviorally meaningful content, and exposes component/template relationships through coherent lifecycle and query boundaries.

This plan addresses five observed problems:

1. External templates are recognized only when their names end in .component.html.
2. Compressed output can associate elements with the wrong control-flow block.
3. Visible text and interpolation expressions are discarded.
4. Components, templates, and used child components are not connected through the public semantic graph.
5. Unsupported-HTML and partial-query diagnostics do not guide callers accurately.

Presentation correctness comes before template-derived graph publication. A false compact representation is worse than no compact representation.

## Compatibility invariant: Angular 15+

All phases must preserve equivalent functionality for Angular 15 and every supported later syntax generation. “Equivalent” means the same information quality and lifecycle behavior, not merely recognizing a token.

For every supported Angular version and syntax family, Clean-CTX must provide:

- the same external and inline template ownership behavior;
- truthful Medium and High structural containment;
- visible text, interpolation, and binding ownership at the same fidelity;
- equivalent component/template and parent/child graph capability;
- the same publication, replacement, deletion, refresh, and restoration rules;
- equally truthful diagnostics and partial-result coverage.

The implementation must support three populations:

1. **Angular 15–16 legacy syntax:** *ngIf, *ngFor, [ngSwitch], *ngSwitchCase, *ngSwitchDefault, ng-template, and ordinary property/event/two-way bindings.
2. **Angular 17+ built-in control flow:** @if, @else if, @else, @for, @empty, @switch, @case, @default, @defer, @placeholder, @loading, @error, and @let.
3. **Mixed migration templates:** legacy directives and built-in blocks in the same file, including nesting one syntax family inside the other.

Legacy syntax must not be implemented as a lossy translation into modern syntax. Both forms normalize into the same ordered semantic template model while retaining written form and source spans. No phase may ship if it improves modern syntax while regressing Angular 15–16 behavior, or vice versa.

## Executive decision

Adopt one structured Angular template model and one Angular template ownership authority.

- Structured @Component metadata is authoritative for templateUrl and inline template ownership.
- The .component.html suffix remains a standalone recognition convention, but is not the authority for linked templates.
- Medium and High output render from an ordered structure preserving control-flow ownership, element ownership, visible text, interpolation, and source order.
- Low fidelity may use aggregate counts and sets when it does not imply containment.
- Legacy directives and modern blocks produce equivalent normalized nodes.
- Component-to-template linkage uses the existing HasTemplate relation.
- Parent-to-child component relationships are added only after an explicitly approved template semantic authority exists.
- Generic HTML remains outside the Angular path unless Angular ownership or the existing filename convention establishes it as a template.

Do not introduce Angular canonical CoreOps or route HTML through the ordinary source-language compiler solely to implement this plan.

## Verified current architecture

### Routing is filename-based

src/mcp/heuristics.rs recognizes Angular templates through a .component.html suffix. src/mcp/tool_handlers/core/provide_angular.rs independently applies the same restriction. A valid templateUrl pointing to detail-page.html is therefore rejected by default processing even though component metadata already knows the link.

### Structural ownership is discarded

src/angular_meta/template.rs stores flat collections for conditions, loops, elements, bindings, and custom elements. Extraction sorts and deduplicates those collections. Medium and High rendering print grouped categories.

That representation cannot retain:

- which elements belong to which block or structural directive;
- which bindings belong to which element;
- sibling source order;
- companion-branch ownership;
- mixed legacy/modern nesting.

Modern control-flow scanning also takes the first matching condition in a text region, so sequential blocks can lose later conditions.

### Meaningful content is not retained

The shape stores an interpolation count, not interpolation expressions, and does not retain visible text. High fidelity therefore cannot explain messages, error states, or displayed values.

### Component metadata is not projected completely

Decorator extraction already captures templateUrl, inline template, and style metadata. The semantic vocabulary already declares HasTemplate and HasStyle, but the Angular component projector does not publish them.

### Template-derived graph facts have no producer

Angular HTML presentation is not canonical IR and is not durably checkpointed as source semantics. Arbitrary linked HTML does not automatically enter normal source hydration. No current producer owns a neutral selector-use fact or a resolved parent-to-child component relationship.

### Batch retrieval is useful but not automatic linkage

provide_code_context already supports a batch of known files. Once linked-template routing works, a caller can request component and template together. A singular component request does not expand its template, and no includeTemplate field exists.

## Architectural boundaries

### Existing behavior to preserve

- Non-Angular TypeScript and generic HTML behavior.
- Existing .component.html support.
- Inline template support.
- Existing decorator, DI, input/output, routing, NgRx, and selector relationships.
- Verbatim source retrieval.
- Neutral unresolved selector evidence.
- Angular 15–16 directive capability while adding Angular 17+ support.

### Corrective work within current responsibilities

1. Recognize decorator-linked templates.
2. Replace misleading flat Medium/High rendering.
3. Retain visible text and interpolation expressions.
4. Project already-declared HasTemplate and HasStyle relations.
5. Improve diagnostics without overstating completeness.
6. Give legacy and modern syntax equivalent output semantics.

### Approval required for new graph capability

Template-derived parent/child component edges require approval of:

- a semantic producer for Angular templates;
- neutral selector-reference and resolved component-use relations;
- the public Template identity;
- linked-HTML hydration without globally admitting HTML;
- assertion ownership and retraction;
- producer keys and generation changes;
- partial-result and coverage semantics.

Do not infer approval for this decision from approval of presentation fixes.

## Target architecture

### Component template reference

Expose one structured value from the existing decorator parser containing:

- owner component identity;
- declaring TypeScript file;
- source kind: inline or external;
- written templateUrl;
- safely resolved normalized workspace path;
- source span.

Resolve external paths relative to the declaring TypeScript file through the repository path-identity and workspace-scope authorities. Reject or report workspace escapes. Presentation, semantic projection, and template lookup must consume this same authority.

### Ordered template representation

Replace the flat Medium/High authority with an ordered tree or event stream:

- Document: nodes in source order.
- Element: tag, attributes and bindings in source order, children.
- ControlBlock: syntax family, block kind, header or trigger, children, companion blocks, source span.
- StructuralDirective: written directive, expression, owning element, embedded-template semantics, source span.
- Interpolation: written expression and source span.
- Text: visible normalized text and source span.
- Comment: optional by fidelity.

Both syntax families normalize into common behavioral concepts without losing their written syntax:

| Behavioral concept | Angular 15–16 | Angular 17+ |
|---|---|---|
| Conditional | *ngIf and ng-template else | @if, @else if, @else |
| Iteration | *ngFor | @for, @empty |
| Selection | [ngSwitch] and case/default directives | @switch, @case, @default |
| Deferred region | existing template/outlet patterns retained structurally | @defer companions |
| Local value | template variables and let- bindings | @let |

Sorting and deduplication are allowed only in derived Low summaries, never in the structural authority.

### Fidelity contracts

**Low:** Aggregate summary. It must not visually imply containment it does not represent.

**Medium:** Behavioral skeleton preserving control-flow headers, companion branches, structural-directive ownership, element nesting required for ownership, component elements, bindings with owning elements, visible text, and interpolation expressions.

**High:** Everything in Medium plus ordinary elements, meaningful static attributes, image src/alt, state/class/style bindings, and content needed to explain rendered behavior.

**Verbatim:** Unchanged source through the existing contract.

If a fidelity deliberately omits categories, the response must disclose them unambiguously. Omission policy must be identical for equivalent legacy and modern constructs.

### Template identity and HasTemplate

Recommended public relationship:

    builtin/Component/<component>
      --HasTemplate-->
    angular/Template/<identity>

External identity is based on normalized workspace-relative path, not basename. Inline identity is deterministically scoped to declaring component and file.

The TypeScript file owns the HasTemplate assertion because it declares the reference. Replacement or deletion of that file retracts the edge.

Before implementation, confirm whether an existing entity kind expresses this without semantic overloading. If not, adding Template is a public schema decision requiring approval. Do not reuse TypeRef, Class, or an unrelated file identity for convenience.

### Template-derived selector facts

Recommended two-stage model:

    Template --UsesSelector--> SelectorRef/<written-selector>
    ParentComponent --UsesComponent--> ChildComponent

The template file owns the neutral written selector fact. A typed relationship is derived only when exactly one in-scope indexed component owns that selector. Neutral evidence remains queryable. Zero or multiple matches remain neutral.

Prefer query-time refinement over persisting a cross-file derived edge unless dependency ownership and invalidation are explicitly approved.

### Hydration and lifecycle

Linked templates enter processing through component ownership, not global .html source admission.

Required path:

    Component declaration
      -> template reference
      -> linked template discovery
      -> ordered template model
      -> presentation and semantic facts
      -> file-owned publication
      -> replacement/deletion/refresh retraction
      -> query resolution
      -> truthful MCP coverage

External edits remain stale until the documented refresh lifecycle observes them. refresh_workspace must discover linked-template creates, replacements, and deletions without leaving stale selector facts.

## Implementation phases

## Phase 0 — Establish RED evidence

Add tracked tests under src/tests before production changes and register them through normal path modules.

Required RED groups:

1. **Ownership**
   - conventional and arbitrary external filenames;
   - inline template;
   - nested relative path and Windows case identity;
   - missing and out-of-scope target;
   - unrelated HTML remains generic.
2. **Angular 15–16**
   - *ngIf with else template;
   - *ngFor with local variables and trackBy;
   - legacy switch cases/default;
   - nested ng-template;
   - text, interpolation, bindings, and child components remain owned.
3. **Angular 17+**
   - sequential and nested @if;
   - @else if and @else;
   - @for and @empty;
   - switch cases/default;
   - defer companions;
   - @let.
4. **Mixed migration**
   - legacy directive inside modern block;
   - modern block inside legacy structural region;
   - sibling constructs from both families preserve order;
   - identical component-use extraction from both forms.
5. **Content**
   - visible messages and interpolation expressions;
   - binding-to-element ownership;
   - meaningful image/accessibility attributes;
   - omission disclosure.
6. **Linkage and diagnostics**
   - stable external and inline template identity;
   - replacement/removal retracts old reference;
   - raw/verbatim fallback wording;
   - partial query does not claim completeness.

The user runs focused tests against unchanged production code. Only intended behavioral assertion failures count as RED. Stash only regressions and module registration before implementation, then restore the exact tests for GREEN.

## Phase 1 — Establish one ownership authority

1. Expose structured references from existing decorator extraction.
2. Resolve external paths through workspace path identity and containment.
3. Add a workspace-scoped normalized template-path lookup.
4. Replace duplicated suffix decisions with one classification: convention-owned, decorator-owned, or not Angular.
5. Preserve inline processing through the same abstraction.
6. Do not globally add HTML to canonical source compilation.

Completion: arbitrary linked filenames use the Angular path, unrelated HTML is unchanged, and behavior is version-independent.

## Phase 2 — Introduce the ordered representation

1. Add focused ordered-node types without enlarging an active oversized file.
2. Preserve tree-sitter HTML element spans and source order.
3. Build a balanced modern-block scanner that emits every block and body range.
4. Model legacy structural directives on their owning elements and embedded-template regions.
5. Associate HTML nodes with the correct modern or legacy region.
6. Represent companion branches explicitly.
7. Capture text and interpolation expressions.
8. Derive Low summaries from the ordered authority.
9. Render Medium and High only from the ordered authority.
10. Remove the superseded flat Medium/High path after migration.

The modern scanner must handle nested braces, strings, expressions, and nested blocks. Independent keyword searches are prohibited. Legacy microsyntax must retain the complete written expression even when only part can be classified.

Completion: neither syntax family can invent containment, reverse conditions, or detach bindings.

## Phase 3 — Prove Angular 15+ semantic equivalence

This is a required phase, not optional cleanup.

1. Create a table-driven cross-version fixture matrix pairing equivalent legacy and modern behaviors.
2. Assert the same normalized behavioral node kinds and ownership where Angular semantics are equivalent.
3. Assert source-specific data remains available where forms differ.
4. Run the same fidelity assertions across both syntax families.
5. Run the same selector extraction and lifecycle assertions across both.
6. Include mixed-migration templates in production-path MCP tests.
7. Verify legacy-only projects do not depend on modern syntax being present.
8. Verify modern projects do not require legacy directive imports for parsing.
9. Document genuine semantic differences rather than forcing false equivalence.

Completion: Angular 15–16, Angular 17+, and mixed templates have equivalent Clean-CTX capabilities and truthful differences.

## Phase 4 — Complete component-to-template semantics

1. Approve the Template identity.
2. Project HasTemplate from structured component metadata.
3. Project HasStyle from the same authority if included.
4. Keep the declaring TypeScript file as assertion owner.
5. Add forward/reverse external and inline queries.
6. Verify replacement, deletion, republishing, restoration, and refresh.
7. Determine and bump the existing Angular semantic producer generation when durable output changes. Reuse its producer key.

Completion: Component and Template queries link correctly regardless of filename or Angular template syntax generation.

## Phase 5 — Add component usage after approval

1. Preserve neutral UsesSelector facts for component elements under both legacy and modern control flow.
2. Produce facts from the ordered model, never rendered skeleton text.
3. Own neutral facts by the template file.
4. Resolve unique selector declarations within effective scope.
5. Derive typed UsesComponent at query time unless dependency tracking is explicitly approved.
6. Hydrate component declarations when the query contract authorizes it and merge hydration diagnostics.
7. Keep absent or ambiguous selectors neutral.
8. Verify refresh and replacement cannot revive stale semantics.
9. Use an existing producer key when the authority is the same; introduce a new key only for a genuinely distinct approved template-file authority.

Completion: parent/child queries behave identically for child selectors inside legacy directives, modern blocks, or mixed nesting.

## Phase 6 — MCP composition and diagnostics

1. Preserve explicit batch retrieval.
2. Decide separately whether singular component requests need includeTemplate or related-file metadata.
3. If approved, define budgeting, per-item status, missing-template behavior, deduplication, and no silent default response growth.
4. Clarify generic HTML versus linked-template errors.
5. Retain neutral alternative-query guidance.
6. State whether missing template/component discovery can change results.
7. Allow response evidence to establish capability, never false completeness.

Recommendation: defer includeTemplate until ownership, structure, and linkage are stable.

## Regression matrix

| Area | Required cases |
|---|---|
| Ownership | conventional/arbitrary name, inline, nested path, missing target, scope escape, shared template |
| Angular 15–16 | ngIf/else, ngFor locals/trackBy, switch, ng-template, template refs/outlets |
| Angular 17+ | if branches, for/empty, switch, defer companions, let |
| Mixed versions | legacy in modern, modern in legacy, ordered siblings, nested child selector |
| Content | text, multiple interpolations, pipes, expressions, images/alt, whitespace, comments by fidelity |
| Bindings | property, event, two-way, class/style, directive, child component |
| Link semantics | external/inline HasTemplate, HasStyle if approved, forward/reverse, dedup |
| Selector semantics | known, missing, duplicate, element/attribute selectors, replacement/deletion |
| Lifecycle | publish, republish, edit, remove, refresh, deleted root, restore, generation mismatch |
| Paths/scopes | additional roots, withinPath, Windows case, separators, escape rejection |
| Diagnostics | generic HTML, missing linked file, partial hydration, neutral alternative |
| Preservation | non-Angular TS, generic HTML, current decorators, current inline output, verbatim |

Every production regression lives under src/tests. A live harness under verification is operator evidence only.

## Production integration checklist

Trace each phase through:

- producer;
- default production invocation;
- result boundary;
- persistent assertion owner;
- publication, replacement, deletion, refresh, and restore lifecycle;
- real presentation or query consumer;
- MCP exposure;
- final live reachability.

Unit tests against a custom extractor do not prove production integration.

## Compatibility and producer generations

- Presentation changes need no semantic generation bump unless versioned presentation data is persisted.
- HasTemplate and HasStyle change durable TypeScript semantic output and normally require an existing producer generation bump.
- Template-file semantic facts may require a distinct generation or, only with approval, a distinct key because assertion ownership differs.
- Query-time typed refinement does not change durable producer output.
- Restores must reject or regenerate incompatible generations; never silently accept snapshots lacking authoritative new facts.

## Documentation updates

Update:

- docs/ANGULAR_META_LAYER.md with ownership, fidelity, and Angular 15+ equivalence contracts;
- docs/ARCHITECTURAL_INVARIANTS.md only if template semantic publication is approved and implemented;
- docs/agent/DISCOVERY_REGISTRY.md with the live filename and control-flow discoveries plus regression links;
- MCP tool descriptions and errors;
- changelog/release notes separating corrections from new graph capability.

## Risks and mitigations

- **Modern parser complexity:** use balanced span-aware scanning; fail visibly rather than guessing.
- **Legacy microsyntax complexity:** preserve the full written expression and classify only proven components.
- **Version skew:** require the cross-version equivalence matrix and mixed templates before each phase completes.
- **Duplicate authorities:** derive summaries and graph facts from the ordered model and remove old structural rendering.
- **Broad HTML admission:** discover through component ownership, not extension.
- **Stale graph edges:** persist neutral file-owned facts and derive typed cross-file results unless dependency tracking is approved.
- **Response growth:** define fidelity policy and keep automatic inclusion opt-in.
- **Path mistakes:** reuse path identity, containment, effective roots, and refresh authorities.

## Commit boundaries

1. test(angular): pin template ownership and fidelity regressions
2. test(angular): pin Angular 15+ syntax equivalence
3. refactor(angular): establish structured template references
4. fix(mcp): recognize decorator-linked Angular templates
5. refactor(angular): add ordered template representation
6. fix(angular): preserve legacy and modern control-flow ownership
7. feat(angular): project component template relationships
8. test(angular): pin selector and lifecycle semantics
9. feat(angular): publish neutral template selector references — after approval
10. feat(query): resolve Angular component usage safely — after approval
11. docs(angular): align Angular 15+ template contracts

## Verification handoff

Agents do not run Cargo builds, checks, tests, Clippy, binaries, or servers. For each phase, hand the user focused copy-pasteable commands for the exact tracked modules. After focused GREEN, hand off the authoritative gate from docs/agent/verification.md.

Live Claude/MCP testing is the last field gate. Batch legacy, modern, mixed, nonstandard-filename, graph, lifecycle, and diagnostics scenarios into one pilot after deterministic local verification and the binary/stdio acceptance path are green.

Report tracked test results separately from live acceptance results.

## Definition of Done

The effort is complete only when:

1. Arbitrary external template filenames work through component ownership.
2. Unrelated HTML is not classified as Angular.
3. Angular 15–16, Angular 17+, and mixed templates have equivalent supported capabilities.
4. Medium and High preserve control-flow ownership and source order for both syntax families.
5. Text, interpolation, and binding ownership meet the fidelity contract.
6. The misleading flat Medium/High authority is removed.
7. Component/template queries work forward and reverse.
8. Selector and typed component-use semantics are identical across legacy and modern nesting.
9. Lifecycle changes retract stale facts.
10. Diagnostics describe partiality and verbatim fallback truthfully.
11. Existing non-Angular and Angular behavior is preserved except for approved changes.
12. Producer generations match durable output.
13. Documentation matches production.
14. Tracked regressions retain RED-to-GREEN evidence.
15. The complete production lifecycle reaches real MCP responses.
16. The final architectural audit finds no critical or high issue.
17. The user-run Final Verification Gate and batched live field check pass.

## Explicit non-goals

- General HTML semantic indexing.
- Angular compiler or TypeScript type-checker parity.
- Guessing components from names or filename conventions.
- Persisting cross-file typed edges without dependency ownership.
- Angular-specific canonical CoreOps solely for presentation.
- Replacing verbatim retrieval.
- Automatic default template inclusion without a separately approved contract.
- Dropping Angular 15–16 support to simplify Angular 17+ parsing.

## Live issue reconciliation - #141 and #142

**Review date:** 2026-10-10
**Reviewer:** Agent MaxHeadRoom
**Evidence baseline:** commits `babdd039` and `26173bd4`; GitHub issues [#141](https://github.com/codeliftsleep2/Clean-CTX/issues/141) and [#142](https://github.com/codeliftsleep2/Clean-CTX/issues/142)

This section supersedes assumptions in the earlier Phase 4-6 outline where the live findings or current production architecture provide more precise evidence. Committed Phases 0-3 remain authoritative completed work. No production semantic capability was added during this review.

### Reconciliation conclusions

- Phases 1-3 resolve the central #141 routing and Medium/High presentation defects.
- No #141 finding invalidates the GREEN Phase 0-3 regressions.
- Phase 0-3 regression coverage does not directly pin every construct named by #141. `@else if`, `@empty`, `@defer` companions/triggers, `@let`, template references, pipes, and i18n retention need focused preservation tests before related semantic work changes the ordered authority.
- `provide_code_context` still reports `degradation: null` for intentionally lossy Angular skeletons. Low remains an unordered aggregate and Medium omits comments. This is a Phase 6 truthfulness remainder, not evidence that the verified Medium/High ordering tests are false.
- The original Phase 4 example incorrectly names the subject as `builtin/Component`. The production Angular declaration identity is `angular/Component/<class-name>`, with physical occurrences distinguished by file provenance.
- The proposed workspace-relative Template name is not currently derivable inside the Angular semantic producer: `MetaLayerContext` receives the declaring path but not the request's primary workspace root. Basename is unacceptable, and silently inventing a root-discovery policy is also unacceptable.
- The current ordered presentation authority is a source-ordered event stream plus a flat custom-element list. It can render conditions and elements truthfully, but it does not expose selector occurrences with source spans and enclosing control ancestry.
- WorkspaceIndex collapses same-file evidence-free duplicate triples. A plain `Template --UsesSelector--> SelectorRef` edge therefore cannot distinguish the same selector used under different conditions. #141's requirement to report the condition of each child use is not satisfied by the earlier Phase 5 proposal.
- #142 does not justify a global coverage, refresh, indexing, statistics, or error-schema redesign on this branch.

### #141 mapping

| Finding | Status | Owning phase | Existing/new regression authority | Remaining production work |
|---|---|---|---|---|
| Arbitrary external `templateUrl` filename fails | RESOLVED - Phase 0-3 | Phase 1 | `provide_angular_template_ownership::decorator_linked_nonstandard_html_uses_angular_template_processing` | None for presentation routing. Phase 4 must reuse the same structured metadata authority. |
| Sequential and nested control-flow ownership | RESOLVED - Phase 0-3 | Phase 2 | sequential and mixed nesting tests in `template_structure_regressions.rs` | Semantic selector extraction must consume structure, never rendered text. |
| Operator-bearing `@if` conditions | RESOLVED - Phase 0-3 | Phase 2 | sequential-if regression includes `||`, `&&`, optional chaining, and negation | None for presentation. |
| `@else if` retained distinctly | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 preservation / before Phase 5 | implementation recognizes and normalizes `@else if`; no direct tracked regression | Add a preservation regression before changing the ordered authority. |
| `@for` loop and `track` clause | RESOLVED - Phase 0-3 | Phases 2-3 | compressor test plus version-equivalence syntax-specific assertion | None for presentation. |
| `@empty` companion | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 preservation / before Phase 5 | implementation recognizes `@empty`; no direct tracked regression | Pin order and ownership before semantic extraction evolves. |
| `@switch`, case values, default, and legacy switch equivalence | RESOLVED - Phase 0-3 | Phase 3 | version-equivalence switch matrix, including literal quote preservation | None for presentation. |
| `@defer` triggers and companions | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 preservation / before Phase 5 | balanced header scanner recognizes defer, placeholder, loading, error; no direct regression | Pin triggers, companion headers, order, and owned children. |
| `@let` name and expression | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 preservation / before Phase 5 | scanner retains through the terminating semicolon; no direct regression | Add a direct preservation assertion. |
| Legacy `*ngIf` / `*ngFor` representation | RESOLVED - Phase 0-3 | Phases 2-3 | legacy structure and cross-version equivalence tests | Retain both normalized behavior and written microsyntax. |
| Visible text and interpolation expressions | RESOLVED - Phase 0-3 | Phase 2 | high-fidelity content regression and structural fidelity matrix | None for verified fidelities. |
| Pipes, template refs, and i18n/static attributes | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 preservation / Phase 6 disclosure | raw ordered text/start-tag events retain them; direct regression matrix is incomplete | Pin representative cases and disclose intentional fidelity omissions. |
| Bindings remain attached to their owning element | RESOLVED - Phase 0-3 | Phases 2-3 | structured and MCP production-path tests assert complete start tags | Selector semantics must read element events, not flattened binding lists. |
| Source order | RESOLVED - Phase 0-3 | Phases 2-3 | sequential, mixed, and live MCP-path order assertions | Preserve while enriching selector occurrence structure. |
| Comments | PARTIALLY RESOLVED - concrete remainder identified | Phase 2 contract / Phase 6 | High/Edit retain comments; Medium deliberately omits them | Report the omission instead of `degradation: null`. |
| Truthful fidelity/degradation | PHASE 6 | Phase 6 | existing error test covers raw verbatim wording; no skeleton degradation regression | Publish fidelity-specific omission metadata. Low must identify itself as aggregate, not ordered containment. |
| Component <-> Template linkage | PHASE 4 | Phase 4 | none yet | Add identity, `HasTemplate`, ownership, lifecycle, and forward/reverse queries after identity approval. |
| Template selector -> child Component linkage | PHASE 5 | Phase 5 | Phase 3 proves selector visibility only, not graph linkage | Add neutral occurrence evidence and safe unique resolution without losing conditions. |
| Include template with component request | PHASE 6 | Phase 6 | explicit batch retrieval already exists | Decide request composition separately; do not grow singular responses by default. |
| Missing/partial template diagnostics | PHASE 6 | Phase 6 | generic HTML fallback wording is covered | Add linked-template missing/partial/publication diagnostics. |
| Unsupported-extension message omits raw/no-savings meaning | RESOLVED - Phase 0-3 | Phase 1 | `unrelated_html_error_explains_that_verbatim_is_raw_uncompressed_content` | None. |
| Inline templates were not field-tested | PHASE 4 | Phase 4 | inline presentation existed before this branch | Add inline identity/link/query and transition lifecycle regressions. |
| Child query must report controlling condition | PARTIALLY RESOLVED - concrete remainder identified | Phase 5 contract revision | presentation shows conditions; graph model has no occurrence/control evidence | Freeze a template-specific occurrence response/evidence contract before Phase 5 implementation. |

### #142 ownership and split

| #142 finding | Ownership | Rationale | Owning phase or workstream |
|---|---|---|---|
| Coverage never establishes global completeness | TEMPLATE SLICE only for Template/selector queries | Template queries must distinguish exact capability from discovery completeness; a global schema redesign is unnecessary | Phase 6 |
| Reason for template-query partiality is absent | TEMPLATE SLICE | Missing template, incomplete discovery, unsupported template structure, ambiguity, and publication failure are materially different | Phase 6 |
| Add coverage to every graph, skeleton, and diff response | PLATFORM | This changes unrelated tools and contracts | Separate coverage-platform investigation |
| Response-derived evidence establishes exact capability | TEMPLATE SLICE | A safely returned `HasTemplate` or typed component-use answer proves that capability, not complete discovery | Phases 4-6 |
| `refresh_workspace` affects configured additional roots | PLATFORM | Existing scope semantics intentionally use the effective configured root set; changing it affects all languages and queries | Separate refresh-scope investigation |
| Refresh must retract stale template facts | TEMPLATE SLICE | Existing WSC-010 mechanics must include whichever files own template assertions | Phases 4-5 |
| `index_repository` does not warm `workspace_query` | PLATFORM | CBM and WorkspaceIndex have separate authorities; Angular must not merge them | Separate indexing/CBM investigation |
| Graph indexing can remain in progress indefinitely | PLATFORM | Not specific to template evidence or resolution | Separate CBM lifecycle investigation |
| Raw verbatim fallback wording | RESOLVED - Phase 0-3 | Phase 1 corrected the Angular/generic-HTML message | No remaining work |
| Generic edit/diff error misclassification | PLATFORM | Unrelated to template semantic linkage | Existing issue workstream |
| Cold reverse query is unactionable | TEMPLATE SLICE when querying Template/SelectorRef/Component use; otherwise PLATFORM | Angular queries need authorized discovery and truthful zero/partial states; generic behavior remains outside scope | Phase 6 plus separate generic-query work |
| `identity_not_indexed` lacks a next step | TEMPLATE SLICE for new identities | Template-specific alternatives can be precise without changing every identity type | Phase 6 |
| Session statistics include prior sessions | PLATFORM | Statistics ownership is unrelated to semantic linkage | Separate statistics investigation |
| Stale external PostToolUse hook | OUT OF SCOPE | User-side configuration, explicitly not a Clean-CTX defect | None |
| Repeated-query recall and concurrent-session behavior untested | PLATFORM unless a template-specific reproduction appears | No current evidence requires global lifecycle changes for this slice | Separate investigation if reproduced |

### Identity authorities before Phase 4/5

| Concept | Current or required authority |
|---|---|
| Component | Existing `angular/Component/<class-name>` declaration identity from structured `@Component` metadata. File provenance distinguishes physical occurrences; no duplicate reference-site Component is created. |
| External Template | Structured `templateUrl` plus the declaring TypeScript path resolves the target. The authoritative public name is the existing canonical physical/semantic-owner path. Workspace-relative paths are presentation only; basename and final-segment identities are rejected. |
| Inline Template | Structured `template:` owned by one Component declaration. Its authoritative name is the declaring file's canonical physical/semantic-owner path plus a deterministic Component-scoped inline discriminator. |
| Selector reference | Required `angular/SelectorRef/<written-element-selector>`. It is neutral evidence and never a Component declaration identity. |
| Resolved child Component | Existing `angular/Component/<class-name>` declaration occurrence reached through an authoritative `HasSelector` declaration edge and exact scope/discovery rules. |

### Phase 4 contract - approved

- Subject identity is `angular/Component/<class-name>`, not `builtin/Component`.
- Relation is the existing `SemanticRelation::HasTemplate`.
- Object type is `angular/Template`.
- Structured component metadata is the sole source authority for inline/external references.
- The declaring TypeScript file owns each `HasTemplate` assertion and therefore owns replacement, external-to-inline transition, republish, deletion of the component source, restore, and compatibility behavior.
- An external target disappearing does not erase the still-written `templateUrl` fact. The edge remains neutral source evidence and Phase 6 reports the target as missing. Changing/removing the component metadata retracts the edge.
- Inline and external identities never alias. Repeated publication deduplicates one assertion occurrence.
- Forward Component-to-Template and reverse Template-to-Component use ordinary exact edge queries after authorized hydration.
- `HasStyle` is excluded from Phase 4. It is not necessary for the template slice and should not share a generation bump merely because its enum variant exists.
- Durable TypeScript semantic output changes, so the existing `AngularSemantic` producer generation increments from 1 to 2. No new producer key is justified for `HasTemplate`.
- Older Angular semantic snapshots are incompatible and must be rejected/regenerated through existing compatibility machinery.
- **Approved identity decision:** the authoritative external Template name is the existing canonical physical/semantic-owner path. Workspace-relative paths are presentation only and never participate in semantic identity, persistence, or compatibility.
- Inline Template names use that same declaring-file authority plus a deterministic Component-scoped inline discriminator.
- Do not propagate workspace-root identity into semantic compilation on this branch.
- Basename and final-segment identities are rejected.

### Phase 5 contract - revisions required before implementation

- Add a neutral `UsesSelector` relation only if repository vocabulary confirms no existing relation has this meaning.
- Neutral object identity is `angular/SelectorRef/<written-selector>`; it never uses `angular/Component`.
- Selector facts originate from the ordered template authority, never the rendered skeleton.
- The ordered authority must expose each component-element occurrence, source span, and enclosing legacy/modern control ancestry before it can produce the #141-required conditional-use answer.
- A plain evidence-free WorkspaceIndex triple is insufficient: same-file duplicate uses under different conditions collapse. Do not persist it until occurrence ownership is represented correctly.
- Recommended smallest model: derive selector occurrences and typed component-use answers ephemerally during the authorized query, returning neutral occurrence records with structural context. Do not persist `UsesComponent`.
- If durable neutral selector edges are later required, approve template-specific occurrence evidence, its edge-key identity, durable encoding, producer compatibility, and template-file lifecycle first. This would be a genuinely distinct template-source authority and may justify a distinct producer key; convenience does not.
- Resolve a typed child only when exactly one authoritative in-scope Component declaration owns the written selector and authorized discovery completed sufficiently for that claim.
- Zero matches, multiple matches, incomplete discovery, failed discovery, and unsupported extraction remain neutral and carry distinct diagnostics.
- Typed refinement never removes neutral evidence.
- A child selector change or declaration removal affects the next query-time answer without persisting a stale cross-file edge.
- Legacy, modern, and mixed nesting use the same selector occurrence model.

### Phase 5 occurrence model/API contract - approved

Phase 5 is authorized only within this Angular template vertical slice. Do not generalize the machinery into a broader semantic-linkage abstraction.

#### Ordered-authority model

Extend the existing private `OrderedTemplate` authority without changing its verified `render()` projection:

```rust
pub(crate) struct SourceSpan {
    pub start_byte: usize,
    pub end_byte: usize,
}

pub(crate) enum TemplateControlKind {
    If,
    ElseIf,
    Else,
    For,
    Empty,
    Switch,
    Case,
    Default,
    Defer,
    Placeholder,
    Loading,
    Error,
}

pub(crate) enum TemplateControlSyntax {
    Legacy,
    Modern,
}

pub(crate) struct TemplateControlFrame {
    pub kind: TemplateControlKind,
    pub syntax: TemplateControlSyntax,
    pub written_header: String,
    pub span: SourceSpan,
}

pub(crate) struct SelectorOccurrence {
    pub written_selector: String,
    pub span: SourceSpan,
    pub control_ancestry: Vec<TemplateControlFrame>,
}
```

- `written_selector` is the exact normalized element tag written in the template, not a class name or filename inference.
- `span` is the custom element opening-tag range (`<child-widget ...>` or `<child-widget .../>`) and distinguishes repeated same-selector uses. Every span is `[start_byte, end_byte)`: zero-based UTF-8 byte offsets, never characters, UTF-16 units, columns, or LSP positions.
- Occurrence identity is `(Template identity, start_byte, end_byte, written_selector)` within one current source snapshot.
- Control ancestry is evidence, not identity. Frames are ordered outermost to innermost.
- Modern control intervals are derived from structured source positions and balanced block boundaries before selector association; rendered event text is never reparsed.
- Legacy structural directives are attached from the owning element and its ancestor elements.
- Mixed templates merge modern and legacy frames by containment/source nesting.
- `OrderedTemplate` retains occurrences in source order and exposes a read-only `selector_occurrences()` accessor. The existing flat `custom_elements` collection may continue serving PrimeNG rendering only; it is not semantic authority.

#### Query-time API shape

Selector facts remain ephemeral and are composed into the exact Angular template query response. They are not inserted into `WorkspaceIndex`, snapshots, or producer output.

Each occurrence returns a neutral relationship:

```text
angular/Template/<canonical-template-id>
    --UsesSelector-->
angular/SelectorRef/<written-selector>
```

The generic MCP edge schema may omit occurrence evidence for relationships that do not have it. Every Phase 5 occurrence-derived `UsesSelector` and `UsesComponent` result MUST carry `evidence.template_occurrence`; discarding that evidence would collapse distinct same-selector uses. The response edge shape is:

```json
{
  "relation": "UsesSelector",
  "subject": { "domain": "angular", "entity_type": "Template", "name": "<template-id>" },
  "object": { "domain": "angular", "entity_type": "SelectorRef", "name": "child-widget" },
  "occurrence": 0,
  "evidence": {
    "template_occurrence": {
      "span": { "start_byte": 128, "end_byte": 176 },
      "control_ancestry": [
        {
          "kind": "If",
          "syntax": "Modern",
          "written_header": "@if(showChild)",
          "span": { "start_byte": 96, "end_byte": 190 }
        }
      ]
    },
    "resolution": {
      "status": "unique",
      "discovery_complete": true
    }
  }
}
```

When and only when authorized Component discovery is complete and exactly one in-scope `HasSelector` declaration matches the written selector, the same occurrence may additionally produce:

```text
angular/Component/<parent-class>
    --UsesComponent-->
angular/Component/<unique-child-class>
```

The typed edge carries the same required occurrence evidence and never replaces the neutral edge. The authoritative neutral `UsesSelector` result remains queryable and present when `UsesComponent` adds uniquely resolved knowledge.

#### Resolution and diagnostics

- `unique`: exactly one authoritative Component declaration and discovery complete; emit neutral plus typed evidence.
- `unresolved`: discovery complete and zero declarations; emit neutral evidence only.
- `ambiguous`: discovery complete and multiple declarations; emit neutral evidence plus candidate identities.
- `incomplete`: discovery is partial; emit neutral evidence and do not type even when one candidate is currently visible.
- `failed`: authorized discovery failed; emit neutral evidence with the failure diagnostic.
- `unsupported`: the template could be read but occurrence/control extraction could not establish the required structure; do not fabricate occurrences.

Capability may be established for the exact neutral occurrence query when occurrences were extracted. Typed capability may be established for returned unique refinements. Neither statement establishes workspace completeness.

#### Lifecycle and ownership

- The current Template source owns occurrence extraction.
- The declaring Component supplies the parent identity through current `HasTemplate` evidence.
- Child Component declarations supply selector candidates through current `HasSelector` evidence.
- Because the typed answer depends on all three authorities, it is recomputed per authorized query and has no arbitrary durable file owner.
- Edit, replacement, deletion, restoration, refresh, selector change, or ambiguity changes are observed from current source/index state; no Phase 5 producer generation is introduced.
- Response occurrence ordinals are assigned after source-order sorting and are presentation handles only. Byte spans plus Template identity are the within-snapshot occurrence key.

#### Approved Phase 5 decisions

1. Public relations are `UsesSelector` and `UsesComponent`.
2. Neutral identity is `angular/SelectorRef/<written-selector>`.
3. Occurrence evidence is optional generically but required on every occurrence-derived Phase 5 edge.
4. Spans are `[start_byte, end_byte)`, zero-based UTF-8 byte offsets.
5. Each occurrence retains its neutral result and may add one typed result only after unique, complete resolution.
6. All Phase 5 relationships are query-time-only: no durable selector edges, producer key, generation change, or snapshot state.
7. Phase 5 introduces no broader shared abstraction; architectural generalization is deferred to the Meta-Layer Architectural Hardening / Semantic Synthesis investigation.


### Phase 6 contract

- Existing explicit batch retrieval remains the supported composition mechanism during Phases 4-5.
- A singular `includeTemplate` option remains deferred. If later approved, it must define budget ownership, item-local errors, missing targets, deduplication, and response growth.
- Exact returned `HasTemplate` or uniquely refined component-use evidence may set `capability_established=true` for that exact relation/query.
- `source_complete=true` requires completed authorized discovery for every source class relevant to that exact query. A non-empty result is insufficient.
- Zero results never assert absence when discovery is partial or failed.
- Template-specific partiality reasons include: `linked_template_missing`, `component_discovery_incomplete`, `template_discovery_incomplete`, `template_parse_unsupported`, `selector_unresolved`, `selector_ambiguous`, and `semantic_publication_failed`.
- Missing external targets retain their neutral `HasTemplate` reference and report missing target state.
- Skeleton responses disclose fidelity omissions. Low declares aggregate/unordered semantics; Medium declares omitted comments; raw passthrough/verbatim remains byte-exact and uncompressed.
- Alternative-query guidance remains available only when it points to a sound neutral query.
- Generic global coverage-schema, refresh-scope, CBM, and statistics changes remain outside this branch.

### Assertion ownership and lifecycle matrix

| Transition | `HasTemplate` owner/result | Selector occurrence/result |
|---|---|---|
| Publish or unchanged republish | TypeScript owner publishes one deduplicated assertion | Query reads current ordered template; no persisted typed edge |
| Change `templateUrl` | TypeScript recompilation replaces old target edge | Next query reads the new target |
| External to inline or inline to external | TypeScript recompilation replaces identity kind atomically | Next query reads the selected current source |
| External template edited | `HasTemplate` is unchanged | Next authorized read/refresh derives current occurrences |
| External template deleted | Written `HasTemplate` remains; target status becomes missing | No selector occurrences; diagnostic is not absence proof |
| External template restored | Same reference becomes reachable again | Current occurrences are derived again |
| Selector added/removed/replaced | No `HasTemplate` change | Current query result changes; no stale durable typed edge |
| Child selector changes or duplicate appears/disappears | No `HasTemplate` change | Unique refinement becomes unresolved/ambiguous or resolves anew |
| Component declaration disappears | TypeScript-owned `HasTemplate` retracts | Candidate declaration is no longer eligible |
| Refresh | Existing root reconciliation retracts affected source owners | Query must not reuse pre-refresh template/candidate evidence |
| Snapshot generation mismatch | Angular semantic snapshot rejected/regenerated | Query-time facts have no snapshot to restore |

### Dependency-ordered remaining sequence

1. Add only the missing Phase 0-3 preservation regressions identified by #141; do not redesign verified presentation.
2. Resolve and explicitly approve the public Template path identity option.
3. Add Phase 4 RED tests for external, arbitrary, nested, inline, forward/reverse, replacement, transition, deduplication, deletion, restore, refresh, and generation mismatch.
4. Implement `HasTemplate` through the existing structured Angular semantic producer; bump only `AngularSemantic`.
5. Enrich the ordered authority with selector occurrence/span/control ancestry without changing its verified rendering projection.
6. Freeze the Phase 5 occurrence response/evidence contract and add RED tests for unique, missing, ambiguous, legacy, modern, mixed, replacement, and child-selector lifecycle.
7. Implement neutral query-time selector evidence and unique typed refinement using existing discovery/hydration mechanics.
8. Add Phase 6 coverage, diagnostics, and fidelity omission metadata locally to the Angular surfaces.
9. Perform the final architectural audit, authoritative user-run verification gate, then one batched live MCP field pilot.

### Explicitly deferred #142 platform work

- Global `refresh_workspace` root-selection semantics.
- Generic WorkspaceIndex/CBM/`index_repository` lifecycle and progress.
- Universal coverage objects and global completeness policy.
- Session-statistics ownership/persistence.
- Generic edit/diff/error-message cleanup.
- Unrelated cold-index and graph-query behavior.
- Concurrent-session recall investigation without an Angular-specific reproduction.

### Review verdict

**PHASE 6 GREEN — FINAL VERIFICATION PENDING.**

The user-reported focused tracked Phase 5 gate is GREEN. This records tracked regression status only; no live MCP field pilot has been run for Phase 5.

The user-reported focused tracked Phase 6 diagnostics gate is GREEN, including the reverse-Component completeness safeguard added during the final audit. Documentation is aligned and the bounded architectural audit found no critical or high-severity implementation gap. The maintainer reported the authoritative final verification gate GREEN. The consolidated live MCP pilot also passed with no stale-binary warning or failed check; its tracked driver is `verification/live-acceptance/angular_template_linkage_live_acceptance.mjs` and generated evidence is under `target/live-acceptance/angular-template-linkage/`.

Phase 4 is implemented and the focused tracked regressions were reported GREEN by the user. The canonical physical/semantic-owner Template identity remains authoritative; workspace-relative rendering is presentation-only, and no workspace-root identity is propagated into semantic compilation.

This does not reopen Phases 0-4. Phase 5 is authorized only as ephemeral Angular template occurrence extraction and query-time refinement under the approved contract. Durable selector publication and generalized semantic-linkage machinery remain prohibited.

## Approval checkpoints

1. **Correctness baseline:** ownership, ordered representation, and Angular 15+ equivalence (Phases 0–3).
2. **Template identity:** selected Template identity, HasTemplate, and producer generation (Phase 4).
3. **New graph capability:** selector relation, producer, hydration, lifecycle, and refinement policy (Phase 5).
4. **Request composition:** includeTemplate only if existing batching and related-file guidance are insufficient.

This separation allows the misleading-output defect to be corrected without implicitly approving every graph or API expansion.
