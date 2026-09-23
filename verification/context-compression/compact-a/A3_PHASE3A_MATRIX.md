# COMPACT-A3 Phase 3A production-scope measurement matrix

**Status:** harness implemented; measurement run pending (zero model calls)
**Predecessor:** Phase 3 economics failed at 26.99% o200k aggregate; Phase 4 blocked
**Scope:** zero-model-call capture/measurement only. No grammar, codec, identity,
production-selection, or canonical-IR changes.

## Purpose

`PHASED_A3_PLAN.md` Checkpoint 3A requires, before any further grammar lever,
that every qualifying large fixture be captured at Low, Medium, High, focused
Edit, and all-body Edit; that TypeScript, Angular, and C#/.NET each contribute a
qualifying large fixture; that every fidelity/focus row is visible; and that the
current baseline and semantic-family token anatomy are frozen. This document
defines the corpus, the matrix, the production-selected aggregate formula, and
the reproduction procedure.

## Corpus

Qualifying fixtures are the tracked source files of at least 8 KiB under
`src/test_files/`. The primary production scopes are covered as follows:

| Language | Fixture | Path |
|---|---|---|
| typescript | `LargeService.ts` | `src/test_files/LargeService.ts` |
| angular | `UserManagementService.ts` | `src/test_files/UserManagementService.ts` |
| csharp | `OrderManagementService.cs` | `src/test_files/dotnet/OrderManagementService.cs` |

`OrderManagementService.cs` is a new synthetic ASP.NET Core order-management
fixture added in this phase because no tracked C#/.NET file previously met the
8 KiB screen (the largest was `dotnet/MultiClassFixture.cs` at ~4.2 KiB). Java
remains secondary and is not part of the required aggregate.

## Matrix

Each qualifying fixture is captured once at Edit fidelity (so the checked IR
carries bodies), then re-rendered as CONTROL-FULL at every fidelity/focus
combination below. Each row is an independent capture directory with its own
`control-full.txt`, `raw-source.txt`, `capture-meta.json`, and A3 candidate.

| Fidelity | Focus mode | Focus target |
|---|---|---|
| low | none | — |
| medium | none | — |
| high | none | — |
| edit | all-bodies | — |
| edit | focused | `UserService.createUser` / `UserManagementService.createUser` / `OrderService.CreateOrderAsync` |

Focused Edit uses a qualified `Owner.method` selector so it remains unambiguous
even where an interface shares the method name. Focused Edit is reported
separately from all-body Edit; only all-body Edit may legitimately select raw.

## Frozen baseline (pre-Phase 3A, high fidelity only)

These are the numbers recorded before this matrix existed, reproduced from the
two tracked TypeScript/Angular fixtures at high fidelity:

| Tokenizer | Raw aggregate | A3 aggregate | Reduction |
|---|---:|---:|---:|
| cl100k | 6,869 | 5,150 | 25.03% |
| o200k | 7,151 | 5,221 | 26.99% |

Individual o200k diagnostics: 38.20% for `LargeService.ts`, 18.60% for
`UserManagementService.ts`. The legend is about 120–130 tokens; cost is
concentrated in calls, fields, methods, parameters, imports, and types.

## Production-selected aggregate formula

Computed independently for each exact tokenizer (cl100k, o200k):

```text
selected(i) = candidate(i) when candidate_tokens(i) < raw_tokens(i), else raw(i)
aggregate reduction = 1 - sum(selected(i)) / sum(raw(i))
```

Exact local counters use a strict `candidate < raw` comparison (raw wins ties).
The calibrated Claude safety buffer is a production concern
(`src/mcp/content_economics.rs`) and is not applied to the exact cl100k/o200k
research counts.

## Semantic-family anatomy

Each candidate is additionally decomposed into four token-anatomy fragments via
`measure a3-anatomy` — `legend`, `declarations`, `facts`, and `bodies` — so the
marginal cost of each family is visible per fidelity, focus, language, and
tokenizer. Fragment counts are independent measurements and need not sum to the
whole payload (BPE merges can cross fragment boundaries).

