# Issue #145 Filtered Declaration Consistency Investigation

**Date:** 2026-10-18  
**Investigator:** Agent MaxHeadRoom  
**Scope:** Static repository investigation only. No Cargo command, build, test,
binary, MCP runtime session, or live verification was run.

## 1. Executive verdict

**Verdict: PUBLIC CONTRACT MISUNDERSTOOD — NO DEFECT.**

**Final recommendation: CLOSE #145 — NO DEFECT.**

The reported pair does not filter one published declaration first loosely and
then exactly. The two requests intentionally select different authority sets:

```text
find_entities(name = ContextA)
  -> every indexed entity occurrence named ContextA
     (declarations and relationship endpoints)

find_entities(
  name = ContextA,
  declarations_only = true,
  domain = dotnet,
  entity_type = DbContext
)
  -> source-proven declaration occurrences named ContextA
  -> exact identity filter dotnet / DbContext
```

A C# declaration `class ContextA : DbContext` is source-proven in the
declaration index as `builtin/Class/ContextA`. A `dotnet/DbContext/ContextA`
occurrence is a framework-semantic relationship endpoint, for example the
subject of a `HasEntity` edge. The latter is query-visible through ordinary
occurrence lookup but is deliberately not registered as declaration authority.
The tracked warm-state authority test explicitly asserts both facts and states
that source Class declarations must not be rewritten as dotnet/DbContext
declarations.

Therefore the answer to the investigation's central question is:

> **No. The filtered request is not filtering the same semantic object that the
> unfiltered request ultimately publishes. Under the current documented and
> tested contract, that is deliberate:** unfiltered lookup publishes occurrences,
> while `declarations_only` switches to the separately owned, source-proven
> declaration catalog before applying exact identity filters.

The premise needed for the proposed invariant is not established by the field
report. The unfiltered response publishes `domain`, `entity_type`, `name`, and
optional `file`; it does not publish `is_declaration=true`. Seeing a semantic
endpoint whose file is the declaration's file does not make that endpoint a
declaration occurrence. This is the intended converse of the existing rule:
relationship participation is not declaration evidence.

This verdict does not decide whether a future API should support “source
declarations classified as framework type.” That would be a new public
authority contract and an architectural design decision, not a repair to the
current declaration filter.

## 2. Exact unfiltered call path

### 2.1 Request parsing and dispatch

`handle_workspace_query` unwraps the MCP `arguments`, validates the operation,
and dispatches `type=find_entities` to
`query::entities::prepare_find_entities`. The schema requires one non-empty
`name`. It describes default `find_entities` as occurrence-oriented.

`try_prepare_find_entities` parses:

- `name` with `required_name`;
- `declarations_only`, defaulting to `false`;
- optional `domain` and `entity_type`;
- `workspaceRoot` and the shared effective `WorkspaceScope`.

For request A, `declarations_only=false` and both identity filters are absent.

### 2.2 Effective scope

`PreparationContext::scope` constructs the same authorized-root plus optional
`withinPath` view used before and after hydration. Scope admits occurrences by
their `EntityRef.file` provenance; it does not reinterpret semantic identity.

### 2.3 Initial acquisition read and hydration

The required initial read calls
`WorkspaceIndex::find_entities_by_name[_in_scope]`. It is intentionally an
occurrence lookup across all domains and entity types.

Both request forms then call the same name-based hydration boundary:

```text
context.hydrate(
    "find_entities",
    name,
    workspaceRoot,
    HydrationRequirement::LegacyEdit,
)
```

The optional C# ASP.NET base-chain hydration loop is also shared. Domain and
entity-type filters do not change this acquisition request.

### 2.4 Candidate production and canonical identity

The default C# semantic pipeline produces a self-`Defines` registration carrier
for the syntactic class as:

```text
builtin / Class / ContextA
```

`WorkspaceIndex::add_edges` recognizes a self-referential `Defines` edge,
registers it in both the general occurrence map and the separate declaration
map, and does not add the carrier to graph adjacency.

The .NET EF meta layer can independently emit a semantic edge such as:

```text
dotnet/DbContext/ContextA --HasEntity--> dotnet/Entity/EntityA
```

The index registers both endpoints in the general occurrence map. It does not
register either endpoint in the declaration map because the edge is a real
relationship, not a self-`Defines` declaration carrier.

The query-time exact EF catalog has the same identity model. It recognizes a
class as an EF `DbContext` only from structured C# declaration/base-reference
evidence plus the `Microsoft.EntityFrameworkCore` authority condition, and it
creates `dotnet/DbContext` as the subject of request-local `HasEntity` edges.
It does not rewrite the source declaration identity.

### 2.5 Refinement, filtering, deduplication, and rendering

For DbContext there is no special `find_entities` refinement step. The only
extra projection in this handler is the ASP.NET Controller path, and it is not
relevant to `dotnet/DbContext`.

