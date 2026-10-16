# Clean-CTX documentation index

> **Status: CURRENT DOCUMENTATION INDEX**
>
> This page classifies authority. A dated plan, investigation, benchmark, or
> acceptance report never overrides a current-authority document merely because
> it is longer or more detailed.

## Normative documentation spine

Read these in order:

1. [Public README](../README.md) — current product and tool surface.
2. [Architectural invariants](ARCHITECTURAL_INVARIANTS.md) — enforced durable rules.
3. [Semantic authority architecture](architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md)
   — identity, acquisition, refinement, completeness, dependencies, optional
   providers, and projection.
4. [Agent tooling](agent/tooling.md) — current code-context workflow.
5. Framework references — domain-specific producers and projections.

If current documents appear to conflict, production code plus the invariant
catalog governs; reconcile the prose rather than selecting the stronger claim.

## Current authority

| Document | Role |
| --- | --- |
| [ARCHITECTURAL_INVARIANTS.md](ARCHITECTURAL_INVARIANTS.md) | Normative invariant catalog |
| [SEMANTIC_AUTHORITY_ARCHITECTURE.md](architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md) | Normative semantic-authority composition |
| [agent/tooling.md](agent/tooling.md) | Normative MCP/code-context workflow |
| [agent/verification.md](agent/verification.md) | Normative final verification gate |
| [ENCODING_POLICY.md](ENCODING_POLICY.md) | Encoding contract |
| [CONFIGURATION.md](CONFIGURATION.md) | Configuration reference |
| [COMPILER_IR.md](COMPILER_IR.md) | Canonical IR and notation contract |
| [BINARY_V04_CONTRACT.md](architecture/BINARY_V04_CONTRACT.md) | Binary format contract |
| [CORE_OP_CONTRACT_MATRIX.md](architecture/CORE_OP_CONTRACT_MATRIX.md) | Core opcode matrix |

## Current user and operator guides

[Troubleshooting](TROUBLESHOOTING.md), [Claude integration](CLAUDE_INTEGRATION_RULES.md),
[diff guide](DIFF_COMMITS_GUIDE.md), [edit types](EDIT_TYPE.md),
[proxy](PROXY.md), [security](SECURITY.md), [developer guide](DEVELOPER_DOCUMENTATION.md),
and the guides under docs/agent are current for their named surface. They do
not independently grant semantic authority or completeness.

## Framework architecture

| Document | Classification |
| --- | --- |
| [ANGULAR_META_LAYER.md](ANGULAR_META_LAYER.md) | Current reference with historical phases |
| [ANGULAR_ECOSYSTEM_DEEPENING.md](ANGULAR_ECOSYSTEM_DEEPENING.md) | Current marker/reference material with history |
| [DOTNET_META_LAYER.md](DOTNET_META_LAYER.md) | Current marker reference with historical phases |
| [SPRING_META_LAYER.md](SPRING_META_LAYER.md) | Current framework reference |

## Historical and evidence collections

| Path | Default classification | Treatment |
| --- | --- | --- |
| docs/plans/** | Implemented/superseded plan, investigation, or proposal | Preserve chronology; not normative |
| docs/architecture/*PLAN*, *AUDIT*, *INVESTIGATION*, *PROPOSAL*, *RECONNAISSANCE* | Migration history, evidence, or future proposal | Not normative unless listed above |
| docs/changelogs/** | Historical release records | Preserve dated claims |
| verification/** | Reusable operator definitions/evidence docs | Never substitutes for tracked tests |
| docs/PERFORMANCE.md | Superseded benchmark baseline | Dated evidence, not a universal guarantee |
| docs/agent/DISCOVERY_REGISTRY.md | Append-only discovery ledger | Evidence ledger |
| handoff documents | Point-in-time operational record | Historical after the session |

Plans may contain approved decisions and useful rationale. Once implemented,
their role is evidence/history; current contracts are distilled into the spine.

## Evidence classes

- Static architecture audit: inspection evidence.
- Tracked regression under src/tests/**: executable CI contract.
- Operator/live harness under verification/**: production-boundary evidence.
- Final gate/CI: release verification at a named revision.

One class does not silently stand in for another. Evergreen public docs avoid
undated “all tests pass” or benchmark guarantees.
