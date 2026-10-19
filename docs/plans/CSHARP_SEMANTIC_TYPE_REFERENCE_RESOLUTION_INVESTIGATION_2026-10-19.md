# C# Semantic Type-Reference Resolution Boundary Investigation

**Agent:** Agent MaxHeadRoom  
**Date:** 2026-10-19  
**Repository:** Clean-CTX  
**Origin:** Issue #147 Campaign A  
**Mode:** Architecture investigation only; no production implementation authorized

## 1. Executive verdict

Clean-CTX has a wider C# authority pattern, not an isolated `DbContext` spelling bug. At least four framework families currently attach semantic meaning to independently interpreted base-type text:

- EF Core `DbContext` can author a durable role and `HasEntity` relationships;
- ASP.NET controller classification can author `Controller`, `HasRoute`, and `ControllerAction` facts;
- AutoMapper `Profile` can author `MapperProfile` mapping relationships;
- SignalR `Hub` can author `HubMethodTargets` relationships.

The repository also has substantial structured C# resolution machinery, but no single reusable boundary currently satisfies the required contract. The durable canonical projection deliberately retains written `TypeRef` evidence. The request-local C# `Catalog` resolves admitted source declarations by namespace, nesting, name, and arity, but it collapses all failure modes into `Option`, omits using aliases, and includes framework-specific ASP.NET interpretation. `EfCoreCatalog` separately reinterprets EF imports and qualification.

A shared C#-owned resolution boundary is therefore required. Its first implementation should be deliberately small: parser-owned base references that are already fully namespace-qualified, with optional `global::`, resolved to a neutral external type identity. `DbContext` should be the only first consumer. Normal imports, aliases, source-declared cross-file symbols, and indirect inheritance should remain outside that first durable authority envelope.

This conclusion does not authorize a repository-wide migration. Other consumers need independent RED tests and migrations because their public facts, ambiguity hazards, and lifecycle requirements differ.

## 2. Trigger from Issue #147 Campaign A

Issue #147 demonstrated that role identity and relationship payload had been coupled. A `DbContext` with no `DbSet<T>` had no durable `dotnet/DbContext` occurrence. The first corrective slice used the existing `csharp.base_type` capture and recognized exactly:

```csharp
Microsoft.EntityFrameworkCore.DbContext
global::Microsoft.EntityFrameworkCore.DbContext
```

That slice also exposed a second issue: the .NET meta-layer applicability gate still depended on legacy textual signals. A relationship-bearing context entered the layer while a zero-relationship fully qualified context did not. The same semantic proposition therefore followed different production paths solely because of spelling and incidental payload.

Per the investigation brief, campaign sequencing remains frozen for architectural purposes:

- 147-C: complete and verified;
- 147-A: architecture approved, production continuation paused at the resolution boundary;
- this investigation: current;
- 147-B: not started.

The working tree contains the earlier narrow Campaign A implementation evidence. This investigation evaluates its boundary but does not authorize further production changes.

## 3. Why this is potentially wider than DbContext

C# type syntax permits unqualified, namespace-qualified, global-qualified, namespace-alias-qualified, and type-alias forms. Source declarations can also shadow framework simple names. A substring such as `: Profile` or `: Hub` is consequently neither necessary nor sufficient proof of framework identity.

The risk becomes architectural when textual spelling does more than render a marker. In the current .NET meta-layer, some textual base checks gate semantic-edge construction. Those facts can enter `WorkspaceIndex`, affect query visibility, and survive until owner replacement or refresh. Equivalent spelling differences can therefore alter externally visible semantic results.

The concern is C# resolution mechanics, not a demand for compiler-grade binding. Clean-CTX only needs an explicit supported authority envelope and fail-closed outcomes beyond it.

## 4. Inventory of current C#/.NET semantic classifiers

