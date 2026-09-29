# SCHEMA-vNext current baseline

**Status:** Refreshed after production A1 + A3
**Captured:** 2026-09-26  
**Capture commit:** `17e49bd2eb96e16521ce24289b275fa1eb34fa2e`
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

The production capture and measurement both recorded a clean worktree at the
same commit. The production binary hash, helper hash, candidate hashes, and
per-capture source hashes are retained in the generated records and capture
metadata.

## cl100k anatomy

| Language | Fidelity/focus | Complete | Legend | Path | Declarations | Facts | Imports | Bodies | Selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Angular | Edit/all | 4008 | 65 | 44 | 763 | 313 | 158 | 2687 | raw: 3912 |
| Angular | Edit/focused | 1524 | 65 | 44 | 758 | 313 | 158 | 203 | SCHEMA-v5: 1524 |
| Angular | High | 1419 | 65 | 44 | 743 | 409 | 175 | 0 | SCHEMA-v5: 1419 |
| Angular | Low | 934 | 65 | 44 | 413 | 313 | 125 | 0 | SCHEMA-v5: 934 |
| Angular | Medium | 1303 | 65 | 44 | 740 | 313 | 158 | 0 | SCHEMA-v5: 1303 |
| C# | Edit/all | 3939 | 65 | 47 | 1255 | 287 | 185 | 2134 | raw: 3590 |
| C# | Edit/focused | 2048 | 65 | 47 | 1250 | 287 | 185 | 243 | SCHEMA-v5: 2048 |
| C# | High | 1934 | 65 | 47 | 1250 | 393 | 208 | 0 | SCHEMA-v5: 1934 |
| C# | Low | 1234 | 65 | 47 | 749 | 287 | 128 | 0 | SCHEMA-v5: 1234 |
| C# | Medium | 1774 | 65 | 47 | 1232 | 287 | 185 | 0 | SCHEMA-v5: 1774 |
| TypeScript | Edit/all | 3143 | 65 | 44 | 299 | 149 | 227 | 2368 | raw: 2957 |
| TypeScript | Edit/focused | 1140 | 65 | 44 | 295 | 149 | 227 | 365 | SCHEMA-v5: 1140 |
| TypeScript | High | 867 | 65 | 44 | 295 | 209 | 259 | 0 | SCHEMA-v5: 867 |
| TypeScript | Low | 585 | 65 | 44 | 108 | 149 | 227 | 0 | SCHEMA-v5: 585 |
| TypeScript | Medium | 775 | 65 | 44 | 295 | 149 | 227 | 0 | SCHEMA-v5: 775 |

## o200k anatomy

| Language | Fidelity/focus | Complete | Legend | Path | Declarations | Facts | Imports | Bodies | Selected |
|---|---|---:|---:|---:|---:|---:|---:|---:|---|
| Angular | Edit/all | 4178 | 66 | 46 | 809 | 314 | 161 | 2805 | raw: 4091 |
| Angular | Edit/focused | 1588 | 66 | 46 | 803 | 314 | 161 | 214 | SCHEMA-v5: 1588 |
| Angular | High | 1472 | 66 | 46 | 787 | 410 | 179 | 0 | SCHEMA-v5: 1472 |
| Angular | Low | 977 | 66 | 46 | 449 | 314 | 128 | 0 | SCHEMA-v5: 977 |
| Angular | Medium | 1355 | 66 | 46 | 784 | 314 | 161 | 0 | SCHEMA-v5: 1355 |
| C# | Edit/all | 4165 | 66 | 49 | 1317 | 287 | 198 | 2287 | raw: 3826 |
| C# | Edit/focused | 2139 | 66 | 49 | 1311 | 287 | 198 | 257 | SCHEMA-v5: 2139 |
| C# | High | 2018 | 66 | 49 | 1311 | 400 | 221 | 0 | SCHEMA-v5: 2018 |
| C# | Low | 1240 | 66 | 49 | 748 | 287 | 133 | 0 | SCHEMA-v5: 1240 |
| C# | Medium | 1844 | 66 | 49 | 1287 | 287 | 198 | 0 | SCHEMA-v5: 1844 |
| TypeScript | Edit/all | 3248 | 66 | 46 | 308 | 150 | 233 | 2454 | raw: 3060 |
| TypeScript | Edit/focused | 1167 | 66 | 46 | 305 | 150 | 233 | 373 | SCHEMA-v5: 1167 |
| TypeScript | High | 886 | 66 | 46 | 305 | 210 | 266 | 0 | SCHEMA-v5: 886 |
| TypeScript | Low | 602 | 66 | 46 | 115 | 150 | 233 | 0 | SCHEMA-v5: 602 |
| TypeScript | Medium | 794 | 66 | 46 | 305 | 150 | 233 | 0 | SCHEMA-v5: 794 |

## Findings

1. **Focused Edit is economically and contractually distinct.** Production
   selected SCHEMA-v5 for every focused fixture, reducing o200k tokens versus
   raw by 61.18% (Angular), 44.09% (C#), and 61.86% (TypeScript). Unfocused
   Edit selected raw source because every complete candidate was larger than
   raw: 2.13% for Angular, 8.86% for C#, and 6.14% for TypeScript. This is the
   expected strict economics decision, not a separate safety-margin policy.
2. **Declarations/signatures are the dominant C# cost.** Independent o200k
   declaration/signature measurements are 60.3% of Low, 69.8% of Medium, and
   65.0% of High candidate tokens. This supports testing recurring method and
   signature grammar before marker-vocabulary changes.
3. **A1 removed the duplicated-path portion of fixed cold cost.** Legend plus
   path framing is now 112 o200k tokens for Angular/TypeScript and 115 for C#,
   down from 154 and 160 respectively. It represents 18.6% of TypeScript Low,
   11.5% of Angular Low, and 9.3% of C# Low. The remaining path family carries
   the visible alias and authoritative exact mapping and is not redundant.
4. **A3 removed only generated import identities.** The imports/type-alias
   family fell by exactly 6 Angular, 24 C#, and 40 TypeScript tokens in every
   lane under both tokenizers. Its remaining independent o200k cost is 233
   tokens at TypeScript Low/Medium and 266 at High; that payload carries real
   module, symbol, and source-written alias meaning and is not an elision
   target merely because the family remains large.
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

The baseline is current after production A1 + A3. A3's production totals match
its isolated prediction exactly in every row, and only the imports/type-alias
family changed. The schema-v5 production and workspace capture verifier passed.
A1 and A3 still require live Claude acceptance. A2 and A4 remain blocked on a
qualifying corpus opportunity rather than implementation effort. B4 and Tier C
remain deferred under the proposal's stronger gates.
