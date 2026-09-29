# COMPACT-A3 Phase 3C — scoped type/callee table (negative result, removed)

**Status:** negative result; table removed, working tree restored to the Phase 3B
checkpoint
**Predecessor:** Phase 3B identity renumbering (token delta 0.00%)
**Outcome:** the table increased the complete payload on every measured row, so it
was removed rather than forced to remain.

## What was tested

An optional file-local table that deduplicated repeated field/parameter/return
types and repeated callee-written names. Tabled strings were referenced as `@<n>`
from `#T`/`#N` records emitted after the header; only strings occurring more than
once were tabled. The table was an explicit on/off form (`encode_tabled`/
`decode_tabled` plus cold variants) with a slightly extended legend
(`#T=type-table #N=callee-table @n=table-ref`). References decoded byte-for-byte
and failed closed on malformed or out-of-range indices.

## Measured result (negative)

The table candidate was larger than the table-less candidate on **every** one of
the 30 measured rows (15 tracked-economics + 15 correctness/lifecycle, under both
cl100k and o200k), with per-row deltas from +54 to +828 tokens. The
production-selected aggregate moved in the wrong direction. Representative
tracked-economics deltas (`table − off`):

| Fixture | cl100k delta |
|---|---|
| typescript (all five fidelities) | +145 |
| angular (low … edit) | +185 … +203 |
| csharp (low … edit) | +154 … +238 |

## Why it failed

The strings that actually repeat on this corpus — `string`, `void`, `boolean`,
`number`, `any`, and short callee names — are already single BPE tokens. An `@n`
reference costs the same one token as the inline spelling, so deduplication saves
nothing while the `#T`/`#N` table rows and the extended legend add cost. There is
nothing to amortize. This is the same reason Phase 3B renumbering measured a
0.00% delta: short identifiers already tokenize as one token.

## Decision

Per the Phase 3C checkpoint rule — "a negative result is documented and removed
rather than forced to remain" — the table was removed. The codec, its tests, the
`a3-tabled` measurement command, and `Measure-CompactA3Table.ps1` were deleted and
the working tree restored to the Phase 3B checkpoint. `A3_GRAMMAR.md` retains no
table grammar. Next lever: Phase 3D (corpus-backed defaults and final row
merging).
