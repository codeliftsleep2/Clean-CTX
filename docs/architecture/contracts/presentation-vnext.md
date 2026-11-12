# Model-facing presentation and canonical separation

Agent: Agent MaxHeadRoom. Current contract extracted in B2 from the sources below. Original chronology and implementation evidence remain in the linked source records. IMPLEMENTED records the operative source contract, not new validation.

## 2. Current production boundary

The representation layers remain deliberately separate:

```text
Canonical semantics
  CompiledIR + aligned semantic edges
          |
          +--> Binary 0x04 / dv:2        code-side persistence and transport
          |
          +--> workspace_query           on-demand cross-file and detailed facts
          |
          `--> SCHEMA-vNext              model-facing file-local presentation
                    |
                    `--> raw source      when the complete presentation is not
                                          safely cheaper, or when explicitly
                                          requested as Verbatim
```

SCHEMA-vNext remains a projection. It is not canonical storage,
an edit protocol, a delta protocol, or a substitute for `workspace_query`.


Source: [docs/architecture/SCHEMA_VNEXT_PROPOSAL.md](../SCHEMA_VNEXT_PROPOSAL.md); original section SHA-256 `9431a5be27acdb2ca9fe3687b8477c4d05a97e684f0dcaafb35dfea59df1ad16`. Original wording and tense are retained; admission is limited to the implemented scope described here.

## 3. Non-negotiable invariants

Every candidate must preserve these boundaries:

1. **No semantic omission disguised as compression.** Required facts for the
   selected fidelity remain available either in the presentation or through an
   already-declared on-demand boundary.
2. **No fabricated identity.** Presentation scoping may elide internal IDs only
   after checked ownership has resolved them; it may never replace unresolved
   identity with a guess.
3. **No loss of order or duplicates where they are meaningful.** A shorter
   layout cannot silently convert an ordered occurrence stream into a set.
4. **No exact-body transformation.** Edit bodies remain byte-exact according to
   the existing body contract. Verbatim remains the whole-document escape hatch.
5. **No assumed conversational state.** A prior response, prompt-resource read,
   or cache hit is not schema acknowledgement.
6–7. Economics are owned by [token economics](token-economics.md).
8. **Canonical state is unchanged.** Presentation collapses do not remove facts
   from `CompiledIR`, hierarchy, Binary `0x04`, persistence, or `dv:2`.

---


Source: [docs/architecture/SCHEMA_VNEXT_PROPOSAL.md](../SCHEMA_VNEXT_PROPOSAL.md); original section SHA-256 `de8191f33100c444802483208dd0ba05f083133e0b085435846b84391f38bd2f`. Original wording and tense are retained; admission is limited to the implemented scope described here.

## 11. Compatibility and lifecycle requirements

- SCHEMA-vNext is the production presentation. SCHEMA-v5 remains the recorded
  baseline for the experiments that justified the migration.
- The grammar change received a new visible version and did not silently
  reinterpret `SCHEMA v5`.
- Persistence does not migrate stored presentation text. Restore/replay decode
  canonical state and regenerate the current approved presentation.
- Binary `0x04`, `dv:2`, semantic-edge persistence, and workspace-query
  contracts do not change.
- Exact-body and `byte_exact` metadata contracts do not change.
- Cache breakers derive from the rendered baseline, so v5 and vNext payloads
  cannot share an acknowledged baseline.
- Raw fallback remains byte-exact and carries no schema wrapper or `PATHMAP`.

---


Source: [docs/architecture/SCHEMA_VNEXT_PROPOSAL.md](../SCHEMA_VNEXT_PROPOSAL.md); original section SHA-256 `283c0c83c59a7f749276bac47b3bec6c65ae0527a78ca1d1896e014cdf46c440`. Original wording and tense are retained; admission is limited to the implemented scope described here.


#### A1. Single path occurrence

Current:

```text
// ── α1 (C:\workspace\src\service.ts) ──
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

Candidate:

```text
// α1
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

The alias remains visible at the file boundary and the authoritative mapping
still supplies the exact path. Multi-file outputs remain request-scoped.


Original examples retain their pre/post labels; the candidate forms in this shipped subset are production grammar. Source section SHA-256 `96cec1445b97c82bec2bb1a826ea23efd9415f3d276e8a9b7971d6a68eb3d424`. B4 and Tier C are not admitted as implemented grammar.

#### A3. Remove unreferenced import handles

Drop the internal import handle only after a production-corpus assertion proves
that it is not referenced elsewhere in the visible document and is not a
source-written alias with user-facing meaning.


Original examples retain their pre/post labels; the candidate forms in this shipped subset are production grammar. Source section SHA-256 `809935b36eea8e1b277cca393e082abb1fd4293056aebc00126b36699074c22e`. B4 and Tier C are not admitted as implemented grammar.

#### B1. Remove the method scope arrow

Candidate:

```text
M methodB → void
M find p:id:string → Order mod:PUBLIC ctl:RET
```

The single remaining arrow always introduces the return type. Parameters are
introduced by `p:`. This removes repeated syntax and the visually confusing
`→ →` shape without making the grammar positional or ambiguous.

#### B2. Replace decorative class comments with a typed record

Current:

```text
// ── OrderService ──
```

Candidate:

```text
C OrderService
```

`C` becomes the class counterpart to the existing `Q` interface record. This
is expected to tokenize more predictably than comment punctuation and box-drawing
characters, but the target tokenizers decide.

#### B3. Group fields at all body-free fidelities

Candidate:

```text
F id:number name:string email:string
```

Field order and type information remain intact. If reasoning tests show that a
single field per line materially improves selection or comparison, this item is
rejected regardless of token savings.


Original examples retain their pre/post labels; the candidate forms in this shipped subset are production grammar. Source section SHA-256 `7c3f4e13c7ea5d967c5899bca0c59c807596afb8e516cf33a9db4eb1416c7ad9`. B4 and Tier C are not admitted as implemented grammar.

```architecture-graph
node: AUTH.PRESENTATION_VNEXT
kind: AUTHORITY
status: IMPLEMENTED
title: Model-facing presentation and canonical separation
track: REPOSITORY
artifact: docs/architecture/contracts/presentation-vnext.md
preserves:
  - ARCH-003
  - CONTENT-001
  - CTX-001
note: Source-described operative contract; migration does not execute or independently certify its witnesses.
```
