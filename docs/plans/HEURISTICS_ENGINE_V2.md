# Heuristics Engine V2 — Auto-Inferred Intent

## Current State (V1)

The current heuristics engine (`src/mcp/heuristics.rs`) decides fidelity on a simple 5-priority system:

| Priority | Rule | Result |
|----------|------|--------|
| 1 | Explicit `fidelity` arg | Use it directly |
| 2 | Explicit `intent` arg | Map via `smart_defaults` (refactor→High, overview→Low, debug→Medium, edit→Low, implement→Medium) |
| 3 | File matches `force_high_fidelity` patterns | High |
| 4 | File > `large_file_threshold` (300 lines) | Low |
| 5 | Config `default_fidelity` | Low |

### Problems

1. **Large files get Low fidelity** — a 1,500-line service with dozens of methods/injections gets stripped to a skeleton, losing the structural detail the LLM needs to understand the dependency graph.

2. **No content analysis** — the engine knows nothing about what the file contains. It can't distinguish a test file from a complex service from a simple config.

3. **"edit" intent gets Low** — editing a complex file is precisely when you need the most context about what's connected to what.

4. **Small files get no differentiation** — a 5-line config and a 200-line type definition file both fall through to the same default.

5. **No session awareness** — doesn't consider file modification time or the fact that the DB has baseline fidelity information from prior compressions.

## V2 Design

### Core Principle

**More complex files → higher fidelity.** The LLM needs more structural information to reason about complex code. Simple files benefit from maximum compression. The engine should auto-classify files based on cheap content signals (no tree-sitter parse required) and map classification to appropriate fidelity.

### Architecture

```
                     ┌─────────────────────┐
                     │   `decide()` entry   │
                     └────────┬────────────┘
                              │
                 ┌────────────▼────────────┐
                 │ 1. Explicit fidelity?   │──Yes──▶ Use it
                 └────────────┬────────────┘
                              │ No
                 ┌────────────▼────────────┐
                 │ 2. Explicit intent?     │──Yes──▶ Map via smart_defaults
                 └────────────┬────────────┘
                              │ No
                 ┌────────────▼────────────┐
                 │ 3. DB has baseline?     │──Yes──▶ Use last fidelity
                 └────────────┬────────────┘      (if file unchanged)
                              │ No/Changed
                 ┌────────────▼────────────┐
                 │ 4. Content classifier   │
                 │  (NEW — cheap signals)  │
                 └────────────┬────────────┘
                              │
                 ┌────────────▼────────────┐
                 │ 5. Complexity fallback  │
                 │  (NEW — reversed logic) │
                 └────────────────────────┘
```

### Step 4: Content-Based File Classification (NEW)

Scans source text for cheap string signals — no tree-sitter parse needed, runs in microseconds.

#### Classification Tiers

| Classification | Detection Signals | Fidelity | Rationale |
|---|---|---|---|
| **test** | `#[test]`, `#[cfg(test)]`, `fn test_`, `@Test`, `describe(`, `it(`, path contains `/test/` or `/tests/` or `/__tests__/` or `.test.` or `.spec.` | **Low** | Test files need structure overview, not implementation detail |
| **config** | Path contains `config`, `settings`, `.json`, `.toml`, `.yaml`, `.env`; OR file < 50 lines with mostly `const`/`static`/`let` declarations | **Low** | Simple key-value/shape files, max compression |
| **model/types** | High ratio of `struct`/`enum`/`type`/`interface`/`class` (fields only) definitions to function definitions (>3:1 struct/enum lines to fn lines) | **Medium** | Need type shapes but not body implementations |
| **service/complex** | >15 imports AND >10 `fn`/`pub fn` | **High** | Large interconnected files — LLM needs full structure to understand dependencies |
| **implementation** | Path contains `.component.`, `.controller.`, `.handler.`, `.middleware.`, `.service.`; OR file >200 lines with mix of imports/functions/types | **Medium** | Moderate detail for UI/handler/business logic |
| **general** | Doesn't match any classifier | → Step 5 |

#### Strategy Per Classification

| Classification | First Call | Subsequent Calls |
|---|---|---|
| test, config, model | FullCompress (cost is low) | DeltaTransport |
| service, implementation | FullCompress (need full context) | DeltaTransport (deltas are efficient once baseline exists) |

### Step 5: Complexity-Based Fallback (REVERSED)

Applies only to files that didn't match any classifier in Step 4.

| Complexity Score | Fidelity |
|---|---|
| Import count > 20 AND function count > 15 AND lines > 500 | **High** — architectural complexity |
| Import count > 10 OR function count > 10 OR lines > 300 | **Medium** — non-trivial implementation |
| Lines > 100 | **Low** — manageable size, skeleton is sufficient |
| Everything else | Config `default_fidelity` (Low) |

The complexity score is computed as a weighted sum:
- `import_count * 2` — imports indicate dependency surface area
- `function_count` — functions indicate behavior surface area  
- `line_count / 100` — raw size matters

Thresholds are configurable in `HeuristicsConfig`.

### Step 3: Session-Aware Fidelity (NEW)

Before running the content classifier, check the DB:

1. Query `ContextStore::load_latest(file_path)`
2. If found AND file modtime matches last compress modtime → **reuse the fidelity from the DB baseline**
3. If found but modtime differs → proceed to classifier (file was edited)
4. If not found → proceed to classifier (first time)

