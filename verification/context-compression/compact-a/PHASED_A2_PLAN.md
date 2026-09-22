# COMPACT-A2 phased implementation plan

**Status:** stopped after deterministic economics; rejected as a production encoding
**Scope:** lightweight navigation references plus explicitly acknowledged
schema reuse. Canonical IR, normalized CONTROL-FULL, exact Edit bodies, and
raw-source economics remain authoritative.

## Final A2 finding

A2 preserved the intended file/workspace authority split and exact semantic
oracle, but its wire encoding failed the raw-source economics gate. The failure
is representational, not evidence that correctness requires inflation:

- the complete inline legend costs 146 cl100k / 147 o200k tokens;
- nested parameters, fields, imports, aliases, patterns, and flow facts remain
  named JSON objects rather than positional rows;
- fixed-width method rows serialize repeated empty arrays and nulls;
- call rows repeat occurrence, caller, spread, and unresolved defaults;
- fidelity is recorded in metadata but does not select a smaller family layout;
- navigation remains measurable overhead even where scoped layout already
  supplies locality.

On the two qualifying tracked economics fixtures, A2 was 19–55% larger than
raw under cl100k and 18–52% larger under o200k. It is therefore ineligible for
model smoke or production selection. A2 remains a frozen diagnostic artifact;
its row layout must not be silently changed. The approved successor work is
defined in `PHASED_A3_PLAN.md`.

## Non-negotiable invariants

1. Decoding produces the same normalized CONTROL-FULL file-context object.
   Workspace semantic edges are verified separately against the authoritative
   workspace-query/index snapshot, not folded into that object.
2. Canonical typed IDs remain identity; display names and serialized positions
   never become identity.
3. Occurrence order, duplicates, nested group boundaries, endpoint-local
   provenance, unresolved calls, body bytes, and spans remain exact.
4. Full raw source is an admissible fallback only when it is semantically
   equivalent. It is not admissible for focused Edit.
5. Cold payload economics are measured against capture-time byte-exact raw
   source. CONTROL-FULL is only the correctness oracle.
6. The inline legend may be omitted only after an explicit schema-version
   acknowledgement. Session history, a previous response, or cache metadata
   alone never proves that the model has the legend.
7. No model calls occur until deterministic roundtrip and raw-source economics
   pass.
8. File-context content is file-local and intent/fidelity scoped. Workspace
   graph edges are indexed, persisted, and returned by `workspace_query`; they
   are not copied into provide/compress/apply/restore/replay model content.
   Canonical file-local calls and injection occurrences remain in the file
   envelope.

## Schema naming

Any wire-shape change receives a new version. The lightweight-navigation and
schema-reference candidate is `COMPACT-A2`; existing `COMPACT-A1` remains a
frozen comparison control. A future authoritative-table redesign, if needed,
must receive another version rather than silently changing A2.

## Phase 0 — stabilize the corrected foundation

Complete the already-started correctness repairs before changing the codec:

- focused Edit always returns selected exact bodies plus other-method
  skeletons;
- unfocused modes retain the raw-source ceiling;
- every capture stores `raw-source.txt` at capture time;
- the CONTROL-FULL file oracle is regenerated from structured IR, while the
  semantic-edge snapshot is retained and verified separately;
- measurement reports raw→candidate as the production comparison;
- paired reasoning uses equivalent questions that do not require IDs absent
  from raw source.
- remove workspace semantic-edge snapshots and edge-navigation copies from
  model-visible file-context and delta content while retaining the same edge
  state for indexing, persistence, auxiliary results, and `workspace_query`;
- prove that file content retains canonical local calls/injections while
  `workspace_query` still returns forward/reverse/framework graph facts.

### Checkpoint 0

- Focused Edit regression is green and never reports `raw_passthrough`.
- Full repository test and zero-warning gates are green.
- The corrected capture, measurement, and decoder scripts complete with zero
  model calls.
- A1 raw-source results are recorded as the baseline, even if they are poor.
- The file-context/workspace-query authority split is green across provide,
  compress, delta/apply, restore, and replay.

Stop if this checkpoint is red. Do not mix navigation or caching changes into
foundation debugging.

## Phase 1 — define lightweight navigation descriptors

Keep `d` and `g` authoritative. Replace duplicated navigation values with
stable typed references.

### Behavior index

Use canonical method ID plus a versioned family tag:

```text
n.V = [[M16,"cf"],[M16,"se"],[M16,"ec"],[M18,"df"]]
```

The descriptor means: resolve the canonical method in typed declarations and
read the field defined by the tag. It never contains a serialized array index.

### DI index

Use canonical class ID plus an explicit injection-state tag:

```text
n.D = [[C6,"inj"],[C22,"inj0"]]
```

`inj` points to a non-empty authoritative `injection_occurrences` field.
`inj0` highlights an authoritative empty field where the model must not infer
core occurrences from constructor parameters. It adds no semantic assertion.

### Edge index

Keep `g.E` as the sole edge value table. Navigation may contain only stable
typed locators and endpoint-field tags. A bare serialized row number is
forbidden. If the existing semantic occurrence is proven unique within one
edge collection, a descriptor may use it with its collection and relation:

```text
n.E = [["E",occurrence,relation,"S.f","O.f","L"]]
```

If that tuple is not unique, use the existing typed subject/object locator
shape. Do not re-embed names, files, layer values, or endpoints.

### Checkpoint 1

- A2 decoder resolves every descriptor to exactly one authoritative field.
- Removing `n` from decoded output and regenerating normalized navigation
  yields equality with CONTROL-FULL.
