# SCHEMA-vNext model-facing presentation proposal

**Status:** Proposed; investigation and measurement only. No production-format
change is authorized by this document.

**Scope:** The model-visible file-context presentation emitted by
`provide_code_context`, `compress_code_context`, restore/replay, and the full
response path of `delta_code_context`. This proposal does not replace canonical
IR, Binary `0x04`, `dv:2`, workspace queries, or exact-source modes.

**Primary authority:** Production code is authoritative for current behavior.
The documents and verification artifacts below provide intent and measurement
history:

- `src/ir/render_llm.rs`
- `src/mcp/tool_handlers/core/content.rs`
- `src/mcp/content_economics.rs`
- `src/mcp/prompts.rs`
- `docs/ARCHITECTURAL_INVARIANTS.md`
- `docs/architecture/LLM_CONTEXT_COMPRESSION_RESEARCH.md`
- `verification/context-compression/schema-v5/`

---

## 1. Executive position

SCHEMA-v5 is a sound correctness-first presentation with a strong final
economics boundary: when the complete candidate is not safely cheaper than raw
source under the selected local tokenizer, raw source wins. It is not yet the
best achievable model-facing encoding.

The next work should not remove reasoning facts. It should target four kinds of
avoidable cost:

1. facts or values already stated elsewhere in the same presentation;
2. internal handles that no model-visible record references;
3. repeated structural syntax with no additional meaning;
4. fixed schema/path text that can be removed only through an explicit,
   verifiable transport contract.

The likely outcome is incremental improvement, not another order-of-magnitude
reduction. The current semantic payload dominates large High-fidelity files.
Substantial additional savings without a grammar change would probably require
semantic omission and are therefore out of scope.

---

## 2. Current production boundary

The representation layers remain deliberately separate:

```text
Canonical semantics
  CompiledIR + aligned semantic edges
          |
          +--> Binary 0x04 / dv:2        code-side persistence and transport
          |
          +--> workspace_query           on-demand cross-file and detailed facts
          |
          `--> SCHEMA-v5                 model-facing file-local presentation
                    |
                    `--> raw source      when the complete presentation is not
                                          safely cheaper, or when explicitly
                                          requested as Verbatim
```

SCHEMA-vNext must remain a projection. It must not become canonical storage,
an edit protocol, a delta protocol, or a substitute for `workspace_query`.

### Current measured checkpoint

The last recorded raw-to-presentation densities were:

| Fixture | Low | Medium | High |
|---|---:|---:|---:|
| TypeScript | 76.3% | 70.1% | 67.1% |
| Angular | 73.9% | 64.6% | 61.8% |
| C# | 64.2% | 48.4% | 43.9% |

These values predate the corrected Observable/Promise classification and the
new presentation collapse for return-derived Observable facts. They are no
longer an acceptable baseline for a vNext decision. A fresh capture is Phase 0.

The accompanying research prose that attributes `pf:OBSERVABLE` to ordinary
C# async methods is likewise stale after `IRFACT-002` and must be corrected when
the new baseline is recorded.

---

## 3. Non-negotiable invariants

Every candidate must preserve these boundaries:

1. **No semantic omission disguised as compression.** Required facts for the
   selected fidelity remain available either in the presentation or through an
   already-declared on-demand boundary.
2. **No fabricated identity.** Presentation scoping may elide internal IDs only
   after checked ownership has resolved them; it may never replace unresolved
   identity with a guess.
3. **No loss of order or duplicates where they are meaningful.** A shorter
   layout cannot silently convert an ordered occurrence stream into a set.
4. **No exact-body transformation.** Edit bodies remain byte-exact according to
   the existing body contract. Verbatim remains the whole-document escape hatch.
5. **No assumed conversational state.** A prior response, prompt-resource read,
   or cache hit is not schema acknowledgement.
6. **No tokenizer-by-character reasoning.** Every candidate is measured using
   actual target tokenizers or an explicitly bounded approximation.
7. **Raw continues to win ties.** The final economics selector remains outside
   the renderer and compares complete alternatives with the same tokenizer.
8. **Canonical state is unchanged.** Presentation collapses do not remove facts
   from `CompiledIR`, hierarchy, Binary `0x04`, persistence, or `dv:2`.

---

## 4. Static cost findings

### 4.1 Fixed per-response costs

The current presentation pays for all of the following on every structural
response:

- the full SCHEMA-v5 inline legend;
- decorative class boundaries;
- a file boundary containing the full source path;
- `§PATHMAP`, which repeats that same full path;
- an additional interface legend when interfaces are present.

Fixed costs matter most on small and medium files, although the raw-selection
gate already prevents an uneconomical structural response in most unfocused
modes.

