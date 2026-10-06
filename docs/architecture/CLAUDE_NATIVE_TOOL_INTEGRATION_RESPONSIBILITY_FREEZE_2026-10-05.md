# Claude-Native Capability Architecture Investigation

**Date:** 2026-10-05

**Status:** Claude-native architecture frozen. The successful-result security
scope was approved, and the first implementation slice is specified in
`CLAUDE_NATIVE_FIRST_VERTICAL_SLICE_FREEZE_2026-10-05.md`.

**Scope:** Architecture investigation and design freeze only. No production
code was restructured and no Cargo command was run.

## 1. Executive Verdict

**CLAUDE-NATIVE ARCHITECTURE READY — RESPONSIBILITY BOUNDARY FROZEN**

The best Claude-native architecture is substantially frozen:

- Claude owns tool lifecycle, native result schemas, caching, context
  management, permissions, discovery, and model selection.
- Clean-CTX adds three narrow, transport-independent text capabilities:
  diagnostic filtering, recognized-secret redaction, and ANSI/control-sequence
  normalization.
- A Claude adapter selects safe textual fields from successful native results,
  calls those narrow capabilities, reconstructs the original native shape, and
  returns `PostToolUse.updatedToolOutput`.
- Unknown, opaque, binary, image, unsupported, and invalid native shapes pass
  through unchanged.
- Failed results remain native and unchanged for filtering, normalization, and
  context management. Proxy parity supplies no independent reason to mutate
  them.
- Proxy cache injection, sliding-window aging, tool dropping, model override,
  and Bash-description trimming do not migrate into Claude-native capability
  architecture.
- The proxy remains an optional compatibility/specialized gateway and may
  legitimately retain different behavior for HTTP-only clients.

The approved security guarantee is deliberately narrow: for explicitly
supported successful tool-result schemas, Clean-CTX may replace recognized
secrets in selected model-visible text fields before delivery to the model.
This is not a guarantee covering execution, hook infrastructure, telemetry,
failed results, transcripts/history, unsupported shapes, or every
provider-bound surface.

## 2. Architectural Correction

The existing HTTP proxy is evidence, not the behavioral specification for
Claude-native integration.

It proves that users value diagnostic compression, secret recognition, and
terminal-noise removal. It also exposes useful filter rules, scrub patterns,
tests, and historical compatibility requirements. It does **not** prove that a
native integration should:

- rescan accumulated provider request envelopes;
- infer tool identity from output text;
- infer failure from words such as `exit code` or `error:`;
- flatten content-block arrays;
- reduce failed diagnostics;
- age messages by position;
- inject provider cache breakpoints;
- remove tool definitions; or
- rewrite host-owned tool descriptions.

Native/proxy differences matter only when an independently justified Clean-CTX
capability contract requires the behavior. No such requirement exists for
failed diagnostic filtering, failed ANSI cleanup, proxy aging, cache injection,
tool dropping, or Bash-description trimming. Those differences are therefore
not native-architecture blockers.

The design test used throughout this report is:

> If the proxy had never existed, given Claude's current authoritative facts
> and lifecycle boundaries, what is the smallest correct Clean-CTX feature?

## 3. Capability-by-Capability Native Decisions

### 3.1 Tool-output filtering

- **Problem:** Successful commands can produce verbose, repetitive diagnostics
  that consume context without improving the next model decision.
- **Still present natively:** Yes.
- **Claude facts:** Exact tool name, tool input, successful lifecycle event,
  tool-specific structured response, and replacement authority.
- **Native mechanism:** Synchronous `PostToolUse` and
  `updatedToolOutput`.
- **Unique Clean-CTX value:** The program-specific filter catalog, deterministic
  diagnostic reductions, JSON-safety behavior, and reduction facts.
- **Needed information:** Tool identity; optional command from tool input;
  explicit success; selected diagnostic text fields. Output-text guessing is
  unnecessary when authoritative identity/input exists.
- **Boundary and mutation:** On successful `PostToolUse`, replace only fields
  the Claude adapter knows how to reconstruct and validate.
- **Shared logic:** Filter rule compilation, selection, and text reduction.
- **Do not migrate:** Whole-request traversal, first-line identity inference,
  text-based failure inference, array flattening, or automatic failed-output
  reduction.
- **Unknown shapes:** Unchanged pass-through.
- **Classification:** **CLEAN-CTX NATIVE CAPABILITY**.

### 3.2 Secret detection/redaction

