# SCHEMA-vNext Phase 1 — A1 single-path experiment

**Status:** Production and refreshed baseline verified; live gate pending
**Measured:** 2026-09-26
**Production renderer:** A1 implemented at the shared content boundary
**Model calls:** Six paired Codex cases

## Candidate

A1 removes only the duplicated full path from the decorative file footer:

```text
// ── α1 (C:\workspace\src\service.ts) ──
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

becomes:

```text
// α1
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

The request-scoped alias remains visible and `§PATHMAP` remains the single
authoritative exact-path mapping.

## Method

`Measure-A1SinglePath.ps1` transforms each complete Phase 0 candidate and
rejects the result unless:

- exactly one decorative file footer existed;
- exactly one mapping existed for that footer's alias;
- no duplicated-path footer remains;
- the authoritative mapping remains exactly once; and
- restoring the one replaced substring reproduces the baseline byte-for-byte.

Generated per-row evidence is retained in
`target/context-compression-verification/captures/schema-vnext-a1-records.json`.

## Token result

The saving is stable for every fidelity because the removed footer is fixed
per fixture.

| Language | cl100k saved | o200k saved | Lowest relative win | Highest relative win |
|---|---:|---:|---:|---:|
| Angular | 40 | 42 | 0.99% (Edit/all) | 4.10% (Low) |
| C# | 43 | 45 | 1.06% (Edit/all) | 3.44% (Low) |
| TypeScript | 40 | 42 | 1.26% (Edit/all) | 6.14% (Low) |

Focused Edit improves by 2.04–3.36% under o200k. Structural High improves by
2.16–4.34%, Medium by 2.35–4.79%, and Low by 3.44–6.14%.

A1 does not make unfocused Edit economical: those complete candidates remain
2.27–9.49% larger than raw under o200k, so production should continue selecting
byte-exact raw source for that lane.

## Decision

A1 passes the economics gate and deterministic candidate-isolation gate. The
win is consistent across both measured tokenizers, removes genuine duplicate
text, and retains the exact path at its authoritative mapping boundary.

A1 passed the pre-production gates and received explicit approval for the
externally visible footer change. Completion requires:

1. add a model-reasoning case that resolves the visible alias through
   `§PATHMAP` and rejects a missing or invented path (**implemented as three
   paired baseline/candidate language cases**);
2. run the candidate reasoning gate with the same model/version as its
   baseline (**passed 6/6: three baseline and three candidate cases, using the
   configured default Codex model in fresh isolated invocations**);
3. implement the production change with tracked contract coverage
   (**implemented; focused contract reported GREEN**);
4. verify the changed footer in the live Claude pilot.

## Reasoning result

The paired gate passed all six cases with no failures or unrun rows. For each
language, baseline and A1 produced the same exact alias-to-path answer and
identified `§PATHMAP` as authoritative. Generated answers and scoring remain
under `target/context-compression-verification/captures/`; they are experiment
evidence, not tracked-test evidence.

## Production verification

The focused footer contract and registered provide/compress presentation suite
were reported GREEN. The complete production capture was then refreshed:

- SCHEMA-v5 and byte-exact economic raw fallbacks passed capture verification;
- all six `workspace_query` oracle captures were present;
- post-A1 candidate totals matched the isolated A1 prediction exactly; and
- only the path-framing family changed (40–45 tokens per response).

The current post-A1 numbers are recorded in `PHASE0_BASELINE.md`. The remaining
completion boundary is live Claude acceptance on the pilot workspace.