| Consumer | Meaning produced | Evidence source | Current matching | Qualification / alias handling | Ambiguity handling | Lifetime |
|---|---|---|---|---|---|---|
| EF Core `DbContext` structured role | `dotnet/DbContext` occurrence, independent of relationships | parser-owned first `csharp.base_type` associated with `class.root` | exact `Microsoft.EntityFrameworkCore.DbContext` after optional `global::` removal | full/global only; no imports or aliases | unsupported forms fail closed | durable owner-file projection |
| Legacy EF Core analyzer | `HasEntity`; EF marker output | tree-sitter-bounded raw class text | `contains(": DbContext")`, then textual `DbSet<...>` | simple written form only | no semantic ambiguity model | durable edges plus marker output |
| Request-local EF catalog | exact `HasEntity` from `OnModelCreating` and configurations | reparsed session source catalog plus admitted declarations | exact qualified EF name or unqualified name with EF using | full/global and ordinary using; no alias | source target resolution can fail closed, but external EF import recognition does not rule out all shadows | request-local, cached per preparation context |
| ASP.NET analyzer | `Controller` endpoint identity through edge subjects, `HasRoute`, `ControllerAction`; markers | tree-sitter-bounded raw class text | attributes or `contains(": ControllerBase")` / `contains(": Controller")` | simple base only; attributes use short textual names | no base ambiguity model in durable producer | durable edges plus marker output |
| ASP.NET query refinement | controller occurrence and query-local route/action projection, including ancestry | request-local C# catalog | exact full framework base, ordinary MVC using, or uniquely resolved source ancestry | full/global, ordinary using, source ancestry; no aliases | cycles, missing targets, and competing source declarations fail closed in covered paths | request-local |
| AutoMapper profile | `MapsFrom` / `MapsTo` from `MapperProfile` | raw class text | `contains(": Profile")` plus `CreateMap<...>` scanning | simple base only; no aliases | none | durable semantic edges plus markers |
| SignalR hub | `HubMethodTargets` from `dotnet/Hub` | raw class text | `contains(": Hub<")` or `contains(": Hub")` | simple base only; no aliases | none | durable semantic edges plus markers |
| MSTest class | `dotnet/TestClass` declaration; request-local `Tests` refinement | class AST boundary, then short attribute text | `[TestClass]` / `[TestClass(...` inside an `attribute_list` | qualified/aliased attribute identity is not resolved | absent rather than guessed; attribute-name collisions are not resolved | durable declaration; query-local test-subject relation |
| C# neutral base classification | `HasBaseType` retained; query-local `Extends` / `Implements` | parser-owned canonical base reference plus request-local source catalog | namespace, nesting, terminal name, arity, and ordinary using candidate paths | qualified names, generics, normal using; no aliases | unresolved and ambiguous both yield no typed refinement | durable neutral fact; typed refinement request-local |
| General DI/service, validation, serialization, identity, caching, logging | Φ marker lines only in the inspected implementation | bounded raw class text | conventions, attributes, generic-call strings | written forms only | no resolution contract | rendered meta output, not semantic identity/relationship authority |
| .NET detector | meta-layer candidate/applicability decision | AST attribute/simple-base query plus string fallback | known short attributes/base names and other signals | simple base/attribute forms; string fallback | not a resolver | compilation-local gate |

No repository evidence was found that the general DI/service or serialization extractors publish `SemanticEdge` framework identities. They remain heuristic rendering concerns for this investigation.

## 5. Authority-bearing textual detectors

The following textual checks participate directly in semantic authority:

1. `src/dotnet_meta/efcore.rs`: `contains(": DbContext")` gates `HasEntity` production.
2. `src/dotnet_meta/aspnet.rs`: controller attributes or `contains(": ControllerBase")` / `contains(": Controller")` gate route and action relationships.
3. `src/dotnet_meta/semantic.rs`: `contains(": Profile")` gates AutoMapper mapping relationships.
4. `src/dotnet_meta/semantic.rs`: `contains(": Hub<")` / `contains(": Hub")` gates SignalR hub-method relationships.
5. `src/dotnet_meta/testing.rs`: short textual attribute recognition inside a structurally selected attribute list gates the `TestClass` declaration.

The first four are base-type resolution concerns. MSTest is a related attribute-resolution concern but is not a consumer of the proposed base-type boundary.

Textual checks in `detect.rs`, `general.rs`, `serialization.rs`, and marker construction are not automatically semantic authority. The detector may still cause inconsistent work or marker output, but candidate detection does not by itself assert a framework identity.

## 6. Existing structured C# resolution machinery

### 6.1 Parser-owned captures

`CS_SEMANTIC_QUERY` captures declaration identifiers and every base-list `type`. The compiler associates captures with the nearest enclosing type span. For classes, the first base entry becomes `CoreOp::BaseTypeRef`; later entries are initially projected as interface-like entries under the existing canonical convention.

Input: capture name, exact source span, raw written text, enclosing declaration span.  
Output: declaration names and written base references.  
Authority: source syntax only; no external identity claim.