- **Problem:** Recognizable credentials in tool output can become model-visible
  and later provider-bound conversation context.
- **Still present natively:** Yes, but the available hook is not a pre-execution
  or pre-infrastructure security boundary.
- **Claude facts:** Successful tool identity/input/structured response and
  model-visible replacement authority after execution.
- **Native mechanism:** `PostToolUse.updatedToolOutput` for successful output.
- **Unique Clean-CTX value:** Deterministic secret patterns, redaction policy,
  and redaction facts.
- **Needed information:** Explicit text segment plus redaction policy. Tool
  identity is optional for the detector.
- **Boundary and mutation:** Redact recognized strings in safe textual fields
  and reconstruct the same successful native result shape.
- **Shared logic:** Pure text detection/redaction only.
- **Do not migrate:** Claims of fail-closed request coverage, whole-transcript
  protection, or telemetry secrecy that the lifecycle cannot enforce.
- **Unknown shapes:** Preserve unchanged; never flatten or stringify opaque
  data to hunt for secrets without an approved schema contract.
- **Classification:** **CLEAN-CTX NATIVE CAPABILITY**, limited to the approved
  successful-result security scope.

### 3.3 ANSI/control-sequence normalization

- **Problem:** Terminal presentation escapes add noise and can interfere with
  deterministic line filters.
- **Still present natively:** Yes for terminal-producing successful tools.
- **Claude facts:** Tool identity and tool-specific structured output.
- **Native mechanism:** `PostToolUse.updatedToolOutput` supplies the mutation
  boundary but not the normalizer.
- **Unique Clean-CTX value:** Deterministic control-sequence removal.
- **Needed information:** One explicit text segment.
- **Boundary and mutation:** Normalize known terminal text fields before
  diagnostic filtering; reconstruct unchanged surrounding structure.
- **Shared logic:** `Text -> Text + normalization facts`.
- **Do not migrate:** Provider-envelope traversal or the assumption that every
  string is terminal text.
- **Unknown shapes:** Unchanged pass-through.
- **Classification:** **CLEAN-CTX NATIVE CAPABILITY**.

### 3.4 Prompt caching

- **Problem:** Repeated stable prefixes increase latency and token cost.
- **Still present natively:** Yes, and native/server request construction owns
  it more authoritatively.
- **Claude mechanism:** Claude/Anthropic cache lifecycle and automatic caching.
- **Unique Clean-CTX value:** None established for native Claude.
- **Needed information/mutation:** None.
- **Do not migrate:** Four-slot breakpoint injection, beta-header injection,
  cache-state bridging, or proxy cache statistics as native policy.
- **Classification:** **USE NATIVE**.

### 3.5 Context reduction/aging

- **Problem:** Long tool-heavy sessions can exhaust context.
- **Still present natively:** Yes.
- **Claude mechanism:** Native compaction lifecycle and Anthropic context
  editing/tool-result clearing.
- **Unique Clean-CTX value:** No measured value beyond native behavior is
  established.
- **Needed information/mutation:** None for the initial native architecture.
- **Do not migrate:** Message-index aging, path-string relevance heuristics,
  replacement stubs, or proxy preservation floors.
- **Classification:** **USE NATIVE**.

### 3.6 Tool availability/filtering

- **Problem:** Control execution authority, model-visible catalog size, and
  discovery cost.
- **Still present natively:** Yes.
- **Claude mechanism:** Permissions for execution; native/API tool discovery
  and deferred loading for visibility and cost.
- **Unique Clean-CTX value:** None established. These mechanisms are related
  but not identical, so configuration must use the mechanism matching the
  user's goal rather than proxy removal as a universal substitute.
- **Needed information/mutation:** None for Clean-CTX core.
- **Do not migrate:** `DROP_TOOLS` request mutation or dependency-free guesses
  that a tool is unnecessary.
- **Classification:** **USE NATIVE**.

### 3.7 Model selection/override

- **Problem:** Select the desired model/provider route.
- **Still present natively:** Yes.
- **Claude mechanism:** Native model configuration and CLI/settings surfaces.
- **Unique Clean-CTX value:** None for Claude-native operation.
- **Needed information/mutation:** None.
- **Do not migrate:** Rewriting top-level model names or model references inside
  system text.
- **Classification:** **USE NATIVE**.

### 3.8 Successful tool-result handling

- **Problem:** Apply Clean-CTX intelligence without corrupting native output.
- **Claude facts:** Explicit success event, tool name, tool input, structured
  response, and schema-checked replacement for built-ins.
