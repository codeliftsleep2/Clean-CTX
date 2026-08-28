# Clean-CTX Tool Enforcement

These rules are **mandatory**. They prevent Claude from bypassing Clean-CTX's compression pipeline, which wastes 4-5× more tokens and leaves the dashboard empty.

---

## RULE 1 — Code Files: Use `provide_code_context`, NOT `Read`

**NEVER use your native `Read` tool on code files** (`.ts`, `.js`, `.cs`, `.rs`, `.java`).

**ALWAYS use `provide_code_context` FIRST** — it compresses code, uses delta transport on repeat calls, and records savings.

`Read` is allowed ONLY for:
- Non-code files (markdown, JSON, TOML, config)
- After `provide_code_context` fails
- Exact line-by-line byte analysis

Always pass an `intent`: `"overview"` | `"edit"` | `"refactor"` | `"debug"` | `"implement"`

```
mcp__clean-ctx__provide_code_context(filePath: "src/services/UserService.ts", intent: "edit")
```

---

## RULE 1b — Single-Unit Edits: Use `apply_edit`, NOT the Host Write Tool

**For a SINGLE-UNIT edit** (replace one method body, insert one method after an anchor, delete one method) on a file you have already read at `fidelity: "edit"` or `fidelity: "verbatim"` **in this same session**, use `apply_edit` instead of the host's native write/edit tool.

**NEVER use the host write tool immediately after `provide_code_context` when the edit targets a single structural unit.** The host tool forces a full raw re-read of the *entire* file (thousands of wasted tokens) purely to satisfy its own staleness precondition. `apply_edit` verifies only the bytes actually being changed — against the unit's current span — then runs an in-memory tree-sitter gate before any byte hits disk.

```
mcp__clean-ctx__apply_edit(filePath: "src/services/UserService.ts",
  operations: [{
    type: "replace_body",
    target: "UserService.processOrder",       # qualified name "Class.method" (or "M3" / unambiguous bare name)
    expectedOldText: "{ ...byte-exact current body from provide_code_context... }",
    newText: "{ ...replacement body... }"
  }])
```

**When NOT to use `apply_edit`** (still use the host write tool):
- Cross-file edits, renames, or signature changes (effects at other call sites).
- Brand-new files that were never read via `provide_code_context` (v1 policy: `apply_edit` requires prior tracked state).
- Multi-unit edits that span whole classes or multiple unrelated regions in ways the operation shapes don't cover.

`apply_edit` forms: `{type: "replace_body", target, expectedOldText, newText}`, `{type: "delete", target, expectedOldText}`, `{type: "insert_after", anchor, unitText}`, `{type: "insert_before", anchor, unitText}`. Add `"verify": true` to echo the new text back as a receipt. A rejected edit means the unit changed underneath you — re-read with `provide_code_context` and retry; never retry blindly.

---

## RULE 2 — CBM Queries: Use `cbm_proxy`, NOT Direct Calls

**NEVER call `search_graph`, `query_graph`, `trace_path`, `get_architecture`, `list_projects`, or `index_repository` directly.** They return raw, uncompressed responses that bypass compression.

**ALWAYS use `cbm_proxy`** — it intercepts the response at the pipe level and compresses it before it reaches you.

**`cbm_tool` must be a REAL CBM tool name:** `search_graph`, `query_graph`, `trace_path`, `get_architecture`, `list_projects`, `index_repository`. `get_symbol_importance` and `get_dead_code` are NOT CBM tools — they are implemented internally via `query_graph` Cypher, so never pass them as `cbm_tool`.

| FORBIDDEN direct call | Use `cbm_proxy` with |
|-----------------------|----------------------|
| `search_graph` | `cbm_tool: "search_graph"`, `name_pattern` |
| `query_graph` | `cbm_tool: "query_graph"`, `query` |
| `trace_path` | `cbm_tool: "trace_path"`, `function_name`, `direction` |
| `get_architecture` | `cbm_tool: "get_architecture"`, `project` |
| `list_projects` | `cbm_tool: "list_projects"`, `parameters: {}` |
| `index_repository` | `cbm_tool: "index_repository"`, `parameters: { repo_path, mode? }` |

**Only direct CBM call allowed:** `get_cbm_status` (tiny status object).

Search:
```
mcp__clean-ctx__cbm_proxy(cbm_tool: "search_graph", parameters: { name_pattern: "UserService", project: "my-project" })
```

Trace:
```
mcp__clean-ctx__cbm_proxy(cbm_tool: "trace_path", parameters: { function_name: "processPayment", direction: "outbound", project: "my-project" })
```

---

## Quick Reference

| Need | Use |
|------|-----|
| Code file context | `provide_code_context` |
| Non-code file | `Read` |
| CBM graph query | `cbm_proxy` |
| CBM availability | `get_cbm_status` |
| CBM project list | `cbm_proxy` with `cbm_tool: "list_projects"` |
| CBM reindex | `cbm_proxy` with `cbm_tool: "index_repository"` (only needed for external edits) |
| Token savings | `context_stats` |

**After `apply_edit`:** Automatic synchronous CBM `fast` reindex runs when CBM is available — no manual `index_repository` needed.

**After external edits** (host write tool, shell, git operation): Clean-CTX cannot observe the mutation. Use `cbm_proxy(cbm_tool: "index_repository", parameters: { repo_path, mode: "fast" })` explicitly if graph freshness is required.

**Before completing any task, verify:** every code file used `provide_code_context`, every CBM query used `cbm_proxy`, and `Read` was only used for non-code files or after `provide_code_context` failed.