# C# Semantic Resolution Correctness Campaign Design

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Origin:** Issue #147 Campaign A and the C# type-reference investigation  
**Phase:** Architecture/design freeze; no production implementation authorized

## 1. Executive verdict

Clean-CTX requires a coordinated C# semantic-resolution correctness campaign. Four known base-type consumers and two attribute families can presently authorize framework identities or relationships from independently interpreted written spelling. A fifth category—the neutral C# base fact—is already structurally sound and must remain the durable source-truth fallback.

The campaign must establish two C#-owned boundaries:

1. type-reference resolution for bases, interfaces, and other type positions;
2. attribute-reference resolution with C# attribute suffix-elision rules.

Both boundaries use a common neutral qualified-name/identity substrate but retain syntax-specific scoping and resolution contracts. Framework modules compare resolved identities with their own framework identities and remain the sole owners of framework meaning.

Campaign GREEN requires migration of every currently known authority-bearing C#/.NET classifier: EF Core `DbContext`, AutoMapper `Profile`, SignalR `Hub`/`Hub<T>`, ASP.NET controller ancestry and authority-bearing attributes, and MSTest `TestClass`. It also requires Level 2 ordinary imports, namespace and type aliases, admitted-source partial symbol composition, generic/nested identity, indirect inheritance for every migrated role, durable/request-local convergence, lifecycle correctness, and retirement of unresolved textual semantic authority.

This is not a Roslyn project. Referenced assemblies, implicit SDK global usings, NuGet metadata, arbitrary compiler binding, and unknown external namespaces remain an explicit fail-closed post-campaign boundary.

## 2. Campaign scope and completion target

Campaign state is frozen as follows:

- 147-C: complete and verified GREEN;
- 147-A: absorbed into this campaign; `DbContext` is one required migration slice;
- C# type-reference investigation: complete; wider pattern proven;
- C# semantic-resolution correctness campaign: current design-freeze phase;
- 147-B: parked until this campaign is GREEN.

The completion target is finite:

> Every known C# framework classifier that publishes or authorizes semantic identity or relationships consumes a structured C# resolution disposition. No known unresolved textual base or attribute spelling remains semantic authority.

Incremental implementation is required for reversibility, but partial slices are not campaign completion. Marker-only heuristics may remain where they cannot create, suppress, or alter semantic truth. Candidate detection may remain textual only when authoritative structured evidence bypasses it.

## 3. Proven authority-bearing consumer inventory

| Consumer | Reference kind | Semantic facts authorized | Current evidence | Current ambiguity handling | Lifetime | Must migrate before GREEN? |
|---|---|---|---|---|---|---|
| EF Core `DbContext` role | BASE_TYPE | durable `dotnet/DbContext` occurrence | exact/global parser-captured first base in Campaign A slice | unsupported simple/alias forms fail closed | durable | yes |
| Legacy EF analyzer | BASE_TYPE | durable `HasEntity`; role-bearing edge subject | `contains(": DbContext")` | none | durable | yes |
| Request-local EF catalog | BASE_TYPE, INTERFACE, OTHER_TYPE_REFERENCE | request-local exact `HasEntity` from configurations/model calls | reparsed catalog plus EF-specific qualification/import helper | source targets may fail closed; external import collision proof incomplete | request-local | yes where it overlaps migrated propositions |
| ASP.NET class role | BASE_TYPE or ATTRIBUTE | framework-classified Controller result | textual base/attribute checks plus stronger request-local ancestry | durable producer has none; request-local handles covered shadows/cycles | durable and request-local | yes |
| ASP.NET route | ATTRIBUTE | durable/request-local `HasRoute` | textual `[Route(...)]` after textual or request-local controller authorization | attribute identity unresolved | durable and request-local | yes |
| ASP.NET action | ATTRIBUTE | durable/request-local `ControllerAction` | textual HTTP verb attributes after controller authorization | attribute identity unresolved | durable and request-local | yes |
| AutoMapper profile | BASE_TYPE | durable `MapsFrom` / `MapsTo` from `MapperProfile` | `contains(": Profile")` | none | durable | yes |
| SignalR hub | BASE_TYPE | durable `HubMethodTargets` from `Hub` | `contains(": Hub")` / `contains(": Hub<")` | none | durable | yes |
| MSTest class | ATTRIBUTE | durable `dotnet/TestClass` declaration, prerequisite for request-local `Tests` | short `[TestClass]` text inside structural attribute list | attribute collision/qualification unresolved | durable and request-local | yes |
| C# neutral base classification | BASE_TYPE / INTERFACE | durable `HasBaseType`; request-local `Extends`/`Implements` | parser capture and admitted-source catalog | unresolved/ambiguous collapse to no refinement | durable neutral, request-local typed | resolver foundation; preserve |
| `TestMethod`, serialization, validation, authorization, caching, DI/service markers | ATTRIBUTE / OTHER | inspected paths emit Φ markers only | textual heuristics | no semantic contract | render-only | no, unless later audit finds semantic authorization |
| .NET applicability detector | CANDIDATE_ONLY | admits or suppresses meta-layer evaluation | AST short names plus string fallback | not a resolver | compilation-local | redesign so it cannot veto proof |