### 6.2 Canonical `BaseTypeRef` and durable `HasBaseType`

`project_inheritance` preserves unresolved external bases as:

```text
builtin/Class/<owner> --HasBaseType--> builtin/TypeRef/<written text>
```

This is an intentional uncertainty boundary. It preserves source truth and never presents an external written name as resolved.

### 6.3 Same-file forward alias resolution

`resolve_forward_aliases` maps a `BaseTypeRef` to canonical `Extends` or `Implements` only when one same-compilation class or interface declaration uniquely matches the written simple name. Its “alias” is an internal canonical declaration ID, not a C# `using X = ...` alias.

It does not resolve namespaces, imports, external framework types, project references, assemblies, or using aliases.

### 6.4 Request-local C# `Catalog`

The query layer reparses current cached C# source snapshots and builds:

- declarations: kind, owner file, namespace/nested path, terminal name, arity, source;
- base and constructor references: owner file/name, namespace, nesting, written spelling, parsed shape, and normal using namespaces;
- indexed lookup by declaration path/arity and reference owner.

It can resolve admitted source declarations by qualified path, namespace/nesting candidates, ordinary imports, and generic arity. It supports class/interface distinction and source-declared indirect ancestry for ASP.NET. It does not represent using aliases. It returns `Option`, so callers cannot distinguish ambiguous, unsupported, malformed, and unresolved outcomes.

The catalog also contains ASP.NET-specific functions. This proves useful machinery exists but also demonstrates that mechanics and framework meaning are not yet fully separated.

### 6.5 `EfCoreCatalog`

`EfCoreCatalog` consumes the request-local catalog but implements its own `is_ef_framework_type`. It recognizes exact/global-qualified EF names or an unqualified name when any captured normal using equals `Microsoft.EntityFrameworkCore`. It does not resolve namespace aliases or type aliases. Its imported-framework check does not first prove the absence of all local or same-namespace shadow declarations.

It resolves source entity/configuration references through the catalog and uses project-root discovery for assembly-configuration scoping. It is an exact query projection, not a reusable neutral external-type resolver.

### 6.6 ASP.NET request-local refinement

ASP.NET ancestry first attempts source-declaration resolution, follows uniquely selected class bases, detects cycles, and only then accepts a fully qualified or imported MVC framework terminal. Existing tests cover local `ControllerBase` shadowing, ambiguous same-name bases, cycles, namespace-qualified targets, multi-hop ancestry, and lifecycle retraction.

This is the strongest current framework-resolution path, but it remains framework-specific and request-local.

## 7. Resolver capability matrix

| Capability | Canonical `BaseTypeRef` | Same-file forward pass | Request-local C# catalog | EF helper | Campaign A structured classifier |
|---|---:|---:|---:|---:|---:|
| Preserve written reference | yes | input only | yes | yes | yes |
| Fully qualified syntax | preserved | no external resolution | source declarations | EF external identity | EF external identity |
| `global::` | preserved | no | shape parser accepts alias-qualified syntax but has no explicit neutral result contract | stripped for EF match | stripped for exact EF match |
| Normal using | captured generically as import op, not bound to base | no | yes, flattened namespace candidates | yes for EF | no |
| Namespace alias | preserved in raw syntax | no | no alias table | no | no |
| Type alias | preserved in raw syntax | no | no alias table | no | no |
| Local/same-namespace shadow | neutral | unique same-file simple declaration only | source candidate lookup | not fully guarded for imported external EF terminal | unsupported simple name avoids risk |
| Competing imports | neutral | no | multiple source targets may fail closed, but failure kind is not explicit | external import recognition can bypass a complete collision model | unsupported |
| Namespace nesting | neutral | no | yes for source declarations | inherited from context for source entities | owner association only |
| Nested types | neutral | no | path model supports nesting for source declarations | source resolution only | unsupported as framework terminal |
| Generic arity | written | limited canonical names | yes for source declarations | EF configuration interface and source types | DbContext arity zero only |
| Partial types | separate declarations | no merge | duplicate declarations generally become ambiguous; no partial-symbol merge | no complete partial model | each captured declaration independent |
| Indirect inheritance | neutral chain | local canonical only | ASP.NET-specific traversal | no shared external traversal | unsupported |
| Cross-file source symbols | durable neutral only | no | yes over admitted snapshots | yes | no |
| Referenced-project symbols | no | no | only source files admitted into scope | no compiler reference graph | no |
| Assembly/package symbols | no | no | framework terminals are bespoke constants | EF constants only | EF constant only |
| Explicit failure state | neutral fact | absence | no (`Option`) | no (`bool`/`Option`) | false/no registration |
| Cache/lifecycle | owner-file durable | compilation-local | request-local cache invalidated after hydration publication | request-local derived cache | owner-file durable producer generation |

