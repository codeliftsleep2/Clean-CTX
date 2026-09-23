# Phase Plan: compact-a3-density

Reference `~/.claude/templates/VERIFICATION_GATES.md` (Clean-CTX section) for
gate commands. One phase = one commit, only after its evidence report is
written and Matt has approved it. Read `FINDINGS.md` and `CONTRACT.md` in this
same directory before starting — they carry the verification evidence this
plan is built on; don't re-derive it.

---

## Phase 0: Close the fidelity/focus-mode measurement gap

- **Objective:** Make the Phase 3 economics screen actually cover what
  `PHASED_A3_PLAN.md`'s Checkpoint 3 requires — results broken out by fidelity,
  focus mode, language, and semantic density — instead of one fixed fidelity
  per tracked fixture. Establish a fresh, honest baseline before changing any
  encoding.
- **Scope (in):** Extend capture generation so `LargeService.ts` and
  `UserManagementService.ts` (and any other qualifying large TS/Angular
  fixture) each produce Low/Medium/High/Edit captures, plus a focused-Edit and
  an unfocused (all-body) Edit variant. Rerun `Measure-CompactA3.ps1` (rebuild
  the helper first) and commit the resulting per-family, per-fidelity,
  per-focus-mode anatomy into `A3_PHASE0_BASELINE.md`-style evidence. Include
  qualifying large TypeScript, Angular, and C#/.NET fixtures; these are the
  primary production scope, not optional follow-up coverage. Java/Spring may
  remain secondary. Record the evidence in a new doc or update using the
  existing table format.
- **Scope (non-goals — explicitly NOT this phase):** No encoding changes. No
  grammar changes. This phase only produces measurement evidence.
- **Files touched:** `verification/context-compression/scripts/*.ps1` (capture
  generation extension), possibly a new fixture-preparation script; docs under
  `verification/context-compression/compact-a/` to record the fresh numbers.
- **Sequence:** confirm/extend capture generation → rebuild measure helper →
  run `Measure-CompactA3.ps1` → record results → update
  `PHASED_A3_PLAN.md`/`A3_GRAMMAR.md`'s stale "22.32%/-2.76%" references with
  the real current numbers (they're stale relative to `d75de76`'s "27%" commit
  message, which was never reflected in the docs — fix that discrepancy here
  too).
- **Invariants this phase must preserve:** zero model calls (per Phase 3's own
  rule); no production wiring; existing deterministic roundtrip tests stay
  green throughout (this phase doesn't touch the codec itself).
- **Tests:** existing `cargo test` suite must stay green (no codec changes
  expected to break anything); new evidence is measurement output, not new
  unit tests.
- **Risks & mitigations:** if extending capture generation requires real
  large fixtures not already in the repo, confirm licensing/size constraints
  before adding them (same constraint the existing two tracked fixtures
  presumably already satisfy — check how they were sourced before assuming a
  new one can be added the same way).
- **Rollback strategy:** revert the commit; no other system depends on this
  data yet.
- **Verification gates:** `cargo check --all-targets`, `cargo test --workspace
  --all-targets --all-features` (Clean-CTX section, `VERIFICATION_GATES.md`).
- **Strict compiler check (rule 15):** `cargo clippy --all-targets -- -D
  warnings` and `cargo fmt --all -- --check` against every file this phase
  touches.
- **Definition of done:** a committed, per-fidelity, per-focus-mode token
  anatomy table for both tracked large fixtures, reconciled with whatever the
  `d75de76` "27%" figure was actually measuring, and docs no longer citing
  stale numbers.
- **Commit boundary:** one commit, after Matt reviews the evidence report.

---

## Phase 1: Bijective local-identity renumbering (design gate + implementation)

- **Objective:** Remove explicit canonical-ID columns from A3 declaration/
  reference rows wherever a kind's rows are already contiguous, replacing them
  with a documented, deterministic local-ordinal rule — the single largest
  lever identified in `FINDINGS.md`.
- **Scope (in):** A design proposal (grammar text + exact renumbering rule +
  which cross-references change) presented to Matt for explicit go/no-go
  **before writing code**, per `CONTRACT.md`'s open question. If approved:
  update `A3_GRAMMAR.md` to define the local-ordinal rule; update
  `compact_a3::declarations` encode/decode to drop explicit IDs where
  reconstructible; update `compact_a3::document::target()` and the round-trip
  tests to compare identity fields under the new bijective-equivalence rule
  instead of literal string equality.
- **Scope (non-goals — explicitly NOT this phase):** No change to
  `src/ir/compiler.rs`'s `next_id`/global counter. No change to
  `src/ir/focus.rs` selector resolution (already name-based, already correct).
  No change to persistence/delta/replay authority.
- **Files touched:** `verification/context-compression/compact-a/A3_GRAMMAR.md`,
  `src/ir/compact_a3/declarations.rs`, `src/ir/compact_a3/declarations/{parse,values}.rs`,
  `src/ir/compact_a3/document.rs`, `src/tests/ir/compact_a3*.rs`.
- **Sequence:** write the design proposal referencing `FINDINGS.md`'s lever-1
  evidence (the `next_id` global-counter finding, the `focus.rs` name-based
  resolution finding, the zero-MCP-callsite finding) → get Matt's explicit
  approval → implement grammar change → implement encode/decode → update
  round-trip test oracle-normalization → re-run Phase 0's measurement to
  quantify the actual token win.
- **Invariants this phase must preserve:** every other `PHASED_A3_PLAN.md`
  invariant unchanged; decode still fails closed on malformed/inconsistent
  streams (an out-of-range or duplicate local ordinal is a hard decode error,
  not a best-effort recovery); reasoning-fidelity non-inferiority is not yet
  claimed by this phase alone — defer that claim to Phase 5's full re-run.
- **Tests:** update existing `compact_a3` roundtrip tests
  (`src/tests/ir/compact_a3*.rs`) to assert bijective-equivalence rather than
  literal ID match; add a new test asserting decode fails closed on a
  duplicate/out-of-range local ordinal (RED/GREEN per Matt's rule 8 — write
  the failing case first).
- **Risks & mitigations:** risk of quietly reintroducing the rejected
  "position as canonical identity" anti-pattern if the local-ordinal rule
  isn't kept strictly internal to the wire (i.e., never leaks out as if it
  were a persisted ID anywhere). Mitigation: the design proposal must state
  explicitly, and a code comment/doc must record, that this local ordinal is
  never valid outside the single decode call that produced it.
