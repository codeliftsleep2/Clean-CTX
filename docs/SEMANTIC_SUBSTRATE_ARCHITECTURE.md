# Clean-CTX Semantic Substrate Architecture

**Status:** Accepted architectural decision
**Decision date:** 2026-09-04
**Supersedes:** Ad-hoc assumptions about how semantic entities and relationships are represented, identified, and queried.
**Companion documents:** `docs/ARCHITECTURAL_INVARIANTS.md` (invariant catalog; the `SEM-*` invariants defined here are staged for adoption into that catalog by the standard process), `docs/plans/SEMANTIC-RELATIONSHIP-IMPLEMENTATION-PLAN.md` (historical design record).

---

## 1. Status and scope

This document records an **accepted architectural decision**, not a proposal. Its statements use normative language: **MUST** denotes a binding requirement, **SHOULD** denotes a strong default that requires explicit justification to deviate from, and **MAY** denotes a permitted option.

It governs:

- the semantic substrate types (`EntityRef`, `SemanticEdge`, `SemanticRelation` in `src/layers/meta/semantic.rs`);
- semantic projection by framework meta-layers (`extract_semantic_edges` / `extract_semantic_edges_paired`);
- `WorkspaceIndex` (`src/workspace/index.rs`);
- the `workspace_query` MCP surface (`src/mcp/tool_handlers/query.rs`);
- all future consumers, extractors, and query capabilities built on semantic facts.

It is intended to be **referenced by future implementation and design work**. Implementation MUST conform to this document unless this decision is explicitly superseded by a newer accepted decision. Where this document and older ad-hoc comments disagree, this document governs.

Out of scope: the Phi text-marker projection (the compressed-output surface), IR/wire formats, CBM integration, and MCP envelope mechanics. Those are governed by their own documents and invariants.

---

## 2. Architectural model

The semantic data path is:

```text
Source Code
    |
    v
Language / IR processing        (syntax -> type-root captures; language layers)
    |
    v
Framework Meta-Layers           (understand technology)
    |
    v
Common Semantic Model           (EntityRef / SemanticEdge / SemanticRelation)
    |
    v
WorkspaceIndex                  (understands the graph)
    |
    v
Generic Workspace Queries       (workspace_query)
    |
    v
Semantic composition            (callers compose facts into patterns)
    |
    v
AI Agent
```

Responsibilities:

- **Language / IR processing** produces type-root captures (`class.root`, `interface.root`, `struct.root`, `enum.root`, `trait.root`, `record.root`, `impl.root`) with canonical per-class source spans (invariant C-22). It is framework-blind.
- **Framework Meta-Layers** translate framework-specific syntax and conventions into (a) Phi text markers for compressed output and (b) `SemanticEdge` facts for the graph. Recognition of a construct is framework-specific; expression of the resulting fact is not.
- **Common Semantic Model** represents reusable architectural relationships in a framework-agnostic form: typed entities and typed, directed edges with layer provenance.
- **WorkspaceIndex** stores, indexes, deduplicates, and traverses the graph generically.
- **Generic Workspace Queries** expose read-only graph capabilities over the index.
- **Semantic composition** is performed by callers and agents: higher-level architectural patterns are derived by composing facts, not stored as facts.

### The critical boundary

**Meta-layers understand technology.** They know Angular, NgRx, .NET, EF, Spring, language/framework syntax, configuration files, and framework conventions. A meta-layer is the only place where a framework's meaning is assigned.

**The semantic substrate understands relationships.** It represents reusable semantic facts: declaration relationships, inheritance, implementation, binding, dependency/consumption, and structural relationships. It does not know what a `DbContext` or a provider array is.

**WorkspaceIndex understands the graph.** It MUST NOT understand EF, Angular, Spring, DI containers, controllers, DbContexts, components, or repositories. It MUST NOT acquire `if framework == ...` logic. It MAY understand generic relation classifications where required for generic traversal policy (see section 15), but never framework semantics.

**Query APIs expose graph capabilities.** `workspace_query` MUST NOT become a framework-specific reasoning engine. It exposes identity lookup, adjacency, filtering, and traversal; interpretation of what those mean architecturally belongs to callers and agents.

This boundary is the decision's core: framework-specific knowledge remains in meta-layers; the substrate grows only in generic, framework-independent terms.

---

## 3. Identity and namespace architecture

Semantic identity is `(domain, entity_type, name)`. Provenance (`file`) is not part of identity. The established namespace model ("Model C") is authoritative:

### Declaration identities

