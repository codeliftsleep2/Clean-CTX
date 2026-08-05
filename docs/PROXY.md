# Clean-CTX Proxy

A Rust HTTP proxy that sits between your LLM client and any AI API (Anthropic, OpenAI, DeepSeek, etc.), automatically injecting prompt-cache breakpoints, compressing verbose tool output, scrubbing secrets, and applying TOML-based filters to reduce token usage by 70-90%.

Works with any client that sends Anthropic-format `POST /v1/messages` or OpenAI-format `POST /v1/chat/completions` requests: Cline, Cursor, Aider, Continue.dev, GitHub Copilot (BYOK), and custom clients.

## Why?

LLM clients don't send the `cache_control` headers needed to activate prompt caching, and tool output (cargo builds, npm installs, git diffs) is often verbose and wastes tokens. This proxy:

1. **Injects cache breakpoints** to achieve ~90% API cost savings on cached turns
2. **Compresses tool output** using TOML-based filters (cargo, npm, pytest, etc.)
3. **Scrubs secrets** (AWS keys, GitHub tokens, JWTs, PEM keys) before they reach the LLM
4. **Strips ANSI codes** from terminal output
5. **Drops unused tools** to reduce token usage

## Quick Start

```bash
# Build and run
cargo run -p clean-ctx-proxy

# Or with full optimization
cargo run --release -p clean-ctx-proxy
```

The proxy binds to `http://127.0.0.1:8787` by default.

## Multi-Platform Support

The proxy supports **any AI provider** that uses the Anthropic or OpenAI API format:

| Provider | Platform | Intercept Path | Detection |
|----------|----------|----------------|-----------|
| Anthropic (Claude) | `anthropic` | `/v1/messages` | `model.contains("claude")` |
| OpenAI (GPT) | `openai` | `/v1/chat/completions` | `model.contains("gpt")` |
| DeepSeek | `openai` or `generic` | `/v1/chat/completions` or `/chat` | `model.contains("deepseek")` or fallback |
| Any other | `generic` | `/chat` | Heuristic fallback |

### Platform Detection

The proxy auto-detects the platform from the request body:

```rust
// In platform/mod.rs
pub fn detect_platform(body: &Value) -> Box<dyn PlatformAdapter> {
    if let Some(model) = body["model"].as_str() {
        if model.contains("claude") {
            return Box::new(AnthropicAdapter);
        }
        if model.contains("deepseek") || model.contains("gpt") {
            return Box::new(OpenAIAdapter);
        }
    }
    Box::new(GenericAdapter)  // Fallback
}
```

You can also override the platform with the `PLATFORM` environment variable:

```bash
PLATFORM=generic cargo run -p clean-ctx-proxy
```

## IDE Integration

### Cline (VS Code Extension)

1. Open **Cline settings** (click the gear icon in Cline panel)
2. Change **API Provider** from "cline" to one of:
   - **"OpenAI Compatible"** — for DeepSeek, GPT, or any OpenAI-compatible API
   - **"Anthropic"** — for Claude models
3. Configure the endpoint:
   - **Base URL**: `http://127.0.0.1:8787`
   - **API Key**: your actual API key
   - **Model ID**: `deepseek-chat` (or `gpt-4o`, `claude-sonnet-4-20250514`, etc.)
4. Restart Cline for changes to take effect

### Cursor

1. Open **Settings** → **Models**
2. Add a custom model with:
   - **API Base URL**: `http://127.0.0.1:8787`
   - **API Key**: your actual API key
   - **Model**: `deepseek-chat` (or your preferred model)
3. Save and restart Cursor

### Continue.dev

1. Open `~/.continue/config.json`
2. Add or modify the model configuration:
   ```json
   {
     "models": [
       {
         "title": "DeepSeek via Proxy",
         "provider": "openai",
         "model": "deepseek-chat",
         "apiBase": "http://127.0.0.1:8787",
         "apiKey": "your-api-key"
       }
     ]
   }
   ```
