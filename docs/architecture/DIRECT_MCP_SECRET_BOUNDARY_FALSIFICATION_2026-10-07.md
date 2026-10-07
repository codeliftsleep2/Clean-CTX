# Direct-MCP Secret-Boundary Falsification

**Date:** 2026-10-07  
**Investigator:** Agent MaxHeadRoom  
**Status:** Bounded protocol experiment complete; live Claude evidence deliberately deferred to the final bundled pilot.  
**Scope:** Synthetic producer, disposable closed MCP stdio tool, serialized result inspection, and repository-established PostToolUse comparison. No Cargo, Clean-CTX production runtime, production implementation, real credential, MCP selection test, or provider-telemetry audit.

## Evidence vocabulary

- **PROVEN** — established by repository state or deterministic inspection within its stated scope.
- **DOCUMENTED** — stated by an authoritative producer, host, or protocol source.
- **OBSERVED** — measured in this bounded experiment without universal generalization.
- **INFERRED** — reasoned from proven, documented, or observed evidence.
- **UNRESOLVED** — insufficient legitimate visibility or missing environment evidence.

---

## 1. Verdict

**DIRECT MCP SECRECY ADVANTAGE SURVIVES WITH LIMITED CLAIM**

The bounded stdio experiment established that a closed MCP server can capture raw producer stdout and stderr, verify that both contain synthetic markers, redact both streams, and serialize only sanitized `content` and `structuredContent`. The property held for a successful producer, a producer exiting 17, and an intentional sanitizer failure that withheld all raw evidence.

The scoped search found the direct-MCP markers only in the deliberately fixture-owned marker file. They were absent from the serialized MCP responses, server audit, server stderr, and every other experiment-controlled artifact. The control marker appeared in the deliberately captured host-tool result before any later transformation.

The strongest defensible claim is:

> **OBSERVED:** Raw synthetic producer evidence remained inside the disposable MCP server's producer-capture boundary and was absent from the serialized MCP result delivered to the controlled client.

This does not prove the secret was absent from every Claude-owned lifecycle surface. No Claude CLI, Claude app, Claude browser tab, or callable Claude MCP host was available. Claude host state, model-visible rendering, Claude persistence, telemetry, and provider egress therefore remain **UNRESOLVED** rather than absent.

Within that limitation, direct MCP materially moves the controllable sanitization boundary earlier than `PostToolUse`: the MCP server sanitizes before emitting the tool response, whereas the existing hook receives host-provided `tool_response` after tool execution.

---

## 2. Exact Claim Tested

The narrow claim was:

> A closed direct MCP diagnostic operation can retain raw producer stdout and stderr inside its own capture/transformation boundary, fail closed when sanitization fails, and emit an MCP tool response containing only sanitized evidence.

The experiment did not test or claim:

- that a secret never exists in producer or server memory;
- that all MCP runtimes avoid process inspection, crash dumps, or unrelated OS telemetry;
- that Claude host internals never persist protocol traffic;
- that no provider receives the sanitized response;
- that arbitrary or hostile executables can be safely run;
- that the production Clean-CTX architecture already implements this boundary.

---

## 3. Synthetic Secret Design

The runner generated one random 96-bit hexadecimal nonce and derived three unmistakable markers:

```text
CLEAN_CTX_AMH_CONTROL_SECRET_<nonce>
CLEAN_CTX_AMH_STDOUT_SECRET_<nonce>
CLEAN_CTX_AMH_STDERR_SECRET_<nonce>
```

The exact values are intentionally omitted from this report. They were recorded locally in:

```text
scripts/investigations/direct-mcp-secret-boundary/evidence/fixture-owned-markers.json
```

That ignored file is intentional fixture input/evidence, not accidental persistence. No real key, password, token, credential, or private user data was used.

Classification: marker construction and local fixture ownership are **OBSERVED**.

---

## 4. Bash/PostToolUse Control

The synthetic producer emitted ordinary text, the control marker, and ordinary text into both stdout and stderr. The bounded control captured the complete producer result as a host-owned result object. Exact-marker search found the control marker in:

```text
control-host-tool-result.json
fixture-owned-markers.json
```

This establishes the mechanical comparison: when a host executes a producer first and constructs the tool result, the raw marker is already in that host-owned result before a later reducer can inspect it. Classification: **OBSERVED** for the disposable host-result control.

