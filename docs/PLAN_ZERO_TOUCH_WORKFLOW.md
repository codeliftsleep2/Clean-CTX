# Clean-CTX Zero-Touch Seamless Workflow — Implementation Plan

> **Status**: Ready for implementation  
> **Branch**: `feature/compilerIR_upgrade`  
> **Created**: 2026-06-09  
> **Persistence**: Deferred to next PR (plan designed to be persistence-ready)

---

## 1. Vision

After a one-time `clean-ctx init`, both developers and LLM agents experience near-zero friction. The system intelligently detects context, chooses the right compression strategy, enables Angular Meta-Layer automatically, uses deltas for follow-up edits, and is designed for seamless SQLite persistence across sessions and IDE restarts.

## 2. Core Principles

- **Automatic Everything**: File type detection, compress vs delta, fidelity selection, Angular Φ markers.
- **Persistence-Ready**: All trait boundaries and hook points are designed so SQLite persistence plugs in cleanly.
- **Natural Language First**: Agents use high-level intent; rich tool descriptions guide the LLM.
- **Single Smart Entry Point**: `provide_code_context` hides complexity.

---

## 3. Current Architecture (What Exists)

| Component | Location | Purpose |
|-----------|----------|---------|
| Compression pipeline | `src/compression/pipeline.rs` | `compress_file()` — full AST compression |
| Text deltas | `src/compression/text_delta.rs` | `TextDeltaComputer` — line-level delta transport |
| IR deltas | `src/ir/delta.rs`, `src/ir/replay.rs` | `DeltaComputer`, `ContextState` — IR-level delta |
| Angular detection | `src/angular_meta/detect.rs` | `is_angular_file()` — decorator-based detection |
| Config | `src/config.rs` | `CleanCtxConfig` — loaded from `.clean-ctx.json` |
| Session state | `src/mcp/state.rs` | `McpState` — dict, cache, config, IR context, text delta |
| Tool dispatch | `src/mcp/tools.rs`, `src/mcp/router.rs` | JSON-RPC tool routing |
| MCP server | `src/mcp/server.rs` | stdin/stdout JSON-RPC loop |

### Existing Tools (Keep All)
- `compress_code_context` — full compression
- `decompress_code_context` — expand compressed output
- `compress_workspace` — directory-level compression
- `diff_code_context` — AST-level diff
- `delta_code_context` — IR-level delta
- `delta_text_context` — text-level delta
- `apply_delta` — apply IR delta envelope

---

## 4. Implementation Steps

### Step 1: Extend `CleanCtxConfig` with Smart Heuristics + Persistence Placeholder

**File**: `src/config.rs`

#### New Types to Add

```rust
/// Smart defaults for intent-based fidelity selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartDefaults {
    #[serde(default = "default_sd_refactor")]
    pub refactor: String,       // "high"
    #[serde(default = "default_sd_overview")]
    pub overview: String,       // "low"
    #[serde(default = "default_sd_debug")]
    pub debug: String,          // "medium"
    #[serde(default = "default_sd_edit")]
    pub edit: String,           // "low"
    #[serde(default = "default_sd_implement")]
    pub implement: String,      // "medium"
}

/// Heuristics configuration for automatic decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeuristicsConfig {
    #[serde(default = "default_large_file_threshold")]
    pub large_file_threshold: usize,       // 300 lines
    #[serde(default)]
    pub force_high_fidelity: Vec<String>,  // ["*.service.ts", "*.component.ts", "*.guard.ts"]
    #[serde(default = "default_true")]
    pub use_angular_meta: bool,
}

/// Persistence configuration (placeholder — SQLite layer coming next)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub auto_save: bool,
    #[serde(default = "default_max_history_days")]
    pub max_history_days: u32,
    #[serde(default = "default_db_path")]
    pub db_path: String,
}
```

#### Fields to Add to `CleanCtxConfig`

