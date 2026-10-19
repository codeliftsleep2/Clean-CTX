# C# Semantic Resolution R8 SignalR Migration

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Phase:** Expanded implementation complete; operator-run verification GREEN

## Result

R8 migrates SignalR `Hub` and `Hub<TClient>` semantic authority from textual
base-name predicates to the shared C# resolution boundary. The neutral known
external universe now contains `Microsoft.AspNetCore.SignalR.Hub` at arities
zero and one. Framework semantics remain owned by the .NET meta-layer.

Resolved direct or indirect ancestry authorizes `HubMethodTargets`. Method
scanning remains payload extraction and cannot create Hub identity. Explicit,
`global::`, ordinary-import, namespace-alias, type-alias, partial, generic,
and admitted-source ancestry forms converge. Local shadows, competing imports,
wrong arity, and every other non-success disposition fail closed.

Canonical C# execution semantics now use the same proof. SignalR-derived
`ExecutionContext::Realtime` is projected from resolved Hub ancestry; the
language layer no longer interprets written `Hub` spelling. Generation
reconciliation removes and rebuilds only the realtime contribution, preserving
independently established contexts such as `Async` and `TransactionScope`.

## Production changes

- Added neutral known-external identities for SignalR Hub arities zero and one.
- Added resolved Hub ancestry classification and owner-attributed projection.
- Preserved the existing fidelity boundary: low fidelity emits no method-target
  relationships.
- Removed `contains(": Hub")` and `contains(": Hub<")` from semantic-edge
  authority.
- Kept the existing SignalR marker extractor as render-only behavior.
- Added structured applicability so textual detection cannot suppress resolved
  Hub authority.
- Added generation reconciliation that retires compilation-local Hub edges and
  reprojects current cross-file resolution.
- Removed written-Hub classification from the canonical C# language layer.
- Carried parser-owned method identities through neutral C# compilation units
  so resolved class symbols can authorize their owned methods.
- Added compilation-local and generation-wide realtime-context projection.
- Published reconciled canonical IR into the session owner alongside resolved
  semantic edges.
- Retracted session-owned canonical IR during workspace refresh so deleted
  owners cannot leave stale execution-context facts behind.
- Bumped C# canonical and semantic-input producer generations because identical
  source now uses a different authority proof.

## Regression coverage

Tracked R8 tests cover Hub arities zero and one, all supported spelling forms,
indirect inheritance, partial declarations, a zero-method Hub, local shadowing,
competing imports, wrong arity, cross-file reconciliation, upstream base
removal/restoration, and owner deletion.

Every identity case asserts both `HubMethodTargets` and canonical per-method
`ExecutionContext::Realtime`. The lifecycle regression uses an async Hub
method and proves that ancestry removal retracts realtime while preserving
`ExecutionContext::Async`.

## Static retirement audit

The production audit covers written `Hub`, `Hub<`, SignalR classification,
`HubMethodTargets`, and realtime execution-context production:

- `src/dotnet_meta/semantic.rs` produces authoritative SignalR facts only from
  resolved Hub ancestry.
- `src/ir/layers/csharp.rs` no longer recognizes Hub spelling or produces
  SignalR realtime contexts.
- `src/mcp/semantic_compilation/artifact.rs` reprojects only the SignalR
  realtime contribution during generation reconciliation.
- `src/dotnet_meta/signalr.rs` retains textual Hub checks solely for Φ marker
  rendering; it returns no semantic edge or canonical instruction.
- `src/dotnet_meta/detect.rs` retains spelling solely for candidate scheduling;
  structured resolved roles bypass detector misses.

No other production Hub-spelling authority or realtime producer was found.

## Scope held

- R0-R7 and Issue #148 are unchanged.
- ASP.NET R9-R11 were not started.
- SignalR markers were not promoted to semantic authority.
- No provider authority or public response contract changed.

## Verification status

The operator reported the complete R8 gate GREEN, including focused SignalR
resolution/lifecycle tests, canonical C# execution-context regressions,
SignalR marker compatibility, R6-R7 and Issue #148 preservation checks,
formatting, zero-warning Clippy, the complete all-feature workspace test suite,
file-size enforcement, and UTF-8/encoding verification.

Agent MaxHeadRoom did not run Cargo, tests, builds, Clippy, binaries, or MCP
commands. The reported GREEN result is operator-run verification.
