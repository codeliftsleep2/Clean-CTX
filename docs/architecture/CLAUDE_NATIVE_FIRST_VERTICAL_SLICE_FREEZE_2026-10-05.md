# Claude-Native First Vertical Slice Freeze

**Date:** 2026-10-05

**Status:** Verified and closed. The deterministic repository gate passed and
the real Claude Code `PostToolUse` lifecycle was confirmed on Work Compute.

**Scope:** Claude-native `PostToolUse` success handling only. The HTTP proxy is
outside this plan and remains unchanged.

## 1. Executive Verdict

**FIRST CLAUDE-NATIVE VERTICAL SLICE VERIFIED AND CLOSED**

The first slice is one successful Claude `Bash` result with the documented
structured output shape. It normalizes and redacts the `stdout` and `stderr`
text fields independently, applies one native-owned `git diff/show` diagnostic
filter to eligible `stdout`, reconstructs the original object without changing
non-target values, validates it, and returns `updatedToolOutput`.

Every unrecognized, malformed, failed, interrupted, image, opaque, or uncertain
case returns no replacement. No provider-neutral result model is needed, and no
proxy change or dependency is required.

## 2. Selected Tool and Schema

| Candidate | Result shape known | Command/input useful | Filter value | ANSI value | Redaction value | Reconstruction complexity | Recommendation |
|---|---:|---:|---:|---:|---:|---:|---|
| `Bash` | Yes: documented structured success object | Yes: `tool_input.command` | High | High | High | Low for a field-local object update | **SELECT** |
| `Read` | Not established precisely enough for replacement | File path, not a command | Low | Low | Medium | Unjustified until its exact response contract is captured | Defer |
| `Grep` | Not established precisely enough for replacement | Pattern/path are useful but not program identity | Medium | Low | Medium | Unjustified until fixture evidence exists | Defer |
| `Write` | A simple result is documented, but it is not diagnostic output | File path only | None | None | Low | Low | Reject: it does not prove the capabilities |
| MCP tools | Server-defined and heterogeneous | Tool-specific | Variable | Variable | High | High without per-tool schemas | Defer |

`Bash` is selected because Claude supplies the authoritative tool name, command,
success lifecycle, and structured text fields. It proves all three Clean-CTX
capabilities without flattening data or inferring identity from output.

The first filter is native-owned support for successful `git diff` and `git
show` output. It is preferable to a Cargo rule for this slice because it is
deterministic, useful on real repositories, writes its primary diagnostic text
to `stdout`, and supports a live proof that does not invoke Cargo.

## 3. Native Event Contract

The adapter accepts a Claude hook JSON document only when all of these
preconditions hold:

- the lifecycle is `PostToolUse`, not `PostToolUseFailure`;
- `tool_name` is exactly `Bash`;
- `tool_input` is an object containing `command: string`;
- `tool_response` is an object containing `stdout: string`, `stderr: string`,
  `interrupted: false`, and `isImage: false`;
- the response can be reconstructed without discarding any member; and
- the reconstructed object passes the same first-slice schema validation.

The contract is deliberately narrower than every possible Claude `Bash`
response. A missing field, unexpected type, `interrupted: true`, `isImage:
true`, or future incompatible shape is unsupported and passes through.

### Field classification

| Location | Classification | Use |
|---|---|---|
| lifecycle/event name | READ FOR SELECTION, VALIDATE | Admit successful `PostToolUse` only |
| `tool_name` | READ FOR SELECTION, VALIDATE | Select the `Bash` adapter |
| `tool_input.command` | READ FOR SELECTION | Select the `git diff/show` filter from authoritative input |
| `tool_response.stdout` | TRANSFORMABLE, VALIDATE | Normalize, redact, then optionally filter |
| `tool_response.stderr` | TRANSFORMABLE, VALIDATE | Normalize and redact; no first-slice diagnostic filter |
| `tool_response.interrupted` | READ FOR SELECTION, PRESERVE EXACTLY, VALIDATE | Require `false` |
| `tool_response.isImage` | READ FOR SELECTION, PRESERVE EXACTLY, VALIDATE | Require `false` |
| other `tool_input` members | IGNORE | Never log or copy into capability facts |
| other `tool_response` members | PRESERVE EXACTLY | Retain the original JSON values and keys |
| top-level hook metadata | IGNORE for transformation | Parse only what routing needs; never emit it as output facts |

