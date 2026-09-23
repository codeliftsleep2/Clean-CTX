# COMPACT-A3 sparse positional phase plan

**Status:** Phase 1 complete; Phase 2 green; initial Phase 3 economics failed;
Phase 3A matrix measured: production-selected 19.30% o200k / 18.32% cl100k;
Phase 3B complete: identity renumbering applied, token delta 0.00% (IDs ≤ 55);
Phase 3C negative: scoped type/callee table removed (see `A3_PHASE3C_TABLE.md`)
**Predecessor:** A2 stopped after failing raw-source economics
**Scope:** model-facing file-context representation only. Canonical IR,
normalized semantic objects, persistence authority, workspace-query authority,
and exact Edit source remain unchanged.

## Finding that authorizes A3

A2 did not fail because its legend or correctness envelope was inherently too
large. Its complete header and legend measure 146 cl100k / 147 o200k tokens.
The candidate failed because it only made the outer records positional while
retaining verbose nested JSON objects, fixed-width empty columns, repeated call
defaults, and fidelity-insensitive layouts.

Measured o200k evidence on qualifying tracked files:

| Fixture | Raw | SCHEMA v5 | A2 |
|---|---:|---:|---:|
| `LargeService.ts` | 3,060 | 1,080 | 3,610 |
| `UserManagementService.ts` | 4,091 | 1,804 | 6,229 |

SCHEMA v5 demonstrates that substantial compression remains possible. It is
not itself an acceptable correctness oracle because it omits stable identities
and local semantic families. A3 must combine its scoped textual density with
the normalized correctness boundary.

## Non-negotiable invariants

1. A3 decodes to the fidelity-appropriate normalized file-context object.
2. Canonical typed identity is retained through explicit or deterministically
   reversible scoped handles. Display names never become identity.
3. File-local calls and injection occurrences remain available, ordered, and
   duplicate-preserving. Unresolved callees remain explicitly unresolved.
4. Workspace edges, reverse/forward dependencies, multi-hop results, framework
   edges, and cross-file provenance are requested through `workspace_query` and
   are not copied into ordinary file context.
5. Exact Edit bodies are separate length-framed UTF-8 segments with exact
   method identity and byte spans. Focused Edit emits only resolved target
   bodies plus non-target skeletons.
6. Low, Medium, High, and Edit have distinct declared layouts. A lower fidelity
   must not silently serialize High-only families.
7. Every non-focused production candidate must be strictly cheaper than the
   byte-exact raw source under the local tokenizer estimate and safety buffer;
   otherwise production returns raw source.
8. No model calls occur before exact deterministic roundtrip, lifecycle, and
   raw-source economics gates pass.
9. A2 remains frozen. A3 uses a new schema/version and cannot be decoded as A2.

## Target notation principles

- Use a short schema-once legend, targeting no more than 200 tokens and failing
  the deterministic gate above 300 tokens.
- Use scoped owner blocks so owner IDs are stated once.
- Convert every nested record to positional form: parameters, fields, imports,
  aliases, patterns, control/data flow, effects, contexts, and injections.
- Use sparse tagged method facts rather than fixed-width rows full of empty
  arrays and nulls.
- Group calls by caller. Occurrence order is row order; omit the repeated
  unresolved default and encode spread only when true.
- Use explicit counts or framing where truncation could otherwise change scope.
- Include navigation only when a controlled locality test proves it improves
  reasoning and its token cost is justified. Never copy fact values into it.
- Keep the cold payload independently understandable. Stable prompt caching is
  a later optimization, never required for correctness or cold economics.

## Phase 0 — freeze evidence and define the target

1. Retain A2 captures and token records as failure evidence.
2. Record token anatomy independently for legend, declarations, local calls,
   navigation, imports/types, and bodies.
3. Define the fidelity-specific normalized target objects. This is semantic
   projection, not deletion from canonical IR.
4. Specify the complete A3 grammar before changing production rendering.
5. Specify malformed/truncated input rejection and version mismatch behavior.

### Checkpoint 0

- A2 failure numbers and SCHEMA-v5 comparison are reproducible.
- A3 legend and every nested row grammar are explicit and versioned.
- The required semantic families for each fidelity are documented.
- No production path emits A3 yet.

