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
    "enabled": false,
    "server_path": "codebase-memory-mcp",
    "connection_timeout_secs": 5,
    "query_timeout_secs": 10,
    "cache_ttl_secs": 600
  },
  "intelligence": {
    "enabled": true,
    "blast_radius_enabled": false,
    "max_blast_radius_files": 10
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
| `PORT` | Proxy server port | `8080` |
| `UPSTREAM_URL` | Upstream API URL | `https://api.anthropic.com` |

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