The final pre-GREEN static audit must repeat this inventory against the then-current branch. Any newly discovered authority-bearing consumer joins the campaign; marker-only consumers do not.

## 4. Current resolution machinery

The repository already contains useful but non-unified components:

- `CS_SEMANTIC_QUERY` captures declaration identifiers and base-list type nodes in the compiler's existing C# parse.
- `CoreOp::BaseTypeRef` and durable `HasBaseType` preserve unresolved written source truth.
- same-compilation forward resolution maps uniquely matching local declarations to canonical `Extends`/`Implements`.
- the request-local C# `Catalog` reparses cached source snapshots and indexes declarations/references by namespace, nesting, terminal name, and arity.
- request-local classification derives typed class/interface edges without changing durable neutral facts.
- ASP.NET request-local ancestry handles unique source chains, local shadow rejection, ambiguity, cycles, and qualified/imported framework terminals.
- `EfCoreCatalog` performs separate EF qualification/import tests and source entity/configuration resolution.
- `PreparationContext` shares derived catalogs inside one request and invalidates them when hydration publishes evidence.
- owner-file publication, refresh, deletion, producer-generation compatibility, and `WorkspaceIndex` occurrence ownership already provide durable lifecycle primitives.

Gaps:

- no neutral explicit resolution disposition;
- no canonical C# using/alias fact model suitable for shared resolution;
- using directives are flattened or stored as raw generic import text;
- request-local catalog mixes C# mechanics with ASP.NET meaning;
- partial declarations are not composed into one source symbol;
- external framework identity is recognized independently by consumers;
- durable and request-local authorities can disagree;
- attribute reference identity is not resolved.

## 5. Failure of independent textual authority

The following conditions prove architectural failure:

- `: Profile` can create AutoMapper semantic edges while `: AutoMapper.Profile` does not.
- `: Hub` can create SignalR semantic edges while a qualified equivalent does not.
- local `Profile` or `Hub` declarations can satisfy framework-looking substrings.
- durable ASP.NET base/attribute checks are weaker than the request-local ancestry path.
- EF simple-name import recognition does not establish complete negative shadow proof.
- `[TestClass]`, `[ApiController]`, `[Controller]`, `[Route]`, and HTTP verb spellings are treated as framework attributes without resolving the attribute type.
- textual applicability previously prevented already-proven qualified `DbContext` evidence from reaching publication.

String scanning may still extract payload after role authorization—for example route templates or `CreateMap` arguments—provided the role/attribute authority is structured and the extraction cannot manufacture a different identity.

## 6. Frozen architectural invariants

### CSR-001 — Type spelling is not semantic type identity

Written type syntax is neutral source evidence until resolved under the supported C# contract.

### CSR-002 — Shared resolution, specialized semantics

C# code owns resolution mechanics. Framework code owns `DbContext`, `Controller`, `Profile`, `Hub`, and `TestClass` meaning.

### CSR-003 — Explicit epistemic result

Resolution preserves `Resolved`, `Ambiguous`, `Unsupported`, and `Unresolved` wherever disposition controls semantic authority. Malformed evidence is represented explicitly or as a reasoned unsupported result; it never becomes success.

### CSR-004 — Fail closed

Ambiguous, unsupported, unresolved, or malformed references authorize no framework identity or relationship.

### CSR-005 — Equivalent resolvable syntax converges

Supported qualification, ordinary import, and alias spellings that resolve to one identity produce the same framework semantics.

### CSR-006 — Resolution dependencies govern validity

Derived semantics are current only while every fact used in their resolution proof remains compatible.

### CSR-007 — Neutral evidence survives failed resolution

Failure to resolve never removes `HasBaseType`, captured attributes, or other parser-owned neutral evidence.

### CSR-008 — Candidate detection is not semantic authority

Heuristics may schedule work, never establish framework truth.

### CSR-009 — Applicability cannot suppress proof

An authoritative structured classification bypasses or positively admits the applicable producer regardless of detector text.

### CSR-010 — No half-migrated semantic truth at GREEN

At campaign GREEN, no known authority-bearing consumer may derive framework identity or relationships from unresolved textual type or attribute spelling. Explicitly unsupported syntax is permitted only when it fails closed, preserves neutral evidence, and is documented. The invariant concerns unmigrated authority, not compiler-grade completeness.

## 7. Neutral type-reference resolution contract

The design requires repository-named equivalents of:

```rust
enum CSharpResolution<T> {
    Resolved(T),
    Ambiguous(ResolutionReason),
    Unsupported(ResolutionReason),
    Unresolved(ResolutionReason),
}
```

Reasons are required for deterministic tests, diagnostics, dependency capture, and future extension. They must be compact closed categories, not arbitrary framework strings. Candidate reasons include competing declarations/imports, missing admitted target, unsupported external universe, malformed syntax, unsupported reference form, and conflicting partial parts.

Neutral type identity contains:

- namespace path;
- containing/nested type path;
- terminal name;
- generic arity;
- identity origin: admitted source symbol or explicitly known external symbol;
- for source symbols, composed declaration identity/owners;
- for external symbols, canonical qualified name and arity;
- resolution basis: explicit/global, lexical declaration, ordinary import, namespace alias, type alias, or inheritance step.

The identity contains no framework role. Framework modules own constants or small predicates comparing neutral identity.

## 8. Attribute-reference resolution contract

Attributes need a distinct resolver because C# permits attribute suffix elision and because attribute syntax/scoping differs from base types.

The attribute resolver reuses:

- neutral qualified-name segments;
- lexical import and alias tables;
- admitted source declaration lookup;
- known external type identities;
- explicit resolution disposition and dependency evidence.

It adds attribute-specific candidate rules:

- written `TestClass` may match type `TestClass` or `TestClassAttribute` under C# precedence;
- explicitly written `TestClassAttribute` has no suffix expansion;
- qualified and alias-qualified forms retain their qualifier authority;
- multiple viable suffix candidates are ambiguous;
- local/source attributes shadow known external imported identities according to the supported precedence model.

Captured attribute facts must include owner declaration, attribute name/type node span, written form, lexical scope, and source owner. Argument payload is separate from attribute identity.

The first required consumers are:

- MSTest `TestClassAttribute`;
- ASP.NET `ApiControllerAttribute` and `ControllerAttribute` for direct controller authority;
- ASP.NET `RouteAttribute` for `HasRoute`;
- supported HTTP method attributes for `ControllerAction`.

`Authorize`, serialization, validation, `TestMethod`, and other marker-only attributes do not become semantic migrations unless a later audit finds an authority-bearing path.

## 9. Resolution level model

| Level | Capability | Campaign status |
|---|---|---|
| L1 | explicit namespace qualification and `global::` | required |
| L2 | ordinary imports and simple names with negative proof | required |
| L3 | namespace aliases | required |
| L4 | type aliases | required |
| L5 | admitted-source cross-file/nested/partial resolution | required; also supports L2 negative proof |
| L6 | indirect inheritance with cycles and dependency chains | required for all migrated framework roles |
| L7 | referenced-project/assembly compiler symbols | explicitly outside campaign except known external identity catalog |

Levels are capability layers, not necessarily merge order. L5 declaration indexing must precede sound L2 because ordinary-import external resolution requires negative source proof.

## 10. Level 1 design

L1 parses a structured name into namespace, containing type path, terminal name, and arity. `global::` marks root qualification and is not part of identity. Resolution succeeds only when the complete written path equals:

- an admitted source identity; or
- an explicitly modeled external identity supplied by the consuming framework module for neutral comparison.

No import, alias, project graph, or provider evidence participates. Owner-file bytes and resolver/producer version are the dependencies. Cold cost is linear in captured references; warm durable semantics reuse the published result.

L1 applies uniformly to EF, ASP.NET, AutoMapper, SignalR, and required framework attributes.

## 11. Level 2 ordinary-import authority

For `using Microsoft.EntityFrameworkCore; class A : DbContext`, the import alone is insufficient. Resolution requires:

1. parser-owned simple reference with known arity;
2. lexical set of applicable normal imports;
3. complete admitted source declaration index for the binding scope;
4. no lexical/nested/current-namespace source declaration winning precedence;
5. no conflicting source candidates from imports;
6. exactly one known external identity candidate;
7. no competing ordinary import whose contents are unknown and could contain the same terminal/arity.

Because Clean-CTX does not load arbitrary assemblies, an additional unknown import makes the external simple name `Ambiguous` or `Unresolved` rather than guessed. A source declaration from an unknown external namespace is not invented.

Negative proof examines:

- nested containing types from inner to outer;
- current namespace and parent namespace lookup supported by the resolver contract;
- admitted partial symbol composition;
- imported admitted-source namespaces;
- known external identities for all applicable imports;
- terminal name and generic arity;
- duplicate/conflicting declarations across admitted roots/projects.

The declaration universe must be current and complete for the configured authority scope before L2-derived durable facts publish. Incomplete acquisition produces `Unresolved`, never partial authority.

## 12. Namespace-alias design

Example:

```csharp
using EfCore = Microsoft.EntityFrameworkCore;
class A : EfCore.DbContext {}
```

Required syntax fact:

- alias identifier;
- target qualified namespace/type syntax;
- lexical scope (compilation unit or namespace body);
- source span and owner file;
- `global using` flag where explicitly present;
- malformed/duplicate state.

Resolution substitutes the alias only as a qualified-name prefix, then resolves the resulting neutral identity under L1/L5 rules. It does not interpret framework meaning. Inner applicable aliases shadow outer aliases under supported C# scope rules. Duplicate applicable aliases or source conflicts yield `Ambiguous`/malformed failure.

Owner edits, alias retargeting/removal, scope movement, and target declaration changes invalidate dependent resolutions.

## 13. Type-alias design

Example:

```csharp
using ContextBase = Microsoft.EntityFrameworkCore.DbContext;
class A : ContextBase {}
```

