# `has_cycle` semantic policy and cycle-witness proposal

**Status:** Complete locally; targeted verification GREEN; repository-wide suite remains CI-owned
**Recorded:** 2026-09-27  
**Current production surface:** `workspace_query(type = "has_cycle")`  
**Primary boundaries:** `WorkspaceIndex`, workspace scope, MCP structured output,
and model-facing completeness claims

## 1. Why this is a separate process

The fidelity-aware `entities_in_file` work cannot be generalized safely to
`has_cycle`. `entities_in_file` has one explicit, authoritative source file;
`has_cycle` is a property of a graph assembled from an effective workspace or
from all evidence retained in the current session.

Compiling one seed file before cycle detection would not make the graph
complete. Compiling the entire workspace implicitly would turn a bounded index
query into an unpredictable workspace-wide compilation operation. This proposal
therefore treats cycle semantics, evidence, and completeness as a distinct
architectural process.

Implementation was authorized after Phase 0 approved the narrow dependency
policy. This document does not authorize exhaustive source compilation or a
breaking change to the existing `has_cycle` field.

## Implementation outcome (2026-09-27)

- `kind` is optional, defaults to `dependency`, and rejects unknown values.
- Eligible relations are exactly `Injects` and `ImportsModule`.
- `WorkspaceIndex` owns one iterative deterministic closed-witness primitive;
  the compatible boolean delegates to it.
- Query-local outgoing-edge ordering makes equivalent insertion histories
  select the same witness. Full cost includes node/edge sorting, not only DFS.
- Witness steps retain asserting-file provenance. Semantic tuple collisions are
  explicitly listed with their admitted occurrence files.
- MCP structured and model-facing responses report
  `indexed_evidence_only` and `source_complete=false`.
- The operation performs no discovery, hydration, source compilation,
  persistence, rendered-context publication, or compression-statistics write.
- Tracked authorities are `src/tests/workspace/index_cycle_policy.rs` and
  `src/tests/mcp/workspace_query_cycle_witness.rs`, with existing WSC-004 suites
  retaining scope-security authority.

Phase 0 findings and the four decisions awaiting approval are recorded in
`HAS_CYCLE_PHASE0_SEMANTIC_AUDIT.md`.

## 2. Code-authority findings

The findings in this section describe the pre-implementation boundary that
motivated the approved change. The implementation outcome above and WSC-007
describe current production behavior.

### 2.1 Current index algorithm

`WorkspaceIndex::has_cycle` and `has_cycle_in_scope` build a deterministic node
index and run three-color depth-first search through `graph_utils::has_cycle`.
The complexity is `O(V + E)` time and `O(V)` working space over retained index
evidence.

The scoped form admits an edge according to the provenance of its asserting
file. This prevents separate repositories from contributing different halves
of a cycle to a scoped workspace result. The unscoped form intentionally uses
all semantic evidence retained in the session.

### 2.2 Current relation policy

Cycle traversal includes every `SemanticRelation` except `Calls`.

That broad policy mixes relationships with different meanings, including
dependency, containment, routing, mapping, testing, declaration, inheritance,
and framework metadata. A graph-theoretic loop across those families is not
necessarily an architectural dependency cycle.

`WorkspaceIndex` already declares a `DEPENDENCY_RELATIONS` set for transitive
dependency traversal. `has_cycle` does not use that set. Its existence is useful
input to the decision, but it must not be adopted automatically: the current set
was designed for another query and includes relations whose cycle meaning still
requires review.

### 2.3 Current external contract

The MCP handler returns only:

```json
{ "has_cycle": true }
```

It does not identify participating entities, relation types, asserting files,
or the edge that closes the cycle. A caller receives no actionable path and may
need additional discovery calls without knowing what to search for.

The operation performs no source discovery, freshness check, hydration, or
compilation. Consequently, `false` means only that no cycle was found in the
admitted indexed evidence. It does not prove that the current source workspace
is complete or fresh.

The shared model-facing envelope currently labels non-file-local queries as an
`authoritative_index_snapshot_for_effective_scope`. That wording is accurate
only as an index-snapshot claim. It can be misread as an authoritative statement
about every current source file in the workspace.

## 3. Problems to solve

The proposed work should resolve four independent problems without conflating
them:

1. **Actionability:** a boolean does not reveal the detected cycle.
2. **Semantic precision:** “every relation except calls” is not an explicit
   architectural-cycle definition.
3. **Coverage honesty:** indexed evidence is not automatically complete or
   fresh source evidence.
4. **Tool economics:** improving the answer must not silently introduce
   workspace-wide compilation.

## 4. Goals and non-goals