```rust
pub struct CleanCtxConfig {
    // ... existing fields ...
    
    /// Smart defaults for intent-based fidelity selection
    #[serde(default)]
    pub smart_defaults: SmartDefaults,
    
    /// Heuristics configuration
    #[serde(default)]
    pub heuristics: HeuristicsConfig,
    
    /// Auto-detect Angular files and enable Meta-Layer
    #[serde(default = "default_true")]
    pub auto_angular: bool,
    
    /// Automatically use deltas for follow-up edits
    #[serde(default = "default_true")]
    pub auto_delta: bool,
    
    /// Persistence configuration (placeholder for SQLite)
    #[serde(default)]
    pub persistence: PersistenceConfig,
}
```

#### New Method

```rust
impl CleanCtxConfig {
    /// Generate the default config file content for `clean-ctx init`
    pub fn generate_default_content() -> String {
        // Returns the full recommended .clean-ctx.json with all fields
    }
}
```

#### Default `.clean-ctx.json` Output

```json
{
  "enabled": true,
  "defaultFidelity": "medium",
  "autoAngular": true,
  "autoDelta": true,
  "persistence": {
    "enabled": false,
    "autoSave": true,
    "maxHistoryDays": 30,
    "dbPath": ".clean-ctx/persistence.db"
  },
  "heuristics": {
    "largeFileThreshold": 300,
    "forceHighFidelity": ["*.service.ts", "*.component.ts", "*.guard.ts"],
    "useAngularMeta": true
  },
  "smartDefaults": {
    "refactor": "high",
    "overview": "low",
    "debug": "medium",
    "edit": "low",
    "implement": "medium"
  }
}
```

---

### Step 2: Create the Heuristics Engine

**New file**: `src/mcp/heuristics.rs`

#### Types

```rust
/// What strategy should provide_code_context use?
pub enum ContextStrategy {
    /// First time seeing this file — full compression
    FullCompress,
    /// File seen before in this session — use delta transport
    DeltaTransport,
    // FUTURE: RestoreFromDB — load baseline from SQLite, replay deltas
}

/// The resolved decision for a provide_code_context call
pub struct ContextDecision {
    pub fidelity: Fidelity,
    pub strategy: ContextStrategy,
    pub is_angular: bool,
    pub source_line_count: usize,
}
```

#### Main Function

```rust
pub fn decide(
    file_path: &str,
    explicit_intent: Option<&str>,
    explicit_fidelity: Option<&str>,
    config: &CleanCtxConfig,
    text_delta_state: &TextDeltaComputer,
    ir_context: &ContextState,
) -> ContextDecision
```

#### Decision Logic

1. **Read file** to get line count (or use cached info from `McpState.source_cache`)
2. **Check if baseline exists**: `text_delta_state.has_baseline(alias)` → if yes, `DeltaTransport`; if no, `FullCompress`
3. **Resolve fidelity** (priority order):
   - Explicit `fidelity` arg → use it
   - Explicit `intent` arg → map via `config.smart_defaults` (e.g., "refactor" → "high")
   - Extension in `config.heuristics.force_high_fidelity` → "high"
   - Large file (> `config.heuristics.large_file_threshold`) → "low"
   - Default → `config.default_fidelity`
4. **Angular detection**: If `config.auto_angular` and extension is `.ts`/`.js`:
   - Read source, call `is_angular_file(source)`
   - If true, set `is_angular = true`

---

### Step 3: Create the `ContextStore` Trait (Persistence Boundary)

**New file**: `src/mcp/context_store.rs`

This is the critical persistence-readiness piece. Define a trait that abstracts context storage, with an in-memory implementation now and SQLite later.

#### Trait Definition

