# ANG-R2 Phase A: proposed public written resolver registration contract

Agent: **Agent MaxHeadRoom**. Architecture accepted by owner disposition 01;
bounded clarifications and controlling candidate pending B11 admission.
Baseline: `2aa988868202bb39307d2d256bab113c9e61b219`, Angular semantic generation 78.
No implementation, new acceptance result or governing authority is asserted.

## Accepted architectural direction

Approve a **typed source-owned resolver registration collection alongside the
existing semantic edges**, produced in the same Angular evaluation and exposed
through one additive `workspace_query` operation, `resolver_registrations`.

Preserve generation-78 `ResolvedBy` edges exactly as unqualified object/target
membership. Do not add graph nodes, relation enum values, CoreOps, generic edge
metadata, endpoint variants or global EntityRef/equality changes for registration.
The new typed collection is part of the same compilation result, source-owned
snapshot, durable semantic payload and index coverage entry, not a second store,
lock, registry, publication owner or query-time semantic interpreter.

The required change is substantive: it adds a public query and a typed semantic
projection/payload. It needs this owner decision and the B11 amendments described
in [compatibility](ANG_R2_COMPATIBILITY.md) before production implementation.

## Exact current loss boundary

`route_resolvers::project` proves a mapping with `resolve_mapping_at`, checks
runtime reference agreement, then calls `targets.insert(id.clone())`. A later
different key selecting the same declaration reaches the duplicate-target branch
and emits no second edge. The first emitted `SemanticEdge` also has no key or
mapping-support field. Thus key-specific semantic projection is lost at production,
before persistence, index insertion or public serialization. Original neutral
`TypeScriptAuthority.resolve_mappings` survives those paths; acquisition is not
the missing mechanism. The deduplication is correct for the existing relation.

## Identity and evidence

| Subject | Proposed rule |
| --- | --- |
| Registration site | Exact existing RouteOccurrence, source-qualified mapping pair occurrence and admitted decoded mapping key. |
| Written registration fact | Registration site plus the exact canonical resolver declaration. |
| Route identity | Existing root CanonicalDeclaration and source/hash/Object byte span, unchanged. Display path is a bucket label. |
| Resolver identity | Existing canonical declaration source/hash/span/name/kind, with the existing target site meaning unchanged. Equal display names do not equate declarations. |
| Key value | Exactly the existing supported `ResolveMapping.name` key semantics; quoted-string escapes use the existing decoder. No case folding, Unicode normalization or new evaluator. |
| Source occurrence | Existing outer field pair/key, mapping Object/pair/key/value spans, source and source hash; retained separately from decoded-key equality. |
| Duplicates | Existing duplicate decoded-key refusal, competing outer-field refusal, unknown-key container refusal and independently proved sibling preservation. Never first/last-wins. |
| Repeated acquisition | Same source-owned generation and same fact yield one registration. Distinct keys remain separate even when their target is identical. |
| Key-only replacement | Replace the old source-owned registration collection with the new collection, even when rendered membership is unchanged. |

**Snapshot identity qualification:** current RouteOccurrence includes source hash
and the root declaration's source hash. A real key-only source edit therefore
changes the snapshot coordinate even if byte offsets, display path and external
resolver declaration remain unchanged. No stable route ID across edits is invented.
The required preservation is semantic continuity of the source location and exact
retirement/replacement, not equality of old/new hashed coordinates. Likewise,
equivalent key spellings keep the same decoded key component; changing source bytes
can still change the overall snapshot identity. Raw spelling is evidence, not an
additional semantic key.

Existing mapping-key proof uses a Rust String: ordinary identifiers and supported
quoted strings. It is not the separate general UTF-16 string-token carrier.
Unpaired-surrogate strings and key forms for which the existing decoder supplies
no key remain unsupported. Empty decoded names currently share the unknown-key
sentinel and cannot establish this container's registrations. Numeric/computed/
spread/shorthand forms gain no support. Escaped identifier normalization is not
certified by source inspection of raw identifier text; do not claim general JS
property-key equivalence or expand decoding under this proposal. Preserve original
unqualified behavior and return any demonstrated required decoding defect separately.

## Proposed typed semantic shape

These are exact semantic fields, not a commitment to these Rust type names:

```text
ResolverRegistrations {
    format_version: 1,
    records: Vec<ResolverRegistration>,
    semantic_omissions: Vec<ResolverRegistrationOmission>
}

ResolverRegistration {
    route: existing RouteOccurrence,
    route_path: String,                 // presentation only
    key: String,                        // admitted decoded key
    resolver: existing CanonicalDeclaration,
    evidence: {
        source: canonical source path,
        source_hash: consumed source hash,
        field_pair: [start, end],
        field_key: [start, end],
        object: [start, end],
        pair: [start, end],
        key: [start, end],
        value: [start, end]
    }
}

ResolverRegistrationOmission {
    source: canonical source path,
    source_hash: consumed source hash,
    start: retained occurrence start,
    end: retained occurrence end,
    reason: one of the bounded codes described below,
    route: optional existing RouteOccurrence, // only if actually proved
    route_path: optional String,              // only if actually proved
    resolver: optional CanonicalDeclaration   // only if actually proved
}
```

