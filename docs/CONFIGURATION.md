# Configuration Guide

Clean-CTX uses a two-tier configuration system with explicit precedence rules.

## Configuration Precedence

**Highest to lowest priority:**

1. **Tool argument** — Per-call overrides (e.g., `fidelity`, `tokenizer`)
2. **Environment variable** — Proxy settings (`PROXY_API_KEY`, `RATE_LIMIT_RPS`, etc.)
3. **Config file** — `.clean-ctx.json` in project root
4. **Default** — Built-in defaults when no other source specifies a value

Example: If `fidelity` is passed as a tool argument, it overrides both the env var and `.clean-ctx.json` setting.

## Configuration File (`.clean-ctx.json`)

Location: Project root (walks up from current directory to find it)

### Complete Example

```json
{
  "default_fidelity": "low",
  "diff_compression": true,
  "workspace_type_detection": true,
  "auto_angular": true,
  "auto_delta": true,
  "tokenizer": "o200k",
  "exclude_patterns": [
    "node_modules",
    "dist",
    "*.test.ts",
    "*.spec.ts"
  ],
  "resource_limits": {
    "max_file_size_bytes": 10485760,
    "max_workspace_files": 10000,
    "max_memory_bytes": 536870912
  },
  "persistence": {
    "enabled": false,
    "auto_save": true,
    "max_history_days": 30,
    "db_path": ".clean-ctx/persistence.db"
  },
  "meta_layers": {
    "angular": {
      "enabled": true
    }
  },
  "smart_defaults": {
    "refactor": "high",
    "overview": "low",
    "debug": "medium",
    "edit": "low",
    "implement": "medium"
  },
  "heuristics": {
    "large_file_threshold": 300,
    "force_high_fidelity": [
      "*.service.ts",
      "*.component.ts",
      "*.guard.ts"
    ],
    "use_angular_meta": true,
    "complex_import_threshold": 15,
    "complex_fn_threshold": 10,
    "medium_lines": 300,
    "high_lines": 500,
    "auto_classify": true,
    "session_aware_fidelity": true
  },
  "cache": {
    "enabled": true,
    "system_prompt_ttl": "1h",
    "tools_ttl": "1h",
    "baseline_ttl": "1h",
    "tail_ttl": "5m",
    "vocab_version": "v1",
    "tool_defs_version": "v1"
  },
  "cbm": {
    "enabled": true,
    "binary_path": null,
    "auto_launch": true,
    "cache_ttl": 300,
    "query_timeout_ms": 30000,
    "max_retries": 3,
    "circuit_cooldown_secs": 30
  },
  "intelligence": {
    "enabled": true,
    "blast_radius_enabled": false,
    "max_blast_radius_files": 10
  },
  "type_aliases": {
    "UserId": "$uid",
    "JsonObject": "$jo",
    "HttpClient": "$http"
  }
}
```

## Environment Variables

### Proxy Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `PROXY_API_KEY` | API key for proxy authentication (enables auth when set) | None (auth disabled) |
| `RATE_LIMIT_RPS` | Max requests per second per client IP | `60` |
| `RATE_LIMIT_BURST` | Max burst size for rate limiter | `10` |
| `MAX_REQUEST_BODY_SIZE` | Max request body size in bytes | `10485760` (10 MB) |
| `PORT` | Proxy server port | `8787` |
| `UPSTREAM_URL` | Upstream API URL | `https://api.anthropic.com` |
| `ANTHROPIC_BASE_URL` | Upstream API base URL (alias for UPSTREAM_URL) | `https://api.anthropic.com` |
| `AUTO_CACHE` | Enable cache breakpoint injection (Anthropic only) | `false` |
| `TAIL_TTL` | TTL for the rolling-tail breakpoint | `5m` |
| `SLIDING_WINDOW` | Enable sliding context window (age-based tool-result truncation) | `false` |
| `SLIDING_WINDOW_MAX_AGE` | Max age in turns before a tool result is aged | `20` |
| `SLIDING_WINDOW_FLOOR` | Number of most recent turns to always preserve | `15` |
| `STRIP_ANSI` | Strip ANSI escape codes from text blocks | `false` |
| `TRIM_BASH_GIT` | Truncate Bash tool description at "Committing changes" | `false` |
| `MODEL_OVERRIDE` | Override model name in every request | _(none)_ |
| `LOG_BODIES` | Log request/response bodies to disk | `false` |
| `LOG_DIR` | Directory for log files | `.clean-ctx/proxy-logs` |
| `SCRUB_SECRETS` | Enable secret scrubbing in tool results | `false` |
| `TOOL_FILTERS` | Enable tool output filtering (TOML-based) | `false` |
| `PLATFORM` | Override platform detection (`anthropic`, `openai`, `generic`) | _(auto-detect)_ |

