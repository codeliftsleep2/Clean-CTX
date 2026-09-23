# COMPACT-A3 Phase 3D — corpus-backed defaults and final row merging

**Status:** complete — one measured win applied (single-value count elision)
**Predecessor:** Phase 3C (scoped type/callee table, negative, removed)
**Outcome:** no implied default was viable; single-value count elision raised the
production-selected aggregate from 18.32% to 19.56% cl100k and 19.30% to 20.48%
o200k (+1.24% / +1.18%).

## Part 1 — corpus-backed defaults (no viable default)

Frequencies measured over the 325 methods of the three tracked-economics fixtures
(high-fidelity CONTROL-FULL, zero model calls):

| Family | with groups | groups | empty | dominant shape/values |
|---|---:|---:|---:|---|
| `lf` legacy flags | 0 / 325 | 0 | 0 | always empty |
| `mo` modifiers | 255 / 325 | 255 | 0 | single-value: `ASYNC` 155, `PRIVATE` 90, `PROTECTED` 5, `[PRIVATE,STATIC]` 5; 70 no-modifier |
| `cs` control summaries | 245 / 325 | 245 | 0 | variable sequences of `IF`/`RET`/`THROW`/`LOOP` |
| `pf` pattern facts | 147 / 325 | 147 | 0 | single-fact: `OBSERVABLE` 137, `CTOR` 10; 178 no-fact |

**Finding:** no "implied default" qualified. `lf` is already free (zero methods
emit it). `mo`/`cs`/`pf` each have three or more genuinely distinct states with no
single dominant value that can be made the default without collapsing distinct
semantics (e.g. `pf`: no-fact vs `OBSERVABLE` vs `CTOR`). There are also zero empty
groups, so no empty-group default was available. Per the checkpoint rule — an
implied default only where the corpus demonstrates a dominant value and the grammar
reconstructs it exactly — nothing was introduced.

## Part 2 — row merging (single-value count elision)

The audit found one structural regularity worth exploiting: occurrence/fact groups
(`cm`, `cf`, `mo`, `cs`, `lf`, `pf`) are single-value in the dominant cases (`mo`
250/255, `pf` 147/147, `cm`/`cf` uniformly single-value in the fixtures). The
leading count column is therefore redundant for those rows.

**Change:** single-value groups elide the count.

- Encoder: a group with exactly one non-numeric value/fact emits `mo|ASYNC` /
  `pf|OBSERVABLE` instead of `mo|1|ASYNC` / `pf|1|OBSERVABLE`. Empty (`mo|0`) and
  multi-value (`mo|2|…`) groups keep the explicit count. A single **numeric** value
  (e.g. `"42"`) is deliberately not elided so it can never be read as a count.
- Decoder: a leading bare unsigned integer is the count; any other first column is
  a single elided value/fact. This is backward-compatible — `mo|1|PUBLIC` and
  `mo|PUBLIC` decode identically.
- Legend/grammar: the cold legend appends `1 value omits count`; `A3_GRAMMAR.md`
  documents the rule.

## Measured delta

Production-selected aggregate (tracked-economics lane, 15 rows):

| Tokenizer | Before (Phase 3A) | After (Phase 3D) | Δ |
|---|---:|---:|---:|
| cl100k | 18.32% (raw 52,295 → 42,717) | 19.56% (raw 52,295 → 42,067) | +1.24% |
| o200k | 19.30% (raw 54,885 → 44,292) | 20.48% (raw 54,885 → 43,642) | +1.18% |

Roughly 650 selected tokens saved across the 15 rows, offset by the one-line legend
change on each cold payload.

## Checkpoint 3D

- [x] Every default is justified by recorded primary-corpus frequency data (Part 1
  found none viable, so no default was added).
- [x] No semantic value, empty group, duplicate, or boundary is discarded (the
  elision is purely a spelling change; both spellings decode identically, and the
  numeric-value guard keeps counts unambiguous).
- [x] Only measured token wins remain in the grammar (the elision is the only
  change; it is a measured +1.2% win).
- [x] All modified/new files remain within the active-file ceiling
  (`declarations.rs` is 602 lines).

## Next

Phase 3E — economics decision: re-run the full matrix and gate the
production-selected aggregate against 50%. Current 19.56% / 20.48% is expected to
fall short, so continue only while another lossless, reasoning-safe measured win
remains.
