# Auto-Start Proxy on Clean-CTX Startup — Implementation Plan

## Goal
Add a configurable option to `.clean-ctx.json` that automatically starts the `clean-ctx-proxy` binary as a child process when the `clean-ctx` MCP server starts, and gracefully shuts it down when the MCP server exits.

## Current Architecture
- `clean-ctx` (MCP server) — synchronous stdio JSON-RPC server (`src/main.rs`)
- `clean-ctx-proxy` (Proxy) — async tokio HTTP server on port 8787 (`proxy/src/main.rs`)
- Completely separate binaries — no shared lifecycle or config

## Design

### 1. New Config Block in `.clean-ctx.json`

```json
{
  "proxy": {
    "auto_start": true,
    "port": 8787,
    "auto_cache": true,
    "tail_ttl": "5m",
    "drop_tools": ["NotebookEdit", "CronCreate"],
    "strip_ansi": true,
    "trim_bash_git": false,
    "model_override": null,
    "scrub_secrets": true,
    "tool_filters": true,
    "upstream_url": "https://api.anthropic.com",
    "api_key": null,
    "rate_limit_rps": 60,
    "rate_limit_burst": 10
  }
}
```

All fields optional with sensible defaults. `auto_start` defaults to `false` (no behavior change for existing users).

### 2. Config → Env Var Mapping

| Config key | Env var | Default |
|---|---|---|
| `auto_start` | — | `false` |
| `port` | `PORT` | `8787` |
| `auto_cache` | `AUTO_CACHE` | `false` |
| `tail_ttl` | `TAIL_TTL` | `"5m"` |
| `drop_tools` | `DROP_TOOLS` | `[]` |
| `strip_ansi` | `STRIP_ANSI` | `false` |
| `trim_bash_git` | `TRIM_BASH_GIT` | `false` |
| `model_override` | `MODEL_OVERRIDE` | `null` |
| `scrub_secrets` | `SCRUB_SECRETS` | `false` |
| `tool_filters` | `TOOL_FILTERS` | `false` |
| `upstream_url` | `PROXY_UPSTREAM_URL` | `null` |
| `api_key` | `PROXY_API_KEY` | `null` |
| `rate_limit_rps` | `RATE_LIMIT_RPS` | `60` |
| `rate_limit_burst` | `RATE_LIMIT_BURST` | `10` |

### 3. New `ProxySpawner` Module (`src/proxy_spawner.rs`)

Responsibilities:
- **Binary resolution** (in order):
  1. `PROXY_BINARY_PATH` env var (explicit override)
  2. Same directory as the MCP binary (`clean-ctx-proxy.exe` / `clean-ctx-proxy`)
  3. `cargo run -p clean-ctx-proxy` fallback (dev mode)
- **Spawn**: `std::process::Command` with all config fields mapped to env vars
- **Lifecycle**: store `Child` handle; on MCP shutdown send `CTRL_C`/`SIGTERM`, wait with timeout
- **Crash detection**: `child.try_wait()` on each MCP request; log if proxy died
- **Non-fatal**: missing binary → log warning, continue without proxy

### 4. `McpState` Integration

Add `proxy_child: Mutex<Option<Child>>` to `McpState` (or a separate `Arc<Mutex<...>>`).

### 5. CLI Subcommand

Add `Proxy` variant to `Cli` enum in `src/main.rs`:
- `clean-ctx proxy` — start the proxy standalone (no MCP server)
- `clean-ctx proxy --stop` — stop a running auto-started proxy

### 6. Lifecycle

```
clean-ctx starts
  → load .clean-ctx.json
  → if proxy.auto_start=true:
      → resolve proxy binary
      → spawn child with env vars from config
      → log "Proxy started on 127.0.0.1:{port}"
  → start MCP server loop
  → ... (run)
  → stdin exhausted → drop(dispatcher)
  → if proxy child exists:
      → send Ctrl+C / SIGTERM
      → wait for exit (with timeout)
      → log "Proxy stopped"
  → exit
```

