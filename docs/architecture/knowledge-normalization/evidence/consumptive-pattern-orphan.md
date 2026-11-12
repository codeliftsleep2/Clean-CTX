# Historical finding DIS-2026-002

Agent: **Agent MaxHeadRoom**, B6, 2026-11-12.

The exact original ledger section follows, including its observed failure,
negative controls, repair/test attribution and historical stated invariant.
It is historical evidence, not a second current normative owner. Current
IRPAT-001 remains controlling. No regression or field scenario was rerun by B6.
Original source: docs/agent/DISCOVERY_REGISTRY.md at 02d8336f983d07734bf0ba43dbb4b3a50d12ef6d.
Exact section SHA-256: 59b04690613b6d04840051bc3f5c3c445b89f6d06ba639cc9bf38b4368108813.

## DIS-2026-002: IR Consumptive Pattern Compression Orphaning Method Identities

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-03 |
| **Environment** | Claude + Clean-CTX IR pipeline (production `PassPipeline::default_production()`) |
| **Repository/context** | Real Angular workspace; a component constructor calling `.subscribe()` (DI param + RxJS callbacks in body). |
| **Symptom** | IR compilation failed with `[E007] DATAFLOW references unknown method 'M16'; [E003] FLAGS references unknown method 'M16'` — the compile itself returned `Err`, not merely corrupt output. |
| **Root cause** | The consumptive CTOR pattern compressed `DefMethod(M)` into `Pattern(CTOR, ..., M)` while leaving surviving `DataFlow(M, ...`/`Flags(M, ...)` ops with no valid owner. The validator registers method identities ONLY from `DefMethod`; `PatternOp` has no payload slot for DataFlow / SideEffect / ExecutionContext / ControlFlow facts. The first invalid state was created by CTOR pattern compression — NOT by TypeScript extraction (the pre-pattern stream was fully valid)and NOT by validation. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/ir/regression_ctor_pattern_orphan.rs` (covers CTOR, EMPTY_CTOR, and the original orphan scenario) |
| **Live scenario required?** | No |
| **Architectural invariant** | IR identity-preservation invariant for consumptive pattern transformations |
| **Status** | Fixed |

**Distillation note:** nested-callback depth alone was ruled out; the four initially suspected AST candidates (bare arrow parameter, nested plain callback, optional chaining, destructuring) were all ruled out;and the TypeScript constructor/arrow capture-kind coverage gap remains a **separate** issue, NOT causal. The fix deliberately declines CTOR / EMPTY_CTOR compression when an unrepresentable M-reference exists; healthy CTOR compression remains intact when only representable trailing `Flags` are present.

**Architectural invariant (IR pattern-transformation layer, framework/language-agnostic):**

> A consumptive IR pattern must never consume a `DefMethod(M)` while leaving behind surviving IR operations that reference `M` without preserving a valid representation/ownership relationship. If a consumptive pattern cannot represent an M-referencing operation (`DataFlow`, `SideEffect`, `ExecutionContext`, `ControlFlow`) within the resulting pattern representation, it must decline compression rather than consume the `DefMethod` and orphan the reference.



Evidence (minimal trigger: constructor with ≥1 DI parameter + `.subscribe()` in its body):

```text
Pre-pattern (valid):                   After faulty CTOR compression (invalid):
  DefMethod(M)                          Pattern(CTOR,, M)
  Param(M, ...)                         DataFlow(M,, ...)
  Return(M, ...)                       Flags(M,, ...)
  Flags(M,, ...)
  DataFlow(M,, ...)
  Flags(M,, ...)

Result: [E007] DATAFLOW references unknown method M; [E003] FLAGS references unknown method M
```
---


```architecture-graph
node: DOC-634
kind: RESEARCH
status: INVESTIGATING
title: Historical DIS-2026-002 consumptive pattern orphan
track: REPOSITORY
artifact: docs/architecture/knowledge-normalization/evidence/consumptive-pattern-orphan.md
preserves:
  - IRPAT-001
note: Source-attributed exact historical subject and limits retained; no current executable claim, independent certification, reclosure or banking.
```