All intervals are zero-based half-open **UTF-8 byte** spans into the named hashed
source, not UTF-16 positions or offsets into an importing owner. `route.root`
retains the actual existing declaration coordinate. No full source, AST, entire
dependency snapshot or new witness database is copied into a registration.
The retained authority remains the source of proof and typed currentness witnesses.
The record's full proof fields are checked at admission even when its semantic
key is already present; equal labels or valid independent endpoints cannot repair
a crossed occurrence. Duplicate decoded keys refuse before producing records.

Known detected unsupported/refused mapping forms retain a typed omission alongside
the records: source, source_hash, actual retained start/end span and a bounded reason
code. Initial reasons distinguish duplicate_resolve_field, duplicate_mapping_key,
unsupported_mapping_key, unsupported_mapping_value, unresolved_mapping_target,
ambiguous_mapping_target and type_only_mapping_target where that distinction is
actually established. Do not guess a more precise reason from missing evidence.
Record a generic mapping_proof_not_established reason only for an actual retained
candidate occurrence; do not invent coordinates or label every ordinary TypeScript
file unsupported. Stale/forged payload or failed acquisition is a non-success,
not an omission used to hide failed validation. These diagnostics belong only to
the new source-owned projection/query and do not change old response diagnostics.

The optional collection distinguishes missing support (`None`) from a produced
current empty collection (`Some` with zero records). Neither an empty collection
nor inventory completion proves whole-workspace source completeness.

## Public requests

Add only this `workspace_query` operation:

```json
{
  "type": "resolver_registrations",
  "direction": "forward",
  "route_path": "dashboard",
  "workspaceRoot": "<authorized workspace root>"
}
```

```json
{
  "type": "resolver_registrations",
  "direction": "reverse",
  "resolver_name": "dashboardResolver",
  "workspaceRoot": "<authorized workspace root>"
}
```

The root placeholder is illustrative, not a discovered repository path.
`direction` must be exactly `forward` or `reverse`. Forward requires one string
`route_path`, including the empty string; reverse requires one nonempty string
`resolver_name`. The opposite selector, arrays, nulls, missing root and wrong
types reject. Existing `name` validation and old query operations do not change.

The two selectors explicitly select **occurrence buckets**. A forward path can
match multiple route Objects; a reverse name can match multiple declarations.
Return every admitted registration with exact coordinates so consumers can separate
them. Do not call a returned name bucket an exact canonical selector. No generic
public canonical-ID selector, stable opaque ID, source-proximity join or cross-edit
identity is introduced.

Both forms accept existing `withinPath` as asserting-source answer filtering.
It does not select the target declaration or narrow prerequisite acquisition.
An unauthorized scope rejects before IO. Batch requests reuse the existing shared
root/scope, per-item IDs, failure isolation and preparation/final validation.
New direction/selector fields must participate in any request-local deduplication.

## Public result

Use the existing QueryAnswer/content/structuredContent envelope. The new operation's
structured result is:

```text
{
  format_version: 1,
  direction: "forward" | "reverse",
  registrations: [ResolverRegistration, ...],
  semantic_omissions: [selected-bucket omissions with proved attribution, ...],
  candidate_source_omissions: [unassociated inspected-source omissions, ...],
  count: number of returned registration records,
  coverage: {
    status: "indexed_evidence_only",
    fact_semantics: "authoritative_indexed_occurrences",
    source_complete: false,
    result_semantics: "lower_bound",
    omitted_possible: true,
    reasons: [{code: "bounded_written_resolver_registration_scope"}]
  },
  discovery: existing HydrationReport projection when present
}
```

The public record uses the semantic fields above, with canonical source paths
serialized as strings and span tuples as two-element integer arrays. It exposes
bounded snapshot coordinates, not the private enum encoding or all internal state.
Model-facing text must preserve exactly the same keys, identities, multiplicity,
scope and refusal meaning as structuredContent; an ordinal is presentation only.
Declare the new result fields and exact record shapes in the registered tools/list
output schema, including the new operation's required fields. Preserve existing
coverage enums: the reason above uses the already permitted object form, not a
new global reason string. The source-qualified omission records are a new-operation
top-level field; do not change the existing coverage.semantic_omissions string enum.
In query/content.rs, render the new direction/route_path/resolver_name explicitly
instead of leaving them null under the old edge-query descriptor.
Do not change the global WORKSPACE-QUERY envelope version merely for this additive
operation. The new operation/result has its own `format_version:1`.

