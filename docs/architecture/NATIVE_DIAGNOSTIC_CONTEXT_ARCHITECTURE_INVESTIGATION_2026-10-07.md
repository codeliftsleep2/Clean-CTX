# Clean-CTX Diagnostic Context Architecture Investigation

**Date:** 2026-10-07  
**Status:** Investigation complete; architecture recommendation only. No implementation authorization is implied.  
**Scope:** Native diagnostic/build/test context acquisition, normalization, redaction, semantic compilation, reduction, and agent exposure for Clean-CTX's supported ecosystems: Rust, TypeScript/Angular, C#/.NET, and Java/Spring.  
**Excluded:** Production implementation, test changes, proxy modification, diagnostic-IR implementation, and Cargo execution.

## Evidence vocabulary

This report uses the following classifications:

- **PROVEN** — established by current repository implementation or existing verified repository evidence.
- **DOCUMENTED** — stated by an authoritative upstream specification or vendor document.
- **OBSERVED** — seen in an existing live or repository-owned experiment but not guaranteed as a platform contract.
- **INFERRED** — a reasoned architectural conclusion from proven or documented facts.
- **UNRESOLVED** — evidence is insufficient and a bounded experiment is required.

The investigation deliberately treats the current HTTP proxy, Claude-native hook, proposed runner, and possible MCP surfaces as candidates rather than predetermined answers.

---

## 1. Executive Verdict

**LAYERED ARCHITECTURE IS REQUIRED — RESTRICTED DIAGNOSTIC EXECUTION SHOULD BECOME THE AUTHORITATIVE PATH, EXPOSED PRIMARILY THROUGH ONE TYPED MCP TOOL**

The strongest practical architecture is:

```text
coding agent
    │
    ├─ preferred: one typed MCP diagnostic tool
    ├─ secondary: Clean-CTX diagnostic CLI
    └─ compatibility: ordinary Bash + PostToolUse
                         │
                         ▼
              closed DiagnosticOperation
                         │
                         ▼
             producer-specific execution policy
                         │
                direct spawn, no shell
                         │
        authoritative streams/status/artifacts
                         │
                         ▼
        producer-native structured output parser
                         │
                         ▼
              typed diagnostic result
                + bounded raw evidence
                + reduction/redaction facts
                         │
                         ▼
              model-facing concise rendering
```

The layers are not equivalent or independently primary:

1. **Producer-native concise or structured output** is the preferred acquisition format, not a sufficient integration architecture.
2. **Restricted diagnostic execution** is the authoritative capture boundary for supported operations.
3. **One typed MCP tool** is the preferred Claude-facing interface.
4. **A CLI adapter** exposes the same execution capability to humans and other agents.
5. **`PostToolUse`** remains a conservative compatibility fallback when an agent uses ordinary Bash.
6. **Provider-egress rewriting** remains optional defense-in-depth and must not be represented as local lifecycle protection.

The central architectural correction is:

> Diagnostics should become a compiled semantic context product, not primarily filtered terminal text.

Clean-CTX should request the producer's strongest stable machine-readable representation, convert it into producer-authored diagnostic facts, preserve bounded supporting evidence, and render only what the current agent decision needs. This aligns diagnostic handling with Clean-CTX's existing purpose: compile noisy source representations into semantically useful context rather than repeatedly forcing a model to interpret raw material.

The runner recommendation is conditional. It remains valid only while the execution surface is a closed set of diagnostic operations. A generic `clean-ctx exec`, arbitrary package-script execution, generic argument passthrough, or shell MCP tool would destroy the authority and security properties that justify owning execution.

### Confidence summary

- **PROVEN:** The current adapter transforms successful Bash `stdout` and `stderr`, uses a conservative command recognizer, retains streams separately, and rejects ambiguous shell compounds.
- **DOCUMENTED:** Claude `PostToolUse` fires after execution; original output may already enter telemetry; failed Bash output is interleaved and can be middle-truncated; failure hooks cannot replace the failed result.
- **DOCUMENTED:** MCP supports validated structured tool results through `outputSchema` and `structuredContent`.
- **INFERRED:** A typed MCP tool should provide more reliable selection and arguments than a CLI convention the model must remember.
- **UNRESOLVED:** Actual MCP-versus-Bash selection reliability must be measured in Claude rather than assumed.

---

## 2. Actual Clean-CTX Requirements

The existing hook schema must not define the requirements. Starting from the desired product outcome produces the following classification.

### MUST HAVE

- **Authoritative operation identity.** Clean-CTX must know that the request is, for example, Cargo check rather than infer it from arbitrary output text.
- **Authoritative process outcome.** This includes exit status where available and a separate process-launch failure classification.
- **Working-directory identity.** Relative diagnostics, project discovery, artifact paths, and executable resolution depend on it.
- **Separate stdout and stderr.** Streams must remain distinct when the producer exposes them distinctly.
- **Complete diagnostic capture up to an explicit local resource boundary.** Model rendering may be bounded; acquisition must not silently lose the evidence required to form the result.
- **Deterministic normalization and recognized-secret redaction before model visibility.**
- **Truthful failure semantics.** A parser or reducer failure must not become a successful or empty diagnostic result.
- **Producer-specific structured parsing.** Producer contracts, not a universal regex catalog, determine semantic facts.
- **Bounded model-facing representation with disclosure.** Any omitted evidence must be explicit.
- **Unparsed-evidence preservation.** Unknown compiler failures, stack traces, plugin output, and parser-version drift must remain diagnosable.
- **Cancellation propagation and process-tree cleanup.**
- **Direct spawning without shell interpretation.**
- **Fail-open-to-evidence behavior.** When semantic compression is unsafe, preserve sanitized evidence rather than invent confidence.

### SHOULD HAVE

- Producer executable identity and version when obtained without altering semantics.
- Timing, cancellation, timeout, and interruption facts.
- Artifact locations produced by the operation.
- Parser and result-schema version.
- Redaction, normalization, parsing, and reduction facts.
- Bounded raw evidence selected according to producer semantics rather than a universal head-only cap.
- Explicit counts of parsed and unparsed records.
- Cross-platform process-lifecycle verification on Windows and Unix.
- A stable rejection reason for unsupported operations or arguments.
- Human-readable and machine-readable renderings produced from the same result authority.

### USEFUL

- Result caching when freshness can be proven from authoritative inputs and filesystem state.
- Environment-difference facts when a producer's behavior specifically depends on them.
- Resource links to locally retained evidence or artifacts.
- Explicitly configured producer paths.
- CLI-oriented human rendering.
- Aggregate performance facts that do not retain sensitive raw content.

### UNNECESSARY

- Complete environment snapshots.
- A universal schema covering every compiler, builder, linter, and test runner.
- Raw terminal text in every successful model-facing result.
- One MCP tool per operation.
- Shell parsing inside Clean-CTX.
- Provider-specific result types in the execution core.
- Persistence of every successful diagnostic invocation.

### ACTIVELY UNDESIRABLE

