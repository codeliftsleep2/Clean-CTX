# Clean-CTX SQLite Persistence Layer — Agent Handoff Document

> **Branch**: `feature/compilerIR_upgrade`  
> **Target**: Add local SQLite persistence so baselines, IR data, symbol tables, and delta histories survive IDE restarts, crashes, and multi-chat workflows.  
> **Philosophy**: Local-first, air-gapped, optional feature flag, zero breaking changes to existing tools.  
> **Created**: 2026-06-10

---

## Table of Contents

1. [Architecture Review & Current State](#1-architecture-review--current-state)
2. [Gaps & Risks (Critical Context)](#2-gaps--risks-critical-context)
3. [Kickoff: Cargo.toml + Module Scaffolding](#3-kickoff-cargotoml--module-scaffolding)
4. [Phase 1: Core Schema + SqliteStore](#4-phase-1-core-schema--sqlitestore)
5. [Phase 2: Integration Hooks into Hot Paths](#5-phase-2-integration-hooks-into-hot-paths)
6. [Phase 3: Restore & Replay from DB](#6-phase-3-restore--replay-from-db)
7. [Phase 4: MCP Tools for Session Management](#7-phase-4-mcp-tools-for-session-management)
8. [Phase 5: Config, CLI, Tests, Simulation](#8-phase-5-config-cli-tests-simulation)
9. [Phase 6: Documentation](#9-phase-6-documentation)
10. [Integration Points Summary Table](#10-integration-points-summary-table)
11. [Commit Sequence](#11-commit-sequence)

---

## 1. Architecture Review & Current State

### 1.1 What Already Exists (Persistence-Ready)

The codebase was intentionally designed for persistence but the SQLite layer was deferred. Here's what's already in place:

| Component | File | Status |
|-----------|------|--------|
| `PersistenceConfig` struct | `src/config.rs:87-121` | ✅ Parsed, **unused** |
| `ContextStore` trait | `src/mcp/context_store.rs:69-105` | ✅ Defines `save_context`, `load_latest`, `has_context`, `append_delta`, `delta_count`, `clear_file` |
| `InMemoryContextStore` | `src/mcp/context_store.rs:112-232` | ✅ HashMap-backed implementation. **Tested** by `src/tests/mcp/context_store.rs` |
| `McpState` | `src/mcp/state.rs:31-141` | ✅ Has `context_store: InMemoryContextStore` field already wired |
| `ContextState` (IR replay) | `src/ir/replay.rs:179-357` | ✅ Full in-memory state machine with `load_ir()`, `apply()`, `render_pretty()`, etc. |
| `DeltaComputer` | `src/ir/delta.rs:111-169` | ✅ Computes instruction-level deltas between `CompiledIR` snapshots |
| `compact_encode()` / `compact_decode()` | `src/ir/delta.rs:361-482` | ✅ Compact delta wire format for smaller BLOB storage |
| `binary_wire::encode()` / `decode()` | `src/ir/binary_wire.rs:168-517` | ✅ Binary encoding for `CompiledIR` — ideal for `ir_binary BLOB` column |
| `GlobalSymbolTable` | `src/ir/symbol_table.rs:63-283` | ✅ Serializable, no `save_to_db()` yet |
| `TextDeltaComputer` | `src/compression/text_delta.rs` | ✅ Line-level delta, already in `McpState` |
| `SessionStats` | `src/mcp/session_stats.rs` | ✅ Session stats accumulator |
| `.clean-ctx/` dir creation | `src/main.rs:25-27` | ✅ Created by `clean-ctx init` |
| Default config template | `src/main.rs:55-79` | ✅ Includes `"persistence": { "enabled": false, ... }` |

### 1.2 What the Existing `PLAN_ZERO_TOUCH_WORKFLOW.md` Documents

The existing plan at `docs/PLAN_ZERO_TOUCH_WORKFLOW.md` has a **Section 6 — "Persistence Integration Points (For Next PR)"** (lines 694-758) that documents:
- Which components change for persistence
- The SQLite schema (4 tables: `contexts`, `deltas`, `symbols`, `sessions`)
- WAL mode + foreign keys

**This handoff document supersedes that section** with concrete implementation detail, fixes its gaps, and adds the 9 architecture risks discovered during codebase exploration.

---

## 2. Gaps & Risks (Critical Context)

### 🚨 GAP 1 — Circular Dependency Risk (Architecture-Blocking)

The `ContextStore` trait lives in `src/mcp/context_store.rs` (inside the `mcp` module). The `SqliteStore` would logically go in `src/ir/persistence.rs` (inside the `ir` module). **But `ir` does NOT depend on `mcp` — `mcp` depends on `ir`.** Putting a `SqliteStore` in `ir` that implements a trait from `mcp` creates a circular dependency.

**✅ FIX**: Place `SqliteStore` as a second implementation alongside `InMemoryContextStore` in the `mcp` module. Create `src/mcp/sqlite_store.rs`. The `ir` module stays pure.

**Action**: Add `pub(crate) mod sqlite_store;` to `src/mcp/mod.rs`. Do NOT add a `persistence` module to `src/ir/`.

### 🚨 GAP 2 — `binary_wire::decode()` Destroys Identity (Data-Loss Risk)

From `src/ir/binary_wire.rs:512-516`:
```rust
Ok(CompiledIR {
    file_id: "bin".to_string(),   // ← ALWAYS "bin"
    instructions,
    version: VERSION as u64,      // ← ALWAYS 1
})
```

If you `binary_wire::encode()` → store BLOB → `binary_wire::decode()` on load, the `file_id` and `version` are **lost**. The `delta.rs` replay system requires these for version-chain validation.

**✅ FIX**: Do NOT rely on the BLOB for identity. Store `file_id` and `version` as **separate columns** in the `contexts` table. On load:
```rust
let mut ir = binary_wire::decode(&blob)?;
ir.file_id = row.file_id;    // restore from DB column
ir.version = row.version;    // restore from DB column
```

**Schema already has these**: The `contexts` table has `file_path TEXT NOT NULL` and the IR's identity can be derived from `file_path` + querying deltas table for version count.

**ALSO**: `binary_wire::encode()` already skips `file_id` and `version` (it only encodes instructions + string table). The current `encode()` implementation (line 168-280) never writes file_id or version into the binary stream. So the BLOB has **never** carried identity — this isn't a regression, it's just something we must handle.

### 🚨 GAP 3 — `PersistenceConfig` Exists But Is Never Read (Dead Code)

`src/config.rs:87-121` defines `PersistenceConfig` with `enabled`, `db_path`, etc. But `McpState::new()` (`src/mcp/state.rs:75`) never reads `config.persistence`. The field is parsed from JSON then ignored.

**✅ FIX**: Add initialization in `McpState::new()`:
```rust
let persistence_store = if config.persistence.enabled {
    let db_path = Path::new(&config.persistence.db_path);
    // Resolve relative to workspace root or cwd
    match SqliteStore::open(db_path) {
        Ok(store) => {
            eprintln!("[clean-ctx] Persistence enabled: {}", db_path.display());
            Some(store)
        }
        Err(e) => {
            eprintln!("[clean-ctx] WARNING: Failed to open DB: {e}");
            None
        }
    }
} else {
    None
};
```

### ⚠️ GAP 4 — No Error Type for DB Operations (Low Risk)

The `ContextStore` trait returns `Result<_, Box<dyn std::error::Error>>`. `rusqlite::Error` implements `std::error::Error` but not `Send` in all cases. With the `bundled` feature, `rusqlite::Error` should be fine, but the agent should verify.

**✅ FIX**: Add `thiserror = "2"` to `Cargo.toml` for a dedicated `PersistenceError` enum, OR just use `Box<dyn std::error::Error>` and add `impl From<rusqlite::Error> for Box<dyn std::error::Error>`.

**Recommendation**: Keep it simple. Use `Box<dyn std::error::Error>` everywhere. The `ContextStore` trait already uses it. No new error type needed.

### ⚠️ GAP 5 — No Two-Tier Cache (Performance Risk)

There's `LocalStateCache` in `src/cache.rs` for content-hash caching. Persistence should coordinate:
- **Hot path**: Check `LocalStateCache` first (O(1), no I/O)
- **Cold path**: If cache miss, check `SqliteStore` (O(1) indexed query)
- **Write-through**: On save to DB, also update `LocalStateCache`

**✅ FIX**: In Phase 2, when hooking `save_context` into the hot path, also update the in-memory cache:
```rust
if let Some(store) = &state.persistence_store {
    store.save_context(...)?;
}
// Also update in-memory cache
state.cache.store_baseline_hash(&cache_key, &source_hash);
```

### ⚠️ GAP 6 — `SessionStats` Rehydration (Observability Gap)

`SessionStats` is in-memory only. On restart after DB restore, the dashboard shows empty stats. The `sessions` and `contexts` tables can rebuild them.

**✅ FIX**: Add `rebuild_stats_from_db()` to `SqliteStore` that queries:
```sql
SELECT file_path, fidelity, 
    (SELECT COUNT(*) FROM deltas WHERE context_id = contexts.id) as delta_count
FROM contexts
```
Then populates `SessionStats`. Call on startup if persistence is enabled.

### ⚠️ GAP 7 — Angular Φ Markers Not Wired to Symbols Table (Schema Gap)

The SQLite schema has `symbols.phi_markers TEXT` column. But `GlobalSymbolTable::SymbolEntry` (`src/ir/symbol_table.rs:43-56`) has no `phi_markers` field. The Angular meta-layer extracts Φ data in `angular_meta/decorators.rs`, but there's no bridge to the symbols table.

**✅ FIX**: Two options:
1. **(Recommended)**: Remove `phi_markers` from the schema for now. Mark as "future" in docs. Ship without it.
2. Add `phi_markers: Option<String>` to `SymbolEntry` and wire it through. Only do this if Θ-time permits.

### ⚠️ GAP 8 — `clean-ctx init` Defaults Persistence to `false` (Policy)

`src/main.rs:61` sets `"persistence": { "enabled": false }`. The goal says "local-first, air-gapped." The default should become `true` **last** — only after Phases 1-5 are stable and tested.

**✅ FIX**: Change to `"enabled": true` only at the very end of the project, after all phases are verified.

### ⚠️ GAP 9 — No Pruning Schedule (Operational Gap)

`PersistenceConfig.max_history_days: 30` exists, but no code prunes old deltas. The `purge_old_deltas` MCP tool in Phase 4 should be manually invocable.

**✅ FIX**: Document that auto-pruning at server startup is a future enhancement. Ship `purge_old_deltas` as a manual MCP tool only.

---

## 3. Kickoff: Cargo.toml + Module Scaffolding

### 3.1 Add Dependencies

```toml
# In [dependencies] section:
rusqlite = { version = "0.32", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }

# Optional: for cleaner error types
thiserror = "2"

# Optional: for dev-only testing
[dev-dependencies]
tempfile = "3.27.0"  # Already present
```

### 3.2 Feature Flag (Optional)

Add a `persistence` feature gate if you want it optional at compile time:
```toml
[features]
default = []
persistence = ["rusqlite", "chrono"]
```

If you use features, gate all `SqliteStore` code behind `#[cfg(feature = "persistence")]`.

### 3.3 Module Scaffolding

```rust
// src/mcp/mod.rs — add this line:
pub(crate) mod sqlite_store;
```

Then create `src/mcp/sqlite_store.rs` with the `SqliteStore` struct. No changes needed in `src/ir/mod.rs` (circular dep avoidance — see Gap 1).

---

## 4. Phase 1: Core Schema + SqliteStore

### 4.1 File: `src/mcp/sqlite_store.rs`

This is the main deliverable of Phase 1. It implements the existing `ContextStore` trait from `src/mcp/context_store.rs`.

```rust
use std::path::Path;
use rusqlite::{Connection, params};
use crate::compression::Fidelity;
use crate::mcp::context_store::{ContextStore, StoredContextMeta};

/// SQLite-backed implementation of [`ContextStore`].
///
/// Uses WAL mode for concurrent read/write safety.
/// Schema versioning via a `_schema_version` pragma table.
pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    /// Open (or create) the SQLite database at the given path.
    /// Automatically creates the `.clean-ctx/` directory if it doesn't exist.
    pub fn open(db_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let conn = Connection::open(db_path)?;
        
        // WAL mode for concurrent safety
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }
    
    /// Run schema migrations. Idempotent.
    fn migrate(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS _schema_version (
                version INTEGER PRIMARY KEY
            );
        ")?;
        
        let current_version: i32 = self.conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM _schema_version", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);
        
        if current_version < 1 {
            self.conn.execute_batch("
                CREATE TABLE IF NOT EXISTS contexts (
                    id TEXT PRIMARY KEY,
                    file_path TEXT NOT NULL,
                    content_hash TEXT NOT NULL UNIQUE,
                    fidelity INTEGER NOT NULL,
                    ir_binary BLOB NOT NULL,
                    pretty_text TEXT,
                    symbol_table_json TEXT,
                    metadata TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
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
                    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                );
                
                CREATE TABLE IF NOT EXISTS symbols (
                    symbol_id TEXT PRIMARY KEY,
                    context_id TEXT REFERENCES contexts(id),
                    name TEXT NOT NULL,
                    kind TEXT NOT NULL,
                    phi_markers TEXT,
                    last_seen TEXT
                );
                
                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    workspace_root TEXT NOT NULL,
                    active_contexts TEXT,
                    last_active TEXT NOT NULL DEFAULT (datetime('now'))
                );
                
                CREATE INDEX IF NOT EXISTS idx_contexts_file ON contexts(file_path);
                CREATE INDEX IF NOT EXISTS idx_deltas_context ON deltas(context_id, edit_sequence);
                CREATE INDEX IF NOT EXISTS idx_symbols_context ON symbols(context_id);
                
                INSERT INTO _schema_version (version) VALUES (1);
            ")?;
        }
        
        Ok(())
    }
}
```

### 4.2 Implement `ContextStore` Trait for `SqliteStore`

```rust
impl ContextStore for SqliteStore {
    fn save_context(
        &mut self,
        file_path: &str,
        fidelity: Fidelity,
        compressed_output: &str,
        ir_blobs: Option<&[u8]>,
        source_hash: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let id = format!("ctx-{}", source_hash);  // deterministic ID from content hash
        let fid = fidelity as i32;
        let ir_binary = ir_blobs.unwrap_or(&[]);
        
        self.conn.execute(
            "INSERT OR REPLACE INTO contexts (id, file_path, content_hash, fidelity, ir_binary, pretty_text, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, datetime('now'))",
            params![id, file_path, source_hash, fid, ir_binary, compressed_output],
        )?;
        
        Ok(id)
    }
    
    fn load_latest(
        &self,
        file_path: &str,
    ) -> Result<Option<StoredContextMeta>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_path, content_hash, fidelity, pretty_text, created_at
             FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1"
        )?;
        
        let mut rows = stmt.query(params![file_path])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let fp: String = row.get(1)?;
            let hash: String = row.get(2)?;
            let fid: i32 = row.get(3)?;
            let _pretty: Option<String> = row.get(4)?;
            let created: String = row.get(5)?;
            
            // Parse Fidelity from i32
            let fidelity = match fid {
                0 => Fidelity::Low,
                1 => Fidelity::Medium,
                2 => Fidelity::High,
                _ => Fidelity::Low,
            };
            
            // Parse created_at string to SystemTime (approximate)
            let created_at = std::time::SystemTime::now();  // TODO: parse datetime string
            
            Ok(Some(StoredContextMeta {
                file_path: fp,
                fidelity,
                version: 0,  // TODO: count deltas for version
                is_angular: false,  // TODO: load from metadata
                source_hash: hash,
                created_at,
            }))
        } else {
            Ok(None)
        }
    }
    
    fn has_context(&self, file_path: &str) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM contexts WHERE file_path = ?1 LIMIT 1",
                params![file_path],
                |_| Ok(()),
            )
            .is_ok()
    }
    
    fn append_delta(
        &mut self,
        context_id: &str,
        delta_payload: &[u8],
        edit_type: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get the next edit_sequence
        let next_seq: i32 = self.conn
            .query_row(
                "SELECT COALESCE(MAX(edit_sequence), 0) + 1 FROM deltas WHERE context_id = ?1",
                params![context_id],
                |row| row.get(0),
            )
            .unwrap_or(1);
        
        self.conn.execute(
            "INSERT INTO deltas (context_id, edit_sequence, delta_payload, edit_type, applied_at)
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            params![context_id, next_seq, delta_payload, edit_type],
        )?;
        
        // Update the context's updated_at
        self.conn.execute(
            "UPDATE contexts SET updated_at = datetime('now') WHERE id = ?1",
            params![context_id],
        )?;
        
        Ok(())
    }
    
    fn delta_count(&self, context_id: &str) -> usize {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM deltas WHERE context_id = ?1",
                params![context_id],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0) as usize
    }
    
    fn clear_file(&mut self, file_path: &str) {
        // DELETE CASCADE should handle deltas
        if let Err(e) = self.conn.execute(
            "DELETE FROM contexts WHERE file_path = ?1",
            params![file_path],
        ) {
            eprintln!("[clean-ctx] Failed to clear file from DB: {e}");
        }
    }
}
```

### 4.3 Key Design Decisions for Phase 1

- **Context ID**: Use content-hash-derived ID (`ctx-{sha256_prefix}`) for deterministic, idempotent saves.
- **Version tracking**: Derive from `COUNT(deltas) + 1` instead of storing a separate column.
- **Pretty text**: Store in `contexts.pretty_text` column. This is the compressed text output (not raw source).
- **Binary BLOB**: `ir_binary` stores `binary_wire::encode()` output. On load, reconstruct with `binary_wire::decode()` + restore `file_id`/`version` from DB columns.
- **Metadata column**: For extensibility (angular flag, heuristics decision, etc.). JSON blob.

---

## 5. Phase 2: Integration Hooks into Hot Paths

### 5.1 Wire into `McpState` Initialization

In `src/mcp/state.rs`, modify `McpState::new()`:

```rust
use crate::mcp::sqlite_store::SqliteStore;

pub struct McpState {
    // ... existing fields ...
    /// Optional SQLite persistence store
    pub persistence_store: Option<SqliteStore>,
}

impl McpState {
    pub fn new(config: CleanCtxConfig) -> Self {
        let persistence_store = if config.persistence.enabled {
            let db_path = std::path::Path::new(&config.persistence.db_path);
            match SqliteStore::open(db_path) {
                Ok(store) => {
                    eprintln!("[clean-ctx] Persistence enabled: {}", db_path.display());
                    Some(store)
                }
                Err(e) => {
                    eprintln!("[clean-ctx] WARNING: Failed to open persistence DB: {e}");
                    None
                }
            }
        } else {
            None
        };
        
        Self {
            // ... existing fields ...
            persistence_store,
        }
    }
}
```

### 5.2 Wire `provide_code_context` (FullCompress Branch)

In `src/mcp/tools.rs`, inside `handle_provide_code_context`, after successful compression (around line 961-978):

**BEFORE** (current code, line 976-978):
```rust
if let Ok(ir) = ir_result {
    state.ir_context.load_ir(ir);
}
```

**AFTER**:
```rust
if let Ok(ir) = ir_result {
    state.ir_context.load_ir(ir.clone());
    
    // Persistence hook: save baseline context + IR binary
    if let Some(store) = &mut state.persistence_store {
        let source_hash = sha2::Sha256::digest(source.as_bytes());
        let hash_hex = format!("{:x}", source_hash);
        let ir_binary = crate::ir::binary_wire::encode(&ir);
        
        if let Err(e) = store.save_context(
            &resolved_path,
            decision.fidelity,
            &output_text,  // the compressed output
            Some(&ir_binary),
            &hash_hex,
        ) {
            eprintln!("[clean-ctx] WARNING: Failed to persist context: {e}");
        }
    }
}
```

### 5.3 Wire `delta_text_context` / Delta Transport Branch

In the `DeltaTransport` branch (around line 1017-1073), after successful delta computation:

```rust
if let Some(store) = &mut state.persistence_store {
    if let Some(delta) = &delta {
        let context_id = format!("ctx-{}", path_alias);  // or query from DB
        let delta_bytes = serde_json::to_vec(delta)?;
        if let Err(e) = store.append_delta(&context_id, &delta_bytes, Some("edit")) {
            eprintln!("[clean-ctx] WARNING: Failed to persist delta: {e}");
        }
    }
}
```

### 5.4 Wire `handle_delta_code_context`

In `src/mcp/tools.rs`, in `handle_delta_code_context`, after computing the delta (around line 596-641):
```rust
if let Some(store) = &mut state.persistence_store {
    if let Some(delta) = &delta_result {
        let delta_bytes = serde_json::to_vec(delta)?;
        let context_id = format!("ctx-{}", &file_alias);
        if let Err(e) = store.append_delta(&context_id, &delta_bytes, Some("ir_delta")) {
            eprintln!("[clean-ctx] WARNING: Failed to persist IR delta: {e}");
        }
    }
}
```

### 5.5 Wire `GlobalSymbolTable` Persistence

Add methods to `src/ir/symbol_table.rs`:

```rust
impl GlobalSymbolTable {
    /// Serialize all symbols to a JSON string for DB storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.all_symbols())
    }
    
    /// Rebuild symbol table from JSON (e.g., loaded from DB).
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let entries: Vec<SymbolEntry> = serde_json::from_str(json)?;
        let mut table = Self::new();
        for entry in entries {
            table.register(entry.alias, entry.original, entry.kind, &entry.file_id);
        }
        Ok(table)
    }
}
```

Then in the `provide_code_context` handler, after IR compilation:
```rust
// Persist symbol table
if let Some(store) = &mut state.persistence_store {
    if let Ok(sym_json) = state.ir_context.symbol_table().to_json() {
        // Store symbol_json in contexts.metadata or a separate query
    }
}
```

**Note**: The current `ContextState` in `replay.rs` doesn't own a `GlobalSymbolTable`. It's stored in the `IRCompiler`. You'll need to extract it. For Phase 2, it's acceptable to persist the symbol table as JSON in `contexts.metadata` without a dedicated query path.

### 5.6 Two-Tier Cache Coordination (Gap 5 Fix)

```rust
// In FullCompress handler, after compression:
let source_hash = state.cache.compute_hash(source.as_bytes());
state.cache.store_baseline_hash(&cache_key, &source_hash);

// Before DB save, check cache first:
if !state.cache.has_baseline(&cache_key) {
    // Cold path — load from DB
    if let Some(store) = &state.persistence_store {
        if let Ok(Some(meta)) = store.load_latest(&resolved_path) {
            state.cache.store_baseline_hash(&cache_key, &meta.source_hash);
        }
    }
}
```

---

## 6. Phase 3: Restore & Replay from DB

### 6.1 Add `.load_context_with_deltas()` to `SqliteStore`

```rust
impl SqliteStore {
    /// Load a context and replay deltas up to a target version.
    /// Returns the reconstructed CompiledIR and the final version reached.
    pub fn load_context_with_deltas(
        &self,
        file_path: &str,
        target_sequence: Option<u32>,
    ) -> Result<Option<(CompiledIR, u32)>, Box<dyn std::error::Error>> {
        // 1. Load baseline IR from contexts table
        let mut stmt = self.conn.prepare(
            "SELECT ir_binary, content_hash, fidelity FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1"
        )?;
        
        let row = match stmt.query_row(params![file_path], |row| {
            let blob: Vec<u8> = row.get(0)?;
            let hash: String = row.get(1)?;
            let fid: i32 = row.get(2)?;
            Ok((blob, hash, fid))
        }) {
            Ok(r) => r,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(e.into()),
        };
        
        // 2. Decode baseline IR
        let mut ir = crate::ir::binary_wire::decode(&row.0)?;
        ir.file_id = file_path.to_string();
        
        // 3. Load deltas up to target_sequence
        let max_seq = target_sequence.unwrap_or(u32::MAX);
        let mut delta_stmt = self.conn.prepare(
            "SELECT edit_sequence, delta_payload FROM deltas 
             WHERE context_id = (SELECT id FROM contexts WHERE file_path = ?1 ORDER BY updated_at DESC LIMIT 1)
             AND edit_sequence <= ?2
             ORDER BY edit_sequence"
        )?;
        
        let delta_rows = delta_stmt.query_map(params![file_path, max_seq], |row| {
            let seq: i32 = row.get(0)?;
            let payload: Vec<u8> = row.get(1)?;
            Ok((seq, payload))
        })?;
        
        // 4. Build a ContextState and replay deltas
        let mut context_state = crate::ir::replay::ContextState::new();
        context_state.load_ir(ir);
        
        for delta_result in delta_rows {
            let (_seq, payload) = delta_result?;
            let delta: crate::ir::delta::IRDelta = serde_json::from_slice(&payload)?;
            context_state.apply(delta)?;
        }
        
        // 5. Reconstruct CompiledIR from context state
        let instructions = context_state.get_ir(file_path)
            .cloned()
            .unwrap_or_default();
        let version = context_state.file_version(file_path).unwrap_or(1);
        
        let instructions_ops: Vec<crate::ir::opcodes::CoreOp> = instructions
            .iter()
            .filter_map(|t| crate::ir::wire::tuple_to_op(t))
            .collect();
        
        let final_ir = crate::ir::compiler::CompiledIR {
            file_id: file_path.to_string(),
            instructions: instructions_ops,
            version,
        };
        
        Ok(Some((final_ir, version as u32)))
    }
    
    /// Rebuild SessionStats from DB (Gap 6 fix).
    pub fn rebuild_stats(&self) -> Result<crate::mcp::session_stats::SessionStats, Box<dyn std::error::Error>> {
        let mut stats = crate::mcp::session_stats::SessionStats::new();
        
        let mut stmt = self.conn.prepare(
            "SELECT c.file_path, c.fidelity, 
                    (SELECT COUNT(*) FROM deltas d WHERE d.context_id = c.id) as delta_count
             FROM contexts c"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let path: String = row.get(0)?;
            let fid: i32 = row.get(1)?;
            let dc: i32 = row.get(2)?;
            Ok((path, fid, dc))
        })?;
        
        for row in rows {
            let (path, _fid, dc) = row?;
            // Record as a full compression with placeholder values
            stats.record_compression(
                &path, 100, 30, "low", false, "restored"
            );
            // Record each delta
            for _ in 0..dc {
                stats.record_delta(&path);
            }
        }
        
        Ok(stats)
    }
}
```

### 6.2 Update `handle_restore_context` for DB

In `src/mcp/tools.rs`, `handle_restore_context`:

```rust
fn handle_restore_context(id: &Value, params: &Value, state: &mut McpState) {
    // ... existing path resolution + exclusion check ...
    
    // Try DB restore first
    if let Some(store) = &state.persistence_store {
        match store.load_context_with_deltas(&resolved_path, None) {
            Ok(Some((ir, version))) => {
                // Successfully restored from DB
                state.ir_context.load_ir(ir);
                let pretty = state.ir_context.render_pretty(&path_alias, fidelity)
                    .unwrap_or_default();
                
                send_response(&serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "content": [{ "type": "text", "text": pretty }],
                        "_meta": {
                            "fidelity": format!("{:?}", fidelity).to_lowercase(),
                            "strategy": "restore_from_db",
                            "version": version,
                        }
                    }
                }));
                return;
            }
            Ok(None) => { /* No DB entry — fall through to full re-compress */ }
            Err(e) => {
                eprintln!("[clean-ctx] DB restore failed: {e}");
                // Fall through to full re-compress
            }
        }
    }
    
    // Fallback: full re-compression from disk (existing code)
    // ... existing handle_restore_context logic ...
}
```

### 6.3 Session Restore on MCP Startup

In `server.rs` or `state.rs`, after initializing `McpState`, optionally restore:

```rust
// At startup, if persistence is enabled, rebuild stats
if let Some(store) = &state.persistence_store {
    match store.rebuild_stats() {
        Ok(stats) => {
            state.session_stats = stats;
            eprintln!("[clean-ctx] Restored stats for {} files", stats.all_file_stats().len());
        }
        Err(e) => eprintln!("[clean-ctx] WARNING: Failed to rebuild stats: {e}"),
    }
}
```

---

## 7. Phase 4: MCP Tools for Session Management

### 7.1 New Tool: `save_context`

**Definition** (in `tool_list()`):
```rust
serde_json::json!({
    "name": "save_context",
    "description": "Explicitly save current in-memory context to the persistence DB. Useful for manual checkpointing before risky edits.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Optional: specific file to save. If omitted, saves all tracked files." }
        }
    }
})
```

**Handler**:
```rust
fn handle_save_context(id: &Value, params: &Value, state: &mut McpState) {
    let store = match &state.persistence_store {
        Some(s) => s,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32603, "message": "Persistence not enabled. Set persistence.enabled=true in .clean-ctx.json" }
            }));
            return;
        }
    };
    
    // Iterate over tracked files and save each
    let file_ids = state.ir_context.file_ids();
    let mut saved = 0;
    for file_id in &file_ids {
        // Get IR from context state
        if let Some(instructions) = state.ir_context.get_ir(file_id) {
            let ir = CompiledIR {
                file_id: file_id.clone(),
                instructions: instructions.iter()
                    .filter_map(|t| crate::ir::wire::tuple_to_op(t))
                    .collect(),
                version: state.ir_context.file_version(file_id).unwrap_or(1),
            };
            let ir_binary = binary_wire::encode(&ir);
            // ... save to DB ...
            saved += 1;
        }
    }
    
    send_response(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [{ "type": "text", "text": format!("Saved {} context(s) to DB.", saved) }]
        }
    }));
}
```

### 7.2 New Tool: `list_sessions`

**Definition**:
```rust
serde_json::json!({
    "name": "list_sessions",
    "description": "List all persistence sessions stored in the DB. Shows workspace roots, active contexts, and last active timestamps.",
    "inputSchema": {
        "type": "object",
        "properties": {}
    }
})
```

**Handler**: Query `sessions` table, return results as text.

### 7.3 New Tool: `replay_history`

**Definition**:
```rust
serde_json::json!({
    "name": "replay_history",
    "description": "Replay deltas from the DB for a file up to a specific edit sequence. Useful for recovering state after a crash.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "filePath": { "type": "string", "description": "Path to the source file." },
            "targetSequence": { "type": "integer", "description": "Optional: replay up to this edit sequence. If omitted, replays all." },
            "fidelity": { "type": "string", "description": "Optional: output fidelity. Default: 'low'." }
        },
        "required": ["filePath"]
    }
})
```

**Handler**: Use `store.load_context_with_deltas(file_path, target_sequence)` from Phase 3.

### 7.4 New Tool: `purge_old_deltas`

**Definition**:
```rust
serde_json::json!({
    "name": "purge_old_deltas",
    "description": "Purge old delta history from the persistence DB. Use to free space or trim history.",
    "inputSchema": {
        "type": "object",
        "properties": {
            "days": { "type": "integer", "description": "Delete deltas older than this many days. Default: 30." },
            "filePath": { "type": "string", "description": "Optional: specific file to purge. If omitted, purges all files." }
        }
    }
})
```


**Handler**:
```rust
fn handle_purge_old_deltas(id: &Value, params: &Value, state: &mut McpState) {
    let days = params["arguments"]["days"].as_u64().unwrap_or(30);
    let file_path = params["arguments"]["filePath"].as_str();
    
    let store = match &state.persistence_store {
        Some(s) => s,
        None => {
            send_response(&serde_json::json!({
                "jsonrpc": "2.0", "id": id,
                "error": { "code": -32603, "message": "Persistence not enabled." }
            }));
            return;
        }
    };
    
    // Build WHERE clause
    let (where_clause, deleted) = if let Some(fp) = file_path {
        // Purge specific file
        let count = store.delta_count_for_file(fp);
        if let Err(e) = store.conn.execute(
            "DELETE FROM deltas WHERE context_id IN (SELECT id FROM contexts WHERE file_path = ?1)",
            params![fp],
        ) { eprintln!("[clean-ctx] Purge error: {e}"); }
        (format!("file {}", fp), count)
    } else {
        // Purge all deltas older than `days`
        let count: i32 = store.conn.query_row(
            "SELECT COUNT(*) FROM deltas WHERE applied_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
            |row| row.get(0),
        ).unwrap_or(0);
        if let Err(e) = store.conn.execute(
            "DELETE FROM deltas WHERE applied_at < datetime('now', ?1)",
            params![format!("-{} days", days)],
        ) { eprintln!("[clean-ctx] Purge error: {e}"); }
        (format!("deltas older than {} days", days), count as usize)
    };
    
    send_response(&serde_json::json!({
        "jsonrpc": "2.0", "id": id,
        "result": {
            "content": [{
                "type": "text",
                "text": format!("Purged {} delta(s) for {}.", deleted, where_clause)
            }]
        }
    }));
}
```

### 7.5 Wire Into Dispatch

Add to `dispatch_tools_call` in `src/mcp/tools.rs`:
```rust
"save_context" => handle_save_context(id, params, state),
"list_sessions" => handle_list_sessions(id, params, state),
"replay_history" => handle_replay_history(id, params, state),
"purge_old_deltas" => handle_purge_old_deltas(id, params, state),
```

---

## 8. Phase 5: Config, CLI, Tests, Simulation

### 8.1 Wire `PersistenceConfig` to Actual Behavior (Gap 3 Fix)

**File**: `src/mcp/state.rs` — Already covered in Section 5.1. Ensure `McpState::new()` reads `config.persistence.enabled` and opens `SqliteStore` at the configured `db_path`.

**File**: `src/config.rs` — The `PersistenceConfig` struct is already defined. No schema changes needed. Verify the JSON serialization matches the expectations:
```json
{
  "persistence": {
    "enabled": false,
    "autoSave": true,
    "maxHistoryDays": 30,
    "dbPath": ".clean-ctx/persistence.db"
  }
}
```

### 8.2 Update `clean-ctx init` Default to `enabled: true` (Gap 8 Fix)

**File**: `src/main.rs:60-64` — Change **last**, after all other phases pass testing:
```rust
// BEFORE:
"persistence": { "enabled": false, ... }

// AFTER:
"persistence": { "enabled": true, ... }
```

### 8.3 Update `fifty_edit_simulation.rs`

**File**: `examples/fifty_edit_simulation.rs`

Add a `--persist` flag. After the 50-edit loop:

```rust
// After simulation with persistence:
if let Some(store) = &state.persistence_store {
    // 1. Save final state
    for (file_id, _) in state.ir_context.file_ids() {
        let ir_binary = binary_wire::encode(&ir);
        store.save_context(&file_id, fidelity, &pretty, Some(&ir_binary), &source_hash)?;
    }
    
    // 2. Simulate crash — drop everything
    let saved_store = state.persistence_store.take();
    state.ir_context = ContextState::new();
    
    // 3. Restore from DB — verify round-trip
    if let Some(store) = &saved_store {
        let (restored_ir, _) = store.load_context_with_deltas(&file_path, None)?
            .expect("Should restore from DB");
        
        // 4. Verify equality with original
        assert_eq!(restored_ir.instructions, ir.instructions,
            "IR round-trip failed: instructions differ after restore");
    }
}
```

### 8.4 Property-Based Round-Trip Test

**New file**: `src/tests/ir/persistence.rs` (or in-memory SQLite test)

```rust
#[test]
fn test_sqlite_round_trip() {
    use crate::ir::compiler::CompiledIR;
    use crate::ir::opcodes::CoreOp;
    use crate::mcp::sqlite_store::SqliteStore;
    use crate::mcp::context_store::ContextStore;
    use crate::compression::Fidelity;
    use tempfile::tempdir;
    
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    
    // Create store
    let mut store = SqliteStore::open(&db_path).unwrap();
    
    // Create sample IR
    let ir = CompiledIR {
        file_id: "test_file.ts".to_string(),
        instructions: vec![
            CoreOp::DefClass("C1".into(), "MyClass".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "doSomething".into()),
            CoreOp::Return("M1".into(), "$v".into()),
        ],
        version: 1,
    };
    
    let ir_binary = crate::ir::binary_wire::encode(&ir);
    
    // Save
    let ctx_id = store.save_context(
        "test_file.ts",
        Fidelity::Low,
        "compressed output",
        Some(&ir_binary),
        "abc123",
    ).unwrap();
    
    // Verify has_context
    assert!(store.has_context("test_file.ts"));
    
    // Load latest metadata
    let meta = store.load_latest("test_file.ts").unwrap().unwrap();
    assert_eq!(meta.file_path, "test_file.ts");
    
    // Load with deltas and verify reconstruction
    let (loaded_ir, _) = store.load_context_with_deltas("test_file.ts", None)
        .unwrap()
        .expect("Should load context");
    
    assert_eq!(loaded_ir.file_id, "test_file.ts");
    assert_eq!(loaded_ir.instructions.len(), 3);
    
    // Append delta
    let delta_bytes = serde_json::to_vec(&serde_json::json!({"test": "delta"})).unwrap();
    store.append_delta(&ctx_id, &delta_bytes, Some("edit")).unwrap();
    assert_eq!(store.delta_count(&ctx_id), 1);
    
    // Clear
    store.clear_file("test_file.ts");
    assert!(!store.has_context("test_file.ts"));
}

#[test]
fn test_sqlite_in_memory() {
    // Use :memory: for isolated tests — no filesystem dependency
    use rusqlite::Connection;
    let conn = Connection::open_in_memory().unwrap();
    // Manually create schema and test queries
    conn.execute_batch("CREATE TABLE IF NOT EXISTS test (id INTEGER PRIMARY KEY, val TEXT);").unwrap();
    conn.execute("INSERT INTO test (val) VALUES (?1)", params!["hello"]).unwrap();
    let val: String = conn.query_row("SELECT val FROM test WHERE id = 1", [], |row| row.get(0)).unwrap();
    assert_eq!(val, "hello");
}
```

### 8.5 Test Files to Create/Update

| File | Type | What to Test |
|------|------|-------------|
| `src/tests/ir/persistence.rs` | **New** | `test_sqlite_round_trip`, `test_sqlite_in_memory` |
| `src/tests/mcp/context_store.rs` | **Existing** | Add `SqliteStore` tests alongside `InMemoryContextStore` tests |
| `src/tests/mcp/session_stats.rs` | **Existing** | Add `test_stats_rebuild_from_db` |
| `src/tests/mcp/tools.rs` | **Existing** | Add `test_save_context_tool`, `test_purge_deltas_tool` |

### 8.6 Link test modules

```rust
// In src/tests/ir/mod.rs — add:
#[path = "persistence.rs"]
mod persistence;
```

```rust
// In src/mcp/mod.rs (test section) — add sqlite_store tests if needed.
```

---

## 9. Phase 6: Documentation

### 9.1 Update `ARCHITECTURE_OVERVIEW.md`

Add a new section after "Compiler IR" or "Layered Encoding":

```markdown
### Persistence Layer

The persistence layer (optional, gated by `persistence` feature flag) backs the
`ContextStore` trait with SQLite. It provides:

- **Cross-session survival**: Baselines, IR data, and delta histories persist
  after IDE restarts, crashes, or multi-chat workflows.
- **Restore & replay**: Load a baseline IR from the `.clean-ctx/persistence.db`
  file and replay deltas to reconstruct any edit state.
- **WAL mode**: Write-Ahead Logging enables safe concurrent reads during writes.
- **Auto-pruning**: Manual `purge_old_deltas` tool trims history.

See `docs/PLAN_SQLITE_PERSISTENCE_LAYER.md` for the agent handoff document.
```

### 9.2 Update `COMPILER_IR.md`

Add a note about the binary-wire identity restoration (Gap 2):

```markdown
### Binary Wire Identity

The `binary_wire::encode()` format does NOT include `file_id` or `version` in
the byte stream — these are stored as separate columns in the SQLite `contexts`
table. On load from DB, reconstruct with:
```rust
let mut ir = binary_wire::decode(&blob)?;
ir.file_id = row.file_path;  // restore from DB column
ir.version = row.version;    // restore from DB column
```
```

### 9.3 Update `ROADMAP.md`

Mark the persistence work. Add a note that `symbols.phi_markers` is deferred.

---

## 10. Integration Points Summary Table

| Integration Point | File | What Changes |
|---|---|---|
| **Cargo.toml** | `Cargo.toml` | Add `rusqlite = { version = "0.32", features = ["bundled"] }`, `chrono`, optional `thiserror` |
| **SqliteStore** | `src/mcp/sqlite_store.rs` | **New** — implements `ContextStore` trait |
| **Module decl** | `src/mcp/mod.rs` | Add `pub(crate) mod sqlite_store;` |
| **McpState** | `src/mcp/state.rs` | Add `persistence_store: Option<SqliteStore>` field, initialize in `new()` |
| **Config reading** | `src/mcp/state.rs` (in `new()`) | Read `config.persistence.enabled`, `config.persistence.db_path` |
| **Tool dispatch** | `src/mcp/tools.rs` | Add `save_context`, `list_sessions`, `replay_history`, `purge_old_deltas` to `dispatch_tools_call` |
| **Tool definitions** | `src/mcp/tools.rs` (`tool_list()`) | Add 4 new tool definitions |
| **FullCompress hook** | `src/mcp/tools.rs` (`handle_provide_code_context`) | After IR compilation, call `store.save_context()` |
| **DeltaTransport hook** | `src/mcp/tools.rs` (Delta branch) | Call `store.append_delta()` |
| **handle_delta_code_context** | `src/mcp/tools.rs` | Call `store.append_delta()` with IR delta |
| **handle_restore_context** | `src/mcp/tools.rs` | Try DB restore first, fall back to full re-compress |
| **Symbol table** | `src/ir/symbol_table.rs` | Add `to_json()` / `from_json()` methods |
| **Binary wire** | `src/ir/binary_wire.rs` | No changes needed — identity restored from DB columns |
| **Delta** | `src/ir/delta.rs` | Reuse `compact_encode()` for smaller BLOB storage (no code change needed) |
| **Replay** | `src/ir/replay.rs` | `ContextState::load_ir()` + `apply()` — no changes needed, used by `load_context_with_deltas()` |
| **SessionStats** | `src/mcp/session_stats.rs` | No changes needed — `rebuild_stats_from_db()` populates it |
| **Default config** | `src/main.rs` (template) | Change `"enabled": false` → `"enabled": true` last |
| **fifty_edit_sim** | `examples/fifty_edit_simulation.rs` | Add `--persist` flag + round-trip verification |
| **Tests** | `src/tests/ir/persistence.rs` | **New** — SQLite round-trip test, in-memory test |
| **ARCHITECTURE.md** | `docs/ARCHITECTURE_OVERVIEW.md` | Add Persistence Layer section |
| **COMPILER_IR.md** | `docs/COMPILER_IR.md` | Add binary-wire identity note |

---

## 11. Commit Sequence

Each phase should be one commit (or a small set of commits) with a descriptive message. Do NOT squash — preserve the progression.

```
branch: feature/compilerIR_upgrade
target branch: feature/sqlite-persistence

  1. P1-Core: feat(db): add rusqlite dependency, SqliteStore, schema migration
     - Cargo.toml: add rusqlite + chrono
     - src/mcp/mod.rs: add sqlite_store module
     - src/mcp/sqlite_store.rs: SqliteStore struct, open(), migrate(), ContextStore impl
     - src/mcp/context_store.rs: no changes (trait already defined)

  2. P2-Hooks: feat(db): wire persistence into McpState and hot-path handlers
     - src/mcp/state.rs: add persistence_store field, read config.persistence
     - src/mcp/tools.rs: hook save_context/append_delta into FullCompress, DeltaTransport, delta_code_context
     - src/ir/symbol_table.rs: add to_json()/from_json()

  3. P3-Restore: feat(db): add restore and replay from DB
     - src/mcp/sqlite_store.rs: add load_context_with_deltas(), rebuild_stats()
     - src/mcp/tools.rs: update restore_context to try DB first
     - src/mcp/state.rs or server.rs: call rebuild_stats() on startup

  4. P4-MCP: feat(db): add MCP tools for session management
     - src/mcp/tools.rs: add save_context, list_sessions, replay_history, purge_old_deltas handlers + definitions
     - src/mcp/tools.rs: add dispatch entries
     - src/mcp/sqlite_store.rs: add helper methods (delta_count_for_file, etc.)

  5. P5-Tests: feat(db): add tests, update simulation, enable persistence by default
     - src/tests/ir/persistence.rs: round-trip + in-memory tests
     - examples/fifty_edit_simulation.rs: --persist flag + restore verification
     - src/main.rs: change persistence default to enabled: true
     - docs/*: update architecture docs
```

---

## Appendix A: Quick Reference — Key File Paths

| What | Path |
|------|------|
| ContextStore trait | `src/mcp/context_store.rs` |
| InMemoryContextStore | `src/mcp/context_store.rs` (lines 112-232) |
| SqliteStore (to create) | `src/mcp/sqlite_store.rs` |
| McpState (wire here) | `src/mcp/state.rs` |
| Tool definitions | `src/mcp/tools.rs` (function `tool_list()`) |
| Tool dispatch | `src/mcp/tools.rs` (function `dispatch_tools_call()`) |
| provide_code_context handler | `src/mcp/tools.rs` (`handle_provide_code_context`, line 895) |
| restore_context handler | `src/mcp/tools.rs` (`handle_restore_context`, line 1078) |
| delta_code_context handler | `src/mcp/tools.rs` (`handle_delta_code_context`, line 539) |
| GlobalSymbolTable | `src/ir/symbol_table.rs` |
| binary_wire encode/decode | `src/ir/binary_wire.rs` |
| DeltaComputer | `src/ir/delta.rs` |
| compact_encode/decode | `src/ir/delta.rs` (lines 361-482) |
| ContextState (replay) | `src/ir/replay.rs` |
| SessionStats | `src/mcp/session_stats.rs` |
| PersistenceConfig | `src/config.rs` (lines 87-121) |
| CleanCtxConfig | `src/config.rs` (lines 127-248) |
| clean-ctx init | `src/main.rs` (`cmd_init()`, line 19) |
| Default config template | `src/main.rs` (`generate_default_config()`, line 54) |
| MCP module root | `src/mcp/mod.rs` |
| Integration test for context_store | `src/tests/mcp/context_store.rs` |

---

## Appendix B: Rust Version & Edition Notes

- The project uses `edition = "2024"` and `rust-version = "1.85"` (from `Cargo.toml`)
- `rusqlite 0.32` with `bundled` feature compiles SQLite from source — no system dependency
- `chrono 0.4` with `serde` feature enables datetime serialization
- The MCP server is single-threaded (stdin/stdout loop) — no `Mutex` needed for `Connection`
- Use `params![]` macro from rusqlite for parameterized queries
- Use `sqlite_version()` from rusqlite to verify the bundled version at runtime if needed
- The `#[allow(dead_code)]` supressions throughout `context_store.rs` can be removed once `SqliteStore` is complete