### 7. Edge Cases

- **Binary not found** — log warning, continue (non-fatal)
- **Proxy crashes** — detect via `try_wait()`, log, optionally restart
- **Port already in use** — proxy exits with bind error; MCP logs it
- **MCP crashes** — child becomes orphan; OS handles it
- **Self-forwarding loop** — already handled by proxy's `validate()`

---

## Test Plan

### A. Config Parsing Tests (`src/tests/config.rs`)

| Test | Verifies |
|---|---|
| `proxy_auto_start_defaults_false` | `auto_start` defaults to `false` when block missing |
| `proxy_config_parses_all_fields` | All 14 fields parse correctly from JSON |
| `proxy_config_missing_block_uses_defaults` | Missing `proxy` block → all defaults |
| `proxy_config_partial_block_uses_defaults` | Partial block → missing fields get defaults |
| `proxy_config_serializes_roundtrip` | Serialize → deserialize → identical |

### B. Proxy Spawner Unit Tests (new `src/tests/proxy_spawner.rs`)

| Test | Verifies |
|---|---|
| `spawner_disabled_when_auto_start_false` | No child spawned when `auto_start=false` |
| `spawner_resolves_binary_same_dir` | Binary resolution finds `clean-ctx-proxy` next to MCP binary |
| `spawner_uses_proxy_binary_path_env` | `PROXY_BINARY_PATH` env var overrides resolution |
| `spawner_maps_config_to_env_vars` | All config fields map to correct env var names/values |
| `spawner_handles_missing_binary_gracefully` | Missing binary → `Ok(None)`, no panic |
| `spawner_kills_child_on_shutdown` | `shutdown()` sends termination signal and waits |
| `spawner_detects_child_crash` | `try_wait()` detects exited child, returns status |
| `spawner_skips_when_port_in_use` | Proxy already running on port → skip spawn, log |

### C. McpState Integration Tests (`src/tests/mcp/state.rs`)

| Test | Verifies |
|---|---|
| `state_holds_proxy_child_handle` | `McpState` can store/retrieve the child handle |
| `state_proxy_child_defaults_none` | Fresh state has `None` proxy child |

### D. CLI Tests (`src/tests/main.rs`)

| Test | Verifies |
|---|---|
| `cli_proxy_subcommand_parses` | `clean-ctx proxy` parses without error |
| `cli_proxy_stop_flag_parses` | `clean-ctx proxy --stop` parses |

### E. Integration Test (new `src/tests/proxy_auto_start_integration.rs`)

| Test | Verifies |
|---|---|
| `auto_start_spawns_real_proxy` | Spawn real proxy binary, verify it binds to port, kill it |
| `auto_start_skips_when_disabled` | No proxy process when `auto_start=false` |
| `auto_start_shutdown_terminates_child` | MCP shutdown terminates the child process |

### F. Proxy Crate Tests (no changes needed)

The proxy crate already has 243 tests covering its own behavior. The auto-start feature only needs to verify the proxy binary can be spawned and killed — the proxy's internal correctness is already covered.

---

## Files to Create/Modify

| File | Action |
|---|---|
| `docs/PROXY_AUTO_START_PLAN.md` | **Create** — this plan document |
| `src/config.rs` | **Modify** — add `ProxyAutoStartConfig` struct + `proxy` field |
| `src/proxy_spawner.rs` | **Create** — new spawner module |
| `src/mcp/state.rs` | **Modify** — add `proxy_child` handle |
| `src/main.rs` | **Modify** — add `Proxy` CLI subcommand |
| `src/mcp/mod.rs` | **Modify** — wire spawner into server startup/shutdown |
| `src/tests/config.rs` | **Modify** — add config parsing tests |
| `src/tests/proxy_spawner.rs` | **Create** — spawner unit tests |
| `src/tests/proxy_auto_start_integration.rs` | **Create** — integration tests |
| `src/tests/main.rs` | **Modify** — add CLI tests |
| `docs/CONFIGURATION.md` | **Modify** — document the `proxy` block |