Checkpoint 0 is complete. `A3_GRAMMAR.md` is the versioned grammar and fidelity
target. The reproduced A2 anatomy is recorded in `A3_PHASE0_BASELINE.md`; it
confirms that declarations and ungrouped call rows, not the legend, dominate
the failed candidate.

## Phase 1 — complete positional encoding

Implement a research encoder/decoder over normalized fixtures:

- file, class, interface, field, method, and parameter rows;
- occurrence-grouped modifiers and behavioral facts;
- imports, type aliases, patterns, and local DI;
- caller-grouped local calls with written arity and spread evidence;
- exact body frames and spans.

Do not use generic object-key substitution. Each family receives an explicit
typed row schema. Empty optional families are omitted, not serialized as empty
fixed columns.

Phase 1A implements the research-only header, typed class/interface scopes,
fields, method signatures, parameters, inheritance/implements, grouped
modifiers, patterns, and core injection occurrences. It is intentionally not
wired into MCP production. Phase 1B implements sparse High/Edit behavior,
caller-run local calls, positional imports/types, and exact body frames as
merge-ready research records. Phase 1C composes 1A and 1B into one framed
High/Edit document and verifies full normalized-target equality, terminal
counts, body framing, and cross-record method references.

The Phase 1 completion matrix runs that document through the real checked
CONTROL-FULL normalizer fixture. It covers overload identity, duplicate DI and
call occurrences, spread evidence, caller-run switches, Edit body retention,
and the requirement that High never expose canonical body facts retained
internally by the hierarchy.

### Checkpoint 1

- Decode equals the normalized target after object-key-order normalization.
- Ordered arrays, duplicates, and occurrence-group boundaries are exact.
- Identity references resolve to exactly one entity of the required type.
- Unresolved calls remain unresolved.
- Bodies and spans are byte/numeric exact, including CRLF and Unicode.
- Malformed, truncated, unknown-version, and scope-invalid streams fail closed.

## Phase 2 — fidelity-specific sparse layouts

Define and test separate layouts:

- **Low:** overview structure and required local relationships only.
- **Medium:** implementation/debug signatures and mapped semantic families.
- **High:** complete body-free file reasoning envelope.
- **Edit:** High structure plus all or resolved focused exact body frames.
- **Verbatim:** raw source; A3 is not involved.

The encoder must receive the effective fidelity and omit families outside that
mode's approved semantic target. The decoder must reconstruct the corresponding
normalized target, not pretend omitted High-only families were present.

The implemented projection now makes these distinctions explicit: Low omits
parameter rows only for unique-name methods while retaining complete
same-owner overload signatures; Medium retains every parameter; Low and Medium
omit detailed control/data flow, effects, and execution contexts; High retains
those families without bodies; Edit adds only the selected/all exact bodies.

### Checkpoint 2

- Low and Medium are measurably smaller than High on the same file.
- Each mode roundtrips its declared target exactly.
- Edit body selection is canonical-ID based and byte-exact.
- Fidelity override, intent mapping, restore, and replay preserve the mode.

## Phase 3 — deterministic economics

Measure without model calls:

- exact cl100k and o200k counts;
- calibrated Claude estimate plus the existing safety buffer;
- byte-exact raw source versus cold self-contained A3;
- SCHEMA-v5 and A2 only as diagnostic references;
- results by fidelity, language, focus mode, and semantic density;
- legend and family marginal costs separately.

Use genuinely large TypeScript/Angular and C# fixtures. Small correctness files
remain in the roundtrip lane but do not determine aggregate economics.

### Checkpoint 3

- Legend is at most 300 tokens, with a target of 200 or fewer.
- The raw ceiling selects raw for every losing non-focused case.
- Production-selected output across the qualifying representative large-file
  corpus saves at least 50% in aggregate against byte-exact raw. There is no
  maximum savings target.
- Every invocation still applies the local economics gate and selects raw when
  the candidate is not cheaper by the configured safety margin. Per-file,
  fidelity, intent, language, and focus rows remain visible so the aggregate
  cannot hide an inflation bug or unsupported mode; those rows are not
  independent 50% requirements.
- Qualifying large TypeScript, Angular, and C#/.NET fixtures are all required;
  they are the primary production scope. Java/Spring remains secondary unless
  existing coverage is inexpensive to retain.