- Generic command execution.
- Inferring success from output text.
- Combining stdout and stderr.
- Treating package-script names as authoritative producer identities.
- Silent truncation.
- Concise success summaries when parsing or redaction failed.
- Persisting unredacted raw output by default.
- Claims of protection before Clean-CTX's actual interception point.
- Global timeout, retry, caching, or truncation defaults introduced as incidental implementation details.

---

## 3. Current `PostToolUse` Boundary

### Current repository behavior

The existing implementation is conservative and coherent within its chosen boundary:

- `src/claude_native/bash.rs` recognizes only `PostToolUse` events for `Bash` with string `stdout` and `stderr`, and rejects interrupted or image results.
- `src/claude_native/pipeline.rs` normalizes, redacts, and optionally filters the two streams separately before reconstructing the original Bash output shape.
- `src/claude_native/command.rs` derives a narrow operation identity from a deliberately limited command tokenizer.
- The recognizer rejects chaining, pipelines, redirection, substitution, subshells, and command groups rather than pretending to be a shell parser.
- `src/claude_native/mod.rs` explicitly records that Claude may create persisted raw output before invoking the hook; dropping a pointer cannot erase the already persisted content.

The existing architecture freeze correctly limits its guarantee to selected successful model-visible results. The current supported-toolchain freeze also correctly makes stream authority operation-sensitive rather than granting generic stderr filtering to every recognized executable.

### What `PostToolUse` actually guarantees

For a supported successful Bash result:

- Claude supplies tool identity, tool input, and the structured successful result.
- Clean-CTX can transform known text fields.
- Clean-CTX can keep stdout and stderr separate.
- Clean-CTX can replace the result before delivery to the model.
- The adapter can validate and reconstruct the original result shape.
- Unsupported commands and shapes can pass through unchanged.
- Existing ordinary Bash workflows require no new agent behavior.

Claude documents that `updatedToolOutput` replaces what Claude receives and must match the tool's output shape. Built-in tool replacements that fail schema validation are ignored in favor of the original output.

### Where the boundary fails

`PostToolUse` cannot guarantee:

- redaction before Claude infrastructure first receives the output;
- redaction before original-output telemetry;
- absence of unredacted local persistence;
- complete hook input if the host has already truncated or externalized output;
- replacement of failed results;
- separate failed-result streams;
- underlying producer identity through aliases, functions, wrappers, or arbitrary package scripts;
- safe semantic reduction of compounds or redirected commands;
- authoritative producer exit semantics beyond the host's event classification.

Claude's current failure-hook documentation is especially important:

- Bash and PowerShell failure output arrives as an `Exit code N` line followed by one block with stdout and stderr interleaved.
- The error string may be middle-truncated.
- Claude may insert its own timeout text.
- Process-start failures can omit an exit-code line.
- `PostToolUseFailure` can add context but cannot replace the failed result.

These are not theoretical weaknesses. They prevent a failure hook from reconstructing stream authority or recovering evidence already removed by the host.

### Conclusion

> `PostToolUse` is a strong compatibility boundary and a weak primary authority boundary.

It should remain, but its truthful product claim is successful-result model-visible transformation for recognized Bash result shapes.

---

## 4. Producer-Native Configuration Findings

Producer-native reduction should be used whenever it preserves the evidence Clean-CTX needs. It eliminates noise before Clean-CTX must capture, parse, retain, or redact it.

The preferred rule is:

```text
request the richest stable machine representation available,
not necessarily the quietest textual representation
```

Quiet modes can suppress diagnostic facts. Structured modes allow Clean-CTX to choose what to render without asking the producer to discard information prematurely.

### What producer configuration can solve

- ANSI, progress bars, and terminal redraw noise.
- Human-oriented repetition.
- Stable diagnostic record extraction.
- Test-result artifacts.
- Artifact and target identity.
- Some summary and timing information.

### What it cannot solve alone

- uniform process outcome and launch-failure semantics;
- redaction timing;
- cancellation and process-tree ownership;
- model-facing bounds and disclosure;
- integration selection by a coding agent;
- cross-producer result rendering;
- unexpected output from build scripts, plugins, tests, or child processes.

### Safe management policy

Clean-CTX may add documented ephemeral command-line flags required for its diagnostic operation. It should not silently edit repository configuration to install reporters or change normal developer behavior. Persistent configuration changes require an explicit user or repository decision.

---

## 5. Restricted Diagnostic Runner Findings

### Why owning execution materially improves correctness

A restricted runner knows the operation before launch. It can therefore:

- select the producer and exact supported operation without command inference;
- construct known arguments directly;
- request the producer's structured format;
- observe process-launch success or failure;
- obtain the exit status directly;
- drain stdout and stderr concurrently and separately;
- associate artifacts with the invocation;
- redact before returning a result to Claude;
- preserve bounded raw evidence not shown in the concise rendering;
- distinguish cancellation, timeout, crash, producer failure, parse failure, and Clean-CTX failure;
- return one typed result across CLI and MCP adapters.

### Responsibility cost

The runner creates serious new obligations:

- executable discovery and validation;
- argument construction and validation;
- working-directory validation;
- environment inheritance policy;
- stdin policy;
- concurrent stream draining;
- bounded memory and optional spooling;
- secret-bearing temporary data;
- cancellation and timeout behavior;
- descendant-process cleanup;
- Windows command resolution and job-object behavior;
- Unix process groups and signal behavior;
- artifact cleanup;
- multiple concurrent operations;
- crash-safe cleanup.

### Feasibility verdict

The burden is acceptable only under a narrow contract:

```text
closed operation enum
+ producer-owned typed arguments
+ direct process spawn
+ trusted executable-resolution policy
+ bounded capture/spooling
+ explicit cancellation
```

The following design is rejected:

```text
clean-ctx exec <arbitrary shell command>
```

So are equivalent forms hidden behind an argument array or MCP string field.

### Capture model

The runner should stream both output handles concurrently. It should parse incremental records where the format permits and retain a bounded sanitized evidence buffer. Complete raw output need not always remain in memory. If local spooling becomes necessary, the architecture must first freeze:

- memory threshold;
- maximum spool size;
- secure creation permissions;
- cleanup behavior;
- crash recovery;
- whether failed runs retain more evidence than successful runs.

Those are externally observable policies and require explicit approval before implementation.

---

## 6. MCP Diagnostic Tool Findings

### Many operation-specific MCP tools

Possible examples include `clean_ctx_cargo_check`, `clean_ctx_tsc`, and `clean_ctx_dotnet_test`.

Advantages:

- narrowly descriptive schemas;
- simple per-tool permission rules;
- strong operation-specific arguments;
- clear names in clients without tool discovery.

Disadvantages:

- tool proliferation;
- repeated workspace, target, and rendering fields;
- larger catalog and description surface;
- repeated lifecycle logic unless carefully centralized;
- pressure to add one tool for every producer variation;
- greater risk of inconsistent result contracts.

This option is not justified for the current operation count.

### One generic diagnostic command tool

A tool such as this is rejected:

```json
{
  "command": "cargo check --all-features"
}
```