- **Rollback strategy:** revert the commit; A3 has no production consumer.
- **Verification gates:** full Clean-CTX gate set (`cargo check`, `clippy
  -D warnings`, `fmt --check`, `cargo test --workspace --all-targets
  --all-features`).
- **Strict compiler check (rule 15):** same command, run against every file in
  this commit.
- **Definition of done:** grammar documents the new rule; roundtrip tests
  green under the redefined equality; measured token delta reported against
  Phase 0's baseline.
- **Commit boundary:** one commit, after Matt's design approval *and* the
  evidence report for the implementation.

---

## Phase 2: Scoped string table (types + callee-written-names only)

- **Objective:** Test a dictionary/back-reference scheme scoped to exactly the
  two columns Phase 0's anatomy data shows are most repetitive, with a hard
  empirical keep/reject rule.
- **Scope (in):** Implement an optional per-file table for parameter/field/
  return type strings and `K`-stream callee-written-names; measure against
  Phase 0's fixtures; **keep only if it wins tokens on the real large
  fixtures** — reject on a character-count argument alone (see the ULTRA
  precedent in `FINDINGS.md`).
- **Scope (non-goals — explicitly NOT this phase):** No generic/all-value
  dictionary (already rejected twice — see `FINDINGS.md`). No change to any
  other record family.
- **Files touched:** `A3_GRAMMAR.md`, `src/ir/compact_a3/facts.rs`,
  `src/ir/compact_a3/declarations.rs`, associated tests.
- **Sequence:** implement behind a clean on/off comparison → measure on Phase
  0's fixtures under both tokenizers → keep only if it improves the
  production-selected aggregate after each invocation applies the raw gate,
  without introducing any inflation bug → otherwise document the negative
  result and drop it (a
  negative result here is still a valid, reportable outcome — don't force it
  to "win" by cherry-picking a fixture).
- **Invariants this phase must preserve:** same as Phase 1's carryover list;
  additionally, the table itself must be included in whatever the legend/
  header wire-cost accounting is, since it's exactly the kind of fixed
  overhead that fooled the ULTRA candidate — measure the *whole* payload, not
  just the theoretical per-value savings.
- **Tests:** roundtrip tests covering table-referenced values, empty-table
  (below-threshold) fallback, and malformed table-index rejection.
- **Risks & mitigations:** the two "already rejected" precedents are the
  central risk — mitigate by keeping this experiment scoped and by measuring
  before committing to keeping it, not after.
- **Rollback strategy:** revert the commit if the measured result is negative
  and the code was still worth committing as a documented negative finding
  (matches Matt's evidence-discipline preference for recording what was
  deliberately not kept, not just what was).
- **Verification gates:** full Clean-CTX gate set.
- **Strict compiler check (rule 15):** same command.
- **Definition of done:** either a measured, kept improvement with numbers, or
  a documented rejection with numbers — not an assumption either way.
- **Commit boundary:** one commit either way (keep-with-evidence or
  documented-rejection-with-evidence).

---

## Phase 3: Occurrence-group default elision (`mo`/`cs`/`lf`/`pf`)

- **Objective:** Apply the same "omit the common default, only emit
  exceptions" treatment already proven on the `K` call stream to the
  remaining occurrence-group families, using Phase 0's real corpus frequency
  data to pick the actual default value(s) rather than assuming one.
- **Scope (in):** Measure real frequency distributions for `mo`/`cs`/`lf`/`pf`
  values across the corpus; if a dominant default exists, elide it the same
  way `K` already elides `unresolved`/non-spread.
- **Scope (non-goals — explicitly NOT this phase):** No change to families
  without a clearly dominant default in the real data — don't force this
  pattern where the data doesn't support it.