Type aliases are stored distinctly from namespace aliases after parsing the target shape. The alias target may be a qualified external identity, admitted source type, nested type, or generic constructed type within the supported grammar.

The resolver first selects the lexically applicable alias, detects duplicates/conflicts, resolves the alias target, and carries both alias declaration and target dependencies into the proof. Alias chains are either explicitly bounded and cycle-checked or initially unsupported; silent recursive expansion is prohibited.

Retarget, edit, deletion, scope movement, target rename, partial composition change, and producer-version change invalidate dependents.

## 14. Source-declaration universe

The source universe is the current admitted C# inventory under configured trusted roots, not the query result filter:

- primary workspace root;
- configured additional roots;
- admitted `.cs` source after path/safety/exclusion policy;
- source-generation identity and current cached source ownership;
- project association when a unique local `.csproj` boundary is provable;
- no CBM semantic authority.

Each symbol key contains namespace, containing types, terminal name, arity, and kind. Declarations retain physical owner files and part spans. Duplicate identical declarations are not automatically ambiguity: valid partial parts compose; non-partial duplicates or incompatible kinds/arity are conflicts.

`withinPath` filters returned query evidence; it must not redefine the binding universe. Requested-root authorization controls visibility, while the configured admitted universe controls negative proof. If policy prevents access to a potentially binding source set, resolution is unresolved rather than falsely complete.

## 15. Partial declaration semantics

Campaign GREEN requires composition of admitted-source partial types relevant to migrated framework roles.

Rules:

- all parts with the same namespace/nesting/name/arity/kind form one candidate symbol only when every part is declared `partial` as required by C#;
- one part may carry the base list while another carries members/attributes;
- framework role is symbol-level; its semantic assertion retains the proving part and complete part dependency set;
- duplicate matching base specifications converge only when they resolve to the same identity;
- conflicting base identities, kind/arity disagreements, non-partial duplicates, or inconsistent source/project ownership fail closed as `Ambiguous` or unsupported malformed composition;
- adding/removing/renaming any part invalidates the symbol and dependents;
- relationship payload may come from any current part only after the composed symbol role is authorized.

This is bounded source composition, not metadata partial-method or arbitrary compiler merging.

## 16. Generic/nested type identity

Neutral identity never erases:

- generic arity at every named type segment where represented;
- containing/nested path;
- distinction between open declaration identity and constructed reference arguments;
- exact written constructed arguments as neutral evidence when argument identity is not needed.

Framework comparison uses canonical definition identity plus required arity:

- `Hub` has arity 0;
- `Hub<TClient>` resolves to the `Hub` definition with arity 1 while preserving the client type reference separately;
- `DbContext`, `Profile`, `Controller`, and `ControllerBase` use their actual supported arities.

Nested source bases resolve through the declaration index. String splitting that loses nested structure or generic arity is prohibited.

## 17. Indirect inheritance authority

Indirect role inheritance is required for every migrated framework role because role semantics attach to derived C# types, not only to the spelling of the immediate base.

The resolver constructs a source-declaration inheritance graph from resolved direct bases. A role classifier walks only resolved class-base arcs until it reaches a supported external framework identity or terminates.

Required controls:

- deterministic cycle detection;
- ambiguity at any hop fails the entire proof;
- unsupported/unresolved hop fails closed;
- interface entries never become class-base ancestry merely by list position after structured classification is available;
- proof records every declaration/reference/import/alias/partial dependency in the chain;
- root/scope authorization cannot expose a role whose proof uses unauthorized source;
- existing ASP.NET multi-hop behavior and shadow rejection remain the compatibility floor.

## 18. Referenced-project/assembly boundary

The campaign does not implement arbitrary assembly symbol loading, NuGet restore, implicit SDK global usings, or Roslyn binding.

Supported external identities are explicit neutral qualified identities needed by migrated framework consumers. Their existence is a product contract, not proof derived from an arbitrary installed assembly. Simple-name import resolution succeeds only when the import-to-known-identity mapping is unique and admitted-source negative proof is complete.

Source from configured roots can participate in L5 even across multiple local projects when admitted and project identity is unambiguous. Actual reference legality between projects is not claimed without a project-reference authority; cases requiring that proof are `Unsupported` or `Unresolved`.

## 19. EF Core migration design

Target identities include `Microsoft.EntityFrameworkCore.DbContext` and the EF configuration interfaces currently used by request-local exact entity projection.

Migration requirements:

- compose the class symbol and resolve direct/indirect base ancestry;
- create the durable `dotnet/DbContext` role independently of `DbSet<T>`;
- authorize all durable and request-local EF relationship extractors from the same resolved role proof;
- preserve zero relationships when no entity evidence exists;
- retain `builtin/Class` as declaration authority;
- keep role registration out of adjacency/declaration maps as currently contracted;
- resolve configuration interfaces and referenced source entity types through the shared mechanics;
- remove `: DbContext` and EF-specific import spelling as semantic fallbacks;
- keep textual `DbSet`/model payload extraction only behind authorized role and structured invocation/type evidence appropriate to its existing contract.