“Preserve exactly” means semantic JSON preservation: the same key/value and
array structure survives. The plan does not promise byte-for-byte whitespace or
object-key ordering after JSON serialization.

The output is a Claude hook response whose `hookSpecificOutput` identifies
`PostToolUse` and whose `updatedToolOutput` is the validated reconstructed Bash
object. If no text changes, the adapter returns no replacement.

## 4. Transformation Pipeline

```text
stdin hook JSON
  -> parse bounded PostToolUse envelope
  -> select tool_name == Bash
  -> recognize BashSuccessV1 response shape
  -> preserve original JSON object
  -> stdout: normalize -> redact -> optional git diagnostic filter
  -> stderr: normalize -> redact
  -> replace only changed stdout/stderr values in a clone
  -> validate BashSuccessV1 invariants and preserved non-target members
  -> serialize hookSpecificOutput.updatedToolOutput
  -> stdout
```

The order is frozen as **normalize -> redact -> filter**. Removing supported
terminal escapes first makes pattern matching deterministic. Redaction runs
before reduction so a recognized secret is removed and counted even when the
filter later removes its line. Filtering is last because it is the lossy
context-reduction step.

The adapter runs synchronously. Standard output contains only the hook JSON
response. Diagnostics and structured capability facts use standard error.

## 5. Diagnostic Filter Contract

The first-slice input is:

```text
Bash tool identity
+ command from tool_input.command
+ Outcome::Success (established by PostToolUse)
+ stdout diagnostic text
```

The result contains transformed text and structured facts: filter identity,
original/reduced byte and line counts, reduction kind, and truncation/collapse
flags. It never receives a provider request, hook envelope, failure guess, or
combined stdout/stderr string.

Only a native-owned `git-diff-v1` rule is enabled initially:

- select with the command pattern `^git\s+(diff|show)\b` after conservative
  leading-whitespace handling;
- retain diff headers, hunk headers, additions, deletions, rename/new/deleted
  markers, and context required to understand a patch;
- remove index hashes, no-newline markers, and binary-file summary noise;
- cap the transformed result at 500 lines and report truncation;
- append one `§FILTERED git-diff: ...` disclosure line only when content was
  actually reduced;
- recognize its own terminal disclosure line and do no second reduction.

The existing `filters/git-diff.toml` is evidence for rule semantics and fixture
content, not a runtime dependency. The native implementation may encode this
single rule directly. Loading the whole proxy filter registry, TOML discovery,
community filters, output-first-line guessing, and proxy statistics are out of
scope.

JSON protection is unnecessary because the eligible field is known Git patch
text. `head`, `tail`, grouping, success-collapse, `on_empty`, stderr filtering,
and every other filter rule are disabled for the first slice.

## 6. Normalization Contract

The primitive accepts one known terminal-text field and returns text plus
`sequences_removed` and `bytes_removed`.

The supported grammar is the implementation already proven useful in the
repository: ESC, `[`, zero or more decimal digits or semicolons, followed by one
ASCII letter (`\x1b\[[0-9;]*[A-Za-z]`). This removes common SGR/color sequences
and the same narrow CSI forms currently recognized by Clean-CTX.

The transformation is deterministic and idempotent. It does not claim to parse
all ECMA-48/ANSI controls, OSC hyperlinks, private CSI parameters such as `?`,
intermediate-byte forms, cursor state, terminal width, carriage-return
rendering, or arbitrary C0 controls. Unsupported sequences remain unchanged.

The native module owns a compiled-once implementation. Existing proxy
normalizers may inform fixtures, but are not imported, moved, or modified.