Owned by the `builtin` domain. A declaration identity states: "this source-language construct exists, of this declaration kind, observed in this file."

```text
builtin/Class/ApplicationDbContext
builtin/Interface/IApplicationDbContext
builtin/Struct/Foo
builtin/Trait/Foo
```

`BuiltinMetaLayer` is the sole emitter of `builtin`-domain entities. Declaration kinds are derived from capture evidence (`capture_name`), never guessed. `BuiltinMetaLayer` registers every type-root capture of every compiled file; framework layers run first and cannot collide with it because the `builtin` domain is disjoint from framework domains by design.

### Framework-role identities

Owned by framework semantic domains. A role identity states: "this declaration participates in framework concept X, as recognized by the owning layer."

```text
dotnet/DbContext/ApplicationDbContext
angular/Component/Foo
spring/Controller/Foo
ngrx/Store/AppState
```

Role `entity_type` vocabularies (`Component`, `DbContext`, `Controller`, `Effect`, ...) are owned by the respective framework layers.

### Value identities

Framework domains also contain semantic values that are not declarations: route paths, endpoint strings, selector literals, field names, pipe names.

```text
angular/Route/<path>
spring/Endpoint/"GET /health"
ngrx/Selector/<name>
```

Value identities carry literal semantic values declared by the source (the Selector-Value discipline, `SEL-001`, applies to their names).

### Reference identities

When a framework layer encounters a **foreign declaration** (one defined in another file) whose declaration kind cannot be established locally, it MAY represent it as a role-typed reference in its **own** domain. Production precedents:

```text
ngrx/Store/AppState        (a TypeScript interface referenced by NgRx store typing)
dotnet/Entity/Customer     (a POCO class referenced by a DbContext's DbSet scan)
```

Reference identities converge with the corresponding declaration identities **by name** (through the name index and `find_entities_by_name`), never by identity mutation. The substrate MUST NOT merge, rewrite, or reconcile reference identities into declaration identities; both continue to exist, and name-based lookup is the designed convergence point.

---

## 4. Namespace rules

The following rules govern how any meta-layer constructs `EntityRef` endpoints.

### P1 - Truthful namespace

An endpoint belongs in the namespace whose semantics describe the endpoint.

- declaration nature -> `builtin/<DeclarationKind>/<Name>`;
- framework role, value, or reference nature -> `<framework>/<Role>/<Name>`.

The `SemanticEdge.layer` field records which layer asserts the edge and is independent of the endpoint domains (a layer may truthfully describe an endpoint that belongs to another layer's namespace; see section 6 and the NgRx precedent).

### P2 - Reference convention

When a foreign declaration's kind cannot be established locally, the emitting framework MAY represent it using its own role/reference vocabulary (for example `ngrx/Store/AppState`, `dotnet/Entity/Customer`). This is an accepted approximation: the reference is honest about what the layer knows, and name-based convergence preserves global coherence.

### P3 - No guessing

Never invent a declaration kind merely to make traversal or lookup convenient. In particular, do not manufacture `builtin/Interface/X` unless the source evidence establishes that X is an interface. A wrong guessed kind **splits the identity** (`builtin/Interface/Foo` versus the real `builtin/Class/Foo`) and permanently breaks name convergence for that fact. When kind is uncertain, use P2 or do not emit the fact (per-relation rules in sections 8-10).

---

## 5. Provenance semantics

`EntityRef.file` is **occurrence/reference provenance**, not a declaration-definition claim.

- `file` is excluded from `PartialEq`/`Hash`; entity identity is `(domain, entity_type, name)` regardless of which file mentioned the entity.
- Occurrence identity is `(domain, entity_type, name, file)`; one occurrence per file is registered, cross-file occurrences are preserved, and none is ever silently overwritten (`WorkspaceIndex::register_entity`).
- When an edge is inserted, endpoints lacking a file receive the canonical path of the file the edge was extracted from. Consequently, **an object on an injection edge carries the file in which the injection reference occurred, not the file where the target is defined**. This is the documented contract of `resolve_inject_type` and MUST NOT be changed.
- The definition file of a declaration appears through the declaring layer's own registration (for `builtin`, the self-referential `Defines` registration record; for framework roles, the layer's own subject endpoints).

The distinction between identity and occurrence is deliberate: it is what allows "the same entity referenced from another file" to compare equal while still tracking where each observation came from.

---

## 6. Parallel identities

The same source declaration MAY legitimately hold multiple semantic identities:

```text
builtin/Class/ShellComponent
angular/Component/ShellComponent
```