- **Native mechanism:** `PostToolUse.updatedToolOutput`.
- **Unique Clean-CTX value:** Selective normalize/redact/filter operations.
- **Needed information:** Tool-specific field map maintained by the Claude
  adapter, not by shared capabilities.
- **Mutation:** Replace only a supported result reconstructed in its original
  shape; otherwise return no replacement.
- **Classification:** **AUGMENT NATIVE**.

### 3.9 Failed tool-result handling

- **Problem:** Preserve diagnostic evidence and optionally observe failures.
- **Claude facts:** Explicit `PostToolUseFailure`, tool identity/input, error
  string, interruption status, and duration.
- **Native mechanism:** Native failure result plus optional
  `additionalContext`; no documented output replacement.
- **Unique Clean-CTX value:** Optional aggregate/filter-performance telemetry
  or future non-mutating hints, neither required initially.
- **Needed information/mutation:** No mutation for filtering, normalization,
  context reduction, or secret handling.
- **Do not migrate:** Success-oriented reductions, textual failure inference,
  or proxy truncation.
- **Classification:** **USE NATIVE**. Failed-result redaction is outside the
  approved native security scope.

### 3.10 Structured tool-result handling

- **Problem:** Mutate useful text without destroying tool-specific semantics.
- **Claude facts:** The native output object; built-in replacement validation;
  MCP output with different validation behavior.
- **Native mechanism:** Tool-specific `updatedToolOutput` reconstruction.
- **Unique Clean-CTX value:** Narrow text transformations only.
- **Needed information:** Adapter-known safe text fields and original shape.
- **Mutation:** Field-local replacement followed by adapter validation.
- **Do not migrate:** Flattening, a provider-neutral result AST, or core
  knowledge of Claude schemas.
- **Classification:** **AUGMENT NATIVE**.

### 3.11 Capability observability/statistics

- **Problem:** Demonstrate token reduction, redaction/normalization activity,
  latency, fallbacks, and unsupported shapes without retaining sensitive raw
  content.
- **Claude facts:** Tool identity, lifecycle event, duration, and hook outcome.
- **Native mechanism:** Hook execution plus Claude-native telemetry where
  appropriate.
- **Unique Clean-CTX value:** Capability-specific counts and reduction facts.
- **Needed information:** Counts, byte/token estimates, filter ID, reason for
  pass-through, and adapter validation result. Raw secret-bearing payloads are
  unnecessary.
- **Mutation:** None to tool output beyond the capability result itself.
- **Do not migrate:** Proxy HTTP/cache metrics or text-only provenance as the
  sole machine-readable record.
- **Classification:** **AUGMENT NATIVE**.

## 4. Native Information Advantages

Claude-native integration deletes inference machinery rather than relocating
it:

| Proxy heuristic | Native authoritative fact | Simplification |
|---|---|---|
| Guess program from first output line/full output | `tool_name` plus structured `tool_input` | Select filter by actual tool and command; keep output matching only as an explicitly configured fallback |
| Treat `tool_use_id` as a tool name | Hook supplies actual tool name and use ID separately | Delete identity conflation |
| Infer failure from `exit code`, `Exit code`, or `error:` | Separate `PostToolUse` and `PostToolUseFailure` events | Delete text-based outcome inference |
| Rescan every `messages[].content[]` block | Hook supplies the one completed result | Transform one event/result |
| Flatten content arrays to obtain diagnostic text | Adapter sees native structured response | Select and replace known text fields only |
| Approximate turns by message index | Native compaction/context lifecycle | Delete proxy aging logic from native mode |
| Approximate relevance with path-string cross-references | Native context manager owns retention; model can reread through tools | Delete path preservation heuristic |
| Remove definitions for availability/token cost | Native permissions and tool discovery/deferred loading | Delete native request mutation |
| Inject four cache breakpoints | Native/server cache lifecycle | Delete native cache policy |
| Rewrite model field/system prose | Native model selection | Delete native model override |
| Identify terminal output by request location | Tool identity plus adapter schema | Normalize only known terminal fields |

## 5. Success Lifecycle Architecture

```text
successful tool execution
  -> PostToolUse(tool_name, tool_input, tool_response)
  -> Claude adapter matches a supported tool/result schema
  -> adapter extracts explicit transformable text fields
  -> ANSI normalizer (when field is terminal text)
  -> secret redactor (within approved successful-output policy)
  -> diagnostic filter (when tool/command has a filter)
  -> adapter replaces only changed fields
  -> adapter validates/reconstructs the original output shape
  -> updatedToolOutput
  -> Claude model-visible context
```