The first complete encoding failed this checkpoint. Subsequent lossless density
iterations added packed caller runs, implicit physical-order ordinals, packed
field/parameter rows, bare strings with JSON fallback, and removal of redundant
typed-ID prefixes. The current tracked High-fidelity baseline is:

| Tokenizer | Raw aggregate | A3 aggregate | Reduction |
|---|---:|---:|---:|
| cl100k | 6,869 | 5,150 | 25.03% |
| o200k | 7,151 | 5,221 | 26.99% |

The current individual o200k diagnostics are 38.20% for `LargeService.ts` and
18.60% for `UserManagementService.ts`. These are diagnostic rows, not separate
50% gates. The aggregate remains below the required 50%, so Phase 4 is blocked.
The legend is about 120–130 tokens and is not the dominant problem. A generic,
byte-profitable repeated-string dictionary was tested and rejected because it
increased o200k cost. Remaining cost is concentrated in calls, fields, methods,
parameters, imports, and types.

### Phase 3A — complete the production-scope measurement matrix

Before selecting another grammar lever, extend the zero-model-call harness so
each qualifying large fixture is captured at Low, Medium, High, focused Edit,
and all-body Edit. The corpus must include representative large TypeScript,
Angular, and C#/.NET inputs. Record language, fidelity, intent/focus mode,
source tokens, candidate tokens, selected representation, and semantic-family
token anatomy. Java/Spring remains secondary.

Compute the production-selected aggregate independently for every tokenizer:

```text
selected(i) = candidate(i) only when the local estimate plus safety buffer
              beats raw(i); otherwise raw(i)
aggregate reduction = 1 - sum(selected(i)) / sum(raw(i))
```

Small lifecycle/correctness fixtures remain mandatory roundtrip coverage but do
not enter this aggregate. Focused Edit is reported separately because full raw
source would expose unselected bodies and is not an acceptable fallback.
All-body Edit may legitimately select raw and contribute zero savings.

#### Checkpoint 3A

- Every primary language has at least one qualifying large fixture.
- Every fidelity/focus row is visible; no blended-only report is accepted.
- Body-bearing rows verify exact UTF-8 bytes and numeric spans.
- The harness makes zero model calls and does not modify production behavior.
- Current baseline numbers and anatomy are recorded before the next grammar
  experiment.

### Phase 3B — typed wire-local identity design gate

The next structural proposal is deterministic file-local alpha-renaming of
wire identities. This is permitted only as a reversible presentation handle
after canonical typed ownership has already been resolved. It must never turn
display names or row position into canonical authority.

Before implementation, document and obtain explicit maintainer approval for:

- the typed local numbering rule and first-appearance scope;
- which declarations retain explicit handles and which may derive them;
- how every owner/caller/parameter/field/pattern reference is rewritten;
- alpha-normalized equality between the canonical oracle and decoded A3;
- the rule that local handles are payload-local, never persisted, never
  accepted as selectors, and never returned as canonical workspace identity;
- deterministic rejection of duplicate, missing, wrong-family, dangling, or
  out-of-range references.

Do not change the compiler's global `next_id`, canonical IR, focus resolution,
persistence, delta/replay authority, or workspace-query identity. Focus still
resolves documented selectors against canonical typed ownership first; only
the already-resolved result is rendered with local handles.

#### Checkpoint 3B

- The identity spelling change is explicitly approved before code changes.
- Alpha-normalized roundtrip equality covers every identity-bearing family.
- Same-name owners, overloads, calls, DI, and focused Edit remain unambiguous.
- The token delta is reported against the Phase 3A baseline.
- Any reasoning-facing identity regression rejects the design regardless of
  savings.

### Phase 3C — scoped type/callee table experiment

Only after Phase 3B measurement, test an optional table limited to repeated
field/parameter/return types and callee-written names. This is not another
generic dictionary. Keep an explicit on/off form and measure the complete cold
payload, including table rows, references, and legend changes.

Selection must use actual cl100k/o200k counts in research and the existing
calibrated local estimate plus buffer in production; character or byte savings
are not evidence. Keep the table only if it improves the production-selected
aggregate without breaking any deterministic or reasoning invariant. Below the
measured break-even threshold, emit no table.

#### Checkpoint 3C

