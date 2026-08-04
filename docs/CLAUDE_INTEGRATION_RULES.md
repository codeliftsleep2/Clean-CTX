# Clean-CTX Tool Enforcement

These rules are **mandatory**. They prevent Claude from bypassing Clean-CTX's compression pipeline, which wastes 4-5× more tokens and leaves the dashboard empty.

---

## RULE 1 — Code Files: Use `provide_code_context`, NOT `Read`

**NEVER use your native `Read` tool on code files** (`.ts`, `.js`, `.cs`, `.rs`, `.java`).

**ALWAYS use `provide_code_context` FIRST** — it compresses (85-96% savings), uses delta transport on repeat calls, and records savings.

`Read` is allowed ONLY for:
- Non-code files (markdown, JSON, TOML, config)
- After `provide_code_context` fails
- Exact line-by-line byte analysis

Always pass an `intent`: `"overview"` | `"edit"` | `"refactor"` | `"debug"` | `"implement"`

```xml
<use_mcp_tool>
<server_name>clean-ctx</server_name>
<tool_name>provide_code_context</tool_name>
<arguments>
{
  "filePath": "src/services/UserService.ts",
  "intent": "edit"
}
</arguments>
</use_mcp_tool>
```

---

## RULE 2 — CBM Queries: Use `cbm_proxy`, NOT Direct Calls

**NEVER call `graph_search`, `graph_query`, `graph_trace`, `get_architecture`, `get_symbol_importance`, or `get_dead_code` directly.** They return raw ~5000-token responses that bypass compression.

**ALWAYS use `cbm_proxy`** — it intercepts the response at the pipe level and compresses it to ~1100 tokens.

| FORBIDDEN direct call | Use `cbm_proxy` with |
|-----------------------|----------------------|
| `graph_search` | `cbm_tool: "graph_search"` |
| `graph_query` | `cbm_tool: "graph_query"` |
| `graph_trace` | `cbm_tool: "graph_trace"` |
| `get_architecture` | `cbm_tool: "get_architecture"` |
| `get_symbol_importance` | `cbm_tool: "get_symbol_importance"` |
| `get_dead_code` | `cbm_tool: "get_dead_code"` |

**Only direct CBM call allowed:** `get_cbm_status` (tiny status object).

```xml
<use_mcp_tool>
<server_name>clean-ctx</server_name>
<tool_name>cbm_proxy</tool_name>
<arguments>
{
  "cbm_tool": "graph_search",
  "query": "UserService",
  "project": "my-project"
}
</arguments>
</use_mcp_tool>
```

---

## Quick Reference

| Need | Use |
|------|-----|
| Code file context | `provide_code_context` |
| Non-code file | `Read` |
| CBM graph query | `cbm_proxy` |
| CBM availability | `get_cbm_status` |
| Token savings | `context_stats` |

**Before completing any task, verify:** every code file used `provide_code_context`, every CBM query used `cbm_proxy`, and `Read` was only used for non-code files or after `provide_code_context` failed.