3. Restart Continue

### Aider

```bash
# Set the base URL before starting Aider
export OPENAI_API_BASE=http://127.0.0.1:8787
export OPENAI_API_KEY=your-api-key
aider --model deepseek-chat
```

### GitHub Copilot (BYOK)

VS Code Copilot supports custom endpoints via Bring Your Own Key:

1. Open Copilot Chat → model dropdown → **Manage Language Models**
2. Click **Add Models** → select **Custom Endpoint**
3. Set API Type to **Messages** (Anthropic format)
4. Set Endpoint URL to `http://127.0.0.1:8787/v1`

For enterprise teams, configure this at the org level:
**GitHub Organization Settings → AI Controls → Copilot → Custom Models**

### Environment Variables

You can also set the base URL via environment variable before starting VS Code:

```bash
# For Anthropic
set ANTHROPIC_BASE_URL=http://127.0.0.1:8787

# For OpenAI-compatible (DeepSeek, GPT, etc.)
set OPENAI_API_BASE=http://127.0.0.1:8787

# Start VS Code
code .
```

## Configuration

All settings are controlled via environment variables. Defaults are sensible for most use cases.

> **⚠️ Cache System Separation:** The proxy's cache system (controlled by `AUTO_CACHE`, `TAIL_TTL`) is **entirely separate** from the MCP server's `CacheConfig` in `.clean-ctx.json`. The proxy injects Anthropic API `cache_control` breakpoints into HTTP request bodies for API cost savings. The MCP server's `CacheConfig` controls `_meta.cache_hints` annotations in JSON-RPC responses for LLM context window optimization. These are independent systems — enabling one does not affect the other, and they have separate configuration paths (environment variables vs `.clean-ctx.json`).

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8787` | Port to bind on (always `127.0.0.1`) |
| `PROXY_UPSTREAM_URL` | _(none)_ | **Dedicated upstream URL** (takes precedence over all others). Use this when the client-facing `ANTHROPIC_BASE_URL` must point at the proxy itself. |
| `COPILOT_BRIDGE_URL` | _(none)_ | Alias for `PROXY_UPSTREAM_URL`. Convenient shorthand when forwarding to a local Copilot bridge (e.g. `http://127.0.0.1:4141`). |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | Legacy upstream URL. **Falls back** to the default when the dedicated vars above are set. |
| `AUTO_CACHE` | `false` | Enable cache breakpoint injection (Anthropic only) |
| `TAIL_TTL` | `5m` | TTL for the rolling-tail breakpoint |
| `DROP_TOOLS` | _(none)_ | Comma-separated tool names to remove (e.g. `NotebookEdit,CronCreate`) |
| `STRIP_ANSI` | `false` | Strip ANSI escape codes from text blocks (opt-in) |
| `TRIM_BASH_GIT` | `false` | Truncate Bash tool's git commit/PR sections |
| `MODEL_OVERRIDE` | _(none)_ | Override model name (e.g. `claude-opus-4-6`) |
| `LOG_BODIES` | `false` | Log request/response bodies to disk |
| `LOG_DIR` | `.clean-ctx/proxy-logs` | Directory for log files |
| `SCRUB_SECRETS` | `false` | Enable secret scrubbing in tool results |
| `TOOL_FILTERS` | `false` | Enable tool output filtering (TOML-based) |
| `PLATFORM` | _(auto-detect)_ | Override platform detection (`anthropic`, `openai`, `generic`) |
| `PROXY_API_KEY` | _(none)_ | Optional API key for `X-Api-Key` header authentication. When set, all requests must include a matching `X-Api-Key` header. Requests with missing/wrong key return `401 Unauthorized`. |
| `RATE_LIMIT_RPS` | `60` | Per-client requests per second (only enforced when `PROXY_API_KEY` is set). |
| `RATE_LIMIT_BURST` | `10` | Per-client burst window size — how many requests a client can make in a sudden burst before rate limiting kicks in. |