Sort this new result deterministically by asserting source, route Object start/end,
mapping pair start/end, then resolver source/start/end. Preserve every distinct
registration; do not apply a name-only edge or JSON deduplicator. This ordering
affects only the new operation. Existing edge order/count/fields remain unchanged.

Failed required acquisition or final currentness validation follows existing
QueryFailure semantics, never successful fabricated empty output. Missing/stale
registration payload must trigger normal reacquisition or truthful non-success;
an old unqualified edge cannot fill it. Valid supported siblings may still appear
in a bounded lower-bound result when other source forms are unsupported, under
existing refusal/currentness rules. No new completeness certificate is invented.

## Forward/reverse examples

R1/R2 and D1/D2 below are symbolic exact coordinates, not stored IDs or receipts.

| Written input | New forward records | New reverse records | Existing membership |
| --- | --- | --- | --- |
| R1: primary -> D1; secondary -> D1 | (R1, primary, D1), (R1, secondary, D1) | Reverse D1 name bucket contains both records | One R1 ResolvedBy D1 |
| R1: primary -> D1; R2: primary -> D1, same display path | Both coordinates remain distinct | Both exact owners retained | One support for each route/target |
| R1 -> D1 and R1 -> D2, equal resolver display names | Distinct target coordinates in records | Name bucket returns both, without choosing one declaration | Existing exact canonical memberships preserved |
| R1: primary changed to fallback, same external target | New snapshot has fallback only | Old primary registration absent | Rendered unqualified membership can remain identical |

## Canonical question matrix for this bounded slice

This is the ANG-R2 contribution to existing AQ13/AQ25, not closure of W06 or #162.
All rows are proposed/UNRUN; each consumes native route ownership, retained mapping
and runtime declaration proof under current source/dependency/package/project witnesses.
The new query's completeness contract is the explicit lower bound above; no row
earns closed-world absence or runtime scope merely from a successful result.

| ID / question and intent | Required fact / authority | Acquisition and public path | Expected answer / adversarial and lifecycle witnesses |
| --- | --- | --- | --- |
| REG-Q1: Which named registrations are written on this route occurrence? Inspect actual route configuration. | Exact route + mapping occurrence + key + target; resolver/foundation/retained/canonical authorities. | Existing scoped inventory/current owner acquisition; new forward route_path bucket; select returned exact coordinate. | Distinguishable records for two keys/one target and equal-path objects; P01/P02/P03/P05/P06, N01/N02. |
| REG-Q2: Where is this declaration registered, and under which keys? Trace reverse written use. | Same exact target identity attached to each proved owner/key; WSC-012 prevents name-only joins. | Cold alias-aware nomination and current publication; reverse resolver_name bucket; returned declaration coordinates distinguish namesakes. | Exact per-route/key rows, not a guessed single declaration; P02/P03/P05/P06, N02/N05. |
| REG-Q3: Why is this registration true? Inspect provenance without reconstructing syntax. | Native route ownership and retained field/Object/pair/key/value joins to the same target; AUTH-001/SEMDEP-001. | Same acquired source-owned collection; public evidence fields in either direction. | Exact source/hash/ranges and current target; forged/borrowed/foreign proof refuses; P04/P06/P07, N01/N03/N04. |
| REG-Q4: What changes when a key, mapping or target changes? Determine retirement impact. | Source-owned registration fact independent from rendered membership equality; retained lifecycle and producer compatibility. | Normal edit/delta/refresh/reacquisition and durable/public paths. | Old key/target absent as current; membership/siblings preserved; P07/P08/P09, N04. |
| REG-Q5: Are these registrations distinct, and what can this answer omit? Avoid identity/completeness overclaims. | Mapping occurrence/key versus target membership; WSC-004/WSC-014 and written-only resolver qualification. | Shared scope; source-qualified records and semantic_omissions; same result in text/structured/batch output. | Registration multiplicity, exact coordinates, bounded omission meaning, no executable eligibility; P01/P02/P04/P05, N05/N06. |

Identity, F/R, why, source scope, change, completeness and lifecycle are covered
above. Composition is only the established explicit native route/children envelope;
effective route/injector composition and runtime cross-layer behavior are excluded.
The required cross-layer join is neutral TS evidence -> Angular written registration
-> canonical/source-owned result -> public query, verified through the default path.

## Native ownership and unsupported applicability

Use only the already admitted native typed-const Routes/Object/resolve envelope
and current neutral proof. Direct/alias/barrel/namespace/local runtime declaration
references remain the intended supported forms. A type-only Routes annotation may
prove the root; a type-only resolver value cannot prove a runtime target.

