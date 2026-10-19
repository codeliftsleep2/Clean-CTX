# C# Semantic Resolution R10 Attribute Foundation and MSTest Migration

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Phase:** Implementation complete; operator-run deterministic and live MCP verification GREEN

## Result

R10 establishes the neutral C# attribute-reference resolution boundary and
migrates MSTest `TestClassAttribute` semantic authority onto it. Attribute
resolution reuses the admitted-source symbol index, lexical imports, namespace
and type aliases, neutral type identities, explicit dispositions, and
dependency-owned generation reconciliation established by R0-R9.

C# attribute suffix elision is syntax-specific: `TestClass` may resolve to
`TestClassAttribute`, while an explicitly written `TestClassAttribute` is not
expanded again. Framework meaning remains in the .NET meta-layer.

## Production changes

- Added parser-owned class-attribute captures alongside the existing neutral
  all-attribute facts.
- Associated direct class attributes with composed C# source symbols while
  excluding method/property and nested-class attributes.
- Added a dedicated neutral attribute resolver with explicit `Resolved`,
  `Ambiguous`, `Unsupported`, and `Unresolved` outcomes.
- Preserved lexical ordinary-import, namespace-alias, type-alias, admitted
  source shadow, competing-candidate, and evidence semantics.
- Added the neutral known-external identity
  `Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute`.
- Published one durable `dotnet/TestClass` role per composed source symbol
  only after resolved MSTest identity.
- Reconciled TestClass roles across partial composition, alias retargeting,
  source edits, restoration, and deletion.
- Preserved the configured testing opt-out through retained artifacts and
  generation reconciliation.
- Kept request-local `Tests` projection declaration-proven; candidate
  extraction no longer interprets attribute spelling as authority.
- Bumped C# semantic-input and .NET semantic producer generations.

## Tracked regression coverage

R10 tests cover explicit/global qualification, suffix and explicit-suffix
forms, ordinary imports, namespace aliases, type aliases, neutral identity and
dispositions, local shadows, competing imports, partial composition,
cross-file durable ownership, request-local `Tests` parity, alias retargeting,
attribute restoration, owner deletion, testing configuration, and producer
compatibility.

The operator reported the focused `r10_` suite, directly affected preservation
gates, and complete deterministic repository gate GREEN.

## Live MCP acceptance

The tracked production-stdio harness verified the registered tool boundary,
the complete positive/negative spelling matrix, fresh and warm history
independence, alias and partial-part lifecycle, testing configuration,
cross-root subject resolution, and a ready-provider plausible-but-incomplete
snapshot. Semantic identity and relationship assertions were kept separate
from discovery and coverage diagnostics.

The operator reported all cases R10-A through R10-G PASS. Evidence was retained
under:

`target/csharp-r10-live-verification/20261019-184152-052b6ccb`

## Static retirement audit

- `src/dotnet_meta/semantic.rs` is the only MSTest identity consumer and
  compares resolved neutral identity.
- `src/dotnet_meta/testing.rs` retains textual TestClass recognition only for
  marker rendering. Its subject candidates cannot publish a role and are
  joined against durable declarations before `Tests` projection.
- `src/dotnet_meta/detect.rs` remains candidate scheduling only; resolved
  MSTest identity positively admits .NET evaluation.
- `src/mcp/semantic_compilation/artifact.rs` removes and rebuilds TestClass
  roles from the current resolved generation.
- No textual TestClass predicate remains capable of publishing durable
  TestClass identity or a `Tests` relationship.

## Scope held

- R0-R9 and Issue #148 semantics remain unchanged.
- ASP.NET class, route, and HTTP-method attributes remain the explicit R11
  migration boundary.
- `TestMethod`, lifecycle, Moq, and other testing markers remain rendering or
  payload behavior and do not become framework identity authority.
- No provider, completeness, or public response schema changed.

## Verification status

Agent MaxHeadRoom ran no Cargo command, test, build, Clippy, binary, MCP
runtime, or server. The operator reported the complete deterministic repository
gate GREEN and the tracked R10 production-stdio live acceptance PASS. R10 is
GREEN and R11 ASP.NET attribute migration is unblocked.
