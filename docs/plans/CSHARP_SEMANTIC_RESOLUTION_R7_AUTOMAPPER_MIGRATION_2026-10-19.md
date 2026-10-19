# C# Semantic Resolution R7 AutoMapper Migration

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Phase:** Implementation complete; operator-run verification pending

## Result

R7 migrates AutoMapper `Profile` semantic authority from the legacy
`contains(": Profile")` predicate to the shared C# resolution boundary built
in R0-R5. `DbContext` remains unchanged, and SignalR and ASP.NET are not part
of this slice.

The C# known-external identity set now includes `AutoMapper.Profile`. A class
can authorize `MapperProfile` mapping relationships only when its resolved
ancestry reaches that arity-zero external identity. Explicit, `global::`,
ordinary-import, namespace-alias, type-alias, admitted-source ancestry, and
partial-class forms therefore share one authority path. Ambiguous, unresolved,
unsupported, and locally shadowed references fail closed.

## Production changes

- Added the neutral known-external identity `AutoMapper.Profile`.
- Added resolved AutoMapper ancestry classification without introducing a
  framework role into the neutral resolver.
- Projected `MapsFrom` and `MapsTo` only after resolved role authorization.
- Preserved `CreateMap<TSource, TDestination>` as payload extraction rather
  than identity authority.
- Removed the legacy Profile substring predicate from semantic-edge
  production.
- Kept the existing AutoMapper marker extractor as render-only behavior.
- Added structured AutoMapper applicability so textual detection cannot veto
  already-resolved authority.
- Added generation reconciliation that removes compilation-local mapping
  edges and reprojects them from the current shared cross-file resolution.
- Preserved physical owner-file attribution on every projected edge.

## Regression coverage

The framework-neutral integration regression covers:

- ordinary import;
- fully qualified and `global::` qualification;
- namespace and type aliases;
- indirect inheritance;
- partial declarations;
- a resolved profile with no mapping payload;
- a local `Profile` shadow;
- competing source and known-external imports.

The MCP production-path regression covers cross-file ancestry, mapping
publication, upstream base removal, restoration, and owner deletion. It uses
the real workspace hydration, reconciliation, publication, refresh, and index
paths.

## Authority boundary after R7

```text
parser-owned C# facts
    -> shared neutral identity/disposition
    -> resolved ancestry proof to AutoMapper.Profile
    -> AutoMapper-owned role authorization
    -> CreateMap payload extraction
    -> durable MapsFrom / MapsTo publication
```

No failed resolution disposition falls back to the written name `Profile`.

## Verification status

No Cargo, test, build, Clippy, binary, or MCP command was run by Agent
MaxHeadRoom. The implementation and tracked regressions are ready for the
operator-run R7 gate. R7 is not declared GREEN until those results are
returned.

## Scope held

- R0-R6 architecture is preserved.
- Issue #148 acquisition authority is unchanged.
- SignalR R8 was not started.
- ASP.NET R9-R11 were not started.
- No provider semantic authority or new public schema was introduced.
