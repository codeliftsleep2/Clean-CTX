# Findings: COMPACT-A3 further density (2026-09-22 investigation)

Written for whoever implements this next. This is verified investigation, not a
request to implement inline — Matt asked for findings + a plan to hand off, not
code changes in this session.

## Goal, as stated by Matt

Maximize token compression on the model-facing COMPACT-A3 file-context codec.
Fifty-percent aggregate reduction across production-selected outputs for the
qualifying representative corpus is the minimum eligibility gate, never the
optimization ceiling. Individual invocations use the raw-economics gate and
are reported separately, but are not each required to save 50%. Optimization
remains
bounded by a hard correctness floor (exact decode-equality to the
fidelity-appropriate normalized target, per `PHASED_A3_PLAN.md`'s 9
non-negotiable invariants) and by "the LLM must still understand it."
Grammar-level (wire format) redesign is
explicitly in scope; canonical IR, compiler identity assignment, persistence
authority, and workspace-query authority are explicitly **not** in scope. The
codec lives on its own unmerged branch — there is currently zero production
compatibility constraint on the wire format.

Current best measured result: ~25% reduction (o200k) on the two tracked large
fixtures (`LargeService.ts`, `UserManagementService.ts`), after two rounds of
density work (`de09beb`, `d75de76`). Checkpoint 3 in `PHASED_A3_PLAN.md` wants
≥50% aggregate corpus savings as a *gate*, not as the actual goal ceiling.

TypeScript, Angular, and C#/.NET are the primary production scopes and must all
receive qualifying large-file economics and reasoning coverage before the
research gate can close. Java/Spring is secondary unless existing coverage can
be retained cheaply.

## What's already been tried and where it stands

`docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md` is the foundational
research document behind the entire COMPACT-A lineage (A0/A1/A2/A3), written
before any of A1/A2/A3 existed. It already ran a structured technique
assessment (Section 9), a risk/anti-pattern list (Section 10), and an
explicitly-rejected list (Section 13). **Read this document before proposing
any "new" idea** — several ideas that look novel from inside `compact_a3/` were
already tried at the concept-candidate stage (COMPACT-A, COMPACT-B, ULTRA) with
measured results:

- A **generic, all-values dictionary/index substitution** (the "ULTRA"
  candidate) was measured and *increased* o200k tokens versus the
  non-dictionary candidate on the same representative sample (504 vs. 389
  tokens on COMPACT-A), despite being 238 fewer raw characters. Cause: BPE
  already compresses natural repeated identifiers more cheaply than "index
  number + delimiter" sequences; the apparent byte win doesn't survive
  tokenization. This is real, measured evidence, not a hunch — see Section 6's
  token table and Section 10's note ("Optimizing characters instead of actual
  target-model tokens").
- The A3 plan's own `PHASED_A3_PLAN.md` records a **second, narrower rejection**
  of the same idea: "A byte-profitable repeated-string dictionary was tested
  and rejected because it increased o200k cost on both large fixtures"
  (Phase 3 section). I could not find the code for this second experiment in
  git history — it appears to have been tried and discarded without a
  preserved commit. Treat "a dictionary was tried, twice, and lost both times"
  as established, but note neither rejection was scoped to *just* the highest
  repeated columns (types, callee names) — both sound like broader/generic
  substitution. See lever 2 below for why this distinction matters.
