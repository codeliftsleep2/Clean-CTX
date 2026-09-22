# A3 Phase 0 baseline

**Status:** complete
**Model calls:** zero
**Source:** captured byte-exact raw files and locally counted A2 payloads

## Reproduced economics

| Tokenizer | Lane | Captures | Raw | A2 | Reduction | Economical |
|---|---|---:|---:|---:|---:|---:|
| cl100k | correctness/lifecycle | 41 | 5,757 | 31,836 | -453.00% | 0/41 |
| cl100k | tracked economics | 2 | 6,869 | 9,581 | -39.48% | 0/2 |
| o200k | correctness/lifecycle | 41 | 5,773 | 32,263 | -458.86% | 0/41 |
| o200k | tracked economics | 2 | 7,151 | 9,839 | -37.59% | 0/2 |

Small correctness fixtures establish exactness and lifecycle behavior but do
not determine production economics. The two tracked files exceed the 8 KiB
screen and are the current economic evidence.

## Tracked-file anatomy (o200k)

| Component | `LargeService.ts` | `UserManagementService.ts` | Combined |
|---|---:|---:|---:|
| Raw source | 3,060 | 4,091 | 7,151 |
| Complete A2 | 3,610 | 6,229 | 9,839 |
| Header + legend | 147 | 147 | 294 |
| File/mode | 84 | 85 | 169 |
| Declarations | 1,012 | 2,882 | 3,894 |
| Local calls | 1,409 | 2,169 | 3,578 |
| Navigation | 307 | 523 | 830 |
| Imports | 527 | 86 | 613 |
| Type aliases | 62 | 276 | 338 |
| Body frames | 0 | 0 | 0 |

Component counts are independent tokenizer measurements and need not sum to
the complete payload because BPE merges can cross component boundaries.

## Finding

The combined legend is 294 tokens, only 3.0% of the complete tracked A2
payload. Declarations and local calls total 7,472 independently counted tokens
and dominate the failure. Navigation costs another 830 tokens even though it
contains only locators.

The evidence supports the A3 design:

- fully positional nested declaration records;
- sparse fidelity-specific facts instead of fixed empty columns;
- run-length caller grouping with implicit call occurrence order and defaults;
- no initial navigation section;
- positional imports and aliases;
- exact bodies framed separately only for Edit.

SCHEMA v5 remains useful diagnostic evidence: on the same o200k fixtures it
used 1,080 and 1,804 tokens. A3 must retain normalized identity/correctness while
recovering scoped-text density.

## Gate result

Phase 0 passes as an investigation/specification checkpoint:

- A2 failure is reproducible;
- legend cost is below the A3 200-token target per cold payload;
- dominant cost families are identified;
- A3 grammar and fidelity target are explicit;
- production still emits no A3 payload.

This does not approve A3 production integration or model evaluation. Phase 1
must first prove deterministic roundtrip over the normalized fixtures.
