# SCHEMA-vNext Phase 1 — A3 import-handle audit

**Status:** Production implemented; focused RED/GREEN passed; broader verification, refreshed baseline, and live gate pending
**Audited:** 2026-09-26
**Production renderer:** A3 implemented in the model-facing import projection
**Model calls:** Eight paired Codex cases

## Decision under investigation

Before A3, SCHEMA-v5 rendered generated import identities such as:

```text
$ IM1 rxjs [Observable, of]
```

A3 changes the model-visible projection to omit only `IM1`:

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
- The pre-A3 SCHEMA-v5 renderer emitted `IMn`, but no other visible record
  consumed it. The production renderer now omits that field only.

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

## Production implementation and RED/GREEN result

The approved implementation changes only `render_hierarchical_for_llm` import
formatting. Canonical `CoreOp::Import` operands and every code-side consumer
remain unchanged.

Three tracked contracts were observed RED against the old renderer, stashed,
restored unchanged after implementation, and reported GREEN:

- generated handles are omitted while modules and named symbols remain;
- an empty module renders without a phantom whitespace column; and
- `SharedName as ImportedShared` remains intact in the named payload.

Existing wildcard, full-class, meta-layer, ordering, and MCP renderer
expectations were updated only where they encoded the superseded visible
spelling.

## Next gate

Run the broader renderer and MCP presentation suites, then refresh the complete
production capture and compression baseline. Completion still requires the
live Claude pilot gate.