## 8. Framework spelling-and-ambiguity matrices

### 8.1 EF Core `DbContext`

| Form | Current durable role | Legacy durable `HasEntity` | Request-local EF catalog | Classification |
|---|---|---|---|---|
| `using Microsoft.EntityFrameworkCore; class A : DbContext` | absent from the narrow structured role unless another EF edge registers the identity | recognized textually; local shadow not ruled out | recognized from using; shadow safety incomplete | HEURISTIC_ONLY for durable framework identity |
| `class A : Microsoft.EntityFrameworkCore.DbContext` | PROVEN | legacy textual gate misses, but structured authorized analyzer can emit relationships | PROVEN | PROVEN in current Campaign A envelope |
| `class A : global::Microsoft.EntityFrameworkCore.DbContext` | PROVEN | structured authorized analyzer can emit relationships | PROVEN | PROVEN in current Campaign A envelope |
| namespace alias `EfCore.DbContext` | absent | missed | missed | UNSUPPORTED |
| type alias `MyContextBase` | absent | missed | missed | UNSUPPORTED |
| local `class DbContext {}; class A : DbContext` | absent without relationship carrier | legacy analyzer may assert EF relationships when `DbSet` text is present | imported EF check can accept the simple spelling if an EF using also exists | AMBIGUOUS / authority-defect risk; must fail closed |
| competing imports + `DbContext` | absent | textual analyzer may accept | no complete external collision result | AMBIGUOUS |
| indirect inheritance | absent for derived class | missed unless derived text names `DbContext` | not a shared EF ancestry resolver | UNSUPPORTED |

### 8.2 ASP.NET controller bases

| Form | Durable analyzer | Request-local refinement | Classification |
|---|---|---|---|
| MVC using + `ControllerBase` | recognized textually | supported with source-shadow check in covered cases | PROVEN request-locally; durable producer remains heuristic |
| `Microsoft.AspNetCore.Mvc.ControllerBase` | missed by `: ControllerBase` substring | supported | contract divergence in durable/marker path |
| `global::Microsoft.AspNetCore.Mvc.ControllerBase` | missed | supported | contract divergence in durable/marker path |
| namespace/type alias | missed | missed | UNSUPPORTED |
| local `ControllerBase` shadow | textual durable analyzer may produce edges | request-local authority rejects in covered case | durable AUTHORITY DEFECT RISK, query layer mitigates selected surfaces |
| indirect unique source ancestry | textual analyzer misses derived type | request-local supported, including multi-hop | PROVEN request-locally |
| ambiguous/cyclic ancestry | no model | fails closed | AMBIGUOUS / UNRESOLVED |

### 8.3 AutoMapper `Profile`

| Form | Current result | Classification |
|---|---|---|
| `class A : Profile` | markers and `MapsFrom`/`MapsTo` when mappings occur | HEURISTIC_ONLY as type authority |
| `class A : AutoMapper.Profile` | no profile classification | UNRESOLVED by current classifier |
| `class A : global::AutoMapper.Profile` | no profile classification | UNRESOLVED |
| alias-qualified or type alias | no classification | UNSUPPORTED |
| local `class Profile {}; class A : Profile` | can produce AutoMapper semantic edges if `CreateMap` text occurs | AUTHORITY DEFECT RISK |
| indirect inheritance | no classification | UNSUPPORTED |

### 8.4 SignalR `Hub`

| Form | Current result | Classification |
|---|---|---|
| `class A : Hub` / `Hub<T>` | markers and `HubMethodTargets` | HEURISTIC_ONLY as type authority |
| `Microsoft.AspNetCore.SignalR.Hub` / qualified generic | missed | UNRESOLVED |
| global-qualified | missed | UNRESOLVED |
| aliases | missed | UNSUPPORTED |
| local `Hub` shadow | can produce SignalR semantic edges | AUTHORITY DEFECT RISK |
| indirect inheritance | missed | UNSUPPORTED |

### 8.5 MSTest attributes