First framework RED: an ordinary imported fully supported context, a qualified context, a global-qualified context, and alias forms converge; local/same-namespace shadow and competing import controls publish no role or relationships.

## 20. AutoMapper migration design

Target identity: `AutoMapper.Profile`.

The composed class role is authorized through resolved direct or indirect ancestry. `CreateMap` extraction remains payload evidence; it cannot create `MapperProfile` identity by itself.

Required outcomes:

- explicit, global, ordinary-import, namespace-alias, and type-alias forms converge;
- local/shadow `Profile` fails closed as framework authority;
- ambiguous imports produce no mapping edges;
- indirect derived profiles are classified when the chain is resolved;
- mappings remain zero when no `CreateMap` evidence exists;
- legacy `contains(": Profile")` becomes candidate/render-only or is removed.

## 21. SignalR migration design

Target identities: `Microsoft.AspNetCore.SignalR.Hub` arity 0 and arity 1.

Required outcomes:

- `Hub`, qualified/global Hub, alias forms, and `Hub<T>` resolve by identity and arity;
- `TClient` remains a structured neutral type reference, not part of the Hub identity string;
- local/shadow Hub and wrong arity cannot authorize `HubMethodTargets`;
- indirect derived hubs resolve through the ancestry graph;
- hub role alone creates no method target; method evidence remains required;
- legacy Hub substring tests lose semantic authority.

## 22. ASP.NET migration design

Target base identities: `Microsoft.AspNetCore.Mvc.Controller` and `ControllerBase`. Target class attributes: `ApiControllerAttribute` and `ControllerAttribute`.

ASP.NET migration must unify, not weaken, the current request-local contract:

- preserve exact source ancestry, namespace/nesting/arity resolution, multi-hop traversal, cycle rejection, and local-shadow rejection;
- authorize durable and request-local controller projections from one resolution proof;
- direct class attributes can independently prove controller role only when their attribute identities resolve;
- route and HTTP action relationships require an authorized controller role and resolved attribute identities;
- textual route/method payload parsing remains behind attribute authorization;
- durable and request-local output agree for the overlapping admitted source generation;
- replacement, ancestry changes, attribute changes, and source removal retract both projections;
- old base/attribute substring authority is retired.

## 23. MSTest/attribute migration design

Target identity: `Microsoft.VisualStudio.TestTools.UnitTesting.TestClassAttribute`, supporting C# suffix elision.

The durable TestClass declaration is published only after attribute resolution. Request-local `Tests` remains declaration-proven and continues to require unique source subject authority.

Required cases:

- `[TestClass]` with a unique ordinary MSTest import;
- fully qualified and global-qualified attribute names;
- explicit `TestClassAttribute` spelling;
- namespace and type aliases where legal;
- local `TestClassAttribute` or `TestClass` shadow;
- competing imports;
- partial class attribute on any composed part;
- attribute removal, alias retargeting, part deletion, and subject changes.

ASP.NET attributes migrate through the same attribute resolver in their own consumer slice. Marker-only attributes stay textual unless they later become semantic authority.

## 24. Applicability-gate redesign

Meta-layer evaluation separates candidate cost control from semantic authorization:

1. the single parser/capture pass produces neutral C# references;
2. resolution produces framework-neutral identities/dispositions;
3. any resolved identity relevant to a registered framework consumer positively admits that consumer's semantic evaluation;
4. textual detector hits may additionally admit candidate/render work;
5. textual detector misses cannot veto step 3.

No permanent solution may be a longer list of textual spellings. Framework comparison remains in each framework module. Marker-only work may still use the detector, but authoritative semantic publication follows structured admission.

## 25. Durable/request-local convergence

One cache is not mandated. The recommended ownership is:

- generation-owned C# symbol facts and resolution primitives shared conceptually and structurally;
- durable producer consumes immutable resolution results during semantic publication;
- request-local queries consume the same identity/disposition algorithms and the same current symbol generation, adding query-specific projections without republishing them;
- request-local preparation may cache views for batching;
- durable and request-local facts compare resolution generation/fingerprint before use.

The current request-local catalog should be decomposed: neutral declaration/reference indexing moves toward C# resolution ownership; ASP.NET and EF interpretation stays in framework/query modules. During migration, parity tests prevent two authorities from disagreeing. At completion, overlapping supported propositions have one resolution result even if projection lifetimes differ.

## 26. Scope and source-universe authority

Binding authority uses the configured admitted source universe, not `withinPath` and not the requested result scope. Visibility rules still filter returned occurrences.

Required distinctions:

- configured roots define potentially authoritative source inventory;
- project association narrows source resolution only when provable;
- requested root authorizes the operation and returned data;
- `withinPath` filters results, never supplies negative binding proof;
- excluded/unadmitted/generated sources are outside authority and may force unresolved status if they could bind;
- CBM can discover files but cannot decide resolution;
- source inventory completeness must be established before negative proof.

Cross-root same-name symbols retain owner identity. Ambiguity is determined by C# binding candidates, not by merging names globally.

## 27. Lifecycle/invalidation matrix

