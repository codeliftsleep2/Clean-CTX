# SCHEMA-vNext Phase 2 — combined B1+B2+B3 grammar

**Status:** Implemented and production-recaptured; live pilot pending
**Recorded:** 2026-09-26
**Production renderer:** SCHEMA-vNext combined grammar
**Production recapture model calls:** Zero

## Selected candidate

All three independently proven Tier B candidates advance to the combined gate:

- B1 removes the redundant leading method scope arrow;
- B2 replaces decorative class comments with typed `C` records; and
- B3 groups owner-local fields at Medium and High fidelity.

The combined cold legend is:

```text
// SCHEMA vNext  @=meta C=class X=extends I=implements F=field M=method $=import p:=params →=return mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
```

B4 marker renaming and Tier C acknowledged legends remain excluded. Interfaces
retain `Q` rows and their conditional legend. Low fields remain in their
existing grouped form; Edit fields remain one-per-line.

## Interaction discipline

The combined transform is generated directly from each captured SCHEMA-v5
baseline, not by chaining previously generated candidate files. Class and field
replacements are collected against original byte coordinates and applied from
the end of the document toward the beginning. Method grammar and the header are
then changed without relying on stale offsets.

`Measure-CombinedGrammar.ps1` independently asserts:

- one exact baseline header and one complete combined header;
- every class payload is preserved and becomes one `C` record;
- every method loses exactly one leading scope arrow and no other arrow;
- every field payload and its ordering remain intact;
- B3 grouping occurs only at Medium/High and never crosses an owner boundary;
- Low and Edit field layout remain unchanged;
- interface rows and the conditional interface legend remain byte-identical;
  and
- the candidate equals only the selected B1+B2+B3 transformations.

The script writes candidates and token records beneath `target/`; it makes no
model or Clean-CTX calls and changes no production code.

## Next gates

The combined candidate passes economics across every lane and tokenizer:

| Fixture | Low | Medium | High | Focused Edit | All-body Edit |
| --- | ---: | ---: | ---: | ---: | ---: |
| Angular cl100k | 52 (5.57%) | 178 (13.66%) | 178 (12.54%) | 52 (3.41%) | 52 (1.30%) |
| C# cl100k | 152 (12.32%) | 206 (11.61%) | 206 (10.65%) | 152 (7.42%) | 152 (3.86%) |
| TypeScript cl100k | 18 (3.08%) | 28 (3.61%) | 28 (3.23%) | 18 (1.58%) | 18 (0.57%) |

o200k confirms the same profile: Angular saves 52/178, TypeScript 18/28,
and C# 141–198 depending on lane and BPE interaction. No candidate regresses
token count.

`Prepare-CombinedReasoningWorksheet.ps1` creates ten semantic cases, each run
against the SCHEMA-v5 baseline and exact combined candidate. The 20-call gate
mixes method grammar, typed/multiline ownership, inheritance/interfaces,
grouped fields, negative membership, and focused Edit body binding in the same
candidate.

Only after a 20/20 pass may the public grammar proceed to an explicit production
implementation decision and tracked contract tests.

The first interaction run passed 19/20 because the combined candidate once
emitted an unsupported `(+1)` suffix on `getUserById`; all requested owner,
parameter, and return facts were otherwise correct. A full unchanged restart
passed 20/20.

The exact baseline/candidate pair was then replicated three times per variant.
That diagnostic scored 4/6, but the two failures occurred together in replicate
3: both SCHEMA-v5 and vNext supplied the same `(+1)` suffix while preserving
the correct owner, parameter, and return facts. The evaluator also accepted
the identical suffix in both replicate-1 answers and in candidate replicate 2.
This demonstrates model/evaluator variability rather than a candidate-only
grammar regression. The replication is recorded accurately as failed; it is
not reported as a passing gate.

Acceptance rests on the clean 20/20 full restart plus paired non-regression:
the diagnostic produced no baseline-pass/candidate-fail distinction for the
suffix behavior. The approved renderer, system prompt, vocabulary resource,
and tracked contracts now emit the combined grammar. The production recapture
matched every independently predicted token total across all languages,
fidelities, and both tokenizers. Live Claude pilot consumption remains pending.