### CI/CD Environment Variables

A-14: Clean-CTX automatically detects CI environments and disables persistence to prevent stale database issues.

| Variable | Description |
|----------|-------------|
| `CI=true` | Generic CI flag (GitHub Actions, GitLab CI, etc.) |
| `TF_BUILD` | Azure DevOps |
| `GITHUB_ACTIONS` | GitHub Actions |
| `GITLAB_CI` | GitLab CI |
| `JENKINS_URL` | Jenkins |
| `CIRCLECI` | CircleCI |
| `TRAVIS` | Travis CI |

**Behavior**: When any of these are detected, `persistence.enabled` is automatically set to `false` regardless of `.clean-ctx.json` settings.

## Resource Limits (A-13)

Prevent OOM crashes on large codebases:

```json
{
  "resource_limits": {
    "max_file_size_bytes": 10485760,      // 10 MB per file
    "max_workspace_files": 10000,         // Max files in workspace
    "max_memory_bytes": 536870912         // 512 MB total memory
  }
}
```

**Enforcement points:**
- `max_file_size_bytes`: Checked in `compress_file` before reading file
- `max_workspace_files`: Checked in `compress_workspace_dir` after file collection
- `max_memory_bytes`: Checked in `compress_workspace_dir` before compression (estimates 2× file size)

## `diff_commits` Tool (R-12)

Diffs an entire workspace between two git refs, emitting per-file AST-level change-sets in a single call. Powers "what changed in this PR?" workflows.

**Arguments:**

| Argument | Required | Description |
|----------|----------|-------------|
| `fromRef` | ✅ | Base ref, e.g. `HEAD~1`, `main`, `abc123`, `v1.0`. Strictly validated. |
| `toRef` | ❌ | Target ref. Defaults to the working tree (uncommitted changes). |
| `workspaceRoot` | ❌ | Project root. Defaults to CWD. Resolved against the trusted root. |
| `fidelity` | ❌ | `low` / `medium` / `high`. Defaults to config `default_fidelity`. |

**Output:** A `§GITDIFF <from>..<to> (N files)` header followed by per-file `┌ FILE αN <path> (+A -D ~M)` sections. Added files emit a compact skeleton; deleted files a one-line entry; renamed files (via `--find-renames`) a `~ FILE αN <old> → <new>` section. Non-compressible extensions (html/css/json) fall back to a line-count delta so the tool never fails on grammar-missing files.

**Security posture:**
- **Ref injection** — refs are validated against the strict allowlist `^[A-Za-z0-9][A-Za-z0-9._/\-~]*$` (first char never `-`), rejecting flag-injection attempts like `--upload-pack`.
- **XPIA** — `workspaceRoot` is resolved via `resolve_file_path_checked` (anchored to the process CWD); git output paths are validated (no absolute escapes).
- **No shell** — all git calls use `std::process::Command` with explicit `arg()` calls and `--end-of-options`; never shell-interpolated.
- **Resource limits** — the changed-file count is capped by `resource_limits.max_workspace_files` and per-file size by `resource_limits.max_file_size_bytes`; files exceeding either are counted in `_meta.skipped`.
- **Fail-closed** — invalid refs, non-git directories, or git errors return structured `-32602`/`-32603` errors, never partial output.

## Persistence Configuration

Controls SQLite-backed cross-session storage:

```json
{
  "persistence": {
    "enabled": false,           // Master switch (default: false)
    "auto_save": true,          // Auto-save after each operation
    "max_history_days": 30,     // Prune history older than this
    "db_path": ".clean-ctx/persistence.db"
  }
}
```

**Note**: Persistence is disabled by default to avoid SQLite file lock contention in parallel test runs. Enable it only when you need cross-session compression history.

## Smart Defaults

Maps high-level intents to compression fidelity:

```json
{
  "smart_defaults": {
    "refactor": "high",      // Full structural detail
    "overview": "low",       // Maximum compression
    "debug": "medium",       // Balanced detail vs compression
    "edit": "low",           // Maximum compression, delta-friendly
    "implement": "medium"    // Moderate detail
  }
}
```

## Heuristics Configuration (V2)

Auto-classify files by content signals:

```json
{
  "heuristics": {
    "large_file_threshold": 300,        // Lines threshold for "large"
    "force_high_fidelity": [],          // Extensions that always get High
    "use_angular_meta": true,           // Enable Angular Meta-Layer
    "complex_import_threshold": 15,     // Min imports for "complex" classification
    "complex_fn_threshold": 10,         // Min functions for "complex"
    "medium_lines": 300,                // Lines → Medium fidelity fallback
    "high_lines": 500,                  // Lines → High fidelity fallback
    "auto_classify": true,              // Enable V2 auto-classification
    "session_aware_fidelity": true      // Check DB for prior fidelity
  }
}
```

