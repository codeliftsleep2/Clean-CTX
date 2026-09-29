# COMPACT-A3 Phase 3B — typed wire-local identity design

**Status:** approved by maintainer (2026-09-22); Phase 3B implementation authorized
**Predecessor:** Phase 3A measured — production-selected 19.30% o200k / 18.32% cl100k
**Scope:** A3 wire identity *spelling* only. No change to `next_id`, canonical IR,
focus resolution, persistence, delta/replay authority, or workspace-query identity.

## Objective

Replace the compiler's global `next_id` counter values that appear in A3 wire
identities (`C487`, `M1204`, `F82`, `P633`) with deterministic file-local
sequential ordinals (`c1`, `m3`, `f2`, `p1`) assigned by first appearance. The aim
is shorter, denser, cheaper-to-tokenize identity spelling on the model-facing wire
without changing what any identity *means*.

## Why this is gated

Global counter IDs grow with workspace size and are sparse within one file: a file
with three methods may carry `M1204`, `M1209`, `M1221`. Dense file-local ordinals
are one to three digits and tokenize cheaper. Renumbering is valid only as a
reversible presentation handle applied **after** canonical typed ownership has
resolved, and it must never leak outside the payload. Identity spelling is a
correctness boundary, so the rule, the scope, and the rejection cases must be
approved before any code.

## Current identity representation (baseline)

Canonical handles are `{C|I|F|M|P}{n}`, `n` from the global counter. Today:

| Record/column | Current spelling |
|---|---|
| `C`, `I`, `M`, `p`, `F` declaration rows | bare numeric `n` (family prefix stripped) |
| `X` (extends) | full typed handle `C1` / `I2` |
| `J` (implements) | full typed handle `I2` |
| `V` (behavior owner) | bare numeric `n` |
| `K` (caller run) | bare numeric `n` |
| `B` (body frame) | full typed handle `M{n}` |
| `D` (injection) | type/name strings — **no ID, unaffected** |
| `pt` (pattern) args | class/method ID references (`CTOR\|C6\|M8`) — **must be rewritten** |
| `pf` (pattern facts) | kind/value — **no ID, unaffected** |
| `$`/`T` (imports/aliases) | names — **no ID, unaffected** |

## Proposal: file-local renumbering

**Numbering rule.** Each typed family receives its own file-local ordinal sequence,
assigned by first appearance in wire (decode) order. The wire keeps emitting the
bare ordinal; the family is implied by the record tag or reference context exactly
as today. Only the number changes (`global counter value → file-local ordinal`).

| Family | Handle | Scope | Referenced by |
|---|---|---|---|
| class | `c1..cK` | file-local | `X`, `J` |
| interface | `i1..iI` | file-local | `X`, `J` |
| method | `m1..mM` | file-local (across owners) | `V`, `K`, `B`, `exact_body_method_ids` |
| field | `f1..fF` | **file-local** | data-flow targets (see validation) |
| parameter | `p1..pP` | method-local (proposed) | none (validated: no op references a param id) |

**First-appearance order.** Ordinals are assigned strictly in the order entities
first appear in the decoded stream: owners in `C`/`I` record order, then fields and
methods in their scoped order, parameters in `p`-row order. Re-numbering is
deterministic given the same decoded stream.

## Which declarations keep explicit vs. derived handles

- **Methods** keep an explicit handle in every `M` row; `V`/`K`/`B` reference it.
- **Classes/interfaces** keep an explicit handle in `C`/`I`; `X`/`J` reference it.
- **Fields** keep an explicit handle in the `F` row; the handle is **file-local**
  because field references can appear in data-flow targets (see validation).
- **Parameters** keep an explicit handle in the `p` row. *(Decision A: keep explicit
  method-local `p1..pP`, or drop to implicit position — see decision points.)*

Nothing is renumbered away from an explicit handle into positional derivation
without approval; the default is explicit-or-reversible, per invariant #2.

## Reference rewriting table

Every identity-bearing reference is rewritten to the target's local ordinal:

| Reference | Today | After | Resolution |
|---|---|---|---|
| `X` extends | `C1` / `I2` | `c1` / `i2` | class or interface ordinal (kind explicit in the reference) |
| `J` implements | `I2` | `i2` | interface ordinal |
| `V` behavior owner | `5` | `3` | method ordinal |
| `K` caller | `5` | `3` | method ordinal |
| `B` body method | `M5` | `m3` | method ordinal |
| `pt` pattern args | `C6`, `M8` | `c2`, `m1` | class/method ordinals (`args[0]`/`args[1]`) |
| `fd`/`fc` target | symbol string | symbol string | opaque — but any field id target must be rewritten (see validation) |
| `M`/`F`/`p` declaration | `5` | `3` | self ordinal (assign on first appearance) |

## Cross-reference validation

Before approval, the "who references whom" claims were checked against the IR
contract (`docs/architecture/CORE_OP_CONTRACT_MATRIX.md`) and the language-layer
emitters (`src/ir/layers/{typescript,rust,csharp}.rs`):

- **Method IDs** are referenced by `DataFlow`/`ControlFlow`/`SideEffect`/
  `ExecutionContext`/`Flags`/`Call`/`Body`/`PatternFacts` (the owner `mid`) — this
  is why methods are file-local.