- **"Position as canonical identity" is an explicitly rejected anti-pattern**
  (Section 13: "truncation/reordering would change meaning... permitted only
  as a reversible rendering handle after internal resolution") — this is
  directly relevant to lever 1 below and is why that lever needs a specific,
  narrow design, not a blanket "just use row position for everything."

Do not re-run the generic dictionary experiment expecting a different answer.
Do not treat "dictionary" as one idea — the failed version and the still-open
version (lever 2) are different in scope, and the plan below only proposes the
latter.

## Verified levers, ranked

### 1. Bijective local-identity renumbering — the biggest lever, needs a decision first

**The problem:** every declaration/reference row in the wire (`M`, `F`, `P`,
`C`, `I`, and every cross-reference like `D`'s dependency target) still spends
a token+delimiter on an explicit canonical ID string (`M187`, `F45`...), even
after the last two density commits. `PHASED_A3_PLAN.md`'s own Phase 3 note says
declarations dominate the remaining cost. This is the largest remaining
class of "we're sending something the LLM doesn't need in this exact form."

**Verified:** `src/ir/compiler.rs:184` (`next_id`) is a **single global
monotonic counter shared across every declaration kind** — classes, interfaces,
fields, methods, parameters, import aliases, and type aliases all draw from the
same counter in raw compiler-traversal order (confirmed by reading every
`next_id(...)` call site — `src/ir/pipeline/core.rs`, `src/ir/pipeline.rs`).
This means:

- IDs of one kind (e.g., all field IDs in a class) are **not contiguous** —
  other kinds' IDs fall in the gaps — so the current wire layout (batch all
  fields into one `F` row, batch each method's `p` row separately) cannot
  recover the true ID by counting same-kind rows. A naive "just drop the ID and
  recompute it from position" will decode to the *wrong* ID string.
- The only way to recover the *true* compiler ID purely from position would be
  to restructure the wire into "one row per `next_id()` call, in that exact
  order" — i.e., interleave classes/interfaces/fields/methods/params/imports
  exactly as the compiler visits them. This is possible, but it **couples codec
  correctness to an internal compiler implementation detail that isn't part of
  any documented contract**, and it's the literal shape of the "position as
  canonical identity" anti-pattern the research doc rejects (Section 13). If
  the compiler ever adds a new node kind that calls `next_id()`, or reorders
  its own traversal, this would silently desync with no wire-visible signal.
  **Do not do this.**

**The safe version, which the research doc already pre-authorizes:** Section 2
footnote 1 of `LLM_CONTEXT_COMPRESSION_RESEARCH.md` states: *"A canonical ID's
literal spelling (`C1`, `M7`) is not irreducible... may be replaced by a
shorter file-local ordinal if decoding/reasoning preserves a bijection and type
family."* I verified this is actually safe to exercise in this codebase,
specifically:

- `src/ir/focus.rs::resolve_focus_method_ids` — the function `apply_edit`/focus
  selection actually calls — resolves selectors **purely by name** (bare method
  name, or `Owner.method`). It never takes or returns a canonical ID from the
  caller/model. Read in full; confirmed no ID-string round-trip requirement
  exists at this boundary.
- `grep -r "compact_a3::" src/mcp` returns **zero matches**. No production MCP
  handler calls any `compact_a3` decode function today — A3's `decode()` exists
  only to support the deterministic round-trip *test*, never a live
  re-hydration path.
- The research doc's Section 14 states restore/replay must "regenerate
  normalized CONTROL-FULL from durable checked IR... never trust stale compact
  text" — i.e., production identity authority always comes from the persisted
  IR, never from parsing wire text back into IDs.

**Conclusion:** nothing in the actual system needs the wire text's identity
strings to equal the compiler's true global-counter IDs. It is safe to define
A3's *own* identity numbering — e.g., a sequential ordinal per kind, assigned
in first-appearance order within the file, entirely independent of the
compiler's counter — and drop the ID column from the wire entirely wherever a
kind's own rows are already contiguous (which most of them are, since owner
scoping already groups same-kind rows together). Cross-references (`D`'s
dependency target, once it's not a raw string) would reference the *local*
ordinal, not the true ID.

**What this requires, and why it's a decision, not a default:**
- Redefining what "correct" means for the identity family in the round-trip
  test: `compact_a3::document::target()` currently echoes the oracle's true IDs
  verbatim for comparison. This would need to become "both sides renumbered by
  the same deterministic rule, then compared" — a change to the test's
  definition of its own oracle, not to canonical IR or persistence.
- A single, explicit, spec'd numbering rule in `A3_GRAMMAR.md` (e.g., "the Nth
  field encountered in file order, independent of class/method boundaries" vs.
  "the Nth field within this owner scope" — these produce different digit
  patterns and different decode logic; pick one and document it).
- Confirming (not assuming) that nothing else added later ever treats
  A1/A3-decoded text as an identity source. This is true today; it's a
  contract this change would newly depend on, so it should probably be written
  down as an explicit invariant once implemented (candidate for
  `docs/ARCHITECTURAL_INVARIANTS.md` per Matt's rule 17 — this generalizes past
  this one bug/feature).

This is exactly the kind of judgment call — "are we willing to formally
decouple wire-visible identity from persisted identity" — that belongs to
Matt's review, not something to implement silently. **Recommend Phase 1 of the
plan below be a design proposal + Matt's explicit go/no-go before any code.**

### 2. Scoped string table — types and callee-written-names only, not generic

Distinct from the rejected ULTRA/dictionary experiments (see above) by being
**narrowly scoped to the two specific columns** Phase 3's own anatomy findings
called out as repetition-heavy: parameter/field/return **type strings** and
**callee-written-names** in the `K` call stream. The prior rejections were
(as far as I could find) generic, all-value substitution attempts — different
scope, different result space. Per the research doc's own Section 11 step 8
("dictionary threshold sweep... allow 'no dictionary' below threshold") and
Section 9 (frequency-ranked / file-scope-local dictionaries are priority 2, not
rejected), this deserves one narrow, measured retry — with a hard rule: **only
emit the table if it empirically wins tokens on the real large fixtures**,
never on a theoretical character-count basis (that's exactly how ULTRA fooled
itself — 238 fewer characters, 115 more tokens).

### 3. Occurrence-group default elision (`mo`/`cs`/`lf`/`pf`)

The `K` call-stream already elides repeated defaults (unresolved, non-spread).
The occurrence-group tags haven't had the same treatment. Needs real frequency
data from a corpus run (Phase 0 below) before picking a specific default value
— don't guess what's "usually" the common case.

### 4. Row/record merging

Where two tags always co-occur (e.g. `M` immediately followed by its `p` row),
merging cuts one tag + one LF per occurrence. Real but marginal per-row; do
this last, after the bigger structural levers, since it compounds on top of
whatever the final row shape looks like rather than the reverse.

## Why binary/alternate transport encodings are not a lever (asked and answered 2026-09-22)

Don't spend time re-investigating "what if we sent binary/base64/a denser
alphabet instead of text." This is already closed:

- The model only ever consumes BPE tokens from its text tokenizer — there is
  no channel in a `content`/tool-use context that hands a model raw bytes for
  direct decode. Binary/base64 data doesn't match the byte-pair patterns a
  text/code-trained BPE vocabulary learned, so it typically tokenizes *worse*
  per byte of information, not better.
- `docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md` Section 13 already
  explicitly rejects this: "Opaque binary/base64 to the model: mechanically
  compact in bytes but generally token-expensive and hostile to reasoning;
  binary remains appropriate for storage, not assumed LLM input."
- The "ULTRA" candidate in that same doc is the closest thing tried to a
  denser/more binary-like scheme (dictionary-index substitution) and it
  measurably *increased* tokens (504 vs. 389 on the same sample) despite being
  238 fewer characters — direct evidence that byte/character compactness and
  token compactness are different axes, and chasing the former can actively
  hurt the latter.
- A denser synthetic/Unicode alphabet has the same predicted failure mode
  (Section 12, hypothesis 6: "ASCII mnemonic opcodes will often beat unusual
  Unicode glyphs despite longer character length").
- Rendering content as an image for a vision-capable model is mechanically
  conceivable but trades exact/lossless correctness for OCR-equivalent
  uncertainty — a direct violation of the hard correctness floor, untested
  here, and not a real candidate under this contract's constraints.
- The one *real* lever in this direction is categorically different from an
  "encoding": prompt-caching/schema-acknowledgment reuse, already scoped as
  `PHASED_A3_PLAN.md` Phase 6. It amortizes the fixed legend cost across a
  session once a client acknowledges a schema version — it does not change any
  per-response fidelity ceiling, and the plan is explicit that caching savings
  can never be used to make an otherwise-losing cold format look economical.

## Gap found in the measurement harness itself

`Measure-CompactA3.ps1` derives each capture's fidelity from the fidelity byte
already baked into that capture's `control-full.txt` — meaning each of the two
tracked large fixtures currently represents **one fixed fidelity**, not
Low/Medium/High/Edit separately, and not focused-vs-unfocused Edit. But
`PHASED_A3_PLAN.md`'s Checkpoint 3 explicitly requires results "by fidelity,
language, focus mode, and semantic density," and Matt separately confirmed this
coverage gap needs to be closed. **Any further optimization is currently
guessing which fidelity/focus combination is actually the worst offender.**
This should be closed before or alongside lever 1–3 work, not after, so effort
isn't spent optimizing the wrong candidate.

## Non-negotiables for whoever implements this

- `PHASED_A3_PLAN.md`'s 9 invariants (decode equality, canonical identity via
  reversible handles, ordered/duplicate-preserving calls and injections,
  workspace-edge exclusion, exact byte-framed Edit bodies, fidelity family
  separation, raw-fallback economics, no model calls before deterministic
  gates pass, A2/A3 schema separation).
- The research doc's dual gate (Section 7 lossless roundtrip, Section 8
  reasoning-fidelity non-inferiority): **"Lossless recovery is necessary, not
  sufficient."** A change that passes decode-equality but degrades the model's
  actual ability to reason about the content (e.g., a too-compressed
  reference the model misreads) is not a win. Section 8's task list and
  non-inferiority methodology should gate any change that touches how
  identity/relationships are *presented*, not just whether bytes round-trip.
- Matt's explicit correctness framing: "100% correctness is a hard boundary" —
  never trade correctness for a compression number, at any percentage.
