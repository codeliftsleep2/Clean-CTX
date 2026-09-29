# Contract: compact-a3-density

- **Repo(s):** Clean-CTX
- **Date opened:** 2026-09-22
- **One-line objective:** Push COMPACT-A3's model-facing token density as low as
  possible without any loss of decode correctness or LLM reasoning fidelity.
  Fifty-percent aggregate reduction across production-selected outputs for the
  qualifying representative corpus is the minimum eligibility gate, not a
  maximum or optimization stopping point. Individual invocations remain
  protected by raw fallback rather than separate 50% requirements.

## Identity & Ownership

The thing being changed is the **presentation/wire layer** only:
`src/ir/compact_a3/**` (declarations, facts, document composition) and
`verification/context-compression/compact-a/A3_GRAMMAR.md` (the versioned wire
grammar). It is research-only and unreachable from MCP production today (`grep
-r "compact_a3::" src/mcp` returns zero matches) and lives on an unmerged
branch — there is no live consumer to break.

Owned by: whoever implements this initiative next (Matt explicitly is not
implementing this himself — see `FINDINGS.md` header). The initiative does not
own the canonical IR (`src/ir/compiler.rs`, `identity.rs`), persistence
(`src/mcp/sqlite_store/**`), or workspace-query authority — those are
read-only reference points, never edit targets here.

## Lifecycle

This is research/pre-production work: Phase 0 (measurement-gap closure) →
Phase 1 (identity-renumbering decision + implementation, gated on Matt's
sign-off) → Phase 2 (scoped string table experiment) → Phase 3 (occurrence
default elision) → Phase 4 (row merging) → Phase 5 (full Checkpoint 3 economics
+ Checkpoint 4 reasoning re-run). There is no production integration
(`Phase 5`/`Phase 6` of `PHASED_A3_PLAN.md`) in this contract's scope — A3 stays
research-only throughout.

## Data Flow

Unchanged from today's design: `normalized CONTROL-FULL v2 semantic object`
(the correctness oracle, produced by the compiler/hierarchical projection) →
`compact_a3::declarations::encode_declarations` + `compact_a3::facts::encode`
+ `compact_a3::document::encode` → wire bytes → (test-only) `decode()` →
recovered semantic object compared against the oracle for exact equality. This
contract's changes act on the encode/decode functions and the grammar they
implement; the oracle production path (compiler → hierarchy → normalization)
is untouched.

## Boundaries

**In scope:** the A3 wire grammar (`A3_GRAMMAR.md`) and its Rust implementation
under `src/ir/compact_a3/**`; the round-trip test harness's definition of its
own decode target (`compact_a3::document::target()`) where lever 1 requires
redefining identity-field equality; the measurement harness
(`verification/context-compression/scripts/Measure-CompactA3.ps1` and its
capture-generation pipeline) to close the per-fidelity/per-focus-mode coverage
gap documented in `FINDINGS.md`.

**Explicitly out of scope, must not cross without a documented reason and a
new decision from Matt:**
- `src/ir/compiler.rs`'s identity assignment (`next_id`, the shared global
  counter) — lever 1 works *around* this, never changes it.
- Canonical IR shape, `CompiledIR`, `HierarchicalIR`.
- Persistence/delta/replay authority (`src/mcp/sqlite_store/**`,
  `src/ir/delta.rs`, `src/ir/replay.rs`) — confirmed these never treat decoded
  A1/A3 text as an identity source; this contract must not make them start.
- `src/ir/focus.rs` selector resolution — confirmed name-based already; no
  change needed or wanted here.
- Any MCP production wiring (`src/mcp/tool_handlers/**`) — A3 stays
  unreachable from production for the duration of this contract.

## Pipeline Ordering

Phase 0 (close the measurement gap) must land before Phases 2–4 (specific
encoding tricks), because those tricks should be validated against real
per-fidelity/per-focus-mode data, not the current single-fidelity-per-fixture
numbers. Phase 1 (identity renumbering) is independent of Phase 0's data but
gated on Matt's explicit design sign-off (see Open Questions) before any code
is written, since it changes what "correctness" means for one semantic family.

## Invariants

Carried forward from `PHASED_A3_PLAN.md` (all 9 non-negotiables) plus:

- Decode equality remains exact for every family **except** identity spelling,
  which — if and only if Matt approves lever 1 — becomes "bijectively
  equivalent under a single documented renumbering rule" rather than literal
  string equality. This redefinition must be written into `A3_GRAMMAR.md`
  itself, not left implicit in code.
- No change may reduce reasoning-fidelity non-inferiority versus
  `CONTROL-FULL` (per `docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md`
  Section 8) — lossless roundtrip is necessary but not sufficient.
- No dictionary/table technique (lever 2) may be kept unless it is measured to
  win tokens on the real tracked large fixtures; a theoretical/character-count
  argument is insufficient (the ULTRA precedent shows why).
- 100% correctness is a hard boundary — no lever in this contract may trade
  correctness for compression at any ratio.

## Scope

Covers: identity-renumbering design + implementation, scoped string table
experiment, occurrence-group default elision, row-merging cleanup, closing
the fidelity/focus-mode measurement gap, and representative large-file
coverage for the three primary production scopes: TypeScript, Angular, and
C#/.NET. Does not cover: production integration of A3 (later, separate phases
in `PHASED_A3_PLAN.md`) or any change to what data a given
fidelity/focus mode is *entitled* to see beyond what's already defined in
`A3_GRAMMAR.md`'s fidelity target matrix (row-level fidelity trimming beyond
family-level, `FINDINGS.md` lever mentioned as "needs data first," is a
candidate for a future contract once Phase 0's data exists — not pre-approved
here).

## Failure Behavior

Every encode/decode error path already fails closed per the grammar's
"Decoder rejection requirements" section — this contract must not weaken any
of those checks. Any new numbering/table scheme must fail closed the same way
(e.g., a local-ordinal mismatch or table-index-out-of-range is a hard decode
error, never a silent best-effort recovery).

## Rollback Strategy

Each phase is one commit on the existing research branch (already isolated
from `main`/production). Rollback is `git revert` of that phase's commit;
because A3 has no production consumer, there is no live-system rollback
concern, only test/measurement-suite consistency to preserve.

## Open Questions / Assumptions Resolved

- **Resolved:** does anything in production need A3-decoded text to carry the
  compiler's true global-counter ID string? No — confirmed via `focus.rs`
  (name-based resolution), zero MCP call sites into `compact_a3::`, and the
  research doc's restore/replay authority statement. See `FINDINGS.md` lever 1.
- **Resolved:** was a repeated-string dictionary already tried? Yes, twice
  (ULTRA candidate in the foundational research doc; a second, code-untraced
  rejection recorded in `PHASED_A3_PLAN.md`'s Phase 3 section) — both appear to
  be generic/all-value substitution, not scoped to the two highest-repetition
  columns. See `FINDINGS.md` lever 2 for why a narrower retry isn't foreclosed.
- **Resolved:** does the current economics measurement cover fidelity × focus
  mode as Checkpoint 3 requires? No — each tracked large fixture is measured at
  one fixed fidelity today. See `FINDINGS.md`'s measurement-gap section.
- **Open, requires Matt's decision before Phase 1 code:** is Matt willing to
  formally redefine decode-equality for the identity family (bijective
  renumbering vs. literal compiler-ID match)? This is presented as a design
  proposal in Phase 1 of `PHASE_PLAN.md`, not pre-approved by this contract.
