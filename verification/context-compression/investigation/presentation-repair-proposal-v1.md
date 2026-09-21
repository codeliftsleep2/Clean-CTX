# CONTROL-FULL presentation repair proposal

**Status:** implemented and verified as the narrowly scoped CONTROL-FULL v2 presentation repair

## Finding

The investigated failures are not canonical-semantic defects. The required DI
and occurrence facts exist in the captured model-visible CONTROL-FULL payload,
retain identity/order/provenance, and pass deterministic equality checks.

They are also not explained solely by invalid tests. Capture-specific questions
and oracles were repaired before the controlled experiment. With identical
semantic facts, organization metadata changed observed reasoning reliability:

| Family | Existing presentation | Navigation-index variant |
|---|---:|---:|
| DI provenance | 2/5 | 2/3 |
| Occurrence grouping | 2/5 | 3/3 |

Occurrence grouping therefore has strong evidence of a model-facing
presentation/locality defect. DI provenance has directional but inconclusive
evidence: one organized trial still collapsed distinct subject/object file fields
into an edge-level provenance statement.

## Proposed minimal boundary

Do not alter canonical IR, checked hierarchy, semantic-edge meaning, IDs,
occurrences, ordering, or MCP channel authority. Change only CONTROL-FULL's
model-facing organization.

### Occurrence groups

Add a schema-level navigation block that explicitly maps each method ID to the
existing occurrence-array fields. The authoritative arrays remain unchanged.
The navigation entry must identify:

- typed owner ID;
- canonical method ID;
- semantic family;
- a stable typed field descriptor, resolved by owner kind/ID, member kind/ID,
  field kind, and field name;
- that the outer array is ordered occurrences and each inner array is one
  occurrence group;
- that repeated values and empty groups remain significant.

No values may be copied into a flattened list, deduplicated, summarized, or
renumbered.

The descriptor must not contain serialized array positions or textual JSON
paths. Its meaning must survive serialization, restore, replay, delta
application, and every structured fidelity mode.

### DI provenance

Keep framework/meta edges distinct from core `injection_occurrences`. Add an
edge navigation block whose endpoint-local descriptors remain visibly separate.
Each edge locator is identified by collection, occurrence, relation, layer, and
typed subject/object identity. Endpoint fields remain distinct:

- `subject.file` — subject endpoint provenance;
- `object.file` — object endpoint provenance;
- `layer` — extraction/provenance layer;
- `occurrence` — ordered edge occurrence;
- `relation`, typed subject, and typed object.

Do not introduce a generic `provenance_file` field: that would collapse two
independently represented endpoint facts and could change meaning when files
differ.

## Required invariants

Any implementation must prove:

1. Removing organization metadata yields exact semantic-object equality with
   the pre-change CONTROL-FULL payload.
2. Every typed navigation descriptor resolves to exactly one existing field.
3. Navigation metadata contains no new semantic assertion.
4. Core injection facts and framework edges remain separate families.
5. Subject and object provenance remain independently addressable.
6. Occurrence arrays, group boundaries, duplicates, and ordering remain byte-
   for-byte equivalent after semantic normalization.
7. Low, Medium, High, Edit, delta, restore, and replay use one versioned rule.

## Decision gate

The maintainer approved the externally visible CONTROL-FULL schema/version
change with the additional requirement that navigation use stable typed
descriptors rather than fragile textual or array-index paths. Implementation
must add tracked regression tests under `src/tests/**`, update the schema
version, and rerun only the affected reasoning smoke cases. Do not begin codec
work as part of this repair.

## Verification outcome

The user-run local implementation gates passed. A fresh capture from the rebuilt
production binary reported CONTROL-FULL v2 with the typed navigation schema.
The two affected reasoning smoke cases both passed: DI provenance and occurrence
grouping. No codec work was included.
