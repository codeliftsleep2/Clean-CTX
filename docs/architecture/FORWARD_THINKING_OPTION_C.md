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

## ADR-001 — Owner-aware file-local call inspection for Option C

**Status:** Implemented on the production `workspace_query` path; final Cargo
verification and Option-C task-based evaluation remain pending.

**Decision date:** 2026-09-25

### Context

Option C aims to keep normal model-visible file context focused on file-local,
reasoning-necessary structure while moving detailed facts behind on-demand
queries. Call facts expose a boundary defect in that goal.

Canonical `CompiledIR` retains the facts needed for precise local inspection:

- caller method ID and therefore typed owner;
- ordered call occurrences and duplicates;
- callee name exactly as written;
- explicit written argument count;
- whether spread makes that count non-exact.

The generic semantic projection intentionally converts the caller to the Model
C WorkspaceIndex identity `(domain, entity_type, name)`. For methods that means
`(builtin, Method, "run")`: the typed owner and overload identity are no longer
present. Consequently, `forward_edges` cannot distinguish `Alpha.run` from
`Beta.run`, and cannot isolate one overload family by signature.

The existing `withinPath` narrowing remains valuable because edge occurrences
retain asserting-file provenance. It resolves many collisions where same-named
callers live in different files. It cannot resolve:

- same-named methods owned by different types in one file;
- overloads under one owner;
- queries where the caller file is not yet known;
- owner-qualified ordering when several matching callers share the narrowed
  provenance.

This is a real correctness limitation, but the current evidence does not show
that its irreducible form is frequent enough to justify changing global
semantic identity. The verification fixtures deliberately stress collisions;
they prove the defect exists, not its population frequency. Recorded live
same-name property-arrow callers were in different paths and were successfully
separated by file narrowing.

### Decision

Preserve the existing WorkspaceIndex identity and cross-file query contract.
Do not attempt to recover typed ownership by adding a filter after projection;
the information has already been discarded at that boundary.

For the narrow case, introduce a focused, logically read-only file-local call
inspection operation backed directly by canonical IR. It is exposed as the
`calls_in_file` operation of `workspace_query` (also described conceptually as
**WithinFile**). This keeps trusted workspace scoping and the existing query
answer envelope while giving the operation a distinct canonical-IR authority.

The operation is keyed by:

```text
trusted file path
+ typed owner kind and name
+ method name
+ optional visible signature selector
```

It returns call occurrences grouped by visible overload signature. An
incomplete selector returns the complete matching overload family rather than
guessing. The result preserves order, duplicates, written arity, and spread,
without exposing canonical IDs or claiming resolved callee identity.

Illustrative request:

```json
{
  "type": "calls_in_file",
  "filePath": "src/example.ts",
  "workspaceRoot": "...",
  "owner": { "kind": "class", "name": "Alpha" },
  "method": { "name": "run" }
}
```

`method.parameters`, when present, is an exact ordered selector over the
visible parameter-signature strings returned by the canonical projection.
This deliberately uses `parameters`, not a claimed `parameter_types` field:
some language frontends currently preserve the complete written parameter
(for example `string value`) where no separately normalized type exists.
`method.return_type` is an optional exact selector. Omitting both selectors
returns the complete same-name overload family in declaration order.

Illustrative semantic result:

```json
{
  "file": ".../src/example.ts",
  "owner": { "kind": "class", "name": "Alpha" },
  "method": "run",
  "overload_count": 1,
  "overloads": [
    {
      "overload_occurrence": 0,
      "parameters": ["string"],
      "return_type": "void",
      "calls": [
        {
          "occurrence": 0,
          "callee_written": "lookup",
          "explicit_argument_count": 1,
          "has_spread": false
        }
      ]
    }
  ],
  "count": 1
}
```

This is the implemented wire contract. `count` is the total number of returned
call occurrences; `overload_count` counts matching declarations. A missing
typed owner or signature returns an empty result. More than one owner with the
same requested kind and name is an invalid/ambiguous request (`-32602`), never
a guessed match.

The handler reuses trusted-root resolution and the shared optional
`withinPath` narrowing, compiles a High-fidelity canonical candidate through
the normal source-read boundary, and does not run WorkspaceIndex hydration.
The candidate is not published as alias, version, IR baseline, semantic edge,
or WorkspaceIndex state. The operation does not acknowledge deltas, mutate
source, expose canonical IDs, or invent overload/callee resolution.

### Alternatives considered

#### 1. Change WorkspaceIndex method identity to include owner/signature — rejected

This would be a repository-wide semantic-identity migration, not a query
extension. It would affect projection, entity registration, edge deduplication,
hydration, traversal, cache identity, every query consumer, public response
semantics, and existing cross-language invariants. Current frequency evidence
does not justify that blast radius.