- Referenced types and callees decode byte-for-byte to their original strings.
- Empty-table fallback and invalid-index rejection have tracked tests.
- The experiment reports complete-payload deltas under both tokenizers.
- A negative result is documented and removed rather than forced to remain.

**Result (negative, removed):** the table increased the complete payload on every
measured row (per-row deltas +54 to +828 tokens across both tokenizers) because
the repeated types/callees are already single BPE tokens, so `@n` references save
nothing while the table rows and extended legend add cost. The table was removed
and the tree restored to the Phase 3B checkpoint; the finding is recorded in
`A3_PHASE3C_TABLE.md`.

### Phase 3D — corpus-backed defaults and final row merging

Measure occurrence/value frequencies for `mo`, `cs`, `lf`, and `pf`. Introduce
an implied default only where the primary corpus demonstrates a dominant value
and the grammar can reconstruct it exactly. Empty groups, duplicate groups,
group boundaries, and order remain explicit and significant.

After identity and table shapes settle, audit always-adjacent records for final
merging. Existing packed calls, fields, and parameters are the starting point;
do not recreate completed work. Every merge needs an isolated before/after
token delta and exact roundtrip evidence.

#### Checkpoint 3D

- Every default is justified by recorded primary-corpus frequency data.
- No semantic value, empty group, duplicate, or boundary is discarded.
- Only measured token wins remain in the grammar.
- All modified/new files remain within the active-file ceiling.

### Phase 3E — economics decision

Rerun the complete Phase 3A matrix. The checkpoint passes when the sum of
production-selected outputs is at least 50% smaller than the same qualifying
raw corpus under the approved estimators/tokenizers. There is no maximum:
continue improving when another lossless, reasoning-safe measured win remains.
All diagnostic rows remain published, and the local raw gate must prevent every
ordinary invocation from inflating model cost.

## Phase 4 — locality and reasoning screen

Run only the smallest bounded cases justified by Phase 3:

- typed ownership and overload identity;
- ordered/duplicate local calls and spread evidence;
- DI occurrence grouping;
- behavior-family lookup;
- focused Edit target and exact-source request behavior.

Workspace dependency/provenance questions use `workspace_query` payloads and
are evaluated separately from A3 file context.

### Checkpoint 4

- No identity fabrication, wrong owner, occurrence loss, or edit-target error.
- Sparse/scoped organization is non-inferior to the correctness control for
  every critical family.
- Navigation is added only for a demonstrated locality failure and is then
  remeasured against raw.

## Phase 5 — production lifecycle integration

Only after Checkpoints 1–4 pass, integrate A3 through provide, compress,
delta/apply, durable restore, and replay. Preserve the existing local economics
gate and raw fallback. Persist canonical checked state, never trusted rendered
A3 text as semantic authority.

### Checkpoint 5

- One encoder and version rule serves every lifecycle path.
- Structured/auxiliary authority remains unchanged.
- Restore and replay regenerate A3 from durable checked state.
- Focused Edit never falls back to full raw source.
- Tracked lifecycle tests and the user-owned final gate are green.

## Phase 6 — optional stable-legend reuse

After cold A3 passes, evaluate a separately versioned acknowledged-schema mode.
Omit the inline legend only when the client explicitly acknowledges the exact
schema version and digest in a prompt region with valid cache semantics. Missing
or stale acknowledgement falls back to cold A3.

Caching savings are reported separately and never used to make a cold losing
format appear economical.

## Stop conditions

Stop and do not spend model tokens if:

- deterministic equality fails;
- any body or span changes;
- a fidelity target is ambiguous;
- cold A3 loses to raw on all representative large fixtures;
- the raw gate selects an inflated candidate;
- scoped notation causes an identity/locality regression.

## Immediate next work

Phase 3B (identity renumbering, 0.00% delta) and Phase 3C (scoped type/callee
table, negative, removed) are complete; see `A3_PHASE3C_TABLE.md` for the Phase 3C
finding. The next lever is Phase 3D — corpus-backed defaults and final row
merging: measure occurrence/value frequencies for `mo`, `cs`, `lf`, and `pf`,
introduce an implied default only where the primary corpus demonstrates a dominant
value and the grammar can reconstruct it exactly, then audit always-adjacent
records for final merging. Each default and merge needs an isolated before/after
token delta and exact roundtrip evidence before it stays in the grammar.