Rules:

- The hook must be synchronous when returning a replacement.
- Capability order is normalize -> redact -> filter. Redaction precedes
  reduction so a filtered-away secret is still counted/redacted in the
  transformed representation.
- If extraction, transformation, reconstruction, or validation is uncertain,
  return no replacement and preserve the native result.
- Tool effects have already occurred; this architecture changes only what the
  model sees.
- The first implementation should support an explicit small schema set, not
  claim all Claude tools.

## 6. Failure Lifecycle Architecture

`PostToolUseFailure` is an authoritative observation boundary, not a mutation
boundary. The native design uses that fact rather than trying to recreate the
proxy.

- **Filtering:** Preserve full failure diagnostics. Success-collapse and
  truncation policies are not justified for failures. Optional future
  additional context may identify a known failure class without replacing
  evidence.
- **ANSI normalization:** Preserve native failure output. Readability benefit
  does not outweigh inventing a second unsupported mutation path.
- **Context reduction:** Native context management owns later retention.
- **Aging:** Not applicable in the event adapter.
- **Secret handling:** A failed result can contain a secret, but the documented
  hook cannot replace it. This matters only if the product promises failed-
  result model redaction. It does not block filtering or normalization.

Consequently, the lifecycle asymmetry is not a general blocker. It is a single
security-scope decision.

## 7. Structured Result Architecture

Native structure belongs to the Claude adapter. Shared capabilities receive
only semantic text they need.

### Result categories

| Native result | Adapter behavior |
|---|---|
| Supported built-in structured object | Select documented safe text fields; transform; rebuild the same object; rely on/perform schema validation |
| MCP textual result | Select text content while preserving MCP content shape and metadata; reconstruct without flattening |
| Multi-part result | Transform supported text parts in place and preserve order, multiplicity, and all opaque parts |
| Plain text result | Transform the string when the tool contract permits string replacement |
| Structured object without an approved field map | Pass through unchanged |
| Image, binary, or opaque content | Pass through unchanged |
| Error/failure result | Follow the native failure policy; no replacement |
| Unknown future shape | Pass through unchanged and record a non-sensitive unsupported-shape fact |

No single canonical `ToolResult` is justified. The capabilities need strings,
not a provider-neutral result tree. The adapter can use small tool-specific
field selectors/rebuilders while keeping the native payload local to the
adapter.

A lightweight internal list of field references may be useful inside the
Claude adapter to avoid duplicating orchestration, but it is not a shared core
wire type and must not erase native type information.

## 8. Filtering Architecture

The filter catalog is best understood as **diagnostic compression**, not a
generic output transformation engine.

Its semantic contract is:

```text
DiagnosticFilterInput {
  tool identity,
  optional command/program hint from native tool input,
  outcome = success,
  diagnostic text
}
  -> ReducedDiagnostic {
       text,
       filter identity,
       original/reduced size facts,
       reduction kind,
       truncated/collapsed flags
     }
```

Design decisions:

- Select primarily from authoritative tool identity and input. Existing output
  regexes may remain only where a single shell tool runs many programs or where
  the command itself is unavailable.
- Initial filtering is success-only. Failures remain untouched.
- Filter rules that collapse known success noise remain valuable.
- Rules that cap, head/tail, or group diagnostics require per-filter review
  against native field structure before activation.
- The `§FILTERED` marker remains useful for model-visible disclosure, but it
  should be rendered by the adapter from structured `FilterFacts`, not be the
  only provenance record.
- JSON guards remain relevant only for text fields containing serialized JSON.
  They are unnecessary for native structured objects that the adapter preserves
  structurally.
- Rules created solely to compensate for flattened mixed stdout/stderr or
  concatenated content blocks must be revised or disabled for native mode.
- Statistics should retain filter ID, sizes, and decision flags; never raw
  unredacted diagnostics.

## 9. Secret Handling and Truthful Security Boundary

The strongest currently supportable native guarantee is:

> For explicitly supported successful tool-result schemas, Clean-CTX can
> replace recognized secrets in selected model-visible text fields before that
> result is delivered to the model.

It cannot truthfully guarantee that Claude infrastructure never observes the
secret.

