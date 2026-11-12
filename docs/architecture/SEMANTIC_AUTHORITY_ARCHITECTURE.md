# Semantic Authority Architecture

> **Status: CURRENT AUTHORITY**
>
> Audited at d039453bcc5825f47917e1956669bd3446a06cc5 on
> feature/post-cbm-native-correctness-fixes (2026-10-16). Production behavior
> and the focused invariant owners indexed by docs/ARCHITECTURAL_INVARIANTS.md
> remain the ultimate authority.

## Governing rule

**The refinement layer may join authorities. It must not manufacture them.**

Code existence, production reachability, semantic authority, declaration
authority, completeness authority, and model-facing projection are separate.
Name is not identity; proximity is not ownership; spelling is not binding; a
reference is not declaration existence; shared storage is not shared semantic
ownership.

An authoritative producer may establish one side of a join and a resolver
another. Refinement may publish the joined fact only when both are current and
compatible.

## Production model

    Source and declared scope
      -> canonical IR and source-bound evidence
      -> domain evidence producers
      -> declaration/reference resolution
      -> compatible source generation and dependency validation
      -> domain refinement
      -> workspace occurrences plus canonical semantics
      -> completeness authority
      -> fidelity projection
      -> LLM context

Optional providers may narrow acquisition work. Acquisition strength is not
publication authority.

## WorkspaceIndex Model C identity

WorkspaceIndex groups semantic occurrences by the tuple
(domain, entity_type, name). The tuple is not a universally unique source
declaration key. Multiple declaration occurrences may legitimately share it;
provenance is retained separately.

Keep three concepts distinct:

1. semantic tuple identity;
2. declaration occurrence and source provenance;
3. exact declaration or binding identity.

Consumers requiring exact declaration, member, overload, or method authority
must prove it from current language-specific evidence. WorkspaceIndex must not
be re-keyed merely to make exact binding look simpler.

## Acquisition, history, and batches

**Warm state may improve performance. It must never improve truth.**

For an authoritative domain:

    query/domain
      -> bounded evidence requirements
      -> domain-blind acquisition union
      -> source acquisition and validation
      -> compatible authoritative generation
      -> FREEZE
      -> domain refinement
      -> semantic result
      -> completeness certificate

Equivalent requests over equivalent authoritative source and scope must not
become more correct because unrelated requests ran first. A batch unions
requirements before final interpretation, then interprets authoritative
index-backed items against the same post-acquisition generation. Batching
changes economics, not truth.

Generic graph/traversal and explicitly indexed-evidence-only families retain
honest lower-bound semantics where domain closure is not proven.

## Completeness authority

Completeness requires declared scope, satisfied evidence requirements, a
compatible source generation, producer support, and valid semantic
dependencies. Cache/index population, stable zero results, or provider success
cannot substitute for this proof.

The internal CompletenessCertificate records bounded scope, generation, and
requirements. Generic graph responses remain source_complete:false and
result_semantics:lower_bound when exhaustiveness cannot be established.
Failure to earn a certificate does not make returned facts wrong; it means
Clean-CTX cannot claim they are exhaustive.

Request-local `AcquisitionOutcome` composes mathematical satisfaction, one
typed disposition per required coverage unit, and processing state. It is not
durable authority and is not negative-cached. The completion cache remains
restricted to complete `AcquisitionSatisfaction`; a compatible cache hit may
reconstruct only the terminal complete request-local outcome. Coverage derives
a bounded additive `coverage_detail` from this state. That projection explains
complete, structural, operational, mixed, unsupported, and indeterminate
conditions while preserving the legacy exact/lower-bound tuple unchanged.
Provider diagnostics remain discovery facts and cannot determine proof state.

## Validity and lifecycle

Source-cache validity, canonical-IR validity, and derived-semantic validity are
separate authorities. A canonical representation can remain valid while a
semantic projection becomes stale.

