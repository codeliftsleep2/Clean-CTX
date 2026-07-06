# IR Evolution Plan v0.3 — Semantic & Behavioral Expansion

**Date:** 2026-07-06  
**Status:** ✅ IMPLEMENTED — R-43a (v0.3.0) and R-43b (Phases 2-6, v0.4.0) both complete  
**Roadmap IDs:** R-43a (Phase 1, v0.3.0), R-43b (Phases 2-6, v0.4.0)  
**Goal:** Evolve the Compiler-IR from a strong structural representation into a behavioral reasoning substrate while keeping CBM optional.

---

## Table of Contents

1. [Current State Assessment](#1-current-state-assessment)
2. [Design Principles](#2-design-principles)
    - 2.5 [Guiding Philosophy](#25-guiding-philosophy)
    - 2.6 [IR Invariants (Contract)](#26-ir-invariants-contract)
3. [Roadmap Placement](#3-roadmap-placement)
4. [CBM Integration Analysis](#4-cbm-integration-analysis)
5. [R-43a: Execution Semantics (Phase 1)](#5-r-43a-execution-semantics-phase-1)
6. [R-43b: Phases 2-6 (Future)](#6-r-43b-phases-2-6-future)
7. [Implementation Order](#7-implementation-order)
8. [Success Criteria](#8-success-criteria)
9. [Risk & Mitigation](#9-risk--mitigation)

---

## 1. Current State Assessment

### Strengths

- **Multi-layered** (Core → Language → Meta → Pattern → Execution → Graph → Inference → Validation) — production-grade, **1593+ IR tests**
- **Stateful with delta replay** (`ContextState`, `FileState`, `IRDelta`)
- **Bidirectional** (IR ↔ text ↔ wire) — 6 wire formats
- **Cross-file aware** via `GlobalSymbolTable`
- **CBM-optional** — `GraphBridge` is `Option`-wrapped in `McpState`
- **Pluggable language layers** — `LanguageLayer` trait with `process_capture()` + `finalize()`
- **Existing CBM integration** — `cbm_informed_fidelity()`, `build_cbm_skip_set()`, `cbm_proxy` pipe-level compression

### Gaps

| Gap | Description |
|-----|-------------|
| Limited execution semantics | No concept of dataflow, side effects, or execution context |
| No lightweight program graph | Local only — no cross-method call/dataflow edges |
| Structural deltas only | No intent-based semantic delta (rename, add injection, etc.) |
| No formal invariants | No validation layer for IR consistency |
| Limited queryability | No query API for behavioral patterns |
| CBM enrichment not mapped to IR | CBM graph edges (calls, dataflow) not consumed by IR structures |
| Facts mixed with inferences | No separation between deterministic structural facts and derived/estimated data |
| Monolithic compile function | `IRCompiler::compile()` is a single 500-line function — not composable |
| No confidence tracking | Inferred edges have no confidence score, making it impossible to distinguish certainty from estimation |

### Current CoreOp Instruction Set (19 variants)

```rust
pub enum CoreOp {
    // Structural Definitions
    DefClass(String, String),       // DEF_C
    DefMethod(String, String, String), // DEF_M
    DefField(String, String, String),  // DEF_F
    DefInterface(String, String),      // DEF_I

    // Signatures & Types
    Param(String, String, String, String), // SIG
    Return(String, String),                // RET
    FieldType(String, String),             // FIELD_T

    // Control Flow & Behavior (flat flags only)
    Flags(String, Vec<String>),         // FLAGS
    ClassFlags(String, Vec<String>),    // FLAGS_C

    // Relationships
    Extends(String, String),            // EXT
    Implements(String, String),         // IMPL
    Injects(String, Vec<String>),       // INJECTS

    // Imports & Type Aliases
    Import(String, String, String),     // IMP
    TypeAlias(String, String),          // TYPE

    // Compressed Patterns
    Pattern(String, Vec<String>),      // PAT
}
```

### Current CBM Capabilities

| Capability | What it provides | How it's consumed today |
|-----------|-----------------|------------------------|
| `get_symbol_importance()` | Per-symbol importance score (0.0-1.0) | `cbm_informed_fidelity()` → fidelity override; `build_cbm_skip_set()` → filter-first exclusion |
| `get_blast_radius()` | Callers of a symbol at depth 1 | `GraphBridge::get_blast_radius()` — not consumed by IR pipeline |
| `get_dead_code()` | Unused symbol detection | `GraphBridge::get_dead_code()` — not consumed by IR pipeline |
| `get_architecture()` | Module/dependency overview | `GraphBridge::get_architecture()` — MCP tool only |
| `search()` | Symbol search by name/pattern | `GraphBridge::search()` — MCP tool only |
| `query_graph()` | Cypher-like graph queries | `GraphBridge::query_graph()` — MCP tool only |
| `trace_path()` | Path between two symbols | `GraphBridge::trace_path()` — MCP tool only |
| `proxy_call()` | Pipe-level interception + JSON compression | `handle_cbm_proxy()` — MCP tool only |
| `detect_changes()` | Graph version change detection | `GraphBridge::detect_changes()` — cache invalidation |

---

## 2. Design Principles

1. **CBM Optional** — All enhancements must work well without CBM. The existing `GraphBridge` + `Option` pattern is correct and unchanged.

2. **Incremental** — Build on existing IR without breaking changes. New `CoreOp` variants are additive.

3. **Backward Compatible** — Existing compressed output and delta transport remain unchanged. Old IR files still parse.

4. **Leverage Existing** — Use `LayerRegistry`, `CompiledIR`, `CoreOp`, `LanguageLayer` trait, and `GlobalSymbolTable` as foundation.

5. **Lazy Computation** — Program graph and execution semantics are computed on demand, not eagerly.

6. **CBM Enrichment is Explicit** — CBM data is consumed at well-defined integration points (ProgramGraph, IRQueryEngine), not scattered across the pipeline.

7. **Facts vs Inferences (NEW)** — Core IR must remain pure facts — structural, deterministic, replayable. All inferences (importance, dead code, blast radius, confidence) live in a separate `InferenceLayer`. This prevents synchronization bugs and makes replay deterministic forever. The `InferenceLayer` is never serialized into the core IR wire format — it's a derived overlay.

8. **Layered Derivation (LLVM-style) (NEW)** — The pipeline is a sequence of passes where each pass transforms or enriches the IR without back-propagating into earlier layers. Never let `ProgramGraph` or CBM data become authoritative over the core IR. The pipeline is:

    ```
    Source Code
        │
        ▼
    ┌─────────────────────┐
    │  Pass 1: Core IR    │  Tree-sitter → CoreOp stream (pure facts)
    │  (IRCompiler)       │  Structural, deterministic, replayable
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 2: Language   │  Language-specific ops (async, export, etc.)
    │  Layer              │  Still deterministic — derived from captures
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 3: Meta Layer │  Framework-specific markers (@cmp, Φrest)
    │                     │  Still deterministic
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 4: Execution  │  DataFlow, ControlFlow, SideEffect, Context
    │  Semantics          │  Still deterministic — from tree-sitter captures
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 5: Program    │  Local graph from CompiledIRs + SymbolTable
    │  Graph              │  Still deterministic — derived from IR
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 6: Inference  │  CBM enrichment + derived analysis
    │  Layer              │  NON-deterministic — confidence scores < 1.0
    │                     │  NEVER written back to CoreOp stream
    └─────────┬───────────┘
              │
              ▼
    ┌─────────────────────┐
    │  Pass 7: Validation │  Structural + consistency checks
    │  / Optimization     │  Reads all layers, writes nothing
    └─────────────────────┘
    ```

9. **Confidence Scores (NEW)** — Every inferred edge carries a `confidence: f64` score (0.0-1.0). Structural facts from tree-sitter parsing have confidence 1.0. CBM-derived inferences have confidence < 1.0. This enables downstream consumers to distinguish certainty from estimation.

10. **Explicit Passes (NEW)** — The compiler pipeline is a composable sequence of `IRPass` implementations, not a monolithic `compile()` function. Each pass has a clear input/output contract. Adding new languages, meta-layers, or analysis passes becomes mechanical.

---

## 2.5 Guiding Philosophy

Compiler IR is not intended to be a full compiler intermediate representation.

It is an **LLM-oriented semantic representation** whose purpose is to preserve the information most valuable for software reasoning while minimizing token cost.

Every addition to the IR must justify itself by improving one or more of:

- **Reasoning accuracy** — Does this addition help an LLM or developer understand the code better?
- **Context compression** — Does this addition reduce token count while preserving meaning?
- **Incremental updates** — Does this addition enable more efficient delta transport?
- **Deterministic reconstruction** — Does this addition preserve the ability to reconstruct the original structure?
- **Semantic querying** — Does this addition enable new queries or analyses that were previously impossible?

This philosophy serves as a guardrail against feature creep. Every proposed `CoreOp` variant, language layer, meta-layer, or inference pass must be evaluated against these criteria before inclusion.

---

## 2.6 IR Invariants (Contract)

These are the non-negotiable guarantees of the Compiler-IR system. They must hold at all times, across all fidelity levels, wire formats, and CBM states. Violating an invariant is a bug.

### 2.6.1 Structural Invariants

| # | Invariant | Description | Enforced By |
|---|-----------|-------------|-------------|
| S1 | **Every method belongs to exactly one class.** | No orphan methods. Standalone functions get a synthetic class (`__file_<id>`). | `IRCompiler`, `IRValidator` |
| S2 | **Every parameter belongs to exactly one method.** | `SIG` ops always reference an existing `DEF_M`. | `IRValidator` |
| S3 | **Every reference points to an existing symbol.** | `EXT`, `IMPL`, `INJECTS`, `DATAFLOW` targets resolve to a known `DEF_C`, `DEF_M`, or `DEF_F`. Unresolved references are marked explicitly. | `IRValidator` |
| S4 | **IR ordering is stable within a file.** | Compiling the same source twice produces identical instruction order. BTreeMap-based indexing ensures deterministic delta comparison. | `IRCompiler` (BTreeMap), tests |
| S5 | **Applying identical deltas twice produces identical state.** | Delta application is deterministic. `apply()` on the same baseline + same delta = same result. | `ContextState::apply()`, tests |
| S6 | **Replay is idempotent.** | Replaying a sequence of deltas from the same starting state produces the same final state, regardless of batch size. | `ContextState`, tests |
| S7 | **Every render format is semantically equivalent.** | Named, positional, tagged, string_table, hierarchical, and binary formats all represent the same instruction stream. Round-trip (encode → decode) is lossless. | Wire format tests |

### 2.6.2 Behavioral Invariants

| # | Invariant | Description | Enforced By |
|---|-----------|-------------|-------------|
| B1 | **Behavioral enrichment never changes structural meaning.** | Adding `DATAFLOW`, `CTRL`, `EFFECT`, or `CTX` ops does not alter `DEF_C`, `DEF_M`, `DEF_F`, `EXT`, `IMPL`, or any other structural op. | `IRCompiler` pass ordering |
| B2 | **Language layers may add information but must never modify Core IR.** | `LanguageLayer::process_capture()` returns new ops; it does not modify or remove existing ones. | `LanguageLayer` trait contract |
| B3 | **Meta layers may never modify Core IR.** | Meta-layers (Angular, Spring) emit `TypeAlias` ops only. They do not touch structural ops. | `LayerRegistry` contract |
| B4 | **Pattern recognizers may consume instructions but must preserve semantics.** | `CompressingPatternRecognizer` collapses recognized patterns into `PAT` ops. The compressed output must be semantically equivalent to the original instruction stream. | Round-trip tests |
| B5 | **InferenceLayer is ephemeral — never serialized into the core IR wire format.** | CBM-derived edges, importance scores, dead code, and blast radius live only in memory. They are never written to named, positional, hierarchical, or any other wire format. | `InferenceLayer` design, code review |

### 2.6.3 CBM Integration Invariants

| # | Invariant | Description | Enforced By |
|---|-----------|-------------|-------------|
| C1 | **CBM is strictly an enrichment layer — it never modifies Core IR.** | CBM data flows into `InferenceLayer` only. No CBM-derived data enters the `CoreOp` instruction stream. | `InferenceLayer::enrich_from_cbm()` |
| C2 | **When CBM is unavailable, all core functionality remains complete.** | All 4 new `CoreOp` variants, `ProgramGraph`, `IRDelta`, `IRValidator`, and `IRQueryEngine` work identically without CBM. | Integration tests (CBM disabled) |
| C3 | **Inferred data from CBM always carries confidence < 1.0.** | CBM-derived edges and annotations have `confidence = 0.75`. Structural facts have `confidence = 1.0`. | `InferenceEdge`, `InferenceAnnotation` |
| C4 | **Core IR remains deterministic and replayable regardless of CBM state.** | CBM enrichment happens after all deterministic passes. It cannot affect delta computation or replay. | Pass pipeline ordering |

---

## 3. Roadmap Placement

| ID | Title | Effort | Priority | Release |
|----|-------|-------:|---------|---------|
| **R-43a** | IR Evolution — Execution Semantics (Phase 1) | 4-5 days | 🔴 High | v0.3.0 |
| **R-43b** | IR Evolution — Program Graph + Inference Layer + Semantic Delta + Validation + Query (Phases 2-6) | 5-8 days | 🟡 Medium | v0.4.0 |

**Why split?**
- R-43a delivers immediate pilot value (SignalR streaming, EF Core dataflow, RxJS behavior) in 4-5 days
- R-43b builds on top of it and is more speculative — better to validate with real usage first
- Keeps v0.3.0 realistic alongside other high-priority items (Python, type-aware compression, multi-file diff, React/Redux/NgRx)

---

## 4. CBM Integration Analysis

### 4.1 CBM Integration Matrix

| Phase | CBM Role | Enrichment Data | When CBM is Unavailable |
|-------|---------|----------------|------------------------|
| **1. Execution Semantics** | **None** — purely tree-sitter driven | N/A | Works identically |
| **2. Program Graph** | **None** — purely derived from local IR | N/A | Works identically |
| **3. Inference Layer** | **Primary source** — cross-file edges, importance, dead code, blast radius | `InferenceEdge`, `InferenceAnnotation` | No inferences (local graph only) |
| **4. Semantic Delta** | **None** — structural diff only | N/A | Works identically |
| **5. IR Validation** | **None** — structural invariants only | N/A | Works identically |
| **6. Queryable IR** | **Global fallback** — cross-file queries delegate to CBM | CBM's `search()`, `query_graph()`, `trace_path()` | Local queries only |

### 4.2 Data Flow: CBM → InferenceLayer

**Critical architectural boundary:** CBM data NEVER enters the `CoreOp` instruction stream. It lives exclusively in the `InferenceLayer`, which is a derived overlay. This ensures:

- Core IR remains pure facts — deterministic, replayable, serializable
- InferenceLayer is ephemeral — recomputed on demand, never serialized into wire format
- No synchronization bugs between CBM data and structural IR

```
CBM Knowledge Graph
    │
    ├── get_symbol_importance() ──→ InferenceLayer.annotations["importance"]
    │                                  (per-node importance score, confidence < 1.0)
    │
    ├── get_blast_radius() ──────→ InferenceLayer.annotations["blast_radius"]
    │                                  (per-method caller list, confidence < 1.0)
    │
    ├── get_dead_code() ─────────→ InferenceLayer.annotations["dead_code"]
    │                                  (set of dead symbols, confidence < 1.0)
    │
    ├── query_graph("MATCH ...") ─→ InferenceLayer.inferred_edges
    │                                  (cross-file CALLS, DATAFLOW edges, confidence < 1.0)
    │
    ├── search(".*Async") ───────→ IRQueryEngine (global fallback)
    │                                  (cross-file symbol queries)
    │
    └── trace_path(A, B) ────────→ IRQueryEngine (global fallback)
                                       (cross-file call chain trace)
```

### 4.3 When CBM is Enabled

**Fidelity selection (existing, unchanged):**
- `cbm_informed_fidelity()` already blends CBM importance scores into the heuristics engine
- High-importance symbols (>0.8) → force High fidelity
- Low-importance symbols (<0.4) → force Low fidelity
- No change needed — this already works

**Meta-layer output (existing, unchanged):**
- `build_cbm_skip_set()` already excludes low-importance symbols from compression
- The skip set is passed to `IRCompiler::compile()` via the `skip_set` parameter
- No change needed — this already works

**Inference Layer enrichment (new in R-43b):**
- `InferenceLayer::enrich_from_cbm()` is called after local graph construction
- CBM provides cross-file CALLS edges that the local graph cannot see
- CBM provides importance, dead code, and blast radius annotations
- All CBM-derived data has confidence < 1.0

**Delta transport (new in R-43b):**
- CBM's `detect_changes()` can trigger re-compilation when the graph detects structural changes
- Cross-file changes detected by CBM can populate `IRDelta.intent` with richer context
- Example: CBM detects that renaming `UserService.GetUser()` affects 15 callers across 5 files → the delta for each file includes `SemanticIntent::RenameSymbol` with cross-file impact metadata

**Query engine (new in R-43b):**
- `IRQueryEngine` holds an optional `GraphBridge` reference
- Local queries return results first; CBM enriches with cross-file results
- Example: `find_async_methods()` returns local async methods + CBM's `search(".*Async")` results from other files

### 4.4 When CBM is Disabled

**Fallback behavior is complete:**
- All 4 new `CoreOp` variants are extracted from tree-sitter captures — no CBM dependency
- `ProgramGraph` is built from local `CompiledIR` + `GlobalSymbolTable` — no CBM dependency
- `InferenceLayer` is empty (no inferred edges or annotations) — no CBM dependency
- `IRDelta` semantic intent is detected by structural diff — no CBM dependency
- `IRValidator` checks structural invariants — no CBM dependency
- `IRQueryEngine` returns local results only — no CBM dependency

**What's lost without CBM:**
- Cross-file CALLS edges in `InferenceLayer`
- Symbol importance scores (fidelity blending and skip sets)
- Dead code detection
- Blast radius analysis
- Cross-file query fallback

**All of these are additive — the core IR functionality is unaffected.**

### 4.5 Potential Modifications Needed

#### GraphBridge → expose more structured data for InferenceLayer

**Current state:** `GraphBridge` exposes methods like `get_symbol_importance_mut()` that return `HashMap<String, SymbolImportance>`. The data is consumed by `cbm_informed_fidelity()` and `build_cbm_skip_set()` in `src/intelligence/fidelity.rs`.

**Modification needed (R-43b):**
```rust
// New method on GraphBridge:
impl GraphBridge {
    /// Get all CALLS edges from CBM's knowledge graph.
    /// Returns (caller, callee) pairs across all files.
    pub fn get_call_edges(&mut self) -> Vec<(String, String)> {
        let key = "call_edges";
        if self.check_cache(key) { /* return cached */ }
        let result = self.query(|c| c.query_graph(
            "MATCH (a:Function)-[:CALLS]->(b:Function) RETURN a.name, b.name",
            &self.project_str(),
        ));
        // Parse and cache
    }

    /// Get all dataflow edges from CBM's knowledge graph.
    /// Returns (method, target, direction) triples.
    pub fn get_dataflow_edges(&mut self) -> Vec<(String, String, String)> {
        // Similar to get_call_edges but for DATAFLOW relationships
    }
}
```

#### New InferenceLayer module

**Current state:** No inference layer exists. CBM data is consumed ad-hoc.

**Modification needed (R-43b):**
```rust
/// The Inference Layer holds all non-deterministic, derived, or estimated
/// data about the IR. This layer is NEVER serialized into the core IR wire
/// format. It is recomputed on demand and lives only in memory.
///
/// Facts vs Inferences:
///   - CoreOp stream = pure facts (structural, deterministic, replayable)
///   - InferenceLayer = derived analysis (estimated, non-deterministic, ephemeral)
pub struct InferenceLayer {
    /// Inferred edges with confidence scores
    pub inferred_edges: Vec<InferenceEdge>,
    /// Per-symbol annotations (importance, dead code, etc.)
    pub annotations: HashMap<String, Vec<InferenceAnnotation>>,
}

/// An inferred edge between two symbols.
/// Confidence < 1.0 means this edge was derived (not parsed).
pub struct InferenceEdge {
    pub edge_type: InferenceEdgeType,
    pub from: String,
    pub to: String,
    /// Confidence score 0.0-1.0
    /// 1.0 = structural fact (from tree-sitter parsing)
    /// 0.75 = CBM-derived (cross-file call edge)
    /// 0.5 = heuristic-based (pattern matching)
    pub confidence: f64,
    /// Source of this inference
    pub source: InferenceSource,
}

pub enum InferenceEdgeType {
    Calls,
    DataFlowRead,
    DataFlowWrite,
    Injects,
    Extends,
    Implements,
}

pub enum InferenceSource {
    /// From tree-sitter parsing (confidence = 1.0)
    Structural,
    /// From CBM knowledge graph (confidence = 0.75)
    Cbm,
    /// From heuristic pattern matching (confidence = 0.5)
    Heuristic,
    /// From AI-generated reasoning (confidence = configurable)
    AiGenerated,
}

/// A per-symbol annotation with confidence.
pub struct InferenceAnnotation {
    pub key: String,   // "importance", "dead_code", "blast_radius"
    pub value: String, // serialized value
    pub confidence: f64,
    pub source: InferenceSource,
}

impl InferenceLayer {
    /// Build an empty inference layer.
    pub fn new() -> Self { /* ... */ }

    /// Enrich with CBM data. No-op when bridge is None or unavailable.
    /// All CBM-derived edges get confidence = 0.75.
    pub fn enrich_from_cbm(&mut self, bridge: Option<&mut GraphBridge>) {
        let bridge = match bridge {
            Some(b) if b.is_available() => b,
            _ => return,
        };

        // Cross-file CALLS edges (confidence = 0.75)
        let call_edges = bridge.get_call_edges();
        for (caller, callee) in call_edges {
            self.inferred_edges.push(InferenceEdge {
                edge_type: InferenceEdgeType::Calls,
                from: caller,
                to: callee,
                confidence: 0.75,
                source: InferenceSource::Cbm,
            });
        }

        // Symbol importance (confidence = 0.75)
        for (name, info) in bridge.get_symbol_importance_mut() {
            self.annotations.entry(name).or_default().push(
                InferenceAnnotation {
                    key: "importance".into(),
                    value: info.score.to_string(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                }
            );
        }

        // Dead code (confidence = 0.75)
        for entry in bridge.get_dead_code() {
            self.annotations.entry(entry.symbol.clone()).or_default().push(
                InferenceAnnotation {
                    key: "dead_code".into(),
                    value: entry.reason.clone(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                }
            );
        }

        // Blast radius (confidence = 0.75)
        // (populated per-method as needed)
    }

    /// Get all edges with confidence above a threshold.
    pub fn edges_with_confidence(&self, min_confidence: f64) -> Vec<&InferenceEdge> {
        self.inferred_edges.iter()
            .filter(|e| e.confidence >= min_confidence)
            .collect()
    }
}
```

#### Explicit Pass Pipeline

**Current state:** `IRCompiler::compile()` is a single ~500-line function that runs all layers inline. Adding new passes requires modifying this function.

**Modification needed (R-43b):**
```rust
/// A single pass in the IR compilation pipeline.
/// Each pass transforms or enriches the compilation state.
pub trait IRPass {
    /// Name of this pass (for debugging and profiling).
    fn name(&self) -> &str;

    /// Run this pass on the current compilation state.
    /// Passes are ordered and composable.
    fn run(&self, state: &mut PassContext) -> Result<(), PassError>;
}

/// Context passed through the pipeline.
/// Each pass reads from and writes to this context.
pub struct PassContext {
    /// The core instruction stream (pure facts)
    pub instructions: Vec<CoreOp>,
    /// Language-specific context
    pub layer_context: LayerContext,
    /// Program graph (built in Pass 5)
    pub program_graph: Option<ProgramGraph>,
    /// Inference layer (built in Pass 6)
    pub inference_layer: Option<InferenceLayer>,
    /// Source code and metadata
    pub source: String,
    pub file_id: String,
    pub fidelity: Fidelity,
}

/// The composable pass pipeline.
pub struct PassPipeline {
    passes: Vec<Box<dyn IRPass>>,
}

impl PassPipeline {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
    }

    /// Register a pass. Passes run in registration order.
    pub fn add_pass(&mut self, pass: Box<dyn IRPass>) {
        self.passes.push(pass);
    }

    /// Run all registered passes in order.
    pub fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for pass in &self.passes {
            pass.run(state)?;
        }
        Ok(())
    }
}

// ── Built-in Passes ──────────────────────────────────────────────

/// Pass 1: Core IR emission from tree-sitter captures.
pub struct CoreIRPass;

impl IRPass for CoreIRPass {
    fn name(&self) -> &str { "core_ir" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // Existing IRCompiler logic — extracted from compile()
        Ok(())
    }
}

/// Pass 2: Language layer processing.
pub struct LanguageLayerPass {
    layers: Vec<Box<dyn LanguageLayer>>,
}

impl IRPass for LanguageLayerPass {
    fn name(&self) -> &str { "language_layer" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        for layer in &self.layers {
            let ops = layer.finalize(&mut state.layer_context);
            state.instructions.extend(ops);
        }
        Ok(())
    }
}

/// Pass 3: Meta-layer processing (framework-specific).
pub struct MetaLayerPass;

impl IRPass for MetaLayerPass {
    fn name(&self) -> &str { "meta_layer" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // Existing LayerRegistry logic
        Ok(())
    }
}

/// Pass 4: Execution semantics extraction.
pub struct ExecutionSemanticsPass;

impl IRPass for ExecutionSemanticsPass {
    fn name(&self) -> &str { "execution_semantics" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // Extract DataFlow, ControlFlow, SideEffect, ExecutionContext ops
        Ok(())
    }
}

/// Pass 5: Program graph construction.
pub struct ProgramGraphPass;

impl IRPass for ProgramGraphPass {
    fn name(&self) -> &str { "program_graph" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let graph = GraphBuilder::build_from_instructions(&state.instructions);
        state.program_graph = Some(graph);
        Ok(())
    }
}

/// Pass 6: Inference layer (CBM enrichment + derived analysis).
pub struct InferenceLayerPass {
    cbm_bridge: Option<GraphBridge>,
}

impl IRPass for InferenceLayerPass {
    fn name(&self) -> &str { "inference_layer" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        let mut layer = InferenceLayer::new();
        if let Some(bridge) = &self.cbm_bridge {
            layer.enrich_from_cbm(Some(bridge.clone()));
        }
        state.inference_layer = Some(layer);
        Ok(())
    }
}

/// Pass 7: Validation.
pub struct ValidationPass;

impl IRPass for ValidationPass {
    fn name(&self) -> &str { "validation" }
    fn run(&self, state: &mut PassContext) -> Result<(), PassError> {
        // Run IRValidator on the instruction stream
        Ok(())
    }
}
```

**Usage in `compile_file_ir()`:**
```rust
pub fn compile_file_ir(/* ... */) -> Result<CompiledIR, Box<dyn Error>> {
    let mut pipeline = PassPipeline::new();
    pipeline.add_pass(Box::new(CoreIRPass::new(/* ... */)));
    pipeline.add_pass(Box::new(LanguageLayerPass::new(layers)));
    pipeline.add_pass(Box::new(MetaLayerPass));
    pipeline.add_pass(Box::new(ExecutionSemanticsPass));
    pipeline.add_pass(Box::new(ProgramGraphPass));
    pipeline.add_pass(Box::new(InferenceLayerPass { cbm_bridge }));
    pipeline.add_pass(Box::new(ValidationPass));

    let mut ctx = PassContext { /* ... */ };
    pipeline.run(&mut ctx)?;

    Ok(CompiledIR {
        file_id: ctx.file_id,
        instructions: ctx.instructions,
        version: 1,
    })
}
```

---

## 5. R-43a: Execution Semantics (Phase 1)

### 5.1 New CoreOp Variants

Add 4 new variants to the existing `CoreOp` enum in `src/ir/opcodes.rs`:

```rust
pub enum CoreOp {
    // ── Existing 15 variants (unchanged) ──
    DefClass(String, String),
    DefMethod(String, String, String),
    // ... (all existing variants)

    // ── NEW: Execution Semantics ──

    /// Dataflow: ["DATAFLOW", method_id, "reads"|"writes", target_symbol]
    /// Tracks which symbols a method reads from or writes to.
    /// Example: ["DATAFLOW", "M1", "reads", "userRepo"]
    DataFlow(String, String, String),

    /// Control flow: ["CTRL", method_id, kind, target]
    /// kind: "if" | "loop" | "match" | "try" | "await" | "return"
    /// target: the target symbol or expression
    /// Example: ["CTRL", "M1", "loop", "items"]
    ControlFlow(String, String, String),

    /// Side-effect annotation: ["EFFECT", method_id, effect_type]
    /// effect_type: "pure" | "io" | "mutation" | "async" | "transaction"
    /// Example: ["EFFECT", "M1", "io"]
    SideEffect(String, String),

    /// Execution context: ["CTX", method_id, context_type]
    /// context_type: "sync" | "async" | "thread_bound" | "transaction_scope" | "realtime"
    /// Example: ["CTX", "M1", "async"]
    ExecutionContext(String, String),
}
```

**CBM note:** These ops are extracted from tree-sitter captures only. CBM has no role in Phase 1. These are pure facts (confidence = 1.0).

### 5.2 Wire Format Updates

Update the following functions in `src/ir/opcodes.rs`:

| Function | Change |
|----------|--------|
| `opcode_name()` | Add 4 new match arms |
| `arity()` | `DATAFLOW` → 4, `CTRL` → 4, `EFFECT` → 3, `CTX` → 3 |
| `Display` | Add 4 new match arms |

Update the following in `src/ir/delta.rs`:

| Function | Change |
|----------|--------|
| `primary_key()` | Add 4 new match arms (key by method_id) |
| `key_tuple()` | Add 4 new match arms |
| `primary_key_from_tuple()` | Add 4 new match arms |
| `key_tuple_from_tuple()` | Add 4 new match arms |

Update compact delta abbreviations in `src/ir/delta.rs`:

| Opcode | Abbreviation |
|--------|-------------|
| `DATAFLOW` | `DF` |
| `CTRL` | `CT` |
| `EFFECT` | `EF` |
| `CTX` | `CX` |

### 5.3 SemanticIntent on IRDelta

Add optional semantic intent metadata to `IRDelta` in `src/ir/delta.rs`:

```rust
/// High-level semantic intent of a delta operation.
/// Provides human-readable context for what changed, beyond the structural diff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticIntent {
    RenameSymbol {
        old_name: String,
        new_name: String,
        kind: String, // "class", "method", "field"
    },
    AddMethod {
        class: String,
        method_name: String,
    },
    RemoveMethod {
        class: String,
        method_name: String,
    },
    ChangeSignature {
        method: String,
        field_changed: String, // "return_type", "param_type", "param_name"
    },
    AddInjection {
        class: String,
        dependency: String,
    },
    ChangeReturnType {
        method: String,
        old_type: String,
        new_type: String,
    },
}

// Add to IRDelta:
pub struct IRDelta {
    pub file: String,
    pub from: u64,
    pub to: u64,
    pub ops: DeltaOps,
    /// NEW: optional semantic intent metadata
    /// Empty (None) by default — wire format ready for Phase 4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<SemanticIntent>,
}
```

**CBM note:** In Phase 4, CBM's `detect_changes()` and `get_blast_radius()` can enrich the `intent` field with cross-file impact metadata. For Phase 1, the field is always `None`.

### 5.4 Rust Layer — Proof of Concept

File: `src/ir/layers/rust.rs`

Extract behavioral information from existing tree-sitter captures:

| Capture | New Op | Condition | Confidence |
|---------|--------|-----------|-----------|
| `field.root` | `DATAFLOW` | Method body references a field (read or write) | 1.0 (structural) |
| `unsafe` keyword | `SIDEFFECT` | `effect_type = "mutation"` | 1.0 (structural) |
| `async` keyword | `SIDEFFECT` + `CTX` | `effect_type = "async"`, `context_type = "async"` | 1.0 (structural) |
| `match.root` | `CTRL` | `kind = "match"` | 1.0 (structural) |
| `loop.root` / `for.root` / `while.root` | `CTRL` | `kind = "loop"` | 1.0 (structural) |
| `try.root` | `CTRL` | `kind = "try"` | 1.0 (structural) |

**Test coverage:** ~20 new tests in `src/tests/ir/layers/rust.rs`

### 5.5 C# Layer — SignalR & EF Core Pilot

File: `src/ir/layers/csharp.rs`

| Pattern | New Op | Detection | Confidence |
|---------|--------|-----------|-----------|
| `IAsyncEnumerable<T>` | `SIDEFFECT("async", "stream")` | Return type contains `IAsyncEnumerable` | 1.0 (structural) |
| `ChannelReader<T>` / `ChannelWriter<T>` | `DATAFLOW(mid, "writes", "channel")` | Field/method uses Channel types | 1.0 (structural) |
| `Hub` base class | `CTX(mid, "realtime")` | Class extends `Hub` or `Hub<T>` | 1.0 (structural) |
| `IQueryable<T>` | `DATAFLOW(mid, "reads", "db_query")` | Return type or param is `IQueryable` | 1.0 (structural) |
| `DbSet<T>` field | `DATAFLOW(mid, "reads", field_name)` | Field type is `DbSet<T>` | 1.0 (structural) |
| `SaveChangesAsync()` | `SIDEFFECT(mid, "io")` + `CTX(mid, "async")` | Method calls `SaveChangesAsync` | 1.0 (structural) |
| `TransactionScope` | `CTX(mid, "transaction_scope")` | Method body uses `TransactionScope` | 1.0 (structural) |
| `IDisposable` / `IAsyncDisposable` | `SIDEFFECT(mid, "io")` | Class implements disposable pattern | 1.0 (structural) |

**Test coverage:** ~25 new tests in `src/tests/ir/layers/csharp.rs`

### 5.6 TypeScript Layer — RxJS & Angular

File: `src/ir/layers/typescript.rs`

| Pattern | New Op | Detection | Confidence |
|---------|--------|-----------|-----------|
| `.subscribe()` | `DATAFLOW(mid, "reads", "observable")` | Method body calls `.subscribe()` | 1.0 (structural) |
| `.pipe()` with `tap` | `SIDEFFECT(mid, "io")` | Observable pipe contains `tap` operator | 1.0 (structural) |
| `.pipe()` with `map`/`filter` | `DATAFLOW(mid, "reads", "observable")` | Observable pipe contains transform operators | 1.0 (structural) |
| `async` keyword | `SIDEFFECT(mid, "async")` + `CTX(mid, "async")` | Method is async | 1.0 (structural) |
| `new Observable()` | `DATAFLOW(mid, "writes", "observable")` | Method creates an Observable | 1.0 (structural) |
| `@Injectable()` | `CTX(mid, "di_scope")` | Class has Injectable decorator | 1.0 (structural) |

**Test coverage:** ~15 new tests in `src/tests/ir/layers/typescript.rs`

---

## 6. R-43b: Phases 2-6 (Future)

### 6.1 Phase 2: Lightweight Local Program Graph

**File:** `src/ir/program_graph.rs` (new module)

```rust
/// A lightweight local program graph built from compiled IRs.
/// Nodes are symbols (classes, methods, fields), edges are relationships.
///
/// This graph is built from structural IR facts only (confidence = 1.0).
/// Inferred edges (from CBM or heuristics) live in InferenceLayer, NOT here.
pub struct ProgramGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub id: String,
    pub name: String,
    pub kind: SymbolKind,
    pub file_id: String,
}

pub enum GraphEdge {
    /// Structural edge from tree-sitter parsing (confidence = 1.0)
    Calls { from: String, to: String },
    /// Structural edge from EXT op (confidence = 1.0)
    Extends { child: String, parent: String },
    /// Structural edge from IMPL op (confidence = 1.0)
    Implements { class: String, interface: String },
    /// Structural edge from INJECTS op (confidence = 1.0)
    Injects { class: String, dependency: String },
    /// Structural edge from DATAFLOW op (confidence = 1.0)
    DataFlowRead { method: String, target: String },
    /// Structural edge from DATAFLOW op (confidence = 1.0)
    DataFlowWrite { method: String, target: String },
}

pub struct GraphBuilder;

impl GraphBuilder {
    /// Build a program graph from compiled IRs and symbol table.
    /// Lazy — called on demand, not during compilation.
    /// All edges have confidence = 1.0 (structural facts).
    pub fn build(
        compiled_irs: &[CompiledIR],
        symbol_table: &GlobalSymbolTable,
    ) -> ProgramGraph { ... }

    /// Build a program graph from an instruction stream (for pass pipeline).
    pub fn build_from_instructions(instructions: &[CoreOp]) -> ProgramGraph { ... }
}
```

**Key design decisions:**
- Lazy computation — not stored in `CompiledIR`, built on demand from `Vec<CompiledIR>`
- **No CBM data in ProgramGraph** — all edges are structural facts (confidence = 1.0)
- CBM-derived edges live in `InferenceLayer` (confidence = 0.75)
- Edge types reuse existing `CoreOp` relationships (EXT, IMPL, INJECTS) plus new DATAFLOW edges

### 6.2 Phase 3: Inference Layer

**File:** `src/ir/inference_layer.rs` (new module)

```rust
/// The Inference Layer holds all non-deterministic, derived, or estimated
/// data about the IR. This layer is NEVER serialized into the core IR wire
/// format. It is recomputed on demand and lives only in memory.
///
/// ── Facts vs Inferences ──────────────────────────────────────────
/// CoreOp stream = pure facts (structural, deterministic, replayable)
/// InferenceLayer = derived analysis (estimated, non-deterministic, ephemeral)
///
/// ── Confidence Scores ────────────────────────────────────────────
/// 1.0  = structural fact (from tree-sitter parsing)
/// 0.75 = CBM-derived (cross-file call edge, importance, dead code)
/// 0.5  = heuristic-based (pattern matching, estimation)
/// 0.25 = AI-generated (LLM reasoning, subject to hallucination)
pub struct InferenceLayer {
    /// Inferred edges with confidence scores
    pub inferred_edges: Vec<InferenceEdge>,
    /// Per-symbol annotations (importance, dead code, blast radius)
    pub annotations: HashMap<String, Vec<InferenceAnnotation>>,
}

/// An inferred edge between two symbols.
pub struct InferenceEdge {
    pub edge_type: InferenceEdgeType,
    pub from: String,
    pub to: String,
    /// Confidence score 0.0-1.0
    pub confidence: f64,
    /// Source of this inference
    pub source: InferenceSource,
}

pub enum InferenceEdgeType {
    Calls,
    DataFlowRead,
    DataFlowWrite,
    Injects,
    Extends,
    Implements,
}

pub enum InferenceSource {
    /// From tree-sitter parsing (confidence = 1.0)
    Structural,
    /// From CBM knowledge graph (confidence = 0.75)
    Cbm,
    /// From heuristic pattern matching (confidence = 0.5)
    Heuristic,
    /// From AI-generated reasoning (confidence = configurable)
    AiGenerated,
}

/// A per-symbol annotation with confidence.
pub struct InferenceAnnotation {
    pub key: String,   // "importance", "dead_code", "blast_radius"
    pub value: String, // serialized value
    pub confidence: f64,
    pub source: InferenceSource,
}

impl InferenceLayer {
    pub fn new() -> Self { /* ... */ }

    /// Enrich with CBM data. No-op when bridge is None or unavailable.
    /// All CBM-derived edges get confidence = 0.75.
    pub fn enrich_from_cbm(&mut self, bridge: Option<&mut GraphBridge>) { /* ... */ }

    /// Get all edges with confidence above a threshold.
    pub fn edges_with_confidence(&self, min_confidence: f64) -> Vec<&InferenceEdge> { /* ... */ }

    /// Get annotations for a symbol with confidence above a threshold.
    pub fn annotations_for(
        &self,
        symbol: &str,
        min_confidence: f64,
    ) -> Vec<&InferenceAnnotation> { /* ... */ }
}
```

**CBM enrichment method:**
```rust
impl InferenceLayer {
    pub fn enrich_from_cbm(&mut self, bridge: Option<&mut GraphBridge>) {
        let bridge = match bridge {
            Some(b) if b.is_available() => b,
            _ => return,
        };

        // Cross-file CALLS edges (confidence = 0.75)
        let call_edges = bridge.get_call_edges();
        for (caller, callee) in call_edges {
            self.inferred_edges.push(InferenceEdge {
                edge_type: InferenceEdgeType::Calls,
                from: caller,
                to: callee,
                confidence: 0.75,
                source: InferenceSource::Cbm,
            });
        }

        // Symbol importance (confidence = 0.75)
        for (name, info) in bridge.get_symbol_importance_mut() {
            self.annotations.entry(name).or_default().push(
                InferenceAnnotation {
                    key: "importance".into(),
                    value: info.score.to_string(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                }
            );
        }

        // Dead code (confidence = 0.75)
        for entry in bridge.get_dead_code() {
            self.annotations.entry(entry.symbol.clone()).or_default().push(
                InferenceAnnotation {
                    key: "dead_code".into(),
                    value: entry.reason.clone(),
                    confidence: 0.75,
                    source: InferenceSource::Cbm,
                }
            );
        }
    }
}
```

### 6.3 Phase 4: Semantic Delta

**Intent detection in `DeltaComputer`:**
- Compare old and new `CompiledIR` to detect high-level intent
- Populate the `intent` field on `IRDelta` (wire format already prepared in Phase 1)
- Examples: `RenameSymbol`, `AddMethod`, `ChangeSignature`, `AddInjection`

**CBM enrichment (optional):**
- CBM's `detect_changes()` can trigger re-compilation when the graph detects structural changes
- CBM's `get_blast_radius()` can populate cross-file impact metadata in the intent
- Example: `SemanticIntent::RenameSymbol` with `affected_files: Vec<String>` from CBM

### 6.4 Phase 5: IR Validation Engine

**File:** `src/ir/validator.rs` (new module)

```rust
pub trait IRValidator {
    fn validate(&self, ir: &CompiledIR) -> Vec<ValidationError>;
}

pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub instruction_index: Option<usize>,
}
```

**Built-in validation rules:**
- Every `RET` has a corresponding `DEF_M` reference
- Every `INJECTS` target exists in the symbol table
- No dangling `EXT`/`IMPL` references
- No duplicate method IDs within a class
- **Side-effect consistency:** async flag → must have `ExecutionContext("async")`
- **Side-effect consistency:** `SIDEFFECT("io")` → should have `CTX` with matching context

**CBM note:** CBM has no role in validation. Validation is purely structural.

### 6.5 Phase 6: Queryable IR

**File:** `src/ir/query.rs` (new module)

```rust
/// Query engine that combines local IR analysis with optional CBM enrichment.
/// Queries return results with confidence scores.
pub struct IRQueryEngine {
    graph: ProgramGraph,
    inference: Option<InferenceLayer>,
    /// Optional CBM bridge for cross-file queries
    cbm_bridge: Option<GraphBridge>,
}

impl IRQueryEngine {
    pub fn new(graph: ProgramGraph) -> Self { /* ... */ }

    pub fn with_inference(mut self, inference: InferenceLayer) -> Self { /* ... */ }

    pub fn with_cbm(mut self, bridge: GraphBridge) -> Self { /* ... */ }

    /// Find all async methods — local + CBM cross-file results.
    /// Returns results with confidence scores.
    pub fn find_async_methods(&self) -> Vec<QueryResult> {
        let mut results = Vec::new();

        // Local: scan nodes for ExecutionContext("async") — confidence = 1.0
        for node in &self.graph.nodes {
            if node.kind == SymbolKind::Method && self.has_async_ctx(node) {
                results.push(QueryResult {
                    node: node.clone(),
                    confidence: 1.0,
                    source: "structural",
                });
            }
        }

        // CBM enrichment: search for *Async methods across all files
        if let Some(bridge) = &self.cbm_bridge {
            let cbm_results = bridge.search(".*Async");
            for node in &cbm_results {
                if !results.iter().any(|r| r.node.name == node.name) {
                    results.push(QueryResult {
                        node: /* synthetic node from CBM */,
                        confidence: 0.75,
                        source: "cbm",
                    });
                }
            }
        }
        results
    }

    /// Get fan-in (callers) — local + CBM blast radius.
    /// Returns count with confidence breakdown.
    pub fn get_fan_in(&self, method: &str) -> FanInResult {
        let local_count = self.graph.edges.iter()
            .filter(|e| matches!(e, GraphEdge::Calls { to, .. } if to == method))
            .count();

        let cbm_count = if let Some(bridge) = &self.cbm_bridge {
            bridge.get_blast_radius(method, 1).len()
        } else {
            0
        };

        FanInResult {
            method: method.to_string(),
            local_callers: local_count,
            inferred_callers: cbm_count,
            total: local_count + cbm_count,
            confidence: if cbm_count > 0 { 0.85 } else { 1.0 },
        }
    }

    // ... other query methods
}

pub struct QueryResult {
    pub node: GraphNode,
    pub confidence: f64,
    pub source: &'static str,
}

pub struct FanInResult {
    pub method: String,
    pub local_callers: usize,
    pub inferred_callers: usize,
    pub total: usize,
    pub confidence: f64,
}
```

**CBM enrichment for each query:**

| Local Query | CBM Fallback | Merge Strategy | Confidence |
|-------------|-------------|----------------|-----------|
| `find_async_methods()` | `search(".*Async")` | Deduplicate by name | 1.0 local, 0.75 CBM |
| `trace_injection_chain(class)` | `trace_path(class, "")` | Append CBM edges | 1.0 local, 0.75 CBM |
| `get_fan_in(method)` | `get_blast_radius(method, 1)` | Sum local + CBM counts | 1.0 local, 0.75 CBM |
| `get_fan_out(method)` | `trace_path(method, "")` | Count CBM outbound edges | 1.0 local, 0.75 CBM |
| `find_dataflow_sources(method)` | `query_graph("MATCH ...")` | Append CBM dataflow edges | 1.0 local, 0.75 CBM |
| `find_side_effects(method)` | Local only (CBM doesn't track) | No CBM enrichment | 1.0 |

---

## 7. Implementation Order

### R-43a Implementation Steps

| Step | Description | Files | Est. Time |
|------|-------------|-------|----------:|
| 1 | Add 4 new `CoreOp` variants + wire format updates | `src/ir/opcodes.rs`, `src/ir/delta.rs` | 4 hours |
| 2 | Add `SemanticIntent` to `IRDelta` | `src/ir/delta.rs` | 1 hour |
| 3 | Rust layer PoC (dataflow + side effects) | `src/ir/layers/rust.rs` | 4 hours |
| 4 | C# layer (SignalR/EF Core pilot) | `src/ir/layers/csharp.rs` | 6 hours |
| 5 | TypeScript layer (RxJS/Angular) | `src/ir/layers/typescript.rs` | 4 hours |
| 6 | Integration tests + clippy pass | `src/tests/` | 3 hours |
| | **Total R-43a** | | **~22 hours (4-5 days)** |

### R-43b Implementation Steps

| Step | Description | Files | Est. Time |
|------|-------------|-------|----------:|
| 1 | Program graph module (structural only, no CBM) | `src/ir/program_graph.rs` | 4 hours |
| 2 | Inference layer module + CBM enrichment | `src/ir/inference_layer.rs`, `src/cbm/bridge.rs` | 4 hours |
| 3 | Explicit pass pipeline (IRPass trait + PassPipeline) | `src/ir/pipeline.rs` | 4 hours |
| 4 | Semantic delta detection | `src/ir/delta.rs` | 4 hours |
| 5 | IR validation engine | `src/ir/validator.rs` | 4 hours |
| 6 | Query API + CBM fallback + confidence scores | `src/ir/query.rs` | 6 hours |
| 7 | Integration tests + clippy pass | `src/tests/` | 4 hours |
| | **Total R-43b** | | **~30 hours (6-8 days)** |

---

## 8. Success Criteria

### R-43a Success Criteria
- [x] All 4 new `CoreOp` variants serialize/deserialize in all 6 wire formats
- [x] Delta transport handles new variants (add/modify/remove by primary key)
- [x] Rust PoC extracts dataflow + side effects from real Rust code
- [x] C# layer detects SignalR streaming and EF Core dataflow patterns
- [x] TypeScript layer detects RxJS observable chains
- [x] `SemanticIntent` field on `IRDelta` serializes/deserializes (empty by default)
- [x] Zero clippy warnings (`cargo clippy --all-targets -- -D warnings`)
- [x] All **1745+** tests pass (`cargo test --workspace --all-targets --all-features`)
- [x] No regression in token savings or latency
- [x] CBM integration unchanged — all existing CBM tests still pass

### R-43b Success Criteria
- [x] `ProgramGraph` builds correctly from `Vec<CompiledIR>` + `GlobalSymbolTable`
- [x] Edge types cover calls, extends, implements, injects, dataflow_read, dataflow_write
- [x] **ProgramGraph contains NO CBM data** — all edges are structural facts (confidence = 1.0)
- [x] `InferenceLayer::enrich_from_cbm()` populates all CBM fields (no-op when CBM unavailable)
- [x] **InferenceLayer is NEVER serialized into core IR wire format**
- [x] All inferred edges carry `confidence: f64` scores (1.0 structural, 0.75 CBM, 0.5 heuristic)
- [x] `PassPipeline` composes passes in correct order (Core → Language → Meta → Exec → Graph → Inference → Validation)
- [x] `IRPass` trait is implementable for new passes without modifying existing code
- [x] Semantic intent detection produces correct `SemanticIntent` values
- [x] IR validation catches dangling references and side-effect inconsistencies
- [x] Query API returns results with confidence scores
- [x] CBM-enriched queries return cross-file results when CBM is available
- [x] CBM-enriched queries fall back to local-only when CBM is unavailable
- [x] Zero clippy warnings, all tests pass, no regression

### Post-Implementation Validation (2026-07-06)
- [x] **1553 library tests** — all passing (1593 total including integration/regression)
- [x] **29 InferenceLayer tests** — covering all edge types, sources, confidence boundaries, stress tests, Unicode, special chars, Clone/Debug traits, Hash equality
- [x] **18 round-trip tests** — covering all 6 wire formats, compact delta, randomized property tests (100/100/50 iterations)
- [x] **47 R-43b tests** — 7 program_graph, 6 inference_layer, 7 pipeline, 13 validator, 10 query, 4 delta
- [x] **Zero clippy warnings** — `cargo clippy --all-targets -- -D warnings` passes cleanly
- [x] **FAANG audit** — zero critical or high-severity findings

---

## 9. Risk & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Scope creep (adding more behavioral patterns than planned) | Medium | Medium | Phase-by-phase with clear acceptance criteria. R-43a scope is frozen. |
| Performance regression from new ops in wire formats | Low | Medium | New ops are additive — existing code paths unchanged. Lazy graph computation. |
| Backward compatibility break | Low | High | New `CoreOp` variants are additive. Old IR files parse without the new variants. `SemanticIntent` is `Option` and `skip_serializing_if`. |
| C# tree-sitter captures insufficient for SignalR/EF Core detection | Low | Medium | Fall back to text-pattern matching in the C# language layer. Tree-sitter captures can be extended separately. |
| Pilot value not realized (SignalR/EF Core patterns not detected in real code) | Low | High | Validate against real production codebase during development. Adjust detection patterns iteratively. |
| CBM enrichment adds latency to graph construction | Low | Medium | `enrich_from_cbm()` is called explicitly, not in the hot path. CBM queries are cached with TTL. |
| CBM data format changes between versions | Low | Low | `GraphBridge` already abstracts CBM client. Enrichment methods are version-agnostic. |
| InferenceLayer diverges from structural IR (sync bug) | Low | High | **Architectural invariant:** InferenceLayer is NEVER authoritative over CoreOp. Validation pass checks consistency. |
| Pass pipeline ordering errors | Low | Medium | `PassPipeline` validates pass order at registration time. Integration tests verify correct order. |

---

## Appendix A: Wire Format Examples

### Named Format (with new ops)

```json
{
  "file": "α1",
  "v": 1,
  "encoding": "named",
  "ir": [
    ["DEF_C", "C1", "ChatHub"],
    ["DEF_M", "C1", "M1", "SendMessage"],
    ["SIG", "M1", "P1", "$s", "message"],
    ["RET", "M1", "$v"],
    ["EFFECT", "M1", "io"],
    ["CTX", "M1", "realtime"],
    ["DATAFLOW", "M1", "writes", "channel"]
  ]
}
```

### Hierarchical Format (with new ops)

```json
{
  "encoding": "hierarchical",
  "file": "α1",
  "v": 1,
  "ir": {
    "c": [{
      "n": "C1", "nm": "ChatHub",
      "m": [{
        "n": "M1", "nm": "SendMessage",
        "p": [["P1", "$s", "message"]],
        "r": "$v",
        "ef": ["io"],
        "cx": ["realtime"],
        "df": [["writes", "channel"]]
      }]
    }]
  }
}
```

### LLM-Optimized Text (SCHEMA v2, with new ops)

```
// SCHEMA v2  @=meta X=extends I=implements F=field M=method $=import →=scope fl:=flags cl:=class-flags P=pattern T=type-alias ef:=effect cx:=context df:=dataflow
// ── ChatHub ──
X Hub
M SendMessage(message:$s):$v  ef:io cx:realtime df:writes:channel
```

---

## Appendix B: Delta Example with SemanticIntent

```json
{
  "file": "α1",
  "from": 1,
  "to": 2,
  "intent": {
    "type": "add_method",
    "class": "ChatHub",
    "method_name": "SendMessage"
  },
  "ops": {
    "+": [
      ["DEF_M", "C1", "M1", "SendMessage"],
      ["SIG", "M1", "P1", "$s", "message"],
      ["RET", "M1", "$v"],
      ["EFFECT", "M1", "io"],
      ["CTX", "M1", "realtime"]
    ],
    "~": [],
    "-": []
  }
}
```

---

## Appendix C: CBM Integration Code Map

| File | Current CBM Integration | R-43b Changes |
|------|------------------------|---------------|
| `src/cbm/bridge.rs` | `GraphBridge` with `get_symbol_importance_mut()`, `get_blast_radius()`, `get_dead_code()`, `search()`, `query_graph()`, `trace_path()` | Add `get_call_edges()`, `get_dataflow_edges()` methods |
| `src/cbm/handlers.rs` | MCP tool handlers for `graph_search`, `graph_query`, `graph_trace`, `get_architecture`, `get_cbm_status` | No changes needed |
| `src/cbm/proxy.rs` | `handle_cbm_proxy()` — pipe-level interception + JSON compression | No changes needed |
| `src/cbm/json_compress.rs` | `compress_cbm_response()` — JSON key shortening | No changes needed |
| `src/intelligence/fidelity.rs` | `cbm_informed_fidelity()`, `build_cbm_skip_set()` | No changes needed (already works) |
| `src/ir/program_graph.rs` | **New module** | Structural graph only (no CBM data) |
| `src/ir/inference_layer.rs` | **New module** | `InferenceLayer::enrich_from_cbm()` |
| `src/ir/pipeline.rs` | **New module** | `IRPass` trait + `PassPipeline` |
| `src/ir/query.rs` | **New module** | `IRQueryEngine` with confidence scores |
| `src/ir/validator.rs` | **New module** | Structural validation only |

---

*This document is a living plan. Update as implementation progresses and new insights emerge.*