Repository production code independently establishes that the current adapter parses a `PostToolUse` envelope and reads `tool_response.stdout` and `tool_response.stderr`:

- `src/claude_native/bash.rs` accepts successful `PostToolUse` Bash results;
- `src/claude_native/pipeline.rs` transforms the already supplied tool response;
- `src/claude_native/mod.rs` records that Claude may persist raw output before invoking the hook;
- `docs/CONFIGURATION.md` records live Claude evidence that sufficiently large raw Bash output may be externalized before `PostToolUse`.

The earliest Clean-CTX authority in the compatibility path is therefore the hook invocation containing a host-provided result. It is not producer capture. Classification: **PROVEN** for repository adapter order; **DOCUMENTED/OBSERVED in prior authoritative project evidence** for Claude's pre-hook persistence behavior.

No new live Claude Bash/PostToolUse invocation was performed because no Claude host was available. The owner also requires live Claude work to be the final batched field gate because each pilot requires stopping the current process, rebuilding the binary, switching environments, and restoring context. Model-visible behavior in this run is **UNRESOLVED**, but does not justify an immediate environment switch.

---

## 5. Direct MCP Path

The disposable experiment used one closed tool:

```text
run_secret_boundary_probe
```

It accepted only three enumerated modes:

```text
success
failed_producer
redaction_failure
```

It accepted no executable, command, path, free-form argument, or producer input.

Observed path:

```text
synthetic producer
→ separate raw stdout/stderr capture
→ internal marker assertions
→ separate stdout/stderr redaction
→ result-object marker assertion
→ JSON-RPC serialization marker assertion
→ controlled MCP client receives response
```

Evidence by boundary:

| Boundary | Result | Classification |
|---|---|---|
| Producer output | Both exact direct markers present | OBSERVED |
| MCP-server raw capture | Both markers present under internal assertions | OBSERVED |
| Post-redaction stdout | Stdout marker absent | OBSERVED |
| Post-redaction stderr | Stderr marker absent | OBSERVED |
| Constructed tool result | Both markers absent | OBSERVED |
| Serialized JSON-RPC response | Both markers absent | OBSERVED |
| Controlled MCP client result | Both markers absent | OBSERVED |
| Actual Claude host result | Not invoked | UNRESOLVED |
| Actual model-visible result | Not invoked | UNRESOLVED |

The official MCP client model places `content` in the model-facing tool result and exposes `structuredContent` to the client/application. Both are carried in the tool-call response. Therefore, sanitizing both before serialization is the relevant server-controlled boundary. Classification: **DOCUMENTED** by the MCP SDK documentation and **OBSERVED** on the wire in this fixture.

---

## 6. stdout/stderr Results

The producer emitted distinct markers to separate pipes. The server used `capture_output=True` without merging streams.

| Stream fact | Result | Classification |
|---|---|---|
| Raw stdout contains stdout marker | Yes | OBSERVED |
| Raw stderr contains stderr marker | Yes | OBSERVED |
| Sanitized stdout contains either marker | No | OBSERVED |
| Sanitized stderr contains either marker | No | OBSERVED |
| MCP `content` contains either marker | No | OBSERVED |
| MCP `structuredContent.stdout` contains marker | No | OBSERVED |
| MCP `structuredContent.stderr` contains marker | No | OBSERVED |

Stream identity survived transformation: sanitized stdout and stderr remained distinct fields in `structuredContent` and separately labeled regions in `content`.

---

## 7. Successful Producer Result

The success case returned exit code 0. Internal audit established:

```text
raw_stdout_contains_marker: true
raw_stderr_contains_marker: true
sanitized_stdout_contains_marker: false
sanitized_stderr_contains_marker: false
result_contains_marker: false
```

The serialized result carried `isError: false`, sanitized `content`, and sanitized `structuredContent`. Classification: **OBSERVED**.

---

## 8. Failed Producer Result

The failed producer emitted both markers and exited 17. The server retained capture authority regardless of the nonzero exit and returned:

```text
status: producer_failed
exitCode: 17
isError: true
```

Both result surfaces contained sanitized, separately identified stdout and stderr. Neither marker appeared in the result or serialized response.