Cache validity follows semantic dependencies, not merely storage ownership.
The file owning an assertion is not necessarily the full set of files proving
it. Refresh retracts absent owners and proven dependents in its declared scope.
Durable restore validates current source and scoped canonical/semantic
compatibility before state becomes live.

## Refinement ownership

A reconciliation stage owns only its semantic domain. Sharing WorkspaceIndex,
a cache, or durable storage does not authorize replacement of unrelated facts.
Neutral evidence remains neutral when refinement has zero candidates, multiple
candidates, unsupported syntax, or missing evidence.

## Projection fidelity

Canonical semantic authority and model-facing projection are distinct:

- Low, Medium, and High are bounded semantic presentations.
- Edit adds byte-exact edit units under current-session preconditions.
- Verbatim presents exact source.
- Omitted semantic categories must be declared machine-readably.
- Stronger internal acquisition never silently upgrades published fidelity.

Low template output may aggregate or omit order; stronger fidelities preserve
ordered structured occurrences. High does not imply Edit or Verbatim.

## Angular and TypeScript

Current producers cover components, directives, services, pipes, constructor
DI, providers, inject calls, signals, inputs/outputs, NgModules, routing/lazy
routes, NgRx, and structured templates within declared support.

Template authority flows from Component declaration to structured decorator
evidence, HasTemplate, canonical inline/external identity, current template
source, structured selector occurrence, UsesSelector, and exact selector
authority. Only one unique candidate yields UsesComponent; zero or multiple
candidates remain unresolved.

Filename convention is not template identity. templateUrl may authorize an
arbitrarily named HTML file. Inline and external sources have distinct
identities. Selector spelling does not manufacture Component identity.
External content is a semantic dependency. Selector refinement is request
local; Component-owned HasTemplate is the durable relation.

## C# and .NET

Written TypeRefs and method names are source evidence, not exact declarations.
Cold reverse lookup, dependencies, base types, overload/member identity, and
exact Calls reconciliation use current language-specific evidence and fail
closed on ambiguity. Same-name methods are not assumed identical.

ASP.NET controllers/actions/routes, DI, EF Core, project/assembly evidence,
SignalR, AutoMapper, and validation remain producer-owned domains. Direct
current controller evidence is independent of unrelated derived edges. A
reference does not prove declaration existence; losing a derived relationship
does not erase independent declaration authority.

## CBM boundary and topology

Provider evidence can narrow where Clean-CTX looks; only authorized Clean-CTX
evidence determines what it may claim. CBM may change acquisition cost, never
core truth. Filesystem discovery and Clean compilation remain coherent when
CBM is absent or degraded.

    Clean-CTX
      -> owned thin MCP frontend/session and private pipes
         -> CBM coordination/admission
            -> shared account-local daemon, workers, projects, and store

Clean-CTX owns and may retire its frontend. It does not own the shared daemon,
other sessions, installation, graph store, or global process tree.
index_repository affects CBM only; it neither populates nor certifies
WorkspaceIndex. External edits may require refresh_workspace for Clean
authority and separate CBM reindexing for graph freshness.

## Mutation authority

Current normative owner: [Exact source mutation authority](contracts/source-mutation.md). Original text and chronology are retained at commit `9400d476`; this location is navigation, not a second current contract.

## Evidence classes and production owners

Static audit, tracked tests, live/operator verification, and final-gate results
are distinct. One never silently substitutes for another.

Production owners include src/workspace/index/**; query/prepare.rs and
coverage.rs; query/angular_templates.rs; query/csharp_resolution.rs and its
dependencies module; mcp/durable_semantics.rs and compatibility validation;
mcp/tool_handlers/edit.rs; and cbm/client/**.

```architecture-graph
node: AUTH.SEMANTIC_MODEL
kind: AUTHORITY
status: IMPLEMENTED
title: Source-qualified semantic authority composition
track: REPOSITORY
artifact: docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md
preserves:
  - AUTH-001
  - WSC-012
  - WSC-013
  - WSC-014
  - SEMDEP-001
  - PROJ-001
  - WSC-019
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
