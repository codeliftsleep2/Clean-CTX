# Clean-CTX documentation map

This index separates current architectural authority from operational guides,
historical evidence, and proposals. A document's age does not determine its
authority; its category and status do.

## Current architecture and contracts

- [`ARCHITECTURE_OVERVIEW.md`](ARCHITECTURE_OVERVIEW.md) — current system and
  module boundaries.
- [`ARCHITECTURAL_INVARIANTS.md`](ARCHITECTURAL_INVARIANTS.md) — authoritative
  durable architectural contracts and their enforcement.
- [`COMPILER_IR.md`](COMPILER_IR.md) — canonical IR, persistence, delta, and
  SCHEMA-vNext presentation contracts.
- [`CONFIGURATION.md`](CONFIGURATION.md) — configuration schema and precedence.
- [`SECURITY.md`](SECURITY.md) — trust and security boundaries.

## Developer and operator guides

- [`DEVELOPER_DOCUMENTATION.md`](DEVELOPER_DOCUMENTATION.md) — extension and
  development entry point; defer to `agent/verification.md` for the exact gate.
- [`agent/tooling.md`](agent/tooling.md) — MCP and code-intelligence tool use.
- [`CLAUDE_INTEGRATION_RULES.md`](CLAUDE_INTEGRATION_RULES.md) — compact
  Claude-facing projection for using Clean-CTX with or without CBM.
- [`agent/verification.md`](agent/verification.md) — single authoritative final
  verification command list.
- [`TROUBLESHOOTING.md`](TROUBLESHOOTING.md), [`PROXY.md`](PROXY.md), and
  [`PERFORMANCE.md`](PERFORMANCE.md) — focused operational references.

## Framework references

- [`ANGULAR_META_LAYER.md`](ANGULAR_META_LAYER.md)
- [`ANGULAR_ECOSYSTEM_DEEPENING.md`](ANGULAR_ECOSYSTEM_DEEPENING.md)
- [`DOTNET_META_LAYER.md`](DOTNET_META_LAYER.md)
- [`SPRING_META_LAYER.md`](SPRING_META_LAYER.md)

## Proposals, audits, and historical evidence

Documents under [`architecture/`](architecture/) and [`plans/`](plans/) record
decisions, investigations, migrations, and measurements. They are not current
production authority unless their header explicitly says so. Completed
proposals should point to the resulting invariant and production owners;
superseded measurements remain historical evidence rather than being silently
rewritten as current benchmarks.

The active phased meta-layer performance migration is documented in
[`architecture/META_LAYER_SINGLE_PASS_MIGRATION_PLAN_2026-09-28.md`](architecture/META_LAYER_SINGLE_PASS_MIGRATION_PLAN_2026-09-28.md).

The chronological release ledger is
[`changelogs/CHANGELOG.md`](changelogs/CHANGELOG.md). The roadmap is directional;
implemented behavior is determined by production code and the invariant catalog.

The completed 2026-09-27 reconciliation record is
[`DOCUMENTATION_AUDIT_2026-09-27.md`](DOCUMENTATION_AUDIT_2026-09-27.md).