### 4.2 Repeated per-declaration costs

The current method grammar emits an initial scope arrow even though the `M`
record already establishes method scope:

```text
M methodB  → → void
M find  → p:id:string → Order
```

The first arrow is repeated for every method and carries no independent fact.
Fields repeat `F` and a newline at Medium and High even though Low already has
an unambiguous grouped form.

### 4.3 Presentation-only semantic duplication

Observable return classification is now correctly retained in canonical IR but
suppressed when the signature already renders `Observable<T>` or
`IObservable<T>`. The same principle is not yet applied consistently:

- `P PROMISE ... Promise<T>` can repeat a `→ Promise<T>` return;
- constructor/getter/setter patterns may repeat declaration structure;
- an override pattern may repeat an authoritative modifier;
- pattern rows expose class/method IDs even though checked hierarchical
  placement already identifies their owner.

Not every pattern row is redundant. Constructor pattern payloads can carry
dependency names, for example. Each family must be reduced according to its
meaning rather than globally deleted.

### 4.4 Unreferenced visible handles

Imports currently render an internal alias:

```text
$ IM1 rxjs [Observable, of]
```

No other SCHEMA-v5 record refers to `IM1`. If corpus verification confirms that
the value is never a source-written import alias, the presentation can render:

```text
$ rxjs [Observable, of]
```

Canonical import identity remains untouched.

---

## 5. Candidate set

Candidates are grouped by risk. Each item is measured independently before a
combined candidate is permitted.

### Tier A — narrow redundancy removals

#### A1. Single path occurrence

Current:

```text
// ── α1 (C:\workspace\src\service.ts) ──
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

Candidate:

```text
// α1
§PATHMAP
  α1 = C:\workspace\src\service.ts
```

The alias remains visible at the file boundary and the authoritative mapping
still supplies the exact path. Multi-file outputs remain request-scoped.

#### A2. Return-derived pattern collapse

When a method's rendered return type already establishes the same contract,
suppress only its visible redundant classification:

```text
P PROMISE C1 M1 Promise<User>
M load  → → Promise<User>
```

becomes, before the method-grammar experiment:

```text
M load  → → Promise<User>
```

The canonical `PatternOp::Promise` remains present. Observable already follows
this rule under `IRFACT-002`.

#### A3. Remove unreferenced import handles

Drop the internal import handle only after a production-corpus assertion proves
that it is not referenced elsewhere in the visible document and is not a
source-written alias with user-facing meaning.

#### A4. Strip checked pattern-owner IDs

Pattern placement already occurs after checked class/method ownership has been
resolved. Visible rows should retain only pattern-specific payload:

```text
P CTOR C1 M1 repo
```

candidate:

```text
P CTOR repo
```

This is safe only for the model-facing projection. Canonical IDs remain in the
pattern operation and all code-side forms.

### Tier B — grammar revision

Tier B changes the recurring grammar and should be versioned as SCHEMA-vNext
rather than silently redefining SCHEMA-v5.

#### B1. Remove the method scope arrow

Candidate:

```text
M methodB → void
M find p:id:string → Order mod:PUBLIC ctl:RET
```

The single remaining arrow always introduces the return type. Parameters are
introduced by `p:`. This removes repeated syntax and the visually confusing
`→ →` shape without making the grammar positional or ambiguous.

#### B2. Replace decorative class comments with a typed record

Current:

```text
// ── OrderService ──
```

Candidate:

```text
C OrderService
```

`C` becomes the class counterpart to the existing `Q` interface record. This
is expected to tokenize more predictably than comment punctuation and box-drawing
characters, but the target tokenizers decide.

#### B3. Group fields at all body-free fidelities

Candidate:

```text
F id:number name:string email:string
```

Field order and type information remain intact. If reasoning tests show that a
single field per line materially improves selection or comparison, this item is
rejected regardless of token savings.

#### B4. Tokenizer-selected marker vocabulary

Measure complete alternatives for labels such as `mod:`, `ctl:`, `df:`, and
`se:`. Do not assume the shortest character spelling is cheapest. Unicode,
punctuation-heavy, and numeric opcode candidates must beat readable ASCII
mnemonics on the actual tokenizers and pass the same reasoning suite.

### Tier C — acknowledged stable schema

The full inline legend is a substantial cold fixed cost. It may be replaced by
a minimal version/digest only when the client explicitly acknowledges the exact
schema contract that the model will receive.

Illustrative negotiation:

```text
client acknowledges: schema=clean-ctx-vNext digest=<exact digest>
server emits:        // SCHEMA vNext <digest-prefix>
```

Neither MCP prompt-resource availability, a previous response, nor provider
prompt caching is acknowledgement. If the host cannot prove schema visibility,
the complete cold legend remains inline.

An adaptive legend containing only markers used in the current payload may be
tested as a separate cold-mode candidate. It must remain self-contained and
must not make absence of a marker look like absence of a fact family.

---

## 6. Illustrative vNext presentation

This example demonstrates the combined direction; it is not an approved wire
grammar.

### Current shape

```text
// SCHEMA v5  @=meta X=extends I=implements F=field M=method $=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
// ── OrderService ──
cmod: EXPORT
F repo:OrderRepository
M load  → p:id:string → Promise<Order> mod:ASYNC ctl:RET cf:await:await se:io
$ IM1 rxjs [Observable, of]
// ── α1 (C:\workspace\src\order.service.ts) ──
§PATHMAP
  α1 = C:\workspace\src\order.service.ts