This ensures that re-visiting the same unchanged file across sessions preserves the fidelity level that was used before.

### Signal Detection Functions

All signals are pure string scans — zero parsing overhead:

```rust
fn count_imports(source: &str) -> usize
    // Rust: lines starting with "use " or "extern crate"
    // TS: lines starting with "import " or "from "
    // C#: lines starting with "using "
    // Counts by extension pattern

fn count_functions(source: &str) -> usize
    // Rust: "fn " at line start (not preceded by other tokens)
    // TS: "function ", arrow functions, method signatures
    // C#: "void ", "Task ", "int " etc. followed by "(" on same line

fn count_structs_enums(source: &str) -> usize
    // Rust: "struct ", "enum ", "trait " at line start
    // TS: "interface ", "type ", "class " at line start
    // C#: "class ", "struct ", "enum ", "interface " at line start

fn count_test_markers(source: &str) -> usize
    // #[test], #[cfg(test)], fn test_, @Test, describe(, it(

fn detect_language(file_path: &str) -> Language
    // .rs → Rust, .ts/.tsx → TypeScript, .cs → C#, .js → JavaScript
```

### HeuristicsConfig Changes

```rust
pub struct HeuristicsConfig {
    // EXISTING — modified behavior
    pub large_file_threshold: usize,  // now used in complexity score, not direct Low trigger
    pub force_high_fidelity: Vec<String>,  // unchanged
    pub use_angular_meta: bool,  // unchanged

    // NEW
    /// Min imports to classify as "service/complex"
    #[serde(default = "default_complex_import_threshold")]
    pub complex_import_threshold: usize,  // default: 15

    /// Min functions to classify as "service/complex"
    #[serde(default = "default_complex_fn_threshold")]
    pub complex_fn_threshold: usize,  // default: 10

    /// Min lines for complexity fallback to Medium
    #[serde(default = "default_medium_lines")]
    pub medium_lines: usize,  // default: 300

    /// Min lines for complexity fallback to High
    #[serde(default = "default_high_lines")]
    pub high_lines: usize,  // default: 500

    /// Whether to auto-classify files by content
    #[serde(default = "default_true")]
    pub auto_classify: bool,  // default: true

    /// Whether to check DB for prior fidelity on re-visits
    #[serde(default = "default_true")]
    pub session_aware_fidelity: bool,  // default: true
}

fn default_complex_import_threshold() -> usize { 15 }
fn default_complex_fn_threshold() -> usize { 10 }
fn default_medium_lines() -> usize { 300 }
fn default_high_lines() -> usize { 500 }
```

### Example: Real Files in This Project

| File | Old Result | New Result | Why |
|---|---|---|---|
| `tool_handlers.rs` (1,500+ lines, 25+ imports, 15+ fns) | Low (P4: large file) | **High** (service/complex) | Massive file with deep dependency graph |
| `heuristics.rs` (240 lines, 5 imports, 8 fns) | Low (default) | **Medium** (implementation: .rs with moderate complexity) | Need API shapes but not full bodies |
| `cache_hints.rs` (268 lines, 3 imports, 8 fns) | Low (default) | **Medium** (implementation: moderate complexity) | Need function signatures and types |
| `mod.rs` (28 lines) | Low (default) | **Low** (small file) | Too small to benefit from higher fidelity |
| `main.rs` (89 lines, 3 imports, 3 fns) | Low (default) | **Low** (small file) | Simple entry point, skeleton suffices |
| Test files (any) | Low (default) | **Low** (test classifier) | Test files just need what's being tested |
| `.clean-ctx.json` (config) | Low (default) | **Low** (config classifier) | Simple JSON config |

### Test Plan

1. **Content classifier tests** — verify each classification tier detects correctly across Rust/TS/C#:
   - Test files: `#[test]` → test
   - Config: `config.rs` path → config
   - Types: file with 10 structs, 1 fn → model
   - Service: file with 20 imports, 15 fns → service
   - Implementation: handler.rs with moderate content → implementation

2. **Complexity fallback tests** — verify scoring thresholds:
   - 0 imports, 2 fns, 50 lines → Low
   - 12 imports, 8 fns, 350 lines → Medium
   - 25 imports, 20 fns, 600 lines → High

3. **Session-aware fidelity tests** — verify DB lookup:
   - File in DB with "high" fidelity → uses "high" on re-visit
   - File changed on disk → runs full classifier

4. **Priority tests** — verify order:
   - Explicit arg always wins
   - Intent always wins over auto-classify
   - DB baseline wins over auto-classify (when file unchanged)

5. **Regression tests** — all existing heuristics tests must still pass with updated logic

### Files to Modify

| File | Change |
|---|---|
| `src/mcp/heuristics.rs` | Full rewrite of `resolve_fidelity()` + new classifier + complexity scorer |
| `src/config.rs` | Add new `HeuristicsConfig` fields |
| `src/mcp/tool_handlers.rs` | Pass source text + DB context to `decide()` (already available) |
| `src/tests/mcp/heuristics.rs` | New tests for all classification tiers |
| `src/main.rs` | Update default config generation to include new fields |

### No Breaking Changes

- All existing config fields preserved
- Explicit `fidelity` and `intent` args still override everything
- Default behavior improves for the no-args case
- Users can disable `auto_classify` in config to get old behavior