### Goals

- Preserve the existing `has_cycle` boolean for compatible callers.
- Return one deterministic cycle witness when indexed evidence contains a
  policy-approved cycle.
- Preserve relation and asserting-file evidence for every witness step.
- Make the exact eligible-relation policy explicit, reviewable, and tested.
- Describe index-only coverage honestly in both structured and model-facing
  output.
- Keep the default operation bounded to the existing in-memory index.
- Preserve workspace-root, additional-root, and `withinPath` scope semantics.

### Non-goals

- Do not auto-compile an entire workspace in the default query.
- Do not claim current-source completeness from index state alone.
- Do not add ordinary method-call recursion to this operation.
- Do not replace dedicated call-graph or recursion analysis.
- Do not enumerate every cycle; one deterministic witness is sufficient for the
  first contract.
- Do not introduce a general graph-query framework or configurable relation DSL.

## 5. Proposed compatible response shape

The existing boolean remains authoritative for the selected indexed relation
policy. New fields make its evidence and limitations explicit:

```json
{
  "has_cycle": true,
  "cycle": [
    {
      "subject": {
        "domain": "angular",
        "entity_type": "Service",
        "name": "OrderService"
      },
      "relation": "Injects",
      "object": {
        "domain": "angular",
        "entity_type": "Service",
        "name": "PaymentService"
      },
      "asserting_file": "src/order.service.ts"
    },
    {
      "subject": {
        "domain": "angular",
        "entity_type": "Service",
        "name": "PaymentService"
      },
      "relation": "Injects",
      "object": {
        "domain": "angular",
        "entity_type": "Service",
        "name": "OrderService"
      },
      "asserting_file": "src/payment.service.ts"
    }
  ],
  "coverage": {
    "status": "indexed_evidence_only",
    "source_complete": false
  }
}
```

For no observed cycle:

```json
{
  "has_cycle": false,
  "cycle": [],
  "coverage": {
    "status": "indexed_evidence_only",
    "source_complete": false
  }
}
```

The cycle array must form a closed directed path in order: each step's object
is the next step's subject, and the final object is the first subject. A
self-loop is represented by one step.

`source_complete: false` is deliberately conservative. Current index ownership
can prove which compiled projections it holds, but it does not own an exhaustive
manifest of every supported source file in the effective workspace. A future
manifest-backed or explicitly exhaustive operation may introduce a stronger
status through a separately approved contract.

## 6. Semantic-policy decision gate

Before implementation, every `SemanticRelation` must be classified for this
specific operation. The classification must be based on what a closed path
means, not merely on whether the relation points from one entity to another.

Initial review categories:

| Category | Examples | Initial direction |
| --- | --- | --- |
| Dependency/lifecycle | `Injects`, `Autowired`, `ImportsModule` | Phase 0 recommends `Injects` and `ImportsModule`; defer `Autowired` because its current target identity is unreliable |
| Type hierarchy | `Extends`, `Implements` | Consider separately; cycles are invalid but semantically distinct |
| Domain relationships | `EntityRelationship` | Requires domain-specific review |
| Containment/declaration | `Defines`, `ControllerAction`, `HasEntity` | Exclude unless a concrete cycle meaning is approved |
| Presentation/routing metadata | `HasRoute`, `HasInput`, `HasSelector`, `RouteMapsTo` | Normally exclude |
| Mapping/testing/event flow | `MapsFrom`, `MapsTo`, `Tests`, NgRx action relations | Review individually; mixed loops may not mean dependency cycles |
| Native calls | `Calls` | Continue excluding; separate call-graph concern |

This table is not the approved relation set. Phase 0 must produce the final
classification with source examples and expected cycle meaning.

Two contract forms remain possible:

1. **One generic approved set:** `has_cycle` means an architectural dependency
   cycle under one documented relation policy.
2. **A small fixed cycle kind:** for example `kind = "dependency"` with future
   separately approved kinds such as `inheritance`.

The second form is clearer if type-hierarchy and dependency cycles require
different relation sets. It must remain a small explicit enum, not a caller-
supplied relation configuration language.

## 7. Determinism and identity requirements

The witness must be stable for identical index state:

- node identities use the existing `(domain, entity_type, name)` model;
- candidate nodes and outgoing edge occurrences have an explicitly defined
  stable traversal order;
- duplicate semantic identities from different files retain occurrence
  provenance;
- the selected witness records the actual asserting occurrence used;
- workspace scope filters edges during traversal, never after reconstruction;
- two repositories may not combine evidence in a scoped result;
- an unscoped result must remain visibly session-scoped in the envelope.

