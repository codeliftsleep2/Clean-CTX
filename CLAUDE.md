# Clean-CTX usage

Use Clean-CTX as the primary code-intelligence layer.

- Start repository discovery with `graph_search`; do not routinely call
  `get_cbm_status` first.
- If graph discovery is unavailable, use `search_codebase`, then
  `provide_code_context`.
- Use `provide_code_context` first for supported `.ts`, `.cs`, `.rs`, and
  `.java` source.
- Always pass `workspaceRoot` explicitly.
- Treat returned repository-relative paths as authoritative; never derive paths
  from namespaces or CBM project names.
- Use `workspace_query` for semantic entities, relationships, dependencies,
  cycles, and calls.
- Use `apply_edit` after Edit context for supported structural edits.
- Do not manually reindex after `apply_edit`; the next graph operation performs
  lazy refresh.
- Use native file tools for unsupported files, exact line inspection, new
  files, signatures, renames, and cross-file edits.

Detailed reference: `docs/CLAUDE_INTEGRATION_RULES.md`.
