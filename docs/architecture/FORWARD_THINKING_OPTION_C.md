# Forward Thinking — Option C and the next presentation work

**Status:** forward-looking hand-off. Read after the Option-A work (SCHEMA-v5 as
`content`) and the annotation-redundancy collapse are finalized and committed.

**Audience:** the next agent (Codex) resuming this work.

**Authoritative sources this builds on:**
- `docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md` §15 — the current
  measurement checkpoint and the presentation/compression findings.
- `docs/agent/DISCOVERY_REGISTRY.md` DIS-2026-020 — the presentation-boundary
  decision (Option A now, Option C follows).
- `verification/context-compression/compact-a/PRESENTATION_BOUNDARY_PLAN.md`
  §6 + §2.2 — the approved Option A/B/C decision and the SHOULD/SHOULDN'T
  contract Option C is authored against.

---

## 1. Where we are

- **Option A is implemented and verified.** `content` is the SCHEMA-v5
  presentation (`render_hierarchical_for_llm`) on every path; the reversible
  codec (CONTROL-FULL / COMPACT-A2/A3) is code-side (`result.ir` + persistence).
- **The annotation-redundancy collapse landed.** The `async` fact was reported
  three times (`mod:ASYNC` + `se:async` + `ec:async`); it is now reported once,
  at the most structural level available (`mod:ASYNC`, else `se:async`). This is
  presentation-only — the codec still stores all three. High-fidelity density
  moved **+1.6pp (TS), +1.3pp (Angular), +2.6pp (C#)**.
- **The measurement + reasoning harness exists.** Density is measured via
  `Capture-Baselines.ps1` → `measure-schema-v5.ps1`; accuracy is measured via
  the task-based edit eval (deterministic grading + `apply_edit` round-trip,
  currently 3/3 green).

Current High-fidelity SCHEMA-v5 density (o200k):

| Fixture (language) | Low | Medium | High |
|---|---:|---:|---:|
| typescript | 76.3% | 70.1% | 67.1% |
| angular | 73.9% | 64.6% | 61.8% |
| csharp | 64.2% | 48.4% | 43.9% |

## 2. The documented next extension — Option C

The presentation-boundary decision approved **"Option A now, then Option C"**:

> Option C — a **purpose-built presentation authored against §2.2's
> SHOULD/SHOULDN'T table** — follows Option A. (Rejected: Option B, the codec
> minus its legend — a positional grammar without its interpretive key is
> undecodable, not presentable.)

So Option C is not "keep tweaking SCHEMA-v5 forever"; it is a *deliberate*
presentation renderer, authored from a contract, rather than the inherited
`render_hierarchical_for_llm`.

### The §2.2 contract (what Option C is authored against)

**SHOULD be in `content` (file-local, reasoning-necessary):** file identity
(source path, version); class/interface identity (name + kind);
`extends`/`implements`; fields (name + type + owner); method identity (name,
params, return); overload disambiguation (arity); collapsed modifiers/flags
(`async`, `static`, `export`); imports + type aliases; exact bodies (Edit +
focused only).

**SHOULDN'T be in `content`:** semantic edges (framework/cross-file) →
`workspace_query`; canonical IDs (`C1/M1/F1/P1`) → regenerate positionally
(code-side); occurrence groups + duplicates → code-side (round-trip);
navigation index → code-side; grammar legend → code-side; body byte-length
framing → code-side.

§2.2's own framing: *"This is structurally main's SCHEMA v2 (names-only) plus
explicit ownership clarity."*

## 3. What we've learned that should shape Option C

1. **Density is language-dependent, and the ceiling is structural.** After the
   async collapse, C# High is 43.9% vs TS 67.1%. The remaining C# gap is *not*
   annotation redundancy — it is verbose signatures
   (`CancellationToken cancellationToken = default`, `Task<ActionResult<T>>`
   returns, `[FromBody]`/`[FromQuery]` attributes) plus the `IOrderService`
   interface mirror.
2. **Redundancy collapse is cheap and safe when it is conditional.** The async
   collapse dropped only the co-derived duplicates, kept the codec untouched,
   and left non-redundant facts (`se:io`, `ec:sync`, …) intact. That pattern
   generalizes.
3. **Compression and accuracy are one experiment, never two.** A presentation
   change is judged by density *and* task accuracy together — size alone is
   meaningless if the model loses the fact it needs to reason with.
4. **The accuracy guardrail is currently Edit-fidelity only.** The task eval
   runs `fidelity="edit"`, but `se:`/`ec:` (and most High-fidelity annotation
   density) are High-only. A High-fidelity presentation change therefore has
   **no accuracy gate today** — this is a real gap (see §4a).

## 4. Concrete forward directions (in dependency order)

### 4a. Build the High-fidelity reasoning guardrail first

The `TASK_EVALUATION.md` "Debug" tasks (4–5) are proposed but not implemented.
Before changing High-fidelity presentation further, add a **High-fidelity**
comprehension/debug task with deterministic grading (name the correct method +
condition + outcome; no wrong facts). Without it, Option C (which changes
High-fidelity output) cannot be validated for reasoning preservation.

### 4b. Incremental SCHEMA-v5 optimizations (optional, smaller wins)

Presentation-only A/Bs on the existing renderer, same pattern as the async
collapse:

- **Signature boilerplate** — elide the repeated
  `CancellationToken cancellationToken = default` (and comparable per-method
  defaults) in the presentation. This is the single largest remaining C# lever.
- **Interface mirror dedup** — the `IOrderService` mirror repeats the concrete
  service's method list; consider a compact mirror notation.
- Any such change: re-measure density (re-capture first) *and* gate against the
  §4a High-fidelity reasoning task.

### 4c. Option C proper — the purpose-built presentation

A new renderer authored against §2.2, not an incremental patch of
`render_hierarchical_for_llm`. Expected to:
- collapse signature verbosity at the schema level (not ad-hoc string tricks);
- make ownership explicit (the §2.2 "explicit ownership clarity" note);
- keep the codec contract fully code-side (never reintroduce the
  legend/IDs/framing that Option A removed).

This is the larger architectural step. It deserves its own design doc + tracked
regressions before implementation, and it must preserve ARCH-003 (presentation)
vs CTX-001 (reversible codec).

## 5. Methodology guardrails (applies to 4b and 4c)

- **Presentation-only.** Never change the canonical IR / codec to optimize the
  presentation. If a compression requires dropping a fact the codec needs, it is
  wrong by construction.
- **Measure size and accuracy together.** Re-capture, then measure; run the
  relevant reasoning task; report both, never size alone.
- **Correct capture order.** `measure-schema-v5.ps1` re-counts existing captures
  — it does *not* re-run the binary. Always `Capture-Baselines.ps1` first.
- **Conditional collapse, not fact deletion.** Drop a fact only when a
  more-structural fact already implies it (the `mod:ASYNC` → `se:async` →
  `ec:async` hierarchy), and preserve every non-redundant value.
- **Tracked regressions.** Every behavior change gets a test under
  `src/tests/**` via `#[path]`, never inline.

## 6. Non-negotiable constraints

- Strict UTF-8, no BOM — `.clinerules/encoding.md`; run `scripts/check-utf8.ps1`
  after any text edit.
- Zero warnings: `cargo clippy --all-targets --all-features -- -D warnings`.
- Files ≤ 615 lines (Markdown exempt); tests in `src/tests/**`.
- Do **not** start long-running processes yourself — hand off `cargo`/binary
  commands and report results accurately.
