# Architectural knowledge delivery boundaries

Agent: **Agent MaxHeadRoom**. Read-only B10 investigation, 2026-11-12.
The owner accepts all ten discovery probes as bounded operational evidence.
They are not repeated. No production or host policy is changed by this report.

## Findings by boundary

| Boundary | Evidence | Finding |
|---|---|---|
| Complete production response | Actual-root production receipts, exact serialized before/after pairs, source/currentness and declared membership checks | Verified for the recorded requests, including complete authority contexts. |
| Complete captured event payload | Native-byte CLI captures contain parseable full tool payloads, unit components, relationships and witnesses | Verified capture content for inspected requests. This is an event-emission boundary, not model delivery. |
| Complete host-delivered response | Raw serialized tool-result items submitted in the model request are not captured | Not established. A completed MCP event proves invocation and a recorded result, not the subsequent model-input representation. |
| Complete model-visible knowledge | No raw model-input/acknowledged-context evidence for these sessions | Not established. Host serialization, output handling or later context management may differ from event capture. |
| Actual agent inspection | Exact acquisitions, citations, source-supported answers, reported truncation and targeted recovery | Demonstrated for decisive cited knowledge; exhaustive inspection of all returned authority content is not established. |

These are separate boundaries. In particular, complete=true is the production
operation's declared completeness, not an end-to-end model-delivery receipt.
The raw event inspection previously described as host exact-content verification
establishes capture/source fidelity only. It does not establish that every byte
was included in the model's effective context. Earlier smoke success remains
actual invocation evidence, with this narrower interpretation of byte fidelity.

## Actual observations and limits

The earlier corrected-schema host turn retrieved AUTH.CANONICAL_IR context twice
after reporting truncated displays. Both recorded context payloads have 84,498
characters. The agent explicitly said its reduced display also truncated. The
native-byte run selected PIPELINE-001, whose declared closure is one unit, then
inspected related authority. The ten probes repeatedly report combined/context
display truncation and targeted acquisitions. These reports show incomplete
agent inspection of some presentations, not the exact internal truncation stage.

Raw captures are external CLI events; they contain no raw model-input messages.
The sessions used --ephemeral. A targeted check for the P02 thread's persisted
rollout in the applicable dated session directory found no matching file. The
installed npm package exposes its launcher and native executables, not the native
model-request preparation implementation. No adapter code, telemetry endpoint,
ordinary configuration or evaluation scenario is changed to obtain more data.

The old PowerShell-string event capture had a separate Unicode corruption issue.
That is resolved by native stdout/stderr copying and the original captures remain
preserved. It must not be confused with omission of large responses from model
context or used as evidence that the model received corrupted source.

## Documented mechanisms, not per-run attribution

The official [Codex MCP documentation](https://learn.chatgpt.com/docs/extend/mcp?surface=cli)
and [configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
describe per-tool output_token_limit as overriding a model-default output
truncation budget. The reference separately describes otel.tool_result.max_bytes
as affecting logged output only. Thus documented host-to-model output handling
and log-only shortening must not be conflated. These documents do not establish
which mechanism affected a particular recorded response, its effective default,
or the exact behavior of installed CLI 0.162.1. No numeric default is inferred.

No output-token override was supplied by the isolated launch manifests. This
does not prove an unlimited model delivery policy or absence of inherited host
defaults. Telemetry log excerpts would not independently prove full model input,
and raw-user-prompt logging is not a model-visible tool-result receipt.

## Disposition and conditional compatibility proposal

The strongest supported answer is: truncation was reported at the agent's available
presentation/inspection boundary; complete event payloads are retained; loss at
the actual host-to-model boundary is possible but not located or excluded by these
captures. It is not justified to call this display-only or to assert both stages
lost required knowledge. The accepted engineering answers remain bounded recovery
evidence, not universal complete-authority delivery evidence.

If an end-to-end guarantee is required, the next bounded investigation should
compare one existing exact authority response with the actual serialized
model-input tool item in a controlled supported host. Record response/argument
identity and hashes at each stage without exporting credentials or unrelated
conversations. Do not rerun the ten accepted discovery questions. Logging-only
truncation, host serialization loss, model-context exclusion and incomplete agent
inspection must receive separate dispositions.

If required knowledge is actually lost before model delivery, return a separately
authorized host compatibility repair preserving complete content, qualifications,
typed identities and truthful delivery/completeness reporting. Do not alter the
knowledge engine, raise or invent ceilings, prune results, split authority
automatically, or claim targeted get recovery is an equivalent complete context.
No such repair is implemented or declared proven necessary here. This is an
observability limitation and conditional proposal, not a new implementation mission.