`[TestClass]` is structurally confined to an attribute list but the attribute type itself is recognized by short written text. Qualified, aliased, and locally shadowed attribute identities are not resolved. This is material risk, but it is an attribute-resolution problem and not evidence that the base-type resolver should absorb attribute semantics.

## 9. Proven equivalent-spelling divergences

Static production conditions prove the following path divergences:

1. `Profile` versus `AutoMapper.Profile`: only the simple form satisfies `contains(": Profile")`; semantic mapping edges can differ. This affects durable index state and query visibility. **AUTHORITY DEFECT RISK.**
2. `Hub` versus `Microsoft.AspNetCore.SignalR.Hub`: only the simple form satisfies the hub checks; hub-method semantic edges can differ. This affects durable index state and query visibility. **AUTHORITY DEFECT RISK.**
3. `ControllerBase` versus `Microsoft.AspNetCore.Mvc.ControllerBase`: the durable analyzer recognizes only the simple textual form, while request-local refinement explicitly recognizes the qualified form. Query-time refinement can compensate on selected queries, but marker output and stored-edge paths diverge. **CONTRACT GAP.**
4. Simple imported `DbContext` versus fully qualified `DbContext`: different authorities and production paths currently recognize them. The narrow Campaign A slice proves full/global qualification, while the legacy analyzer and request-local catalog recognize simple imported forms. **CONTRACT GAP with shadow-related authority risk.**

Different internal paths are not defects by themselves. The problem is that these paths can assert or omit different semantic propositions under spellings that C# may bind to the same type.

## 10. Authority-risk classification

| Finding | Rank | Reason |
|---|---|---|
| AutoMapper and SignalR base spelling gates durable semantic relationships | AUTHORITY DEFECT RISK | local shadows can be misclassified and qualified equivalents are omitted |
| ASP.NET durable analyzer versus request-local exact ancestry | CONTRACT GAP | request-local surface is stronger, but durable and rendering paths still use weaker evidence |
| EF imported simple-name helper without complete shadow/collision proof | AUTHORITY DEFECT RISK | presence of a using is not proof that the simple name binds to EF |
| Exact/global-qualified Campaign A `DbContext` role | NO ISSUE inside declared envelope | parser-owned syntax plus exact supported external identity; unsupported forms fail closed |
| General DI/service and serialization marker scans | HEURISTIC LIMITATION ONLY | inspected paths render markers and do not publish semantic identities/relationships |
| .NET file detection spelling differences | HEURISTIC LIMITATION ONLY unless used as an exclusive gate | detection is candidate/applicability logic; the prior Campaign A gate incident shows it must not suppress independently proven authority |

## 11. Shared resolver necessity analysis

The boundary is required because:

- multiple authority-bearing consumers repeat the same qualification problem;
- the parser already produces neutral structured base-reference evidence;
- existing request-local resolution proves the mechanics are reusable;
- separate framework helpers currently disagree on imports, qualification, shadowing, and lifecycle;
- leaving each consumer to add one more string variant perpetuates inconsistent authority.

The existing request-local catalog is not sufficient as-is because:

- it rebuilds by reparsing cached source snapshots at query time;
- it returns `Option`, erasing why resolution failed;
- it does not model using aliases;
- it mixes ASP.NET meaning into C# mechanics;
- its external framework recognition is bespoke;
- its dependency and cache model is request-local, not a durable producer contract.

The appropriate response is not a generic resolver framework. It is one small C#-specific type-reference value and one explicit resolution result, first used only for exact external identity at Level 1.

## 12. Proposed minimum resolver contract

Repository naming should be finalized during implementation design, but the minimum semantics are:

```rust
enum CSharpTypeReferenceResolution {
    Resolved(CSharpTypeIdentity),
    Ambiguous,
    Unsupported,
    Unresolved,
}
```

Minimum neutral input:

- owner declaration identity and source span;
- parser-owned type-reference span and parsed shape;
- exact written form;
- source owner/provenance;
- resolution evidence available for the selected support level.

Minimum resolved identity for Level 1:

- namespace segments;
- nested type path/terminal name;
- generic arity;
- external-known versus source-declared identity kind;
- resolution basis, such as `FullyQualified` or `GlobalQualified`;
- source provenance of the reference.

The neutral identity must not contain `DbContext`, `Controller`, `Profile`, `Hub`, or any framework role enum. Framework code compares a resolved neutral identity with its supported framework identity constant.

