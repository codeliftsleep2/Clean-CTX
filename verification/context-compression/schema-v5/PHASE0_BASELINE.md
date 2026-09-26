# SCHEMA-vNext current baseline

**Status:** Refreshed after production A1
**Captured:** 2026-09-26  
**Capture commit:** `db15d37c1106f88cf93194aa79b8f389f91a67ec`
**Tokenizer implementation:** `tiktoken-rs 0.12.0`  
**Model calls:** Zero

## Method

`Capture-Baselines.ps1` captured the real production-selected content for
Low, Medium, High, unfocused Edit, and focused Edit. The focused lane uses a
real `provide_code_context` request with a qualified `focusMethods` selector;
its model-visible response remains separate from the IR-bearing response used
to regenerate the code-side CONTROL-FULL oracle.

Candidate and oracle regeneration prefer the captured complete named IR in
`result.pretty`, decoded through the production wire-format detector. The
reduced hierarchical `result.ir` remains a compatibility fallback only; using
it as the primary source undercounted facts in the initial Phase 0 Edit
measurements.

`Measure-SchemaV5Anatomy.ps1` measures:

- the complete SCHEMA-v5 candidate;
- the representation production actually selected;
- fixed legend text;
- file/path framing;
- declarations and signatures;
- behavior facts;
- imports and type aliases;
- exact Edit bodies.

Family counts are independent diagnostic tokenizations. They are deliberately
non-additive because BPE merges can cross family boundaries. Whole-candidate
counts are authoritative for economics.

Generated detail remains under
`target/context-compression-verification/captures/` in
`schema-v5-token-records.json` and `schema-v5-anatomy-records.json`.

## Fixture provenance

| Language | Fixture SHA-256 |
|---|---|
| Angular | `94fb64a75e1130fd9199afc3c01c0e9c0e95bfe19902c7f0c980af727171f0c7` |
| C# | `1526963b10b77f2a89e904fce15c508a3266b92cdab39a718d5077803fdd9210` |
| TypeScript | `98ad5d1a8a58db220020e2944fe0836e24e715ca4d88df95df8afae3605570c5` |

The capture recorded a dirty worktree because the measurement harness and its
regressions were under development. The production binary hash, helper hash,
candidate hashes, and per-capture source hashes are retained in the generated
records and capture metadata.

## cl100k anatomy

| Language | Fidelity/focus | Complete | Legend | Path | Declarations | Facts | Imports | Bodies | Selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Angular | Edit/all | 4014 | 65 | 44 | 763 | 313 | 164 | 2687 | raw: 3912 |
| Angular | Edit/focused | 1530 | 65 | 44 | 758 | 313 | 164 | 203 | SCHEMA-v5: 1530 |
| Angular | High | 1425 | 65 | 44 | 743 | 409 | 181 | 0 | SCHEMA-v5: 1425 |
| Angular | Low | 940 | 65 | 44 | 413 | 313 | 131 | 0 | SCHEMA-v5: 940 |
| Angular | Medium | 1309 | 65 | 44 | 740 | 313 | 164 | 0 | SCHEMA-v5: 1309 |
| C# | Edit/all | 3963 | 65 | 47 | 1255 | 287 | 209 | 2134 | raw: 3590 |
| C# | Edit/focused | 2072 | 65 | 47 | 1250 | 287 | 209 | 243 | SCHEMA-v5: 2072 |
| C# | High | 1958 | 65 | 47 | 1250 | 393 | 232 | 0 | SCHEMA-v5: 1958 |
| C# | Low | 1258 | 65 | 47 | 749 | 287 | 152 | 0 | SCHEMA-v5: 1258 |
| C# | Medium | 1798 | 65 | 47 | 1232 | 287 | 209 | 0 | SCHEMA-v5: 1798 |
| TypeScript | Edit/all | 3183 | 65 | 44 | 299 | 149 | 267 | 2368 | raw: 2957 |
| TypeScript | Edit/focused | 1180 | 65 | 44 | 295 | 149 | 267 | 365 | SCHEMA-v5: 1180 |
| TypeScript | High | 907 | 65 | 44 | 295 | 209 | 299 | 0 | SCHEMA-v5: 907 |
| TypeScript | Low | 625 | 65 | 44 | 108 | 149 | 267 | 0 | SCHEMA-v5: 625 |
| TypeScript | Medium | 815 | 65 | 44 | 295 | 149 | 267 | 0 | SCHEMA-v5: 815 |

## o200k anatomy