```rust
use crate::compression::Fidelity;

/// Metadata about a stored compression context
pub struct StoredContextMeta {
    pub file_path: String,
    pub fidelity: Fidelity,
    pub version: u64,
    pub is_angular: bool,
    pub source_hash: String,
    pub created_at: std::time::SystemTime,
}

/// Trait for persisting and restoring compression contexts.
///
/// Current: InMemoryContextStore (session-only, lives in McpState)
/// Future: SqliteContextStore (survives IDE restarts, backed by persistence.db)
pub trait ContextStore {
    /// Save a full compression context (baseline)
    fn save_context(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        compressed_output: &str,
        ir_blobs: Option<&[u8]>,
        source_hash: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Load the latest context for a file, if any
    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>>;

    /// Check if a context exists for this file (fast path)
    fn has_context(&self, file_path: &str) -> bool;

    /// Append a delta to the context's history
    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Get delta history count for a context
    fn delta_count(&self, context_id: &str) -> usize;
}
```

#### In-Memory Implementation

```rust
pub struct InMemoryContextStore {
    contexts: HashMap<String, StoredContextMeta>,
    deltas: HashMap<String, Vec<DeltaRecord>>,
}

struct DeltaRecord {
    payload: Vec<u8>,
    edit_type: Option<String>,
    applied_at: std::time::SystemTime,
}
```

This wraps the existing in-memory state so the rest of the code talks to the trait, not the concrete type.

---

### Step 4: Add `provide_code_context` Tool

**File**: `src/mcp/tools.rs`

#### Tool Definition (in `tool_list()`)

```rust
serde_json::json!({
    "name": "provide_code_context",
    "description": "Automatically provides the best possible compressed context for a file. This is the RECOMMENDED single entry point for any file-related coding task. First call performs full compression; subsequent calls automatically use delta transport for minimal token usage. Auto-detects Angular files and enables Meta-Layer with Φ markers. Chooses optimal fidelity based on file characteristics and intent. Use this tool for ANY coding task involving code context.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Path to the source file." },
            "intent": { "type": "string", "description": "Optional intent: 'edit', 'refactor', 'overview', 'debug', 'implement'. Controls fidelity selection.", "enum": ["edit", "refactor", "overview", "debug", "implement"] },
            "workspaceRoot": { "type": "string", "description": "Optional workspace root for relative paths." }
        },
        "required": ["filePath"]
    }
})
```

#### Dispatch

Add to `dispatch_tools_call`:
```rust
"provide_code_context" => handle_provide_code_context(id, params, state),
```

#### Handler Logic

```rust
fn handle_provide_code_context(id: &Value, params: &Value, state: &mut McpState) {
    // 1. Extract filePath, intent, workspaceRoot
    // 2. Resolve absolute path (handle relative paths)
    // 3. Check exclusion via state.config.is_excluded()
    // 4. Call heuristics::decide() → ContextDecision
    // 5. Based on decision.strategy:
    //    - FullCompress → delegate to existing compression logic + store baseline
    //    - DeltaTransport → delegate to existing delta_text_context logic
    // 6. Return rich response with _meta:
    //    - fidelity used
    //    - strategy chosen
    //    - angular_detected
    //    - line_count
    //    - version
    // 7. Persistence hook: if config.persistence.enabled → store.save_context()
}
```

---

### Step 5: Add `restore_context` Tool

**File**: `src/mcp/tools.rs`

```rust
serde_json::json!({
    "name": "restore_context",
    "description": "Explicitly restores compressed context for a file. Forces full re-compression from on-disk source, clearing any in-memory delta baselines. Use when you need a guaranteed fresh context state.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Path to the source file." },
            "fidelity": { "type": "string", "description": "Compression fidelity: 'low', 'medium', 'high'." }
        },
        "required": ["filePath"]
    }
})
```

Handler: clears text delta baselines + IR baselines for the file, then performs full compression.

---

### Step 6: Add `context_history` Tool

**File**: `src/mcp/tools.rs`

```rust
serde_json::json!({
    "name": "context_history",
    "description": "View compression history and savings for tracked files. Shows per-file version count, delta hit rate, and estimated token savings.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Optional: specific file. If omitted, shows all tracked files." }
        }
    }
})
```

Handler: queries text delta versions + IR versions. Returns summary.

---

### Step 7: Add `context_stats` Dashboard Tool + `dashboard` Prompt

**Files**: `src/mcp/tools.rs`, `src/mcp/state.rs`, `src/mcp/prompts.rs`