`Unsupported` means the syntax category is intentionally outside the implemented envelope, such as a using alias in the first slice. `Unresolved` means the syntax is supported but no target can be proven. `Ambiguous` means supported evidence admits more than one target. These states must not authorize a framework fact.

## 13. Supported authority envelope

### First slice

Support only parser-owned class first-base references whose shape is:

- non-generic;
- exactly namespace-qualified `Microsoft.EntityFrameworkCore.DbContext`; or
- the same name prefixed by `global::`.

This is Level 1. It is file-local, deterministic, provider-neutral, and has no cross-file invalidation dependency. It is sufficient to resume Campaign A as the first proving consumer.

### Candidate later levels

Level 2, unqualified name plus ordinary using, should not be called resolved until the resolver can rule out applicable same-namespace/source shadowing and competing imports from the complete admitted authority set. The current request-local catalog is close but its result type and external-terminal logic need strengthening.

Levels 3 and 4 require authoritative namespace-alias and type-alias capture with lexical scope. No such reusable C# alias table exists today.

Levels 5 and 6 may reuse declaration path/arity and dependency machinery after explicit lifecycle design. Level 7 requires separately approved project/reference or assembly authority and is not justified now.

## 14. Unsupported/fail-closed envelope

The first shared boundary must return `Unsupported` or otherwise produce no resolution authority for:

- unqualified names, even with a normal using;
- namespace aliases;
- type aliases;
- indirect inheritance;
- generic framework bases;
- malformed or error-node references;
- partial-symbol synthesis;
- referenced-project-only symbols;
- assembly/package-only symbols other than explicitly supported fully qualified external constants;
- attribute identity;
- any case requiring provider-supplied semantic truth.

Ambiguous and unresolved cases publish no framework role or relationship. The neutral written `HasBaseType` fact remains available.

## 15. Resolver ownership boundary

The boundary belongs in the C# language semantic layer, adjacent to parser-owned C# type-reference production—not in generic cross-language core and not inside `dotnet_meta`.

The owner must:

- consume parser-owned C# syntax once;
- preserve written reference evidence;
- expose neutral identity and explicit failure states;
- remain unaware of framework roles;
- be reusable by durable producers and, later, request-local refinements;
- expose dependency information appropriate to the selected resolution level.

The existing `src/ir/pipeline/core/csharp.rs`, `src/ir/semantic_projection.rs`, and query `csharp_resolution` machinery provide source material. The implementation design should choose a small language-owned module rather than move the entire request-local catalog wholesale.

## 16. Framework-consumer boundary

Framework classifiers retain semantic ownership:

```text
parser-owned C# base reference
    -> neutral C# resolution result
    -> framework comparison
    -> framework role/relationship
```

Examples:

- EF compares the result with `Microsoft.EntityFrameworkCore.DbContext`;
- AutoMapper later compares with `AutoMapper.Profile`;
- SignalR later compares with `Microsoft.AspNetCore.SignalR.Hub`;
- ASP.NET later compares with the supported MVC controller identities.

The shared resolver must not maintain a registry of framework roles or invoke framework analyzers. A small external type identity constant in each framework module is sufficient for the first migrations.

## 17. Lifecycle and cache dependencies

| Support level | Dependencies | Validity / invalidation requirement |
|---|---|---|
| Level 1 exact/global qualification | owner file bytes, parser grammar/query generation, C# semantic producer generation | normal owner-file replacement, refresh, deletion, and producer compatibility |
| Level 2 ordinary using | owner file plus complete declarations/import collisions applicable to its lexical/project scope | must invalidate when any candidate shadow/import target is added, removed, renamed, or changes namespace/kind |
| Alias resolution | owner file lexical alias declarations and any referenced source target | alias edit and target authority changes invalidate result |
| Cross-file source resolution | complete admitted declaration manifest and scope policy | generation/fingerprint over relevant declarations; refresh and root changes revoke currentness |
| Indirect inheritance | all nodes/edges in the proof chain | any chain member or resolution dependency invalidates derived framework role |
| Project/assembly resolution | project reference graph, target frameworks, package/assembly identity | separately designed compatibility and invalidation contract required |

The current request-local `PreparationContext` caches one C# catalog per request and invalidates it whenever hydration publishes new candidates. That is correct for its current ephemeral role but is not warm durable reuse across requests.

