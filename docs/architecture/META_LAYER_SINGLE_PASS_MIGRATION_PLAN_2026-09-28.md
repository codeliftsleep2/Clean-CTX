# Meta-layer single-pass migration plan — 2026-09-28

**Status:** Approved investigation and phased implementation plan; Phases 0–5
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

**Status:** Complete; focused work-count RED/GREEN regressions reported for
every planned extractor family.

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

The testing, Signals, Routing, RxJS, NgRx, Reactive Forms, and Formly
extractors now accept the compilation-scoped index. Helpers that operate on a
distinct nested expression or reducer-body slice build one local index for
that slice. The only remaining Angular reference to
`is_inside_comment_or_string` is the compatibility re-export in
`angular_meta::util`; no Angular production extractor calls it.

Regression authority:

- `src/tests/angular_meta/testing.rs::testing_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/ir/pipeline_meta_layer.rs::production_testing_family_reuses_meta_pass_lexical_index`
- `src/tests/angular_meta/signals.rs::signal_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/angular_meta/routing.rs::routing_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/angular_meta/rx_lexical.rs::rxjs_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/angular_meta/ngrx.rs::ngrx_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/angular_meta/reactive_forms.rs::reactive_forms_extraction_does_not_use_legacy_prefix_membership_scans`
- `src/tests/angular_meta/formly.rs::formly_forms_extraction_does_not_use_legacy_prefix_membership_scans`

**Exit criterion:** No Angular full-source match loop calls the byte-zero legacy
predicate repeatedly. Legitimate bounded substring scans may remain where they
operate on a distinct local slice.

### Phase 5 — Evaluate each framework once

**Status:** Complete; focused registry and production-lifecycle RED/GREEN
regressions reported.

**Goal:** Collapse marker and semantic extraction into one registry traversal.

1. Add a combined layer evaluation method returning `MetaLayerEvaluation`.
2. Provide a compatibility adapter while frameworks migrate.
3. Migrate Angular, .NET, Spring, then the always-on builtin layer.
4. Update `MetaLayerPass` to consume output and edges from the same evaluation.
5. Preserve marker-alias insertion order and provenance attachment.
6. Remove the second `is_applicable` traversal when every registered layer uses
   the combined boundary.

`MetaLayerPass` now calls one combined registry route. The registry makes one
applicability decision and consumes one `MetaLayerEvaluation` from every
applicable layer. Angular, .NET, Spring, and builtin own explicit evaluation
overrides; the trait default remains only as a compatibility adapter for future
out-of-tree or incremental implementations. Angular additionally shares its
NgRx and Routing shapes between marker rendering and semantic projection.

Regression authority:

- `src/tests/layers/registry.rs::combined_registry_evaluation_checks_applicability_once_per_layer`
- `src/tests/layers/registry.rs::combined_registry_dispatch_invokes_one_layer_evaluation_hook`
- `src/tests/ir/pipeline_meta_layer.rs::production_meta_pass_uses_one_combined_registry_evaluation`
- `src/tests/ir/pipeline_meta_layer.rs::production_angular_evaluation_extracts_ngrx_shape_once`
- `src/tests/ir/pipeline_meta_layer.rs::production_angular_evaluation_extracts_routing_shape_once`
- `src/tests/ir/pipeline_meta_layer.rs::production_dotnet_layer_uses_combined_evaluation_override`
- `src/tests/ir/pipeline_meta_layer.rs::production_spring_layer_uses_combined_evaluation_override`
- `src/tests/ir/pipeline_meta_layer.rs::production_builtin_layer_uses_combined_evaluation_override`

**Exit criterion:** One applicability decision and one evaluation occur per
registered layer per compilation.

### Phase 6 — Remove redundant detection parses

**Status:** Complete (2026-09-28).

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

Angular now carries its decorator-detection result through applicability,
marker extraction, and semantic projection. .NET and Spring similarly retain
their successful applicability evidence and use explicit applicable-source
evaluation paths rather than invoking framework detection again. Focused
production regressions enforce one detection pass per compilation:

- `production_angular_layer_runs_framework_detection_once`
- `production_dotnet_layer_runs_framework_detection_once`
- `production_spring_layer_runs_framework_detection_once`

### Phase 7 — Measure before broader scan fusion

**Status:** Measurement harness implemented; results pending maintainer run.

**Goal:** Decide whether P4 warrants more architecture.

- Add deterministic instrumentation or a bounded benchmark harness outside the
  correctness gate.
- Measure representative Angular, ASP.NET/.NET, and Spring files after Phases
  2–6.
