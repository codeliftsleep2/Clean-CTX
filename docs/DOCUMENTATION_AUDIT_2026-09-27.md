# Documentation architecture audit — 2026-09-27

**Status:** Complete
**Scope:** Product-level documentation after the 0.8.0-rc architectural
hardening, SCHEMA-vNext rollout, and query-boundary work

## Why this audit exists

The repository accumulated several kinds of documents without a stable
taxonomy: living architecture, developer instructions, shipped design records,
future proposals, audit evidence, and benchmark snapshots. As implementation
moved through canonical IR, SCHEMA-vNext, `WorkspaceIndex`, explicit delta
transport, semantic persistence, and stricter MCP boundaries, some older prose
remained technically useful but was still positioned as current authority.

The new `docs/README.md` is the navigation boundary. It does not replace any
subsystem reference; it tells readers which kind of claim each document owns.

## Reconciled in this pass

- Added a repository-wide documentation map separating current authority,
  operational guides, framework references, and historical/proposal records.
- Updated Architecture Overview terminology to SCHEMA-vNext and all four
  feature-gated language layers.
- Replaced its compression-centered top-level and obsolete dispatcher diagrams
  with the canonical-IR fan-out and actual interior-mutability boundaries.
- Repositioned the June 2026 performance and 50-edit tables as historical
  baselines; current repeatable measurements remain in the context-compression
  verification package.
- Updated Developer Documentation to the 0.8.0-rc architecture, all-feature
  local development convention, current language/framework scope, canonical IR
  role, `dv:2`, and current module ownership.
- Reconciled its persistence section with binary `0x04`, schema v5,
  semantic-edge snapshots, auto-save policy, edit intents, transactional
  mutation, deletion, restore, and replay.
- Removed duplicated volatile test counts and redirected verification authority
  to `docs/agent/verification.md`.
- Corrected primary changelog links to `docs/changelogs/CHANGELOG.md`.
- Repositioned the Roadmap as a directional/historical ledger rather than
  production authority.
- Closed stale “still requires two calls” and pending-live language in the
  workspace-query proposal, leaving only the optional focus-methods field run.

## Completed scope from the original work list

### Living references

- Compared every top-level `CleanCtxConfig` field and the nested config owners
  with `CONFIGURATION.md`. The guide now enumerates accepted top-level keys,
  identifies nested source authority, marks `auto_delta` inactive, and states
  that no public dispatcher configuration block exists.
- Compared all registered names in `src/mcp/tools.rs` and `src/cbm/tools.rs`
  with the tooling inventory. Every current public tool is represented;
  retired tools are not presented as callable.

### Subsystem references

- Reconciled Angular, Angular ecosystem, and .NET documents with typed semantic
  edge production and `WorkspaceIndex` ownership.
- Added a Spring living reference grounded in `src/spring_meta/` and its current
  semantic relations.
- Corrected CBM project resolution and call-scoped `cbm_proxy` behavior in the
  operator guidance.
- Reworked Security around the actual core-MCP, CBM, HTTP-proxy, persistence,
  edit, trusted-root, hydration, and narrowing-only `withinPath` boundaries.

### Historical records and proposals

- Added explicit status/authority headers to every record under
  `docs/plans/**` and `docs/architecture/**`, plus the root-level design and
  audit records.
- Marked the IR lockdown/audit continuation series as historical certification
  evidence and pointed it to `IR_ARCHITECTURE_CERTIFICATION.md`.
- Collapsed the Roadmap's completed-item ledger and made the versioned changelog
  canonical for release history.
- Preserved SCHEMA-v5 and SCHEMA-v2 wording where it truthfully names a
  historical renderer or measured source state.

## Source-level documentation drift discovered

The stale `src/mcp/context_store.rs` “future SQLite” comments were corrected in
this pass without changing behavior.

## Validation boundary

Bounded documentation validation compares registered tool names, checks plan
and architecture status headers, resolves relative Markdown links in changed
documents, and runs `git diff --check`. No Cargo build, test, check, Clippy, or
formatting command is part of this documentation-only audit; repository-wide
verification remains CI-owned.
