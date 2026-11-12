# MCP cache hints and proxy cache transport

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

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
| `provide_code_context` | `baseline` | `bl_<sha256 of compressed output>` | `baseline_ttl` |
| `delta_code_context` | `tail` | `rolling` | `tail_ttl` |
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


Source: [docs/CONFIGURATION.md](../../CONFIGURATION.md); original section SHA-256 `851f4ef6091e84fc082cd008c58da9e299afdac9193c011477bc5869e3d878d8`. Original wording and tense are retained; admission is limited to the implemented scope described here.

## Two Cache Systems

Clean-CTX has **two independent** prompt-cache breakpoint systems that target different layers:

### 1. MCP-side `_meta.cache_hints` (client hinting)

The MCP server injects `_meta.cache_hints` into **JSON-RPC responses** sent back to the MCP client (Cline, Claude Code). These hints instruct the *client* to set `cache_control` breakpoints on its own request payloads.

- Regions: `system_prompt`, `tools`, `baseline`, `tail`
- Controlled by: `cache.enabled` in `.clean-ctx.json`
- TTLs: `cache.system_prompt_ttl` (1h), `cache.tools_ttl` (1h), `cache.baseline_ttl` (1h), `cache.tail_ttl` (5m)

### 2. Proxy-side HTTP `cache_control` injection

The `clean-ctx-proxy` intercepts the raw HTTP **request** from the client to `api.anthropic.com` and injects `cache_control` breakpoints directly into the system, tools, and message blocks.

- Slots: tools, system (last block > 500 chars), messages[0], tail
- Controlled by: `proxy.auto_cache` in `.clean-ctx.json`
- TTL: `proxy.tail_ttl` (default `"5m"` — keep this at 5m; the tail changes every turn and a longer TTL forces re-writes)

### How they relate

These systems are **complementary** — the MCP hints target the client's request construction, while the proxy targets the HTTP stream. They are **not** mutually exclusive. If the client already sent its own `cache_control` breakpoints, the proxy's injection is **skipped** (client-sent breakpoints are presumed intentional and never clobbered).

### Cache-state transport (Phase 5)

The MCP server queries the proxy's `GET /cache/state` endpoint at startup to learn the proxy's actual cache configuration:

```
[clean-ctx] Proxy cache state: auto_cache=true tail_ttl=5m
```

This bridges the two systems so the MCP server can align its `_meta.cache_hints` breakers with the proxy's real injection behavior.

Source: [docs/CONFIGURATION.md](../../CONFIGURATION.md); original section SHA-256 `83931f2334e544e0e3bc8ddfddc60b27c0da88f38dee280929a610d9b9f29b77`. Original wording and tense are retained; admission is limited to the implemented scope described here.


```architecture-graph
node: AUTH.CACHE_HINTS
kind: AUTHORITY
status: IMPLEMENTED
title: MCP cache hints and proxy cache transport
track: REPOSITORY
artifact: docs/architecture/contracts/cache-hints.md
preserves:
  - CONTENT-001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