This is production behavior asserted through the real compile path (the decorated-class/NgRx production fixture in `src/tests/mcp/workspace_query.rs`), and the disjointness of `builtin` from framework domains is the documented mechanism that keeps these identities collision-free regardless of layer registration order.

Parallel identities are **not duplicates and not errors**. They are different semantic interpretations of the same underlying construct: the declaration ("this construct exists, as a class, here") and the role ("this construct is an Angular component"). The substrate MUST NOT collapse them into one identity, and consumers MUST NOT treat their coexistence as inconsistency. Convergence between them is by name, per section 3.

---

## 7. Semantic relationship architecture

A semantic relation MUST represent a meaningful relationship whose semantics are independent of the syntax used by a particular framework. A relation is admissible when its meaning can be stated as subject/object/direction without naming a framework.

### Generic architectural relationships

Relations whose meaning holds across frameworks. The foundation designates:

```text
Implements    (declaration satisfies a contract)          - section 8
Extends       (declaration inherits/derives)              - section 9
Binds         (container maps implementation to a token)  - section 10
```

Existing relations that are generic in substance but framework-named today (for example `Injects` and `Autowired`, both expressing consumer-side dependency) keep their current names and meanings (section 14).

### Framework-specific relationships

Relations whose meaning is only defined inside a framework's model - state-management wiring (`Dispatches`, `Selects`, `HandlesAction`, `ProducesAction`, `TriggersReducer`, `HasStore`, `CallsService`), routing (`RouteMapsTo`, `GuardedBy`, `ResolvedBy`), endpoints (`EndpointMapsTo`), entity-set membership (`HasEntity`), mapping (`MapsFrom`, `MapsTo`), component contracts (`HasInput`, `HasOutput`, `HasSelector`), lifecycle/configuration (`BeanProduces`, `ConfigurationProperties`, `ControllerAction`, `HubMethodTargets`). These are legitimate substrate citizens: they are typed, directed facts with layer provenance. They are simply not candidates for framework-independent reasoning.

### The uniformity rule

Do not force genuinely different semantics into a generic relation merely for uniformity, and do not fragment one generic fact into per-framework relations. When a new fact class appears, first test whether ONE relation with framework-independent meaning can carry it across all frameworks; if it cannot, it is framework-specific by definition and belongs beside the existing framework-specific relations.

---

## 8. `Implements`

`Implements(A, B)` means: **A is a declaration declared to satisfy B's contract.**

- Direction: `implementation -> contract`.
- Subject: the implementing/satisfying declaration.
- Object: the implemented contract declaration (interface, trait, abstract contract).
- Endpoints: `builtin` declaration -> `builtin` declaration, **only when both declaration kinds are established by source syntax**. The relation records a fact about declarations; a framework-role identity (for example `dotnet/DbContext/X`) MUST NOT be the subject of a declaration-level `Implements` edge.
- Emitter: the layer that observes the syntax (it knows its own capture kind; object kind follows the language rules below).
- Cross-domain emission: mechanically legal (cross-domain edges are supported), but not semantically indicated for this relation.
- Multiple emitters: two layers observing the same syntax MUST NOT both emit the same edge; the edge identity dedups only on `(relation, subject identity, object identity)`, so divergent endpoints from two layers would produce two facts. The emitter is the layer whose syntax recognition produces the fact.

### Language certainty rules

The relation is emitted only where the syntax establishes both endpoint kinds (P3). Where it does not, the fact is deferred, never guessed.

| Language / construct | Certainty | Rule |
|---|---|---|
| C# `class X : Base, I1, I2` | Certain | Exactly one base class, first in the base list; remaining entries are interfaces (language rule, not convention). Emit `Extends(X, Base)` and `Implements(X, I1)`, `Implements(X, I2)` with `builtin` endpoints. For `struct` captures all base-list entries are interfaces. |
| C# `record X : Y` | Not certain | Y may be a class or an interface. Do not emit. |
| Java `class X extends B implements C, D` | Certain | `extends` target is a class; `implements` targets are interfaces. Emit both with `builtin` endpoints. |
| Java `interface X extends B, C` | Certain | Interface inheritance. Emit `Extends(X, B)` with Interface-kind endpoints. |
| TypeScript `class X extends B` | Certain | Class-extends target must be class-like. Emit `Extends` with `builtin/Class/B`. |
| TypeScript `class X implements C` | **Not certain** | C may be an interface, class, or type alias. Do NOT manufacture `builtin/Interface/C`. Defer the fact (or, if a consumer later proves the need, project it with a P2 reference endpoint that does not claim a declaration kind). |
| TypeScript `interface X extends B, C` | Certain | Interface inheritance. Emit `Extends` with Interface-kind endpoints. |
| Rust `impl Trait for Struct` | Certain | Both names and kinds are local to one `impl.root` capture. Emit-capable; projection is future work (no Rust meta-layer exists; `impl.root` captures are currently not registered by `BuiltinMetaLayer`). |
| Rust `trait A: B` (supertrait) | Different semantics | A bound, not inheritance. MUST NOT be mapped to `Extends`. |