It recreates the shell authority problem behind an MCP label. The same objection applies to `{ executable, args[] }` when both are arbitrary.

### One typed diagnostic tool

The preferred surface is one tool with a closed discriminated request:

```text
run_diagnostic {
  operation:
      CargoCheck
    | CargoBuild
    | CargoClippy
    | TscCheck
    | AngularBuild
    | AngularTest
    | Eslint
    | DotnetBuild
    | DotnetTest
    | MavenBuild
    | MavenTest,
  workspace_root,
  operation-specific fields
}
```

The schema should use a tagged union or equivalent closed representation. It must not combine an operation name with free-form arguments.

### Why MCP is a good interface

The MCP tool contract supports:

- JSON Schema input validation;
- an optional output schema;
- `structuredContent` for machine-readable results;
- ordinary `content` for model-facing or backward-compatible text;
- error results;
- resource links for locally retained evidence or artifacts.

Current Claude Code documentation also states that MCP tool search defers tool definitions and discovers them on demand. That materially reduces, but does not eliminate, the context-cost objection to a typed diagnostic surface.

### Remaining uncertainty

MCP supplies a technically strong interface. It does not prove that Claude will select the tool instead of familiar Bash commands. Selection reliability remains an experiment, not an architectural fact.

---

## 7. CLI vs MCP Interface Findings

The CLI and MCP surfaces should be adapters over one execution and parsing core:

```text
CLI adapter ─┐
             ├─ DiagnosticRequest → execution/parsing core → DiagnosticResult
MCP adapter ─┘
```

This separation is justified because:

- humans and non-MCP agents need a CLI;
- MCP needs typed schemas and structured output;
- execution, redaction, parsing, and failure logic must not be duplicated;
- deterministic CLI tests can validate the core independently of Claude;
- provider-specific transport details should remain outside producer implementations.

This does not justify an adapter framework, plugin registry, or trait hierarchy in the first slice. A direct request enum and explicit match dispatch are sufficient until multiple concrete implementations reveal a stable abstraction.

The distinction is:

```text
execution capability ≠ agent integration surface
```

---

## 8. Structured Diagnostic Findings

Clean-CTX should move from:

```text
raw terminal text
→ remove known noise
→ return shorter terminal text
```

to:

```text
authoritative producer evidence
→ producer parser
→ diagnostic facts
→ task-specific rendering
```

### Is diagnostic output source material?

Yes. Compiler, linter, builder, and test-runner output is a representation of program facts. It commonly contains identity, locations, severities, codes, relationships, outcomes, and artifacts. Repeatedly asking a model to recover those facts from decorated terminal text is equivalent to repeatedly asking it to parse a noisy source representation.

This makes diagnostics an appropriate Clean-CTX compilation domain.

### Minimal diagnostic representation

A minimal result is justified:

```text
DiagnosticResult
├─ operation
├─ producer
├─ outcome
├─ diagnostics[]
│  ├─ severity
│  ├─ message
│  ├─ code?
│  ├─ primary_location?
│  ├─ related_locations[]
│  └─ producer_metadata?
├─ test_summary?
├─ artifacts[]
├─ raw_evidence[]
├─ parser_coverage
└─ transformation_facts
```

The model should not force every producer to supply every field. Builds, tests, and linters are related domains, not identical domains. Producer-owned extensions or result variants are preferable to fabricated common fields.

### Authority rules

- The process supervisor owns launch and exit facts.
- The producer parser owns diagnostic interpretation.
- Artifacts own facts parsed from their documented format.
- Raw text never overrides an authoritative exit status.
- The renderer owns model-facing selection, not canonical diagnostic truth.
- A concise rendering is non-reversible presentation, not persistent authority.

---

## 9. Raw Evidence Preservation

Structured extraction must not imply that all raw output is disposable.

Raw evidence remains necessary for:

- internal compiler failures;
- toolchain crashes;
- unknown plugin output;
- stack traces;
- malformed or future structured records;
- incomplete test artifacts;
- launch and environment failures;
- debugging Clean-CTX's parser;
- producer messages that do not fit the diagnostic representation.

The correct model is neither discard-all nor return-all:

```text
DiagnosticResult
├─ authoritative operation and outcome
├─ structured facts
├─ concise summary
├─ bounded sanitized supporting evidence
├─ unparsed-record facts
└─ reduction/redaction disclosure
```

### Evidence selection

A universal head-only truncation rule is unsafe because important summaries often occur at the tail. Producer policies should choose among:

- first relevant launch evidence;
- every structured diagnostic within a diagnostic count bound;
- tail completion summary;
- first and last unparsed regions;
- failure-local stack trace segments;
- explicit omission counts.

### Full local evidence

If complete local evidence is retained temporarily, the model-facing result may expose a non-secret resource handle rather than embed it. Retention is optional and requires an approved lifecycle policy. A handle must never imply that evidence is durable if cleanup may remove it.

---

## 10. Secret / Persistence Boundary Matrix

| Architecture | Before Clean-CTX sees it | Before Claude infrastructure | Before Claude persistence/telemetry | Before model visibility | Before provider egress |
|---|---:|---:|---:|---:|---:|
| Producer-native configuration | No | No | No | Only when producer omits the value | Indirect only |
| `PostToolUse` | No | No | No | Supported successes only | Supported replacement only |
| Restricted runner through MCP | No — producer emits to Clean-CTX | Yes for returned producer output | Yes for the returned result; invocation metadata remains host-owned | Yes | Yes |
| Diagnostic CLI invoked through Bash | No | No — Bash owns the result lifecycle | Not guaranteed | Hook may protect a successful result | Hook/gateway dependent |
| HTTP gateway | No | No | No | Too late for local lifecycle protection | Yes for covered requests |
| Structured artifact ingestion | Artifact already exists locally | Yes when read directly by Clean-CTX | Yes for the returned result | Yes | Yes |

### Precise guarantees

A direct MCP runner can prevent raw producer output from becoming the Claude tool result because Clean-CTX owns the subprocess and returns only its sanitized result. It cannot prevent secrets from existing in:

- the producer process;
- inherited environment;
- Clean-CTX process memory;
- producer-created artifacts;
- explicitly enabled temporary spools;
- external systems contacted by the producer.

The CLI loses this advantage when invoked through Bash because its own output becomes a Bash result and re-enters Claude's ordinary shell lifecycle.

### Logging policy

- Raw producer output must not be logged by default.
- Secret-bearing hook payloads must not be logged.
- Structured telemetry should contain counts, operation identity, sizes, timing, parser coverage, and fallback reasons.
- Temporary evidence must use restrictive permissions and deterministic cleanup.
- Redaction facts may be retained; matching secret values may not.

---

## 11. Failure-Semantics Findings

The primary path should distinguish at least:

```text
LaunchFailed
Cancelled
TimedOut
Exited { code }
Signaled { signal }          // where the platform provides it
Terminated { reason }        // portable projection
ParseIncomplete
InternalProcessingFailed
```

These are not all mutually exclusive layers. A process may exit non-zero while parsing is incomplete, and both facts must survive.

### Required rules