### Recommended Setup

```bash
AUTO_CACHE=1 DROP_TOOLS=NotebookEdit,CronCreate STRIP_ANSI=1 SCRUB_SECRETS=1 TOOL_FILTERS=1 cargo run -p clean-ctx-proxy
```

This enables all cost-saving features:
- **Cache injection** on tools, system prompt, and message tail (Anthropic only)
- **Tool dropping** for tools you never use
- **ANSI stripping** to remove terminal escape codes from tool results
- **Secret scrubbing** to redact AWS keys, GitHub tokens, JWTs, etc.
- **Tool output filtering** to compress verbose cargo/npm/pytest output

### GitHub Copilot Enterprise (Claude via Copilot Gateway)

Organizations running **GitHub Copilot Enterprise** serve Claude through Copilot's infrastructure — not AWS Bedrock or Anthropic directly. In this setup `ANTHROPIC_BASE_URL` points at an **ephemeral Copilot relay port** (e.g. `http://127.0.0.1:52644`) that changes every session.

Clean-CTX's proxy intercepts this by sandwiching itself between Claude Code and a **Copilot bridge**:

```
Claude Code
    │  ANTHROPIC_BASE_URL = http://127.0.0.1:8787
    ▼
Clean-CTX Proxy (8787)
    │  PROXY_UPSTREAM_URL = http://127.0.0.1:4141
    │  applies tool filtering, secret scrubbing,
    │  cache hints, sliding window
    ▼
copilot-proxy-api (4141)
    │  translates Anthropic → Copilot protocol
    │  handles GitHub Enterprise auth (Bearer token)
    ▼
GitHub Copilot Enterprise Gateway
    │
    ▼
Claude via Copilot
```

**Setup:**

1. **Run the Copilot bridge** — `copilot-proxy-api` on a stable port (4141). It handles GitHub Enterprise auth automatically using your GitHub token.

2. **Point the Clean-CTX proxy at the bridge:**
   ```bash
   PROXY_UPSTREAM_URL=http://127.0.0.1:4141 clean-ctx-proxy
   ```

3. **Point Claude Code at the Clean-CTX proxy** in `~/.claude/settings.json`:
   ```json
   {
     "env": {
       "ANTHROPIC_BASE_URL": "http://127.0.0.1:8787",
       "ANTHROPIC_API_KEY": "dummy"
     }
   }
   ```

The `dummy` key is intentional — auth is handled by the Copilot bridge using your GitHub token, not an Anthropic key. The bridge's `Authorization: Bearer` header passes through the proxy untouched (hop-by-hop headers are filtered, all others are forwarded).

> **Why a dedicated `PROXY_UPSTREAM_URL`?** Without it, the proxy inherits the client's `ANTHROPIC_BASE_URL` (which points at the proxy itself for Claude Code). This creates a **self-forwarding loop**. The dedicated var decouples the proxy's upstream target from the client-facing URL, and the proxy refuses to start if it detects this configuration (validation error: `self-forwarding loop detected`).

> **Note:** Some enterprise Copilot policies block reverse-engineered Copilot API access. Check with IT before investing in this setup.

## How It Works

### Request Flow

```
Client (Cline, Copilot, Cursor, etc.) → Proxy (127.0.0.1:8787) → AI API (Anthropic, OpenAI, DeepSeek, etc.)
```

1. Client sends a normal API request (`/v1/messages` or `/v1/chat/completions`)
2. Proxy intercepts it, parses the JSON body
3. **Platform detection** determines the API format (Anthropic, OpenAI, or Generic)
4. **Transforms** are applied (tool drop, ANSI strip, Bash trim, model override, secret scrub, tool filtering)
5. **Cache breakpoints** are injected (Anthropic only)
6. Modified request is forwarded to the upstream API
7. Response is returned to the client unchanged