| Level/fact | Evidence dependencies | Cache/projection owner | Invalidated by | Warm reuse condition |
|---|---|---|---|---|
| L1 explicit/global | owner bytes, parse/query version | owner artifact/projection | owner edit/delete/rename, producer bump | same owner hash and versions |
| L2 ordinary import | owner, lexical imports, applicable declaration/external candidate set, inventory generation | C# resolution generation | using edit, source add/remove/rename, namespace/arity/kind change, root/inventory change | identical dependency fingerprint |
| L3 namespace alias | L2 plus alias declaration/scope/target | C# resolution generation | alias edit/retarget/delete/scope move, target change | alias and target fingerprints compatible |
| L4 type alias | alias declaration plus resolved target chain | C# resolution generation | alias/target edit, alias cycle, source target lifecycle | complete chain compatible |
| L5 source symbol | all composed declaration parts and project/scope identity | symbol index generation | part add/remove/edit, duplicate/conflict, root/source exclusion change | symbol fingerprint unchanged |
| L6 ancestry | resolved symbol/base chain and external terminal | derived role projection | any hop/reference/alias/import/part change, cycle/ambiguity, producer bump | complete proof-chain fingerprint compatible |
| Attribute | owner/part, lexical imports/aliases, suffix candidate set | symbol/resolution generation | attribute/using/alias/part/inventory change | identity proof compatible |
| Framework durable fact | resolution proof plus payload facts | owner semantic projection | proof or payload change, framework producer bump | both resolution and payload compatible |
| Request-local refinement | current resolution generation and hydrated source completeness | preparation context | publication/hydration change, generation change | same request and compatible generation |

Invalidation is dependency-driven. A global generation may conservatively mark affected capability views dirty, but recompilation/resolution work must be limited through reverse dependency indexes rather than global reconciliation per publication.

## 28. Performance model

Cold construction:

- one existing parser pass emits declarations, bases, attributes, imports, and aliases: O(source bytes + facts);
- declaration/partial index: O(D log D) deterministically or expected O(D) hashed;
- lexical import/alias index: O(U + A);
- each direct resolution uses bounded lexical candidates and indexed path/arity lookup: O(scope depth + imports + candidate collisions), not O(D);
- ancestry classification is O(nodes/edges in the reached proof subgraph), cycle-checked and memoized per generation.

Warm behavior:

- unchanged owner and dependency fingerprints reuse resolution/role results in O(1) currentness checks plus result retrieval;
- request batches reuse one prepared generation view;
- one changed declaration invalidates reverse dependents, not every C# file.

Prohibited:

- whole-workspace reparse per query;
- scan of every declaration per reference;
- N×N matching;
- separate parser passes per framework;
- provider blocking or provider semantic input;
- global resolution rebuild per unrelated publication.

Do not introduce speculative indexes beyond declaration path/arity, lexical imports/aliases, partial grouping, and reverse resolution dependencies until measurement shows need.

## 29. Legacy textual detector retirement matrix

| Legacy path | Final state |
|---|---|
| `contains(": DbContext")` semantic gate | removed as authority; optional candidate/render hint only |
| EF import/qualification bespoke helper | replaced by shared type resolution; framework comparison remains |
| `contains(": Profile")` semantic gate | removed as authority; optional render candidate only |
| `contains(": Hub")` / `Hub<` semantic gate | removed as authority; optional render candidate only |
| ASP.NET `: Controller*` semantic gate | removed as authority after durable/request-local convergence |
| ASP.NET `[ApiController]` / `[Controller]` role gate | replaced by attribute resolution |
| ASP.NET `[Route]` and HTTP verb relationship gates | identity authorized by attribute resolver; payload extraction may remain textual within captured attribute span |
| MSTest short `[TestClass]` declaration gate | replaced by attribute resolution |
| .NET detector strong signals | candidate-only; cannot veto resolved identities |
| general/serialization/validation marker scans | render-only, retained unless separately defective |

Final static audit searches every semantic-edge and framework declaration producer, not just named helpers. No hidden textual fallback is permitted.

## 30. Ordered implementation campaigns

1. **R0 — Neutral syntax facts and dispositions.** Add structured declarations, base/type shapes, attribute references, lexical using/alias facts, neutral identity, and explicit result types. No framework migration.
2. **R1 — Admitted-source symbol index and partial composition.** Namespace/nesting/name/arity/kind keys, partial grouping, conflicts, generation identity.
3. **R2 — L1 and L5 direct resolution.** Explicit/global and admitted-source qualified identities; preserve neutral facts.
4. **R3 — L2 ordinary imports.** Implement negative shadow/collision proof and unknown-import fail-closed behavior.
5. **R4 — L3/L4 aliases.** Namespace aliases, type aliases, lexical scope, cycles, retarget lifecycle.
6. **R5 — L6 ancestry.** Shared resolved base graph, cycles, proof dependencies, generic/nested identities.
7. **R6 — EF Core migration.** Absorb Campaign A, role independence, relationship authorization, request-local EF convergence; retire EF textual authority.
8. **R7 — AutoMapper migration.** Profile role and mappings with shadow/ambiguity controls; retire Profile textual authority.
9. **R8 — SignalR migration.** Hub arity 0/1, client generic structure, indirect roles; retire Hub textual authority.
10. **R9 — ASP.NET base convergence.** Move durable and request-local controller ancestry to shared results without losing existing behavior.
11. **R10 — Attribute resolver and MSTest migration.** Suffix elision, imports/aliases/shadows/partials; retire TestClass textual authority.
12. **R11 — ASP.NET attribute migration.** Direct controller, route, and action attribute authority; retire attribute textual authority.
13. **R12 — Applicability and cross-consumer convergence.** Structured admission, detector demotion, parity, lifecycle, cross-root and warm/cold acceptance.
14. **R13 — Final retirement/audit/documentation.** Static authority scan, obsolete path removal, invariant/docs updates, complete verification gate.

