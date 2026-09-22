# COMPACT-A3 sparse positional phase plan

**Status:** Phase 1 complete; Phase 2 green; Phase 3 cold economics implemented, verification pending
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
- At least one representative large fixture is economically eligible before
  reasoning evaluation begins.
- No aggregate hides an individual inflation case.
- C# economics remain explicitly pending until a qualifying fixture exists.

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

Begin Phase 1 with a research-only A3 encoder/decoder over normalized fixtures.
Implement typed owner/member/signature rows first, followed by sparse behavior,
local calls, imports/types, and exact body frames. Do not modify production
selection until the research encoder/decoder clears deterministic roundtrip and
economics.