Representative C# example:

```text
class ApplicationDbContext : DbContext, IApplicationDbContext { ... }

Extends(ApplicationDbContext, DbContext)             // inheritance
Implements(ApplicationDbContext, IApplicationDbContext)  // contract satisfaction
```

---

## 9. `Extends`

`Extends(A, B)` means: **A is a declaration that inherits/derives from B.**

- Direction: `derived -> base`.
- Subject: the deriving declaration. Object: the base declaration.
- Endpoints: `builtin` declaration -> `builtin` declaration when declaration kinds are established (per the table in section 8).
- `Extends` is for inheritance only. Do not use it for semantically different constructs that merely resemble inheritance syntactically: Rust supertraits (a bound), TypeScript structural compatibility, interface *implementation* (that is `Implements`), or framework-specific "extends" vocabulary (for example an Angular module importing another module is `ImportsModule`, not `Extends`).

---

## 10. `Binds`

`Binds(implementation, token)` means: **within an injection/binding system, the subject is registered or resolvable as the object's abstraction/token.**

- Direction: `implementation -> token`, mirroring `Implements` (provider-side subject, contract-side object). Reverse lookup from a token therefore yields its providers, the same shape as reverse-`Implements`.
- Subject: the implementation, as a role/reference identity in the **emitting layer's domain**. A binding is usually observed at a registration site that references the implementation cross-file, so its declaration kind is not locally certain (P3); a `builtin` endpoint MUST NOT be minted from the binding site.
- Object: the abstraction **token**, as a role/reference identity in the emitting layer's domain. Registration syntax does not establish that the token is an interface (self-binding `AddScoped<Impl>()` is legal), so `builtin/Interface/...` MUST NOT be manufactured from registration syntax either. The token converges with its declaration identity by name.
- Emitters: each framework layer emits `Binds` within its own domain for its own binding system.

`Binds` is distinct from its neighbors by design:

| Relation | Fact class | Direction |
|---|---|---|
| `Implements` | language-level contract relationship (declared in source syntax) | implementation -> contract |
| `Binds` | container/runtime provision relationship (asserted by a binding system) | implementation -> token |
| `Injects` / `Autowired` | consumer-side dependency relationship (someone depends on the abstraction) | consumer -> dependency |

Neither of the following implications holds: an implementation can satisfy a contract without ever being bound (unregistered service), and a token can be bound to an implementation without a syntactic implements relationship (`useExisting` aliasing, non-interface tokens, self-bindings). They are separate, composable facts.

`Binds` deliberately does NOT encode: lifetime (scoped/singleton/transient), qualifiers (`@Qualifier("...")`), multi-provider semantics, alias semantics (`useExisting`), or any other framework-specific registration configuration, unless and until such facts are separately represented. The edge records the structural mapping only. Multiple registrations of the same pair collapse under edge deduplication; this is an accepted limitation of the structural-fact model.

The concept is generic even though each framework recognizes it through different syntax: .NET `AddScoped/AddSingleton/AddTransient/AddDbContext<I, Impl>()` (a scan for these already exists in `src/dotnet_meta/general.rs`, currently projected only into Phi text), Spring `@Bean` factories and component scanning, and Angular `providers: [{provide, useClass, useExisting}]` (provider arrays are not yet extracted). One common relation represents them all; the syntax recognition stays in the meta-layers.

---

## 11. Pattern-as-composition principle

> **Architectural patterns are compositions of semantic facts, not primary semantic entities.**

This is the most consequential commitment in this document. Clean-CTX does not store patterns; it stores facts, and patterns emerge when callers and agents compose them.

The EF example. With the generic vocabulary in place, the facts are:

```text
CustomerRepository
        |
     Injects
        v
[token] IApplicationDbContext

ApplicationDbContext
        |
     Binds
        v
[token] IApplicationDbContext

builtin/Class/ApplicationDbContext
        |
     Implements
        v
builtin/Interface/IApplicationDbContext

dotnet/DbContext/ApplicationDbContext
        |
     HasEntity
        v
dotnet/Entity/Customer
```