Non-intercepted requests pass through untouched.

### Transform Pipeline

The proxy applies transforms in this order:

1. **Tool Drop** — Removes unused tools from `body.tools[]`
2. **ANSI Strip** — Removes `\x1B[...m` escape sequences from text blocks
3. **Bash Git Trim** — Truncates Bash description at "Committing changes"
4. **Model Override** — Rewrites model name in `model` field and system blocks
5. **Secret Scrub** — Redacts AWS keys, GitHub tokens, JWTs, PEM keys, etc.
6. **Tool Filtering** — Compresses verbose tool output using TOML-based filters

### Tool Output Filtering

The proxy includes **26 built-in filters** that compress verbose tool output:

| Filter | Program | What It Does |
|--------|---------|--------------|
| `cargo` | `cargo` | Compact cargo build/test/check/clippy output |
| `npm` | `npm/yarn/pnpm/bun` | Compact package manager install/build output |
| `git-diff` | `git` | Compact git diff/show output |
| `pytest` | `pytest` | Compact pytest output |
| `tsc` | `tsc` | Compact TypeScript compiler output |
| `dotnet` | `dotnet` | Compact dotnet build/test/run output |
| `ng` | `ng` | Compact Angular CLI build/test/lint output |
| `eslint` | `eslint` | Compact ESLint linting output |
| `ruff` | `ruff` | Compact Ruff Python linter output |
| `biome` | `biome` | Compact Biome lint/format output (filter_stderr) |
| `go` | `go` | Compact Go test/build/vet output |
| `make` | `make` | Compact GNU Make output |
| `pip` | `pip` | Compact pip install/sync output |
| `docker` | `docker` | Compact Docker build/pull/push/compose output |
| `docker-logs` | `docker logs` | Cap docker logs output |
| `gh` | `gh` | Compact GitHub CLI output |
| `curl` | `curl` | Compact curl output (filter_stderr) |
| `mvn` | `mvn` | Compact Maven build output |
| `mypy` | `mypy` | Compact mypy Python type checker output |
| `pyright` | `pyright` | Compact pyright type checker output |
| `shellcheck` | `shellcheck` | Compact shellcheck output |
| `golangci-lint` | `golangci-lint` | Compact golangci-lint output |
| `kubectl` | `kubectl` | Compact kubectl output |
| `apt` | `apt/apt-get` | Compact apt/apt-get output |
| `brew` | `brew` | Compact brew install/upgrade output |
| `pre-commit` | `pre-commit` | Compact pre-commit output |
| `ssh` | `ssh` | Compact ssh output |
| `systemctl` | `systemctl` | Compact systemctl status output |
| `hadolint` | `hadolint` | Compact hadolint Dockerfile linting output |
| `yamllint` | `yamllint` | Compact yamllint output |
| `node-build` | `npm/yarn/pnpm/bun run build` | Compact Node.js build output |
| `dotnet-build` | `dotnet build` | Compact dotnet build output |
| `dotnet-test` | `dotnet test` | Compact dotnet test output |

Filters are loaded from TOML files in the `filters/` directory. You can add custom filters by placing TOML files in `.clean-ctx/filters/`.

### Secret Scrubbing

The proxy detects and redacts secrets in tool results:

- AWS access keys (`AKIA...`)
- GitHub tokens (`ghp_...`, `gho_...`)
- JWTs (`eyJ...`)
- PEM private keys
- Authorization headers
- Database URLs
- Stripe keys (`sk_live_...`)
- Slack tokens (`xoxb-...`)
- Google API keys (`AIza...`)
- OpenAI keys (`sk-...`)
- PyPI tokens (`pypi-...`)
- Vault tokens (`hvs...`)

Secrets are replaced with `[REDACTED]` before they reach the LLM.

### Cache Breakpoint Strategy