- Tests reject unknown tags, wrong entity kinds, missing IDs, ambiguous edge
  locators, and any serialized-array-index descriptor.
- A1 remains decodable as a frozen prior version.

## Phase 2 — deterministic economics and locality screen

Render A2 with its complete inline legend and run the corrected capture matrix.
Keep the semantic/lifecycle fixtures as the correctness lane. Use the tracked
`src/test_files/angular`, `src/test_files/dotnet`, and
`src/test_files/typescript` trees as a separate production-economics lane;
preserve their relative layout so companion files and meta-layer relationships
remain discoverable. Measure supported TypeScript and C# inputs; retain Angular
HTML as companion context but classify it as raw-only because it has no
structured IR/codec candidate. Require at least 8 KiB per file for aggregate
compression economics. The tracked `LargeService.ts` and
`UserManagementService.ts` currently qualify; no tracked C# fixture does, so C#
economic conclusions remain pending a genuinely large fixture.
Record per capture and tokenizer:

- raw-source tokens;
- A1 cold tokens;
- A2 cold tokens;
- A2 minus raw tokens and percentage;
- navigation tokens by `D`, `V`, and `E`;
- legend tokens separately;
- selected production representation.

Workspace graph edges are excluded from file-envelope token totals. Measure
their cost separately against versioned `workspace_query` content only when a
request actually asks for graph information.

Do not aggregate away fidelity, language, focus mode, or lifecycle path.

### Checkpoint 2A — correctness

- Exact normalized roundtrip passes every deterministic case.
- Focused/all-body Edit frames remain byte-exact.
- Restore, replay, delta apply, TypeScript/Angular, and C# edge cases pass.

### Checkpoint 2B — economics

- A2 cold is compared to raw, never merely to CONTROL-FULL.
- Every production selection obeys the raw ceiling except approved focused
  Edit, where full raw is not equivalent.
- Results identify wins and losses individually as well as request-weighted
  totals.

### Checkpoint 2C — locality

Only after 2A and 2B, run the smallest bounded smoke covering:

- occurrence-family lookup through `n.V`;
- DI honesty through `n.D`;
- independent subject/object provenance through `n.E`→`g.E`.

Reject A2 if lightweight references recreate the prior locality failures.

## Phase 3 — move the legend to a genuinely stable prompt region

Do not try to cache an isolated tool-result block: Anthropic caching applies to
the full prompt prefix through a breakpoint. A split result block alone does
not make the legend independently reusable across changing conversations.

Instead:

1. Publish the exact A2 legend and a stable schema digest through the existing
   Clean-CTX vocabulary/system-prompt resource.
2. Attach the `system_prompt` cache hint to that stable prompt resource.
3. Add an explicit request capability such as
   `acceptedContextSchemas:["clean-ctx/compact-a/2:<digest>"]`.
4. Default to the cold, self-contained A2 payload with inline legend.
5. When and only when the exact digest is acknowledged, emit a short schema
   reference and omit the inline legend.
6. Fail closed to cold A2 for absent, stale, malformed, or unknown
   acknowledgements.

The acknowledgement is request-scoped and portable. Server memory that a
legend was emitted previously is insufficient because it does not prove that
the current model context contains it.

### Checkpoint 3

- Cold A2 is independently understandable and decodable.
- Warm A2 requires the exact version+digest acknowledgement.
- Wrong/stale acknowledgements deterministically fall back to cold A2.
- Tool schema, prompt resource, cache hint, documentation, restore/replay, and
  delta lifecycle all use one version rule.
- Cold and acknowledged token economics are reported separately.

## Phase 4 — production integration gate

Integrate A2 through:

- provide full and delta-baseline paths;
- direct compress;
- apply-delta post-state;
- durable restore and replay;
- Low, Medium, High, all-body Edit, and focused Edit;
- raw fallback and missing-source durable restoration.

Keep A1 available only as a decoder/research compatibility control unless a
maintainer explicitly authorizes runtime negotiation with older clients.

### Checkpoint 4

- Complete repository verification and zero-warning gates pass.
- Registered-path captures prove the selected representation and schema mode.
- No response claims cached-schema mode without exact acknowledgement.
- No lifecycle path silently returns duplicated A1 navigation.

## Phase 5 — field validation

Use Claude Enterprise on real Angular/TypeScript and C# repositories. New Relic
is authoritative for actual Claude input/cache metrics.

Validate separately:

- cold A2 request;
- acknowledged-schema A2 request;
- raw fallback request;
- focused Edit;
- repeated calls, DI, overload ownership, and endpoint provenance;
- delta and restored-session behavior.

### Checkpoint 5

- No correctness-critical reasoning or edit failure.
- Observed representation matches the local gate decision.
- New Relic distinguishes normal input, cache creation, and cache-read tokens.
- Any estimator miss or client incompatibility is recorded in the discovery
  registry and distilled into a tracked regression when reproducible.

## Decision outcomes

- **A2 cold beats raw and reasoning passes:** enable normal production
  selection.
- **Only acknowledged A2 beats raw:** keep cold requests on raw; enable A2 only
  for explicitly schema-aware clients.
- **A2 loses locality:** retain A1/raw production behavior and investigate an
  authoritative-table successor under a new schema version.
- **A2 is still economically weak:** use the measured family breakdown to
  decide whether authoritative `D/V/E` tables justify a separate candidate.
- **No lossless candidate beats raw:** raw remains the correct production
  result; do not recover apparent savings through semantic omission.