#### 7a. Session Stats Tracker

**New file**: `src/mcp/session_stats.rs`

A lightweight in-memory accumulator that tracks token savings across the session. Every tool call that performs compression or delta updates the stats.

```rust
/// Per-file stats entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileStats {
    pub file_path: String,
    pub raw_tokens: usize,
    pub compressed_tokens: usize,
    pub savings_pct: f64,
    pub version: u64,
    pub delta_count: usize,
    pub fidelity: String,
    pub is_angular: bool,
    pub strategy: String,  // "full" or "delta"
}

/// Session-level stats accumulator
#[derive(Debug, Clone, Default)]
pub struct SessionStats {
    /// Per-file stats (keyed by file path)
    files: HashMap<String, FileStats>,
    /// Total raw tokens across all files
    total_raw_tokens: usize,
    /// Total compressed tokens across all files
    total_compressed_tokens: usize,
    /// Number of full compressions performed
    full_compress_count: usize,
    /// Number of delta operations performed
    delta_count: usize,
    /// Session start time
    started_at: std::time::SystemTime,
}

impl SessionStats {
    pub fn new() -> Self { ... }
    
    /// Record a compression event (full or delta)
    pub fn record_compression(
        &mut self,
        file_path: &str,
        raw_tokens: usize,
        compressed_tokens: usize,
        fidelity: &str,
        is_angular: bool,
        strategy: &str,
    ) { ... }
    
    /// Record a delta event
    pub fn record_delta(&mut self, file_path: &str) { ... }
    
    /// Get per-file stats
    pub fn file_stats(&self, file_path: &str) -> Option<&FileStats> { ... }
    
    /// Get all file stats
    pub fn all_file_stats(&self) -> &HashMap<String, FileStats> { ... }
    
    /// Get session summary
    pub fn summary(&self) -> SessionSummary { ... }
    
    /// Get session duration in seconds
    pub fn session_duration_secs(&self) -> u64 { ... }
}

/// Session summary for dashboard display
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionSummary {
    pub total_files: usize,
    pub total_raw_tokens: usize,
    pub total_compressed_tokens: usize,
    pub total_savings_pct: f64,
    pub full_compress_count: usize,
    pub delta_count: usize,
    pub delta_hit_rate: f64,  // delta_count / (full_compress_count + delta_count)
    pub session_duration_secs: u64,
    pub avg_savings_pct: f64,
}
```

#### 7b. Wire into `McpState`

Add to `McpState`:
```rust
pub struct McpState {
    // ... existing fields ...
    /// Session-level stats accumulator for the dashboard
    pub session_stats: SessionStats,
}
```

#### 7c. Update `provide_code_context` handler

After compression/delta, call `state.session_stats.record_compression(...)` with the token counts from `calculate_savings()`. This is the primary data source for the dashboard.

#### 7d. Add `context_stats` MCP Tool

```rust
serde_json::json!({
    "name": "context_stats",
    "description": "View the Clean-CTX dashboard: token savings, compression stats, and session metrics. Shows per-file breakdown and session summary. Use this to monitor compression efficiency at any time.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Optional: specific file to show stats for. If omitted, shows full session dashboard." },
            "format": { "type": "string", "description": "Output format: 'text' (human-readable, default) or 'json' (structured).", "enum": ["text", "json"] }
        }
    }
})
```

**Handler** `handle_context_stats`:
- If `filePath` provided → return per-file stats
- If no `filePath` → return full session dashboard