| Surface | Intercept before exposure? | Can mutate? | Desired policy |
|---|---:|---:|---|
| Tool execution/effects | No | No | Use permissions/sandboxing/pre-execution controls; redaction is irrelevant to side effects |
| Successful model-visible result | Yes, before model delivery, but after hook infrastructure receives it | Yes | Redact recognized secrets in supported text fields; pass through unsupported shapes |
| Failed model-visible result | Observe only through documented failure hook | No documented replacement | Do not claim coverage; decide whether this limitation is acceptable before naming the feature broadly |
| Hook input | No; original result is the hook input | No | Treat as sensitive in memory; do not log raw input |
| Claude telemetry | No; documentation says original output is captured before rewrite | No | Explicitly outside Clean-CTX guarantee |
| Transcript/history | Not established by current evidence | Not safely claimable | Mark unproven; do not claim at-rest transcript redaction |
| Clean-CTX logs/metrics | Yes, because Clean-CTX owns them | Yes/omit | Never log raw result by default; record only non-sensitive facts |
| Later provider request/model context | Successful replacement is intended to affect what Claude sees; exact persistence/replay behavior needs an integration test | Successful result indirectly | Claim provider-bound protection only after live contract verification |
| Optional proxy provider-bound history | Yes at the outbound HTTP request boundary for covered request shapes | Yes | Legitimate stronger gateway boundary, subject to provider-shape coverage and its own logging policy |

The current proxy may remain useful for deployments requiring provider-bound
request scrubbing across accumulated history. That is a gateway security mode,
not evidence that a Claude hook offers the same boundary.

The chosen product scope is narrowly named successful-result model redaction.
Broader secrecy guarantees remain explicitly outside Claude-native capability.

## 10. ANSI/Normalization Architecture

The shared primitive is intentionally small:

```text
normalize_terminal_text(text, policy)
  -> { text, sequences_removed }
```

The Claude adapter decides whether a field is terminal text based on tool
identity and schema. Normalization runs before diagnostic filtering so escape
codes do not break line rules. It does not traverse arbitrary objects, inspect
provider envelopes, or mutate failures/unknown shapes.

The existing global proxy regex and filter-local regex should be reconciled
during a later extraction so there is one semantic implementation. That is an
implementation concern, not grounds for a normalization framework.

## 11. Cache/Context/Permissions/Discovery Decisions

| Area | Native authority | Clean-CTX addition | Native decision |
|---|---|---|---|
| Prompt caching | Claude/Anthropic request and server cache lifecycle | None established | **USE NATIVE**; do not port four-slot injection |
| Context management | Claude compaction plus Anthropic context editing/clearing | None measured beyond native behavior | **USE NATIVE**; do not port sliding-window aging |
| Tool execution permission | Claude permission policy | None required | **USE NATIVE**; do not port request-level dropping as permission policy |
| Tool visibility/token cost | Native/API discovery and deferred loading | No proven gap | **USE NATIVE**; configure discovery rather than delete definitions |
| Model selection | Claude configuration | None | **USE NATIVE**; do not rewrite model/system text |

Permissions, visibility, token cost, and discovery are distinct. The adapter
must not claim that one setting solves all four, but Clean-CTX also should not
invent a parallel authority layer without a measured gap.

## 12. Minimum Shared Capability Contracts

No shared result AST is required. Three narrow contracts suffice.

### Terminal normalization

```text
TextSegment + NormalizationPolicy
  -> NormalizedText + NormalizationFacts
```

### Secret redaction

```text
TextSegment + RedactionPolicy
  -> RedactedText + RedactionFacts
```

Redaction facts identify rule classes/counts, not secret values.

### Diagnostic filtering

```text
ToolIdentity
+ optional CommandHint
+ Outcome::Success
+ DiagnosticText
  -> ReducedDiagnosticText + FilterFacts
```

These primitives may share a small text/facts vocabulary if extraction proves
it removes duplication. They do not need shared knowledge of Claude outputs,
Anthropic/OpenAI messages, HTTP, hook JSON, images, or provider metadata.

## 13. Claude-Native Adapter Contract

The Claude adapter owns:

- hook registration/configuration and version/capability checks;
- parsing `PostToolUse` and `PostToolUseFailure` events;
- tool identity/input interpretation;
- an explicit supported tool/schema map;
- safe text-field extraction;
- capability ordering and configuration;
- reconstruction of the exact native output shape;
- schema validation/fallback;
- `updatedToolOutput`, optional non-sensitive context, and adapter telemetry;
- unchanged pass-through for every unsupported or uncertain case.

It must not:

- contain duplicate filter/scrub/normalization algorithms;
- flatten native content for convenience;
- transform a field merely because it is a string;
- replace failed output through an undocumented mechanism;
- log raw hook payloads by default;
- claim support for all built-in/MCP tools from a few schemas;
- control cache, context, permissions, discovery, or model selection.

## 14. Relationship to Optional HTTP Gateway

The architecture is sibling adapters over selected shared primitives:

```text
                 Clean-CTX text capabilities
                /                           \
       Claude-native adapter          HTTP compatibility gateway
       native lifecycle/shapes        transport/provider envelopes
```

Shared implementation primitives:

- filter catalog/compiler/reducer;
- secret pattern engine/redactor;
- ANSI/control-sequence normalizer;
- non-sensitive capability facts where semantics match.

Proxy-only compatibility behavior:

- full request-history traversal for supported provider envelopes;
- sliding-window aging;
- tool-definition dropping;
- proxy auto-start/cache-state bridging;
- gateway body logging and HTTP statistics.

Provider-gateway behavior:

- routing/forwarding, headers, auth, limits, buffering, response projection;
- Anthropic cache injection and cache-usage parsing where retained;
- model override for explicit compatibility deployments.

Obsolete behavior:

- Bash-description trimming.

The two modes need not produce identical output. They share only capability
contracts that independently make sense in both modes.

## 15. Capability Decision Matrix

| Capability | Underlying Problem | Claude Native Mechanism | Clean-CTX Unique Value | Native Strategy | Proxy Strategy | Classification |
|---|---|---|---|---|---|---|
| Diagnostic tool-output filtering | Verbose successful diagnostics consume context | `PostToolUse.updatedToolOutput` | Filter catalog and deterministic reduction | Transform supported successful diagnostic fields | May continue request-history filtering for compatible envelopes | CLEAN-CTX NATIVE CAPABILITY |
| Secret detection/redaction | Recognized secrets become model-visible/provider-bound | Successful output replacement | Secret patterns and redaction facts | Narrow successful-result redaction under the approved scope | Gateway may scrub covered provider-bound history | CLEAN-CTX NATIVE CAPABILITY |
| ANSI/control normalization | Terminal escapes add noise and disrupt filters | Successful output replacement | Normalization primitive | Normalize supported terminal fields | Continue for covered gateway representations | CLEAN-CTX NATIVE CAPABILITY |
| Prompt caching | Repeated prefixes cost tokens/latency | Native/server caching | None proven | Do nothing | Retain only for compatibility clients needing envelope injection | USE NATIVE |
| Context reduction/aging | Long sessions exhaust context | Compaction/context editing | None measured | Do nothing | Compatibility heuristic may remain opt-in | USE NATIVE |
| Tool permissions | Control execution authority | Native permission policy | None | Configure native policy | Gateway auth is separate; do not equate it with tool permission | USE NATIVE |
| Tool discovery/visibility | Large catalogs cost context and impair selection | Native/API deferred discovery | None proven | Configure native discovery | Tool dropping remains HTTP compatibility only | USE NATIVE |
| Model selection | Choose model/provider | Native model configuration | None | Configure Claude | Gateway override remains compatibility routing | USE NATIVE |
| Successful result lifecycle | Safe mutation point and authoritative outcome | `PostToolUse` plus schema-checked replacement | Apply narrow capabilities | Adapter extracts/rebuilds supported schemas | Gateway rewrites supported envelopes | AUGMENT NATIVE |
| Failed result lifecycle | Preserve useful diagnostics | `PostToolUseFailure` and native result | No initial mutating capability | Pass through; optional observe/augment | Gateway may retain historical behavior | USE NATIVE |
| Structured result preservation | Avoid corrupting native output | Native object and replacement validation | Text transformations only | Tool-specific field adapters | Provider-specific envelope adapters | AUGMENT NATIVE |
| Capability observability | Prove value and detect fallback | Hook lifecycle/telemetry | Filter/redaction/normalization facts | Record non-sensitive structured facts | Keep gateway metrics separate | AUGMENT NATIVE |
| Bash-description trim | Historical tool-definition token reduction | Native tool definition | None | Do not implement | Deprecate and retire | DON'T PORT |
| Proxy tool dropping | Historical catalog mutation | Permissions/discovery | None proven | Do not implement | Optional compatibility behavior | DON'T PORT |
| Proxy sliding aging | Historical context heuristic | Native context lifecycle | None proven | Do not implement | Optional compatibility behavior pending its own lifecycle | DON'T PORT |
| Proxy cache injection | Historical request optimization | Native cache lifecycle | None proven | Do not implement | Provider compatibility behavior | DON'T PORT |

