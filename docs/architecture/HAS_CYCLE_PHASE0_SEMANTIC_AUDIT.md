# `has_cycle` Phase 0 semantic-policy audit

**Status:** Approved 2026-09-27; Phase 1 authorized  
**Recorded:** 2026-09-27  
**Parent:** `HAS_CYCLE_WITNESS_PROPOSAL.md`  
**Scope:** Current producers, cycle meaning, identity ambiguity, and stable-order
strategy; no production implementation

## 1. Executive finding

The current `has_cycle` implementation is a generic graph-cycle predicate over
every indexed semantic relation except native `Calls`. That behavior is broader
than any coherent architectural-cycle meaning.

The production extractors reveal a narrower reality:

- many relations are one-way metadata or containment edges whose emitted
  subject/object entity types cannot close a producer-real cycle;
- real dependency cycles are currently meaningful primarily for Angular
  `Service --Injects--> Service` and `Module --ImportsModule--> Module` edges;
- Spring `Autowired` is conceptually dependency evidence, but its current
  producer can use a declaration modifier as the service name and therefore is
  not reliable enough for cycle policy;
- hierarchy and entity-relationship variants exist in the enum but have no
  current production producer; and
- the existing synthetic `has_cycle_structural_cycle` test proves the old broad
  graph rule by manually creating edge shapes that production extractors do not
  emit.

The recommended contract is a small fixed `kind` enum whose first and default
value is `dependency`. It should not expose caller-configurable relations. The
initial dependency policy should admit only `Injects` and `ImportsModule`.
Additional kinds or relations require separately demonstrated meaning and
reliable producers.

## 2. Audit method and authority

This audit traced:

- all variants in `src/layers/meta/semantic.rs`;
- every production `SemanticRelation` construction under `src/`;
- index insertion, occurrence provenance, removal, and query identity;
- current traversal in `src/workspace/index/traversal.rs`;
- current graph regressions in `src/tests/workspace/index_graph.rs`; and
- MCP dispatch and response projection in
  `src/mcp/tool_handlers/query/graph.rs` and `query/content.rs`.

Code is the authority for current behavior. Existing documentation and tests
were used to identify intended and historically protected behavior, not to
override the production call path.

No Cargo command or live server was run during this investigation. Phase 1 will
create the tracked RED contract after the decisions in section 8 are approved.

## 3. Complete relation inventory

“No producer” means the enum variant exists but current production code does not
construct it. “Cannot currently close” describes producer-emitted entity types,
not arbitrary manually inserted `SemanticEdge` values.