- **Files touched:** `A3_GRAMMAR.md`, `src/ir/compact_a3/declarations.rs`,
  associated tests.
- **Sequence:** extract frequency data from Phase 0's corpus → decide per
  family whether a default exists → implement only where justified → measure.
- **Invariants this phase must preserve:** empty and duplicate occurrence
  groups remain explicit and significant per the existing grammar rule — this
  phase changes what's considered a "default," not the significance rule
  itself.
- **Tests:** roundtrip tests for both the default-elided and explicit-value
  paths per family changed.
- **Risks & mitigations:** picking a default from too small a sample; mitigate
  by requiring Phase 0's full corpus, not just the two large tracked fixtures.
- **Rollback strategy:** revert the commit.
- **Verification gates:** full Clean-CTX gate set.
- **Strict compiler check (rule 15):** same command.
- **Definition of done:** measured token delta per family changed, with the
  frequency data that justified each change recorded in the commit/evidence
  report.
- **Commit boundary:** one commit.

---

## Phase 4: Row/record merging cleanup

- **Objective:** Cut per-row tag/LF overhead where two record types always
  co-occur (e.g. `M` immediately followed by its `p` row), now that the
  surrounding row shapes from Phases 1–3 are settled.
- **Scope (in):** Identify always-co-occurring record pairs from the
  *post-Phase-3* grammar and merge them if it measurably helps.
- **Scope (non-goals — explicitly NOT this phase):** Don't do this before
  Phases 1–3 land — merging now and re-merging after identity/table changes
  would waste work and risk skipping proper before/after measurement.
- **Files touched:** `A3_GRAMMAR.md`, `src/ir/compact_a3/**`, tests.
- **Sequence:** identify candidates → measure → implement only measured wins.
- **Invariants this phase must preserve:** all carried-forward invariants.
- **Tests:** roundtrip tests for merged row shapes.
- **Risks & mitigations:** low risk, marginal impact — deprioritize if time is
  short; this phase is explicitly the last and smallest lever.
- **Rollback strategy:** revert the commit.
- **Verification gates:** full Clean-CTX gate set.
- **Strict compiler check (rule 15):** same command.
- **Definition of done:** measured token delta, roundtrip green.
- **Commit boundary:** one commit.

---

## Phase 5: Full Checkpoint 3/4 re-run and correctness/reasoning sign-off

- **Objective:** Confirm the cumulative effect of Phases 1–4 against
  `PHASED_A3_PLAN.md`'s actual gates — Checkpoint 3 (at least 50% aggregate
  corpus savings after per-invocation raw selection, with every diagnostic row
  still reported) and, if the numbers justify
  moving forward, Checkpoint 4's bounded reasoning screen (Section 8 of the
  research doc: lossless roundtrip is necessary, not sufficient).
- **Scope (in):** Re-run the full per-fidelity/per-focus-mode measurement from
  Phase 0's harness against the final grammar; if economics clear the gate,
  run the smallest bounded reasoning cases per Phase 4 of
  `PHASED_A3_PLAN.md` (ownership/overload identity, ordered/duplicate calls,
  DI grouping, behavior-family lookup, focused-Edit target correctness) —
  Matt owns the actual model-call budget/approval for this step.
- **Scope (non-goals — explicitly NOT this phase):** No production
  integration (`PHASED_A3_PLAN.md` Phase 5/6) — that's a separate, later
  initiative, not this contract's scope.
- **Files touched:** measurement docs/evidence only; no further codec code
  expected unless Phase 5's data reveals a regression to fix.
- **Sequence:** re-measure → update `PHASED_A3_PLAN.md`'s checkpoint status →
  if economics pass, propose the reasoning screen to Matt as its own
  model-call-spending decision (do not spend model tokens without his
  explicit go-ahead, per the plan's own stop conditions).
- **Invariants this phase must preserve:** all of them, now confirmed
  end-to-end rather than per-phase.
- **Tests:** full `cargo test --workspace --all-targets --all-features`.
- **Risks & mitigations:** if an invocation loses after the safety margin,
  production must select raw and that row contributes zero savings rather than
  inflation. The candidate fails Checkpoint 3 only when the production-selected
  qualifying corpus remains below 50% aggregate savings. Report every row and
  never weaken correctness. Passing 50% does not end optimization because
  there is no maximum savings target.
- **Rollback strategy:** N/A — this is a measurement/reporting phase.
- **Verification gates:** full Clean-CTX gate set.
- **Strict compiler check (rule 15):** N/A unless a regression fix is needed.
- **Definition of done:** an evidence report Matt can review covering: final
  per-fixture/per-fidelity/per-focus-mode economics, whether Checkpoint 3
  passes, and an explicit recommendation on whether Checkpoint 4's reasoning
  screen is worth spending model calls on next.
- **Commit boundary:** one commit for the final docs update; no code commit
  expected unless this phase surfaces a fix.

---

<!-- Don't manufacture fake historical phase boundaries after the fact if they
can't be reconstructed honestly — squash into one checkpoint instead. -->