## 7. Redaction Contract

The exact security claim remains:

> For the supported successful Bash schema, Clean-CTX may replace recognized
> secrets in `stdout` and `stderr` before that result is delivered to the
> model.

The first slice uses a fixed built-in policy and `[REDACTED]` placeholders. Its
initial pattern classes mirror the deterministic repository evidence:

- PEM private keys;
- recognized token formats;
- authorization-header credentials;
- secret-bearing command flag values;
- URL user information;
- secret assignment values; and
- database URL credentials.

Rules run in a fixed order against each field independently. No custom patterns
or runtime pattern loading are included. Pattern construction is completed
before event transformation; initialization failure prevents the hook handler
from advertising a usable transformer. A caught transformation panic or other
unexpected runtime error returns no replacement and records only a
non-sensitive failure reason. This is fail-safe pass-through, consistent with
the approved “may replace” guarantee; it is not an end-to-end secrecy claim.

Facts contain only rule-class counts and totals. They never contain matched
text, surrounding lines, replacement templates, raw output, command input, or
the reconstructed payload. Logging APIs accept facts rather than source text,
making accidental raw-payload logging structurally unnecessary.

The native implementation is independent. It may reproduce the stable pattern
semantics from `proxy/src/scrub.rs` and `proxy/src/scrub_patterns.rs` as
reference evidence, but the first slice does not extract or modify those files.

## 8. Claude Adapter Contract

The adapter owns:

- reading one bounded hook document from standard input;
- parsing lifecycle, tool identity, tool input, and response;
- selecting the `BashSuccessV1` schema;
- choosing only semantically eligible fields;
- invoking the three primitives in frozen order;
- reconstructing from the original object;
- validating required types, flags, and non-target preservation;
- returning `updatedToolOutput` only for a changed valid object;
- unchanged fallback for every unsupported or uncertain case; and
- emitting non-sensitive native facts to standard error.

It does not own transformation algorithms, HTTP/provider envelopes, caching,
context policy, permissions, discovery, model selection, or failure mutation.

The CLI surface is one explicit subcommand:

```text
clean-ctx claude-hook post-tool-use
```

It reads one hook event from stdin and writes one valid hook response to stdout.
The library handler remains independently testable without a subprocess.

## 9. Repository Ownership

The smallest proposed production layout is:

```text
src/native_text/
  mod.rs
  ansi.rs
  redaction.rs
  secret_patterns.rs
  git_diff_filter.rs
  facts.rs

src/claude_native/
  mod.rs
  hook.rs
  bash.rs
  pipeline.rs
  facts.rs
```

- `native_text` owns provider-independent text inputs/results and the three
  native implementations. It contains no Claude or HTTP types.
- `claude_native` owns Claude hook wire parsing and Bash schema knowledge.
- `src/lib.rs` exposes the two modules required by the binary.
- `src/main.rs` adds only CLI parsing and dispatch; hook behavior remains in
  the library.
- `Cargo.toml` gains only the direct dependency needed by the native pattern
  implementations (`regex`), because the root package cannot depend on a
  transitive proxy dependency.

No `proxy/**` file, proxy configuration, proxy test, or proxy package dependency
is part of the slice. A future evidence-based deduplication task may establish a
shared package, but this slice does not pre-design one.

## 10. Configuration

Hook registration is the complete first-slice opt-in. No Clean-CTX
configuration fields are added: installing `clean-ctx claude-hook
post-tool-use` means the three frozen transformations are active. A redundant
master flag could create an installed-but-silent no-op, and no current product
requirement justifies independent capability switches. Such switches remain a
future policy decision if real usage establishes a need.

There are no proxy, cache, context, permission, discovery, model, aging, or
tool-drop settings in the native path.

## 11. Capability Facts

One event-local `ClaudeNativeFacts` value contains only:

- event outcome (`replaced` or `passed_through`);
- tool identity (`Bash` only after safe recognition);
- adapter/schema identifiers and versions;
- targeted field names, never their contents;
- normalization sequence/byte counts per field;
- redaction counts grouped by stable rule class per field;
- filter identity, original/reduced byte and line counts, reduction kind,
  truncation, and collapse flags;
- validation outcome;
- a closed pass-through reason enum; and
- total handler duration in microseconds.

Facts are emitted as one structured standard-error event through the existing
stderr-oriented observability boundary. They are not added to model-visible
output and do not reuse proxy HTTP/cache counters. Tests must recursively scan
serialized facts to prove that sentinel secrets, raw output, and command input
are absent.

## 12. Pass-Through Matrix

| Condition | Result | Fact reason |
|---|---|---|
| native integration disabled | no replacement | `disabled` |
| unknown/unsupported tool | no replacement | `unsupported_tool` |
| event is not `PostToolUse` | no replacement | `unsupported_event` |
| `PostToolUseFailure` | no replacement | `failure_event` |
| hook JSON malformed or exceeds the bounded input limit | no replacement/error response per hook protocol; never synthesize tool output | `invalid_envelope` when observable |
| `tool_input` not an object or `command` not a string | no replacement | `unsupported_input` |
| response not an object | no replacement | `unsupported_schema` |
| required field missing or wrong type | no replacement | `unsupported_schema` |
| `interrupted` is true | no replacement | `interrupted` |
| `isImage` is true | no replacement | `image_result` |
| binary/opaque/future incompatible data | no replacement | `unsupported_schema` |
| ambiguous field semantics | no replacement | `ambiguous_extraction` |
| normalizer, redactor, or filter errors | no replacement | `transform_error` |
| no field changes | no replacement | `unchanged` |
| reconstruction loses or changes a non-target value | no replacement | `reconstruction_failed` |
| reconstructed schema validation fails | no replacement | `validation_failed` |
| hook response serialization fails | no replacement/error response per hook protocol | `serialization_failed` when observable |

No fallback converts structured data to text, combines streams, deletes unknown
fields, or returns a partially transformed result.

## 13. Tracked Regression Plan

All verification contracts are tracked under `src/tests/**` and registered from
their owning production modules through the repository's `#[path]` convention.

```text
src/tests/native_text/ansi.rs
src/tests/native_text/redaction.rs
src/tests/native_text/git_diff_filter.rs
src/tests/claude_native/bash_schema.rs
src/tests/claude_native/pipeline.rs
src/tests/claude_native/hook.rs
src/tests/claude_native/cli_contract.rs
```

Required cases:

1. `BashSuccessV1` admits the documented success fixture and rejects each
   missing/wrong-type required field, interruption, image, non-object response,
   unknown tool, and failure event.
2. Replacing stdout/stderr preserves every other response key/value, nested
   unknown value, array order, multiplicity, and top-level hook input value.
3. Common supported CSI sequences are removed; unsupported OSC/private forms
   remain; a second normalization is unchanged.
4. Each built-in secret class is redacted, benign near-matches remain, a second
   redaction is unchanged, and facts contain neither sentinel values nor source
   fragments.
5. `git diff/show` selection uses `tool_name` and `tool_input.command`, never
   output text. A different command containing Git-looking output is not
   filtered.
6. The Git fixture retains functional patch content, removes only frozen noise,
   reports correct sizes, marks truncation at 500 lines, and emits exactly one
   disclosure marker when reduced.
7. JSON guards, failure inference, stderr filtering, and unrelated catalog
   rules are demonstrably absent from the first-slice path.
8. A combined fixture proves normalize -> redact -> filter order, including a
   secret on a line later removed by the filter and ANSI around a retained line.
9. A forced transform error, reconstruction mismatch, and validation failure
   each return no replacement and the original input object remains untouched.
10. No-change input returns no `updatedToolOutput`.
11. `PostToolUseFailure` never produces replacement output.
12. Serialized facts contain only the allowlisted fields and never raw event,
    command, stdout, stderr, secret, or reconstructed payload.
