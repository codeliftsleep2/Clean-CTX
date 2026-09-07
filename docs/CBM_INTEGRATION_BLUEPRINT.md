# CBM-Clean-CTX Integration Blueprint

> **Status:** Architecture approved — implementation pending  
> **Created:** 2026-09-06  
> **Phases:** 34 (capability audit) → 35 (architectural reconstruction) → 36 (production design)  
> **Decision:** **D — Mixed** — Some mechanisms wire directly; others require targeted adaptation.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architectural Principles](#2-architectural-principles)
3. [The Two CBM Flows](#3-the-two-cbm-flows)
4. [Component Responsibility Model](#4-component-responsibility-model)
5. [Complete Data-Flow Diagrams](#5-complete-data-flow-diagrams)
6. [Layered Architecture](#6-layered-architecture)
7. [Migration History & Impact](#7-migration-history--impact)
8. [Mechanism Assessment](#8-mechanism-assessment)
9. [CBM 0.10.8 Capability Interpretation](#9-cbm-0108-capability-interpretation)
10. [Request-Scoped Intelligence Ownership](#10-request-scoped-intelligence-ownership)
11. [CBM Consultation Timing](#11-cbm-consultation-timing)
12. [Fidelity Pipeline Design](#12-fidelity-pipeline-design)
13. [Skip-Set Lifecycle Design](#13-skip-set-lifecycle-design)
14. [Failure Semantics](#14-failure-semantics)
15. [Freshness & Invalidation](#15-freshness--invalidation)
16. [GraphBridge Responsibility Boundary](#16-graphbridge-responsibility-boundary)
17. [Structured vs. Proxy-Rendered Data](#17-structured-vs-proxy-rendered-data)
18. [Implementation-Readiness Matrix](#18-implementation-readiness-matrix)
19. [Responsibility Matrix](#19-responsibility-matrix)
20. [Migration Gap Matrix](#20-migration-gap-matrix)
21. [Smallest Correct Implementation Sequence](#21-smallest-correct-implementation-sequence)
22. [Explicit Non-Goals](#22-explicit-non-goals)
23. [Boundary Constraints](#23-boundary-constraints)
24. [Open Questions](#24-open-questions)

---

## 1. Executive Summary

### Finding

The CBM 0.8.1 → 0.10.8 migration successfully updated the **transport and retrieval** layer but did **not** break the consumption architecture — because the consumption architecture was never completed in the first place.

The repository contains an **intentionally designed but never-completed** compiler-mediated CBM intelligence architecture. The mechanisms exist; the wiring does not.

### Architectural Intent (Established)

> **Clean-CTX should use CBM as an external advisory intelligence provider to improve context compilation — fidelity, symbol filtering, context expansion, and eventually other relevance decisions — while Clean-CTX's semantic model remains authoritative.**

### Two Distinct Flows

| Flow | Path | Status |
|------|------|--------|
| **A — Agent-direct** | LLM → cbm_proxy → CBM → LLM | ✅ Complete |
| **B — Compiler-mediated** | LLM → Clean-CTX → CBM → interpretation → context → LLM | ❌ Unwired |

### Decision

**D — Mixed.** Some mechanisms can be wired directly; others require targeted architectural adaptation.

---

## 2. Architectural Principles

### Non-Negotiable Rules

| Rule | Rationale |
|------|-----------|
| Clean-CTX semantic facts are **authoritative** | Core architectural invariant |
| CBM intelligence is **advisory** | CBM is external, unreliable |
| CBM facts must **never** become authoritative semantic facts | Boundary preservation |
| `cbm_proxy` is an **agent-facing rendering boundary** | Separation of concerns |

---

## 3. The Two CBM Flows

### Flow A — Agent-Direct (COMPLETE)

```
LLM/Agent
    ↓
Clean-CTX MCP (graph_search, graph_query, etc.)
    ↓
cbm_proxy (pipe-level interception + compression)
    ↓
CBM 0.10.8
    ↓
Raw CBM stdout → JSON compression → LLM
```

**Responsibilities:**
- Agent decides when to query CBM
- CBM result goes directly to LLM
- Clean-CTX does NOT process the result

**Status:** Fully functional. All 6 whitelisted tools + `cbm_proxy`.

### Flow B — Compiler-Mediated (UNWIRED)

```
LLM/Agent
    ↓
provide_code_context()
    ↓
Clean-CTX context request
    ↓
Task classification (content-based)
    ↓
CBM consultation decision (if needed)
    ↓
GraphBridge (structured access)
    ↓
CBM 0.10.8
    ↓
Structured CBM result (importance scores)
    ↓
Advisory interpretation (pure functions)
    ├── cbm_informed_fidelity() → fidelity recommendation
    └── build_cbm_skip_set() → symbol exclusion set
    ↓
Context-selection decisions
    ↓
Context compiler (receives fidelity + skip-set)
    ↓
Compressed context → LLM
```

**Status:** Mechanisms exist but are not wired into production.

---

## 4. Component Responsibility Model

### MCP Request Handling (`handle_provide_code_context`)

| Owns | Does NOT own |
|------|--------------|
| Parameter validation | CBM intelligence decisions |
| File resolution | Fidelity policy |
| Orchestrating compilation | Semantic interpretation |

### Task/Intent Analysis (`compute_strategy()`)

| Owns | Does NOT own |

---

## 5. Complete Data-Flow Diagrams

### Production Context-Compilation Path (CURRENT)

```
LLM/Agent
    ↓
provide_code_context(filePath, intent, fidelity, focusMethods)
    ↓
handle_provide_code_context()
    ↓
compute_strategy() → ContextDecision { fidelity, strategy, cbm_informed: false }
    ↓
compile_file_ir_focused()
    ├── state.get_skip_set(file_path) → None [ALWAYS EMPTY]
    └── compiler.compile_focused(..., skip_set=None, ...)
            ↓
        PassPipeline::default_production() (6 passes)
            ├── Pass 1: CoreIRPass (should_skip_capture with empty set)
            ├── Pass 2: LanguageLayerPass
            ├── Pass 3: MetaLayerPass (produces SemanticEdges)
            ├── Pass 4: PatternRecognitionPass
            ├── Pass 5: AliasResolutionPass
            └── Pass 6: ValidationPass
    ↓
semantic_edges → WorkspaceIndex
    ↓
Render IR → text → LLM
```

### Write Path (apply_edit)

```
apply_edit(filePath, operations)
    ↓
handle_apply_edit()
    ├── Recompile file at Edit fidelity (fresh UnitTable)
    ├── Verify expected-old-text, splice, syntax gate
    ├── Commit to disk
    ├── Post-edit recompile → WorkspaceIndex::add_edges()
    └── bridge.mark_project_dirty(path) → lazy reindex on next query
```

### Intended Compiler-Mediated Path (TARGET)

```
provide_code_context()
    ↓
handle_provide_code_context()
    ↓
compute_strategy()
    ├── File classification (content-based)
    ├── Baseline fidelity selection
    ├── Check: needs CBM? ────────────────────→ No ──→ cbm_informed=false
    │                                          │
    │                                          ▼
    │                                     compile_file_ir_focused()
    │                                          │
    │                                          ▼
    │                                     PassPipeline (6 passes)
    │                                          │
    │                                          ▼
    │                                     Render → LLM
    │
    └── Yes → CBM CONSULTATION
               │
               ▼
         GraphBridge.get_symbol_importance_mut()

---

## 6. Layered Architecture

### Layer 1 — Transport

**Status:** ✅ COMPLETE

Clean-CTX ↔ CBM communication. CBM 0.10.8 daemon-backed architecture, wire formats, retry, timeout.

### Layer 2 — Retrieval

**Status:** ✅ COMPLETE

Obtaining graph/architecture/intelligence data from CBM. 6 whitelisted tools + `cbm_proxy`.

### Layer 3 — Advisory Intelligence

**Status:** ❌ INCOMPLETE

Representing CBM-derived information inside Clean-CTX without making it authoritative. `InferenceLayer` exists but is not wired.

### Layer 4 — Decision Support

**Status:** ❌ INCOMPLETE

Using CBM intelligence to influence fidelity, symbol selection, context expansion, retrieval candidates.

### Layer 5 — Context Compilation

**Status:** ✅ COMPLETE (but no CBM input)

Producing the actual context that the LLM receives. CBM does not currently participate.

### Layer 6 — Agent-Direct Advisory Access

**Status:** ✅ COMPLETE

Allowing the agent to explicitly ask CBM questions and receive results.

---

## 7. Migration History & Impact

### Key Commits

| Commit | Date | Description | Consumption Impact |
|--------|------|-------------|-------------------|
| `edaa359` | June 17, 2026 | Phase 2: intelligence layer + proxy | Created `pagerank.rs`, `fidelity.rs`, `blast_radius.rs` — NOT wired |
| `79abc9c` | June 22, 2026 | Filter-first architecture | Created `CbmFilterState`, `build_cbm_skip_set()`, `IntelligenceConfig` — read path wired, write path NOT wired; REMOVED `enrich_with_cbm()` |
| `565c220` | July 6, 2026 | R-43a/R-43b: Inference Layer | Created `InferenceLayer`, `InferenceLayerPass` — NOT added to `default_production()` |
| `029ec9b` | Sept 4, 2026 | CBM 0.10.8 migration | Pure transport/retrieval — NO consumption changes |

### What the 0.10.8 Migration Changed

| Change | Type | Impact on Consumption |
|--------|------|----------------------|
| `map_search_result()` removed | API removal | None — was for 0.8.1 wire format |
| `parse_architecture_response()` removed | API removal | None — replaced by `wire::parse_architecture` |
| `extract_trace_edges()` removed | API removal | None — replaced by `wire::parse_trace` |
| `get_symbol_importance` Cypher updated | Schema change | None — still returns same type |
| `trace_path()` signature changed | API change | None — added unused `mode` parameter |
| `wire.rs` module added | New module | None — pure adapter |


---

## 9. CBM 0.10.8 Capability Interpretation

### For Initial Implementation

| Capability | Role | Mechanism |
|-----------|------|-----------|
| Importance / degree | Compiler intelligence | Feed fidelity + skip-set |

### For Future Phases

| Capability | Role | Timing |
|-----------|------|--------|
| Data-flow tracing | Context expansion | Future |
| Cross-service tracing | Context expansion | Future |
| Architecture hotspots | Context prioritization | Future |
| Architecture boundaries | Agent-direct advisory | Current |
| Architecture clusters | Architecture advisory | Future |
| Coverage | Context qualification | Future |
| Missed graph | Context qualification | Future |
| `IMPLEMENTS` corroboration | Context validation | Future |
| Cross-repository | Cross-repo expansion | Future |

---

## 10. Request-Scoped Intelligence Ownership

### Problem

CBM intelligence is gathered for a **specific context-compilation request**, but `CbmFilterState` is session-scoped. Skip-sets depend on which file is being compiled and should not leak across requests.

### Solution

**Request-scoped ownership via `ContextDecision`.**


---

## 11. CBM Consultation Timing

### Decision: After task classification, before compilation

```
request
    ↓
task/file classification (cheap, no CBM)
    ↓
CBM consultation (if needed)
    ↓
fidelity + filtering decisions
    ↓
compilation
```

### Consultation Trigger

```rust
let needs_cbm = match (&explicit_fidelity, &explicit_intent) {
    (Some(_), _) => false,  // explicit fidelity, don't consult
    (_, Some(_)) => false,  // explicit intent maps to fidelity
    _ => bridge.is_available() && config.intelligence.enabled,
};
```

### Required Data

| Consumer | Required CBM data | Query method |
|----------|------------------|--------------|
| `cbm_informed_fidelity` | Per-symbol importance | `get_symbol_importance_mut()` |
| `build_cbm_skip_set` | Per-symbol importance | `get_symbol_importance_mut()` (same query) |

### Caching Scope

| Data | Scope | Invalidation |
|------|-------|--------------|
| Importance | Session (GraphBridge) | `mark_project_dirty()` |
| Skip-set | Request | Derived per-request |

---

## 12. Fidelity Pipeline Design

### The Pipeline

```rust
// 1. Baseline fidelity (existing)
let baseline = resolve_fidelity(explicit_fidelity, explicit_intent, ...)?;

// 2. CBM consultation (new)
let (final_fidelity, cbm_informed) = if needs_cbm {
    let importance = bridge.get_symbol_importance_mut()?;
    let recommendation = cbm_informed_fidelity(file_path, &importance, baseline);
    (apply_recommendation(recommendation, baseline), true)
} else {
    (baseline, false)
};
```

### Override Rules

| Scenario | CBM can override? |
|----------|------------------|
| Explicit `fidelity` arg | **No** — explicit always wins |
| Explicit `intent` arg | **No** — explicit always wins |
| Auto fidelity (file classification) | **Yes** |
| CBM ForceHigh | Upgrade to High |
| CBM ForceLow | Downgrade to Low |
| CBM NoRecommendation | No change |

---

## 14. Failure Semantics

### The Invariant

> **CBM is opportunistic. CBM unavailability must not make ordinary context compilation fail.**

### Failure Matrix

| Failure | Behavior | `cbm_informed` | Log level |
|---------|----------|----------------|-----------|
| CBM not installed | Proceed without enhancement | `false` | Debug |
| CBM startup timeout | Proceed without enhancement | `false` | Warn |
| Daemon conflict | Proceed without enhancement | `false` | Error |
| Query timeout | Proceed without enhancement | `false` | Warn |
| Malformed response | Proceed without enhancement | `false` | Error |
| Empty importance map | Proceed (NoRecommendation) | `true` | Debug |
| Partial importance | Proceed with available data | `true` | Debug |
| Stale graph | Proceed (lazy reindex on next query) | `true` | Debug |

---

## 15. Freshness & Invalidation

### Current Mechanism (Sufficient)

```
apply_edit()
    ↓
mark_project_dirty(path)
    ↓
ProjectFreshness { dirty_generation > indexed_generation }
    ↓
Next graph query → ensure_indexed() → synchronous reindex
    ↓
Fresh importance data
```

### Invalidation Triggers

| Event | Invalidation |
|-------|-------------|
| `apply_edit()` | `mark_project_dirty()` → lazy reindex |

---

## 18. Implementation-Readiness Matrix

| Mechanism | Existing | Correct | Needs wiring | Needs redesign | Future |
|-----------|----------|---------|--------------|----------------|--------|
| GraphBridge structured access | ✅ | ✅ | No | No | — |
| `cbm_informed_fidelity()` | ✅ | ✅ | Yes (caller) | No | — |
| `build_cbm_skip_set()` | ✅ | ✅ | Yes (caller) | No | — |
| `CbmFilterState` | ✅ | ❌ (scope) | No | Yes (scope) | — |
| `InferenceLayer` | ✅ | ✅ | No | No | Yes (consumer) |
| `InferenceLayerPass` | ✅ | ❌ (placement) | No | Yes (placement) | — |
| `pagerank` | ✅ | ❌ (blend model) | No | Yes | Yes |
| `blast_radius` | ✅ | ✅ | No | No | Yes |
| Data flow | ❌ | — | — | — | Yes |
| Cross-service | ❌ | — | — | — | Yes |
| Coverage | ❌ | — | — | — | Yes |
| Architecture intelligence | ❌ | — | — | — | Yes |
| `IntelligenceConfig` | ✅ | ✅ | Yes (checked) | No | — |
| `ContextDecision.cbm_informed` | ✅ | ✅ | Yes (set true) | No | — |
| Request-scoped CbmIntelligence | ❌ | — | — | — | — |

---

## 19. Responsibility Matrix

| Component | Owns | Does not own | Scope |
|-----------|------|--------------|-------|
| MCP handler | Request orchestration, file resolution | CBM decisions, fidelity policy | Request |

---

## 21. Smallest Correct Implementation Sequence

### Phase A — Wire Fidelity + Filtering

**Changes:**
1. Add `CbmIntelligence` struct
2. Extend `ContextDecision` with `cbm_intelligence: Option<CbmIntelligence>`
3. Wire `cbm_informed_fidelity()` into `compute_strategy()`
4. Wire `build_cbm_skip_set()` into `compile_file_ir_focused()`
5. Add tests

### Phase B — Coverage Lifecycle (Independent)

### Phase C — Cleanup (Future)

### Phase D — Advanced Intelligence (Future)

---

## 22. Explicit Non-Goals

1. Context expansion (data-flow, cross-service, architecture hotspots)
2. InferenceLayer production consumer
3. Pagerank 60/40 blend (obsolete)
4. Blast radius integration
5. Coverage qualification
6. Cross-repository intelligence
7. Replacing Cypher with native degree filters
8. Changing `cbm_proxy`
9. Modifying `WorkspaceIndex`
10. Modifying `EntityRef` or semantic identity

---

## 23. Boundary Constraints

### Non-Negotiable (Do NOT Modify)

| Constraint | Rationale |
|-----------|-----------|
| `EntityRef` equality/hash | Semantic identity model |
| Semantic identity `(domain, entity_type, name)` | Core architectural invariant |
| `WorkspaceIndex` identity semantics | Authoritative semantic substrate |
| CBM's external advisory role | |
| `InferenceLayer` separation from `CompiledIR` | Type-system enforced boundary |
| Graceful degradation without CBM | |

### Preserve These Statements

> **Clean-CTX semantic facts are authoritative.**

> **CBM intelligence is advisory.**

> **`cbm_proxy` is an agent-facing rendering boundary.**

> **Internal compiler-mediated CBM consumption must use structured data, not proxy-compressed text.**

---

## 24. Open Questions

1. **Should skip-sets be file-scoped or task-scoped?** — Current: File-scoped
2. **Should the importance query use Cypher or native CBM filters?** — Current: Cypher
3. **Should `CbmFilterState` be deprecated immediately?** — Current: Transitional
4. **What is the production consumer for `InferenceLayer`?** — Future: Context expansion

---

## Appendix A: File Reference

| File | Role | Status |
|------|------|--------|
| `src/cbm/bridge.rs` | CBM transport + caching | ✅ 0.10.8 migrated |
| `src/cbm/proxy.rs` | Agent-facing proxy | ✅ Functional |
| `src/intelligence/fidelity.rs` | Fidelity + skip-set functions | ⚠️ Not wired |
| `src/ir/inference_layer.rs` | Advisory intelligence structure | ⚠️ Not wired |
| `src/mcp/heuristics.rs` | Fidelity selection | ⚠️ CBM consultation missing |
| `src/mcp/state.rs` | Session state | ⚠️ CbmFilterState scope wrong |

## Appendix B: Glossary

| Term | Definition |
|------|-----------|
| **Flow A** | Agent-direct CBM access via `cbm_proxy` |
| **Flow B** | Compiler-mediated CBM consumption |
| **CbmIntelligence** | Request-scoped advisory CBM data |
| **Importance** | CBM in-degree centrality score |
| **Skip-set** | Symbols to exclude from compression |
| **cbm_informed** | Flag: CBM was consulted for this request |

---

*This is a living document. Update as implementation progresses.*
| ContextDecision | Compilation parameters, CBM intelligence | CBM transport, semantic interpretation | Request |
| GraphBridge | CBM transport, wire parsing, caching, domain conversion | Fidelity policy, skip-set policy, context selection | Session |
| CbmIntelligence | Structured advisory data for this request | CBM transport, compilation | Request |
| InferenceLayer | Advisory edges with confidence scores | Serialization, authoritative status | Request (future) |
| CbmFilterState | (Legacy) per-file skip sets | Request-scoped intelligence | Session (transitional) |
| WorkspaceIndex | Authoritative semantic facts | CBM-derived data, advisory edges | Session |
| Context compiler | Source → IR, skip-set application | CBM consultation, fidelity decisions | Request |
| cbm_proxy | Agent-facing CBM forwarding, compression | Internal structured consumption | Request |
| `compute_strategy()` | Fidelity selection, CBM consultation trigger | CBM transport, wire parsing | Request |
| `cbm_informed_fidelity()` | Importance → fidelity recommendation | CBM transport, importance calculation | Pure function |
| `build_cbm_skip_set()` | Importance → symbol exclusion set | CBM transport, importance calculation | Pure function |

---

## 20. Migration Gap Matrix

| Capability | Pre-0.10.8 | Current 0.10.8 | Intended | Migration Impact | Correct Action |
|-----------|-----------|---------------|----------|-----------------|----------------|
| CBM transport | 0.8.1 | 0.10.8 | 0.10.8 | ✅ Updated | None |
| Graph retrieval | 0.8.1 envelopes | 0.10.8 tree/table | 0.10.8 | ✅ Updated | None |
| Importance | Broken (no Function label) | Works (Cypher) | Works | ✅ Fixed | Wire consumer |
| Fidelity | Unwired | Unwired | Wired | ❌ Never wired | Wire `cbm_informed_fidelity()` |
| Skip-set | Read wired, write unwired | Read wired, write unwired | Fully wired | ❌ Never wired | Wire `build_cbm_skip_set()` |
| Inference | Optional pass | Optional pass | In default production | ❌ Never added | Add to default |
| Data flow | N/A | Available | Context expansion | ✅ New | Add when consumption wired |
| Cross-service | N/A | Available | Context expansion | ✅ New | Add when consumption wired |
| Architecture | Partial | Full | Agent-direct + advisory | ✅ Updated | None |
| Coverage | Simple | Rich | Context qualification | ✅ Enhanced | Wire when ready |
| Agent-direct access | Full | Full | Full | ✅ Preserved | None |
| External edit (git checkout, shell) | `index_repository()` call → reindex |
| CBM daemon restart | Automatic (new daemon = fresh index) |

---

## 16. GraphBridge Responsibility Boundary

### GraphBridge OWNS

- CBM process management (spawn, kill, restart)
- CBM transport (JSON-RPC over stdio)
- CBM retry/timeout (circuit breaker, backoff)
- CBM wire parsing (delegates to `wire.rs`)
- CBM result caching (per-project, invalidated on dirty)
- Clean-CTX domain conversion (CBM types → `SymbolImportance`, `GraphEdge`, etc.)

### GraphBridge DOES NOT OWN

- Fidelity policy
- Skip-set policy
- Context selection
- Context expansion
- Compression policy
- Semantic interpretation

### The Line

> GraphBridge answers: "What does CBM know about this project?"  
> It does NOT answer: "What should Clean-CTX do with that knowledge?"

---

## 17. Structured vs. Proxy-Rendered Data

### Agent-Direct (Flow A): Proxy-Rendered

```
CBM → raw response → cbm_proxy compression → LLM
```

The proxy's compression is appropriate — the agent receives compressed text.

### Compiler-Mediated (Flow B): Structured

```
CBM → raw response → GraphBridge → structured data → policy functions
```

Compression would DESTROY the structured data needed for decisions.

### Boundary

```
CBM subprocess
    ↓ (raw JSON)
wire.rs (CBM schema knowledge)
    ↓ (CBM domain types)
GraphBridge (caching, transport)
    ↓ (Clean-CTX domain types: SymbolImportance, GraphEdge, etc.)
cbm_informed_fidelity / build_cbm_skip_set (Clean-CTX policy)
```

### What `cbm_informed` Means

`cbm_informed = true` means "CBM intelligence was available and consulted." It does NOT mean "CBM changed the fidelity."

---

## 13. Skip-Set Lifecycle Design

### The Lifecycle

```
1. TRIGGER: compile_file_ir_focused() called for file X
    ↓
2. CONSULT: If CBM available and importance not cached:
    ↓
    bridge.get_symbol_importance_mut() → importance map
    ↓
3. DERIVE: Filter importance map to file X's symbols
    ↓
    build_cbm_skip_set(file_path, &importance) → skip_set
    ↓
4. STORE: Request-scoped (in ContextDecision.cbm_intelligence)
    ↓
5. APPLY: PassContext.skip_set = skip_set
    ↓
    CoreIRPass.should_skip_capture() drops low-importance symbols
    ↓
6. EXPIRE: Request ends, skip_set discarded
```

### CBM Importance Semantics

In CBM 0.10.8, importance = in-degree (number of incoming CALLS edges). This is **graph centrality** — hub functions with many callers.

| In-degree | Meaning | Fidelity implication |
|-----------|---------|---------------------|
| High (≥10) | Core API, shared utility | High fidelity needed |
| Medium (1-9) | Standard function | Default fidelity |
| Low (0) | Dead code or entry point | Skip or low fidelity |

**Verdict:** The semantic meaning justifies the filtering policy.
```rust
pub struct ContextDecision {
    pub fidelity: Fidelity,
    pub strategy: ContextStrategy,
    pub is_angular: bool,
    pub source_line_count: usize,
    pub file_class: FileClass,
    pub cbm_informed: bool,
    // NEW: request-scoped CBM intelligence
    pub cbm_intelligence: Option<CbmIntelligence>,
}

pub struct CbmIntelligence {
    pub importance: HashMap<String, SymbolImportance>,
    pub skip_set: HashSet<String>,
    // Future: data_flow, cross_service, coverage
}
```

### Ownership Chain

```
GraphBridge (cache raw importance, session scope)
    ↓ returns
CbmIntelligence (request scope, owned by ContextDecision)
    ↓ feeds
compile_file_ir_focused (reads skip_set)
```

### Why Not CbmFilterState?

`CbmFilterState` is session-scoped and shared across all files/requests. This is wrong because importance is file-specific and skip-sets are file-specific. Transitional approach: keep `CbmFilterState` for backward compatibility but use request-scoped `CbmIntelligence` for new wiring.
**Conclusion:** The 0.10.8 migration made ZERO changes to consumption paths. All consumption gaps predate the migration.

### What Was Intentionally Removed

The `enrich_with_cbm()` and `enrich_workspace_with_cbm()` functions were **intentionally removed** in commit `79abc9c`:

> "BREAKING CHANGE: CBM integration moves from post-compression enrichment to pre-compression filtering. The `enrich_with_cbm()` and `enrich_workspace_with_cbm()` functions are removed. CBM now reduces token output instead of increasing it."

This was an architectural decision to replace post-compression enrichment with pre-compression filtering. The replacement (filter-first) was designed but never wired for production population.

---

## 8. Mechanism Assessment

### Directly Wireable

| Mechanism | Location | What's Needed |
|-----------|----------|---------------|
| `cbm_informed_fidelity()` | `src/intelligence/fidelity.rs:41` | Caller in `compute_strategy()` |
| `build_cbm_skip_set()` | `src/intelligence/fidelity.rs:93` | Caller before compilation |
| `IntelligenceConfig.enabled` | `src/config.rs:478` | Check in `compute_strategy()` |
| `InferenceLayer` | `src/ir/inference_layer.rs` | Production consumer (future) |

### Requires Targeted Adaptation

| Mechanism | Issue | Required Adaptation |
|-----------|-------|---------------------|
| `InferenceLayerPass` | Should NOT be a compiler pass | Move to request-level orchestration |
| `CbmFilterState` | Session scope is wrong | Supplement with request-scoped state |
| `pagerank.rs` | 60/40 blend obsolete | Future redesign |
| `get_symbol_importance_mut()` | Misleading name | Future rename |

### Not Yet Needed

| Mechanism | Phase |
|-----------|-------|
| `InferenceLayer` production consumer | Future (context expansion) |
| Data-flow tracing | Future |
| Cross-service tracing | Future |
| Coverage qualification | Future |

### Old Intelligence Module Assessment

| Module | Verdict | Action |
|--------|---------|--------|
| `pagerank.rs` | Retain but redesign | 60/40 blend obsolete; future CBM importance ranking |
| `blast_radius.rs` | Future capability | Retain; integrate with context expansion later |
| `fidelity.rs` | Retain and wire | `cbm_informed_fidelity()` + `build_cbm_skip_set()` ready |
               │
               ▼
         ┌─────┴──────┐
         │            │
         ▼            ▼
    cbm_informed_    build_cbm_
    fidelity()       skip_set()
         │            │
         ▼            ▼
    ContextDecision   Request-scoped
    .fidelity         skip_set
    .cbm_informed     │
         │            │
         └─────┬──────┘
               │
               ▼
         compile_file_ir_focused()
               │
               ▼
         PassContext.skip_set = skip_set
               │
               ▼
         CoreIRPass.should_skip_capture()
               │
               ▼
         CompiledIR (filtered)
               │
               ▼
         Render → LLM
```
|------|--------------|
| File classification | CBM transport |
| Baseline fidelity selection | CBM response parsing |
| Deciding WHEN CBM is needed | |

### CBM Consultation

| Owns | Does NOT own |
|------|--------------|
| Deciding WHICH CBM data to request | How data influences fidelity |
| Caching scope | |
| Failure handling | |

### GraphBridge

| Owns | Does NOT own |
|------|--------------|
| CBM transport | Fidelity policy |
| Wire parsing | Skip-set policy |
| Caching raw CBM results | Context-selection policy |
| Returning Clean-CTX domain types | |

### InferenceLayer

| Owns | Does NOT own |
|------|--------------|
| Holding advisory CBM edges with confidence | Being serialized |
| Provenance tracking (`InferenceSource::Cbm`) | Being authoritative |
| | Entering WorkspaceIndex |

### ContextDecision

| Owns | Does NOT own |
|------|--------------|
| Final compilation parameters | CBM transport |
| CBM intelligence for this request | CBM parsing |

### Context Compiler (`PassPipeline`)

| Owns | Does NOT own |
|------|--------------|
| Source → IR compilation | CBM consultation |
| Applying skip-sets | Fidelity decisions |

### WorkspaceIndex

| Owns | Does NOT own |
|------|--------------|
| Authoritative semantic facts (confidence 1.0) | CBM-derived data |
| | Advisory edges |

### cbm_proxy

| Owns | Does NOT own |
|------|--------------|
| Agent-facing CBM forwarding | Internal structured consumption |
| Compression for token efficiency | Compiler-mediated flows |
| Internal consumption uses **structured data**, not proxy-compressed text | Data integrity |
| CBM unavailability must **not** make compilation fail | Graceful degradation |
| Explicit user parameters **win** over CBM recommendations | User authority |

### Design Principles

1. **Request-scoped intelligence:** CBM data belongs to a specific compilation request
2. **Separation of transport and policy:** GraphBridge transports; policy functions decide
3. **Filter-first:** Low-importance symbols excluded before fidelity-based compression
4. **Pure policy functions:** `cbm_informed_fidelity()`, `build_cbm_skip_set()` are pure
5. **Compiler receives parameters:** The compiler doesn't consult CBM; it receives fidelity + skip-set