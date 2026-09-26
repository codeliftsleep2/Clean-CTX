# SCHEMA-vNext Phase 1 — A3 import-handle audit

**Status:** Pre-production audit, economics, and paired reasoning gates passed; production approval pending
**Audited:** 2026-09-26
**Production renderer:** Unchanged
**Model calls:** Eight paired Codex cases

## Decision under investigation

SCHEMA-v5 currently renders generated import identities such as:

```text
$ IM1 rxjs [Observable, of]
```

A3 tests whether the model-visible projection can omit only `IM1`:

```text
$ rxjs [Observable, of]
```

For an import whose canonical module field is empty, the same projection also
removes the now-meaningless empty field separator:

```text
$ IM1  [using System;]
$ [using System;]
```

Canonical `CoreOp::Import` identity, Binary `0x04`, hierarchical round trips,
delta identity, persistence, and replay remain unchanged.

## Code authority audit

- `emit_import_ir` creates the first `CoreOp::Import` field exclusively with
  `next_id("IM")`; it is not parsed from a source import alias.
- Source-written alias syntax remains inside the visible import payload (for
  example `Foo as Bar`) and is not represented by the generated `IMn` value.
- Hierarchical encode/decode, named/binary wire formats, identity validation,
  delta keys, persistence, and replay retain the generated handle code-side.
- The SCHEMA-v5 renderer emits `IMn` but no other visible record consumes it.

## Corpus assertion

The current complete economics corpus contains 15 candidates and 155 import
rows. It contains exactly 155 standalone `IMn` occurrences: every handle
appears once, solely in its defining `$` row, with zero cross-record
references.

`Measure-A3ImportHandles.ps1` repeats these assertions before transforming any
candidate. It aborts if a handle is non-unique, has any second visible
reference, or if the transform changes character count beyond the exact handle
plus separator removed from each import row. For an empty module field it also
asserts and removes the single separator that would otherwise leave a phantom
visible column.

## Token result

The finalized candidate saves the same absolute number of tokens under cl100k
and o200k:

| Language | Imports | Tokens saved | Relative range |
|---|---:|---:|---:|
| Angular | 3 | 6 | 0.14–0.64% |
| C# | 8 | 24 | 0.57–1.91% |
| TypeScript | 20 | 40 | 1.22–6.40% |

This clears the economics gate, especially for import-heavy TypeScript. The
empty-module cleanup improves C# by another eight tokens over the conservative
handle-only transform. Unfocused Edit remains governed by the existing
raw-versus-structured economics boundary.

## Reasoning result

All eight paired cases passed with no failures or unrun rows using fresh,
isolated invocations of the configured default Codex model:

- baseline and A3 preserved Angular module-to-symbol association;
- baseline and A3 preserved C# namespace-to-symbol association;
- baseline and A3 preserved TypeScript module-to-symbol association; and
- baseline and A3 both read `SharedName as ImportedShared` correctly as an
  exported name and its source-written local alias.

Generated answers and scoring remain under
`target/context-compression-verification/captures/`; they are experiment
evidence, not tracked-test evidence.

## Next gate

Obtain explicit approval for the externally visible SCHEMA-v5 grammar change.
If approved, implement A3 only in the presentation renderer, add tracked
contracts for generated-handle omission, empty-module spacing, and
source-written alias preservation, then refresh the production baseline and
complete the live Claude gate.
