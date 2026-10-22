# Issue #162 — Retained TypeScript authority contract

Agent: Agent MaxHeadRoom
Date: 2026-10-22
Status: implementation contract within approved Phase 3; not implemented or verified.

## Requirement and bounded scope

AQ01 requires the actual Angular API/declaration identity; AQ25 requires exact
canonical joins with current provenance. Their required consumers are Angular
declaration/API evaluators, followed by family producers and public queries.
These consumers require the native declaration/reference results already
captured by TypeScriptAuthority. Returning only projected semantic edges loses
that evidence before those consumers can use it.

The owner approved retained authority integration and the question-driven
admission rule on 2026-10-22. This contract adds no language semantics, public
query schema, global identity policy or parallel publication registry.

## Observed boundary

- semantic_compilation/artifact.rs owns the capture and source identity, then
  explicitly drops the capture in into_parts.
- semantic_compilation/dependencies.rs::current_parts returns into_parts for
  non-C# artifacts. C# dependency partitions are not a TypeScript policy.
- edit, compress and delta also consume into_parts directly.
- compatibility/validator.rs::CompatibleSemanticProjection carries edges,
  ordinary dependencies, source hash and fidelity; no typed TS authority.
- durable_semantics.rs::SemanticSnapshot owns session edges/dependencies.
  Its currentness check considers only those ordinary dependencies.
- DurableSemanticSnapshot serializes edges/dependencies and owner/hash/version.
  SQLite semantic_state and edit_intent carry that snapshot through persistence.

These are static observations, not executed lifecycle verification. A capture
getter or removal of drop alone cannot satisfy the requirement.

## Minimum transport and owner contract

Preserve an optional typed TypeScriptAuthority payload alongside the existing
compiled result and semantic projection. Absence means no retained TS proof,
never an empty complete inventory. Keep non-TypeScript callers behaviorally
unchanged. Reuse the source owner, source hash, fidelity and producer identity
already associated with the artifact/projection; do not derive identity from
names or encode witnesses into synthetic dependency paths.

Carry this payload through compilation decomposition, compatible projection,
session snapshot and durable snapshot. Thread it through every migrated producer
path, including hydration/provide, explicit-file acquisition, compress, edit,
delta and restore. Canonical-only decomposition may intentionally discard
semantic data only where the caller cannot publish it as semantic authority.
Each such caller must be classified explicitly during implementation.

Retain declaration families, occurrence spans, type/value distinctions, project
and scope evidence and typed witnesses intact. Transport does not reinterpret
type-query tuples as runtime references or certify capture coverage. Framework
API meaning stays in Angular.

## Currentness, compatibility and replacement

The owner hash must agree with the supplied source snapshot. The capture's
source hash alone is insufficient: revalidate consumed witnesses and the
existing compatibility identities before admission and publication. Preserve
the supplied-source versus disk-dependency distinction; supplied editor bytes
must not be required to equal the on-disk target.

Persisted payload is untrusted until normal compatibility and typed witness
validation succeed. A historical capture_current boolean is not fresh proof.
Old records without the payload may preserve their existing compatible
projection but cannot serve as newly migrated canonical TS authority. Required
consumers must reacquire proof rather than infer it from cached edges.

Publish and retire typed evidence with its existing source-owned semantic
generation. Source edits, dependency/config/package mutations, deletion, reset,
restore and replacement must not leave the old payload available as current.
Do not add a separate cache, publication lock order or retry policy. Any required
change to existing ordering must be identified before implementation.

## Consumer and query stopping conditions

Transport is the first slice, not feature completion. Next migrate one actual
Angular API/declaration evaluator to retained endpoints and package-role proof,
with alias, foreign namesake, shadow and type-only negatives. Then migrate all
required family consumers using the same lower authority.

Completion still requires source-owned publication, persistent restoration,
actual MCP query answers, required reverse closure and truthful completeness.
No new query or completeness claim follows merely from adding the payload.
Live consumer verification remains the final batched acceptance boundary.

## Required tracked regressions and execution sequence

1. Capture a nonempty aliased declaration through the real compilation entry;
   carry exact source/hash/span through decomposition into the normal owner.
2. Round-trip that owned payload through actual durable serialization/restore;
   compare the complete retained declaration and witness evidence.
3. Mutate a consumed export/config/negative candidate and require stale authority
   to be refused despite unchanged initiating source bytes.
4. Exercise edit/delete/reset and replacement; no earlier proof may survive as
   current. Supplied immutable source must retain its existing disk distinction.
5. Preserve non-TS and old-record compatibility behavior, while refusing to
   promote a missing payload into canonical proof.
6. Exercise a real migrated Angular consumer and public query, including
   same-name distinct owners and lifecycle invalidation.

Author the first behavioral test against the existing production boundary before
repair. If the intended contract needs an internal API addition to compile,
separate that mechanical interface preparation from behavioral RED; do not call
a compiler failure RED. Use the normal narrow test/registration stash lifecycle
after operator-observed RED, then repair and restore the exact test. Operator
execution owns Cargo and live runs. No test or runtime run is claimed here.

## Qualified erased-alias disposition

UNRESOLVED. The bounded Angular source search for type_queries, typeof and
type_reference_sites found no consuming implementation. That does not establish
absence of a required future declaration-metadata consumer. The three observed
RED positives remain preserved; later negative/lifecycle assertions remain
unclassified. No repair or test removal is authorized by this record.