The implementation must not infer filesystem paths from entity names or other
namespaces.

### 7.1 Semantic identity collisions

The current node key intentionally excludes file identity. Multiple physical
declarations with the same `(domain, entity_type, name)` therefore occupy one
semantic graph node while their entity and edge occurrences retain file
provenance.

That model can create an apparent closed semantic path by joining edges that
refer to two different physical declarations sharing one tuple. For example,
one file may assert `X1 -> Y` while another asserts `Y -> X2`; if `X1` and `X2`
share the same semantic tuple, the index sees `X -> Y -> X`.

Phase 0 must determine whether that behavior is intended semantic convergence
or a false cycle for each approved relation family. The witness must not make a
semantic tuple look like a uniquely identified physical declaration. The
approved contract must choose and test one of these responses:

1. preserve semantic-tuple cycle detection and expose identity ambiguity plus
   all relevant occurrence provenance;
2. reject or qualify a witness whose node has multiple admitted physical
   occurrences; or
3. approve a cycle-specific occurrence identity without changing the global
   `WorkspaceIndex` identity model.

Changing the repository-wide entity identity model is outside this proposal.
Silently choosing one occurrence file as the node's declaration is prohibited.

### 7.2 Deterministic ordering and complexity

The node list is currently sorted, but outgoing adjacency vectors retain edge
insertion order. Query-time sorting of every admitted adjacency list would make
the witness deterministic across equivalent insertion histories, but the cost
would be `O(V + E log E)` in the coarse bound (more precisely
`O(V + sum(d(v) log d(v)))`), not the proposed `O(V + E)` target.

Phase 2 must select and document one honest strategy:

- treat canonical insertion order as an enforced index invariant and traverse
  it directly in `O(V + E)`;
- maintain ordered adjacency at the index write boundary and account for its
  insertion cost; or
- sort during the query and report the sorting complexity accurately.

“Deterministic” may not rely on incidental `HashMap` order or on an insertion
order that is not itself guaranteed. The chosen strategy must be covered by a
test that builds equivalent graph evidence in different insertion orders and
expects the same witness.

## 8. Phased implementation plan

### Phase 0 — relation-policy audit and approval — complete

1. Inventory every current `SemanticRelation` producer.
2. Record what a cycle containing each relation would mean.
3. Test representative real fixtures for accidental mixed-family cycles.
4. Test same-tuple entities from different files for false semantic joins.
5. Choose the semantic-tuple ambiguity contract.
6. Choose the generic-set or fixed-kind contract.
7. Approve the exact relation classification and compatibility behavior.

**Exit gate:** maintainer approval of the semantic policy. No production change
occurs in this phase.

### Phase 1 — tracked index-level RED contract — complete

Add the narrowest tracked regressions under `src/tests/**` proving:

1. a two-node approved-relation cycle returns an ordered witness;
2. an approved self-loop returns one witness step;
3. an excluded-relation loop does not produce a cycle;
4. a mixed loop containing an excluded edge does not produce a cycle;
5. two possible cycles select the same deterministic witness every run;
6. equivalent evidence inserted in different orders selects the same witness;
7. every witness step preserves relation and asserting-file provenance;
8. same-tuple declarations in different files follow the approved ambiguity
   contract rather than appearing uniquely identified;
9. scoped queries cannot combine edges asserted by different workspaces; and
10. an acyclic graph returns `false` plus an empty witness.

Follow the repository RED/GREEN procedure exactly: observe focused RED against
the unfixed implementation, stash only the test and registration, implement the
production change, restore the unchanged test, and observe focused GREEN.

### Phase 2 — index-owned witness primitive — complete

Introduce the smallest index-owned return type required to carry a witness.
Extend or complement the existing three-color DFS so it retains parent edge
occurrences and reconstructs the first deterministic closed path.

Requirements:

- `O(V + E)` traversal time and `O(V)` working space remain the target only if
  the chosen stable-order strategy supports that bound; otherwise the documented
  complexity must include sorting or ordered-insertion cost;
- no filesystem access or compilation;
- relation eligibility is one explicit policy, not duplicated by callers;
- `has_cycle` may delegate to the witness primitive so boolean and witness
  semantics cannot drift;
- the obsolete generic boolean-only `graph_utils::has_cycle` is removed after
  the index-owned witness primitive becomes the sole production cycle path.

### Phase 3 — MCP contract and coverage honesty — complete

Add tracked MCP regressions proving:

1. `structuredContent.has_cycle` remains compatible;
2. `cycle` exposes the exact ordered steps and provenance;
3. no-cycle responses expose an empty array;
4. coverage is explicitly index-only and not source-complete;
5. the model-facing envelope does not claim full source-workspace authority;
6. `workspaceRoot`, additional roots, and `withinPath` retain their existing
   security and traversal behavior; and
7. the operation performs no hydration or compilation side effects.

The shared content renderer must not weaken truthful completeness descriptions
for unrelated query types merely to fix `has_cycle`. Prefer query-specific
authority/status inputs or a narrow cycle-specific projection over a global
semantic downgrade.

### Phase 4 — documentation and discovery record — complete

After tracked tests are GREEN:

- update `docs/ARCHITECTURAL_INVARIANTS.md` with the approved cycle policy;
- update `docs/agent/DISCOVERY_REGISTRY.md` with the discovered broad-relation
  and non-actionable-boolean limitations;
- update `docs/agent/tooling.md` with the exact index-only semantics;
- update the originating query-boundary proposal to mark the `has_cycle`
  decision resolved; and
- remove stale claims that a negative result proves source-workspace
  completeness.

### Phase 5 — live stdio MCP verification

**Result (2026-09-27): PASS.** The operator-run
`target/tmp/verify_has_cycle_live.ps1` harness drove a freshly built server and
confirmed schema exposure, first-touch index-only behavior without implicit
compilation, exclusion of a real two-file native `Calls` loop, and a scoped
two-step `Injects` witness with asserting-file provenance. This is field
evidence only; the tracked Phase 1/3 tests remain the regression authority.

Create an optional operator harness under `target/tmp/**` that drives a freshly
built server and demonstrates:

1. a scoped approved-relation cycle returns the expected witness;
2. an excluded-relation loop returns no cycle;
3. a no-cycle result visibly reports index-only coverage; and
4. no prior `provide_code_context` is implied or silently performed by the
   query.

The live harness is field evidence only. It cannot replace the tracked tests or
be reported as a CI gate.

### Phase 6 — final audit and user-run verification gate

**Audit result (2026-09-27):** the production lifecycle is coherent from
approved semantic-edge producers through occurrence storage/removal, scoped
deterministic traversal, MCP dispatch/schema, the model-facing envelope, and
the live stdio boundary. The audit found one presentation inconsistency:
no-cycle responses exposed an empty witness but reported
`completeness.zero_result=false`. A tracked RED/GREEN regression now derives
that field from `has_cycle` for this query family while leaving count-based
families unchanged. No critical or high-severity gap remains. The operator-run
targeted local gate passed: formatting, all-target/all-feature Clippy, cycle
policy and MCP witness suites, file-size guard tests and active-file guard,
UTF-8 validation, and the encoding test. The repository-wide test suite was
not run locally and remains CI-owned by explicit maintainer policy.

Audit the complete production lifecycle:

```text
semantic edge producers
  -> WorkspaceIndex occurrence storage and removal
  -> scoped deterministic witness traversal
  -> MCP structured response
  -> model-facing envelope
  -> live stdio boundary
```

Then hand off the authoritative repository verification commands from
`docs/agent/verification.md`. Do not claim an unrun gate passed.

## 9. Compatibility policy

- Preserve `structuredContent.has_cycle` as a boolean.
- Additive witness and coverage fields are preferred over renaming the query.
- Existing callers that read only the boolean continue to function.
- A changed eligible-relation set may change some historical `true` results;
  this is intentional only after Phase 0 approval and must be recorded as a
  semantic correction.
- Do not return a witness assembled from relations the boolean excludes.
- Do not silently reinterpret the query as method-call recursion.

## 10. Deferred exhaustive-source mode

A definitive source-workspace answer remains valuable but is not part of this
proposal's default implementation. It requires its own design for:

- supported-file enumeration and exclusions;
- symlinks and additional roots;
- freshness and fidelity requirements;
- compilation failures and partial coverage;
- resource limits without silent truncation;
- cancellation and latency; and
- whether compilation state is published or ephemeral.

If a concrete host later requires one-call exhaustive cycle detection, prefer
an explicit mode or separate operation whose cost and failure semantics are
visible. Do not retrofit hidden exhaustive work into the default index query.

## 11. Completion criteria

This process is complete only when:

1. the relation policy is explicitly approved and documented;
2. tracked RED/GREEN evidence protects witness correctness;
3. boolean and witness semantics share one index-owned implementation;
4. scope and provenance are preserved during traversal;
5. responses state index-only coverage honestly;
6. no default workspace-wide compilation has been introduced;
7. documentation matches production behavior;
8. the final architectural audit finds no high-severity gap; and
9. the applicable user-run verification gate and live field scenario are
   reported separately and accurately.