**Text format output** (default):
```
═══════════════════════════════════════════════════════════════
  Clean-CTX Dashboard — Session Stats
═══════════════════════════════════════════════════════════════
  Session Duration: 12m 34s
  Files Tracked: 6
  Total Raw Tokens: 12,450
  Total Compressed Tokens: 3,210
  Total Savings: 74.2%
  Full Compressions: 6
  Delta Operations: 14
  Delta Hit Rate: 70.0%
───────────────────────────────────────────────────────────────
  Per-File Breakdown:
  ┌──────────────────────────────────────┬───────┬───────┬───────┬───────┐
  │ File                                 │ Raw   │ Comp  │ Save% │ Deltas│
  ├──────────────────────────────────────┼───────┼───────┼───────┼───────┤
  │ UserManagementService.ts             │ 2,450 │   670 │ 72.7% │     3 │
  │ rate-limit.service.ts                │ 1,890 │   520 │ 72.5% │     5 │
  │ user-card.component.ts               │ 3,200 │   890 │ 72.2% │     2 │
  │ ...                                  │       │       │       │       │
  └──────────────────────────────────────┴───────┴───────┴───────┴───────┘
═══════════════════════════════════════════════════════════════
```

**JSON format output**:
```json
{
  "session": {
    "duration_secs": 754,
    "total_files": 6,
    "total_raw_tokens": 12450,
    "total_compressed_tokens": 3210,
    "total_savings_pct": 74.2,
    "full_compress_count": 6,
    "delta_count": 14,
    "delta_hit_rate": 70.0
  },
  "files": [
    {
      "file_path": "UserManagementService.ts",
      "raw_tokens": 2450,
      "compressed_tokens": 670,
      "savings_pct": 72.7,
      "version": 4,
      "delta_count": 3,
      "fidelity": "low",
      "is_angular": true,
      "strategy": "delta"
    }
  ]
}
```

#### 7e. Add `dashboard` MCP Prompt

Add to `src/mcp/prompts.rs`:

```rust
pub const DASHBOARD_PROMPT: &str = r#"
You have access to the Clean-CTX Dashboard. To view token savings and compression stats:

- Call `context_stats` with no arguments to see the full session dashboard.
- Call `context_stats` with a `filePath` to see stats for a specific file.
- Use `format: "json"` for structured data, or `format: "text"` for human-readable output.

The dashboard shows:
- Session duration and file count
- Total raw vs compressed tokens and savings percentage
- Full compression vs delta operation counts
- Per-file breakdown with version history and delta counts
- Delta hit rate (how often deltas were used instead of full re-compression)
"#;
```

Register in `prompt_list()`:
```rust
serde_json::json!({
    "name": "dashboard",
    "description": "View the Clean-CTX token savings dashboard. Shows session stats, per-file breakdown, and compression efficiency metrics.",
    "arguments": []
})
```

#### 7f. Persistence-Awareness

When persistence arrives:
- `SessionStats` can be serialized to SQLite for cross-session history
- `context_stats` gains a `--history` flag to show historical sessions
- The dashboard can show "today vs yesterday" comparisons
- The `SessionSummary` struct maps directly to the `sessions` table in the SQLite schema

---

### Step 8: Add `clean-ctx init` CLI Subcommand

**File**: `src/main.rs`