| Language | Fidelity/focus | Complete | Legend | Path | Declarations | Facts | Imports | Bodies | Selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Angular | Edit/all | 4184 | 66 | 46 | 809 | 314 | 167 | 2805 | raw: 4091 |
| Angular | Edit/focused | 1594 | 66 | 46 | 803 | 314 | 167 | 214 | SCHEMA-v5: 1594 |
| Angular | High | 1478 | 66 | 46 | 787 | 410 | 185 | 0 | SCHEMA-v5: 1478 |
| Angular | Low | 983 | 66 | 46 | 449 | 314 | 134 | 0 | SCHEMA-v5: 983 |
| Angular | Medium | 1361 | 66 | 46 | 784 | 314 | 167 | 0 | SCHEMA-v5: 1361 |
| C# | Edit/all | 4189 | 66 | 49 | 1317 | 287 | 222 | 2287 | raw: 3826 |
| C# | Edit/focused | 2163 | 66 | 49 | 1311 | 287 | 222 | 257 | SCHEMA-v5: 2163 |
| C# | High | 2042 | 66 | 49 | 1311 | 400 | 245 | 0 | SCHEMA-v5: 2042 |
| C# | Low | 1264 | 66 | 49 | 748 | 287 | 157 | 0 | SCHEMA-v5: 1264 |
| C# | Medium | 1868 | 66 | 49 | 1287 | 287 | 222 | 0 | SCHEMA-v5: 1868 |
| TypeScript | Edit/all | 3288 | 66 | 46 | 308 | 150 | 273 | 2454 | raw: 3060 |
| TypeScript | Edit/focused | 1207 | 66 | 46 | 305 | 150 | 273 | 373 | SCHEMA-v5: 1207 |
| TypeScript | High | 926 | 66 | 46 | 305 | 210 | 306 | 0 | SCHEMA-v5: 926 |
| TypeScript | Low | 642 | 66 | 46 | 115 | 150 | 273 | 0 | SCHEMA-v5: 642 |
| TypeScript | Medium | 834 | 66 | 46 | 305 | 150 | 273 | 0 | SCHEMA-v5: 834 |

## Findings

1. **Focused Edit is economically and contractually distinct.** Production
   selected SCHEMA-v5 for every focused fixture, reducing o200k tokens versus
   raw by 61.04% (Angular), 43.47% (C#), and 60.56% (TypeScript). Unfocused
   Edit selected raw source because every complete candidate was larger than
   raw: 2.27% for Angular, 9.49% for C#, and 7.45% for TypeScript. This is the
   expected strict economics decision, not a separate safety-margin policy.
2. **Declarations/signatures are the dominant C# cost.** Independent o200k
   declaration/signature measurements are 59.2% of Low, 68.9% of Medium, and
   64.2% of High candidate tokens. This supports testing recurring method and
   signature grammar before marker-vocabulary changes.
3. **A1 removed the duplicated-path portion of fixed cold cost.** Legend plus
   path framing is now 112 o200k tokens for Angular/TypeScript and 115 for C#,
   down from 154 and 160 respectively. It represents 17.4% of TypeScript Low,
   11.4% of Angular Low, and 9.1% of C# Low. The remaining path family carries
   the visible alias and authoritative exact mapping and is not redundant.
4. **Imports/type aliases are material in TypeScript.** Their independent
   o200k count is 273 tokens at Low/Medium and 306 at High, or 42.5% of the
   TypeScript Low candidate. A3 requires the planned handle-meaning audit
   because the family total is not the removable handle cost.
5. **High behavior facts are meaningful but not the largest structural cost.**
   Relative to Medium, independent fact cost rises by 96 o200k tokens for
   Angular, 113 for C#, and 60 for TypeScript. Removing that payload wholesale
   would trade away the reasoning facts High exists to provide.
6. **A2 and A4 have no economics opportunity in this corpus.** None of the 15
   complete candidates contains a standalone `P` row. Promise return
   classification is already carried by signatures here, and there are no
   visible pattern-owner IDs to strip. A2/A4 must not claim savings until a
   qualifying pattern-rich fixture demonstrates an actual recurring cost; A4's
   adversarial ownership reasoning gates remain required independently.

## Phase result

The baseline is current after production A1. Its measured totals match the
isolated candidate exactly, and the presentation/workspace capture verifier
passed. A1 still requires live Claude acceptance. A3 follows after its semantic
audit; A2 and A4 remain blocked on a qualifying corpus opportunity rather than
implementation effort. B4 and Tier C remain deferred under the proposal's
stronger gates.