Composing these - consumption, binding, declaration contract, and framework role, connected through name convergence - yields the architectural conclusion:

> The repository ultimately operates through an interface-bound EF DbContext over Customer.

No entity named `EFRepositoryPattern` (or `DbContextPattern`, `AngularStatePattern`, `SpringServicePattern`) is required, and none MUST be introduced. The same facts compose differently in an Angular workspace (token/provider/state) or a Spring codebase (bean/type/wiring) without any new pattern vocabulary.

Framework-specific pattern entities are explicitly rejected unless a future, concrete consumer proves they are necessary - and any such proposal would have to answer the questions this composition approach avoids: what identity a pattern instance carries, which file owns it, and how it survives recompilation.

---

## 12. WorkspaceIndex architecture

`WorkspaceIndex` (`src/workspace/index.rs`) is the framework-agnostic graph store. It provides:

- semantic entity storage with occurrence tracking;
- identity indexing (`(domain, entity_type, name)`) plus a name index for cross-domain lookup;
- forward and reverse adjacency by identity;
- deduplication at the index write boundary (edge identity `(relation, subject identity, object identity)`, first occurrence wins; entity occurrence identity adds `file`);
- file lifecycle (precise per-file removal on recompilation/deletion);
- generic traversal (whitelisted dependency BFS, whole-graph cycle detection);
- generic lookup and resolution primitives.

WorkspaceIndex MUST remain framework-agnostic. It MUST NOT acquire `if framework == ...` logic and MUST NOT understand EF, Angular, Spring, DI containers, controllers, DbContexts, components, or repositories.

It MAY understand **generic relation classifications** where required for traversal policy - the existing `DEPENDENCY_RELATIONS` whitelist (section 15) is exactly that: a relation-class distinction, not framework knowledge. New generic relations are classified for traversal policy at the index boundary, never by framework name.

No framework-specific pattern resolution belongs in the index. Resolution primitives that exist today (`resolve_inject_type`, `resolve_selector`) are relation-keyed, not framework-keyed; any future resolver must likewise be defined in terms of relations and identities.

---

## 13. Query architecture

`workspace_query` is a thin, read-only boundary over `WorkspaceIndex`. The current identity-oriented query model is retained:

- identity-oriented access is by `(domain, entity_type, name)` - the established identity API (forward/reverse adjacency, transitive dependencies) is NOT redefined as a bare-name lookup;
- the separate cross-domain name lookup (`find_entities` via `find_entities_by_name`) remains what it is today: a convenience that returns all occurrences of a name across domains and types, preserving ambiguity;
- queries compose: every operation's outputs (identities, edge endpoints) are valid inputs to other operations.

The immediate composability improvement is an optional relation filter:

```text
forward_edges(identity, relation?)
reverse_edges(identity, relation?)
```

The optional `relation` parameter is a narrowing/filtering mechanism over an existing operation's result set - not a new semantic layer, not a new identity model. It is what makes projected generic facts (`Implements`, `Binds`) practically consumable without client-side scanning of unrelated edges.

Explicitly deferred until an actual consumer requires them:

- query DSLs and Cypher-like matching;
- framework-specific query operations (for example a `resolve_ef_dbcontext`-style tool);
- arbitrary server-side multi-hop path languages;
- framework-aware reasoning inside `workspace_query`.

Generic multi-hop traversal MAY be introduced later, defined in terms of relations and identities, when a concrete consumer demonstrates the need. Convenience never justifies moving framework semantics into the query layer.

---

## 14. Existing relation vocabulary and preservation

Additive-growth principle: **existing semantic relations retain their existing meanings.** Relations are never silently renamed, merged, reinterpreted, or re-endpointed merely because the generalized architecture exposes inconsistencies. Behavioral preservation applies to the semantic graph exactly as it does to code.

Known technical debt is recorded here as future cleanup items, explicitly NOT repaired in this decision:

- **Overloaded `Defines`:** the relation currently carries three meanings - the builtin registration carrier (normalized at the index write boundary), a value-naming relation (`Pipe -> Defines -> PipeName`), and a pseudo-implementation relation (`Guard -> Defines -> Guard-kind "CanActivate"` in routing extraction, which is semantically an `Implements` fact). Future work must not add further `Defines` overloads; a safe migration of the routing usage is a later, separately approved change.
- **`Injects` vs `Autowired`:** one generic consumer-side dependency fact with two framework-named relations. Generic consumers already handle both (`resolve_inject_type`, the dependency whitelist). An equivalence mapping or consolidation is later work requiring a defined migration.
- **Reference-quality defects in existing facts:** Spring `Autowired` objects carry field-declaration-derived names rather than declared types; `HasSelector` objects store selector strings under a `Component`-typed object entity. These facts are preserved untouched; new semantics MUST NOT depend on their exact shape.
- **Dead relation variants:** `HasModel`, `HasTemplate`, `HasStyle`, `EntityRelationship`, `Extends`, `Implements`, and `Calls` are declared in `SemanticRelation` with no emitter anywhere. `Extends`/`Implements` are activated by this decision's roadmap; the others require a documented meaning before any emitter may use them (`Calls` overlaps conceptually with NgRx's `CallsService` and must be reconciled first).
- **Documentation/code drift:** historical documents may overstate which layers emit semantic edges (for example Signals). Where prose and code disagree, the code and this document govern.

---

## 15. Relation classification

`WorkspaceIndex::DEPENDENCY_RELATIONS` is a fixed whitelist of eight relations (`Injects`, `Autowired`, `ImportsModule`, `HandlesAction`, `CallsService`, `HasEntity`, `MapsFrom`, `ConfigurationProperties`) used by `transitive_dependencies` to distinguish dependency-like relations from structural/metadata relations.

Accurate characterization:

- It is **generic** and **framework-independent**: it classifies relations, never frameworks, and contains no framework names or branching.
- It is **traversal policy**, not semantic truth: it defines which forward edges the dependency BFS follows; it does not redefine what any relation means.
- It lives at the index boundary by design; relations are classified where they are traversed, not where they are emitted.

Relation metadata (a per-variant classification owned beside the enum - dependency / structural / value / binding) MAY eventually replace the hardcoded whitelist so that traversal policy follows vocabulary growth automatically. That is a desirable future refinement and is **not part of this decision**. Until then the whitelist remains unchanged; whether a new relation (for example the provision-class `Binds`) joins dependency traversal is decided per-relation at the time of its first emission, documented with the relation.

---

## 16. Architectural invariants

The following invariants are authoritative for the semantic substrate. They use fresh, non-colliding identifiers (`SEM-*`; the existing catalog occupies `WIRE`, `VALID`, `DELTA`, `ARCH`, `PIPELINE`, `C-22`, `CBM-*`, `IDENT`, `EDIT`, `MCP`, `IRPAT`, `SEL`, `IDX`). Adoption into `docs/ARCHITECTURAL_INVARIANTS.md` follows the standard catalog process and is outside this document.

**SEM-001 - Entity Identity**
Identity is `(domain, entity_type, name)`. `file` MUST NOT participate in `PartialEq`/`Hash`. Type: STRUCTURAL (currently enforced by `src/tests/layers/meta/semantic.rs`).

**SEM-002 - Occurrence Provenance**
Occurrences are distinguished by `(domain, entity_type, name, file)`; one occurrence per file per identity; occurrences are never silently overwritten. Endpoint files on an edge record where the edge was extracted, never a definition claim. Type: ENFORCED (`WorkspaceIndex::register_entity`, `add_edges`, and index tests).

**SEM-003 - Declaration Namespace Ownership**
`builtin` is the sole emitter of `builtin`-domain entities; declaration kinds are derived from capture evidence only. Type: DOCUMENTED (verified by exhaustive search; to be pinned by test when generic relations land).

**SEM-004 - Role Namespace Ownership**
Framework domains own role, value, and reference identities. Declaration-kind entity types minted by `builtin` (`Class`, `Interface`, `Struct`, `Enum`, `Trait`, `Record`) MUST NOT appear as `entity_type` values in framework domains. Type: DOCUMENTED.

**SEM-005 - Parallel Identities Are Intentional**
One source declaration MAY hold a declaration identity and any number of role identities simultaneously. Consumers and future code MUST NOT collapse, merge, or de-duplicate them by anything other than the documented name-convergence mechanism. Type: DOCUMENTED (production-path test exists).

**SEM-006 - Cross-Domain Edges Are Legal**
No index, dedup, traversal, or query code may assume `subject.domain == object.domain`. Type: STRUCTURAL today (no such assumption exists in `src/workspace/index.rs`); to be pinned by test.

**SEM-007 - Layer Provenance Independence**
`SemanticEdge.layer` records the asserting layer and is independent of both endpoint domains. Type: DOCUMENTED (NgRx angular-subject production precedent).