## Meta-Layer Configuration

Per-framework meta-layer settings:

```json
{
  "meta_layers": {
    "angular": {
      "enabled": true
    },
    "spring_boot": {
      "enabled": true
    },
    "dotnet": {
      "enabled": true
    }
  }
}
```

Currently supported: `angular`, `spring_boot`, `dotnet`. Each meta-layer can be toggled at compile time via Cargo feature flags (see `Cargo.toml` for the full dependency tree). Future: `react`, `vue`, `svelte`.

**Compile-time feature flags vs runtime config:**

| Layer | Cargo Feature | Implies | Includes | Default |
|-------|---------------|---------|----------|---------|
| TypeScript | `typescript` | — | Base TypeScript/JavaScript tree-sitter grammar | ✅ |
| C# | `csharp` | — | Base C# tree-sitter grammar | ✅ |
| Rust | `rust` | — | Base Rust tree-sitter grammar | ❌ |
| Java | `java` | — | Base Java tree-sitter grammar | ❌ |
| Angular | `angular` | `typescript` | Components, Services, DI, Pipes, Directives, Modules, Input/Output, Template/Shape extraction, Style extraction, NgRx, RxJS, Signals, PrimeNG, Bundle graph | ✅ |
| Spring Boot | `spring_boot` | `java` | RestController, Controller, Service, Repository, Configuration, RequestMapping, Autowired, Value, Bean, ConfigurationProperties, Cross-file graph | ❌ |
| .NET | `dotnet` | `csharp` | ASP.NET Core (Controllers, Actions, Routes, Auth), EF Core (DbContext, DbSet, Entities), SignalR (Hubs, Clients, Streaming), AutoMapper (Profiles, Mappings), JSON Serialization, DI, Validation, Identity, Caching, Logging, Cross-file graph | ❌ |

The runtime `meta_layers.*.enabled` config in `.clean-ctx.json` controls whether an already-compiled meta-layer is active at runtime. Disabling it at compile time via `--no-default-features` removes the entire module from the binary.

## Intelligence Layer Configuration

CBM-informed fidelity and blast radius:

```json
{
  "intelligence": {
    "enabled": true,                    // Master switch
    "blast_radius_enabled": false,      // Enable depth-1 affected files
    "max_blast_radius_files": 10        // Prevent token explosion
  }
}
```

## Type Aliases (R-02)

Type-aware compression: replaces configured type names in compressed
output with short alias tokens, reducing token usage on type-heavy files.

```json
{
  "type_aliases": {
    "UserId": "$uid",
    "JsonObject": "$jo",
    "HttpClient": "$http"
  }
}
```

**How it works:**
- At Medium/High fidelity, type names in method signatures, field types,
  and IR ops are replaced with the alias token.
- A `§TA` footer (`§TA $uid→UserId $jo→JsonObject`) is emitted in the
  text compression path so the LLM can resolve every alias.
- In the IR path, `CoreOp::TypeAlias(alias, original)` ops are appended
  for each used alias.
- At Low fidelity, types are already stripped — aliases are a natural
  no-op.

**Alias token rules:**
- Must start with `$` (distinguishes from structural markers `⊕`, `Φ`, `§`)
- Must be ≥ 2 chars total (at least one char after `$`)
- Chars after `$` must be `[A-Za-z0-9_]`
- Must NOT be numeric-only after `$` (`$1`, `$2`, …) — the symbol
  dictionary owns the `$N` opcode space
- Original type must be ≥ 4 chars (avoids replacing trivial types like
  `int`, `str` where savings are negligible)

**Token-boundary matching:**
- `$` is treated as an identifier character (like `_`) so `$`-prefixed
  tokens (aliases, symbol-dictionary refs) are atomic.
- `User` matches in `id:User`, `Map<string,User>`, `Promise<User>`,
  `A | User`, but NOT in `UserService`, `GitUserProfile`, or `user_id`.

**Estimated savings:** 5-15% additional on type-heavy files at
Medium/High fidelity. Low fidelity is unaffected.

## Cache Configuration

Anthropic API prompt cache optimization:

```json
{
  "cache": {
    "enabled": true,
    "system_prompt_ttl": "1h",      // Stable vocabulary
    "tools_ttl": "1h",              // Stable tool definitions
    "baseline_ttl": "1h",           // Stable baselines
    "tail_ttl": "5m",               // Dynamic content
    "vocab_version": "v1",
    "tool_defs_version": "v1"
  }
}
```