- Exit status remains authoritative even if output says `success` or `failed` incorrectly.
- Non-zero exit does not prevent structured diagnostic extraction.
- Launch failure is distinct from a producer-reported diagnostic failure.
- Parser failure does not replace the process outcome.
- Unparsed content is retained as bounded sanitized evidence.
- Failure rendering normally preserves more evidence than clean-success rendering.
- A Clean-CTX crash or parser defect must never produce an empty success.
- A redaction failure is more severe than a parse failure because unsafe evidence must not be returned.
- When safe semantic rendering is impossible, return an explicit processing failure plus minimal sanitized process facts.

### Failure fallback hierarchy

```text
structured facts complete
    ↓ otherwise
structured facts + bounded unparsed evidence
    ↓ otherwise
sanitized bounded raw evidence + authoritative process outcome
    ↓ otherwise
explicit Clean-CTX processing error + minimal process outcome
```

Aggressive reduction must never outrank truthful failure evidence.

---

## 12. Process-Security Findings

If Clean-CTX launches processes, the following minimum responsibility boundary must be frozen before production implementation.

### Operation and executable policy

- Closed operation allowlist.
- Producer-owned typed arguments.
- No arbitrary executable parameter.
- No shell.
- No arbitrary package scripts.
- No free-form environment overrides initially.
- Explicit producer resolution strategy.

Acceptable producer resolution sources may include:

1. an explicitly configured absolute executable path;
2. a trusted system/toolchain installation;
3. an ecosystem-approved project-local binary resolution with explicit identity rules.

A repository-local executable must not be trusted merely because it appears first on `PATH`.

### Working directory

- The requested root must be canonicalized.
- It must fall within the caller-approved workspace boundary.
- Operation-specific project paths must remain within that boundary.
- Symlink traversal and path escape must be rejected.
- Relative artifact paths must be resolved against the authoritative operation cwd.

### Arguments

- Construct arguments from typed fields.
- Reject unknown operation fields.
- Do not reinterpret repository strings as switches.
- Do not allow response files or producer-specific escape hatches until explicitly reviewed.
- Treat plugin goals and build targets as executable capability expansion, not harmless strings.

### Environment

- Start with inherited environment minus explicitly dangerous or unnecessary variables only if that policy is approved.
- Never return the inherited environment wholesale.
- Do not allow arbitrary environment maps in the first version.
- Record only environment facts required to interpret the result.

### Standard I/O

- stdin closed by default.
- stdout and stderr drained concurrently.
- decoding errors represented explicitly; do not silently replace arbitrary bytes as if they were authoritative text.
- structured protocols parsed incrementally when possible.

### Cancellation and children

Windows requires explicit consideration of:

- `.exe`, `.cmd`, and `PATHEXT` resolution;
- toolchain shims;
- job objects or another descendant-ownership mechanism;
- console control behavior;
- forceful fallback termination.

Unix requires explicit consideration of:

- process groups or sessions;
- graceful signal followed by bounded escalation;
- descendants that detach or re-parent;
- signal-derived termination status.

### Timeouts and resource limits

Timeouts, byte caps, memory bounds, and spool limits change externally observable behavior. They are architectural policies, not incidental constants. The implementation phase must stop for approval before selecting global defaults.

---

## 13. Supported-Toolchain Findings

### Rust

Cargo offers the strongest first structured boundary. With `--message-format=json`, Cargo emits JSON-lines records for compiler messages, artifacts, build-script execution, and build completion. Compiler diagnostics include structured rustc data and a rendered representation.

Limitations:

- the flag controls Cargo and rustc output, not arbitrary procedural-macro or executed-program output;
- tests do not have an equivalently stable general-purpose structured stream on stable Rust;
- build scripts and native tools may still emit unstructured evidence;
- `cargo test` crosses compiler, test-harness, and arbitrary test-program boundaries.

Conclusion: start with `cargo check`; defer Cargo test.

### TypeScript

The standard `tsc` CLI supports non-pretty deterministic text via `--pretty false` but does not provide an equivalent documented rich JSON diagnostic stream. Clean-CTX will likely need a producer-specific textual parser or a compiler-API adapter.

Using the compiler API would increase dependency and version responsibility. It should not be chosen until the CLI parser's limits are measured.

### Angular

Angular CLI output depends on the configured builder, Angular version, and test runner. Build artifacts and stats may provide useful structure, but there is no single universal Angular diagnostic stream covering build, lint, and test.

The runner must preserve operation identity and discover the configured builder rather than assume one output dialect.

Watch and dev-server operations remain excluded.

### ESLint

ESLint's JSON formatter supplies structured lint results with file, message, location, severity, and rule identity. This is a strong structured candidate.

Clean-CTX must preserve warnings and distinguish clean success from a successful process that reports lint findings under configuration-specific exit policies.

### .NET build

`dotnet build` exposes:

- verbosity controls;
- terminal logger control;
- MSBuild logger configuration;
- binary logs;
- build artifacts.

The binary log is rich but introduces a binary artifact parser and retention concern. The first .NET build slice should compare a structured/custom logger approach against parsing concise deterministic console output before adopting binlog ownership.

### .NET test

VSTest supports TRX and other result loggers. TRX is a strong semantic artifact for test identity, outcome, timing, and failure detail.

Limitations include:

- multi-targeted projects producing multiple result files;
- overwritten filenames when improperly configured;
- missing or incomplete artifacts after test-host crashes;
- framework output outside the result document;
- build diagnostics that precede test execution.

The process result and bounded raw evidence remain necessary even when TRX is primary.

### Maven/Spring build

Maven is a plugin execution framework. Core lifecycle status is stable, but arbitrary plugins can produce heterogeneous output and side effects. Quiet mode may remove useful context and cannot create a universal structured build stream.

The allowed goal set must remain closed. Arbitrary plugin goals are equivalent to capability expansion.

### Maven test

Surefire and Failsafe produce XML reports with test cases, outcomes, timings, failures, and captured output. These are strong semantic artifacts.

They may be missing or incomplete after early build failure, fork startup failure, timeout, or crash. Process status and sanitized raw evidence remain necessary.

### Toolchain summary

| Ecosystem | Preferred structured evidence | Primary limitation |
|---|---|---|
| Rust check/build/clippy | Cargo JSON messages | Non-JSON child/build-script evidence |
| TypeScript | Non-pretty deterministic text initially | No standard rich CLI JSON stream |
| Angular | Builder/test artifacts plus bounded console evidence | Builder and runner variability |
| ESLint | JSON formatter | Plugin and exit-policy differences |
| .NET build | Logger output or binlog | Parser and artifact ownership |
| .NET test | TRX plus process outcome | Crash and multi-target completeness |
| Maven build | Lifecycle status plus plugin-specific evidence | Plugin heterogeneity |
| Maven test | Surefire/Failsafe XML | Missing or incomplete crash evidence |

---

## 14. UX / Adoption Findings

### Ordinary Bash

Advantages:

- natural existing agent behavior;
- no additional tool-selection burden;
- transparent developer experience;
- works across coding agents.

Disadvantages:

- ambiguous producer identity;
- host owns output lifecycle before Clean-CTX;
- failure stream merging and truncation;
- package-script opacity;
- weaker redaction timing.

### Clean-CTX CLI

Advantages:

- provider-neutral;
- useful for humans, CI, and other agents;
- simple manual reproducibility;
- one process result can have human and JSON renderings.

Disadvantages:

- the agent must remember to use it;
- when invoked through Bash, Claude still owns the outer Bash lifecycle;
- command syntax consumes prompt/instruction space;
- permissions may not distinguish it from other shell activity.

### Typed MCP tool

Advantages:

- schema-guided arguments;
- discoverability;
- distinct permission surface;
- structured return channel;
- no outer Bash result lifecycle;
- provider-neutral protocol and reusable execution core.

Disadvantages:

- selection reliability is unproven;
- setup and server connectivity are required;
- clients vary in MCP feature support;
- tool descriptions and server instructions require maintenance.

### Recommended UX policy

- Make the typed MCP operation the documented preferred path.
- Keep the CLI available and deterministic.
- Preserve Bash compatibility without blocking or surprise redirection.
- Record non-sensitive fallback facts so adoption can be measured.
- Consider hook guidance only after measuring actual fallback frequency.
- Never claim MCP exclusivity until tool-selection reliability is demonstrated.

---

## 15. Performance / Context Findings

### Expected economics

Producer-native structured formats may generate more local bytes than concise terminal text because records repeat keys and metadata. That is acceptable if Clean-CTX sends substantially fewer, more meaningful tokens to the model.

The relevant measures are therefore separate:

- raw bytes generated by the producer;
- bytes read by Clean-CTX;
- peak in-memory bytes;
- spooled bytes;
- parsed diagnostic count;
- unparsed bytes or records;
- bytes returned to the client;
- estimated and actual model tokens;
- subprocess duration;
- parsing and rendering duration.

### Architecture effects

| Candidate | Producer bytes | Clean-CTX-held bytes | Model tokens | Added latency |
|---|---|---|---|---|
| Producer quiet mode | Low–medium | None or low | Low–medium | Minimal |
| `PostToolUse` filter | Unchanged | Full host-provided result | Low on supported success | Hook process and parse |
| Structured runner | Medium–high locally | Bounded with streaming | Lowest expected | Spawn + parse + MCP serialization |
| Artifact ingestion | Artifact-dependent | Bounded parser state | Low | Post-process artifact read |
| HTTP gateway rewrite | Unchanged | Provider request envelope | Low at egress only | Gateway traversal |

No numerical savings are claimed without measurements.

### Caching

Execution-result caching is not recommended initially. It creates difficult freshness, environment, artifact, and filesystem authority questions. Parser-level memoization of immutable retained artifacts may be considered later, but only with explicit identity and lifecycle rules.

---

## 16. Architecture Comparison Matrix

Ratings are qualitative. For burden, friction, and complexity, **High** is unfavorable.

| Candidate | Authority correctness | Complete-output access | Stream identity | Exit authority | Redaction timing | Local-persistence protection | Egress protection | Structured potential | Failure truthfulness | Shell ambiguity | Process-security burden | Cross-platform burden | Claude adoption | Other-agent portability | Developer friction | Implementation complexity | Maintenance complexity | Reduction potential | Extensibility |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| A. Producer-native configuration | Low | Producer-dependent | Producer-dependent | Low | Low | Low | Low | Medium–High | Medium | Medium | Low | Low | High | High | Low | Low | Medium | Medium | Medium |
| B. Existing `PostToolUse` | Medium on success | Low | High on successful Bash; low on failure | Host event only | Late | Low | Medium | Low–Medium | Low on failure | High | Low | Low | High | Claude-specific | Low | Medium | Medium–High | Medium | Low–Medium |
| C. Restricted CLI runner | High | High within bounds | High | High | High before returned output | Medium when invoked directly; low through Bash | Medium | High | High | Low | High | High | Medium | High | Medium | High | High | High | High |
| D1. Many MCP tools | High | High | High | High | High | High | High | High | High | Low | High | High | Medium | Medium–High | Medium | High | High | High | Medium |
| D2. One typed MCP tool | High | High | High | High | High | High | High | High | High | Low | High | High | High but unproven | High | Low–Medium | Medium–High | Medium | High | High |
| E. CLI + MCP shared core | High | High | High | High | High through direct adapter | High through MCP | High | High | High | Low | High | High | High but unproven | High | Low–Medium | High initially | Medium | High | High |
| F. Hook guidance hybrid | Low | Low | Host-dependent | Host-dependent | Late | Low | Low | Low | Low | High | Low | Low | Unproven | Low | Low | Low | Medium | Low | Low |
| G. HTTP/provider gateway | Low locally | Request-history only | Envelope-dependent | Low | Egress-only | None | High | Medium | Medium | Medium | Medium | Medium | Transparent | Provider-specific | Low | High | High | Medium | Medium |
| H. Deliberate layered design | High | High | High on primary path | High | High on primary path | High on primary path | High with optional gateway | High | High | Low on primary path | High | High | High but requires guidance | High | Medium | High | High | High | High |
| I. Structured artifact ingestion | Medium–High | Artifact-dependent | Artifact-defined | Process status external | High before return | High for returned result | High | High | Medium–High | None | Medium | Medium | Medium | High | Medium | Medium | Medium | High | Medium–High |

### Candidate I: artifact ingestion

The strongest additional architecture discovered is first-class ingestion of producer-owned artifacts such as:

- Cargo JSON-lines captures;
- ESLint JSON;
- TRX;
- Surefire/Failsafe XML;
- SARIF where a supported producer owns it;
- MSBuild binary logs or structured logger output.

Artifact ingestion avoids owning execution when a trusted external system already ran the operation. It is valuable for CI results and existing developer workflows. It cannot be the sole primary path because artifacts may not exist after launch failure, crash, cancellation, or early configuration failure.

---

## 17. Falsification Results

### Producer configuration as sufficient

**Rejected.** It cannot give Clean-CTX uniform process status, launch failure, cancellation, redaction timing, or model-delivery authority. Toolchain capabilities are heterogeneous.

**Rejecting fact:** A producer can emit excellent JSON while Claude still receives the raw Bash lifecycle first, and some failures produce no artifact.

### `PostToolUse` as primary

**Rejected.** It is after execution and after original-output telemetry. Failed Bash output is interleaved, can be truncated, and cannot be replaced through the failure hook.

**Rejecting fact:** The host removes information before Clean-CTX can establish failure-stream authority.

### Restricted runner as primary

**Not falsified, conditionally accepted.** It wins on authority and redaction timing.

**Future rejecting fact:** If safe direct executable resolution, bounded output handling, or cross-platform descendant cleanup cannot be demonstrated, Clean-CTX should not own execution.

### Many operation-specific MCP tools

**Rejected for now.** They provide typing but create unnecessary catalog and maintenance expansion.

**Rejecting cost:** The same authority is achievable with one closed discriminated tool schema.

### One typed MCP tool as primary interface

