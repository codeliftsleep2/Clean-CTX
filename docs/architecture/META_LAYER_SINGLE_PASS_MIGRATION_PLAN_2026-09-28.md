# Meta-layer single-pass migration plan — 2026-09-28

**Status:** Approved investigation and phased implementation plan; Phases 0–3
complete, later phases pending RED/GREEN implementation.

**Scope:** Framework detection, marker extraction, and semantic-edge extraction
for Angular, .NET, and Spring Boot. The migration removes redundant work inside
one file compilation without changing emitted markers, semantic facts, feature
defaults, fidelity behavior, ordering, or public MCP contracts.

**Motivation:** A field audit found repeated whole-prefix lexical scans,
duplicate framework-detection parses, two registry traversals that repeat
applicability checks, and independent framework sub-layer scans over the same
source. The first small duplication in Angular testing extraction has already
been removed under focused RED/GREEN coverage. The remaining work crosses the
shared meta-layer boundary and must be migrated incrementally.

---

## 1. Current production lifecycle

The current architecture has two consumers of framework presentation:

1. the compression output path calls the layer registry to produce structured
   and rendered framework blocks; and
2. the IR `MetaLayerPass` calls the registry for marker output and then calls it
   again for semantic edges.

The IR path currently behaves approximately as follows:

```text
capture pipeline
  -> PassContext.source + Vec<CapEntry>
  -> materialize owned class-capture Strings
  -> registry.run_meta_layers_pipeline_with_path
       -> for each layer: is_applicable -> enrich
  -> append marker aliases
  -> registry.collect_semantic_edges_with_path
       -> for each layer: is_applicable -> extract_semantic_edges
  -> attach file provenance
```

Important facts:

- `PassContext` retains source text, capture entries, language, query string,
  path identity, fidelity, and configuration.
- It does **not** retain the tree-sitter `Tree` produced during capture.
- Angular, .NET, and Spring applicability detection may construct and parse a
  new framework-language tree.
- In the IR path, applicability can therefore execute twice for one framework
  during one compilation.
- Angular sub-layers make many calls to
  `is_inside_comment_or_string(source, pos)`. That helper starts at byte zero
  for every query, so repeated matches can create quadratic work.
- Marker production and semantic-edge production use overlapping framework
  extraction but are exposed through separate trait calls.
- Class captures are cloned into owned strings at multiple adapter boundaries.

The existing implementation is the behavioral reference until each replacement
phase has reached production and its old path has been removed.

---

## 2. Problem classification

### P1 — Repeated prefix lexical scans

`meta_util::is_inside_comment_or_string` performs a state-machine scan from byte
zero to the requested position. It is correct for its supported lexical model,
but using it once per textual match makes ordinary NgRx, RxJS, routing, forms,
signals, and testing extraction potentially quadratic.

This is presently concentrated in Angular, but the helper is layer-agnostic and
the correct reusable owner belongs in `meta_util`.

### P2 — Repeated applicability work

The registry independently asks whether a layer applies before marker
enrichment and before semantic-edge extraction. Expensive applicability checks
are not owned by one compilation-scoped evaluation.

This affects the shared meta-layer architecture, not only Angular.

### P3 — Duplicate tree-sitter parses

Angular, .NET, and Spring detection may parse source solely to decide whether a
framework applies even though the upstream language capture pipeline already
processed the file. The upstream `Tree` is not retained, so “reuse the existing
tree” is not currently an available local edit.

The migration must first determine whether existing `CapEntry` evidence is
sufficient. If a framework-specific detection query still requires a tree, the
compilation-scoped meta-layer evaluation may parse once and share that result;
it must not parse once for markers and again for edges.

### P4 — Independent framework sub-layer scans

Angular and .NET intentionally compose specialized extractors. Multiple linear
passes are not automatically a defect: simple independent passes can be safer
than a generic tokenizer. They become a follow-up target only after P1–P3 are
removed and measurement shows material remaining cost.

No generic tokenizer, registry-within-a-registry, global cache, mutex, or
persistent parse cache is authorized by this plan.

---

## 3. Target architecture

Introduce one borrowed, compilation-scoped input and one combined result:

```text
MetaLayerContext<'a>
  source: &'a str
  path: &'a Path
  captures: borrowed capture view
  fidelity: Fidelity
  config: Option<&'a CleanCtxConfig>
  lexical_regions: lazily/once constructed per source
  framework parse evidence: optional and invocation-local

MetaLayerEvaluation
  output: Option<MetaLayerOutput>
  semantic_edges: Vec<SemanticEdge>
```

The registry performs one ordered traversal:

```text
for layer in configured order:
    evaluate(context) once
      -> decide applicability once
      -> reuse lexical/capture/parse evidence
      -> return markers/block and semantic edges together
```

The target preserves these ownership rules:

- The compilation owns source, captures, path, fidelity, and configuration.
- `meta_util` owns language-neutral lexical-region indexing.
- Each framework layer owns its vocabulary and any framework-specific parse.
- The registry owns configured layer order and one evaluation per layer.
- `MetaLayerPass` owns conversion of marker text to aliases and file-provenance
  attachment.