| Relation | Production shape | Current producer | Closed-path meaning | Recommendation |
| --- | --- | --- | --- | --- |
| `Injects` | Angular Component/Service/Directive -> Service | Angular semantic layer | Runtime/DI dependency; Service -> Service can close | Include in `dependency` |
| `HasInput` | Component -> InputField | Angular semantic layer | Declaration containment | Exclude |
| `HasOutput` | Component -> OutputField | Angular semantic layer | Declaration containment | Exclude |
| `HasModel` | None | No producer | Unspecified | Exclude until produced and reviewed |
| `HasSelector` | Component -> Component-valued selector marker | Angular semantic layer | Presentation metadata | Exclude |
| `HasTemplate` | None | No producer | Unspecified | Exclude until produced and reviewed |
| `HasStyle` | None | No producer | Unspecified | Exclude until produced and reviewed |
| `DeclaresInModule` | Module -> Component/Directive/Pipe | Angular semantic layer | Containment/declaration | Exclude |
| `ImportsModule` | Module -> Module | Angular semantic layer | Module dependency; can close | Include in `dependency` |
| `ExportsFromModule` | Module -> Component/Directive/Pipe | Angular semantic layer | Public containment/export metadata | Exclude |
| `RouteMapsTo` | Route -> Component | Angular semantic/routing layers | Navigation metadata | Exclude |
| `GuardedBy` | Route -> Guard | Angular semantic/routing layers | Navigation policy metadata | Exclude |
| `ResolvedBy` | Route -> Resolver | Angular semantic/routing layers | Navigation metadata | Exclude |
| `Dispatches` | Component/DispatchSite -> Action | NgRx shape | Event flow; cannot currently close | Exclude from dependency cycle |
| `Selects` | Component/SelectSite -> Selector | NgRx shape | Data selection | Exclude |
| `HandlesAction` | Effect -> Action | NgRx shape | Event consumption | Exclude from dependency cycle |
| `CallsService` | Effect -> ServiceMethod | NgRx shape | Operational dependency; cannot currently close | Defer; exclude initially |
| `HasStore` | Component -> Store | NgRx shape | Held framework dependency; cannot currently close | Defer; exclude initially |
| `TriggersReducer` | Action -> Reducer | NgRx shape | Event/state flow | Exclude from dependency cycle |
| `ProducesAction` | Effect -> Action | NgRx shape | Event production | Exclude from dependency cycle |
| `ControllerAction` | Controller -> Action | .NET semantic layer | Declaration containment | Exclude |
| `HasRoute` | Controller -> Route | .NET semantic layer | Routing metadata | Exclude |
| `HasEntity` | DbContext -> Entity | .NET semantic layer | Data containment | Exclude |
| `EntityRelationship` | None | No producer | Potential data-model cycle | Exclude until a separate data-model policy exists |
| `MapsFrom` | MapperProfile -> Entity | .NET semantic layer | Mapping metadata | Exclude |
| `MapsTo` | MapperProfile -> Entity | .NET semantic layer | Mapping metadata | Exclude |
| `HubMethodTargets` | Hub -> HubMethod | .NET semantic layer | Invocation-target metadata | Exclude |
| `Tests` | TestArtifact/TestClass -> production type | Angular/.NET testing layers | Test coverage, not production dependency | Exclude |
| `Autowired` | Spring Controller -> nominal Service | Spring semantic layer | Conceptual DI dependency, but current object name may be a modifier such as `private` | Exclude until producer identity is corrected and proven |
| `EndpointMapsTo` | Controller -> Endpoint | Spring semantic layer | Routing metadata | Exclude |
| `BeanProduces` | Configuration -> Bean | Spring semantic layer | Factory/containment metadata | Exclude initially |
| `ConfigurationProperties` | Configuration -> Properties | Spring semantic layer | Configuration binding | Exclude |
| `Extends` | None | No producer | Type-hierarchy cycle | Reserve for future `inheritance` kind |
| `Implements` | None | No producer | Type-hierarchy cycle | Reserve for future `inheritance` kind |
| `Defines` | Self-registration; Pipe -> PipeName; Guard/Resolver -> kind marker | Builtin/Angular layers | Registration or declaration metadata | Exclude |
| `Calls` | Method -> Method | Generic IR projection | Call recursion/call-graph cycle | Continue excluding; separate operation |

## 4. Mixed-family cycle finding

The current regression `has_cycle_structural_cycle` manually constructs:

```text
Route --RouteMapsTo--> Component
Module --DeclaresInModule--> Route
Component --Injects--> Module
```

Production extractors do not emit two of those shapes:

- `DeclaresInModule` targets Component, Directive, or Pipe, not Route; and
- `Injects` targets Service, not Module.

The test is valid evidence of the current generic graph rule, but not evidence
that such a cycle represents production architecture. Under the proposed
dependency policy it must become an excluded mixed-family regression rather
than remain an expected cycle.

This is a deliberate semantic correction, not behavioral preservation. It
requires explicit approval because historical `true` results formed only by
excluded relations will become `false`.

### 4.1 Spring `Autowired` producer boundary

`collect_field_annotations` takes the first whitespace-delimited token after
`@Autowired` as `field_name`. For a conventional field declaration such as:

```java
@Autowired
private final UserService userService;
```

that token can be `private`. `extract_spring_semantic_edges` then publishes a
`spring/Service/private` object. Even with a corrected type/name extractor, the
current `Controller -> Service` shape cannot form a same-family cycle because
no Spring producer emits `Service -> Controller` or `Service -> Service`.

`Autowired` is therefore excluded from the initial cycle kind. Correcting its
semantic target is legitimate separate RED/GREEN work, but it is not necessary
to implement a precise initial cycle witness and must not be folded into this
process without explicit scope approval.

## 5. Identity-collision audit

### 5.1 Current model

Graph nodes use `(domain, entity_type, name)`. Files are deliberately excluded
from semantic identity. Entity occurrences and edge occurrences retain file
provenance, but all outgoing occurrences for one semantic tuple share one
adjacency node.

Consequently, this evidence:

```text
file-a: Service X --Injects--> Service Y
file-b: Service Y --Injects--> Service X
```

forms a semantic cycle even if the two `X` or `Y` names refer to physically
different declarations in separate namespaces within the same admitted
workspace. Namespace or resolved-symbol identity is not available at this
boundary.

The same risk applies directly to `Module -> Module` imports. It is less
relevant to one-way metadata families because their producer-emitted entity
types cannot close a path.

### 5.2 Recommended ambiguity contract

Preserve the repository's approved semantic-tuple identity rather than invent a
cycle-local resolution system. A witness should state:

- `identity_model: "semantic_tuple"`;
- whether any witness node has more than one admitted entity occurrence;
- for each ambiguous node, the admitted occurrence files known to the index;
  and
- the exact asserting file for every witness edge.

Example qualification:

```json
{
  "identity_model": "semantic_tuple",
  "identity_ambiguous": true,
  "identity_ambiguities": [
    {
      "entity": {
        "domain": "angular",
        "entity_type": "Service",
        "name": "OrderService"
      },
      "occurrence_files": ["src/a/order.service.ts", "src/b/order.service.ts"]
    }
  ]
}
```

These paths are occurrence provenance, not proof of a uniquely resolved
declaration. The witness must not silently select one file as the entity owner.

The boolean remains a statement about the semantic-tuple graph. Ambiguity
qualifies the result instead of suppressing real indexed evidence. A future
symbol-resolved graph may provide physical-identity cycle detection through a
separate contract; changing global `WorkspaceIndex` identity is out of scope.

## 6. Contract-form audit

### 6.1 One unqualified relation set

This is superficially compatible but leaves no vocabulary for separating
dependency, inheritance, data-model, event-flow, and call cycles. Adding one of
those families later would silently change the meaning of existing answers.

**Finding:** reject.

### 6.2 Small fixed `kind` enum

Proposed initial public values:

```text
dependency
```

Omitting `kind` defaults to `dependency`. Future values such as `inheritance`
or `data_model` require their own producers, relation policy, tests, and explicit
approval. Unknown values return `-32602`.

This creates a stable semantic namespace without a configurable relation DSL.

**Finding:** recommend.

### 6.3 Compatibility consequence

The query name and `has_cycle` boolean remain. The semantic meaning narrows from
“any indexed non-call relation loop” to “an indexed loop under the selected,
documented cycle kind.” That is an externally observable correction and must be
called out in tooling and invariant documentation.

## 7. Stable-order and complexity audit

Current node identities are sorted before DFS. Outgoing `StoredEdge` vectors
retain insertion order; equivalent semantic evidence inserted in another order
can therefore select a different first cycle.

Three strategies were considered:

1. **Canonical insertion invariant:** would require every producer and every
   index publication path to supply one order, expanding the change across
   write boundaries.
2. **Permanently ordered adjacency:** changes insertion/removal costs and shared
   index ownership for the sake of one query.
3. **Query-local ordering:** clone or reference admitted eligible occurrences,
   sort by a complete stable key, and traverse that view.

The recommended first implementation is query-local ordering. It is the
smallest local change, does not impose costs on unrelated queries, and makes
equivalent graph evidence deterministic regardless of publication history.

The stable edge key should include, in order:

```text
subject (domain, entity_type, name)
relation rank
object (domain, entity_type, name)
asserting_file
```

The relation rank must be an exhaustive policy-owned match, not debug text or
serialized spelling. Because the initial dependency kind has only two
relations, the rank is small and explicit.

Honest complexity:

```text
node ordering: O(V log V)
admitted-edge ordering: O(sum(d(v) log d(v)))
DFS traversal/reconstruction: O(V + E)
working space: O(V + E) for the deterministic adjacency view and DFS state
```

The old `O(V + E)` traversal statement remains true only for DFS after ordering;
it is not the full query bound.

## 8. Decisions requiring maintainer approval

Phase 0 closed with explicit maintainer approval of all four decisions on
2026-09-27:

1. **Cycle kind:** add optional `kind`, initially/default `dependency`.
2. **Dependency relation policy:** initially admit exactly `Injects` and
   `ImportsModule`; defer `Autowired` until its producer identity is corrected
   and the relation can participate meaningfully in a closed path.
3. **Identity ambiguity:** retain semantic-tuple boolean semantics and expose
   admitted occurrence ambiguity rather than suppressing or pretending to
   resolve it.
4. **Determinism:** use query-local stable edge ordering and document the
   sorting-inclusive complexity.

## 9. Phase 1 RED implications

Once section 8 is approved, the unchanged RED contract must prove:

- `Injects` and `ImportsModule` policy behavior, with `Autowired` explicitly
  excluded;
- former mixed structural loops are excluded;
- `Calls` remains excluded;
- ordered witness closure and self-loop representation;
- exact asserting-file provenance;
- same-tuple cross-file ambiguity is surfaced;
- equivalent evidence inserted in different orders returns the same witness;
- workspace scope prevents cross-repository edge composition; and
- MCP omission/default/invalid handling for `kind` is exact.

No production implementation should begin before those tests are observed RED
and stashed under the repository regression procedure.
