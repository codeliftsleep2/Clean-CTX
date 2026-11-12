# Proxy request transformation boundary

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

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


Source: [docs/PROXY.md](../../PROXY.md); original section SHA-256 `c7a6355e41bf852790c9fcaaa377e9b0f12b26585a4571f4099d7cf6a0f75e9c`. Original wording and tense are retained; admission is limited to the implemented scope described here.

### Cache Breakpoint Strategy

The proxy follows the same proven strategy as [Pino](https://github.com/esetnik/pino):

- **Tools slot** gets a 1-hour TTL (via the `extended-cache-ttl` beta header)
- **System slot** targets the largest system block, avoiding waste on small blocks
- **Messages[0] slot** caches the first user message content
- **Tail slot** is a rolling cache that moves forward with each conversation turn

Any existing `cache_control` headers sent by the client are stripped first to avoid conflicts.


Source: [docs/PROXY.md](../../PROXY.md); original section SHA-256 `1283d4f119d9975fc5d93b336ec0ed50f3701d29fcd0f65266568e5045ae6db6`. Original wording and tense are retained; admission is limited to the implemented scope described here.


The ordered pipeline also has optional aging where configured, as recorded in [the pipeline audit](../../agent/AF_PERF_002_PROXY_SCRUB_DISCOVERY.md). Logging has a separate fail-closed scrub boundary; this request contract does not certify logging.

```architecture-graph
node: AUTH.PROXY_TRANSFORMS
kind: AUTHORITY
status: IMPLEMENTED
title: Proxy request transformation boundary
track: REPOSITORY
artifact: docs/architecture/contracts/proxy-transforms.md
preserves:
  - AUTH.NETWORK_EFFECTS
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