- No framework result or parse state survives the file compilation.

---

## 4. Non-negotiable invariants

1. Marker text and structured framework blocks remain byte-for-byte equivalent
   for the same source, fidelity, path, features, and configuration.
2. Semantic edges retain relation, subject/object identity, layer, order,
   duplicates, call evidence, and file provenance.
3. Layer registration and evaluation order remain unchanged.
4. Disabled layers retain zero framework-extraction work.
5. Detection remains conservative: comments, strings, templates, attributes,
   decorators, and using/import directives preserve existing classification.
6. Edit/Verbatim exact-body behavior is unaffected.
7. No global or cross-session parse/lexical cache is introduced.
8. No new limits, timeouts, truncation, parallel mutation, or public MCP fields
   are introduced.
9. Existing feature gates remain authoritative; Java/Rust/Spring/Angular/.NET
   defaults do not change.
10. Performance tests use deterministic work counters or structural ownership
    assertions, not wall-clock thresholds.

---

## 5. Phased implementation

### Phase 0 — Baseline and migration authority

**Status:** Complete through investigation and this plan.

- Trace both production entry points.
- Record that the upstream tree is not retained.
- Treat existing outputs and semantic edges as the equivalence oracle.
- Keep repository-wide tests CI-owned; use focused user-run regressions per
  phase plus bounded formatting, size, encoding, and diff checks.

### Phase 1 — Remove local duplicate scans

**Status:** Complete; focused RED/GREEN reported.

- Angular testing extraction now derives `describe_count` and `test_count`
  from the literal calls already collected.
- The separate count-only scan path was removed.
- Regression authority:
  `src/tests/angular_meta/testing.rs::testing_shape_has_no_count_only_rescan_path`.

This phase is intentionally local and does not claim the shared problem fixed.

### Phase 2 — Add a reusable lexical-region index

**Status:** Complete; focused RED/GREEN reported.

**Goal:** Replace repeated byte-zero prefix scans with one O(n) source scan and
O(log n) or O(1) membership checks.

1. Add a small immutable `LexicalRegions` type under `meta_util`.
2. Build ordered non-code spans for the exact lexical constructs currently
   supported by `is_inside_comment_or_string`.
3. Preserve the existing opening-marker boundary semantics.
4. Add equivalence regressions that compare the legacy predicate and the index
   at every byte boundary across line comments, block comments, quoted strings,
   escapes, templates, and interpolation.
5. Do not remove the legacy helper until all production call sites migrate.

Regression authority:
`src/tests/meta_util.rs::lexical_regions_match_legacy_scanner_at_every_byte_boundary`.

**Exit criterion:** The new index is behaviorally equivalent on the supported
lexical corpus and has no framework vocabulary.

### Phase 3 — Establish a compilation-scoped meta context

**Status:** Complete; focused adapter, registry-dispatch, and production-lifecycle
RED/GREEN regressions reported.

**Goal:** Give every layer access to shared immutable per-file evidence without
global caching or repeated allocation.

1. Add a borrowed `MetaLayerContext` at the shared layer boundary.
2. Carry source, path, fidelity, configuration, and a borrowed capture view.
3. Construct/share `LexicalRegions` once when a participating layer needs it.
4. Avoid cloning every class capture solely to cross the registry boundary.
5. Add adapters so existing layer behavior can migrate incrementally.
6. Pin layer order, disabled-layer behavior, and output equivalence.

The production `MetaLayerPass` now constructs one `LexicalRegions` and one
borrowed `MetaLayerContext`, then reuses that context for marker and semantic
dispatch. Compatibility adapters deliberately retain the pre-existing owned
class-text projection until framework implementations migrate; the registry
boundary itself adds no capture cloning.

Regression authority:

- `src/tests/layers/registry.rs::compilation_context_adapter_preserves_legacy_layer_output`
- `src/tests/layers/registry.rs::compilation_context_adapter_preserves_legacy_semantic_edges`
- `src/tests/layers/registry.rs::registry_context_dispatch_preserves_applicable_layer_output`
- `src/tests/layers/registry.rs::registry_context_dispatch_preserves_applicable_semantic_edges`
- `src/tests/ir/pipeline_meta_layer.rs::production_meta_pass_constructs_one_shared_lexical_index`

**Exit criterion:** Existing layers can consume the context through adapters;
no production behavior changes and no old path has been removed prematurely.

### Phase 4 — Migrate Angular lexical consumers

**Goal:** Eliminate the known quadratic comment/string checks.

Migrate complete extractor families rather than isolated call sites:

1. Testing
2. Signals and routing
3. RxJS
4. NgRx
5. Reactive forms and Formly

For each family:

- add a focused work-count or ownership regression;
- pass the shared lexical index through its internal helpers;
- preserve all existing false-positive regressions;
- remove its legacy predicate calls only after equivalence is green.