The final indexed evaluator calls
`find_entities_by_name[_in_scope]`. Because `declarations_only=false`, the two
domain/type filter closures are unconditional passes. Normal
`WorkspaceIndex::register_entity` occurrence identity is:

```text
(domain, entity_type, name, file)
```

so duplicate edge participation in one file collapses while occurrences in
different files remain distinct. `serde_json::to_value` renders the stored
`EntityRef` directly. There is no later renderer that converts
`builtin/Class` into `dotnet/DbContext`.

Thus an unfiltered `dotnet/DbContext/ContextA` result was already that semantic
endpoint in the general occurrence index. It was not a neutral declaration
that rendering relabeled.

## 3. Exact filtered call path

### 3.1 Parsing, scope, and acquisition

Request B parses `declarations_only=true`, `domain=dotnet`, and
`entity_type=DbContext`. It constructs the same effective scope and performs
the same `LegacyEdit` name hydration as request A. The filters do not select a
different provider, fidelity, compilation requirement, or candidate name.

The initial read differs: it calls
`declarations_by_name[_in_workspace]`, not the general occurrence lookup. This
read is discarded, preserving the established initial-read-before-hydration
lifecycle; the same authority choice is repeated for the final answer.

### 3.2 Declaration selection and exact filters

The final evaluator obtains only source-proven declaration occurrences:

```text
declarations_by_name[_in_workspace]("ContextA")
  -> builtin / Class / ContextA
```

It then applies exact filters to that declaration object's own identity:

```text
entity.domain == "dotnet"
entity.entity_type == "DbContext"
```

The source declaration fails both comparisons. The separately stored
`dotnet/DbContext/ContextA` relationship endpoint is never a candidate because
it has no declaration authority. Serialization therefore correctly returns
`count=0` under the current contract.

The ASP.NET Controller projection is explicitly disabled when
`declarations_only=true`; no analogous DbContext projection exists in this
handler in either branch.

## 4. Earliest semantic divergence

The earliest semantic divergence is **authority-set selection**, before final
filtering:

```text
declarations_only=false
  -> find_entities_by_name
  -> general occurrence map

declarations_only=true
  -> declarations_by_name
  -> source-proven declaration map
```

This is not a filter-before-refinement defect. The filtered branch does not
receive a `dotnet/DbContext` declaration projection and then accidentally drop
it. It receives the canonical source declaration identity
`builtin/Class/ContextA`, as intended, and applies the requested exact identity
filters to that object.

The user's suspected pipeline is therefore falsified statically. The actual
shape is:

```text
source class capture
  +-> self-Defines builtin/Class -> declaration catalog
  |
  `-> EF semantic relationship -> dotnet/DbContext endpoint in occurrence map

unfiltered lookup -> occurrence map -> can publish both identities
filtered declaration lookup -> declaration catalog -> only builtin/Class
```

## 5. Declaration-authority representation

The authority type is the separate `WorkspaceIndex::declarations` map, keyed
by `(domain, entity_type, name)` and containing file-distinct `EntityRef`
occurrences. `file_declarations` owns their file lifecycle.

Declaration authority is produced only when `WorkspaceIndex::add_edges`
normalizes a self-referential `SemanticRelation::Defines` carrier. It is
canonical source-derived authority, not inferred from:

- being an endpoint of another edge;
- having a source file;
- sharing a name with a declaration;
- framework classification;
- provider discovery; or
- response rendering.

`declarations_only=true` therefore means: **return occurrence records that are
present in this source-proven declaration catalog.** It does not mean “return
every semantic alias or framework projection associated with a syntactic
declaration.”

Yes, a semantic projection can render `dotnet/DbContext/ContextA` while failing
the declaration predicate. That is exactly the present design. It cannot render
with an `is_declaration` claim, because `EntityRef` has no such public field.

## 6. Domain and entity-type authority representation

`EntityRef.domain` and `EntityRef.entity_type` are part of semantic identity,
not detachable labels applied after lookup.

- The source declaration producer authors `builtin/Class/ContextA`.
- The EF producer authors `dotnet/DbContext/ContextA` as a relationship
  endpoint when its own evidence requirements are satisfied.
- General occurrence lookup can expose both identities.
- Declaration filtering tests the identity stored in declaration authority.

The producer and consumer do not silently disagree about fields on one object;
they operate on two different objects with the same name and possibly the same
file provenance. The current public contract exposes that distinction through
identity, but the compact result shape makes it easy for a caller to infer an
unsupported declaration alias.

The proposed principle—“a public semantic attribute used as a query predicate
must have one compatible authority interpretation between publication and
filtering”—does not apply across these two query modes without first requiring
them to share an authority set. Current schema text explicitly says that
domain/type filters on `find_entities` are declaration-only filters.

## 7. Refinement/filter/projection ordering

The relevant ordering is:

```text
compile source
  -> builtin source declaration projection
  -> .NET semantic relationship projection
  -> WorkspaceIndex publication
       -> self-Defines goes to entities + declarations
       -> real relationship endpoints go to entities + graph only