**Conditionally accepted.** MCP provides the correct typed return surface.

**Future rejecting fact:** If field experiments show Claude consistently selects Bash despite clear tool descriptions and project guidance, MCP cannot be the exclusive primary UX.

### CLI as the sole interface

**Rejected.** A CLI is portable and testable, but selection depends on agent instruction-following, and invoking it through Bash gives up the earliest Claude boundary.

### Layered architecture as automatically better

**Rejected as a principle.** Layers are retained only when they provide distinct guarantees:

- producer structure reduces acquisition ambiguity;
- the runner owns authoritative capture;
- MCP owns typed agent integration;
- CLI owns portable/manual access;
- the hook owns Bash compatibility;
- the gateway owns optional provider-egress defense.

Duplicate generic filtering at every layer is not recommended.

### HTTP gateway as a complete security boundary

**Rejected.** It can scrub covered provider-bound requests but cannot undo local transcript, telemetry, hook-input, or process exposure.

### Structured diagnostics without raw evidence

**Rejected.** Parser gaps, crashes, stack traces, plugins, and future formats require supporting evidence.

---

## 18. Role of Existing `PostToolUse` Work

The verified `PostToolUse` implementation is not wasted.

Its long-term role should be:

- compatibility fallback for ordinary Bash;
- migration support while agents adopt the typed operation;
- successful-result ANSI normalization;
- successful-result recognized-secret redaction under the existing narrow guarantee;
- deterministic reduction for recognized simple commands;
- an evidence source for producer semantics;
- a measurement point for fallback frequency.

It should not:

- become the canonical operation authority;
- attempt to recover identity from rejected shell compounds;
- claim failed-result protection;
- silently redirect an already executed command;
- block ordinary Bash merely to force adoption;
- be preserved indefinitely only because it already exists.

Removal would be justified only if direct Bash diagnostics become negligible and another host boundary provides equivalent compatibility. Current evidence does not support removal.

---

## 19. Role of Existing Shared Reduction Kernel

The current native mechanics remain valuable:

- shared line and byte accounting;
- deterministic bounded reduction;
- model-visible disclosure;
- repeat protection;
- `FilterFacts`;
- `FilteredText`;
- ANSI normalization;
- recognized-secret redaction;
- producer-specific semantic filters;
- operation-sensitive authority.

### Retain directly

- normalization;
- redaction;
- size and line accounting;
- reduction/disclosure facts;
- bounded evidence rendering;
- repeat-marker protection.

### Reposition

`FilteredText` should become one rendering or compatibility product rather than the canonical diagnostic domain model. A structured producer parser should produce diagnostic facts first; a renderer may then reuse the kernel for bounded textual evidence.

### Do not force reuse

- A Cargo JSON parser should not reconstruct terminal text merely to reuse a line filter.
- TRX and Surefire XML should not be flattened before semantic extraction.
- Structured facts should not be stored as filter markers.
- Provider transport objects should not enter the shared reduction kernel.

The governing rule remains:

> **Share mechanics. Specialize semantics.**

---

## 20. Recommended Architecture

### Complete causal path

```text
Claude or another coding agent
        │
        ├── MCP: run_diagnostic(typed request)       preferred
        ├── CLI: clean-ctx diagnostic ...            secondary
        └── Bash                                     compatibility
                 │
                 └── PostToolUse legacy reducer
        │
        ▼
DiagnosticRequest
  - closed operation
  - canonical workspace root
  - typed producer options
        │
        ▼
ProducerPolicy
  - executable resolution
  - argument construction
  - native structured-output settings
  - expected artifact types
        │
        ▼
Direct Process Supervisor
  - no shell
  - closed stdin
  - separate concurrent stream drains
  - process status and cancellation
  - bounded memory and approved spool policy
        │
        ▼
Producer Parser
  - structured stream or artifact first
  - producer-specific textual fallback
  - coverage and unparsed-evidence accounting
        │
        ▼
DiagnosticResult
  - authoritative outcome
  - diagnostic/test/build facts
  - artifacts
  - bounded sanitized raw evidence
  - parser/reduction/redaction facts
        │
        ▼
MCP structuredContent + concise content
or CLI human/JSON rendering
```

### Ownership boundaries

| Component | Owns | Does not own |
|---|---|---|
| MCP adapter | Schema, tool description, MCP result mapping | Process execution semantics |
| CLI adapter | CLI parsing and rendering selection | Producer parsing policy |
| Diagnostic request | Closed operation identity and typed parameters | Arbitrary command strings |
| Producer policy | Executable resolution, allowed arguments, native format | Cross-producer generic semantics |
| Process supervisor | Spawn, streams, cancellation, exit facts | Diagnostic interpretation |
| Producer parser | Producer-specific semantic facts | Process outcome authority |
| Diagnostic result | Canonical invocation result | Model-specific presentation |
| Renderer | Concise context and disclosure | Canonical evidence mutation |
| `PostToolUse` adapter | Bash compatibility | Primary diagnostic authority |
| HTTP gateway | Covered provider-egress rewriting | Local lifecycle secrecy |

### Minimality

This is intentionally not:

- a generic task runner;
- a build orchestration framework;
- a universal process service;
- a plugin ecosystem;
- a replacement for Cargo, npm, dotnet, or Maven;
- a global CI system;
- a permanent raw-log store.

---

## 21. Migration Strategy

1. **Establish and commit a green baseline.** No diagnostic architecture experiment begins until the current repository state has passed the authoritative verification gate and the exact baseline commit and environment have been recorded.
2. **Freeze one request/result contract.** Limit the first contract to Cargo check.
3. **Implement one direct producer path.** Do not build a generic registry or framework first.
4. **Preserve current hook behavior.** No existing successful Bash behavior needs to change for the experiment.
5. **Establish structured parsing.** Use Cargo JSON as the canonical semantic evidence and bounded non-JSON output as supporting evidence.
6. **Add the CLI adapter.** Support deterministic JSON and human rendering over the same result.
7. **Add one typed MCP tool.** Map its input and output schema directly to the established request/result contract.
8. **Measure selection.** Compare MCP use, CLI-through-Bash use, and direct Bash use in representative Claude tasks.
9. **Add producer families in roadmap order.** Rust, TypeScript/Angular, .NET, then Maven/Spring.
10. **Add artifact ingestion where authoritative.** TRX and Surefire XML are likely early examples.
11. **Reassess hook guidance.** Only add guidance if measured direct-Bash fallback is material and a concise hint changes subsequent behavior.
12. **Leave the HTTP proxy unchanged.** Revisit provider-egress integration separately.

### Mandatory green baseline checkpoint

The baseline is an experimental control, not merely a confidence-building test run. Without it, a later failure cannot be attributed reliably to the diagnostic experiment rather than to the branch's starting state, local configuration, toolchain version, or unrelated worktree changes.

Before Experiment 1 or any implementation experiment:

1. Resolve or explicitly isolate unrelated working-tree changes. Do not silently include them in the baseline.
2. Record the exact Git commit, branch, working-tree status, operating system, Rust toolchain, Clean-CTX feature mode, Claude Code version, and relevant producer versions.
3. Have the repository owner run the complete authoritative Final Verification Gate from `docs/agent/verification.md`:

   ```powershell
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test --workspace --all-targets --all-features
   pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/tests/check-file-sizes.tests.ps1
   pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
   pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
   cargo test --all-features encoding
   ```

4. Record every command and its actual outcome. A failed or unrun command cannot be reported as green.
5. Preserve the existing verified Slice 1 results as historical evidence; do not repeat its deterministic proof unless the current gate fails, relevant dependencies changed, or an experiment specifically depends on an unestablished property.
6. Commit the verified state as the green-baseline architectural checkpoint before adding experiment fixtures or production changes.
7. Attach every later experiment result to that baseline commit and record any intervening changes.

If the gate is not green, experimentation stops. The failure must be classified and resolved or explicitly removed from the experiment's scope before a baseline can be declared.

The baseline record should contain:

```text
BaselineRecord
├─ commit
├─ branch
├─ working_tree_state
├─ operating_system
├─ Rust/Cargo version
├─ Claude Code version
├─ relevant producer versions
├─ Clean-CTX configuration identity
├─ verification commands and outcomes
├─ known pre-existing limitations
└─ timestamp
```

### Behavioral preservation

The migration should follow:

```text
verified green baseline checkpoint
→ existing hook behavior
→ add independent restricted execution path
→ prove typed result authority
→ add MCP/CLI adapters
→ measure adoption
→ narrow overlapping responsibilities only with evidence
```

Do not rewrite current filters while establishing the new boundary unless an existing filter is deliberately reused as a renderer.

---

## 22. First Vertical Slice

### Operation

**Cargo check only.**

Illustrative request:

```text
run_diagnostic {
  operation: "cargo_check",
  workspace_root,
  package?,
  all_features?,
  targets?
}
```

The exact schema requires an implementation plan and architectural approval. The first slice should not accept an arbitrary Cargo argument array.

### Why Cargo check

- It belongs to the highest-priority supported ecosystem.
- Cargo exposes a documented structured message stream.
- Check avoids arbitrary executed-program output.
- It exercises compiler diagnostics, artifacts, build scripts, and non-JSON fallback evidence.
- It can prove the result contract without test-harness ambiguity.

### What the slice must prove

- trusted Cargo resolution without a shell;
- canonical workspace-root validation;
- typed argument construction;
- `--message-format=json` acquisition;
- concurrent stdout/stderr draining;
- authoritative exit status;
- compiler diagnostic parsing;
- build-finished parsing;
- bounded non-JSON record handling;
- recognized-secret redaction before MCP response;
- cancellation and descendant cleanup;
- structured MCP result plus concise textual rendering;
- CLI rendering over the same result;
- parity between producer outcome and returned outcome;
- explicit parser-coverage and omitted-evidence facts.

### Explicit exclusions

- Cargo test;
- generic Cargo subcommands;
- arbitrary feature strings without validation;
- execution-result caching;
- long-term raw-log persistence;
- repository configuration edits;
- global timeout or byte-limit decisions without approval.

---

## 23. Required Experiments

### Experiment prerequisite — frozen green control

Every experiment below is conditional on the mandatory green baseline in Section 21. The baseline commit is the control for attribution and comparison. Experimental fixtures, configuration, logs, and measurements must be kept distinguishable from baseline artifacts.

An experiment is invalid for architectural comparison when:

- it began before the baseline gate completed;
- its starting commit or working-tree state is unknown;
- it includes unrelated uncommitted changes that can affect the result;
- producer or Claude versions changed without being recorded;
- it reports a harness result as a repository test result; or
- it compares against recollection rather than recorded baseline evidence.

After an experiment, restore or switch back to the baseline checkpoint before beginning an independent experiment unless the experiment plan explicitly defines a cumulative sequence. This prevents one experiment's state, fixtures, configuration, or generated artifacts from contaminating another.

### Experiment 1 — MCP selection reliability

**Question:** Does Claude reliably choose the typed diagnostic tool over Bash?

Compare:

- MCP tool available with concise server instructions;
- MCP tool plus project guidance;
- CLI guidance only;
- current Bash-only baseline.

Record tool selected, retries, user intervention, and context cost. Do not infer reliability from a single successful demonstration.

### Experiment 2 — Hook input completeness

**Question:** Does the successful hook receive complete Bash output at relevant sizes?

Generate deterministic output with distinct head, middle, and tail markers. Compare producer bytes, Bash result, hook input, persisted-output metadata, transcript representation, and model-visible replacement.

### Experiment 3 — Cargo mixed output

**Question:** Can the parser preserve non-JSON evidence without corrupting JSON message authority?

Use a bounded fixture containing compiler messages, a build script, and deterministic non-JSON lines. Confirm parsed/unparsed accounting.

### Experiment 4 — Memory and spool behavior

**Question:** Can both streams be drained without deadlock or unbounded memory?

Measure peak memory and output accounting under large concurrent stdout/stderr. This experiment informs, but does not itself authorize, global resource limits.

### Experiment 5 — Cross-platform cancellation

**Question:** Does cancellation terminate descendants on Windows and Unix?

Use a disposable bounded fixture that creates a child process and reports lifecycle markers. Verify graceful and forceful cleanup separately.

### Experiment 6 — Executable resolution attacks

**Question:** Can a malicious repository-local shim replace the intended producer?

Test PATH manipulation, local `.cmd`/`.exe` shims on Windows, symlinked binaries on Unix, and configured absolute paths.

### Experiment 7 — Parser fallback truthfulness

**Question:** What happens when structured data is malformed or truncated?

Inject malformed records and verify that the authoritative process outcome remains, parsing becomes incomplete, and bounded sanitized evidence survives.

### Experiment 8 — Failure secrecy

**Question:** Does raw producer output remain outside Claude-owned result state on the MCP path?

Use synthetic recognizable secrets and inspect only authorized local lifecycle surfaces: MCP response, Claude transcript, debug logging, and telemetry configuration. Do not make claims about surfaces not inspected.

### Experiment 9 — Artifact completeness

**Question:** Which structured artifacts survive abnormal termination?

Evaluate TRX, Surefire/Failsafe XML, and candidate MSBuild evidence after normal failure, test-host crash, cancellation, and launch failure.

### Experiment 10 — Context economics

**Question:** Does semantic compilation materially outperform current filtering?

Compare raw output, current filtered text, and structured rendering using bytes, estimated tokens, actual tokenizer counts where available, and whether the next coding decision retains required evidence.

---

## 24. Explicitly Rejected Assumptions

This investigation rejects the following assumptions:

1. `PostToolUse` is the earliest useful secrecy boundary.
2. Successful hook access implies failed-result equivalence.
3. Bash preserves authoritative producer identity.
4. Package-script names identify their underlying producer.
5. Quiet output is always preferable to structured output.
6. Structured diagnostics eliminate the need for raw evidence.
7. MCP itself solves process ownership.
8. One MCP tool per operation is required for typing.
9. A CLI wrapper alone will reliably change agent behavior.
10. Provider-egress scrubbing protects local transcripts or telemetry.
11. Complete acquisition requires keeping all output in memory.
12. Existing text filters should define the future diagnostic domain model.
13. Every producer must fit one universal diagnostic record.
14. Existing hook investment requires the hook to remain primary.
15. A layered design is correct merely because it is comprehensive.
16. Build success text is stronger than process status.
17. A result artifact is guaranteed to exist after every failed run.
18. Removing a persisted-output pointer removes the persisted raw output.
19. stdout and stderr can be recombined without losing authority.
20. Test execution is just another compiler diagnostic operation.

---

## 25. Stop Conditions

The recommended architecture must be reconsidered if any of the following occurs:

- Direct execution cannot be secured without accepting arbitrary commands or arguments.
- Trusted executable resolution cannot be made portable and understandable.
- Cross-platform cancellation cannot reliably terminate descendant processes.
- Required resource bounds necessarily hide authoritative failure evidence without truthful fallback.
- A producer's structured mode materially changes build or test semantics.
- The first vertical slice shows no material improvement in context quality or token use.
- MCP selection remains poor after clear tool descriptions and project guidance.
- Claude or another primary host introduces a native pre-persistence execution middleware with authoritative separate streams and status.
- A standard agent execution protocol supplies typed process results without requiring Clean-CTX to own subprocesses.
- Supporting a second producer demonstrates that the proposed result model cannot represent both without invented semantics.
- An implementation needs a generic plugin framework before two concrete producer implementations establish the shared boundary.
- The runner would require global timeout, caching, persistence, truncation, or retry policy without explicit architectural approval.
- Secret-safe temporary evidence cannot be created and cleaned reliably.
- Artifact ingestion proves sufficient for the actual workflows and makes in-process execution unnecessary.

---

## 26. Decision Record

### Recommended

- Treat diagnostic output as semantic source material.
- Establish a restricted typed execution core.
- Prefer producer-native structured evidence.
- Expose one closed typed MCP diagnostic tool.
- Provide a CLI adapter over the same core.
- Retain `PostToolUse` as Bash compatibility.
- Preserve bounded sanitized raw evidence.
- Add artifact ingestion where a producer already owns a reliable format.

### Not recommended

- Generic shell execution.
- Many MCP tools at the initial scale.
- CLI-only adoption.
- Hook-only authority.
- Proxy-only protection.
- Text filtering as the future canonical abstraction.
- Universal diagnostic schemas with fabricated commonality.

### Not yet decided

- Global capture and spool bounds.
- Default timeout behavior.
- Long-term evidence persistence.
- Exact executable resolution per ecosystem.
- Whether the MCP tool should be advertised eagerly or deferred in every client.
- Whether hook guidance changes agent behavior enough to justify its complexity.
- Whether compiler APIs are preferable to CLI parsing for TypeScript.
- Whether MSBuild binlogs are worth owning compared with a narrower logger.

---

## 27. Sources and Repository Evidence

### Repository authority

- `src/claude_native/bash.rs` — supported Bash success recognition and reconstruction validation.
- `src/claude_native/command.rs` — conservative operation classification and shell-compound rejection.
- `src/claude_native/pipeline.rs` — normalize → redact → filter orchestration with separate streams.
- `src/claude_native/hook.rs` — hook input size boundary and JSON adapter behavior.
- `src/claude_native/mod.rs` — persisted-output limitation.
- `src/native_text/filter_facts.rs` — existing reduction-fact representation.
- `docs/architecture/CLAUDE_NATIVE_TOOL_INTEGRATION_RESPONSIBILITY_FREEZE_2026-10-05.md` — approved native responsibility boundary.
- `docs/architecture/CLAUDE_NATIVE_FIRST_VERTICAL_SLICE_FREEZE_2026-10-05.md` — existing first hook slice.
- `docs/architecture/CLAUDE_NATIVE_SUPPORTED_TOOLCHAINS_FILTERING_FREEZE_2026-10-06.md` — supported ecosystem order and operation-sensitive field authority.

### Authoritative external documentation

- [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) — hook lifecycle, `PostToolUse`, `updatedToolOutput`, original-output telemetry, failure-hook input, truncation, and failure replacement limitations.
- [Claude Code MCP documentation](https://code.claude.com/docs/en/mcp) — MCP configuration, resources, and deferred tool search.
- [Model Context Protocol tool specification](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) — tool schemas, structured content, output schemas, result content, and resource links.
- [Cargo external-tools documentation](https://doc.rust-lang.org/cargo/reference/external-tools.html) — JSON message stream, compiler messages, artifacts, build completion, and non-JSON limitations.
- [rustc JSON output documentation](https://doc.rust-lang.org/rustc/json.html) — structured compiler diagnostic representation.
- [TypeScript compiler options](https://www.typescriptlang.org/docs/handbook/compiler-options.html) — `tsc` CLI behavior and deterministic non-pretty output controls.
- [ESLint formatters](https://eslint.org/docs/latest/use/formatters/) — machine-readable lint result formats.
- [.NET `dotnet build` documentation](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-build) — verbosity, terminal logger, binary logger, and artifacts.
- [.NET `dotnet test` documentation](https://learn.microsoft.com/en-us/dotnet/core/tools/dotnet-test-vstest) — console and TRX loggers and test result behavior.
- [Maven Surefire Plugin](https://maven.apache.org/components/surefire/maven-surefire-plugin/) — generated text and XML test reports.
- [Maven Surefire test goal](https://maven.apache.org/surefire/maven-surefire-plugin/test-mojo) — report, captured-output, and reporter configuration.

---

## 28. Final Answer to the Governing Questions

### If no existing hook, proxy, or filters existed, what boundary would we choose?

We would choose a closed typed diagnostic operation that directly invokes a known producer, requests its strongest stable structured representation, owns process status and separate streams, redacts before returning anything to the agent host, compiles evidence into diagnostic facts, and exposes those facts through a typed agent tool with bounded raw support.

We would still provide a CLI because execution capability and agent integration surface are separate concerns.

We would not choose post-hoc interception of arbitrary shell output as the primary authority boundary.

### Can Clean-CTX compile diagnostic output into context as it compiles source code?

Yes. That is the recommended long-term model.

The analogy is precise:

```text
source text                producer diagnostic evidence
    ↓                                  ↓
language parser             producer-specific parser
    ↓                                  ↓
typed semantic facts        typed diagnostic facts
    ↓                                  ↓
task-specific context       task-specific diagnostic context
```

The analogy does not mean one universal diagnostic grammar. It means canonical facts remain distinct from model-facing presentation, producer semantics remain specialized, and lossy rendering never becomes durable authority.

### Final architectural position

`PostToolUse` proved that Clean-CTX diagnostic compression is valuable. It did not prove that a successful-result hook is the correct primary boundary.

The next investment should validate a restricted semantic diagnostic path, beginning with Cargo check, before expanding additional text filters. Existing hook work remains useful as compatibility and migration infrastructure while that stronger boundary is proven.
