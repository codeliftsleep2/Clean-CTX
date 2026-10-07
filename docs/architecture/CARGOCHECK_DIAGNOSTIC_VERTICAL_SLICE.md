# CargoCheck Diagnostic Vertical-Slice Architecture Freeze

**Date:** 2026-10-07  
**Architect:** Agent MaxHeadRoom  
**Status:** Frozen for a narrow first production slice; all operational policies approved on 2026-10-07.  
**Operation:** `CargoCheck` only.  
**Production implementation:** Phase 1 pure semantic-compilation boundary implemented and owner-verified on 2026-10-07. Phase 2 authority admission is implemented with owner-run verification pending; process execution and adapters are not implemented.
**Phase 1 verification:** Owner reported the focused CargoCheck parser suite GREEN on 2026-10-07.
**Live-Claude work:** Deferred to one final bundled pilot.

## 1. Executive Verdict

**CARGOCHECK VERTICAL SLICE READY — NO REMAINING POLICY BLOCKER**

The prior falsification chain supports a narrow production slice that owns one operation: run Cargo's `check` operation in one previously approved workspace, capture both producer streams under an OS-owned lifetime boundary, compile recognized Cargo JSON messages into a Cargo-specific semantic result, sanitize every producer-derived value before exposure, and return the same bounded core result through CLI and MCP adapters.

The freeze does not authorize a generic command runner, a generic diagnostic framework, model-selected executables or arguments, inherited `PATH` resolution, or a universal cross-producer result schema. It also does not claim that live Claude behavior has been verified.

The owner approved the complete operational envelope recorded in `CARGOCHECK_OPERATIONAL_POLICY_CALIBRATION_2026-10-07.md`. Architecture and operational policy are ready for a separately authorized Phase 1 implementation task.

The governing rules are:

> **Resolve by identity. Render by position.**

> **Operation identity precedes execution. Share mechanics; specialize semantics. Shared code never infers authority.**

> **Failure to sanitize is not authority to expose evidence. Failure to understand evidence is not authority to discard it.**

> **Process outcome and semantic interpretation are independent authorities. Root exit is not process-tree completion. Cross-platform truth does not require symmetry.**

## 2. Authoritative Evidence Baseline

This freeze is derived from the following repository evidence:

- `NATIVE_DIAGNOSTIC_CONTEXT_ARCHITECTURE_INVESTIGATION_2026-10-07.md` established the candidate architecture and required falsification gates.
- `DIAGNOSTIC_ARCHITECTURE_DECISION_LEVERAGE_FALSIFICATION_2026-10-07.md` ranked the decision leverage and recorded the green experimental baseline supplied by the owner.
- `DIAGNOSTIC_PROCESS_LIFETIME_FALSIFICATION_2026-10-07.md` found that Windows descendant ownership survived and that Linux ownership survived subject to explicit no-detachment and live-controller limitations.
- `DIRECT_MCP_SECRET_BOUNDARY_FALSIFICATION_2026-10-07.md` observed that a closed stdio MCP server could sanitize before serialization and fail closed, while correctly leaving actual Claude-host surfaces unresolved.
- The bounded semantic-compilation, executable-resolution, and parser-truthfulness work recorded by the investigation chain survived within their stated scopes.

The evidence classifications remain narrow:

| Claim | Classification |
|---|---|
| Cargo semantic compilation can improve bounded diagnostic evidence | OBSERVED in the prior bounded falsifier |
| Absolute producer resolution can exclude repository substitution | OBSERVED within the tested resolution constraints |
| Mixed and malformed records can be represented truthfully | OBSERVED in the bounded parser falsifier |
| Windows Job ownership can own tested descendants | OBSERVED |
| Linux process-group ownership can own non-detaching descendants while the controller lives | OBSERVED |
| Sanitized MCP serialization can exclude captured synthetic markers | OBSERVED |
| Actual Claude selection, rendering, persistence, and model visibility | UNRESOLVED |
| Production Cargo behavior across real repositories and toolchain versions | UNRESOLVED until implementation verification |

Cargo's documented machine-message boundary is one JSON object per line on stdout when `--message-format=json` is selected. Cargo also documents that other tools may emit non-JSON output, so non-JSON and malformed records are first-class evidence rather than impossible states. See [Cargo: External Tools](https://doc.rust-lang.org/cargo/reference/external-tools.html).

## 3. Scope and Non-Goals

The frozen scope is exactly one local operation:

```text
approved workspace + approved absolute Cargo executable
→ cargo check --message-format=json
→ separately captured stdout/stderr
→ Cargo-specific parsing and sanitized semantic compilation
→ bounded CargoCheck result
→ CLI and MCP presentation adapters
```

Non-goals for this slice are:

- arbitrary commands, shells, scripts, executable names, or executable paths supplied by a tool caller;
- `cargo build`, `cargo clippy`, `cargo test`, or other producers;
- caller-selected packages, workspace mode, targets, target triples, features, profiles, jobs, message formats, verbosity, color, or passthrough arguments;
- remote workspaces, containers, sandboxes, or remote MCP transport;
- detached descendants or processes that intentionally escape the owned group;
- a universal `DiagnosticResult`, producer registry, runner trait, operation registry, or plugin framework;
- caching, replay, persistent raw evidence, artifact harvesting, or cross-session result storage;
- a guarantee about Claude/provider telemetry, crash dumps, OS inspection, hostile producers, or memory zeroization;
- replacement or behavioral modification of the existing `PostToolUse` compatibility path.

## 4. CargoCheck Operation Identity

`CargoCheck` is a closed domain operation known before any process is resolved or started. It is not inferred from a command string.

The operation identity fixes:

- producer family: Cargo;
- Cargo subcommand: `check`;
- message format: line-delimited Cargo JSON;
- working directory: one approved Cargo workspace/package root;
- permitted executable: one operator-approved absolute Cargo path;
- public operation name: `cargo_check` for MCP and `cargo-check` for CLI;
- result type: `CargoCheckResult`.

The existing late command classifier in `src/claude_native/command.rs` remains compatibility-path logic. It must not confer execution authority and must not construct this request.

## 5. Request Contract

The domain request is deliberately smaller than either adapter:

```text
CargoCheckRequest {
    workspace: ApprovedWorkspaceRoot
}
```

`ApprovedWorkspaceRoot` is an opaque validated value, not a string alias. Only the workspace-admission boundary may construct it.

There are no feature or target arguments in the first contract. In particular, the request cannot select:

- `--workspace`, `--package`, `--exclude`, or a manifest path;
- `--features`, `--all-features`, or `--no-default-features`;
- `--lib`, `--bin`, `--example`, `--test`, `--bench`, or `--all-targets`;
- `--target`, a profile, job count, or unstable flag;
- environment overrides or arbitrary Cargo configuration.

The resulting semantics are Cargo's default `check` selection for the approved root. The result must identify those semantics and must not claim complete workspace, all-target, or all-feature coverage.

## 6. Workspace Authority

Workspace selection is an authority decision made before tool invocation, not an MCP model argument.

Admission must:

1. receive an operator/host-selected absolute candidate;
2. canonicalize it;
3. require an existing directory;
4. require `Cargo.toml` at that directory for the first slice;
5. reject a root outside the host's explicit approval source;
6. retain the canonical root and approval source as immutable request authority;
7. revalidate the directory and manifest before spawn.

The first MCP schema accepts an empty object. It uses the single root admitted into the MCP session at startup. The model cannot choose or alter that root.

The CLI accepts an explicit `--workspace-root <absolute-path>` from the local operator and passes it through the same admission constructor. That is local operator authority, not producer input.

The current `find_project_root()` heuristics and `resolve_file_path_checked()` mechanics may inform implementation, but neither is sufficient authority as written. Walking from an executable directory, falling back to the current directory, accepting additional roots, or trusting a caller-supplied root must not silently authorize execution.

Nested-package selection, multiple approved roots, and inferred roots are deferred.

## 7. Trusted Cargo Resolution

The Cargo executable is resolved independently of the repository and independently of the request.

The first slice requires an operator-configured absolute path. Admission must:

1. reject relative paths and bare `cargo` names;
2. canonicalize the path;
3. require a regular executable appropriate to the platform;
4. record the canonical path and approval source;
5. capture stable file identity/metadata where the platform exposes it;
6. reopen or revalidate immediately before process creation and fail on identity change where enforceable;
7. pass the absolute path directly to the OS process API without a shell.

