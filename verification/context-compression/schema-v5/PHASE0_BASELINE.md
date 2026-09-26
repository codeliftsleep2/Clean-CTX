# SCHEMA-vNext Phase 0 baseline

**Status:** Complete  
**Captured:** 2026-09-26  
**Capture commit:** `2f13a7536f4b88e169a2c7638d6e7e1b49c87dd1`
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
| Angular | Edit/all | 4054 | 65 | 84 | 763 | 313 | 164 | 2687 | raw: 3912 |
| Angular | Edit/focused | 1570 | 65 | 84 | 758 | 313 | 164 | 203 | SCHEMA-v5: 1570 |
| Angular | High | 1465 | 65 | 84 | 743 | 409 | 181 | 0 | SCHEMA-v5: 1465 |
| Angular | Low | 980 | 65 | 84 | 413 | 313 | 131 | 0 | SCHEMA-v5: 980 |
| Angular | Medium | 1349 | 65 | 84 | 740 | 313 | 164 | 0 | SCHEMA-v5: 1349 |
| C# | Edit/all | 4006 | 65 | 90 | 1255 | 287 | 209 | 2134 | raw: 3590 |
| C# | Edit/focused | 2115 | 65 | 90 | 1250 | 287 | 209 | 243 | SCHEMA-v5: 2115 |
| C# | High | 2001 | 65 | 90 | 1250 | 393 | 232 | 0 | SCHEMA-v5: 2001 |
| C# | Low | 1301 | 65 | 90 | 749 | 287 | 152 | 0 | SCHEMA-v5: 1301 |
| C# | Medium | 1841 | 65 | 90 | 1232 | 287 | 209 | 0 | SCHEMA-v5: 1841 |
| TypeScript | Edit/all | 3223 | 65 | 84 | 299 | 149 | 267 | 2368 | raw: 2957 |
| TypeScript | Edit/focused | 1220 | 65 | 84 | 295 | 149 | 267 | 365 | SCHEMA-v5: 1220 |
| TypeScript | High | 947 | 65 | 84 | 295 | 209 | 299 | 0 | SCHEMA-v5: 947 |
| TypeScript | Low | 665 | 65 | 84 | 108 | 149 | 267 | 0 | SCHEMA-v5: 665 |
| TypeScript | Medium | 855 | 65 | 84 | 295 | 149 | 267 | 0 | SCHEMA-v5: 855 |

## o200k anatomy

| Language | Fidelity/focus | Complete | Legend | Path | Declarations | Facts | Imports | Bodies | Selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Angular | Edit/all | 4226 | 66 | 88 | 809 | 314 | 167 | 2805 | raw: 4091 |
| Angular | Edit/focused | 1636 | 66 | 88 | 803 | 314 | 167 | 214 | SCHEMA-v5: 1636 |
| Angular | High | 1520 | 66 | 88 | 787 | 410 | 185 | 0 | SCHEMA-v5: 1520 |
| Angular | Low | 1025 | 66 | 88 | 449 | 314 | 134 | 0 | SCHEMA-v5: 1025 |
| Angular | Medium | 1403 | 66 | 88 | 784 | 314 | 167 | 0 | SCHEMA-v5: 1403 |
| C# | Edit/all | 4234 | 66 | 94 | 1317 | 287 | 222 | 2287 | raw: 3826 |
| C# | Edit/focused | 2208 | 66 | 94 | 1311 | 287 | 222 | 257 | SCHEMA-v5: 2208 |
| C# | High | 2087 | 66 | 94 | 1311 | 400 | 245 | 0 | SCHEMA-v5: 2087 |
| C# | Low | 1309 | 66 | 94 | 748 | 287 | 157 | 0 | SCHEMA-v5: 1309 |
| C# | Medium | 1913 | 66 | 94 | 1287 | 287 | 222 | 0 | SCHEMA-v5: 1913 |
| TypeScript | Edit/all | 3330 | 66 | 88 | 308 | 150 | 273 | 2454 | raw: 3060 |
| TypeScript | Edit/focused | 1249 | 66 | 88 | 305 | 150 | 273 | 373 | SCHEMA-v5: 1249 |
| TypeScript | High | 968 | 66 | 88 | 305 | 210 | 306 | 0 | SCHEMA-v5: 968 |
| TypeScript | Low | 684 | 66 | 88 | 115 | 150 | 273 | 0 | SCHEMA-v5: 684 |
| TypeScript | Medium | 876 | 66 | 88 | 305 | 150 | 273 | 0 | SCHEMA-v5: 876 |

## Findings

1. **Focused Edit is economically and contractually distinct.** Production
   selected SCHEMA-v5 for every focused fixture, reducing o200k tokens versus
   raw by 60.01% (Angular), 42.29% (C#), and 59.18% (TypeScript). Unfocused
   Edit selected raw source because every complete candidate was larger than
   raw: 3.30% for Angular, 10.66% for C#, and 8.82% for TypeScript. This is the
   expected strict economics decision, not a separate safety-margin policy.
2. **Declarations/signatures are the dominant C# cost.** Independent o200k
   declaration/signature measurements are 57.1% of Low, 67.3% of Medium, and
   62.8% of High candidate tokens. This supports testing recurring method and
   signature grammar before marker-vocabulary changes.
3. **Fixed cold cost matters most at Low.** Legend plus path framing is 154
   o200k tokens for Angular/TypeScript and 160 for C#. It represents 22.5% of
   TypeScript Low, 15.0% of Angular Low, and 12.2% of C# Low. Only part of the
   path family is removable; A1 must isolate the duplicated path before making
   a savings claim.
4. **Imports/type aliases are material in TypeScript.** Their independent
   o200k count is 273 tokens at Low/Medium and 306 at High, or 39.9% of the
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

Phase 0 is complete. The first measured Tier A work should isolate A1's single
duplicated-path removal. A3 follows after its semantic audit. A2 and A4 are
blocked on a qualifying corpus opportunity rather than implementation effort.
B4 and Tier C remain deferred under the proposal's stronger gates.