Each slice follows RED → user-observed intended failure → implementation → unchanged GREEN → lifecycle/negative control. A slice does not silently migrate the next consumer.

## 31. RED/GREEN regression matrix

| Layer/consumer | Positive RED | Negative control | Lifecycle |
|---|---|---|---|
| Resolution disposition | explicit/global identity equality; distinct failure states | malformed/unsupported does not resolve | owner edit/version change |
| Source symbols/partials | qualified nested/generic and one base-bearing partial part | non-partial duplicate/conflicting partial base | part add/remove/rename |
| L2 imports | unique known import and no shadow resolves | local/same-namespace/nested shadow, competing/unknown import | import and source candidate add/remove |
| Namespace alias | alias-qualified external/source identity | duplicate/unknown alias | retarget/delete/scope move |
| Type alias | direct external/source target | cycle/ambiguous target | retarget/delete/target rename |
| Ancestry | multi-hop role reaches resolved terminal | cycle, ambiguous or unresolved hop | middle-hop edit/delete |
| EF | every supported spelling yields same role and relationships | local `DbContext`, ambiguity, zero-edge non-fabrication | refresh/rename/base removal/delete/cross-root/warm |
| AutoMapper | supported Profile spellings yield same mappings | local Profile and no-CreateMap zero relation | alias/base/mapping edit/delete |
| SignalR | Hub and Hub<T> spellings/arity converge | local Hub, wrong arity, no-method zero relation | base/client/method edit/delete |
| ASP.NET bases | supported direct/indirect forms converge | shadow, ambiguity, cycle | ancestry/route/action edit/delete |
| Attribute resolver | qualified/imported/alias and suffix forms converge | local suffix collision/competing import | attribute/alias/part change |
| MSTest | supported TestClass forms yield one declaration/Tests behavior | local TestClassAttribute and absent subject | attribute/part/subject edit/delete |
| Cross-framework | bounded workspace proves shared mechanics | no role contamination across identical simple names | mixed edits and warm/cold parity |
| Durable/request-local parity | same generation returns same role identity | incomplete generation yields no authoritative refinement | hydration/publication generation change |

Tests remain layered rather than one monolith: resolver unit contracts, framework integration, lifecycle suites, and one bounded cross-framework acceptance fixture.

## 32. Definition of C# semantic resolution GREEN

GREEN requires all of the following:

1. neutral explicit resolution dispositions exist for types and attributes;
2. every known authority-bearing base classifier is migrated;
3. every known authority-bearing attribute classifier is migrated;
4. full and `global::` qualification converge;
5. ordinary imports work with complete admitted-source shadow/ambiguity protection;
6. namespace aliases work with lexical scope and lifecycle;
7. type aliases work with target/cycle/lifecycle controls;
8. framework-relevant partial declarations compose as one symbol;
9. generic arity and nested identity are preserved, including `Hub<T>`;
10. indirect inheritance works for all migrated framework roles;
11. durable and request-local results converge within the shared supported envelope;
12. owner, dependency, partial, alias, ancestry, root, refresh, and deletion lifecycle tests pass;
13. cross-root and warm/cold behavior agree;
14. neutral evidence survives every failed resolution;
15. applicability gates cannot suppress proof;
16. no unresolved textual base or attribute matcher authorizes known semantic facts;
17. no provider semantic authority or per-query whole-workspace reparse is introduced;
18. documentation describes implemented capability and unsupported boundaries accurately;
19. focused RED/GREEN evidence exists per slice;
20. the authoritative final verification gate is user-run GREEN.

Campaign GREEN does not claim arbitrary assembly binding or full C# compiler equivalence.

## 33. Explicitly unsupported post-campaign boundary

After GREEN, these remain explicit truthful boundaries:

- arbitrary referenced-assembly and NuGet symbol resolution;
- implicit SDK/global usings not present in admitted source;
- project-reference legality not proven by current source/project authority;
- unknown external namespace contents when needed for negative proof;
- arbitrary generic constraint and variance binding;
- method overload/invocation binding beyond existing separate contracts;
- arbitrary expression/name lookup;
- malformed C# beyond structured fail-closed handling;
- cross-language type resolution;
- framework identities not included in migrated consumers until independently approved.