13. CLI fixture input produces exactly one protocol-valid JSON response on
    stdout, with diagnostic facts confined to stderr.

These tests are frozen, not run in this planning phase.

## 14. Native End-to-End Verification Result

No repository fixture substitutes for the real Claude lifecycle. The operator
registered the built binary on Work Compute as:

```text
PostToolUse matcher: Bash
command: <built clean-ctx> claude-hook post-tool-use
```

The live proof ran successful Bash Git output through the hook and confirmed:

- a real successful Bash result reached `PostToolUse`;
- supported ANSI sequences were removed;
- a synthetic recognized credential was replaced;
- `git-diff-v1` reduced a 60-line result to 59 lines;
- the `§FILTERED git-diff: 60 → 59 lines` disclosure count matched the observed
  reduction;
- file headers, `+++`/`---` lines, and `@@` hunk headers remained readable;
- `updatedToolOutput` was accepted and consumed by Claude; and
- a compound command outside the conservative leading `git diff/show` selector
  passed through without a filter disclosure.

This evidence closes the supported first slice. It does not extend the security
claim to Claude telemetry, transcripts at rest, failed results, other tools, or
unsupported result shapes. The compound-command observation is conservative
pass-through evidence, not authorization to add shell parsing implicitly.

## 15. Implementation Sequence

1. Add RED tracked fixtures/tests for the Bash schema, primitive contracts,
   pipeline order, preservation, fallback, observability, and CLI wire output.
2. Add native-owned ANSI normalization and its facts.
3. Add native-owned fixed secret patterns/redaction and facts, with no raw-text
   logging surface.
4. Add the single native Git diff/show filter and model-visible disclosure.
5. Implement `BashSuccessV1` recognition, reconstruction, and validation over
   preserved original JSON.
6. Compose the synchronous pipeline and closed pass-through reasons.
7. Keep hook registration as activation; add no redundant configuration.
8. Add the `claude-hook post-tool-use` CLI dispatch and strict stdout/stderr
   separation.
9. Have the repository owner run the applicable authoritative verification
   gate; do not claim it passed until results are supplied.
10. Only after deterministic verification, execute the native end-to-end field
    proof and record any reproducible discovery.

No step touches the proxy. Later tool schemas are separate vertical slices.

## 16. Explicit Non-Goals

The first slice does not:

- modify, restructure, extract from, coordinate with, test, or retire the proxy;
- establish proxy/native parity or double-execution policy;
- support failed-result mutation;
- support tools other than the documented Bash success schema;
- load the full existing filter catalog or custom/community filters;
- introduce a generic provider framework or canonical `ToolResult` AST;
- flatten structured content or transform arbitrary strings;
- normalize all terminal control languages;
- promise secrecy for execution, hook infrastructure, telemetry, failures,
  transcripts, unsupported shapes, or every provider-bound surface;
- add caching, context, permission, discovery, model, aging, or tool-drop policy;
- address the separate OpenAI/Anthropic proxy-header candidate; or
- perform production implementation in this phase.

## Evidence Basis

Repository reference evidence:

- `filters/git-diff.toml`
- `proxy/src/filters.rs`
- `proxy/src/scrub.rs`
- `proxy/src/scrub_patterns.rs`
- `proxy/src/transform.rs`
- `src/main.rs`, `src/lib.rs`, `src/config.rs`, and `src/observability/`

These proxy paths were inspected only as behavioral evidence. They are not part
of the proposed dependency graph or change set.

Native contract evidence:

- [Claude Code hooks reference](https://code.claude.com/docs/en/hooks) for
  `PostToolUse`, `PostToolUseFailure`, Bash structured output,
  `updatedToolOutput`, validation, and telemetry timing.

## Frozen Principle

> **Build Claude-native Clean-CTX as a native integration, not as a migration
> of the proxy. Use Claude's authoritative lifecycle and structure first, add
> only narrow Clean-CTX intelligence, and leave the proxy alone.**