No new canonical registration outside native ownership; legacy membership remains
legacy. Preserve duplicate/partial policy, owned no-weak-fallback and independent
guard/component/loadChildren/provider/config/bootstrap/marker facts. No callable
eligibility, execution, navigation, runtime DI, provider selection or value result.

## Owner disposition 01: exact omission attribution

Accepted architectural direction; these bounded clarifications supersede the initial
proposal's undifferentiated omission list. They do not assert implementation.

Retained omissions contain source/hash/span/reason and optional `route` and
`route_path` only where native ownership and the path are actually proved; optional
`resolver` is present only with an independently established exact canonical runtime
declaration for that occurrence. Absence is not an inferred name, target or route.
The reason vocabulary remains the bounded codes above, supported by actual proof.

The new response separates two arrays:

- `semantic_omissions`: omissions with proved association to the selector bucket.
  Forward requires a proved route occurrence and path equal to `route_path`, and
  carries `attribution:"selected_route"`. Reverse requires a proved canonical
  resolver declaration whose name equals `resolver_name`, and carries
  `attribution:"selected_resolver"`. A positive sibling or an import of the requested
  name does not attribute a refused mapping to that target. Equal names still form
  a bucket, not one declaration. Coordinates identify the actual proved association.
- `candidate_source_omissions`: remaining detected occurrences within inspected
  candidate sources, with `attribution:"candidate_source"`. Their source evidence
  establishes inspection and a local unsupported form, **not association with the
  requested route/resolver**. They must be described as unassociated inspection
  diagnostics, never failures of a selected registration. Do not copy the requested
  selector into an unproved field. A reverse unresolved value normally belongs here.

Both arrays obey asserting-source withinPath filtering and final currentness.
An occurrence appears in exactly one array per answer. Exact same source/span/reason
and proved association deduplicate diagnostics; distinct facts are not collapsed by
display name. Sort diagnostics by source/start/end/reason/attribution and exact
optional coordinates. `count` counts positive registrations only. Both arrays and
optional association fields must be declared in the operation-specific output schema
and conveyed with the same attribution in model-facing text and batch results.

General bounded-discovery limitations belong in `coverage`/`discovery`, not fabricated
source occurrences. Empty arrays do not mean no omissions are possible. Unknown
forms without retained coordinates cannot acquire invented spans. Missing required
proof or collection, stale/forged payload and failed required acquisition remain
reacquisition/non-success conditions, not either successful omission category.

## Owner disposition 01: cold nomination contract

For both directions, enumerate configured authorized roots with the existing
`source_inventory`, nominate eligible TypeScript owners independent of selector text,
deduplicate physical owners with `select_candidates`, and acquire High semantics
through `compile_candidates_for`. Existing `prepare_candidate` checks current
source/hash/fidelity before scheduling compilation; `select_candidates` alone does
not skip current semantic owners. Keep normal dependencies, authorization, exclusion,
publication tickets and final footprint validation. withinPath filters asserting
answers, never prerequisite discovery. No resolver-name/path grep gate, alternate
index, provider fallback or query-time semantic interpreter is permitted.

Source inspection establishes available shared mechanics, not working new dispatch.
The missing integration is an operation-local preparation call and consumption of
its failure/currentness outcomes; no absent alias nomination primitive is presently
demonstrated. An owner with no textual occurrence of the target declaration name
remains nominated, so alias/barrel/namespace/local forms use existing native TS
resolution. If actual cold testing disproves this, report the exact prerequisite
before attempting a broader repair. Do not silently narrow supported forms.

Required nominated acquisition/publication failure cannot be converted to successful
empty collections by copying bootstrap's partial report alone. Preserve existing
QueryFailure categories; carry incomplete inventory honestly as bounded discovery
without certifying absence. No global failure policy or completeness change is made.

Cold acceptance must start each forward and reverse case in separate fresh state and
fresh durable workspace, without preseeded owners, direct index mutation or a warm-up
query. Exercise direct, alias, barrel, namespace and local forms, including sources
that never spell the selected target name, namesakes and authorized additional roots.
Inspect first-query records and acquisition receipts; then warm, restart and mutate
separately. Measure enclosing inventory/preparation/currentness/compilation and query
costs. These are required future witnesses, not executed results.

## Approval boundary

Owner disposition 01 accepts the representation, identity and public operation.
The exact omission/nomination clarification and compatibility candidate are now
prepared for the existing governed authority checkpoint.
The controlling authority amendments must then pass B11; approval of this proposal
is not an agent-generated governance grant. Phase B execution permissions remain
per exact test/build/production invocation. See the accompanying disposition and
verification reports before deciding.