These cases return `Unsupported`, `Unresolved`, or `Ambiguous`; they never fall back to framework-looking strings.

## 34. Documentation/invariant plan

Update only as implementation lands:

- `docs/ARCHITECTURAL_INVARIANTS.md`: add enforced CSR invariants and update DOTNET-001 ownership;
- `docs/agent/DISCOVERY_REGISTRY.md`: record wider-pattern discovery, each migration, and final retirement audit;
- `docs/CLAUDE_INTEGRATION_RULES.md`: project the supported C# authority and fail-closed boundary only if agent-facing behavior changes;
- C# IR/semantic documentation: neutral facts, identity/dispositions, imports/aliases/partials, dependency generation;
- `docs/DOTNET_META_LAYER.md`: per-framework migration status and removal of textual authority;
- query authority/completeness documentation: durable/request-local convergence and incomplete-universe behavior;
- Issue #147 planning record: Campaign A absorption and verification status.

Never document a planned level as current capability. Each invariant cites its actual type/test enforcement.

## 35. Explicit non-goals

- Roslyn integration or a full C# compiler.
- Package restore, NuGet metadata, or arbitrary assembly symbol loading.
- Arbitrary expression, overload, or method binding.
- Replacing the separate exact C# call contract.
- CBM-authored semantic identity.
- Cross-language type resolution.
- Campaign B.
- Marker cleanup unrelated to semantic authority.
- Repository-wide stylistic refactoring.
- A generic framework registry without an earned need.
- Broadening string matchers as a substitute for resolution.

## 36. Runtime evidence still required, if any

No runtime evidence is required to freeze this design. Production branches, current types, existing tests, and lifecycle ownership statically establish the consumer inventory and architectural gaps.

Runtime evidence will be required during implementation for each RED/GREEN slice, performance work-count assertions, lifecycle behavior, and the final gate. Under repository policy, Agent MaxHeadRoom will hand exact commands to the user and will not run Cargo, builds, tests, Clippy, binaries, MCP live commands, or servers without explicit per-run authorization.

## 37. Final recommendation

Approve the ordered R0–R13 campaign as one completion contract implemented through reversible consumer-specific slices. Build shared neutral type and attribute resolution, establish the admitted-source declaration/partial/import/alias/ancestry authority, migrate all known semantic consumers, demote or remove textual authority, and finish only after cross-consumer parity and lifecycle verification.

C#-SEMANTIC-RESOLUTION-CAMPAIGN:
REQUIRED — KNOWN AUTHORITY-BEARING CLASSIFIERS MUST MIGRATE

TYPE-REFERENCE-BOUNDARY:
SHARED C# RESOLUTION REQUIRED

ATTRIBUTE-BOUNDARY:
REQUIRED IN THIS CAMPAIGN

LEVEL-2:
REQUIRED BEFORE CAMPAIGN GREEN

NAMESPACE-ALIASES:
REQUIRED BEFORE CAMPAIGN GREEN

TYPE-ALIASES:
REQUIRED BEFORE CAMPAIGN GREEN

PARTIAL-TYPES:
REQUIRED BEFORE CAMPAIGN GREEN

INDIRECT-INHERITANCE:
REQUIRED FOR ALL MIGRATED FRAMEWORK ROLES

CAMPAIGN-A:
ABSORBED — DBCONTEXT BECOMES ONE VERIFIED MIGRATION SLICE OF THE C# SEMANTIC RESOLUTION CAMPAIGN

CAMPAIGN-B:
PARKED UNTIL C# SEMANTIC RESOLUTION CAMPAIGN IS GREEN

Production implementation authorized: NO
Repository-wide textual matcher broadening authorized: NO
Provider semantic authority authorized: NO

1. **Exact ordered implementation slices:** R0 neutral facts/dispositions; R1 admitted-source symbol index and partial composition; R2 L1/L5 direct resolution; R3 L2 imports; R4 aliases; R5 ancestry; R6 EF; R7 AutoMapper; R8 SignalR; R9 ASP.NET bases; R10 attribute foundation/MSTest; R11 ASP.NET attributes; R12 applicability and convergence; R13 retirement/audit/documentation.
2. **Exact GREEN checklist:** all 20 requirements in Section 32, including every known consumer migrated, Levels 1–6 supported as bounded, type and attribute aliases supported, partials composed, durable/request-local parity, lifecycle/cross-root/warm-cold proof, textual authority retired, documentation current, and the user-run final gate GREEN.
3. **Post-campaign unsupported boundary:** arbitrary assembly/NuGet/project-reference binding, implicit SDK evidence absent from admitted source, unknown external namespace contents, compiler-grade expression/overload binding, cross-language resolution, and unmigrated future frameworks—all explicit fail-closed outcomes, never textual fallback.
4. **Smallest first RED:** a framework-neutral resolver contract test proving that a parser-owned fully qualified and `global::` type reference produce one identical neutral `Resolved` identity while malformed and intentionally unsupported forms return distinct non-success dispositions and preserve the neutral written reference; no framework role or relationship may appear in this test.