For Level 1, no new global cache is needed. Resolution can be computed once from the existing capture during compilation and owned by the file's semantic projection. For later cross-file levels, a capability fingerprint or dependency manifest similar in principle to exact C# call authority is preferable to per-query whole-workspace rebuilding, but that architecture is not authorized here.

## 18. Performance analysis

The proposed first slice is O(B) over base references already emitted by the single C# parse, with constant-time segment comparison per reference. It adds no source reread, provider call, whole-workspace scan, or declaration comparison. Warm queries reuse the durable owner-file projection.

The existing request-local catalog is O(F + D + R) to read/parse admitted C# snapshots and build declaration/reference indexes, then indexed lookup is approximately O(number of candidate paths plus collisions). It is shared within one batch/request but rebuilt across requests.

Any later source-declaration resolver should build static indexes once per compatible authority generation:

- declaration lookup by namespace/nesting/name/arity;
- import/alias evidence by lexical owner;
- reverse dependency keys for invalidation.

It must avoid whole-workspace scans per type reference, N×N comparisons, provider semantic authority, and global reconciliation. Static machinery must be generation-owned and reused.

## 19. Campaign A resume decision

**Decision: A-RESUME-2.**

Existing machinery is close but no existing boundary has the correct reusable ownership and explicit failure contract. Campaign A should earn a narrow shared C# resolver strengthening with only Level-1 fully/global-qualified external identities. `DbContext` is the sole consumer in that slice.

The prior DbContext-specific predicate demonstrates the correct authority envelope, but it should move behind or consume the neutral shared result rather than remain the permanent place where C# qualification mechanics live.

Campaign A must not add ordinary-using support until a negative local-shadow and competing-import contract exists. It must not absorb ASP.NET, SignalR, AutoMapper, or attribute migration.

## 20. Other-consumer migration classification

| Consumer | Classification | Reason |
|---|---|---|
| EF Core `DbContext` role and its authorization of EF relationship extraction | MIGRATE NOW WITH A | proving consumer and original contract gap |
| Legacy EF request-local external-type check | MIGRATE LATER WITH OWN RED | broader Level-2/import and cross-file behavior; separate lifecycle |
| ASP.NET controller bases | MIGRATE LATER WITH OWN RED | already has strong request-local ancestry behavior that must be preserved while durable authority is corrected |
| AutoMapper `Profile` | MIGRATE LATER WITH OWN RED | durable semantic mappings and clear qualified/shadow risks |
| SignalR `Hub` | MIGRATE LATER WITH OWN RED | durable semantic mappings and generic base shape requirements |
| MSTest attributes | MORE EVIDENCE REQUIRED | attribute resolution is a distinct syntax/authority boundary |
| General DI/service, serialization, validation, logging, caching markers | TEXTUAL USE IS NON-AUTHORITATIVE — NO MIGRATION NEEDED | inspected implementation emits marker output only |
| .NET applicability detector | MIGRATE LATER only as needed | should admit proven structured consumers but is not the semantic resolver |

## 21. Proposed architectural invariants

These invariants are warranted by repository evidence but must not be added to authoritative production documentation until implementation architecture is approved.

### CSHARP-RESOLUTION-001 — Type spelling is not semantic type identity

A framework classifier may not treat written C# type spelling as durable semantic identity when the supported C# resolution boundary can distinguish resolution, ambiguity, unsupported syntax, or absence.

### CSHARP-RESOLUTION-002 — Shared resolution, specialized meaning

The C# semantic layer owns type-reference mechanics. Framework producers own the meaning of resolved identities. Neutral resolution code never emits framework roles.

### CSHARP-RESOLUTION-003 — Fail closed outside the authority envelope

Unsupported, ambiguous, unresolved, or malformed references do not authorize framework identity or relationships. Their neutral source facts remain available.

### CSHARP-RESOLUTION-004 — Equivalent resolvable syntax converges

References proven to denote the same type under the supported contract produce the same framework classification regardless of supported qualification spelling.

### CSHARP-RESOLUTION-005 — Resolution dependencies govern validity

A derived framework classification remains reusable only while every source, declaration, scope, and producer dependency required by its resolution proof remains compatible.

## 22. Future RED matrix

### Campaign A REDs

| Case | First implementation expectation |
|---|---|
| fully qualified EF base | resolve to neutral EF type identity; emit durable DbContext role |
| `global::` qualified EF base | resolve to the same identity and role |
| zero `DbSet<T>` payload | role remains present; no `HasEntity` edge |
| local short-name shadow | unsupported short form; no DbContext role |
| ordinary using + unqualified base | remain unsupported in first shared slice |
| namespace alias | remain unsupported |
| type alias | remain unsupported |
| indirect inheritance | remain unsupported |