The secrecy boundary therefore did not depend on `exit code == 0`. Kill condition C was not observed. Classification: **OBSERVED**.

---

## 9. Redaction-Failure Result

The intentional redaction-failure case first captured and verified both raw markers. It then skipped construction of any evidence-bearing result and returned only:

```text
status: redaction_failure
rawEvidenceWithheld: true
isError: true
```

The text surface stated that sanitization failed and raw evidence was withheld. Neither raw nor partially sanitized producer output was used as fallback. The serialized response contained neither marker.

The experimental direct-MCP path can uphold:

> Failure to sanitize is not authority to expose unsanitized producer evidence.

Kill condition E was not observed. Classification: **OBSERVED**.

---

## 10. MCP Result Surfaces

| Surface | Inspection result | Classification |
|---|---|---|
| `content` success | Sanitized; markers absent | OBSERVED |
| `structuredContent` success | Sanitized; markers absent | OBSERVED |
| `content` failed producer | Sanitized; markers absent | OBSERVED |
| `structuredContent` failed producer | Sanitized; markers absent | OBSERVED |
| `content` redaction failure | Generic fail-closed text only | OBSERVED |
| `structuredContent` redaction failure | Status and withheld flag only | OBSERVED |
| `isError` | Correct for producer and sanitizer failures; no evidence payload bypass | OBSERVED |
| JSON-RPC `error` data | Used only for invalid method/tool/arguments; contains no producer evidence | PROVEN by fixture implementation |
| Result metadata/debug fields | No raw-evidence metadata introduced | OBSERVED |
| Server stdout | JSON-RPC responses only; markers absent | OBSERVED |
| Server stderr | Empty; markers absent | OBSERVED |
| Server audit | Boolean facts only; markers absent | OBSERVED |

The experiment avoided the asymmetric mistake where one of `content` or `structuredContent` remains raw.

---

## 11. Model-Echo Probe

**NOT PERFORMED — UNRESOLVED.**

Environment inspection found:

- no `claude` executable;
- no enabled Claude native app;
- no enabled browser or Claude browser tab;
- no callable Claude MCP host tool.

Asking Agent MaxHeadRoom to guess the marker would not test Claude and would contaminate the experiment. A model-echo result is useful only through the actual intended host after the sanitized MCP tool result is delivered.

That probe must be bundled into the eventual Clean-CTX Claude pilot with every other remaining Claude-only question. It should not trigger a dedicated rebuild/environment switch while local work remains.

Even a future negative echo would show only that the model could not reproduce the marker. It would not prove absence from host persistence, telemetry, or provider infrastructure.

---

## 12. Local Persistence

The scoped audit inspected only `scripts/investigations/direct-mcp-secret-boundary/evidence/`.

Exact-marker locations were:

| Marker | Locations |
|---|---|
| Control marker | Intentional marker file and intentional raw control result |
| Direct stdout marker | Intentional marker file only |
| Direct stderr marker | Intentional marker file only |

The direct markers were absent from:

- serialized wire-response capture;
- server audit JSONL;
- server stderr;
- summary JSON;
- all other experiment-controlled artifacts.

This supports only the claim that the disposable fixture did not persist raw direct-MCP producer evidence in its controlled outputs. Whole-machine logs, OS process inspection, crash dumps, security products, terminal-host persistence, and inaccessible host state were not searched. Those surfaces are **UNRESOLVED**.

The evidence directory is ignored and intentionally retained locally for review. It is not a tracked test and does not satisfy any CI gate.

---

## 13. Provider/Host Boundary

### Observed

- The controlled MCP client received only serialized sanitized responses.
- The server emitted no raw marker on stdout or stderr.
- No actual Claude host was connected.

### Documented

- MCP tool calls return server-constructed tool results containing `content` and, when used, `structuredContent`.
- Claude `PostToolUse` runs after tool execution and receives the existing tool result.

### Inferred

- A conforming host that obtains this tool result solely through the observed stdio channel cannot obtain the raw marker from that response because the marker is not present on the wire.
- Direct MCP therefore moves the Clean-CTX-controlled sanitization boundary before the host's tool-result construction, assuming the production server preserves the tested no-raw-logging rule.

### Unresolved