**Exit criterion:** No Angular full-source match loop calls the byte-zero legacy
predicate repeatedly. Legitimate bounded substring scans may remain where they
operate on a distinct local slice.

### Phase 5 — Evaluate each framework once

**Goal:** Collapse marker and semantic extraction into one registry traversal.

1. Add a combined layer evaluation method returning `MetaLayerEvaluation`.
2. Provide a compatibility adapter while frameworks migrate.
3. Migrate Angular, .NET, Spring, then the always-on builtin layer.
4. Update `MetaLayerPass` to consume output and edges from the same evaluation.
5. Preserve marker-alias insertion order and provenance attachment.
6. Remove the second `is_applicable` traversal when every registered layer uses
   the combined boundary.

**Exit criterion:** One applicability decision and one evaluation occur per
registered layer per compilation.

### Phase 6 — Remove redundant detection parses

**Goal:** Use the cheapest already-owned evidence that preserves detection
semantics.

For Angular, .NET, and Spring independently:

1. Determine whether capture/import/decorator/annotation evidence is sufficient
   for current detection behavior.
2. Prefer existing captures where equivalence can be proven.
3. If a framework AST query remains necessary, parse once inside that layer's
   compilation-scoped evaluation and reuse it for markers and edges.
4. Never replace AST detection with an unproven substring heuristic merely for
   speed.

**Exit criterion:** No framework reparses the same file more than once within
one meta-layer evaluation, and zero reparses occur where capture evidence is
sufficient.

### Phase 7 — Measure before broader scan fusion

**Goal:** Decide whether P4 warrants more architecture.

- Add deterministic instrumentation or a bounded benchmark harness outside the
  correctness gate.
- Measure representative Angular, ASP.NET/.NET, and Spring files after Phases
  2–6.
- Retain independent linear sub-layer passes unless remaining cost is material.
- Any shared tokenizer or broader scan-fusion design requires a separate
  approval with measured evidence and explicit complexity tradeoffs.

### Phase 8 — Finalize the migration

- Remove obsolete adapters, duplicate trait methods, legacy lexical calls, and
  stale comments.
- Update `docs/ARCHITECTURAL_INVARIANTS.md` with the resulting durable ownership
  and single-evaluation contract.
- Record the original field discovery and its regression authorities in
  `docs/agent/DISCOVERY_REGISTRY.md`.
- Perform the post-task architectural audit from `docs/agent/architecture.md`.
- Run focused local gates only; repository-wide tests remain CI-owned under the
  maintainer's local-development policy.

---

## 6. Regression strategy

Each phase must establish RED before changing its production path.

| Boundary | Regression evidence |
|---|---|
| Local duplicate scan | Structural single-owner assertion in Angular testing |
| Lexical index | Every-byte equivalence against the legacy predicate plus existing false-positive fixtures |
| Shared context | Layer order, feature/config gating, borrowed capture identity, one lexical construction |
| Angular migration | Per-family scan-count/ownership tests plus unchanged semantic fixtures |
| Combined evaluation | One applicability/evaluation count per layer; byte-identical marker blocks and equal semantic edges |
| Detection parse reuse | Deterministic parse-count seam per framework plus unchanged applicability corpus |
| Production integration | Registered `provide_code_context` and `workspace_query` paths for Angular, .NET, and Spring |

Test-only counters must be compilation/request-local or serialized through the
repository's established test guards. They must not introduce production
atomics, global state, timing thresholds, or persistent caches.

---

## 7. File-size and decomposition expectations

Several involved files are near the active-file target. New responsibilities
must be placed at semantic boundaries rather than appended to orchestration
files:

- lexical indexing under `src/meta_util/`;
- compilation context/evaluation types under `src/layers/meta/`;
- registry orchestration in `src/layers/registry.rs`;
- framework-specific adapters inside their framework modules;
- tracked tests under matching `src/tests/**` paths.

Every new or materially modified code file must remain at or below 615 lines,
preferably 600 or fewer. The migration may not move code into another oversized
hand-maintained file.

---

## 8. Explicit non-goals

- Changing framework facts or adding new framework detection heuristics.
- Changing enabled-by-default language or meta-layer features.
- Persisting syntax trees or lexical indexes across calls.
- Parallelizing meta-layer mutation.
- Combining language parsing and framework parsing into one generic grammar.
- Replacing clear independent O(n) extractors without measured justification.
- Changing SCHEMA-vNext, semantic-edge schemas, or MCP response shapes.

---

## 9. Completion definition

The migration is complete only when:

1. repeated byte-zero lexical scans are absent from production match loops;
2. every registered meta-layer is evaluated once per file compilation;
3. applicability is decided once per layer per compilation;
4. framework parsing is zero-or-one per layer evaluation;
5. marker output and semantic edges remain behaviorally equivalent;
6. obsolete adapters and duplicate paths are removed;
7. focused local regressions are green;
8. the architectural audit is complete; and
9. CI verifies the repository-wide all-feature gate.