- **Field IDs** can appear as data-flow/control-flow `target` symbols (the Rust
  layer documents "references to fields → DataFlow(reads/writes, field)"). Field
  scope is therefore **file-local**, never owner-local.
- **Class/interface IDs** are referenced by `Extends`/`Implements` and by pattern
  args (`args[0]`).
- **Pattern args** carry class and method IDs (`args[0]`, `args[1]`), so `pt` rows
  must be rewritten, not treated as opaque.
- **Parameter IDs** are not referenced by any op; method-local `p1..pP` is safe.
- **`fd`/`fc` target strings** are currently hardcoded symbols in the TS/C#/Rust
  emitters (`"observable"`, `"channel"`, `"db_query"`, `"await"`, `"loop"`), but the
  contract permits arbitrary target symbols, so the renumbering pass must rewrite
  any target that is a declared field/method ID and leave true symbol names
  untouched. This is a mandatory implementation detail, not an open question.

## Alpha-normalized equality

Decode-equality is redefined for the identity family. Instead of comparing decoded
IDs literally against the canonical oracle, both sides are normalized by the same
bijective renumbering (`canonical handle → local ordinal`), then compared
field-by-field for **all other** families (names, order, duplicates, occurrence
groups, spans, bodies remain literal-exact).

This redefinition is written into `A3_GRAMMAR.md` itself (identity spelling is
"bijectively equivalent under the documented renumbering rule", not literal string
equality). It is the only family whose equality semantics change.

## Payload-locality and identity authority

Local handles exist only within a single A3 payload:

- **Never persisted** — persistence stores canonical checked state; a local handle
  is never written back.
- **Never accepted as selectors** — `focusMethods` still resolves documented names
  against canonical typed ownership; the resolved result is then rendered with
  local handles.
- **Never returned as canonical workspace identity** — workspace-query and
  cross-file provenance keep the compiler's canonical IDs.
- **Never the source of `next_id`** — the compiler's global counter is untouched.

## Decoder rejection

The decoder fails closed on any identity-renumbering violation:

- duplicate local ordinal within a family/scope;
- missing ordinal (a gap in the sequence);
- wrong-family reference (a method reference resolving to a field ordinal, etc.);
- dangling reference (an ordinal with no declared target);
- out-of-range reference (ordinal greater than the declared family count).

## What is NOT changed

`next_id`, canonical IR shape, `CompiledIR`/`HierarchicalIR`, focus resolution,
persistence, delta/replay authority, workspace-query identity, and the fidelity
target matrix are all untouched. A3 remains research-only and unreachable from MCP
production.

## Measured token delta (Phase 3B)

Re-measured against the frozen Phase 3A baseline: **zero change** (18.32% cl100k /
19.30% o200k production-selected, byte-identical). The renumbering is applied
(verified: the wire emits dense `M|1`, `M|2`, … ordinals), but the qualifying
fixtures' canonical IDs top out at 55 (1–2 digits), which tokenize as one token —
the same as a single digit. The renumbering therefore changes the wire spelling
without changing the token count on this corpus.

The savings this lever was designed to capture require 4–5 digit global-counter
IDs, which only a much larger workspace produces. On the current qualifying corpus
the renumbering is economically neutral. It remains correct, deterministic (a
file's wire is now independent of the workspace's global counter), and fail-closed.

## Reasoning-fidelity risk

Any reasoning-facing identity regression rejects the design regardless of token
savings. Local ordinals are file-local and dense, so the model sees `m1`, `m2`, …
instead of `M1204`. This is expected to be non-inferior (and likely clearer), but
the bounded Phase 4 screen must confirm no identity/ownership/overload/DI/Edit
ambiguity before the design is accepted.

## Approved decisions (2026-09-22)

The maintainer approved the following, which govern Phase 3B implementation:

1. **Numbering scope** — per-family file-local (classes `c1..cK`, interfaces
   `i1..iI`, methods `m1..mM`, fields `f1..fF`, parameters `p1..pP`).
2. **Field scope** — file-local (field references appear in data-flow targets).
3. **Parameter scope** — method-local explicit `p1..pP`; implicit positional
   params are deferred (not part of Phase 3B).
4. **Pattern args** — `pt` args `args[0]`/`args[1]` rewritten as class/method
   ordinals.
5. **`fd`/`fc` target rewriting** — exact-match: rewrite a target only when it
   equals a declared field/method ID; leave other symbols opaque.
6. **Reference spelling** — `X`/`J` keep the kind prefix (`c1`/`i2`).
7. **Alpha-normalized equality** — identity family only; all other families stay
   literal-exact; the redefinition is written into `A3_GRAMMAR.md`.
8. **Rejection cases** — all five fail-closed cases (duplicate / missing /
   wrong-family / dangling / out-of-range).

## Checkpoint 3B (acceptance, post-approval)

- [x] The identity spelling change is explicitly approved before code changes.
- [ ] Alpha-normalized roundtrip equality covers every identity-bearing family.
- [ ] Same-name owners, overloads, calls, DI, and focused Edit remain unambiguous.
- [ ] The token delta is reported against the Phase 3A baseline.
- [ ] Any reasoning-facing identity regression rejects the design regardless of
  savings.