- Retain independent linear sub-layer passes unless remaining cost is material.
- Any shared tokenizer or broader scan-fusion design requires a separate
  approval with measured evidence and explicit complexity tradeoffs.

`examples/meta_layer_benchmark.rs` provides the bounded production-path
comparison. It measures each representative framework source with its
meta-layer enabled and explicitly disabled, reports the directional delta, and
makes no timing assertion. Run it in release mode with all features enabled;
repeat runs before using the results to make an architectural decision.

Three maintainer-run samples on 2026-09-28 produced:

| Framework | Run 1 | Run 2 | Run 3 | Mean share | Mean delta |
|---|---:|---:|---:|---:|---:|
| Angular | 5.98% | 5.58% | 9.81% | 7.12% | 4,205.34 us |
| .NET | 13.85% | 18.12% | 13.59% | 15.19% | 13,502.10 us |
| Spring | 3.33% | 7.10% | 1.39% | 3.94% | 456.36 us |

These initial measurements used block-ordered modes and are retained as the
historical decision evidence, not as a reliable before/after baseline. A later
.NET-only change coincided with large apparent Angular and Spring movement,
including negative Spring deltas, proving that run-order/environment drift was
material. The harness now alternates enabled and disabled modes for every
sample and reports the median paired delta. Measurements from the two harness
versions must not be compared directly.

The evidence does not justify repository-wide scan fusion. Spring should retain
its current independent passes. Angular remains below the threshold for added
architecture and should remain unchanged unless a larger representative corpus
shows a stable material cost. .NET is the only targeted follow-up candidate.

The .NET inventory found seven independent marker extractors per applicable
class, followed by a semantic projection that re-extracts overlapping ASP.NET,
EF Core, AutoMapper, SignalR, and testing facts. A shared .NET per-class
evaluation shape could remove that duplicate work, but it changes ownership
between marker and semantic projections and therefore requires explicit
architectural approval before implementation.

The maintainer approved the targeted .NET change on 2026-09-28. The first
increment moves ASP.NET controller facts into one per-class analysis result
consumed by both marker and semantic projections. The production regression
`production_dotnet_layer_analyzes_each_class_once_for_both_projections` was
observed RED at two analyses and GREEN at one. Remaining .NET families stay on
their established paths until focused equivalence checks and a follow-up
measurement show whether further sharing is warranted.

Three runs of the corrected paired-median harness after the ASP.NET increment
produced:

| Framework | Run 1 | Run 2 | Run 3 | Mean share |
|---|---:|---:|---:|---:|
| Angular | 9.14% | 8.82% | 9.82% | 9.26% |
| .NET | 13.38% | 13.03% | 13.12% | 13.18% |
| Spring | 0.66% | -1.61% | -0.37% | -0.44% |

Spring requires no broader fusion. The .NET cost remains material and supports
continuing the approved targeted migration with EF Core next. Angular also has
a stable measurable cost, but expanding its architecture is a separate
decision and is not authorized by the .NET approval.

The EF Core shared shape preserved behavior but produced no measurable
production-path improvement: the next paired run reported 13.22% .NET overhead
against the 13.18% pre-EF mean. Inspection identified the remaining standalone
C# tree-sitter parse and two detection queries as the more plausible dominant
cost. The next .NET increment therefore targets reuse of compilation-scoped
lexical evidence for applicability, with zero additional framework-detection
parse, before any more extractor families are consolidated.

The first corrected paired-median run after removing that parse reported .NET
at 0.66% / 498.75 us, down from 13.22% / 11,674.80 us immediately before the
change. This identifies the redundant detection parse as the dominant cost;
further .NET extractor fusion is not justified unless confirmation runs
contradict this result. The standalone detector retains its AST contract for
callers outside the compilation pipeline, while the production compilation
route uses the already-owned lexical evidence.

A confirmation run reported .NET at 0.25% / 189.90 us and Spring at -0.37%,
closing both as broader-fusion targets. Angular remained material at 11.12% /
4,919.90 us. The maintainer explicitly directed that stable overhead above 9%
is not acceptable, authorizing the targeted Angular investigation. Inspection
found the same dominant pattern: a standalone TypeScript parse and decorator
query during applicability despite an existing compilation parse and shared
lexical index.

The first corrected paired-median run after removing the Angular applicability
parse reported Angular at 0.63% / 238.10 us, down from 11.12% / 4,919.90 us.
The same run measured .NET at -0.08% and Spring at 0.26%. All three framework
meta-layers are therefore within measurement noise, and the evidence rejects
broader scan fusion unless confirmation runs materially contradict it.

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