```

### Candidate shape

```text
// SCHEMA vNext C=class Q=interface F=field M=method $=import p:=params →=return
C OrderService
cmod:EXPORT
F repo:OrderRepository
M load p:id:string → Promise<Order> mod:ASYNC ctl:RET cf:await:await se:io
$ rxjs [Observable,of]
// α1
§PATHMAP
α1=C:\workspace\src\order.service.ts
```

The candidate still exposes every fact shown by the current example. Whether
each punctuation and spacing choice saves tokens is an empirical question.

---

## 7. Correctness prerequisites discovered during the audit

Compression work must not make questionable producer facts harder to notice.
Before final vNext evaluation, audit at least:

1. **`tap(...)` and `se:io`:** RxJS `tap` establishes a side-effect hook, but
   does not by itself prove external I/O. The existing producer may be
   semantically over-specific.
2. **Modifier/body substring recognition:** confirm that language-layer
   recognizers cannot classify keywords found only in comments, strings, or
   neighboring declarations.
3. **Pattern-family redundancy:** classify which payloads add information
   beyond signatures/modifiers and which are pure restatements.
4. **Import-handle meaning:** prove that the first import value is always an
   internal handle before removing it from the presentation.

These are correctness investigations. They must use RED/GREEN regression
discipline if a reproducible defect is confirmed.

---

## 8. Measurement and reasoning gate

### Phase 0 baseline

Capture the post-`IRFACT-002` production output without changing the renderer.
Record by fixture, language, fidelity, and tokenizer:

- raw-source tokens;
- complete SCHEMA-v5 candidate tokens;
- selected representation and selected tokens;
- fixed legend tokens;
- file/path tokens;
- declaration/signature tokens;
- behavior-fact tokens;
- imports/type-alias tokens;
- exact-body tokens for Edit.

The old density table remains historical evidence, not the comparison baseline.

### Candidate isolation

Apply one candidate at a time to captured canonical fixtures. For every row,
record absolute and percentage deltas against both raw source and the fresh
SCHEMA-v5 baseline. Only then measure a combined candidate to detect interactions
between tokenizer merges.

### Tokenizers

At minimum:

- `cl100k_base`;
- `o200k_base`;
- the native tokenizer/counting API for the Claude model used in field tests;
- Llama only when a bounded local or native counter is available.

Character count is diagnostic only.

### Deterministic correctness gates

Each candidate must pass:

1. exact expected presentation facts for every fixture/fidelity;
2. owner and overload disambiguation cases;
3. order and duplicate-preservation cases;
4. focused Edit body and `apply_edit` round trips;
5. raw-fallback and focused-no-raw-fallback economics contracts;
6. restore/replay regeneration from Binary `0x04` and `dv:2` state;
7. malformed/truncated-presentation rejection where a decoder or grader exists.

### Model reasoning gates

Run the existing corrected reasoning matrix and task-based edit suite using the
same model/version, system instructions, question order randomization, and
scoring rules for baseline and candidate. Add explicit cases for:

- reading the vNext method signature without the scope arrow;
- distinguishing class and interface scopes under `C`/`Q`;
- selecting one field from a grouped field row;
- associating a pattern payload with its method after owner IDs are hidden;
- requesting `workspace_query` for facts deliberately outside file context;
- requesting Edit or Verbatim when the structural presentation is insufficient.

Zero tolerance remains in force for wrong ownership, wrong edit target,
fabricated resolution, lost duplicates/order, or source corruption.

### Live gate

For any externally visible grammar change, Claude must consume the candidate on
the real approximately 100K-line pilot workspace. The field test should include
at least one large TypeScript/Angular file, one declaration-dense file, and one
focused edit. Laboratory token wins do not replace this gate.

---

## 9. Decision matrix

| Candidate | Expected impact | Correctness risk | Implementation scope | Initial recommendation |
|---|---|---|---|---|
| A1 single path | Small fixed | Very low | content assembly + contracts | Measure first |
| A2 Promise collapse | Small-to-medium on async-heavy TS | Very low | renderer + tests | Measure first |
| A3 import handle removal | Small recurring | Low pending meaning audit | renderer/prompt/tests | Audit then measure |
| A4 pattern owner-ID stripping | Medium on pattern-dense files | Low-to-medium | renderer/reasoning tests | Measure independently |
| B1 one method arrow | Medium recurring | Low | versioned grammar/prompt/tests | Highest-priority vNext experiment |
| B2 `C` class record | Small-to-medium | Low | versioned grammar/prompt/tests | Pair with B1 only after isolation |
| B3 grouped fields | Workload-dependent | Medium reasoning risk | renderer/reasoning tests | A/B test |
| B4 marker vocabulary | Unknown | Medium | broad grammar changes | Micro-benchmark before design |
| C acknowledged legend | Large fixed | High transport risk | MCP/client contract | Defer until host trace |

---

## 10. Rollout plan

### Phase 0 — refresh truth

1. Capture the current production presentation after all recent correctness
   fixes.
2. Update stale measurement prose and retain the prior numbers as historical.
3. Record per-family token anatomy.

### Phase 1 — Tier A experiments

1. Implement candidates in the measurement harness first.
2. Measure each independently on the full corpus.
3. Run deterministic and model reasoning gates for candidates that win.
4. Approve and implement only the individually proven subset.

### Phase 2 — versioned grammar experiment

1. Define a complete vNext cold legend and grammar.
2. Prototype B1, B2, and B3 independently.
3. Select marker spellings from tokenizer results, not visual preference.
4. Run the complete laboratory and live gates.
5. If approved, update renderer, MCP prompt resource, vocabulary resource,
   tests, examples, and documentation atomically.

### Phase 3 — optional stable-schema acknowledgement

1. Trace what the real MCP host places in model context.
2. Specify explicit version/digest acknowledgement.
3. Preserve cold self-contained output for all unacknowledged clients.
4. Measure cached and uncached economics separately.

---

## 11. Compatibility and lifecycle requirements

- SCHEMA-v5 remains the production format until a candidate clears every gate
  and receives explicit architectural approval.
- A grammar-changing candidate receives a new visible version; it does not
  silently reinterpret `SCHEMA v5`.
- Persistence does not migrate stored presentation text. Restore/replay decode
  canonical state and regenerate the current approved presentation.
- Binary `0x04`, `dv:2`, semantic-edge persistence, and workspace-query
  contracts do not change.
- Exact-body and `byte_exact` metadata contracts do not change.
- Cache breakers must include the presentation version so cached v5 and vNext
  payloads cannot be confused.
- Raw fallback remains byte-exact and carries no schema wrapper or `PATHMAP`.

---

## 12. Stop conditions

Reject or defer a candidate when any of the following occurs:

- it loses a required fact, occurrence, owner, order, or duplicate;
- it makes an edit target ambiguous;
- it depends on conversation memory or an unverified prompt-resource read;
- it increases selected tokens on the qualifying corpus;
- it wins aggregate tokens only by losing one language/fidelity family badly;
- it reduces reasoning reliability or increases unsupported claims;
- it adds a dictionary/table whose own legend and lookup cost erase the win;
- it requires canonical, persistence, or delta changes merely to shorten the
  model-facing projection.

“SCHEMA-v5 is already better for this candidate class” is a valid result.

---

## 13. Explicitly rejected approaches

- learned or perplexity-based token deletion;
- body summarization in place of exact Edit content;
- opaque binary/base64 model input;
- one global dictionary emitted for every response;
- dropping High-fidelity facts solely to improve density;
- using raw source as focused-Edit fallback;
- relying on automatic delta or prior conversation state to reconstruct a
  complete file view;
- declaring a win from character count or from comparison against verbose
  CONTROL-FULL rather than raw source and the current production presentation.

---

## 14. Open decisions

1. Should proven Tier A presentation collapses land under SCHEMA-v5, or should
   every externally visible textual change wait for vNext?
2. Is the first import operand always internal presentation machinery?
3. Which pattern payloads remain reasoning-relevant after their declaration
   signature and modifiers are visible?
4. Does grouping fields degrade real model selection or comparison tasks?
5. Can Claude's native token counter be used in the repeatable local/field
   measurement loop?
6. Does any supported MCP host reliably expose server initialization
   instructions or acknowledged prompt resources to the model?
7. Is a minimal cold header plus optional detailed vocabulary more reliable
   than a dynamically generated used-marker legend?

These questions are measurement and architecture gates, not details for an
implementer to decide implicitly.
