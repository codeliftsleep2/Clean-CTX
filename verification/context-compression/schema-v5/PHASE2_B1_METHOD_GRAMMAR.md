# SCHEMA-vNext Phase 2 — B1 method grammar

**Status:** Economics and paired reasoning passed; production decision pending
**Recorded:** 2026-09-26
**Production renderer:** Unchanged
**Model calls:** 10 isolated paired-gate calls

## Candidate

B1 removes the first, scope-marking arrow from every method row. Parameters
remain explicitly introduced by `p:`, and the one remaining arrow introduces
the return type:

```text
M methodB  → → void
M find  → p:id:string → Order mod:PUBLIC ctl:RET
```

becomes:

```text
M methodB → void
M find p:id:string → Order mod:PUBLIC ctl:RET
```

This is a grammar revision, not a silent SCHEMA-v5 abbreviation. The measured
candidate therefore carries a complete cold header that changes the version to
`SCHEMA vNext`, defines `p:=params`, and redefines `→=return`. Measuring only
the deleted arrow while retaining the old `→=scope` legend would overstate the
win and evaluate a self-contradictory payload.

## Corpus audit

The current production economics corpus contains 15 complete candidates and
325 method rows:

- 232 begin a parameter region after the scope arrow;
- 93 proceed directly to the return arrow; and
- four constructor signatures continue their parameter payload across physical
  lines before reaching the return arrow.

Every method row has exactly one leading scope-arrow slot. The transform is
anchored to the physical `M ` row, so multiline parameter continuations,
return arrows, fact suffixes, and exact Edit bodies are outside its match.

## Measurement assertions

`Measure-B1MethodArrow.ps1` aborts unless:

- the exact current SCHEMA-v5 header occurs once;
- every method row has exactly one removable leading scope arrow;
- method-row count is unchanged;
- the candidate contains no old leading scope-arrow form;
- total arrow count falls by exactly one per method; and
- the candidate begins with the complete vNext legend.

The script writes generated candidates and token records beneath `target/` and
makes no model or Clean-CTX calls.

## Economics result

B1 saves a stable number of tokens under both measured tokenizers. The
complete legend change is included in every result.

| Fixture | Methods | Tokens saved | Best relative reduction |
| --- | ---: | ---: | ---: |
| Angular | 26 | 49 | 5.25% (cl100k Low) |
| C# | 30 | 57 | 4.62% (cl100k Low) |
| TypeScript | 9 | 15 | 2.56% (cl100k Low) |

Across non-Edit High output, the reductions are 3.45% Angular, 2.95% C#, and
1.73% TypeScript under cl100k; o200k reports 3.33%, 2.82%, and 1.69%.
Focused Edit saves 3.22%, 2.78%, and 1.32% under cl100k. All-body Edit gains
are smaller because exact source bodies dominate those payloads.

The equal absolute savings across cl100k and o200k are useful evidence that
B1 is a structural reduction rather than a tokenizer-specific spelling trick.

## Paired reasoning gate

`Prepare-B1ReasoningWorksheet.ps1` creates five semantic cases, each evaluated
once against the captured SCHEMA-v5 baseline and once against the exact B1
candidate:

- parameter and return binding;
- a parameterless method;
- a multiline constructor signature;
- same-written-name methods under different owners; and
- a focused Edit signature plus exact-body binding.

The gate passed 10/10 with no failure categories:

| Semantic case | SCHEMA-v5 | B1 candidate |
| --- | --- | --- |
| Parameter and return binding | Pass | Pass |
| Parameterless method | Pass | Pass |
| Multiline constructor | Pass | Pass |
| Same method name under class/interface owners | Pass | Pass |
| Focused Edit body/signature binding | Pass | Pass |

B1 therefore clears its isolated laboratory gate. Production remains unchanged;
adopting the versioned grammar still requires an explicit implementation
decision and the production integration, regression, documentation, and live
verification work described in the proposal.