Add clap argument parsing:

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clean-ctx", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Clean-CTX config in the current directory
    Init,
}
```

`main()` logic:
- No args → run MCP server (current behavior)
- `clean-ctx init` → write `.clean-ctx.json` + create `.clean-ctx/` directory

The `.clean-ctx/` directory creation is persistence-ready — when SQLite arrives, the DB file goes here.

---

### Step 9: Add Tests

**New test files**:
- `src/tests/mcp/heuristics.rs` — decision engine tests
- `src/tests/mcp/provide_code_context.rs` — integration tests
- `src/tests/mcp/context_store.rs` — `InMemoryContextStore` round-trip tests
- `src/tests/mcp/session_stats.rs` — dashboard stats tests
- `src/tests/config_smart_defaults.rs` — new config fields

#### Test Cases

| Test | Description |
|------|-------------|
| `test_first_call_full_compress` | First call → FullCompress strategy |
| `test_second_call_delta` | Second call → DeltaTransport strategy |
| `test_intent_refactor_high_fidelity` | Intent "refactor" → High fidelity |
| `test_intent_overview_low_fidelity` | Intent "overview" → Low fidelity |
| `test_large_file_low_fidelity` | Large file (> 300 lines) → Low fidelity |
| `test_angular_detection` | Angular file detected → is_angular = true |
| `test_init_creates_files` | `clean-ctx init` creates correct files |
| `test_restore_clears_baselines` | `restore_context` clears baselines |
| `test_context_history_returns_data` | `context_history` returns correct data |
| `test_context_store_round_trip` | InMemoryContextStore save/load round-trip |
| `test_session_stats_record` | SessionStats records compression events |
| `test_session_stats_summary` | SessionStats summary computes correctly |
| `test_context_stats_text_format` | Dashboard text format output |
| `test_context_stats_json_format` | Dashboard JSON format output |

---

## 5. File Change Summary

| File | Action | Description |
|------|--------|-------------|
| `src/config.rs` | **Modify** | Add `SmartDefaults`, `HeuristicsConfig`, `PersistenceConfig`, `generate_default_content()` |
| `src/mcp/heuristics.rs` | **New** | `ContextDecision`, `ContextStrategy`, `decide()` |
| `src/mcp/context_store.rs` | **New** | `ContextStore` trait + `InMemoryContextStore` |
| `src/mcp/session_stats.rs` | **New** | `SessionStats`, `FileStats`, `SessionSummary` — dashboard accumulator |
| `src/mcp/tools.rs` | **Modify** | Add 4 new tools + handlers (`provide_code_context`, `restore_context`, `context_history`, `context_stats`) |
| `src/mcp/prompts.rs` | **Modify** | Add `dashboard` prompt |
| `src/main.rs` | **Modify** | Add clap subcommand parsing for `init` |
| `src/lib.rs` | **Modify** | Add `pub(crate) mod context_store`, `pub(crate) mod session_stats` |
| `src/mcp/state.rs` | **Modify** | Add `context_store: InMemoryContextStore` + `session_stats: SessionStats` to `McpState` |
| `src/tests/mcp/heuristics.rs` | **New** | Heuristics tests |
| `src/tests/mcp/provide_code_context.rs` | **New** | Integration tests |
| `src/tests/mcp/context_store.rs` | **New** | Store tests |
| `src/tests/mcp/session_stats.rs` | **New** | Dashboard stats tests |
| `src/tests/config_smart_defaults.rs` | **New** | Config tests |

---

## 6. Persistence Integration Points (For Next PR)

This section documents exactly where the persistence layer will plug in, so the next implementer knows the contract.

| Component | Current | Future (Persistence) |
|-----------|---------|---------------------|
| `McpState` | `text_delta: TextDeltaComputer`, `ir_context: ContextState` | Add `store: Box<dyn ContextStore>` (default `InMemoryContextStore`, swap to `SqliteContextStore`) |
| `ContextStore` trait | `InMemoryContextStore` (HashMap-backed) | `SqliteContextStore` (rusqlite, WAL mode) |
| `ContextStrategy` | `FullCompress`, `DeltaTransport` | Add `RestoreFromDB` variant |
| `decide()` | Checks `text_delta.has_baseline()` | Also checks `store.has_context()` + `store.load_latest()` |
| `provide_code_context` handler | Compresses/deltas in-memory | After operation: `store.save_context()` + `store.append_delta()` |
| `restore_context` handler | Clears in-memory baselines | Also loads from DB: `store.load_latest()` → replay deltas |
| `context_history` tool | In-memory version counts | Also queries `store.delta_count()` + DB history |
| `CleanCtxConfig` | `persistence: PersistenceConfig` (parsed, unused) | Read by `SqliteStore::open()` to find DB path |
| `clean-ctx init` | Creates `.clean-ctx/` dir | DB file (`persistence.db`) goes here |
| `Cargo.toml` | No rusqlite | `rusqlite = { version = "0.32", features = ["bundled"] }` behind `persistence` feature flag |

### SQLite Schema (For Reference)

```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS contexts (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    fidelity INTEGER NOT NULL,
    ir_binary BLOB NOT NULL,
    pretty_text TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS deltas (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    context_id TEXT REFERENCES contexts(id) ON DELETE CASCADE,
    edit_sequence INTEGER NOT NULL,
    delta_payload BLOB NOT NULL,
    raw_tokens INTEGER,
    delta_tokens INTEGER,
    edit_type TEXT,
    description TEXT,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS symbols (
    symbol_id TEXT PRIMARY KEY,
    context_id TEXT REFERENCES contexts(id),
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    phi_markers TEXT,
    last_seen DATETIME
);

CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    workspace_root TEXT NOT NULL,
    active_contexts TEXT,
    last_active DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_contexts_file ON contexts(file_path);
CREATE INDEX idx_deltas_context ON deltas(context_id, edit_sequence);
CREATE INDEX idx_symbols_context ON symbols(context_id);
```

---

## 7. Implementation Order

Execute steps in this exact order:

1. **Step 1** — Config changes (foundation for everything else)
2. **Step 2** — Heuristics engine (depends on config)
3. **Step 3** — ContextStore trait (persistence boundary)
4. **Step 4** — `provide_code_context` tool (depends on heuristics + context store)
5. **Step 5** — `restore_context` tool (depends on context store)
6. **Step 6** — `context_history` tool (depends on context store)
7. **Step 7** — `context_stats` dashboard tool + `dashboard` prompt (depends on session stats)
8. **Step 8** — `clean-ctx init` CLI (independent, can be done anytime)
9. **Step 9** — Tests (after all tools are implemented)

---

## 8. Key Design Decisions

1. **Backward compatible**: All existing tools remain. `provide_code_context` is the recommended entry point but old tools still work.

2. **No persistence yet**: `ContextDecision` and baselines live in `McpState` (in-memory). The `.clean-ctx/` directory is created by `init` but the SQLite layer is explicitly deferred.

3. **Reuse existing handlers**: `provide_code_context` internally delegates to existing compression/delta logic rather than duplicating it. The new tool is pure orchestration.

4. **Trait-based persistence boundary**: `ContextStore` trait with `InMemoryContextStore` now, `SqliteContextStore` later. Zero changes to tool handlers when persistence arrives.

5. **Dashboard is dual-access**: The `context_stats` tool provides both text (human-readable) and JSON (structured) output. The `dashboard` prompt gives agents a natural language entry point. Both read from the same `SessionStats` accumulator.

6. **Stats accumulate automatically**: Every `provide_code_context` call records token savings into `SessionStats`. No manual tracking needed. The dashboard is always up-to-date.

7. **Config is immutable per session**: The config is loaded once at startup. Edits require server restart. This is intentional for simplicity.

---

## 9. Agent System Prompt Guidance

Add to the agent's system prompt:

```
You have access to Clean-CTX — an intelligent, high-efficiency context engine.

For any file-related task:
- Always start by calling `provide_code_context` with the filePath and a brief intent.
- Do NOT paste raw file content.
- Trust the tool to automatically handle:
  - Full compression on first use
  - Delta updates on follow-up edits
  - Angular Meta-Layer detection
  - Optimal fidelity selection

This enables extremely long, high-fidelity editing sessions with minimal token usage.
```

---

## 10. Example Natural Language Flows

### Flow 1: Simple Edit
```
User: "Add optional fields support to getUserById in UserManagementService.ts"
Agent: Calls provide_code_context(filePath: "UserManagementService.ts", intent: "edit")
  → System auto-restores baseline + applies delta
  → Gets optimized context (low fidelity)
  → Proceeds with edit
```

### Flow 2: Refactor
```
User: "Refactor this Angular service for rate limiting"
Agent: Calls provide_code_context(filePath: "rate-limit.service.ts", intent: "refactor")
  → System detects Angular → enables Meta-Layer + Φ markers
  → High fidelity selected via smart_defaults
  → Gets optimized context with Angular awareness
  → Proceeds with refactor
```

### Flow 3: Overview
```
User: "Give me an overview of the codebase"
Agent: Calls provide_code_context(filePath: "main.ts", intent: "overview")
  → Low fidelity selected
  → Gets maximally compressed context
  → Proceeds with overview