The proxy follows the same proven strategy as [Pino](https://github.com/esetnik/pino):

- **Tools slot** gets a 1-hour TTL (via the `extended-cache-ttl` beta header)
- **System slot** targets the largest system block, avoiding waste on small blocks
- **Messages[0] slot** caches the first user message content
- **Tail slot** is a rolling cache that moves forward with each conversation turn

Any existing `cache_control` headers sent by the client are stripped first to avoid conflicts.

### Transform Details

| Transform | What It Does | Token Savings |
|-----------|-------------|---------------|
| Tool Drop | Removes unused tools from `body.tools[]` | ~24k tokens per dropped tool |
| ANSI Strip | Removes `\x1B[...m` escape sequences from text blocks | Varies |
| Bash Git Trim | Truncates Bash description at "Committing changes" | ~1,800 tokens |
| Model Override | Rewrites model name in `model` field and system blocks | — |
| Secret Scrub | Redacts AWS keys, GitHub tokens, JWTs, etc. | — |
| Tool Filtering | Compresses verbose tool output (cargo, npm, pytest, etc.) | 70-90% per tool result |

## Architecture

```
proxy/
├── src/
│   ├── main.rs              # Entry point, env-var parsing, Ctrl+C shutdown
│   ├── lib.rs               # Library root (re-exports for tests)
│   ├── server.rs            # HTTP server, routing, upstream forwarding
│   ├── cache.rs             # 4-slot cache breakpoint injection
│   ├── transform.rs         # Tool drop, ANSI strip, Bash trim, model override, secret scrub, tool filtering
│   ├── config.rs            # Pino-compatible env-var configuration
│   ├── logger.rs            # Request/response body logging
│   ├── error.rs             # Error types
│   ├── filters.rs           # TOML-based filter engine (7-step pipeline)
│   ├── filter_rules.rs      # Filter rule compilation (TOML → compiled regex)
│   ├── filter_registry.rs   # Filter selection (most-specific-match-wins)
│   ├── filter_loader.rs     # Filter loading (built-in + community)
│   ├── filter_stats.rs      # Per-program filter savings tracking
│   ├── community_filters.rs # Community filter loading from .clean-ctx/filters/
│   ├── scrub.rs             # Secret scrubbing engine
│   ├── scrub_patterns.rs    # Secret detection patterns
│   ├── pipeline.rs          # Pluggable transform pipeline (OCP compliance)
│   └── platform/
│       ├── mod.rs            # PlatformAdapter trait + detect_platform
│       ├── anthropic.rs      # Anthropic API adapter
│       ├── openai.rs         # OpenAI API adapter
│       └── generic.rs        # Generic fallback adapter
├── tests/
│   ├── integration_test.rs  # End-to-end test with mock upstream
│   └── audit_regression.rs  # 18 regression tests for all audit findings
├── filters/                 # Built-in TOML filter files (26 filters)
│   ├── cargo.toml           # cargo build/test/check/clippy
│   ├── npm.toml             # npm/yarn/pnpm/bun install/build
│   ├── git-diff.toml        # git diff/show
│   ├── pytest.toml          # pytest
│   ├── tsc.toml             # TypeScript compiler
│   ├── dotnet.toml          # dotnet build/test/run
│   ├── dotnet-build.toml    # dotnet build (split)
│   ├── dotnet-test.toml     # dotnet test (split)
│   ├── ng.toml              # Angular CLI
│   ├── eslint.toml          # ESLint
│   ├── ruff.toml            # Ruff Python linter
│   ├── biome.toml           # Biome lint/format
│   ├── go.toml              # go test/build/vet
│   ├── make.toml            # GNU Make
│   ├── pip.toml             # pip install/sync
│   ├── docker.toml          # docker build/pull/push
│   ├── docker-logs.toml     # docker logs
│   ├── gh.toml              # GitHub CLI
│   ├── curl.toml            # curl
│   ├── mvn.toml             # Maven build
│   ├── mypy.toml            # mypy type checker
│   ├── pyright.toml         # pyright type checker
│   ├── shellcheck.toml      # shellcheck
│   ├── golangci-lint.toml   # golangci-lint
│   ├── kubectl.toml         # kubectl
│   ├── apt.toml             # apt/apt-get
│   ├── brew.toml            # brew install/upgrade
│   ├── pre-commit.toml      # pre-commit
│   ├── ssh.toml             # ssh
│   ├── systemctl.toml       # systemctl status
│   ├── hadolint.toml        # hadolint
│   ├── yamllint.toml        # yamllint
│   └── node-build.toml      # npm/yarn/pnpm/bun run build
└── Cargo.toml
```

### Key Design Decisions

- **Pure Rust** — No external JS dependencies. All transforms run natively.
- **Stateless proxy** — No file-based caching (Anthropic handles that). The proxy only modifies request bodies.
- **Lock-light** — The shared state mutex is held only briefly for reads, never across async I/O.
- **Connection-per-request** — Each incoming connection is spawned independently for concurrency.
- **Platform-agnostic** — Works with Anthropic, OpenAI, DeepSeek, and any OpenAI-compatible API.
- **Pluggable transforms** — New transforms added via `Pipeline::build()` without modifying existing code.
- **TOML-based filters** — Easy to add custom filters without modifying code.

## Testing

```bash
# Run all proxy tests (unit + regression + integration)
cargo test -p clean-ctx-proxy

# Run only unit tests
cargo test -p clean-ctx-proxy --lib

# Run regression tests only (audit findings)
cargo test -p clean-ctx-proxy --test audit_regression

# Run integration test only
cargo test -p clean-ctx-proxy --test integration_test
```

The test suite includes 243 tests: 112 unit tests, 18 audit regression tests (covering all FAANG-principal code review findings), and 1 end-to-end integration test with a mock upstream server.

## Limitations

- **Response streaming**: The proxy buffers the full response from the upstream API before returning it to the client. This means Copilot and other IDEs will see responses as a complete block rather than token-by-token streaming. Response content is fully intact and correct — only the progressive rendering timing is affected.
- **Copilot requires BYOK or a bridge**: Default Copilot traffic routes through GitHub's own API gateway and never reaches the proxy. You must configure a custom endpoint (BYOK) or use a Copilot bridge (`copilot-proxy-api`) as described in the GitHub Copilot Enterprise section above.
- **Cache injection is Anthropic-only**: The `cache_control` breakpoint injection only works with Anthropic's API. Other providers don't support this feature.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `ECONNREFUSED` on port 8787 | Proxy isn't running. Start it with `cargo run -p clean-ctx-proxy` |
| Proxy refuses to start: `self-forwarding loop detected` | Upstream resolves to the proxy's own port (e.g. `ANTHROPIC_BASE_URL=http://127.0.0.1:8787` inherited by the proxy). Set `PROXY_UPSTREAM_URL` to the real upstream target. |
| Proxy returns 502 | Upstream URL is wrong or API is unreachable. Check `PROXY_UPSTREAM_URL` (or `ANTHROPIC_BASE_URL` for legacy setups) |
| Cache savings not showing | Make sure `AUTO_CACHE=1` is set (Anthropic only) |
| Tools still appear in request | Check `DROP_TOOLS` is set correctly (comma-separated, no spaces) |
| Copilot not using proxy | Make sure you've configured a custom endpoint in VS Code (not using default GitHub routing) |
| Response appears all at once | Expected — the proxy buffers responses. Content is correct; only streaming timing differs |
| Secrets not being scrubbed | Make sure `SCRUB_SECRETS=1` is set |
| Tool output not being filtered | Make sure `TOOL_FILTERS=1` is set |
| Wrong platform detected | Set `PLATFORM=anthropic` or `PLATFORM=openai` or `PLATFORM=generic` |

## License

CC0-1.0 (same as the main Clean-CTX crate)