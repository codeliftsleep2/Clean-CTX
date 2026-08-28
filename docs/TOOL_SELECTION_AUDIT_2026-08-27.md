# Tool Selection Audit Report (Full Detail)

**Date:** 2026-08-27
**Auditor:** Clean-CTX Code Audit
**Scope:**
- Full tool-selection audit of the Clean-CTX / graph-service / MCP surface as consumed by a real multi-repo client workspace (referred to below as **"Meridian"** — a fictional stand-in, consistent with this repo's existing audit convention of anonymizing the client codebase; see `LINGUAFORGE_ISSUES_AUDIT.md` for precedent).
- Meridian is a 5-repository workspace: one primary backend/API project, plus four sibling repositories (an Angular frontend, a serverless-functions project, an LMS-integration proxy, and a shared core-data API), with one additional lint/runtime-audit tooling repository outside that immediate family.
- Live tool calls against Meridian's indexed graph, full commit-history pickaxe search of this repo, inspection of PreToolUse/PostToolUse/SubagentStart/SessionStart hook source, and review of Meridian's own two consumer-side rule documents (a global trigger-loaded skill doc and a path-scoped project instructions doc).

**Objective:** Determine the actual capability and behavior of every tool in this surface (not inferred from names/descriptions alone), build a tool-selection model, define intended workflows, and audit Meridian's own consumer-facing rules against verified reality. Investigatory only — no files were modified in Meridian, and this report itself is the only file added here.

**Companion document:** `GRAPH_TOOL_SURFACE_AUDIT_2026-08-27.md` in this same directory covers the subset of findings below that are actionable as bugs/gaps in this tool itself, in the shorter format used for that kind of report. This document is the full working audit those findings were drawn from.

---

## 0 — Summary

Clean-CTX is the only MCP server Meridian's agent has registered; the graph service (an internally-spawned subprocess) is never its own server. That single fact explains most of what's wrong with Meridian's own consumer docs: both of its rule documents describe a table of "direct graph-service calls" that are either retired, never real, or internal-only — not reachable under any name those docs give.

History confirms one genuine retirement (a workspace-level compression tool, deliberately removed and correctly scrubbed from this repo's own README — just never resynced downstream in Meridian) and shows seven other "direct call" entries were never implemented in this repo at all, at any point in its history. Separately, three of this repo's own "direct" graph wrapper tools sit completely undocumented in both of Meridian's rule files, and a live test shows one of them returning a bare count with no data at all, while the proxied path returns full records for the identical query. Meanwhile the proxy tool itself silently accepts at least two working values beyond the four its own schema documents.

None of this is catastrophic — the mechanical hooks that gate raw file-read/search calls on code files are in fact the most accurate layer in the whole system — but the two markdown rule files Meridian's agent actually reads are stale enough to send it toward tools that don't exist, and silent about tools that do.

---

## 1 — Architecture, confirmed

`claude mcp list`-equivalent output in the Meridian session shows exactly one registered server: the Clean-CTX binary. Its own config file confirms why nothing else appears: the graph-service binary path is configured with `auto_launch: true` — Clean-CTX spawns it itself and proxies it on a local port. The graph service's own tool vocabulary, whatever it documents for itself, is invisible to Meridian's agent except through whatever Clean-CTX chooses to forward.

| Layer | What it is | Reachable how |
|---|---|---|
| Clean-CTX binary | The one registered MCP server | All tool calls |
| Graph-service binary | Graph-intelligence subprocess, spawned internally | Only via the proxy tool, an architecture-overview tool, a status tool, and three "direct" graph wrapper tools |
| A second proxy-adjacent binary present on disk | Undocumented role in any rule file or hook | Not exposed as an MCP tool; purpose unconfirmed |

---

## 2 — Tool inventory

### 2.1 File context & edit tools — real, in active use

| Tool (role) | Status | Does | Mutating |
|---|---|---|---|
| Provide-code-context | Confirmed | Auto-compresses one file. A fidelity parameter runs low→verbatim; an intent parameter tunes it; a focus-methods parameter (only meaningful alongside edit-fidelity) renders byte-exact bodies for named methods only, signature-only elsewhere | No |
| Apply-edit | Confirmed | Structural single-unit edit (replace-body / delete / insert-after / insert-before), gated by an in-memory parse check and a byte-exact match against the tool's own tracked span for that unit | **Yes** |
| Diff-commits | Confirmed | Whole-workspace git-ref diff as AST-level change-sets, one call | No |
| Stats / history / sessions dashboard tools | Confirmed | Compression-savings dashboard — file-context domain only | No |
| Lower-level compression primitive | Unverified role | Sits behind provide-code-context; manual encoding/tokenizer control. No rule anywhere says when a caller should reach for this directly instead of the higher-level tool | No |
| Two single-file diff/delta tools | Unverified overlap | Both described as single-file, in-session (AST diff vs. IR-level delta) — the distinction between the two is never documented anywhere a caller would see it | No |
| Persistence-maintenance tools (save/restore/replay/purge) | Out of normal workflow | DB-level maintenance for the compression persistence store; no rule ever calls for these during ordinary work | DB only |

### 2.2 Graph access — two unequal paths

The same underlying graph is reachable two ways in this integration. Live test, identical query (a real domain-service class name), identical project, run against Meridian's indexed primary repository:

- **Proxied path** (`cbm_tool: "search_graph"`, full canonical project slug) → **14 full records**: qualified names, file paths, in/out-degree, signatures, complexity metrics — complete.
- **Direct wrapper tool** (`graph_search`, identical query and project) → **`"Found 14 symbol(s)."`** — no names, no paths, no other field. Same inputs, same project, same underlying graph.

| Tool | Status | Note |
|---|---|---|
| Proxy tool (primary path) | Confirmed, primary | Requires the **full canonical project slug** (a workspace-path-derived identifier), confirmed live — an unqualified call fails, and its own error message points the caller at a "list projects" capability that is not independently callable under its own name (see §4) |
| Architecture-overview tool | Confirmed | Live-tested against Meridian's Angular frontend sibling repository: real payload, several thousand nodes and edges, correct per-language (TS/HTML/SCSS) breakdown |
| Status tool | Confirmed | The one safe, cheap, always-direct call |
| Direct "search" wrapper | Confirmed low-value | Returns a bare count, no data — not mentioned in either of Meridian's rule documents, so nothing warns an agent away from it |
| Direct "trace"/"query" wrappers | Unverified | Terse one-line summaries in spot checks (e.g. "0 edge(s)"); the symbols used weren't a confirmed-linked pair, so this neither proves nor disproves correctness — see §7 |

### 2.3 The "direct graph-service call" table — reclassified against commit history

Both of Meridian's consumer-side rule documents list a table of tools callable "directly" against the graph service. A full pickaxe search (`git log -S"<name>" -- '*.rs'`) across this repo's entire history settles each one:

| Name in Meridian's docs | Status | What history actually shows |
|---|---|---|
| A workspace-level compression tool | **Retired** | Real once. Removed in a "Phase C1/C2: retire legacy workspace compression" commit (net −4,748 lines across 39 files). This repo's own README was correctly scrubbed of it in the same window across three follow-up docs commits — Meridian's own docs were simply never resynced afterward |
| Seven distinct tool names (a code-search tool, a graph-schema tool, a code-snippet tool, an ADR-management tool, a trace-ingestion tool, an index-status tool, a project-deletion tool) | **Never real** | **Zero hits, ever, in any commit, for any of the seven.** Not retired — never implemented for this integration at all. They read as lifted from the underlying graph service's own generic capability list without ever being wired in |
| A change-detection tool | **Internal-only** | Real, but it's this repo's own bridge code calling the graph service internally, to invalidate its own cache on a graph-version bump. Never surfaced to an external caller under that name |
| A repository-reindex tool | **Undocumented pass-through** | Also internal in origin (background indexing kicked off at bridge construction) — but confirmed reachable today as an undocumented value passed through the proxy tool's own `cbm_tool` parameter. Currently the only way to force a reindex after an edit |
| A project-listing tool | **Undocumented pass-through** | An end-to-end test in this repo's own suite exercises this exact call through the proxy tool's `cbm_tool` parameter directly — a second confirmed working value beyond the four the schema documents, and the actual resolution to the proxy tool's circular error message (§4) |

> **Not the same migration.** A separate, real commit corrected this repo's own system prompt to teach the compression *notation* it actually renders (structural letter codes for classes/methods/fields/imports) rather than a retired symbolic legend from the old text compressor. It landed in the same week as the compression-tool retirement above, which is likely why the two read as one event from the outside — they are unrelated changes.

---

## 3 — Tool-selection matrix

| Task | Preferred tool | Required mode / args | Follow-up | Avoid |
|---|---|---|---|---|
| First look at a file | Provide-code-context | overview intent | — | raw file-read on code files |
| Investigate a bug | Provide-code-context | debug intent | Proxy tool's trace call if cross-file | direct search/trace wrappers for real discovery |
| About to touch 1–2 known methods | Provide-code-context | edit fidelity **+ focus-methods** | Apply-edit | unfocused edit fidelity — costs *more* than a raw read (measured: 2,956 vs. 2,744 tokens on a real 6-method file) |
| Single-unit method edit | Apply-edit | byte-exact prior body from the last provide-code-context call | automatic parse-gate | generic host edit tool (forces a full raw re-read first) |
| Rename, signature change, new file, multi-unit edit | Generic host edit tool | prior raw read (host requirement, not this tool's) | — | apply-edit (explicitly out of scope by its own tool description) |
| Who calls X / what does X call | Proxy tool, trace operation | full canonical project slug | — | direct trace wrapper (unverified reliability) |
| Find symbol by pattern | Proxy tool, search operation | project required | — | direct search wrapper (confirmed count-only, no data) |
| Architecture / hotspots / fan-in-out | Proxy tool, architecture operation | project | — | — |
| What changed (PR / branch / range / working tree) | Diff-commits | one ref required; omit the second for uncommitted changes | provide-code-context on flagged files | reading files one-by-one, shelling out to a raw diff |
| Reindex after an edit lands | Proxy tool, undocumented reindex value | repo path, fast mode | re-run the graph query | trusting graph results against a just-edited repo |
| Resolve the correct project slug | Proxy tool, undocumented list-projects value | — | use the returned slug in later calls | guessing the short repo name |
| Token-savings self-check | Stats dashboard tool | — | — | treating a nonzero file-tracked count as proof graph calls were compressed too — it only covers file-context |

---

## 4 — Workflows

**A · Bug investigation.** Provide-code-context (debug intent) on the suspect file → if the bug crosses files, the proxy tool's trace operation in both directions → provide-code-context on whatever that surfaces → confirm root cause against byte-exact bodies (focus-methods) before proposing a fix — never off a compressed skeleton alone.

**B · Targeted method change.** Provide-code-context at edit fidelity with focus-methods naming the exact target → apply-edit with the byte-exact body just retrieved → fall back to the generic edit tool only for a signature change, a multi-method span, or a file new this session.

**C · Architectural / cross-file change.** Proxy tool's architecture operation and/or search operation with degree filters to scope blast radius **before** touching anything → per-file provide-code-context for each affected file → after editing, the undocumented reindex value **before** re-querying the graph — the index does not auto-refresh on write.

**D · Per-language differences.** No schema or rule differentiates workflow by language. The edit tool and the context tool between them cover all four languages present in Meridian (a JVM language, a systems language, a typed web-scripting language, and a CLR language) as first-class. The one real wrinkle: the stats dashboard carries a framework-specific flag for one of Meridian's sibling repos (the Angular frontend), but nothing addresses that framework's template or stylesheet files specifically — a genuine gap, not something to infer.

**E · Verification after editing.** Force the undocumented reindex value before trusting graph results on a just-edited repo. The file-tracked-count self-check proves file-context compression happened — it says nothing about whether graph queries were compressed, or whether a delegated sub-task bypassed everything. There is currently no self-check that covers the proxy tool's usage at all.

**F · Multi-file work.** Diff-commits once for the whole set, triage, then per-file provide-code-context only where real inspection is needed. Reach for the proxy tool's search operation over a literal text search when the question is semantic ("what else references this symbol") rather than literal — but only inside indexed repos.

---

## 5 — Rules gap analysis (Meridian's own consumer docs)

| Governs | Current state | Gap |
|---|---|---|
| Global, trigger-loaded skill doc | Documents three numbered rules, a decision matrix, a full "direct graph-service" tool list | Lists 10+ tools that are retired or never real (§2.3). **Missing the apply-edit tool entirely** — a reader of only this document wouldn't know it exists. Never mentions the three direct graph wrapper tools, so nothing warns against them either |
| Path-scoped project instructions doc | More current than the skill doc — has the apply-edit rule, a focus-methods token benchmark, a retry protocol for context-tool failures | Same phantom-tool table, copy-pasted from the skill doc. No mention of the two undocumented pass-through values, the project-slug requirement, or the three unreliable graph wrapper tools |
| Pre-tool-use / post-tool-use / subagent-start / session-start hooks | **Most accurate layer.** Already knows the graph service isn't a separate server; already knows the undocumented reindex workaround; gates raw file-read *and* raw text-search on single code files, with a one-time retry allowance for the host's required pre-edit read | That retry nuance, and the fact raw text-search is gated at all, exist only in hook source comments — neither rule document tells the agent, so it's learned only by triggering a denial |
| Client-side project memory | Describes a multi-root indexing config key | That key is absent from the current workspace config. Cross-repo indexing still works live (confirmed against the Angular frontend sibling) — but the documented mechanism for how it got that way is stale |
| Multi-repo coverage | The path-scoped instructions doc is scoped to the primary repository only | None of Meridian's four sibling repositories have their own copy of that instructions doc — they rely entirely on the global skill doc (trigger-gated, unreliable) and the global hooks (always-on, reliable). An architectural gap, not a wording one |

---

## 6 — Proposed rule changes (for Meridian's own docs — described, not applied)

1. **Delete the retired compression tool** from both of Meridian's tool tables — genuinely retired here, this repo's own README already reflects it.
2. **Delete the seven never-real "direct" entries** — they describe the graph service's own standalone capability list, not this integration's.
3. **Reclassify the change-detection tool** as internal-only and remove it from any "direct call" table — Meridian's agent cannot reach it under any name.
4. **Reclassify the reindex tool** from "internal-only" to "undocumented proxy pass-through for forcing a reindex" — and document when to call it (Workflow E).
5. **Add the project-listing pass-through** to that same category, as the actual fix for the proxy tool's circular error message.
6. **Document the three direct graph wrapper tools explicitly** as low-value/unverified, rather than leaving them as silent traps a plausible-sounding tool name could lead an agent into.
7. **Sync Meridian's two rule documents** — add the apply-edit rule to the global skill doc, or better, make the path-scoped instructions doc the single source of truth and have the skill doc point to it.
8. **Write down the text-search gate and the read-tool one-time-retry mechanic** in both documents — currently discoverable only by being denied.
9. **Decide sibling-repo rule coverage explicitly** — either mirror the instructions doc into each sibling repo, or state in writing that the global hooks are the intended enforcement layer there.
10. **Extend the compliance self-check** — the stats dashboard covers file-context only; either add a counter for proxy-tool usage or note the blind spot explicitly wherever the check is invoked.

---

## 7 — Open questions — flagged, not guessed at

- **Lower-level compression primitive vs. the higher-level context tool** — no rule says when a caller should ever reach for the primitive directly.
- **The two single-file diff/delta tools** — both single-file/in-session; the actual distinction (AST diff vs. IR delta, and which use case favors which) is never spelled out.
- **Direct trace/query wrapper reliability** — spot checks didn't exercise a confirmed-linked symbol pair, so a zero-edge result proves nothing either way. The clean test: run a known-good caller→callee pair (a controller method calling directly into its backing service interface method, confirmed linked in an earlier retest) through the direct wrappers and compare against the proxy tool's count on the same pair.
- **Multi-root config drift** — the workspace config has no multi-root entry, yet a sibling repo is demonstrably indexed today. Don't assume this survives a fresh machine, a service restart, or a config edit — the mechanism keeping it alive is currently unknown.
- **The second, proxy-adjacent binary** present on disk alongside the main one — role unconfirmed, not referenced in any rule file or hook.

---

## 8 — Evidence log

| Claim | Evidence |
|---|---|
| Only one MCP server is registered in the Meridian session | Live server-list output |
| The graph service is internally spawned, not its own server | Workspace config: binary path + auto-launch flag |
| The direct search wrapper returns count-only | Live call vs. the proxy tool's search operation on identical query/project |
| The proxy tool requires the full project slug | Live call failed unqualified, succeeded with the full canonical slug |
| A sibling repo is indexed despite a missing config key | Live architecture-operation call returned several thousand nodes/edges |
| The compression tool was retired deliberately | `a2d6d54`, `cc08e13`, `c6015ba`, `67d26a2` |
| The "SCHEMA v2" fix is a notation correction, not a tool-exposure change | `b24e6e3` commit message + diff |
| 7 tool names never appear in this repo's source | `git log -S"<name>" -- '*.rs'`, zero hits each, across full history |
| The change-detection tool is bridge-internal only | `f26dddb` diff: internal bridge method, not an exposed tool |
| The reindex tool is reachable via the proxy tool | Post-edit hook source comment, cross-checked against `47bff7e` |
| The project-listing tool is reachable via the proxy tool | `src/tests/cbm/e2e.rs:924` |
| Small-file compression can cost more tokens than a raw read | Live dashboard data: one small model class, 373 raw tokens → 810 compressed tokens at medium fidelity (a −117% outcome) — generalizes the existing token-savings warning beyond the specific case it currently documents |

---

*No files in the Meridian workspace were modified in the course of this audit. Commit hashes above belong to this repository's own history and are safe to keep as-is; all other identifiers have been generalized per this repo's standing anonymization convention for client-derived audits.*
