# Live Discovery Registry (`docs/agent/DISCOVERY_REGISTRY.md`)

**Purpose:** Record discoveries made in the **real-world discovery environment**
(Claude + Clean-CTX operating against large real repositories) and their
resolution, so field findings become institutional engineering knowledge rather
than disappearing into a development conversation.

This is NOT a changelog (see `docs/CHANGELOG.md`) and NOT release accounting
(see `docs/agent/releases.md`). It exists to close the two-environment gap:

```text
REAL-WORLD DISCOVERY (Claude + real repo)
      ↓
root-cause investigation
      ↓
classify (Protocol | Semantic | Emergent)
      ↓
can it be distilled deterministically?  → local regression
      ↓                                        (cheap Rust test)
not reproducible at small scale
      ↓
retain as a live acceptance scenario → documented record here
```

## Format

One entry per significant discovery, newest first. An entry is closed when a
local regression covers it, a live scenario has been re-verified, or the
behavior is superseded.

## Entry Template

| Field | Value |
|-------|-------|
| **Discovered** | YYYY-MM-DD |
| **Environment** | Claude + Clean-CTX vX.Y.Z |
| **Repository/context** | Description (approximate size, languages, frameworks) |
| **Symptom** | What was observed |
| **Root cause** | Precise technical cause |
| **Classification** | Protocol / Semantic / Emergent |
| **Reproducible locally?** | Yes / No |
| **Local regression** | Test path(s) or N/A |
| **Live scenario required?** | Yes / No |
| **Architectural invariant** | INV-XXX or N/A |
| **Status** | Open / Fixed / Verified |

---

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

## DIS-2026-003: IR CTOR Compression Orphaning with Empty Constructor + Method-Scoped References

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-04 |
| **Environment** | Clean-CTX IR pipeline (production `PassPipeline::default_production()`) |
| **Repository/context** | Angular component with empty constructor using a private parameter property (`constructor(private store: Store) {}`) and Store operations in a separate method (`ngOnInit`). |
| **Symptom** | IR compilation failed with `[E003] FLAGS references unknown method 'M5'`. |
| **Root cause** | The consumptive CTOR pattern compressed the empty constructor's `DefMethod(M)` into `Pattern(CTOR, ..., M)` while leaving surviving IR operations that reference a different method (`ngOnInit`, M5) without a valid owner. The existing CTOR orphan-prevention fix (DIS-2026-002) covers the case where unrepresentable M-referencing operations exist within the constructor body, but not the case where the constructor is empty and the references belong to a separate method. |
| **Classification** | Semantic |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/ir/regression_ctor_pattern_orphan.rs::edit_fidelity_param_property_ctor_does_not_orphan_flags` (compiler-level regression at Fidelity::Edit); `src/tests/mcp/workspace_query.rs::builtin_decorated_class_and_ngrx_semantic_names_production_path` (production-path fixture restored with the empty constructor) |
| **Live scenario required?** | No |
| **Architectural invariant** | Same as DIS-2026-002 (IRPAT-001): IR identity-preservation for consumptive pattern transformations |
| **Status** | Fixed |

**Fix (2026-09-04):** `op_is_unrepresentable_method_ref` in `src/ir/patterns.rs` now treats `Body(M, ...)` as an unrepresentable M-referencing operation, alongside `DataFlow`, `SideEffect`, `ExecutionContext`, and `ControlFlow`. At Edit fidelity the TS language layer emits `Body(M)` between a constructor's `Return(M)` and its trailing `Flags(M, ["PRIVATE"])` (parameter property); the `Body` op breaks the wrapper's adjacent trailing-Flags run, so `Flags(M)` would survive as an orphan (E003) if compression consumed `DefMethod(M)`. The extended guard makes the CTOR/EMPTY_CTOR patterns decline compression for that region, preserving the full valid sequence. The production-path fixture (empty constructor restored) and the new compiler-level Edit-fidelity regression both pass.

**Distillation note:** This defect predates the selector-representation fix and is unrelated to it. The existing CTOR orphan-prevention fix (DIS-2026-002) covers the constructor-body case but NOT this shape. Separate IR/compiler defect.

**Minimal trigger:**
```typescript
@Component({ selector: 'widget-shell' })
export class ShellComponent {
  constructor(private store: Store) {}  // EMPTY + private parameter property
  ngOnInit() {
    this.store.pipe(select('panelState'));  // Store ops in SEPARATE method
    this.store.dispatch({ type: TOGGLE_PANEL });
  }
}
```

---

**Resolved (2026-09-04):** Production fix implemented in `src/ir/patterns.rs`. The earlier fixture workaround (removing the empty constructor) is no longer needed — and has been reverted so the production-path test exercises the exact DIS-2026-003 scenario again.
---
## DIS-2026-001: workspace_query MCP Response Envelope Bypass

| Field | Value |
|-------|-------|
| **Discovered** | 2026-09-02 |
| **Environment** | Claude + Clean-CTX (post-0.5.0 structured-output migration) |
| **Repository/context** | Large heterogeneous real workspace |
| **Symptom** | Every `workspace_query` response carried bare domain fields (`entities`, `edges`, `count`, ...) directly under JSON-RPC `result` with NO MCP `content` channel — a schema-validating MCP client had nothing renderable to display. |
| **Root cause** | All six `workspace_query` sub-handlers built bare `result` objects, bypassing the canonical MCP `CallToolResult` envelope (`content` + optional `structuredContent`/`_meta`) established by the 0.5.0 migration. The handler was introduced after the migration (commit `2d18377`, 2026-09-01) with no contract audit gate for new tools; its tests validated the ad-hoc result shape rather than the envelope. |
| **Classification** | Protocol |
| **Reproducible locally?** | Yes |
| **Local regression** | `src/tests/mcp/workspace_query.rs` (all operations validated via `assert_valid_mcp_envelope` + `structuredContent`); `src/tests/mcp/phase3_contract.rs` (shared envelope helpers); `src/tests/mcp/envelope_contract.rs` (complete remaining produced-tool coverage) |
| **Live scenario required?** | No |
| **Architectural invariant** | MCP-001 (docs/ARCHITECTURAL_INVARIANTS.md) |
| **Status** | Fixed |

**Distillation note:** the root cause was NOT workspace complexity. It was an
MCP response-contract violation, reproducible with the existing sample corpus —
no large fixture was required. This is the canonical example of a **Protocol**
class discovery: field evidence surfaced it; a cheap deterministic Rust
regression now protects it permanently.