# SCHEMA-vNext Phase 2 — B2 class record

**Status:** Economics and paired reasoning passed; production decision pending
**Recorded:** 2026-09-26
**Production renderer:** Unchanged
**Model calls:** 12 isolated paired-gate calls

## Candidate

B2 replaces the decorative class-boundary comment with an explicit typed
record:

```text
// ── OrderService ──
```

becomes:

```text
C OrderService
```

This is a grammar revision rather than an undocumented abbreviation. The
candidate carries a complete cold header that changes the version to
`SCHEMA vNext` and adds `C=class`. It otherwise retains the SCHEMA-v5 legend,
including `→=scope`, so B2 is measured independently from B1.

Interfaces are already typed with `Q` rows and are outside B2. The conditional
`// Q=interface` legend remains unchanged; folding it into the cold header or
removing it would be a separate candidate.

## Corpus audit

The current production economics corpus contains 15 complete candidates and
85 class-boundary records:

| Fixture family | Candidates | Class rows per candidate | Total class rows |
| --- | ---: | ---: | ---: |
| Angular | 5 | 1 | 5 |
| C# | 5 | 15 | 75 |
| TypeScript | 5 | 1 | 5 |

All 85 boundaries use the exact logical form `// ── <payload> ──`. Four C#
record payloads span multiple physical lines in each C# candidate. The
transform is anchored from the boundary-opening line through its closing marker,
so schema headers, interface legends, interfaces, path aliases, comments inside
exact Edit bodies, and all class contents remain outside its match.

Several C# record declarations currently reach the renderer as declaration-like
class-name payloads. B2 preserves each payload byte-for-byte and changes only
its boundary marker. Correcting declaration identity would be a separate
compiler/extractor change and must not be hidden inside this presentation
experiment.

## Measurement assertions

`Measure-B2ClassRecord.ps1` aborts unless:

- the exact current SCHEMA-v5 header occurs once;
- every single-line or multiline decorative class boundary has a non-empty
  payload;
- at least one class boundary exists;
- class-boundary count is unchanged by the transform;
- no decorative class boundary remains;
- every captured payload appears as a `C` record without byte changes;
- interface rows and the conditional interface legend are byte-identical; and
- the candidate begins with the complete B2 vNext legend.

The script writes generated candidates and token records beneath `target/` and
makes no model or Clean-CTX calls.

## Next gate

The complete versioned candidate passes the economics gate:

| Fixture | Classes | cl100k saved | o200k saved | Best reduction |
| --- | ---: | ---: | ---: | ---: |
| Angular | 1 | 3 | 3 | 0.32% |
| C# | 15 | 95 | 84–87 | 7.70% |
| TypeScript | 1 | 3 | 3 | 0.51% |

The C# o200k all-body Edit result saves 84 rather than 87 tokens because BPE
boundaries interact with the surrounding exact-body payload. All other C#
o200k lanes save 87. The candidate remains positive in every lane and under
both tokenizers.

`Prepare-B2ReasoningWorksheet.ps1` creates six semantic cases, each evaluated
against the captured SCHEMA-v5 baseline and the exact B2 candidate. They cover
single-class ownership, multi-class ownership, multiline record declarations,
inheritance, class/interface distinction, and focused Edit body ownership.

The gate passed 12/12 with no failure categories:

| Semantic case | SCHEMA-v5 | B2 candidate |
| --- | --- | --- |
| Angular single-class ownership | Pass | Pass |
| TypeScript single-class ownership | Pass | Pass |
| C# multi-class field ownership | Pass | Pass |
| C# multiline record declaration | Pass | Pass |
| Inheritance and class/interface distinction | Pass | Pass |
| Focused Edit body ownership | Pass | Pass |

B2 therefore clears its isolated laboratory gate. Production remains unchanged;
adopting the versioned grammar still requires an explicit implementation
decision and the production integration, regression, documentation, and live
verification work described in the proposal.
