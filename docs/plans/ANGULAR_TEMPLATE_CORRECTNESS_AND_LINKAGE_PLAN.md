# Angular Template Correctness and Linkage Plan

**Author:** Agent MaxHeadRoom  
**Date:** 2026-10-10  
**Status:** Phase 1 GREEN; Phase 2 ordered rendering awaiting GREEN  
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

## Approval checkpoints

1. **Correctness baseline:** ownership, ordered representation, and Angular 15+ equivalence (Phases 0–3).
2. **Template identity:** selected Template identity, HasTemplate, and producer generation (Phase 4).
3. **New graph capability:** selector relation, producer, hydration, lifecycle, and refinement policy (Phase 5).
4. **Request composition:** includeTemplate only if existing batching and related-file guidance are insufficient.

This separation allows the misleading-output defect to be corrected without implicitly approving every graph or API expansion.