The smallest new architectural RED before production implementation is a unit-level or tracked integration assertion that the neutral resolver returns the same `Resolved` external identity for full and global qualification while returning `Unsupported` for the short imported and alias forms. Existing Campaign A lifecycle coverage should remain unchanged and prove consumption.

### Later resolver/framework REDs

- Level 2: ordinary using resolves only when local/same-namespace shadows and competing imports are absent; each negative control must fail closed.
- ASP.NET: qualified, global, and supported imported controller bases converge without regressing multi-hop ancestry or local-shadow rejection.
- AutoMapper: simple imported and qualified `Profile` converge; local `Profile` never authors mapping semantics.
- SignalR: simple imported, qualified, and generic `Hub<T>` converge; local `Hub` never authors hub semantics.
- Alias support: namespace alias and type alias each require independent scope, shadow, retarget, refresh, and deletion REDs.
- Attribute support: qualified/aliased MSTest or ASP.NET attributes require a separate attribute-reference contract.

## 23. Explicit non-goals

- Roslyn or compiler-grade whole-solution binding.
- Resolving every C# symbol or expression.
- Project/assembly/package metadata authority.
- Provider-authored type identity.
- Repository-wide classifier migration in Campaign A.
- Changing neutral `HasBaseType` storage.
- Replacing exact C# call authority.
- Changing provider lifecycle, readiness, indexing ownership, Campaign C, or Campaign B.
- Treating markers as durable semantic identities.
- Inferring framework meaning in shared C# code.
- Adding a generic resolver registry or cross-language framework.

## 24. Runtime evidence still required, if any

No runtime evidence is required for the architecture verdict. The decisive facts are explicit production branches and type boundaries:

- semantic-edge producers are gated by textual base substrings;
- qualified equivalents do not satisfy those predicates;
- the request-local catalog has the documented fields and `Option` result;
- alias representation is absent from that catalog;
- lifecycle ownership is request-local for the catalog and owner-file durable for the Campaign A role.

Runtime RED/GREEN evidence is required only after a production slice is approved. No Cargo, build, test, Clippy, binary, MCP live command, or provider operation was run for this investigation.

## 25. Final recommendation

Approve a small design/implementation slice that extracts a neutral C# Level-1 type-reference resolver from the already-proven parser-capture path. Its explicit result distinguishes resolved, ambiguous, unsupported, and unresolved evidence. Initially it recognizes only fully namespace-qualified and `global::`-qualified external type identities. EF Core `DbContext` consumes it; no other framework migrates in the slice.

After Campaign A proves the boundary and lifecycle, migrate AutoMapper, SignalR, and ASP.NET independently, each with its own RED and negative shadow controls. Design Level 2 ordinary-import resolution separately because it introduces declaration-universe dependencies. Treat attribute resolution as a distinct future investigation.

WIDER-PATTERN:
PROVEN — MULTIPLE AUTHORITY-BEARING C# CLASSIFIERS INDEPENDENTLY REINTERPRET TYPE SPELLING

SHARED-BOUNDARY:
REQUIRED

CAMPAIGN-A:
A-RESUME-2 — NARROW SHARED RESOLVER STRENGTHENING REQUIRED; DBCONTEXT SHOULD BE FIRST PROVING CONSUMER

Campaign A production implementation authorized: NO
Campaign B production implementation authorized: NO
Repository-wide classifier migration authorized: NO

1. **Smallest next architecture/implementation slice:** approve and implement the neutral C# Level-1 base-type resolution result for fully qualified and `global::`-qualified external identities, with explicit failure states and owner-file lifecycle.
2. **Consumers in that slice:** EF Core `DbContext` role classification and the authorization handoff to its existing relationship extractor only.
3. **Consumers explicitly deferred:** request-local EF Level-2/import resolution, ASP.NET controllers, AutoMapper profiles, SignalR hubs, MSTest/framework attributes, general DI/service markers, and every referenced-project/assembly or indirect-inheritance case.
4. **Smallest future RED before production:** full and `global::` EF base references resolve to one neutral external identity and produce the same durable role, while unqualified imported, namespace-alias, type-alias, local-shadow, and indirect forms produce no authority in the first slice.
