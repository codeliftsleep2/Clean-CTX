# Clean-CTX × codebase-memory-mcp Integration Plan

**Status:** 📋 Planning
**Created:** 2026-06-14
**Last updated:** 2026-06-14
**Roadmap ID:** R-35
**Target Release:** v0.2.0

---

## Executive Summary

Integrate [codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) (CBM) with Clean-CTX to provide graph intelligence across 158 languages while Clean-CTX provides deep token compression. The two tools are **complementary, not competing** — CBM answers "what code matters and how does it connect?" while Clean-CTX answers "how do I send this code to the LLM cheaply?"

**Key decision:** Communication via MCP JSON-RPC (loose coupling, no direct DB access). Both run as separate MCP servers.

---

## What Each Project Does

| Dimension | Clean-CTX | codebase-memory-mcp |
|-----------|-----------|---------------------|
| **Language** | Rust | C |
| **Core purpose** | Token compression (85-96% reduction) | Code intelligence (knowledge graph) |
| **Parsing** | Tree-sitter (TS, C#, Rust) | Tree-sitter (158 languages) + Hybrid LSP |
| **Data model** | IR + path/symbol dictionaries | Knowledge graph (nodes + edges) |
| **Query language** | None (file-level tools) | Cypher-like graph queries |
| **Persistence** | SQLite (deltas, sessions) | SQLite (nodes, edges, FTS5) |
| **MCP tools** | 18 (compression, delta, stats) | 14 (index, search, trace, architecture) |

### Why They're Complementary

```
Agent needs context about codebase
  → CBM: "UserOrder calls ProcessPayment which calls StripeAPI"
  → Clean-CTX: compress ProcessPayment.rs from 400 lines to 60 tokens
```

CBM's parsers build graphs. Clean-CTX's parsers compress code. They serve different purposes despite both using tree-sitter.

---

## Architecture

### Final Design

```
Agent (Cline/Claude Code/etc.)
     ↕ MCP protocol
Clean-CTX (Rust) ──JSON-RPC stdin/stdout──→ codebase-memory-mcp (C binary)
     │                                      │
     ├── Compression + IR + Delta           ├── Knowledge Graph (158 languages)
     ├── Angular Meta-Layer                 ├── Hybrid LSP type resolution
     └── Intelligence Layer (Hybrid)        └── Cypher-like queries
```

### The Integration Model

```
codebase-memory-mcp
    │
    ├─ 158-language parsing (tree-sitter + Hybrid LSP)
    ├─ Knowledge graph (nodes, edges, call chains)
    └─ Type resolution (cross-file, cross-package)
          │
          ↓ CBM graph output (structural relationships)
          │
Clean-CTX Language Layer (e.g., Java)
    │
    ├─ Tree-sitter parse (syntactic AST)
    ├─ CBM enrichment (types, call chains, relationships)
    ├─ Micro-opcode compression ($c, $m, $f, $a)
    ├─ Behavior markers (⊕guard, ⊕loop, ⊕⇒)
    ├─ Huffman coding + VarInt encoding
    └─ Fidelity-gated output
          │
          ↓ Compressed notation
          │
Clean-CTX Meta-Layer (e.g., Spring Boot)
    │
    ├─ Φ markers (@Service → Φsvc:, @Controller → Φctrl:)
    ├─ DI graph compression
    └─ Framework-specific patterns
          │
          ↓ Final compressed context
          │
LLM receives minimal token representation
```

### Critical Distinction: CBM Enriches, We Compress

| What CBM Does | What Our Language Layer Does |
|---------------|------------------------------|
| Discovers symbols and call graph | Compresses into `$c UserService` |
| Resolves types and call chains | Applies `⊕guard`, `⊕loop` markers |
| Maps cross-file relationships | Huffman-coded micro-opcodes |
| Provides Hybrid LSP type info | VarInt encoding, field-level delta |

**CBM is an input enrichment layer** — it feeds richer structural data into your language layer. Your language layer still owns compression. CBM doesn't replace it, it feeds it.

For Java specifically: CBM discovers the symbols and call graph, our Java language layer compresses them with the full micro-opcode stack. We still build the Java language layer — we just have a richer input to work with because CBM has already resolved types and call chains that tree-sitter alone would miss.

### Locked Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Communication** | MCP JSON-RPC | Loose coupling, both already speak MCP |
| **Runtime model** | Separate servers | Independent updates, no cross-language build deps |
| **DB access** | None (MCP calls only) | Schema changes won't break integration |
| **Launch model** | Lazy (on first query) | Don't slow down Clean-CTX startup |
| **Caching** | TTL-based in Clean-CTX | Avoid re-querying CBM for same data |
| **Intelligence Layer** | CBM seeds → Clean-CTX PageRank | Best of both worlds |

### Decision Details

#### 1. Binary Distribution — Separate Install

CBM ships as its own binary with its own release cadence. Clean-CTX provides a setup command:

```bash
clean-ctx setup --with-cbm
```

This command:
1. Checks if CBM is already installed
2. Downloads the correct version if not
3. Validates the binary hash
4. Configures the integration automatically

**Version compatibility:** Document minimum compatible CBM version in `.clean-ctx.json` as `cbm_min_version`.

#### 2. Error Handling — Degrade Gracefully

**Never error to the user because CBM crashed.** Clean-CTX existed before CBM and works perfectly without it.

```rust
pub enum CbmStatus {
    Available,
    Degraded(String),  // error message
    Unavailable,       // not installed
}
```

**Fallback behavior when CBM is unavailable:**
- Fall back to Clean-CTX's own TS/C#/Rust language layers
- For unsupported languages, emit `§CBM_UNAVAILABLE` warning in manifest
- Log crash to `.clean-ctx/proxy-logs` for diagnostics
- Surface in `context_stats` as degraded mode indicator

**Retry policy:** One retry with exponential backoff on transient failures, then degrade. Don't retry indefinitely — turns CBM problem into Clean-CTX hang.

#### 3. Cache Invalidation — Content-Hash Wiring

CBM uses content-hash-based re-indexing. Clean-CTX's SqliteStore tracks file hashes. Wire them together:

**On each `provide_code_context` call:**
1. Compare file hash against CBM's last-indexed hash via `detect_changes`
2. If CBM has re-indexed (hash differs), invalidate affected symbols in baseline cache

**Schema addition:**
```sql
ALTER TABLE sessions ADD COLUMN cbm_graph_version TEXT;
```

When `cbm_graph_version` changes between sessions, invalidate baseline for affected files and trigger delta recompression. Zero redundant work, clean invalidation chain.

---

## Implementation Phases

### Phase 0: Side-by-Side Configuration (30 min)

**What:** Document both servers as complementary. No code changes.

**Files to modify:**
- `README.md` — Add "With codebase-memory-mcp" section

**User config example:**
```json
{
  "mcpServers": {
    "clean-ctx": { "command": "path/to/clean-ctx.exe" },
    "codebase-memory": { "command": "path/to/codebase-memory-mcp.exe" }
  }
}
```

---

### Phase 1: MCP Client + Graph Bridge (5-8 days)

#### Module 1: `src/mcp/cbm_client.rs`

**Purpose:** Send JSON-RPC requests to CBM's stdin, read responses from stdout.

**Key structures:**
```rust
pub struct CbmClient {
    child: Child,                    // CBM subprocess
    stdin: BufWriter<ChildStdin>,   // Send requests
    stdout: BufReader<ChildStdout>, // Read responses
    request_id: AtomicU64,          // JSON-RPC request counter
}

impl CbmClient {
    pub fn launch(binary_path: &Path) -> Result<Self>;
    pub fn call_tool(&self, tool_name: &str, args: Value) -> Result<Value>;
    pub fn search_graph(&self, params: SearchGraphParams) -> Result<Vec<GraphNode>>;
    pub fn trace_path(&self, params: TracePathParams) -> Result<Vec<GraphEdge>>;
    pub fn detect_changes(&self, params: DetectChangesParams) -> Result<ChangeSet>;
    pub fn get_architecture(&self) -> Result<Architecture>;
    pub fn query_graph(&self, cypher: &str) -> Result<QueryResult>;
}
```

**Design notes:**
- Subprocess model (CBM launched by Clean-CTX)
- JSON-RPC 2.0 over stdin/stdout
- Lazy launch: CBM started on first graph query
- Timeout: 30s default for graph queries
- One retry with exponential backoff, then degrade

#### Module 2: `src/mcp/graph_bridge.rs`

**Purpose:** Translate CBM graph data into Clean-CTX concepts.

**Key structures:**
```rust
pub struct GraphBridge {
    client: CbmClient,
    cache: DashMap<String, CachedGraphData>,
    status: CbmStatus,
}

impl GraphBridge {
    pub fn get_symbol_importance(&self, project: &str) -> Result<HashMap<String, f64>>;
    pub fn get_blast_radius(&self, symbol: &str, depth: usize) -> Result<Vec<String>>;
    pub fn get_dead_code(&self, project: &str) -> Result<Vec<String>>;
    pub fn get_architecture(&self, project: &str) -> Result<ArchitectureOverview>;
    pub fn detect_changes(&self, diff: &str) -> Result<Vec<AffectedSymbol>>;
    pub fn status(&self) -> &CbmStatus;
}
```

**Caching strategy:**
- Symbol importance: per session (refresh on `index_repository`)
- Blast radius: per symbol + depth (invalidated on file change)
- Dead code: per session
- Architecture: per session

**Cache invalidation:**
- Store `cbm_graph_version` in sessions table
- On `provide_code_context`, compare hashes via `detect_changes`
- Invalidate affected symbols when version changes

#### Module 3: `src/mcp/cbm_config.rs`

**Config schema:**
```rust
pub struct CbmConfig {
    pub binary_path: Option<String>,    // Auto-detected if not set
    pub auto_launch: bool,              // default: true
    pub cache_ttl: u64,                 // default: 300 (5 min)
    pub enabled: bool,                  // default: true
    pub query_timeout_ms: u64,          // default: 30000 (30s)
    pub cbm_min_version: Option<String>,// Minimum compatible CBM version
}
```

**Binary auto-detection:**
1. Check `cbm_config.binary_path` if set
2. Check `PATH` for `codebase-memory-mcp`
3. Check common install locations
4. If not found, log warning and disable gracefully

#### Module 4: Integration Points

**`src/mcp/state.rs`:**
```rust
pub struct McpState {
    // ... existing ...
    pub graph_bridge: Option<GraphBridge>,  // None if CBM not available
}
```

**`src/mcp/tools.rs`** — New tool `smart_compress`:
```json
{
  "name": "smart_compress",
  "description": "Compress code based on a graph query. Finds relevant symbols via CBM, then compresses only those files.",
  "inputSchema": {
    "query": "Cypher-like query or natural language",
    "project": "CBM project name",
    "budget": "optional token budget"
  }
}
```

**`src/mcp/heuristics.rs`** — Feed CBM scores into fidelity:
- High importance (>0.8) → force High fidelity
- Medium (0.4-0.8) → use intent-based selection
- Low (<0.4) → force Low fidelity

---

### Phase 2: Intelligence Layer Integration (3-5 days)

**Prerequisite:** R-29 Intelligence Layer Phase 1

#### PageRank Enhancement

```rust
// src/intelligence/pagerank.rs
pub fn compute_pagerank(ir_graph: &IrGraph, cbm_bridge: &GraphBridge) -> HashMap<String, f64> {
    let ir_scores = pagerank_from_ir(ir_graph);      // 60% weight
    let cbm_scores = cbm_bridge.get_symbol_importance(project);  // 40% weight
    combine_scores(ir_scores, cbm_scores, 0.6, 0.4)
}
```

#### Blast Radius Enhancement

```rust
// src/intelligence/blast_radius.rs
pub fn get_blast_radius(symbol: &str, cbm_bridge: &GraphBridge) -> Vec<AffectedFile> {
    // CBM handles cross-file, cross-package, HTTP routes, async chains
    cbm_bridge.get_blast_radius(symbol, 2)
}
```

---

### Phase 3: Convenience Features (2-3 days)

#### `clean-ctx setup --with-cbm`

```bash
# Checks if CBM is installed
# Downloads correct version if not
# Validates binary hash
# Configures integration automatically
clean-ctx setup --with-cbm
```

#### `--with-cbm` Runtime Flag

```rust
// src/main.rs
#[derive(Parser)]
struct Args {
    #[arg(long)]
    with_cbm: bool,
}
```

#### Auto-detection in `provide_code_context`

Enrich compressed output with CBM metadata:
- Architecture context
- Dead code markers
- Importance scores
- `§CBM_UNAVAILABLE` warning if degraded

---

## Error Handling Matrix

| Scenario | Behavior | User Impact |
|----------|----------|-------------|
| CBM not installed | Disable CBM features, continue | No graph intelligence |
| CBM crashes mid-session | Degrade to `CbmStatus::Degraded`, log error | Reduced intelligence |
| CBM query timeout (30s) | One retry, then degrade | Brief delay |
| CBM returns error | Log, degrade for that query | Partial intelligence |
| CBM re-indexes | Detect via hash comparison, invalidate cache | Automatic recovery |
| CBM version incompatible | Warn at startup, disable | No graph intelligence |

---

## Cache Invalidation Flow

```
provide_code_context(file.rs)
  │
  ├─ 1. Compute file hash (existing)
  │
  ├─ 2. Query CBM detect_changes (if available)
  │     └─ Returns: list of changed symbols since last check
  │
  ├─ 3. Compare cbm_graph_version in sessions table
  │     └─ If changed → invalidate affected baselines
  │
  ├─ 4. Compress with valid baselines
  │
  └─ 5. Store updated cbm_graph_version
```

---

## File Summary

| File | Action | Est. Lines | Purpose |
|------|--------|------------|---------|
| `src/mcp/cbm_client.rs` | **New** | ~200 | MCP client for CBM |
| `src/mcp/graph_bridge.rs` | **New** | ~250 | Graph data translation |
| `src/mcp/cbm_config.rs` | **New** | ~80 | CBM config |
| `src/mcp/mod.rs` | Modify | +3 | Add modules |
| `src/mcp/state.rs` | Modify | +5 | Add GraphBridge |
| `src/mcp/tools.rs` | Modify | +25 | Add smart_compress |
| `src/mcp/tool_handlers.rs` | Modify | +60 | Implement handler |
| `src/mcp/server.rs` | Modify | +15 | --with-cbm flag |
| `src/config.rs` | Modify | +20 | Add CbmConfig |
| `src/main.rs` | Modify | +5 | CLI arg + setup |
| `README.md` | Modify | +30 | Document integration |

**Total:** ~530 new lines across 3 new files + 9 modified files

---

## Implementation Schedule

| Day | Focus | Deliverables |
|-----|-------|--------------|
| 1-2 | Foundation | `cbm_client.rs`, `cbm_config.rs`, config integration |
| 3-4 | Graph Bridge | `graph_bridge.rs`, `McpState` integration, cache invalidation |
| 5 | Smart Compress | `smart_compress` tool + handler |
| 6 | CLI Integration | `--with-cbm` flag, `setup --with-cbm`, auto-launch |
| 7 | Intelligence Layer | R-29 synergy hooks, PageRank integration |
| 8 | Polish | Tests, docs, README, error handling matrix |

---

## Post-CBM: Pilot Stack Roadmap

After CBM integration completes, the pilot stack coverage:

| Priority | Item | Depends On | Effort | Rationale |
|----------|------|------------|--------|-----------|
| **1** | CBM Integration (R-35) | Nothing | 5-8 days | Unlocks graph intelligence for all languages |
| **2** | Java Language Layer | CBM (enriched input) | 2-3 days | CBM resolves types/calls, we compress |
| **3** | Spring Boot Meta-Layer | Java Layer + CBM | 3-4 days | DI maps to Angular pattern, CBM provides Java graph |
| **4** | React Meta-Layer | CBM (JS/TS graph) | 3-4 days | Highest demand frontend |
| **5** | Redux Meta-Layer | React Meta-Layer | 2-3 days | Follows React naturally |
| **6** | NgRx Meta-Layer | Angular (existing) | 3-4 days | If not already shipped |

**Key insight:** CBM makes our Java language layer **more powerful** because CBM has already done the hard work of type resolution that tree-sitter alone can't do. We don't skip the Java layer — we build it with better inputs.

---

## What CBM Has That We Don't

- 158-language support (we have 3)
- Knowledge graph with Cypher queries
- Semantic search with embeddings
- Dead code detection
- Cross-service HTTP route linking
- Call graph traversal
- Hybrid LSP type resolution

## What We Have That CBM Doesn't

- Token compression (85-96% reduction)
- Delta transport (send only what changed)
- IR system with field-level deltas
- Angular Meta-Layer
- Token budget packing
- Multiple fidelity levels
- Heuristics engine
- Micro-opcode compression ($c, $m, $f, $a)
- Behavior markers (⊕guard, ⊕loop, ⊕⇒)
- Huffman coding + VarInt encoding

---

## References

- [codebase-memory-mcp GitHub](https://github.com/DeusData/codebase-memory-mcp)
- [CBM Research Paper](https://arxiv.org/abs/2603.27277)
- [Clean-CTX Intelligence Layer Plan](./INTELLIGENCE_LAYER_PLAN.md)
- [Clean-CTX Roadmap](./ROADMAP.md) (R-35)