## 16. Failure Matrix

| Capability | Current Proxy Failure Behavior | Desired Native Failure Behavior | Native Mutation Required? | Native Surface Available? | Decision |
|---|---|---|---:|---:|---|
| Diagnostic filtering | Detects failure from text; skips success collapse but can still strip/group/cap/truncate | Preserve complete native failure diagnostics | No | Observation/additional context only | UNCHANGED PASS-THROUGH |
| Secret redaction | Redacts recognized text in covered failure/tool-result blocks without consulting authoritative status | Preserve the native failed result; failed-result secrecy is outside the approved scope | No | No documented failure replacement | UNCHANGED PASS-THROUGH |
| ANSI normalization | Removes matching escapes in covered failed result content | Preserve Claude-native failure representation | No | Observation only | UNCHANGED PASS-THROUGH |
| Truncation/reduction | Filter caps and aging may replace historical failed content | Let native context management retain/clear it | No | Native context lifecycle | NOT APPLICABLE |
| Sliding aging | Later message-position heuristic can age a failed result | No native adapter aging | No | Native context lifecycle | NOT APPLICABLE |
| Observability | Proxy counters infer failure indirectly | Count failure events/duration without raw payload retention | No | Yes | OBSERVE/AUGMENT ONLY |

No capability currently yields `NATIVE ACTION REQUIRED`. Failure replacement
is not a general architectural blocker. It becomes one only if the approved
secret guarantee independently requires failed-output redaction.

## 17. Native-Information Replacement Matrix

| Proxy Heuristic | Why It Exists | Native Authoritative Fact | Can Heuristic Be Deleted? |
|---|---|---|---|
| Output-first-line program guessing | Proxy lacks the originating command/tool | `tool_name` and `tool_input` | Yes for known native tools; retain explicit shell-command parsing only where one shell tool launches many programs |
| `tool_use_id` used as name | Anthropic result block lacks colocated name | Hook supplies both identity values | Yes |
| Failure string matching | Accumulated request lacks lifecycle event | Success and failure hooks are separate | Yes |
| Whole-history content traversal | Proxy sees only provider request | Hook receives one completed result | Yes |
| Flatten arrays before filtering | String filter engine is embedded in envelope traversal | Adapter can select native text fields | Yes |
| Message index as age/turn | Proxy lacks lifecycle authority | Native compaction/context editing | Yes |
| Path-string preservation | Proxy guesses future relevance | Native context lifecycle and reread tools | Yes |
| Drop definitions by configured name | Proxy is the only visible interception point | Native permission and discovery state | Yes in native mode |
| Four cache slots | Proxy constructs cache hints externally | Native/server cache lifecycle | Yes in native mode |
| Model/system text rewriting | Client routing was externally constrained | Native model selection | Yes in native mode |

## 18. Frozen Architecture

```text
Claude lifecycle
  |
  +-- PostToolUse(success)
  |      |
  |      v
  |   Claude Adapter
  |   - authoritative tool identity/input
  |   - explicit supported result schemas
  |   - safe text-field selection
  |      |
  |      +--> normalize_terminal_text
  |      +--> redact_recognized_secrets
  |      +--> reduce_success_diagnostics
  |      |
  |   reconstruct + validate original native shape
  |      |
  |   updatedToolOutput or unchanged pass-through
  |
  +-- PostToolUseFailure
  |      |
  |      +--> native diagnostic pass-through
  |      +--> optional non-sensitive observation
  |      +--> secret scope unresolved
  |
  +-- native cache / context / permissions / discovery / model selection
         |
         +--> use directly; no parallel Clean-CTX authority

Shared Clean-CTX layer: three narrow text capabilities and facts.
Optional HTTP gateway: sibling compatibility adapter, not parent architecture.
```

## 19. Genuine Remaining Blockers

No architectural blockers remain. The narrow successful-result secret
guarantee is approved; broader coverage is an explicit non-goal rather than an
implementation requirement.

The following are implementation-scope decisions, not architecture blockers:

- which small set of built-in/MCP schemas ships first;
- whether model-visible filter disclosure uses the existing marker wording;
- telemetry storage/export mechanics;
- performance thresholds for enabling filters.

Proxy output differences, absence of failed-result mutation for filtering, and
lack of OpenAI/generic parity do not block the Claude-native design.

## 20. Future Implementation Sequence

