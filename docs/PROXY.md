# Clean-CTX Anthropic Proxy

A Rust HTTP proxy that sits between your LLM client and the Anthropic API, automatically injecting prompt-cache breakpoints to achieve ~90% API cost savings on cached turns.

Works with any client that sends Anthropic-format `POST /v1/messages` requests: Cline, Cursor, Aider, Continue.dev, GitHub Copilot (BYOK), and custom Anthropic clients.

## Why?

Claude's prompt caching can reduce costs by 90% on the tools + system prompt portions of your requests, but LLM clients don't send the `cache_control` headers needed to activate it. This proxy intercepts every `/v1/messages` request, injects the headers, and forwards the modified request to Anthropic — no client configuration required.

## Quick Start

```bash
# Build and run
cargo run -p clean-ctx-proxy

# Or with full optimization
cargo run --release -p clean-ctx-proxy
```

The proxy binds to `http://127.0.0.1:8787` by default. Point your client at it:

### Cline / Cursor / Aider / Continue.dev

```bash
# PowerShell
$env:ANTHROPIC_BASE_URL = "http://127.0.0.1:8787"

# Bash
export ANTHROPIC_BASE_URL=http://127.0.0.1:8787
```

### GitHub Copilot (BYOK)

VS Code Copilot supports custom endpoints via Bring Your Own Key:

1. Open Copilot Chat → model dropdown → **Manage Language Models**
2. Click **Add Models** → select **Custom Endpoint**
3. Set API Type to **Messages** (Anthropic format)
4. Set Endpoint URL to `http://127.0.0.1:8787/v1`

For enterprise teams, configure this at the org level:
**GitHub Organization Settings → AI Controls → Copilot → Custom Models**

## Configuration

All settings are controlled via environment variables. Defaults are sensible for most use cases.

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8787` | Port to bind on (always `127.0.0.1`) |
| `ANTHROPIC_BASE_URL` | `https://api.anthropic.com` | Upstream Anthropic API URL |
| `AUTO_CACHE` | `false` | Enable cache breakpoint injection |
| `TAIL_TTL` | `5m` | TTL for the rolling-tail breakpoint |
| `DROP_TOOLS` | _(none)_ | Comma-separated tool names to remove (e.g. `NotebookEdit,CronCreate`) |
| `STRIP_ANSI` | `false` | Strip ANSI escape codes from text blocks (opt-in) |
| `TRIM_BASH_GIT` | `false` | Truncate Bash tool's git commit/PR sections |
| `MODEL_OVERRIDE` | _(none)_ | Override model name (e.g. `claude-opus-4-6`) |
| `LOG_BODIES` | `false` | Log request/response bodies to disk |
| `LOG_DIR` | `.clean-ctx/proxy-logs` | Directory for log files |

### Recommended Setup

```bash
AUTO_CACHE=1 DROP_TOOLS=NotebookEdit,CronCreate STRIP_ANSI=1 cargo run -p clean-ctx-proxy
```

This enables all cost-saving features:
- **Cache injection** on tools, system prompt, and message tail
- **Tool dropping** for tools you never use
- **ANSI stripping** to remove terminal escape codes from tool results

## How It Works

### Request Flow

```
Client (Cline, Copilot, Cursor, etc.) → Proxy (127.0.0.1:8787) → Anthropic API
```

1. Client sends a normal `/v1/messages` request
2. Proxy intercepts it, parses the JSON body
3. **Transforms** are applied (tool drop, ANSI strip, Bash trim, model override)
4. **Cache breakpoints** are injected at 4 slots:
   - **Slot 1**: Last tool in `body.tools[]`
   - **Slot 2**: Largest `system` text block (>500 chars)
   - **Slot 3**: Last cacheable block in `messages[0].content`
   - **Slot 4**: Rolling tail — last text/tool_result across all messages
5. The `anthropic-beta: extended-cache-ttl-2025-04-11` header is added
6. Modified request is forwarded to Anthropic
7. Response is returned to the client unchanged

Non-`/v1/messages` requests pass through untouched.

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

## Architecture

```
proxy/
├── src/
│   ├── main.rs          # Entry point, env-var parsing, Ctrl+C shutdown
│   ├── lib.rs           # Library root (re-exports for tests)
│   ├── server.rs        # HTTP server, routing, upstream forwarding
│   ├── cache.rs         # 4-slot cache breakpoint injection
│   ├── transform.rs     # Tool drop, ANSI strip, Bash trim, model override
│   ├── config.rs        # Pino-compatible env-var configuration
│   ├── logger.rs        # Request/response body logging
│   └── error.rs         # Error types
├── tests/
│   ├── integration_test.rs    # End-to-end test with mock upstream
│   └── audit_regression.rs   # 18 regression tests for all audit findings
└── Cargo.toml
```

### Key Design Decisions

- **Pure Rust** — No external JS dependencies. All transforms run natively.
- **Stateless proxy** — No file-based caching (Anthropic handles that). The proxy only modifies request bodies.
- **Lock-light** — The shared state mutex is held only briefly for reads, never across async I/O.
- **Connection-per-request** — Each incoming connection is spawned independently for concurrency.

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

The test suite includes 51 tests: 32 unit tests, 18 audit regression tests (covering all FAANG-principal code review findings), and 1 end-to-end integration test with a mock Anthropic server.

## Limitations

- **Response streaming**: The proxy buffers the full response from Anthropic before returning it to the client. This means Copilot and other IDEs will see responses as a complete block rather than token-by-token streaming. Response content is fully intact and correct — only the progressive rendering timing is affected.
- **Copilot requires BYOK**: Default Copilot traffic routes through GitHub's own API gateway and never reaches the proxy. You must configure a custom endpoint as described above.

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `ECONNREFUSED` on port 8787 | Proxy isn't running. Start it with `cargo run -p clean-ctx-proxy` |
| Proxy returns 502 | Upstream URL is wrong or Anthropic is unreachable. Check `ANTHROPIC_BASE_URL` |
| Cache savings not showing | Make sure `AUTO_CACHE=1` is set |
| Tools still appear in request | Check `DROP_TOOLS` is set correctly (comma-separated, no spaces) |
| Copilot not using proxy | Make sure you've configured a custom endpoint in VS Code (not using default GitHub routing) |
| Response appears all at once | Expected — the proxy buffers responses. Content is correct; only streaming timing differs |

## License

CC0-1.0 (same as the main Clean-CTX crate)