A rustup Cargo proxy may be approved. Rustup documents that the executables in its proxy directory select toolchains according to overrides and workspace files; therefore the proxy path is the trusted executable identity while workspace-local toolchain selection remains explicit producer configuration. See [rustup proxies](https://rust-lang.github.io/rustup/concepts/proxies.html) and [rustup overrides](https://rust-lang.github.io/rustup/overrides.html).

The result records at least:

- canonical Cargo executable path;
- approval source;
- observed executable identity metadata supported by the platform;
- whether the path is known/configured as a rustup proxy;
- workspace toolchain-selector files observed at admission/execution time, without claiming they fully describe rustup's decision.

Inherited `PATH` search, repository-local shims, shell aliases, `PATHEXT` search, and request-supplied executable paths are prohibited. Automatic toolchain installation/network behavior is not silently authorized; its exact environment policy is deferred in §30.

## 8. Invocation Contract

The semantic invocation is fixed:

```text
<approved-absolute-cargo> check --message-format=json
```

Process construction must use an argument-vector API, never command-string concatenation or a shell. The current directory is the canonical approved workspace root. Standard input is closed unless a platform API requires an inert handle.

No caller-controlled value may enter the argument vector. No wrapper executable, response file, environment-provided subcommand, or Cargo alias is used.

The environment begins from a deliberately selected production policy, not an accidental full inheritance contract. Variables required for normal Cargo/rustup/toolchain behavior may be inherited only under the approved policy; variables that could change executable discovery or inject wrappers must be addressed explicitly before implementation. The exact allow/inherit policy is deferred rather than invented here.

## 9. Process Ownership

### Windows

The controller creates Cargo suspended with inherited handles restricted to the intended stdio handles, creates a private Job Object, configures kill-on-job-close and no breakaway semantics, assigns the suspended process to the Job, and resumes it only after successful assignment.

If Job creation, configuration, or assignment fails, Cargo must never be resumed. Cancellation and forced cleanup target the Job. Completion requires the root process outcome plus evidence that the Job contains no active processes. Handles remain under one owner and are closed deterministically.

### Linux

The child establishes a new session or dedicated process group before `exec`. The controller records the group identity and directs cancellation/cleanup signals to the group, not just the root PID.

Completion requires the root outcome and observation that no member remains in the owned group. Explicit daemonization, double-fork detachment, or reassignment to another session is outside the guarantee and must be stated as such.

### Cross-Platform Contract

Both implementations must satisfy:

```text
ownership established before producer work is allowed
root outcome observed independently of parser outcome
cancellation targets the owned descendant boundary
normal completion includes descendant quiescence
cleanup uncertainty is represented, never converted to success
```

### Explicit Asymmetries

Windows Job membership is a kernel-owned containment object and can kill members when its controlling handle closes. Linux process groups do not provide an equivalent controller-death guarantee, and a descendant can deliberately detach. The public result must not flatten these differences into a false universal guarantee.

No macOS production claim is made by this freeze. A Unix implementation may later be evaluated there, but Linux evidence does not establish macOS behavior.

## 10. Stream Capture and Authority

stdout and stderr are captured concurrently and remain distinct through the core result.

- stdout is the documented Cargo JSON-message channel, but it may contain non-JSON evidence.
- stderr is unstructured producer/toolchain evidence and may include progress, configuration failures, toolchain errors, or third-party output.
- neither stream is authoritative for process termination;
- neither stream is merged into the other;
- no raw line is logged, traced, echoed, or placed in an adapter result;
- stream order is preserved within each stream; no false total ordering across separate pipes is asserted.

The OS process outcome is authoritative for launch and exit. Cargo `build-finished` is producer evidence only.

## 11. Cargo Structured Evidence

The parser recognizes the Cargo message reasons documented for external tools:

- `compiler-message`;
- `compiler-artifact`;
- `build-script-executed`;
- `build-finished`.

Only `compiler-message` produces semantic diagnostics. The others contribute bounded producer facts, coverage, and completion corroboration. They never override the observed process outcome.

A line is considered a JSON candidate only after bounded framing and when its first non-whitespace byte is `{`. Successful JSON parsing does not imply a known Cargo record. Unknown reasons and structurally incompatible known reasons remain categorized evidence.

The parser must not deserialize arbitrary records into logs for debugging. Parsed raw values remain inside the capture/transformation boundary until all retained strings have been sanitized or discarded.

## 12. Normalization and Redaction Boundary

The required order is:

```text
bounded byte capture
→ bounded line framing and explicit decoding facts
→ structural JSON attempt for candidate stdout lines
→ extraction of recognized fields
→ normalization and redaction of every retained producer-derived string
→ semantic compilation from sanitized values
→ bounded evidence selection
→ adapter rendering
```

Structural JSON parsing occurs before text redaction because editing serialized JSON text can corrupt escaping and structure. This does not authorize raw parsed strings to leave the boundary.

Every retained string is sanitized independently, including messages, codes, filenames, span text, labels, suggestions, rendered diagnostics, unknown-record summaries, stdout/stderr evidence, error descriptions derived from producer data, and any metadata exposed to adapters.

The existing ANSI-normalization and recognized-secret redaction mechanics may be extracted and reused. A redaction failure is an internal processing failure: all producer-derived evidence is withheld. There is no raw fallback.

## 13. Parser Coverage and Truthfulness

`ParserCoverage` reports what was seen and what was understood. At minimum it distinguishes:

- total stdout frames;
- recognized valid Cargo records by reason;
- valid JSON objects with unknown reason;
- valid JSON that is incompatible with the expected record shape;
- malformed JSON candidates;
- ordinary non-JSON stdout frames;
- truncated or over-limit frames;
- stderr frames;
- decoding replacements or invalid-byte events;
- records/evidence omitted by the bounded policy.

Coverage is not a percentage unless its denominator and interpretation are unambiguous. Counts and categories are preferable.

Malformed, truncated, unknown, and non-JSON evidence must reduce the reported confidence/coverage and contribute sanitized bounded evidence. They must never disappear silently or be treated as diagnostic absence.

`build-finished.success`, if present, is recorded as Cargo's assertion. A disagreement with the actual process outcome becomes an explicit mismatch fact.

## 14. CargoCheck Semantic Result

The first semantic model is Cargo/rustc-specific:

```text
CargoDiagnostic {
    level
    message
    code?
    primary_spans[]
    related_spans[]
    children[]
    rendered_evidence?
}
```

A span retains sanitized file name, line/column range, primary flag, label, and—when present—sanitized suggested replacement and suggestion applicability. Byte offsets and source-text fragments are retained only if the approved bounded policy permits them and they materially support the diagnostic decision.

Child diagnostics retain level, message, relevant spans, and sanitized rendering where bounded. The parser does not manufacture a primary location when rustc supplied none.

Diagnostic identity is semantic, not rendered-text identity. Sanitized `rendered` output is optional bounded evidence useful for details the initial typed model does not yet carry; it is not the authority for severity, code, or location.

The model is not named `DiagnosticResult` and is not generalized for .NET, Maven, or other producers.

## 15. Raw Evidence Policy

No unsanitized raw evidence crosses the core boundary. “Raw evidence” in the public result means minimally interpreted, sanitized producer evidence.

Retention is category-aware and stream-aware:

1. retain evidence that explains launch, parser, completion, or authority mismatches;
2. retain malformed, truncated, unknown, and non-JSON categories so coverage claims are auditable;
3. retain useful stderr tail evidence because terminal errors often occur late;
4. preserve bounded head/tail context within a category where it improves interpretation;
5. never use a first-N-only policy that can systematically hide terminal causes;
6. preserve per-stream order and label the source;
7. disclose original, retained, and omitted counts/bytes and the selection reason.

Recognized semantic records need not be duplicated wholesale as evidence. Evidence is a truthful fallback and coverage aid, not a second unlimited transcript.

Evidence is memory/session data for the returned result. Persistent raw-output files and evidence caching are outside this slice.

## 16. Process Outcome and Failure Semantics

The result uses an explicit outcome rather than a success boolean:

```text
NotStarted { reason }
LaunchFailed { sanitized_reason }
Exited { code }
Cancelled { cleanup }
Terminated { platform_reason, cleanup }
InternalProcessingFailed { stage, raw_evidence_withheld, cleanup }
```

`cleanup` separately reports `Complete`, `Incomplete`, or `Uncertain` with platform facts. An exit code of zero does not erase parser incompleteness, redaction failure, completion mismatch, or incomplete descendant cleanup.

The architecture does not invent Unix signals on Windows or reduce all termination to an exit code. A future timeout is a distinct cause, not a synonym for cancellation, and is deferred until a timeout policy is approved.

Adapter/protocol failures remain distinct from producer failures. MCP JSON-RPC errors are for invalid protocol/method/arguments; a started Cargo operation returns a typed tool result even when Cargo fails.

## 17. Cancellation and Completion

Cancellation authority belongs to the core operation controller, not to the parser or adapter.

For the initial slice, cancellation must reliably stop the owned boundary. No claim of graceful Cargo shutdown is frozen. An optional graceful phase requires an approved signal and duration policy; absent that approval, implementation uses the platform's bounded forced-termination mechanism and reports that fact.

Completion means all of the following have settled:

- process creation either failed or the root process terminated;
- stdout and stderr pipes reached EOF or an explicit capture failure was recorded;
- parser work completed over every accepted frame;
- the owned descendant boundary is quiescent or cleanup uncertainty is recorded;
- the result was sanitized and bounded;
- no raw producer evidence is included in errors, traces, or adapters.

The controller must continue draining both pipes while awaiting the process so a full pipe cannot deadlock completion.

## 18. Bounded Capture Requirements

Boundedness is mandatory at every accumulation point, not only final rendering. The approved policy must contain independent limits for:

- bytes read per stream and in aggregate;
- maximum frame length;
- retained diagnostic count;
- retained spans/children/suggestions per diagnostic;
- retained sanitized evidence bytes/lines per stream and category;
- retained unknown structured summaries;
- parser nesting/record complexity where the JSON library exposes a practical control;
- final structured result size and text rendering size;
- optional internal spool size, if spooling is approved later.

The implementation must stream and discard or summarize beyond the limits. It may not first buffer unlimited output and truncate afterward.

Limit activation is a result fact. Exceeding a retention limit does not change the authoritative process outcome, but it reduces completeness and is disclosed. Conditions that make safe framing, parsing, or redaction impossible fail closed.

Numeric values are intentionally not frozen here; see §30.

## 19. Core Result Contract

`CargoCheckResult` separates authority, semantics, transformation, and evidence:

```text
CargoCheckResult
├─ operation: CargoCheck
├─ execution
│  ├─ approved_workspace
│  ├─ producer_identity
│  ├─ invocation_identity
│  ├─ process_outcome
│  └─ ownership_and_cleanup
├─ cargo_evidence
│  ├─ build_finished?
│  ├─ artifacts_seen
│  ├─ build_scripts_seen
│  └─ outcome_mismatch?
├─ diagnostics[]
├─ parser_coverage
├─ transformations
│  ├─ decoding
│  ├─ normalization
│  ├─ redaction
│  └─ reduction
└─ sanitized_evidence
   ├─ stdout[]
   └─ stderr[]
```

Paths exposed in the result follow an approved representation policy. The workspace root and Cargo executable identity are authority facts, but adapters may render a stable display form rather than leaking unrelated machine path details. No field may contain an unsanitized producer-derived string.

The core result contains no MCP `content`, CLI formatting, JSON-RPC error, color, terminal width, or Claude-specific field.

## 20. MCP Adapter

The first MCP tool is a narrow static tool named `cargo_check`.

Its input schema is an object with no properties and `additionalProperties: false`. It cannot accept a workspace, command, executable, environment, or argument list. The handler obtains the session's admitted workspace and configured Cargo producer, invokes the core operation, and performs no second execution path.

The result uses:

- `structuredContent`: the complete bounded `CargoCheckResult` projection;
- `content`: a concise rendering derived only from that sanitized structured result;
- `isError`: true for launch, Cargo nonzero, cancellation, incomplete cleanup, or internal processing failure as defined by adapter semantics.

Both surfaces must be checked for marker leakage in tests. Protocol errors must never embed producer evidence. Logging and tracing may contain operation IDs and boolean/count facts, but not raw arguments, environment, stdout, stderr, parsed strings, or exact synthetic markers.

The MCP adapter is the intended primary agent interface, but that UX claim remains subject to the final Claude selection pilot.

## 21. CLI Adapter

The first CLI form is:

```text
clean-ctx cargo-check --workspace-root <absolute-path>
```

The Cargo executable comes from the same operator configuration used by MCP, not a CLI passthrough argument. No `--` passthrough is allowed.

The CLI is included in the first implementation sequence as the deterministic local adapter used before any Claude environment switch. It invokes the same core and renders only the sanitized result. A machine-readable rendering may be added only if it is a direct serialization of the same versioned result projection; it must not expose internal raw fields.

CLI exit behavior must distinguish producer failure from Clean-CTX processing/authority failure. Exact Clean-CTX exit-code numbers are a public contract and require separate approval before implementation.

Using this CLI through an ordinary host Bash tool does not acquire the direct-MCP pre-host secrecy advantage. That limitation must remain documented.

## 22. PostToolUse Compatibility Role

The existing `PostToolUse` path remains a compatibility adapter for already executed Bash results. Its command classification, Cargo stderr filtering, repeated-output handling, and replacement behavior are not reused as execution authority.

This slice does not remove, redirect, or silently change it. Its limitations remain:

- the host executes first;
- raw output may enter host-owned state before the hook;
- supported successful results can be reduced later;
- failed producer results may not be replaceable through the current hook contract.

Direct MCP and CLI results must not flow back through the PostToolUse parser.

## 23. Shared Mechanics Reuse Matrix

| Existing area | Decision | Boundary |
|---|---|---|
| ANSI/control normalization | **REUSE MECHANIC ONLY** | Apply to retained text fields/evidence; do not import compatibility pipeline authority |
| Recognized-secret redactor | **REUSE AFTER EXTRACTION** | Reuse compiled-pattern mechanics and facts; sanitize every retained field; fail closed |
| `FilterFacts` | **REUSE MECHANIC ONLY** | Its accounting ideas inform new reduction facts; it is not the CargoCheck contract |
| `FilteredText` | **DO NOT REUSE AS CORE RESULT** | Text replacement is not semantic diagnostics |
| Head/tail line retention | **REUSE MECHANIC ONLY** | Extend to category- and stream-aware bounded evidence |
| Byte/line accounting | **REUSE MECHANIC ONLY** | Preserve exact disclosure in CargoCheck-specific facts |
| Repeat protection | **DO NOT REUSE** | It solves a PostToolUse presentation concern, not direct execution |
| String command classification | **DO NOT REUSE FOR AUTHORITY** | Operation identity is typed before execution |
| `resolve_file_path_checked` | **REUSE MECHANIC ONLY** | Canonicalization ideas are useful; caller-supplied roots are insufficient authority |
| MCP static schema/registry/dispatch | **REUSE AS-IS WHERE POSSIBLE** | Add one narrow schema and handler without a generic tool framework |
| MCP error/result helpers | **REUSE MECHANIC ONLY** | Ensure producer failures remain typed tool results |
| Session state | **EXTEND NARROWLY** | Store one immutable admitted execution root and producer identity |

Reuse must not couple the new core to Claude hook envelopes or make presentation types authoritative.

## 24. Cargo-Specific Responsibilities

The following remain inside the CargoCheck module:

- the exact Cargo argument vector and default-selection semantics;
- Cargo/rustc JSON record discrimination;
- `compiler-message` field parsing and semantic compilation;
- artifact, build-script, and build-finished evidence interpretation;
- Cargo outcome-mismatch reporting;
- Cargo-specific coverage categories;
- CargoCheck diagnostic and result types;
- any interpretation of rustup/toolchain selector evidence relevant to this operation.

OS ownership mechanics may be locally separated by platform, but the first implementation must not expose a public generic process-runner API or producer trait merely because another producer might later need similar mechanics.

## 25. Threat Model

The slice protects against:

- repository-controlled executable substitution through `PATH`, current-directory, shell, alias, and `PATHEXT` resolution;
- model/caller attempts to choose commands, arguments, environment, or workspace;
- accidental raw stdout/stderr exposure through MCP/CLI results and routine logs;
- nonzero exits or parser failures bypassing sanitation;
- sanitizer failure falling back to raw evidence;
- ordinary non-detaching descendants surviving normal cancellation unnoticed;
- malformed, unknown, or mixed output being silently reported as fully parsed.

It does not protect against:

- a malicious or compromised approved Cargo/toolchain executable;
- compiler plugins, build scripts, proc macros, or other code executed by Cargo acting maliciously;
- privileged OS inspection, memory scraping, crash dumps, endpoint-security capture, or compromised kernels;
- deliberate Linux process detachment;
- inaccessible Claude/client/provider telemetry;
- secrets already exposed before Clean-CTX starts;
- semantic secrets not recognized by the approved redaction rules.

Cargo check can execute build scripts and procedural macros. Therefore this is controlled producer execution, not safe evaluation of an untrusted repository.

## 26. Architecture Invariants

1. `CargoCheck` is known before executable resolution.
2. No tool caller supplies an executable, command string, argument, environment entry, or message format.
3. MCP callers do not select the workspace in the first slice.
4. Only `ApprovedWorkspaceRoot` can cross into process construction.
5. Workspace and Cargo paths are absolute, canonical, validated, and rechecked before spawn.
6. Cargo is launched directly without a shell or inherited search.
7. The argument vector is exactly the frozen CargoCheck invocation.
8. Process ownership is established before producer execution proceeds.
9. stdout and stderr are drained concurrently and never merged.
10. Process outcome remains authoritative independently of producer records.
11. `build-finished` never overrides the OS process outcome.
12. Parsing never requires a line to be valid JSON merely because JSON mode was requested.
13. Unknown, malformed, truncated, and non-JSON records are disclosed.
14. Structural parsing may see raw data only inside the capture boundary.
15. Every exposed producer-derived string is normalized/redacted before result construction.
16. Redaction failure withholds all producer-derived evidence.
17. Nonzero producer exit cannot bypass sanitation.
18. Capture is bounded while reading; post-hoc truncation of an unbounded buffer is forbidden.
19. Every omission or activated limit is represented in result facts.
20. Sanitized evidence remains separated by stream and truthfully ordered only within a stream.
21. Core result types contain no adapter-specific presentation state.
22. MCP `content` and `structuredContent` derive exclusively from the sanitized core result.
23. Routine logs contain facts and counts, never producer text or exact secret markers.
24. CLI and MCP invoke one core implementation.
25. The PostToolUse compatibility path neither authorizes nor wraps this operation.
26. Platform ownership asymmetries remain visible in the result and documentation.
27. Incomplete or uncertain descendant cleanup cannot be reported as clean success.
28. No implementation claim becomes a production-integration claim until the complete lifecycle is traced and verified.

## 27. Repository Change Map

The intended smallest coherent map is:

| Path | Change |
|---|---|
| `src/diagnostics/mod.rs` | New private/public module boundary exporting only CargoCheck contracts needed by adapters |
| `src/diagnostics/cargo_check/mod.rs` | New operation orchestration; keep below active-file ceiling |
| `src/diagnostics/cargo_check/model.rs` | Request, authority, outcome, diagnostic, coverage, and result types |
| `src/diagnostics/cargo_check/parser.rs` | Cargo JSON discrimination and semantic compilation |
| `src/diagnostics/cargo_check/evidence.rs` | Category-aware bounded sanitized evidence assembly |
| `src/diagnostics/cargo_check/windows.rs` | Windows suspended-process/Job ownership implementation under `cfg(windows)` |
| `src/diagnostics/cargo_check/unix.rs` | Linux session/process-group ownership under the appropriate `cfg`; no unsupported platform claim |
| `src/tests/diagnostics/...` | Tracked parser, truthfulness, secrecy, limits, and platform regression tests |
| `src/lib.rs` | Register the diagnostics module |
| `src/main.rs` | Add the narrow CLI adapter; currently below but close enough to the ceiling to require planned decomposition if needed |
| `src/mcp/tools.rs` | Add one static `cargo_check` schema; currently below the ceiling |
| `src/mcp/tool_handlers/diagnostics.rs` | New narrow MCP handler |
| `src/mcp/tool_handlers/registry.rs` | Register the handler |
| `src/mcp/state.rs` | Add immutable approved workspace/producer authority; at 605 lines it must be decomposed before or as part of any material edit so every modified result remains at or below 615 lines, preferably 600 |
| `src/mcp/server.rs` | Construct admitted authority at startup rather than granting it through heuristic discovery |
| `src/native_text/...` | Only narrowly extract/re-export reusable sanitizer mechanics if required; do not route through the PostToolUse pipeline |
| `docs/CONFIGURATION.md` | Document Cargo executable/workspace approval and limitations after implementation |
| `docs/ARCHITECTURAL_INVARIANTS.md` | Record only durable invariants once implementation makes them true |

This map is directional, not authorization for unrelated refactors. If implementation demonstrates that a public API, global lifecycle, or repository-wide policy must change, the Architectural Approval Gate applies.

## 28. Implementation Sequence

1. Obtain owner approval for the concrete policies listed in §30.
2. Add CargoCheck-specific result and authority types with no process execution.
3. Add bounded pure parser/evidence tests and implement semantic compilation, mixed-record truthfulness, and fail-closed sanitized construction.
4. Implement workspace and Cargo executable admission as opaque values with focused tests.
5. Implement Windows ownership and tracked lifecycle tests using the already established process-tree fixture principles.
6. Implement Linux ownership and tracked lifecycle tests under Linux, preserving its weaker controller-death/detachment contract.
7. Compose the exact CargoCheck operation around those fixed contracts.
8. Add the CLI adapter and use it for owner-run local binary verification before Claude work.
9. Decompose MCP state as required by the file-size policy, then add one empty-input MCP tool and handler.
10. Trace startup → admitted authority → handler → core → bounded result → both MCP surfaces.
11. Run the complete owner-operated verification gate.
12. Build one pilot candidate and execute all remaining Claude-only checks in one final environment switch.

At every step, stop on a kill condition rather than broadening the operation.

## 29. Test Strategy

All regression tests belong under `src/tests/**`; investigation scripts and live harnesses do not satisfy the gate.

Required tracked test groups are:

- request closure: arbitrary fields/arguments/executables cannot enter the operation;
- workspace admission: relative, absent, non-directory, missing-manifest, changed, and unauthorized roots fail;
- producer admission: relative/bare/repository-local/substituted/changed Cargo paths fail;
- invocation: exact executable, arguments, cwd, stdio, and environment policy;
- parser: every recognized reason plus unknown, non-JSON, malformed, truncated, incompatible, and mixed streams;
- semantic fidelity: codes, levels, primary/related spans, children, suggestions, and missing locations;
- authority mismatch: exit status versus `build-finished` combinations;
- secrecy: distinct stdout/stderr markers absent from diagnostics, evidence, errors, logs, MCP `content`, and `structuredContent`;
- failure behavior: nonzero exit, launch failure, decode failure, parser failure, redaction failure, and result-serialization failure;
- boundedness: each approved limit independently activated with truthful counts and no unbounded pre-buffer;
- concurrent capture: large interleaved stdout/stderr cannot deadlock and stream identity survives;
- Windows lifecycle: normal completion, cancellation, child/grandchild cleanup, parent exit, failed Job assignment, and incomplete-cleanup reporting;
- Linux lifecycle: normal completion, group cancellation, child/grandchild cleanup, parent exit, and documented detachment/controller limitations;
- adapter equivalence: CLI and MCP authority/semantic facts match for the same core result;
- compatibility: existing PostToolUse behavior remains unchanged.

Real Cargo fixtures should be minimal and deterministic. Long-running Cargo, tests, builds, Clippy, binaries, and servers remain owner-run commands under repository policy. The agent must hand off exact commands and accurately report what the owner ran.

## 30. Approved Operational Policies

The owner approved these values/contracts on 2026-10-07. The authoritative rationale, sensitivity analysis, failure behavior, and disclosure requirements are recorded in `CARGOCHECK_OPERATIONAL_POLICY_CALIBRATION_2026-10-07.md`:

1. stdout 8 MiB, stderr 4 MiB, aggregate 12 MiB, and maximum frame 2 MiB;
2. 64 diagnostics, 4 primary spans, 8 related spans, 8 children, and 4 suggestions;
3. 96 KiB categorized sanitized evidence, 512 KiB structured result, and 24 KiB/240-line text rendering;
4. no temporary spool;
5. immediate forced owned-boundary cancellation with no graceful interval;
6. no intrinsic operation timeout;
7. strict minimal explicit environment, named rustup override allowed, rustup auto-install disabled, ordinary dependency network allowed, and environment credentials stripped;
8. CLI exit codes 0–5 as defined by the calibration;
9. workspace-relative/logical path display with no absolute paths in default MCP output;
10. startup option over environment variable, followed by failure, with no heuristic authority fallback.

Unsupported environment-dependent projects fail truthfully. Operator-controlled environment forwarding is deferred unless real-world evidence establishes a need. No implementation may silently alter these approved policies.

## 31. Final Claude Pilot Requirements

Claude work is the absolute last gate and must be batched into one planned pilot after all safe local work and the owner-run verification gate are complete.

The pilot uses one rebuilt candidate and one preserved context package to evaluate independently:

1. selection of `cargo_check` for neutral, obviously matching requests;
2. fallback behavior when the MCP tool is absent or unavailable;
3. actual rendering/consumption of `content` and `structuredContent`;
4. a synthetic model-echo secret probe after the sanitized result;
5. legitimately inspectable transcript/debug/local persistence surfaces;
6. representative real-workspace diagnostic usefulness and decision quality;
7. nonzero Cargo failure behavior;
8. cancellation behavior observable through the real host;
9. any difference caused by concise project guidance.

Selection, secrecy, rendering, diagnostic quality, and persistence remain separate verdict cells. A success in one cannot mask failure in another. Any reproducible field discovery must be recorded and distilled into a cheap tracked local regression where possible.

## 32. Remaining Risks

- Approved Cargo runs repository code through build scripts and procedural macros; executable trust is not repository safety.
- Rustup/toolchain resolution may change with workspace configuration and environment even when the proxy path is stable.
- TOCTOU can be reduced by identity checks but not universally eliminated by path canonicalization alone.
- Linux descendants can deliberately detach, and controller death lacks the Windows Job close guarantee.
- Unknown Cargo/rustc schema evolution may reduce coverage until parser support is updated.
- Redaction recognizes configured patterns; it cannot prove absence of every semantic secret.
- Bounded evidence can omit a decisive detail; truthful disclosure reduces but does not eliminate this risk.
- Inaccessible Claude/client/provider surfaces remain outside the server-controlled secrecy proof.
- Numeric limits selected without representative measurements may harm either resource safety or diagnostic utility.
- Platform-specific process code is high-risk and requires real OS verification, not only mocks.

## 33. Final Freeze Verdict

**FREEZE THE CARGOCHECK ARCHITECTURE; DO NOT GENERALIZE IT.**

The first production slice is authorized architecturally as a closed CargoCheck operation with:

- one pre-approved workspace root;
- one pre-approved absolute Cargo executable;
- an exact no-shell invocation;
- pre-execution Windows/Linux descendant ownership;
- concurrent separate stream capture;
- Cargo-specific semantic compilation;
- truthful parser coverage;
- fail-closed normalization/redaction;
- bounded sanitized evidence;
- one core result shared by CLI and MCP;
- an unchanged PostToolUse compatibility path;
- one final bundled Claude pilot after local verification.

No architecture or operational-policy blocker remains. Phase 1 production coding requires a separate implementation authorization; readiness is not permission to introduce a generic runner, universal diagnostic schema, arbitrary execution surface, or unmeasured global defaults.

Any discovery that requires arbitrary caller arguments, inherited executable search, unbounded buffering, raw fallback, silent parser loss, unsupported descendant guarantees, or a cross-producer abstraction is a stop condition and returns the design for review.

The design must also return for review if CargoCheck proves not to be meaningfully closed, approved Cargo resolution still permits repository-controlled authority, structured output cannot support truthful diagnostics, sanitation cannot precede every model-facing surface, supported behavior requires intentional detachment, or completion cannot be determined under the frozen platform contracts. Unresolved policy must never masquerade as architectural fact.