query preparation
  -> scope
  -> initial read from authority selected by declarations_only
  -> common name hydration
  -> optional ASP.NET-only projection when declarations_only=false

query evaluation
  -> repeat selected-authority lookup
  -> exact domain filter (declaration mode only)
  -> exact entity_type filter (declaration mode only)
  -> serialize the surviving EntityRef values
```

There is no DbContext renderer after the filters. Deduplication occurs during
index registration and preserves distinct domain/type identities. Fidelity
affects which semantic facts compilation can publish, but does not merge the
declaration and framework identities.

## 8. Cross-root relationship to Issue #146

Issue #145 does not require Issue #146's cross-root classification asymmetry.
With one source declaration and one EF semantic relationship, the authority
split already exists:

- `builtin/Class/ContextA` is declaration-authoritative;
- `dotnet/DbContext/ContextA` is an occurrence endpoint, not a declaration.

With duplicates across roots, declaration storage preserves file-distinct
occurrences, and scope filters each by its own file provenance. A root that
lacks EF evidence may lack the dotnet endpoint, which is Issue #146 territory,
but no combination turns that endpoint into declaration authority.

Thus duplicate-root behavior can change which occurrences are visible; it does
not cause the reported declaration-filter result. The issues should not be
merged.

## 9. Query-history and generation analysis

Static control flow yields the same authority semantics for all requested
orders:

- cold filtered first;
- cold unfiltered first;
- unfiltered then filtered;
- filtered then unfiltered;
- warm repeated filtered; and
- equivalent batched items.

Both forms share the same hydration key inputs and `LegacyEdit` requirement.
Batch preparation can deduplicate equivalent hydration, but each item retains
its own final evaluator and authority selection. Warm state may populate more
current semantic occurrences, but it does not promote real relationship
endpoints into the declaration map. Publication replacement owns declarations
and relationship projections through their normal file lifecycle.

The tracked warm-state authority regression is especially decisive: across
provider states and two roots it requires the source Class declarations to
remain `builtin/Class`, and asserts that
`declarations_by_identity("dotnet", "DbContext", ...)` is empty.

No query-history/generation defect is needed to explain the report.

## 10. Provider-independence analysis

CBM/provider state affects candidate discovery and hydration completeness, not
the meaning of declaration authority. Clean-CTX compilation and index
publication establish semantic truth. `WorkspaceIndex::add_edges` is the only
boundary relevant to the declaration-versus-endpoint distinction.

With CBM disabled, unavailable, initializing, or ready, equivalent admitted
source evidence has the same identity split. Provider state may affect whether
a file is acquired in a lower-bound query, but it cannot legitimately:

- manufacture a declaration from a relationship endpoint;
- rewrite `builtin/Class` as `dotnet/DbContext`; or
- change exact declaration-filter semantics.

Provider availability is therefore not the causal boundary for Issue #145.

## 11. Deterministic regression design

No RED regression should be added for the proposed invariant because it would
assert behavior contrary to the current documented and tested contract.

If maintainers want a cheap guard against this misunderstanding, the smallest
contract test should be a tracked public-query test under `src/tests/**` using
one C# file with enough EF evidence to create a durable `HasEntity` edge:

```csharp
using Microsoft.EntityFrameworkCore;
public class EntityA { }
public class ContextA : DbContext
{
    public DbSet<EntityA> Entities { get; set; }
}
```

It should assert through actual `workspace_query` responses:

1. unfiltered `find_entities(name=ContextA)` includes
   `builtin/Class/ContextA` and `dotnet/DbContext/ContextA`;
2. declaration-only `builtin/Class` includes the source declaration;
3. declaration-only `dotnet/DbContext` is empty;
4. a reference-only occurrence remains excluded;
5. a non-DbContext class has no dotnet DbContext occurrence; and
6. wrong domain/type declaration filters are empty.

That would be a contract-clarification regression, not a defect regression.
It should remain independent of duplicate roots and Issue #146.

If the desired product behavior is instead a framework-classified declaration
query, the test must wait for explicit architectural approval of a new public
contract. At that point the test should target a new representation or query
mode, not silently reverse the meaning of `declarations_only`.

## 12. Rejected shortcut fixes

- **Special-case DbContext:** invents a one-framework alias and leaves the
  authority model incoherent for Controllers, Hubs, tests, Spring, and Angular.
- **Apply filters to rendered strings:** rendering is not authority and there
  is no post-filter DbContext relabeling in this path.
- **Move all filters to the end:** does not answer which occurrence identities
  possess declaration authority and risks admitting references/endpoints.
- **Weaken `declarations_only`:** violates its schema and the declaration-index
  invariant.
- **Treat every named entity as a declaration:** directly contradicts the
  declaration-authority tests and permits reference-only identities.
- **Manufacture dotnet classification from spelling:** violates structured EF
  evidence requirements.
- **Broaden acquisition only when filters exist:** both forms already use the
  same hydration boundary; acquisition cannot convert endpoint evidence into
  declaration authority.
- **Warm with an unfiltered query:** changes history, not semantics, and still
  cannot populate a dotnet declaration carrier.
- **Disable deduplication:** duplicates are not the cause; the identities are
  intentionally distinct.
- **Couple the answer to CBM state:** makes an optional discovery provider an
  implicit truth authority.

## 13. Smallest architectural boundary requiring repair

**None for the current contract.** The implementation, schema, index authority
model, architectural documentation, and tracked tests agree.

The smallest useful change, if desired, is documentation/UI clarification that
unfiltered `find_entities` returns semantic occurrences including relationship
endpoints and that `declarations_only` filters canonical source-declaration
identities rather than framework aliases.

If maintainers instead want `dotnet/DbContext` to be queryable as a declaration,
the required decision is cross-cutting and externally observable:

1. whether one source declaration may own multiple public semantic identities;
2. whether those are aliases, classifications, or independent declarations;
3. how declaration lifecycle, deduplication, scope, and ambiguity work;
4. whether every framework projection receives the same treatment; and
5. whether existing exact identity and edge-query contracts change.

That design needs explicit approval under the Architectural Approval Gate. It
must not be smuggled in as a local filter fix.

## 14. Files and symbols inspected

- `src/mcp/tool_handlers/query.rs` — `handle_workspace_query`, preparation and
  response dispatch.
- `src/mcp/tool_handlers/query/entities.rs` —
  `prepare_find_entities`, `try_prepare_find_entities`, final evaluator.
- `src/mcp/tool_handlers/query/prepare.rs` — `PreparationContext`, hydration,
  scope, and derived catalogs.
- `src/mcp/tool_handlers/query/csharp_resolution.rs` — structured C#
  declaration/reference catalog.
- `src/mcp/tool_handlers/query/csharp_resolution/dependencies/efcore.rs` —
  request-local EF classification and `dotnet/DbContext` edge subject.
- `src/mcp/tool_schemas.rs` — public `find_entities`, `declarations_only`,
  domain, and entity-type descriptions.
- `src/workspace/index.rs` — general occurrence storage, name lookup, identity,
  scope, and registration.
- `src/workspace/index/edges.rs` — self-`Defines` normalization versus real
  relationship endpoint registration.
- `src/workspace/index/declarations.rs` — declaration authority and lifecycle.
- `src/layers/meta/builtin.rs` — source declaration registration carriers.
- `src/layers/meta/semantic.rs` — `EntityRef`, `SemanticEdge`, and relations.
- `src/dotnet_meta/efcore.rs` — durable EF marker/relationship producer.
- `src/ir/semantic_projection.rs` — declaration-only callable registration.
- `src/tests/workspace/index_declaration_authority.rs` — separation of
  declarations from relationship occurrences.
- `src/tests/mcp/workspace_query_builtin.rs` — builtin declaration and
  framework coexistence expectations.
- `src/tests/mcp/workspace_query_dotnet_efcore_configurations.rs` — public EF
  query behavior.
- `src/tests/mcp/workspace_query_warm_state_authority.rs` — explicit
  `builtin/Class` declaration versus empty `dotnet/DbContext` declaration
  authority across provider/root conditions.
- `docs/plans/WORKSPACE_QUERY_CORRECTNESS_COMPLETENESS_WARM_STATE_INVESTIGATION_2026-10-16.md`
  — prior declaration-filter trace and contract statement.
- `docs/plans/ISSUE_144_REVERSE_EDGE_COMPLETENESS_INVESTIGATION_2026-10-16.md`
  — separation of the reported contradiction from reverse-edge completeness.
- `docs/plans/ISSUE_139_FINAL_ACCEPTANCE_RECONCILIATION.md` — accepted
  occurrence-oriented default and declaration-only exact filtering semantics.
- `docs/ARCHITECTURAL_INVARIANTS.md` — declaration authority, provider, and
  lifecycle constraints.

## 15. Recommended next phase

**CLOSE #145 — NO DEFECT.**

Record the issue resolution with the exact authority distinction and, if the
field UX remains confusing, clarify the public tool description or integration
guidance. Do not implement a production filter change and do not run a Claude
pilot: static implementation, schema, architectural documentation, and tracked
test authority already agree.

If the maintainer wants framework-classified declarations as a new capability,
open a separate architecture-design issue. Its first deliverable should define
the public identity/authority contract before any RED regression or production
implementation.