This sequence is superseded for the first implementation slice by
`CLAUDE_NATIVE_FIRST_VERTICAL_SLICE_FREEZE_2026-10-05.md`. The proxy remains
outside that slice and unchanged.

1. Add tracked tests for pure normalization, redaction, and successful
   diagnostic filtering contracts without moving production callers.
2. Extract the three narrow primitives while keeping the proxy as the only
   production consumer; prove existing intended text behavior is preserved.
3. Define one supported Claude success schema, preferably a terminal-producing
   tool whose command and stdout/stderr are explicit.
4. Implement a thin adapter slice: parse one `PostToolUse`, extract known text,
   normalize, filter, reconstruct, validate, and pass through on every mismatch.
5. Add redaction to that slice under the approved narrowly worded guarantee;
   ensure no raw hook payload is logged.
6. Add structured capability facts and double-application protection without
   coupling them to proxy HTTP statistics.
7. Expand one schema at a time with tracked shape-preservation regressions.
8. Add MCP textual/multipart support only after its exact native shape is
   captured by fixtures.
9. Measure latency, context reduction, diagnostic retention, redaction corpus
   accuracy, fallback rate, and task quality.
10. Recommend native mode only after creation -> hook invocation -> rewrite ->
    model consumption -> persistence/resume behavior is verified end to end.
11. Separately deprecate Bash trim and decide proxy compatibility lifetimes;
    do not bundle proxy restructuring into the native vertical slice.

## 21. Verification Implications

No tests were run. Future deterministic regressions should cover:

- each pure primitive independently, including idempotence where expected;
- filter selection from authoritative tool identity/input;
- successful structured result field-local replacement;
- preservation of every non-target field, block order, multiplicity, metadata,
  image/binary/opaque value, and unknown field;
- invalid or unknown replacement shape falling back unchanged;
- no transformation of failed events;
- no raw hook payload in Clean-CTX logs/metrics;
- structured filter/redaction/normalization facts without sensitive values;
- existing marker rendering, if retained;
- serialized JSON filtering only when the selected field is truly text;
- hook+proxy double-application prevention;
- unsupported future shapes passing through unchanged;
- successful redaction persistence into later model/provider context, if that
  becomes part of the approved guarantee;
- explicit evidence that telemetry/transcript surfaces remain outside or inside
  the promised security boundary.

Required regressions must follow repository policy and live under
`src/tests/**`. Existing inline proxy tests and `proxy/tests/**` remain useful
historical evidence but are not substitutes for the required migration
regressions under the current policy.

## 22. Separate Current-Product Findings

The OpenAI/Anthropic cache-header candidate remains plausible and separate from
the native architecture:

- request-body cache injection uses the auto-detected adapter and runs only
  when `adapter.platform_name() == "anthropic"`;
- upstream forwarding instead checks configured `platform` and injects the
  Anthropic beta header whenever `AUTO_CACHE=1` and the configured platform is
  not explicitly `openai`;
- with auto-detected OpenAI (`platform=None`), body mutation can be skipped
  while the Anthropic header is still added.

Do not fix this inside native integration work. Open a separate falsification
investigation with a tracked request-capture regression for auto-detected
OpenAI, explicit OpenAI, Anthropic, and generic configurations.

## Evidence Index

Repository evidence inherited from the completed capability audit:

- `proxy/src/server.rs`, `pipeline.rs`, `transform.rs`, `platform/*.rs`
- `proxy/src/filters.rs`, `filter_rules.rs`, `filter_registry.rs`,
  `filter_loader.rs`, `community_filters.rs`, `filter_stats.rs`
- `proxy/src/scrub.rs`, `scrub_patterns.rs`, `cache.rs`, `logger.rs`,
  `config.rs`, `rate_limiter.rs`
- `src/config.rs`, `src/proxy_spawner.rs`, `src/mcp/server.rs`, `src/main.rs`
- `docs/PROXY.md`, `docs/CONFIGURATION.md`, and relevant proxy history

Native platform evidence consulted on 2026-10-05:

- [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) —
  successful/failed lifecycle events, structured result replacement, schema
  validation, and original telemetry timing.
- [Anthropic prompt caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) —
  automatic caching and explicit breakpoints.
- [Anthropic context editing](https://platform.claude.com/docs/en/build-with-claude/context-editing) —
  server-side tool-result clearing.
- [Anthropic tool search](https://platform.claude.com/docs/en/agents-and-tools/tool-use/tool-search-tool) —
  deferred tool definitions and on-demand discovery.