- Claude host internal tool-result state and transcript representation;
- Claude local persistence and debug logs;
- host process inspection or crash capture;
- host or provider telemetry outside the MCP response;
- provider/network egress;
- whether the actual Claude MCP integration consumes both surfaces exactly as expected in the target version.

No claim of zero leakage, no egress, or universal host secrecy is made.

---

## 14. Boundary Matrix

| Boundary | Bash/PostToolUse | Direct MCP | Classification |
|---|---|---|---|
| B0 producer output | Raw marker present | Raw stdout/stderr markers present | OBSERVED |
| B1 Clean-CTX raw capture | Hook receives host-provided result after execution | Server directly captured separate streams | PROVEN/OBSERVED |
| B2 post-redaction representation | Supported successes only; later than host result creation | Both markers absent | OBSERVED for direct path |
| B3 MCP serialized response | Not applicable | Both markers absent in every mode | OBSERVED |
| B4 Claude host/tool-result state | Raw result necessarily precedes PostToolUse authority; live run not repeated | Actual Claude host unavailable | INFERRED/DOCUMENTED versus UNRESOLVED |
| B5 model-visible result | Replacement possible on supported success; live run not repeated | Actual model unavailable | UNRESOLVED in this run |
| B6 local conversation/tool persistence | Prior project evidence shows possible pre-hook persistence | Fixture persistence clean; Claude persistence unavailable | DOCUMENTED/OBSERVED versus UNRESOLVED |
| B7 provider/network egress | Not inspected | Not inspected | UNRESOLVED |
| Failed producer | Current hook cannot replace failed result | Sanitized failure returned with `isError: true` | PROVEN/OBSERVED |
| Redaction failure | No pre-host fail-closed authority | Raw evidence withheld | OBSERVED |
| stdout/stderr | Host-provided fields after execution | Captured and sanitized separately | PROVEN/OBSERVED |

Direct MCP comparison result:

**YES — SUPPORTED BUT SOME HOST BOUNDARIES REMAIN UNRESOLVED**

---

## 15. Kill Conditions

| Kill condition | Tested? | Observed? | Result |
|---|---:|---:|---|
| A. Raw output must enter Claude-owned result state before sanitation | Protocol boundary yes; actual Claude host unavailable | No on controlled MCP wire; Claude internals unresolved | Not established |
| B. MCP construction requires raw evidence exposure | Yes | No | Rejected within fixture scope |
| C. Failed producer bypasses sanitation | Yes, exit 17 | No | Rejected within fixture scope |
| D. stderr bypasses sanitation | Yes, distinct marker | No | Rejected within fixture scope |
| E. Sanitizer failure falls back to raw | Yes | No | Rejected within fixture scope |
| F. Runtime automatically emits/persists raw output uncontrollably | Controlled stdout/stderr/artifacts yes; actual Claude runtime no | No controlled leak; wider runtime unresolved | Not established |
| G. Advantage indistinguishable from PostToolUse | Yes at producer/server/protocol boundary | No | Rejected at controlled boundary; live Claude comparison still needed |

No architecture kill condition was established. The missing Claude environment narrows the verdict; it does not supply contrary evidence.

---

## 16. Architectural Consequence

**Direct MCP retains a material secrecy advantage**, limited to the server-controlled producer-capture and MCP-response boundary.

PostToolUse first gains authority over a host-provided result after tool execution. Direct MCP can first gain authority at producer capture, before result construction. That is a material change in ownership and failure handling, not merely a different rendering of the same late result.

The advantage is not yet a complete Claude-host security claim. It must be stated as:

> Clean-CTX can sanitize captured producer evidence before emitting its MCP tool response; the bounded response contained no synthetic marker.

It must not be stated as:

> The raw secret can never enter any Claude, host, telemetry, or provider state.

---

## 17. Surviving Diagnostic Architecture

The surviving candidate remains:

```text
closed typed diagnostic request
→ trusted absolute producer
→ bounded OS-owned execution
→ separate authoritative stdout/stderr capture
→ fail-closed normalization/redaction
→ producer-specific semantic compilation
→ bounded sanitized raw evidence
→ MCP content + structuredContent built only from sanitized state
→ serialized model-facing result
```

Required constraints supported by this experiment:

- raw producer evidence must never be used directly to build `content`, `structuredContent`, error data, metadata, or logs;
- stdout and stderr must be sanitized independently before rendering;
- nonzero producer exit must not bypass redaction;
- redaction failure must withhold raw evidence and return an explicit processing failure;
- parser failure may retain already-sanitized bounded evidence, but redaction failure may not retain unsanitized evidence;
- exact marker/secret values must not enter routine audit facts.

This is a boundary statement, not a production design or implementation authorization.

---

## 18. Remaining Unknowns

### Blocks final Claude pilot signoff, not continued local design work

- One live Claude MCP invocation with the exact disposable closed tool or an equivalent probe.
- Inspection of the returned Claude tool result and legitimately available transcript/debug state.
- Model-echo probe through the actual Claude session.
- Confirmation that the target Claude version consumes sanitized `content` and `structuredContent` without exposing another server-controlled raw field.

These items should be executed together with selection reliability and any other accumulated Claude-only checks in one planned pilot session. They do not justify stopping local work individually.

### Blocks production hardening

- Panic/exception/crash behavior after capture but before redaction.
- Production logging and tracing configuration under every enabled mode.
- Memory lifetime and zeroization requirements, if any; none are authorized here.
- Bounded artifact/spool policy for large output.
- Redactor correctness against approved real secret classes.
- OS crash-dump and process-inspection threat model.

### Host/provider limitation Clean-CTX cannot resolve alone

- Inaccessible Claude host telemetry and persistence.
- Provider/network handling of the sanitized tool result.
- Guarantees about platform infrastructure outside the MCP server process.

### Later concern

- Other MCP clients and transports.
- Remote MCP deployment.
- Toolchains beyond the first closed diagnostic operation.
- Deliberately hostile producers, which remain outside scope.

---

## 19. Updated Falsification Chain

```text
semantic compilation       → survived bounded falsification
executable resolution      → survived with absolute-path constraint
parser truthfulness        → survived bounded falsification
Windows process ownership  → survived
Linux process ownership    → survived with no-detachment/live-controller limitations
direct-MCP secret boundary → survived with limited protocol-boundary claim; live Claude host unresolved
```

---

## 20. Next Gate

**Recommend MCP selection reliability as the next Claude-only gate, but defer it to the final bundled pilot.** The direct-MCP path remains a serious primary-interface candidate after surviving the producer-capture and serialized-response test.

Before that pilot, continue every safe local activity that can reduce uncertainty without Claude: architecture review, closed-operation contract refinement, deterministic fixture work, fail-closed behavior tests, protocol-surface auditing, and implementation planning or implementation only when separately authorized. Do not switch environments merely to close one observational cell.

The eventual pilot should use one rebuilt candidate and one preserved context package to answer, at minimum:

```text
1. Claude MCP tool selection reliability
2. actual content/structuredContent rendering
3. model-echo secrecy probe
4. legitimately inspectable transcript/debug persistence
5. representative real-workspace diagnostic behavior
```

Each outcome must remain independently classified; batching execution must not collapse selection, secrecy, and diagnostic-quality evidence into one verdict.

No MCP selection test was performed in this phase.

---

## Evidence inventory

Tracked disposable fixture:

```text
scripts/investigations/direct-mcp-secret-boundary/synthetic_producer.py
scripts/investigations/direct-mcp-secret-boundary/secret_boundary_server.py
scripts/investigations/direct-mcp-secret-boundary/run_falsifier.py
```

Ignored local evidence:

```text
scripts/investigations/direct-mcp-secret-boundary/evidence/fixture-owned-markers.json
scripts/investigations/direct-mcp-secret-boundary/evidence/control-host-tool-result.json
scripts/investigations/direct-mcp-secret-boundary/evidence/server-audit.jsonl
scripts/investigations/direct-mcp-secret-boundary/evidence/mcp-wire-responses.json
scripts/investigations/direct-mcp-secret-boundary/evidence/server-stderr.txt
scripts/investigations/direct-mcp-secret-boundary/evidence/summary.json
```

Bounded execution result:

```text
Python: 3.13.5
Platform: win32
Server exit: 0
All protocol assertions: passed
Controlled direct-marker leaks: none
```

No Cargo command, production binary, production MCP server, Windows/Ubuntu process-lifetime fixture, or MCP selection experiment was run.
