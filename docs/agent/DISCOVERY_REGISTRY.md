# Live Discovery Registry (`docs/agent/DISCOVERY_REGISTRY.md`)

**Purpose:** Record discoveries made in the **real-world discovery environment**
(Claude + Clean-CTX operating against large real repositories) and their
resolution, so field findings become institutional engineering knowledge rather
than disappearing into a development conversation.

This is NOT a changelog (see `docs/CHANGELOG.md`) and NOT release accounting
(see `docs/agent/releases.md`). It exists to close the two-environment gap:

```text
REAL-WORLD DISCOVERY (Claude + real repo)
      ↓
root-cause investigation
      ↓
classify (Protocol | Semantic | Emergent)
      ↓
can it be distilled deterministically?  → local regression
      ↓                                        (cheap Rust test)
not reproducible at small scale
      ↓
retain as a live acceptance scenario → documented record here
```

## Format

One entry per significant discovery, newest first. An entry is closed when a
local regression covers it, a live scenario has been re-verified, or the
behavior is superseded.

## Entry Template

| Field | Value |
|-------|-------|
| **Discovered** | YYYY-MM-DD |
| **Environment** | Claude + Clean-CTX vX.Y.Z |
| **Repository/context** | Description (approximate size, languages, frameworks) |
| **Symptom** | What was observed |
| **Root cause** | Precise technical cause |
| **Classification** | Protocol / Semantic / Emergent |
| **Reproducible locally?** | Yes / No |
| **Local regression** | Test path(s) or N/A |
| **Live scenario required?** | Yes / No |
| **Architectural invariant** | INV-XXX or N/A |
| **Status** | Open / Fixed / Verified |

---

## DIS-2026-001: workspace_query MCP Response Envelope Bypass

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-02 |
| **Environment** | Claude + Clean-CTX (post-0.5.0 structured-output migration) |
| **Repository/context** | Large heterogeneous real workspace |
| **Symptom** | Every `workspace_query` response carried bare domain fields (`entities`, `edges`, `count`, ...) directly under JSON-RPC `result` with NO MCP `content` channel — a schema-validating MCP client had nothing renderable to display. |
| **Root cause** | All six `workspace_query` sub-handlers built bare `result` objects, bypassing the canonical MCP `CallToolResult` envelope (`content` + optional `structuredContent`/`_meta`) established by the 0.5.0 migration. The handler was introduced after the migration (commit `2d18377`, 2026-09-01) with no contract audit gate for new tools; its tests validated the ad-hoc result shape rather than the envelope. |
| **Classification** | Protocol |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query.rs` (all operations validated via `assert_valid_mcp_envelope` + `structuredContent`); `src/tests/mcp/phase3_contract.rs` (shared envelope helpers); `src/tests/mcp/envelope_contract.rs` (complete remaining produced-tool coverage) |
| **Live scenario required?** | No |
| **Architectural invariant** | MCP-001 (docs/ARCHITECTURAL_INVARIANTS.md) |
| **Status** | Fixed |

**Distillation note:** the root cause was NOT workspace complexity. It was an
MCP response-contract violation, reproducible with the existing sample corpus —
no large fixture was required. This is the canonical example of a **Protocol**
class discovery: field evidence surfaced it; a cheap deterministic Rust
regression now protects it permanently.