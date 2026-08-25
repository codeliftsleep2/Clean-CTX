# Clean-CTX — Intelligence Layer Plan
**Version:** 0.1.0 (proposed)
**Status:** 📋 Proposed · Last updated: 2026-06-10

---

## Core Principle

The Intelligence Layer is **purely additive** — it never modifies existing compression output. It adds ranked, budget-aware, and impact-aware context selection on top of the existing compression and delta pipelines. Existing users see no change; users who opt in get smarter context delivery.

---

## Background: What Already Exists

Before defining what's new, the full current stack:

| Subsystem | Status | Key modules |
|-----------|--------|-------------|
| Tree-sitter AST parse (TS + C#) | ✅ | `queries.rs`, `compression/capture_pipeline.rs` |
| Three-fidelity text compression | ✅ | `compression/pipeline.rs`, `fidelity.rs` |
| Behavior markers (⊕guard, ⊕loop, ⊕⇒, ⊕!) | ✅ | `compression/markers.rs` |
| Symbol + path dictionaries | ✅ | `dictionary/path.rs`, `dictionary/symbol.rs` |
| Huffman-coded symbol dictionary | ✅ | `dictionary/huffman.rs` |
| Global cross-file symbol table | ✅ | `dictionary/workspace.rs` |
| Micro-opcodes (VarInt) | ✅ | `compression/micro_opcodes.rs` |
| IR compile → wire → string table → binary | ✅ | `ir/` subsystem |
| IR delta transport (field-level) | ✅ | `ir/delta.rs`, `ir/replay.rs` |
| Text delta transport | ✅ | `compression/text_delta.rs` |
| Angular Meta-Layer (Phases 1+2+3) | ✅ | `angular_meta/` |
| Zero-touch workflow + heuristics engine | ✅ | `mcp/heuristics.rs`, `mcp/tools.rs` |
| SQLite persistence (ContextStore) | ✅ | `mcp/sqlite_store.rs`, `mcp/context_store.rs` |
| Session stats dashboard | ✅ | `mcp/session_stats.rs` |
| 945 tests, 0 unsafe, clippy clean | ✅ | |

**The Intelligence Layer adds one new capability:** relevance-ranked, budget-constrained, impact-aware delivery on top of all of the above. The stack compounds:

```
Compression → reduces token cost of what's sent
Delta Transport → sends only what changed
Intelligence Layer → sends the most important things first, within a budget
```

---

## Decisions Locked

| Question | Decision |
|----------|----------|
| Graph input | Reuse existing IR graph + `AngularGraph` (Phase 3) — no new graph infrastructure |
| Language scope | Language-agnostic. Identical algorithm for TS and C# language layers |
| Meta-layer scope | Angular/NgRx/RxJS meta-layers feed richer graph edges automatically when they exist |
| Marker approach | No new Φ markers. PageRank scores are internal metadata only; `§RANK` summary emitted in workspace manifest header |
| Phasing | Three independently shippable phases. Each phase is useful on its own |
| Default state | All three phases **off by default**. Opt-in per-feature via `.clean-ctx.json`. Zero overhead when disabled — byte-identical output to current pipeline |
| Token budget | Optional. System works without a budget; budget enables knapsack packing (Phase 3 only) |
| New dependencies | Phase 1: none. Phase 2: none. Phase 3: none |
| Persistence integration | Phase 2 blast radius uses the existing `ContextStore` / `SqliteStore` session history — no new storage |
| Heuristics engine integration | Phase 1 adaptive fidelity feeds into `heuristics.rs` decision — PageRank score becomes one input alongside file size, language, and intent |

---

## Notation Map (additions to existing)

| Prefix | Job | Examples |
|--------|-----|---------|
| `$xx` | Opcodes — language primitives (existing) | `$c`, `$ctor`, `$a` |
| `⊕` | Behavior markers (existing) | `⊕guard`, `⊕loop`, `⊕⇒`, `⊕!` |
| `Φ` | Framework annotation markers (existing) | `Φcmp:`, `Φsvc:`, `Φinjects:`, `Φgraph:` |
| `§RANK` | Symbol rank summary in manifest **(new)** | `§RANK UserService=0.91 UserCard=0.74` |
| `§BUDGET` | Token budget header in manifest **(new)** | `§BUDGET 8192 used=7841 dropped=12` |
| `§IMPACT` | Blast radius summary in delta **(new)** | `§IMPACT UserService→[UserCardComponent@α3,UserProfileComponent@α7]` |

---

## Phase 1 — PageRank Symbol Scoring + Adaptive Fidelity

### Goal

Score every symbol in the workspace by structural importance using graph edges that already exist in the IR and Angular graph. Use the score to automatically choose fidelity **per symbol** rather than applying uniform fidelity to the whole file.

**Before:** All symbols in a file compressed at the same fidelity level (set by heuristics engine or caller).
**After:** High-importance symbols → higher fidelity. Low-importance symbols → lower fidelity or skeleton-only. Same or lower token cost, more information where it matters.

### How PageRank Maps to the Existing Graph

**Seed nodes (score = 1.0) — by language layer:**

| Language Layer | Seeds |
|---------------|-------|
| TypeScript | Exported symbols (`$e` opcode present in compressed output) |
| C# | `public` class members; controller action methods |
| Angular meta-layer | `@Component` selectors; `@Injectable({providedIn: 'root'})` |
| NgRx meta-layer (future) | Action creators dispatched from components |

**Graph edges — already exist:**

| Edge type | Source |
|-----------|--------|
| Method calls | IR `CoreOp` instruction graph |
| Import / `using` statements | IR import graph |
| DI injections | `Φinjects:` from `angular_meta/graph.rs` |
| Selector linkages | `Φuses:` from `angular_meta/graph.rs` |
| Cross-language symbols | `dictionary/workspace.rs` global symbol table |

**Cross-language note:** The global symbol table already unifies TS and C# symbols. A `UserService` appearing in both `UserService.cs` and `user.service.ts` is one node in the frequency table. PageRank inherits this — the shared symbol scores higher because it has more inbound edges across both language layers.

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/ranking/mod.rs` | Public surface: `SymbolRank`, `RankConfig`, `rank_workspace()` |
| Create | `src/ranking/pagerank.rs` | PageRank on IR + Angular graph; configurable damping (default 0.85), iterations (default 20) |
| Create | `src/ranking/seeds.rs` | Seed detector per language layer (TS `$e`, C# `public`, Angular decorators) |
| Create | `src/ranking/adaptive.rs` | `fidelity_from_rank()` — maps rank score to `Fidelity` enum |
| Modify | `src/compression/pipeline.rs` | After IR compile, run `rank_workspace()`, pass per-symbol fidelity to compaction |
| Modify | `src/mcp/heuristics.rs` | Accept `Option<SymbolRank>` as additional input to strategy selection |
| Modify | `src/mcp/workspace.rs` | Emit `§RANK` summary (top-20 by score) in workspace manifest header |
| Modify | `src/config.rs` | Add `intelligence.pagerank` config block (see Config section below) |
| Create | `src/tests/ranking/pagerank.rs` | PageRank on known graph with expected scores |
| Create | `src/tests/ranking/seeds.rs` | Seed detection for TS / C# / Angular — positive and negative cases |
| Create | `src/tests/ranking/adaptive.rs` | Fidelity boundary conditions (rank → fidelity thresholds) |
| Modify | `docs/ARCHITECTURE_OVERVIEW.md` | Add Intelligence Layer section + ranking module to module tree |
| Modify | `docs/ROADMAP.md` | Add R-29 Intelligence Layer as 🚧 in-progress |

### Key Structs

```rust
// src/ranking/mod.rs

pub struct SymbolRank {
    pub alias: String,          // α-alias from PathDictionary
    pub symbol: String,         // class / method name
    pub rank: f32,              // 0.0–1.0 normalized
    pub fidelity: Fidelity,     // derived: rank ≥ high_threshold → High, etc.
    pub language: Language,     // TS | CSharp — language-agnostic scoring
}

pub struct RankConfig {
    pub enabled: bool,
    pub damping: f32,           // default 0.85 (standard PageRank)
    pub iterations: u8,         // default 20 (converges on typical codebases)
    pub high_threshold: f32,    // default 0.7 → High fidelity
    pub medium_threshold: f32,  // default 0.4 → Medium fidelity
                                // below medium_threshold → Low fidelity
}
```

### Completion Criteria — Phase 1

**Functional**
- A TypeScript workspace produces a `§RANK` block in the manifest listing symbols by score.
- Exported symbols score higher than unexported symbols in the same file.
- A C# workspace seeds on `public` class members; `internal` members score lower.
- Angular `@Injectable({providedIn: 'root'})` services score higher than module-scoped.
- Per-symbol fidelity is applied: a rank-0.9 method in a medium-fidelity call compresses at High; a rank-0.2 utility in the same call compresses at Low.
- Cross-language: a symbol present in both TS and C# global symbol table scores higher than a symbol present in only one language.
- Non-Angular `.ts` and `.cs` files produce correct rankings with zero Φ markers required.

**Non-regression**
- All 945+ existing tests pass.
- `intelligence.pagerank.enabled: false` (default) produces byte-identical output to current pipeline.
- `cargo clippy --all-targets -- -D warnings` is clean.
- `provide_code_context` with no config change produces byte-identical output.

**Tests**
- At least 10 new tests: PageRank on known graph (3 tests), seed detection TS/C#/Angular (3 tests), fidelity boundary conditions (2 tests), manifest `§RANK` format (1 test), cross-language symbol boost (1 test).

**Effort:** ~2 days. **Risk:** Low (additive pass after existing IR compile; heuristics engine already accepts extensible inputs).

---

## Phase 2 — Blast Radius Analysis

### Goal

When a file changes, automatically identify which other files are **affected** by that change and include their compressed summaries in the delta output — proactively. The LLM gets the change AND the immediate impact surface without being asked.

**Before:** Delta sends what changed in file A.
**After:** Delta sends what changed in file A + skeleton summaries of depth-1 files that depend on A.

### How It Works

The `AngularGraph` (Phase 3) and IR graph already contain the edges. Blast radius traverses outward from the changed file.

```
UserService.ts changes
  → blast_radius(depth=1):
      UserCardComponent (Φinjects:[UserService@α2])
      UserProfileComponent (Φinjects:[UserService@α2])
  → delta output:
      [UserService delta]
      §IMPACT UserService@α2→[UserCardComponent@α3,UserProfileComponent@α7]
      [UserCardComponent — Low fidelity skeleton]
      [UserProfileComponent — Low fidelity skeleton]
```

**Cross-language blast radius** works automatically via the global symbol table:
```
UserService.cs changes
  → global symbol table: "UserService" also in user.service.ts
  → §IMPACT includes cross-language entry (UserService@α12 TS side)
```
Depth is configurable. Default `max_depth: 1` (direct dependents only). `max_depth: 0` disables blast radius entirely (byte-identical to current delta output).

### Integration with Existing Tools

Blast radius plugs into the existing `provide_code_context` zero-touch workflow — when `DeltaTransport` strategy is chosen by `heuristics.rs`, blast radius runs post-delta and appends the `§IMPACT` block. It also integrates with the `SqliteStore` sessions table to resolve workspace-aware cross-file relationships.

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/ranking/blast_radius.rs` | `blast_radius()` traversal — returns `Vec<AffectedFile>` sorted by proximity |
| Modify | `src/angular_meta/graph.rs` | Expose `dependents_of(alias) -> Vec<String>` (inverse of existing DI edges) |
| Modify | `src/mcp/tools.rs` | In `DeltaTransport` path: run blast radius, append `§IMPACT` + skeletons |
| Modify | `src/mcp/heuristics.rs` | Pass `blast_radius_enabled` flag through to delta path |
| Modify | `src/mcp/prompts.rs` | Add "Blast Radius" section to `SYSTEM_PROMPT` explaining `§IMPACT` |
| Modify | `src/config.rs` | Add `intelligence.blast_radius` config block |
| Create | `src/tests/ranking/blast_radius.rs` | Traversal tests (direct, transitive, cross-language, no-dep, depth=0) |
| Create | `src/test_files/blast_radius/` | Multi-file fixture: 1 service + 2 components + 1 C# controller |
| Modify | `docs/PERFORMANCE.md` | Blast radius token overhead table (depth 0 / 1 / 2) |

### Key Structs

```rust
// src/ranking/blast_radius.rs

pub struct AffectedFile {
    pub alias: String,                  // α-alias
    pub depth: u8,                      // 1 = direct dependent, 2 = transitive, etc.
    pub language: Language,             // TS | CSharp
    pub compressed_skeleton: String,    // Low-fidelity skeleton (already compressed)
    pub token_cost: usize,              // token count of skeleton
}

pub struct BlastResult {
    pub changed_alias: String,
    pub affected: Vec<AffectedFile>,    // sorted by depth asc, rank desc
}
```

### Completion Criteria — Phase 2

**Functional**
- A delta on `user.service.ts` injected into `UserCardComponent` produces `§IMPACT` listing `UserCardComponent@α3` at depth=1.
- Delta output includes Low-fidelity skeleton of each depth-1 affected file.
- A file with no dependents produces no `§IMPACT` block and zero overhead.
- Cross-language: change to `UserService.cs` whose name matches a TS Angular service produces a cross-language `§IMPACT` entry.
- `max_depth: 0` produces byte-identical delta output to Phase 1.
- `intelligence.blast_radius.enabled: false` (default) produces byte-identical output to Phase 1.

**Non-regression**
- All Phase 1 tests pass.
- `provide_code_context` without blast radius config change produces byte-identical output.

**Tests**
- At least 7 new tests: direct dependency, transitive (depth=2), no-dependency file, cross-language symbol, depth=0 no-op, multi-component fixture, `§IMPACT` format validation.

**Effort:** ~2 days. **Risk:** Medium (touches delta path in `tools.rs`, isolated behind config flag).

---

## Phase 3 — Token Budget + Knapsack Packing

### Goal

Allow callers to specify a token budget. The system packs the highest-ranked symbols that fit within the budget using greedy knapsack. Output is always ≤ budget tokens. No symbol is partially included — all-or-nothing packing preserves semantic integrity.

**Before:** Compress everything, return whatever size results.
**After:** Compress everything, rank everything, pack the best content into the budget.

This completes the picture for constrained agent loops where context windows are managed externally.

### How It Works

```
Input: workspace compressed IR + SymbolRank scores + budget=8192 tokens

Step 1: Sort symbols by rank descending
Step 2: Greedy pack — add symbols until budget exhausted
Step 3: Emit packed manifest with §BUDGET header
Step 4: Symbols that didn't fit → omitted entirely (not truncated, not summarized)
Result: output ≤ budget tokens, highest-value content prioritized
```

**Budget source options** (in priority order):
1. `token_budget` parameter passed directly to `compress_workspace` tool
2. `intelligence.budget.default_tokens` in `.clean-ctx.json`
3. Not set → no budget, full workspace output (current behavior)

### Integration with Existing Pipeline

Knapsack packing runs as the **final pass** after:
- IR compilation ✅
- Angular meta-layer ✅
- PageRank scoring (Phase 1) ✅
- Compression ✅
- Blast radius append (Phase 2) ✅

It operates on already-compressed token-counted output, so it never needs to re-compress.

### Scope

| Action | File | Purpose |
|--------|------|---------|
| Create | `src/ranking/knapsack.rs` | `pack_to_budget()` — greedy knapsack on `Vec<RankedSymbol>` |
| Create | `src/ranking/budget.rs` | `TokenBudget` struct — tracks limit / used / dropped |
| Modify | `src/mcp/workspace.rs` | Accept optional `token_budget`; run knapsack pass if set; emit `§BUDGET` header |
| Modify | `src/mcp/tools.rs` | Add `token_budget: Option<usize>` to `compress_workspace` tool schema |
| Modify | `src/config.rs` | Add `intelligence.budget.default_tokens` (null = disabled) |
| Create | `src/tests/ranking/knapsack.rs` | Packing tests (exact fit, overflow, single oversized symbol, budget > workspace) |
| Create | `src/tests/ranking/budget.rs` | Budget tracking tests |
| Modify | `docs/ARCHITECTURE_OVERVIEW.md` | Add knapsack pass to pipeline diagram |

### Key Structs

```rust
// src/ranking/budget.rs

pub struct TokenBudget {
    pub limit: usize,
    pub used: usize,
    pub dropped: usize,         // count of symbols dropped due to budget
    pub dropped_tokens: usize,  // tokens those symbols would have cost
}

// src/ranking/knapsack.rs

pub fn pack_to_budget(
    symbols: Vec<RankedSymbol>,   // already rank-sorted descending
    budget: &mut TokenBudget,
) -> Vec<RankedSymbol>
// Greedy: include if fits, skip if doesn't, never partial
```

### Completion Criteria — Phase 3

**Functional**
- `compress_workspace` with `token_budget: 8192` returns output ≤ 8192 tokens.
- Output contains highest-ranked symbols that fit, in rank order.
- No symbol is partially included.
- `§BUDGET 8192 used=7841 dropped=12 dropped_tokens=651` header emitted.
- Budget larger than total compressed workspace → full workspace, `§BUDGET` header with `dropped=0`.
- Budget = 0 → only `§BUDGET` header emitted (edge case, not an error).
- Without `token_budget` parameter and without config default → byte-identical output to Phase 2.

**Non-regression**
- All Phase 1 + 2 tests pass.
- `compress_workspace` without budget produces byte-identical output to Phase 2.

**Tests**
- At least 7 new tests: exact fit, overflow (drops lowest-ranked), single oversized symbol, budget > workspace size, budget = 0, no-budget no-op, `§BUDGET` header format.

**Effort:** ~1.5 days. **Risk:** Low (final pass on existing output, no pipeline changes required).

---

## `.clean-ctx.json` Schema Addition

```json
{
  "intelligence": {
    "pagerank": {
      "enabled": false,
      "damping": 0.85,
      "iterations": 20,
      "fidelity_thresholds": {
        "high": 0.7,
        "medium": 0.4
      }
    },
    "blast_radius": {
      "enabled": false,
      "max_depth": 1
    },
    "budget": {
      "default_tokens": null
    }
  }
}
```

All three sections default to **disabled / null** — existing users see zero change.

---

## Updated Pipeline Diagram

```
Source Files (TS + C#)
        │
        ▼
Tree-sitter AST Parse (existing)
        │
        ├──► Text Pipeline (existing)
        │         │
        │         ▼ Fidelity Filter + Opcode Encode
        │
        └──► IR Pipeline (existing)
                  │
                  ▼ CoreOp instructions → wire → string table → binary

        ▼
Angular Meta-Layer (existing, additive)
  detect → decorators → markers → bundler → graph
        │
        ▼
PageRank Scoring ◄────────────────────── Phase 1 (new)
  seeds (TS exports, C# public, Angular decorators)
  edges (IR graph + Φinjects + Φuses + global symbol table)
        │
        ▼
Compression (per-symbol adaptive fidelity) ◄── Phase 1 (modified)
  rank ≥ 0.7 → High fidelity
  rank ≥ 0.4 → Medium fidelity
  rank  < 0.4 → Low fidelity
        │
        ▼
Delta Transport (existing — text + IR)
        │
        ├──► Blast Radius Append ◄──────────── Phase 2 (new, delta path only)
        │         §IMPACT + depth-1 skeletons
        │
        ▼
Knapsack Budget Pack ◄─────────────────── Phase 3 (new, optional)
  sort by rank desc → greedy pack → §BUDGET header
        │
        ▼
Manifest Output
```

---

## Module Structure Addition

```
src/
├── ranking/                          # Intelligence Layer (new)
│   ├── mod.rs                        # Public API: SymbolRank, RankConfig, rank_workspace()
│   ├── pagerank.rs                   # PageRank algorithm on IR + Angular graph
│   ├── seeds.rs                      # Seed node detector (TS / C# / Angular)
│   ├── adaptive.rs                   # fidelity_from_rank() threshold mapping
│   ├── blast_radius.rs               # blast_radius() traversal + AffectedFile
│   ├── knapsack.rs                   # pack_to_budget() greedy packing
│   └── budget.rs                     # TokenBudget tracking
│
└── src/tests/ranking/                # Intelligence Layer tests
    ├── pagerank.rs
    ├── seeds.rs
    ├── adaptive.rs
    ├── blast_radius.rs
    ├── knapsack.rs
    └── budget.rs
```

---

## Cross-Phase Non-Goals (deliberately deferred)

| Item | Reason | Future roadmap |
|------|--------|---------------|
| TF-IDF within-file content ranking | Requires query context not always available at tool call time | R-31 |
| Attention-guided pruning (track what LLM referenced) | Requires response parsing, outside MCP scope | R-32 |
| Progressive summarization across sessions | Builds on persistence layer history | R-32 |
| Shell output compression (git, cargo, Angular CLI) | Separate scope from source compression | R-33 |
| React / Vue / NgRx meta-layer seeds | Blocked on those meta-layers shipping first | R-22b/c/d |
| RLE delta batching (Idea #5 from ULTRA_COMPACT_PLAN) | Lower priority, deferred in original plan | Future Phase V |

---

## Tracking

Each phase ends with:
- A passing test suite (`cargo test`)
- A clean linter (`cargo clippy --all-targets -- -D warnings`)
- A ROADMAP status update (📋 proposed → 🚧 in-progress → ✅ done)
- An entry in `CHANGELOG.md`

A phase is not complete until the user signs off on its completion criteria. We do not start the next phase until the current one is signed off.

---

## Capability Comparison

| Capability | ForgeIndex | LeanCTX | Clean-CTX today | Clean-CTX + IL |
|-----------|:---------:|:-------:|:---------------:|:--------------:|
| PageRank symbol scoring | ✅ | ❌ | ❌ | ✅ |
| Blast radius / impact analysis | ✅ | ❌ | ❌ | ✅ |
| Token budget knapsack packing | ✅ | ❌ | ❌ | ✅ |
| Huffman-coded symbol encoding | ❌ | ❌ | ✅ | ✅ |
| VarInt micro-opcodes | ❌ | ❌ | ✅ | ✅ |
| IR delta transport (field-level) | ❌ | ❌ | ✅ | ✅ |
| Text delta transport | ❌ | Partial | ✅ | ✅ |
| Angular framework meta-layer | ❌ | ❌ | ✅ | ✅ |
| NgRx / RxJS meta-layers | ❌ | ❌ | 🚧 planned | ✅ (post-meta) |
| Cross-language (TS + C#) symbol table | ❌ | ❌ | ✅ | ✅ |
| Adaptive per-symbol fidelity | ❌ | ❌ | ❌ | ✅ |
| Zero-touch workflow + heuristics | ❌ | Partial | ✅ | ✅ |
| SQLite cross-session persistence | ✅ | Partial | ✅ | ✅ |
| Zero network footprint | ✅ | Partial | ✅ | ✅ |
| Single static binary | ✅ | ✅ | ✅ | ✅ |
| Air-gap / DLP certified | ❌ | ❌ | ✅ | ✅ |
| CC0 public domain | ❌ | ❌ | ✅ | ✅ |