## Reproduction (hand off to the operator)

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Build-MeasureHelper.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Capture-Baselines.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/context-compression/scripts/Measure-CompactA3.ps1
```

`Measure-CompactA3.ps1` writes `target/context-compression-verification/captures/compact-a3-token-records.json`
and prints the per-language/per-fidelity/per-focus rows plus the
production-selected aggregate. Zero model calls are made.

## Measured results (final, direct per-fidelity capture)

Production-selected aggregate: **o200k 19.30%** (raw 54,885 → selected 44,292; 10/15 economical);
**cl100k 18.32%** (raw 52,295 → selected 42,717; 10/15 economical).

o200k per fixture (`raw → A3`, reduction%):

| Fixture (language) | low | medium | high | edit all-bodies | edit focused |
|---|---|---|---|---|---|
| LargeService.ts (typescript) | 3060→1409 (53.95%) | 3060→1673 (45.33%) | 3060→1891 (38.20%) | 3060→4451 (-45.46%) | 3060→2261 (26.11%) |
| UserManagementService.ts (angular) | 4091→2438 (40.41%) | 4091→2993 (26.84%) | 4091→3330 (18.60%) | 4091→6557 (-60.28%) | 4091→3638 (11.07%) |
| OrderManagementService.cs (csharp) | 3826→2521 (34.11%) | 3826→3509 (8.29%) | 3826→3933 (-2.80%) | 3826→6628 (-73.24%) | 3826→4309 (-12.62%) |

cl100k per fixture (`raw → A3`, reduction%):

| Fixture (language) | low | medium | high | edit all-bodies | edit focused |
|---|---|---|---|---|---|
| LargeService.ts (typescript) | 2957→1385 (53.16%) | 2957→1655 (44.03%) | 2957→1872 (36.69%) | 2957→4340 (-46.77%) | 2957→2236 (24.38%) |
| UserManagementService.ts (angular) | 3912→2392 (38.85%) | 3912→2942 (24.80%) | 3912→3278 (16.21%) | 3912→6359 (-62.55%) | 3912→3572 (8.69%) |
| OrderManagementService.cs (csharp) | 3590→2439 (32.06%) | 3590→3307 (7.88%) | 3590→3741 (-4.21%) | 3590→6256 (-74.26%) | 3590→4098 (-14.15%) |

## Findings

- The production-selected aggregate is 18.32% (cl100k) / 19.30% (o200k), far below
  the 50% gate. This is the Phase 3A baseline: it measures, Phases 3B–3E optimize.
- Low/Medium/High are economical for typescript and angular (typescript Low is best
  at ~54%). C# high is slightly negative (-2.80% o200k / -4.21% cl100k), confirming
  C# is a harder scope.
- Edit all-bodies inflates massively (-45% to -73%) and correctly selects raw.
  Focused Edit is positive only for typescript (24–26%).
- **Baseline reproduced exactly.** The high rows now match the frozen pre-Phase 3A
  baseline: typescript high o200k = 1,891 (38.20%) and angular high o200k = 3,330
  (18.60%), totaling 5,221 o200k (26.99%) / 5,150 cl100k (25.03%). The earlier ~1%
  drift came from re-rendering Low/Medium/High from a single Edit capture; compiling
  each fidelity directly resolves it (edit rows are unchanged, as expected).

## Checkpoint 3A status

- [x] Every primary language (typescript, angular, csharp) has a qualifying fixture.
- [x] Every fidelity/focus row is visible (15 rows; no blended-only report).
- [ ] Body-bearing rows verify exact UTF-8 bytes and numeric spans (covered by the
  tracked roundtrip tests and `Verify-Captures.ps1`; pending that verify run).
- [x] Harness makes zero model calls and does not modify production behavior.
- [x] Baseline numbers and anatomy are frozen (reproduces the 25.03% cl100k /
  26.99% o200k high-fidelity baseline exactly).
