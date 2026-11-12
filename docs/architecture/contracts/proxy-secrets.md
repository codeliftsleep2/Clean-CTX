# Bounded proxy secret scrubbing

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

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


Source: [docs/PROXY.md](../../PROXY.md); original section SHA-256 `5801248a4721942d3e568c82df1330d2a43ceb08a034100f414a8df747bfe89f`. Original wording and tense are retained; admission is limited to the implemented scope described here.


Existing rule replacements must propagate when content changes even if byte length is equal. The prefilter must admit the approved eight-root correction without changing regex rules. These are owner-approved FIX / PROTECTED obligations documented in [the exact-content repair](../../agent/AF_PERF_002_PROXY_SCRUB_DISCOVERY.md) and [the eight-root repair](../../agent/AF_PERF_002_PROXY_PREFILTER_DISCOVERY.md). Their tests and results remain attributed there; they are not re-executed or independently validated by B2. Unicode/rule-language completeness and universal secret coverage remain open.

```architecture-graph
node: AUTH.PROXY_SECRETS
kind: AUTHORITY
status: IMPLEMENTED
title: Bounded proxy secret scrubbing
track: REPOSITORY
artifact: docs/architecture/contracts/proxy-secrets.md
preserves:
  - AUTH.NETWORK_EFFECTS
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
