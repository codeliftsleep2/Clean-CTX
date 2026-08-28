# Graph Tool Surface Audit Report

**Date:** 2026-08-27
**Auditor:** Clean-CTX Code Audit
**Scope:**
- Live comparison of the proxied graph path (`cbm_proxy`) against the direct wrapper tools (`graph_search`, `graph_query`, `graph_trace`) against a real multi-repo workspace ("Meridian" below — a fictional stand-in for the actual client codebase, consistent with this repo's existing audit convention)
- Full pickaxe history review (`git log -S"<name>" -- '*.rs'`) of every tool name referenced in downstream consumer documentation
- Inspection of `cbm_proxy`'s own schema vs. its observed accepted inputs
- Config vs. runtime behavior check for multi-root indexing

**Objective:** Validate each finding against source and live behavior, determine root cause, and recommend next steps. No fixes applied.

---

## Summary of Findings

| Issue | Status | Severity | Root Cause |
|---|---|---|---|
| 1 — `graph_search` returns no usable data | **VALIDATED** | 🔴 High | Direct wrapper tool returns a bare count string; does not surface records the proxied path returns in full for the identical query |
| 2 — `cbm_proxy` accepts undocumented `cbm_tool` values | **VALIDATED** | 🟡 Medium | Schema documents 4 values; at least 2 more (`index_repository`, `list_projects`) are accepted and functional, unenforced by the schema |
| 3 — Circular error message on missing `project` | **VALIDATED** | 🟡 Medium | Error tells the caller to run a "list projects" op that isn't in the tool's own documented value set (it's Finding 2's undocumented value) |
| 4 — Consumer docs reference retired/never-implemented tools | **VALIDATED** | 🟡 Medium | 1 tool genuinely retired (docs never resynced downstream); 7 tool names never existed in this repo's history at all; 2 more are internal-only and never exposed |
| 5 — `graph_trace`/`graph_query` reliability | **UNCONFIRMED** | ⚪ Needs retest | Spot check returned terse zero-result summaries against a symbol pair without a verified real relationship — inconclusive either way |
| 6 — Multi-root indexing persists without a config key | **VALIDATED** | 🟢 Low | A secondary project queries successfully with real data despite the workspace config having no `additional_roots`-equivalent entry |

---

## Finding 1 — `graph_search` returns no usable data, while `cbm_proxy` returns full records for the identical call

Calling both paths with identical inputs (same symbol-name query, same project) against the Meridian workspace:

- `cbm_proxy(cbm_tool: "search_graph", query: "<symbol>", project: "<slug>")` → 14 complete records: qualified names, file paths, in/out-degree, signatures.
- `graph_search(query: "<symbol>", project: "<slug>")` → `"Found 14 symbol(s)."` — no names, no paths, no other fields.

Not a compression-format difference — the direct wrapper simply doesn't emit the data. It returns success, which is the concerning part: a caller has no signal that anything is missing.

**Repro:** any indexed project, any symbol name with ≥1 real match. Call `graph_search` and `cbm_proxy(search_graph)` with identical `query`/`project` and diff output shape.

**Recommendation:** either fix `graph_search` to emit the same record set `cbm_proxy` does, or have it fail/warn rather than silently truncate to a count.

---

## Finding 2 — `cbm_proxy` silently accepts `cbm_tool` values outside its documented set

The tool's own parameter schema states `cbm_tool` "must be a real CBM tool name" from a set of four. In practice, at least two more values are accepted and functional:

- `"index_repository"` — triggers a repository reindex. Confirmed via the proxy's own internal bridge code (background-indexing call, same call shape) and via a hook script in a consuming project's tooling that already relies on this as the only way to force a refresh after an edit.
- `"list_projects"` — lists indexed project slugs. Confirmed via an existing end-to-end test in this repo's own suite exercising this exact call.

**Recommendation:** either document these two as first-class supported values, or add real enforcement so the schema description matches actual behavior.

---

## Finding 3 — Circular error message when `project` is omitted

`cbm_proxy` without an explicit `project` fails (correctly, for a multi-project workspace) with an error directing the caller to run a "list projects" call to discover valid values. That operation is not one of the four values the tool's schema documents (see Finding 2) — a caller going by the schema alone hits a dead end resolving the error.

**Recommendation:** document the `list_projects` pass-through so the error is actionable, or have the error return available slugs directly instead of naming a separate call.

---

## Finding 4 — Downstream documentation lists retired or never-implemented tools

Reviewing docs maintained by a consuming project (describing how to use this integration) surfaced a "direct tool" table containing:

- One tool that **was real and was deliberately retired** — the `Phase C1/C2` legacy-workspace-compression retirement (net −4,748 lines), with this repo's own README correctly updated in the same window. The downstream docs were never resynced after that.
- Seven tool names that a full pickaxe search across this repo's entire history returns **zero hits for, ever** — never implemented here at all. They read as copied from the underlying graph service's own generic capability list without verifying they were wired into this integration.
- Two more names that are real but are **internal-only** — calls this repo's own bridge makes to the graph service for its own housekeeping (cache invalidation, startup indexing), never exposed externally under those names.

**Recommendation:** when a public-surface tool is retired, consider a downstream-facing changelog note (beyond this repo's own README) — "the source of truth was fixed" doesn't propagate to every consumer's local notes automatically.

---

## Finding 5 — `graph_trace`/`graph_query` reliability unconfirmed (flagged, not resolved)

Spot checks against `graph_trace`/`graph_query` returned terse zero-result summaries (`"0 edge(s)"`, `"N node(s), 0 edge(s)"`). The symbol pair used did not have a verified real relationship going in, and one query had no edge pattern at all — so a zero result is expected either way and proves nothing about tool health.

**Recommendation:** rerun both against a symbol pair with a *known* real edge (e.g., a controller method calling directly into its backing service interface method) and compare the edge count against `cbm_proxy`'s equivalent call on the same pair before drawing any conclusion.

---

## Finding 6 — Multi-root indexing persists without a corresponding config entry

A workspace's `.clean-ctx.json` has no entry declaring additional indexed repository roots beyond the primary project. Despite that, querying a known secondary repository through `cbm_proxy(get_architecture)` returned a full, real payload — thousands of nodes/edges, correct per-language breakdown — i.e., still indexed and queryable with no visible config explaining why.

**Recommendation:** not a functional break today, but worth surfacing what currently keeps that index alive (a prior config value that was later removed? a DB-persisted state independent of config?) so it doesn't silently vanish on a fresh machine, a service restart, or a future config edit.

---

## Notes on method

All findings above are either directly observed via live tool calls against a real (anonymized) workspace, or sourced to a specific commit in this repository via `git log -S` pickaxe search across full history — not inferred from tool names or descriptions alone. Finding 5 is explicitly left open rather than asserted either way, since the evidence available doesn't settle it.
