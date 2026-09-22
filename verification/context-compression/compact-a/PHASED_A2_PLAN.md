# COMPACT-A2 phased implementation plan

**Status:** approved direction; implementation checkpoints not yet passed  
**Scope:** lightweight navigation references plus explicitly acknowledged
schema reuse. Canonical IR, normalized CONTROL-FULL, exact Edit bodies, and
raw-source economics remain authoritative.

## Non-negotiable invariants

1. Decoding produces the same normalized CONTROL-FULL semantic object.
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
- CONTROL-FULL is regenerated from structured IR plus semantic edges;
- measurement reports raw→candidate as the production comparison;
- paired reasoning uses equivalent questions that do not require IDs absent
  from raw source.

### Checkpoint 0

- Focused Edit regression is green and never reports `raw_passthrough`.
- Full repository test and zero-warning gates are green.
- The corrected capture, measurement, and decoder scripts complete with zero
  model calls.
- A1 raw-source results are recorded as the baseline, even if they are poor.

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
Record per capture and tokenizer:

- raw-source tokens;
- A1 cold tokens;
- A2 cold tokens;
- A2 minus raw tokens and percentage;
- navigation tokens by `D`, `V`, and `E`;
- legend tokens separately;
- selected production representation.

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