**SEM-008 - Reference Convergence By Name**
Reference identities converge with declaration identities only through name lookup. The substrate MUST NOT rewrite, merge, or re-namespace reference identities. Type: DOCUMENTED.

**SEM-009 - Edge Identity and Deduplication**
Edge identity is `(relation, subject identity, object identity)`; first occurrence wins; endpoint domains participate in the key. Registration-record self-`Defines` is normalized at the write boundary and never enters the graph. Type: ENFORCED (`src/tests/workspace/index.rs`).

**SEM-010 - Relation Meaning Stability and Additive Growth**
Existing relations retain their existing meanings; relations are never silently renamed, merged, or re-endpointed. Vocabulary growth is additive. New relations document subject/object/direction before first emission. Type: POLICY.

**SEM-011 - `Implements` Semantics**
`Implements(A, B)`: A is declared to satisfy B's contract; direction `implementation -> contract`; endpoints are `builtin` declarations only when both kinds are established by source syntax; kind-uncertain cases are deferred, never guessed; framework-role identities MUST NOT be subjects. Type: DOCUMENTED (contract; enforcement tests arrive with the first emitter).

**SEM-012 - `Extends` Semantics**
`Extends(A, B)`: A inherits/derives from B; direction `derived -> base`; declaration endpoints when kinds are established; reserved for inheritance only (no supertrait bounds, no structural compatibility, no module imports, no interface implementation). Type: DOCUMENTED.

**SEM-013 - `Binds` Semantics**
`Binds(implementation, token)`: within a binding system the subject is registered/resolvable as the object's token; direction `implementation -> token`; endpoints are emitting-domain role/reference identities (never guessed `builtin` kinds); lifetime/qualifier/multi-provider/alias configuration is deliberately not encoded. Type: DOCUMENTED.

**SEM-014 - No Guessed Declaration Kinds**
No declaration kind may be asserted without syntactic establishment (generalization of P3). A wrong kind splits identity and breaks name convergence. Type: POLICY (enforced per-emitter when generic extraction lands).

**SEM-015 - WorkspaceIndex Framework Independence**
The index interprets relations and identities only. Zero framework vocabulary, zero `if framework == ...` logic. Generic relation-class whitelists for traversal policy are permitted; framework semantics are not. Type: STRUCTURAL today; to be pinned by test.

**SEM-016 - Query Composability**
Every `workspace_query` operation's output MUST be valid input to another operation. Filtering parameters narrow result sets; they never introduce new semantics. Type: DOCUMENTED (to be pinned by test when the relation filter lands).

**SEM-017 - Patterns Are Compositions**
Architectural patterns are derived by composing semantic facts. First-class pattern entities are forbidden absent a proven, concrete consumer need, and any such proposal must answer identity, ownership, and recompilation-lifecycle questions first. Type: POLICY.

**SEM-018 - Relation Direction Contract**
Every relation has a documented subject/object/direction. Reverse traversal semantics follow from direction (for example, reverse-`Implements` and reverse-`Binds` yield providers of a contract/token; forward-`Injects` yields dependencies of a consumer). Type: DOCUMENTED.

---

## 17. Implementation roadmap

This roadmap is **subordinate to the architectural principles** above: no phase may violate a section 4 rule, a section 8-10 semantics definition, or a section 16 invariant for the sake of progress.

### Foundation

1. Adopt and document the semantic contract (this document; extend per-relation documentation as relations activate).
2. Add relation-filtered forward/reverse query capability (`forward_edges(identity, relation?)` / `reverse_edges(identity, relation?)`), preserving all existing call forms.
3. Add `Binds` to `SemanticRelation` with its documented meaning.
4. Project existing framework facts into the common vocabulary, smallest first: .NET base-list `Extends`/`Implements` (kind-certain cases), .NET `Binds` from the existing DI-registration scan, .NET constructor-injection consumption via the existing `Injects` relation; then Spring and Angular under the same certainty rules; Rust `impl Trait for Struct` as a later candidate.

### Later

5. Generalized traversal and path composition, defined in terms of relations and identities, when a concrete consumer justifies them.
6. Normalize duplicated/overloaded relations (the `Defines` overloads; `Injects`/`Autowired` equivalence) only when a safe migration can be defined.
7. Higher-level semantic resolution and architectural pattern discovery - resolver generalization, richer agent-facing queries - built strictly by composing the substrate capabilities above.

---

## 18. Consequences

### Positive