### `_meta.cache_hints` Contract

When `cache.enabled` is `true`, the MCP server injects `_meta.cache_hints`
into JSON-RPC responses. This is the **consumer contract** for LLM clients
(Claude Desktop, Cline, custom MCP clients) that want to set Anthropic
`cache_control` breakpoints on stable content regions.

**Response shape:**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [{ "type": "text", "text": "..." }],
    "_meta": {
      "cache_hints": {
        "breakpoints": [
          {
            "region": "baseline",
            "ttl": "1h",
            "breaker": "bl_<sha256>"
          }
        ]
      }
    }
  }
}
```

**Regions emitted by each tool:**

| Tool | Region | Breaker | TTL |
|------|--------|---------|-----|
| `tools/list` | `tools` | `tools-<version>` | `tools_ttl` |
| `prompts/list` | `system_prompt` | `vocab-<version>` | `system_prompt_ttl` |
| `prompts/get` (cleanctx-notation) | `system_prompt` | `vocab-<version>` | `system_prompt_ttl` |
| `prompts/get` (clean-ctx-vocabulary) | `system_prompt` | `vocab-<version>` | `system_prompt_ttl` |
| `compress_code_context` | `baseline` | `bl_<sha256 of compressed output>` | `baseline_ttl` |
| `provide_code_context` (full) | `baseline` | `bl_<sha256 of compressed output>` | `baseline_ttl` |
| `provide_code_context` (delta) | `tail` | `rolling` | `tail_ttl` |
| `delta_code_context` | `tail` | `rolling` | `tail_ttl` |
| `delta_text_context` | `tail` | `rolling` | `tail_ttl` |
| `apply_delta` | `tail` | `rolling` | `tail_ttl` |
| `diff_code_context` | `tail` | `rolling` | `tail_ttl` |
| `restore_context` | `baseline` | `bl_<sha256 of compressed output>` | `baseline_ttl` |
| `compress_workspace` | `baseline` | `ws_<sha256 of manifest>` | `baseline_ttl` |
| `diff_commits` | `baseline` | `ws_<sha256 of manifest>` | `baseline_ttl` |

**How clients should consume the hints:**

1. Read `result._meta.cache_hints.breakpoints[]` from each response.
2. For each breakpoint, set `cache_control: {"type": "ephemeral"}` on the
   corresponding stable content block in the next Anthropic API request.
3. The `breaker` value is the cache invalidation key — when it changes,
   the cached content is stale and must be re-sent.
4. The `tail` region is always `rolling` — it represents dynamic content
   that changes each turn and should never be cached across turns.

**Deduplication:** The MCP server deduplicates breakpoint emission per
session via `state.emitted_breakpoints`. The same `{region}::{breaker}`
combo is only emitted once — subsequent calls increment `cache_metrics.hits`
and return a token-savings estimate instead of re-emitting.

**Real cache savings:** When the Clean-CTX proxy is running with
`AUTO_CACHE=1`, it parses Anthropic's `usage.cache_read_input_tokens` and
`usage.cache_creation_input_tokens` from upstream responses. These REAL
token counts are surfaced in the `context_stats` dashboard's `prompt_cache`
domain — no fabricated estimates.

## Debug: Print Resolved Configuration

To see the final resolved configuration (after all precedence rules), run:

```bash
# Via MCP tool (when implemented in A-15)
clean-ctx --config-dump

# Or programmatically
let config = CleanCtxConfig::load(Path::new("."));
println!("{:#?}", config);
```

## Migration Notes

- **No breaking changes**: All new fields have sensible defaults
- **Backward compatible**: Old `.clean-ctx.json` files continue to work
- **Opt-in features**: New features (persistence, CBM, intelligence) default to `false`/`disabled`

## Common Patterns

### Disable persistence in CI (A-14)

**Automatic**: Clean-CTX detects CI env vars and disables persistence automatically.

**Manual override**: Set in `.clean-ctx.json`:
```json
{
  "persistence": {
    "enabled": false
  }
}
```

### Exclude directories from compression

```json
{
  "exclude_patterns": [
    "node_modules",
    "dist",
    "build",
    "*.test.ts",
    "*.spec.ts"
  ]
}
```

### Force high fidelity for specific files

```json
{
  "heuristics": {
    "force_high_fidelity": [
      "*.service.ts",
      "*.component.ts",
      "*.guard.ts",
      "*.repository.ts"
    ]
  }
}
```

### Use custom tokenizer

```json
{
  "tokenizer": "cl100k"  // Options: o200k, cl100k, claude, llama3
}
```

Or override per-tool-call:
```python
result = provide_code_context(
    file_path="src/foo.ts",
    tokenizer="claude"  # Override for this call only
)