#### 2. Add an owner filter to `forward_edges` — rejected

A filter cannot recover owner data that semantic projection did not retain.
Adding a field to the request without changing stored identity would create the
appearance of precision without the evidence to enforce it.

#### 3. Rely exclusively on `withinPath` — rejected as the complete solution

File provenance resolves cross-file collisions and remains the preferred first
narrowing step. It does not distinguish owners or overloads inside one file.

#### 4. Keep detailed calls in every Option-C presentation — rejected as the default

This preserves availability but charges every model-visible response for facts
needed only by some tasks. It also prevents the intended on-demand separation.
Call facts may not be removed from presentation merely on the promise of a
future query, however: the WithinFile operation must be implemented, reachable,
and reasoning-tested before Option C relies on it.

#### 5. Remove calls and rely on current `workspace_query` — rejected

This would knowingly lose owner-qualified call order, duplicates, and overload
separation. A coarse name bucket is not an equivalent authority.

#### 6. Create a generic WithinFile query framework — rejected for now

Only one concrete missing capability is established. A generic file-query
registry or multi-operation abstraction would be premature. Start with
`calls_in_file`; generalize only when another independently justified
file-local fact family appears.

### Consequences

**Benefits:**

- closes the precise owner/overload ambiguity without changing global identity;
- uses facts already present in canonical IR;
- preserves call order, duplicate occurrences, arity, and spread honestly;
- allows Option C to avoid paying the call-detail token cost on every response;
- keeps cross-file graph queries and file-local exact inspection as distinct
  authority boundaries.

**Costs and limitations:**

- tasks needing detailed calls require an additional tool invocation;
- each request performs canonical candidate compilation through the normal
  metadata-invalidated source-read boundary rather than reading WorkspaceIndex;
- the written callee remains unresolved unless a separate resolver proves it;
- Option C needs task evaluation proving that models discover and use the
  operation when call detail is required.

### Frequency and promotion gate

No claim is made that the irreducible collision is common. Before considering
any broader identity migration, measure representative repositories for:

1. call-bearing method-name collision rate;
2. the fraction resolved by file-level `withinPath` narrowing;
3. the residual same-file owner and overload collision rate;
4. actual tasks requiring ordered/duplicate/spread-aware outgoing calls;
5. response pollution when querying only by bare method name.

The focused WithinFile operation is acceptable even at modest frequency because
its boundary is narrow and correctness-preserving. A global WorkspaceIndex
identity migration remains unjustified without materially stronger field
evidence.

### Relationship to Option C and R-46

Option C may defer detailed local call facts only after `calls_in_file` exists
on the real production path and passes task-based evaluation. The production
path now exists; task-based evaluation is still required before Option C may
remove detailed calls from its default presentation. The generic cross-file
operations remain intentionally owner-incomplete; `calls_in_file` is the
owner-aware authority for this narrow file-local question.

This decision is independent of R-46. It neither migrates legacy result-level
fields nor changes delta transport. Delta remains entirely code-side.

---

## 1. Where we are

- **Option A is implemented and verified.** `content` is the SCHEMA-v5
  structural presentation (`render_hierarchical_for_llm`), with explicitly
  classified raw-source, Angular-template, and delta-acknowledgement forms on
  their dedicated paths. Canonical semantics are `CompiledIR` plus semantic
  edges; binary `0x04` plus the aligned edge snapshot is durable authority;
  reduced `result.ir` is non-reversible auxiliary output. CONTROL-FULL is the
  regenerated correctness oracle, A1/A3 are research codecs, and A2
  `pretty_text` is non-authoritative.
- **The annotation-redundancy collapse landed.** The `async` fact was reported
  three times (`mod:ASYNC` + `se:async` + `ec:async`); it is now reported once,
  at the most structural level available (`mod:ASYNC`, else `se:async`). This is
  presentation-only — canonical IR retains all three. High-fidelity density
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
`workspace_query`; detailed owner-qualified local calls → the accepted
WithinFile operation once implemented; canonical IDs (`C1/M1/F1/P1`) →
regenerate positionally (code-side); occurrence groups + duplicates → code-side
(round-trip); navigation index → code-side; grammar legend → code-side; body
byte-length framing → code-side.

WithinFile is now implemented as `workspace_query.calls_in_file`. Until it is
reasoning-tested, Option C must not rely on tool discoverability as justification
for removing detailed calls from the default presentation.

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
vs CTX-001 (canonical semantics and representation authorities).

Detailed call removal is additionally gated on ADR-001: the focused WithinFile
operation must be production-reachable and task-verified first.

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
