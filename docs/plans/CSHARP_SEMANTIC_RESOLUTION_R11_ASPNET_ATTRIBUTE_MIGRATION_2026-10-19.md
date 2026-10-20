# C# Semantic Resolution R11 ASP.NET Attribute Migration

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Phase:** Implementation complete; focused operator-run verification GREEN; preservation gate pending

## Result

R11 migrates ASP.NET Core controller, route, and HTTP-method attribute
authority from textual spelling to the shared C# attribute-resolution
boundary established by R10. Framework meaning remains in the .NET
meta-layer; C# resolution remains framework-neutral.

`ApiControllerAttribute` and `ControllerAttribute` can establish controller
identity. `RouteAttribute` can publish `HasRoute` only for an already
authorized controller. Supported HTTP method attributes can publish
`ControllerAction` only for an already authorized controller and the exact
method that owns the resolved attribute.

## Production changes

- Added neutral known-external identities for ASP.NET controller, route, and
  supported HTTP-method attributes.
- Added parser-owned method-attribute captures to the existing single C# parse.
- Associated method attributes with their direct method and composed class
  symbol without admitting nested-class attributes.
- Preserved written attribute payload separately from resolved identity so
  route templates can be extracted only after identity authorization.
- Unified base-derived and attribute-derived controller classification.
- Projected one controller role per composed symbol and owner-attributed
  route/action facts from resolved attributes.
- Preserved the Low-fidelity boundary for controller actions.
- Removed the temporary textual direct-controller authority retained after R9.
- Removed textual ASP.NET semantic-edge construction from marker analysis.
- Bumped C# semantic-input and .NET semantic producer generations.

## Tracked regressions

R11 tests cover imported, explicit suffix, fully qualified, `global::`,
namespace-alias, type-alias, and partial attribute forms. They cover direct
controller role, class route, method action, source shadows, competing
imports, alias retargeting/restoration, partial composition, independent
payload retraction, role retraction, and owner deletion.

The focused `r11_` identity and lifecycle regressions are operator-reported
GREEN.

## Static retirement audit

- `src/dotnet_meta/semantic.rs` is the sole ASP.NET controller semantic
  projector and compares resolved neutral identities.
- `src/dotnet_meta/aspnet.rs` retains textual checks only for Φ marker
  rendering and constructs no semantic edges.
- `src/dotnet_meta/detect.rs` remains candidate scheduling only.
- `src/ir/csharp_resolution/compilation.rs` and its method-input helper own
  neutral class/method attribute association and resolution.
- Generation reconciliation removes and rebuilds controller roles, routes,
  and actions from the current admitted-source universe.
- No textual ASP.NET base or attribute predicate remains capable of publishing
  controller identity, `HasRoute`, or `ControllerAction`.

## Scope held

- R0-R10 and Issue #148 behavior remain unchanged.
- Route-template and action-name payload extraction remains bounded behind
  resolved attribute identity.
- Authorization, model binding, serialization, validation, and marker-only
  attributes remain outside semantic authority unless a later audit proves
  otherwise.
- No provider, completeness, or public response schema changed.

## Verification status

Agent MaxHeadRoom ran no Cargo command, test, build, Clippy, binary, MCP
runtime, or server. Focused GREEN evidence is operator-run. R11 remains pending
the consolidated preservation and repository verification gate.