- **Framework-specific extraction remains isolated.** Recognition logic for Angular, NgRx, .NET/EF, and Spring stays inside meta-layers; the substrate and index never learn a framework's name.
- **Semantic facts become reusable across frameworks.** One `Binds` fact class serves .NET, Spring, and Angular binding systems; one `Implements` fact class serves C#, Java, TypeScript, and Rust.
- **Graph traversal becomes composable.** Identity-oriented operations with optional narrowing compose into arbitrarily rich caller-side queries without server-side semantics growth.
- **Architectural patterns emerge without a combinatorial pattern taxonomy.** New framework x pattern combinations require no new pattern types - only the projection of facts.
- **New frameworks project into the existing substrate.** A new meta-layer registers, recognizes its constructs, and emits facts under the namespace rules without index or query changes.
- **AI agents reason over architecture rather than reconstructing it from raw syntax.** Composition of typed facts (consume / bind / implement / role) answers architectural questions that raw-symbol search cannot.

### Negative / limitations

- **The vocabulary must be designed carefully.** A generic relation is permanent; a badly chosen meaning is worse than a missing one. Section 7's admissibility rule and sections 8-10's semantics exist because of this.
- **Ambiguous source syntax cannot always be represented as certain declaration relationships.** C# records with base lists, TypeScript `class implements`, and DI token kinds are deferred rather than guessed; some facts are consciously absent.
- **Some framework-specific detail remains outside generic relationships.** Lifetimes, qualifiers, providers, and aliases are not encoded in `Binds`; structural-fact deduplication collapses re-registrations of the same pair.
- **Existing legacy relations create technical debt.** The `Defines` overloads, the `Injects`/`Autowired` duplication, and reference-quality defects in older facts are preserved until a safe migration exists.
- **Richer path reasoning may eventually require additional query primitives.** Server-side multi-hop traversal is deliberately absent; if composition overhead proves real for consumers, a relation-parameterized traversal is the designated future extension - still framework-free.

---

## 19. Evidence and provenance

This decision is grounded in the implementation as inspected on 2026-09-04. Key sources supporting the statements above:

- `src/layers/meta/semantic.rs` - substrate types; identity model header (`(domain, entity_type, name)`, file excluded); `SemanticRelation` variant set; `SemanticEdge` provenance contract.
- `src/tests/layers/meta/semantic.rs` - `EntityRef` equality/hash semantics (file ignored; all identity fields distinguishing).
- `src/layers/meta/builtin.rs` - `BuiltinMetaLayer` purpose, declaration-kind mapping from capture names, `builtin` disjointness rationale.
- `src/layers/meta/mod.rs` - `MetaLayer` trait, `extract_semantic_edges` / `extract_semantic_edges_paired`, Angular projection scope (decorators + NgRx + routing; RxJS deferral note).
- `src/layers/registry.rs` - registered meta-layers and builtin-last registration order.
- `src/ir/pipeline.rs`, `src/ir/compiler.rs` - capture pairing (C-22), `collect_semantic_edges` dispatch, provenance attachment, `CompiledIR.semantic_edges` boundary, optional InferenceLayerPass drain.
- `src/workspace/index.rs` - storage, `add_edges` write boundary, `register_entity` occurrence model, `remove_file` lifecycle, `find_entities_by_name`, `resolve_inject_type`, `resolve_selector`, `DEPENDENCY_RELATIONS`, `transitive_dependencies`, `has_cycle`.
- `src/mcp/tool_handlers/query.rs`, `src/mcp/tool_handlers/core.rs`, `src/mcp/tool_handlers/edit.rs`, `src/mcp/tool_helpers.rs` - `workspace_query` surface and the production write lifecycle (`remove_file` + `add_edges` per compiled file).
- `src/dotnet_meta/semantic.rs`, `src/dotnet_meta/general.rs` - .NET role extraction, `HasEntity`, and the existing DI-registration scan (currently Phi-only).
- `src/spring_meta/semantic.rs`, `src/spring_meta/annotations.rs` - Spring extraction including the field-name reference defect recorded in section 14.
- `src/angular_meta/semantic.rs`, `src/angular_meta/ngrx.rs`, `src/angular_meta/routing.rs` - Angular/NgRx/routing projection; cross-domain edges (`HasStore`, `Dispatches`, `Selects`); the `Defines` overloads.
- `src/tests/workspace/index.rs`, `src/tests/mcp/workspace_query.rs` - dedup, traversal, identity, ambiguity, selector-value, and parallel-identity production-path fixtures.

Where this document cites behavior, the cited source is the authority for the current state; this document is the authority for the intended contract.