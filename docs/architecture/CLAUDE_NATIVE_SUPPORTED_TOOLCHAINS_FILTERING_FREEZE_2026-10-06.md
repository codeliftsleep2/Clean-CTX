# Claude-Native Supported Toolchains Filtering Freeze

**Date:** 2026-10-06

**Status:** Supported-first scope and operation-sensitive field authority
approved. The shared reduction kernel and Rust build/check/clippy slice are
owner-verified green. The TypeScript/Angular slice is owner-verified green. The
C#/.NET slice is implemented and awaiting owner verification.

**Scope:** Architecture freeze and incremental implementation record for the
supported-toolchain native diagnostic slices. Cargo verification is performed
by the repository owner.

## 1. Executive Verdict

**SLICE 2 REQUIRES NEW AUTHORITY POLICY**

The previously proposed Python/Go batch is rejected as the next product step.
Clean-CTX should complete evidence-backed native diagnostic coverage for its
existing language ecosystems first:

1. Rust (`cargo`);
2. TypeScript and Angular (`tsc`, `ng`, `eslint`, and bounded Node build
   scripts);
3. C#/.NET (`dotnet build` and `dotnet test`); and
4. Java/Spring (`mvn` build operations).

This order follows the repository's actual language support rather than the
accidental breadth of the proxy filter catalog. Root Cargo features currently
name TypeScript, Angular, C#, .NET, Rust, and Java; they do not name Python or
Go. HTML diagnostics are covered through Angular's template/build lifecycle,
not through an invented standalone HTML producer.

The first supported-family expansion cannot remain inside Slice 1's
stdout-only diagnostic-filter authority. `filters/cargo.toml` explicitly marks
Cargo as `filter_stderr = true`. Treating Cargo output as stdout, combining the
two streams, or silently filtering stderr would each be architecturally
untruthful. The required new policy is narrow and operation-sensitive:

> A native diagnostic filter may transform an explicitly named Bash result
> field only when its recognized producer **operation** identifies that field
> as diagnostic output. Executable identity alone grants no field authority.
> Streams remain separate, and no generic stderr-filter authority is created.

Under that policy Cargo may own an explicit stderr filter. Other producers do
not inherit stderr authority. Approval of this policy is the stop condition
before implementation planning for the Rust slice.

## 2. Established Baseline

Slice 1 remains authoritative and is not reopened:

```text
Claude Bash success
  -> PostToolUse
  -> BashSuccessV1
  -> normalize stdout/stderr
  -> redact stdout/stderr
  -> optional stdout git-diff-v1 filtering
  -> reconstruct and validate
  -> updatedToolOutput
```

Tracked regressions and live Claude evidence established ANSI normalization,
recognized-secret redaction, Git filtering, structural preservation, accepted
`updatedToolOutput`, one disclosure line, and conservative pass-through for a
compound command.

The native lifecycle boundary remains success-only. `PostToolUseFailure` has
no approved replacement path, so “complete toolchain coverage” means complete
reviewed filtering of supported **successful** Bash results. It does not mean
filtering failed diagnostics or claiming authority Claude does not expose.

## 3. Supported-First Coverage Boundary

| Priority | Clean-CTX ecosystem | Native producer coverage | Explicit exclusions |
|---:|---|---|---|
| 1 | Rust | `cargo build`, `cargo check`, `cargo test`, `cargo clippy` | arbitrary `cargo run` program output; failed results |
| 2 | TypeScript/Angular | `tsc`; `ng build/test/lint`; `eslint`; exact build-script forms | arbitrary package scripts; dev/watch servers; failed results |
| 3 | C#/.NET | `dotnet build`, `dotnet test` | `dotnet run`; broad generic `dotnet` matching |
| 4 | Java/Spring | Maven compile/package/install lifecycle | arbitrary exec goals; tools absent from repository evidence |
| Deferred | Unsupported ecosystems | Python and Go catalog entries | all work until the four supported families above are complete |

“Complete” is bounded by repository evidence. It means every safe existing
catalog candidate in these supported ecosystems has an explicit native
decision: implement, narrow, or reject. It does not mean adding every tool that
can exist in a language ecosystem. Gradle, for example, is not added merely to
make Java coverage appear broader when no current Clean-CTX rule establishes
its semantics.

Each family is a separate vertical slice. Finishing one includes tracked
regressions, the repository-owner verification gate, and applicable live
Claude evidence before the next family begins. This preserves small reviewable
authority increments without allowing unsupported ecosystems to jump the
queue.

## 4. Simple-Command Eligibility Contract

All new producers share one conservative selection boundary. This is an
identity gate, not a shell parser.

Eligible commands:

- contain one supported executable and its ordinary arguments;
- may have leading or trailing whitespace;
- may contain quoted argument values when quotes are balanced and no control
  construct is interpreted outside the quoted value; and
- match one exact producer-owned command form after conservative tokenization.

Ineligible commands include:

- chaining or backgrounding: `;`, `&&`, `||`, or shell-control `&`;
- pipelines: `|`;
- input/output redirection, including `>`, `>>`, `<`, `2>`, and `2>&1`;
- command substitution using `$()` or backticks;
- subshells and command groups using `(...)` or `{ ...; }`;
- leading environment assignments;
- shell functions, aliases that cannot be identified from the command text,
  or wrapper commands not explicitly approved by a producer contract; and
- path-prefixed executables unless that exact path form is approved for the
  producer.

The gate prefers false negatives. It must never inspect output to recover an
identity rejected by the command gate. Slice 1's `git diff/show` selection can
be migrated behind the same gate in a behavior-preserving step, but that is
not permission to redesign its filter.

## 5. Producer Identity Matrix

| Command shape | Native identity | Eligible? | Reason |
|---|---|---:|---|
| `cargo build ...` | `cargo-build-v1` | Yes, after stream-policy approval | Repository rule names the operation and stderr |
| `cargo check ...` | `cargo-check-v1` | Yes, after stream-policy approval | Deterministic compiler-progress producer |
| `cargo test ...` | `cargo-test-v1` | Yes, after stream-policy approval | Cargo-owned stderr can be reduced; test stdout remains separate |
| `cargo clippy ...` | `cargo-clippy-v1` | Yes, after stream-policy approval | Deterministic Cargo/Clippy diagnostics |
| `cargo run ...` | none | No | stdout is arbitrary program output |
| `tsc ...` | `tsc-v1` | Yes | Direct compiler invocation is deterministic |
| `npx tsc ...`, `bunx tsc ...` | `tsc-v1` | Yes | Existing catalog explicitly supplies these wrappers |
| `pnpm tsc ...`, `yarn tsc ...` | none initially | No | Not established by the current `tsc` rule |
| `ng build/test/lint ...` | operation-specific Angular identity | Yes | Existing rule bounds these diagnostic operations |
| `eslint ...`, `npx eslint ...`, `bunx eslint ...` | `eslint-v1` | Yes | Existing rule supplies direct and wrapper evidence |
| exact `npm/pnpm/yarn/bun` build-script form | `node-build-v1` | Conditionally | Only approved build/compile/bundle script names |
| arbitrary `npm run <script>` | none | No | Script output may be arbitrary and overlaps other producers |
| `npm install` / `npm ci` | none in language slices | No | Package installation is not language diagnostic coverage |
| `dotnet build ...` | `dotnet-build-v1` | Yes | Specialized rule wins; generic rule is not imported |
| `dotnet test ...` | `dotnet-test-v1` | Yes | Specialized rule wins; result structure is test-oriented |
| `dotnet run ...` | none | No | Application stdout is arbitrary |
| Maven compile/package/install forms established by `mvn.toml` | `maven-build-v1` | Yes | Bounded build lifecycle output |
| arbitrary Maven goal/plugin execution | none | No | Plugin output can be arbitrary |
| Python or Go commands | none | No | Deferred until supported ecosystems are complete |

Producer identities are disjoint. A specialized identity is selected directly
from the command; there is no fall-through to a broad generic filter.

## 6. Filter Contract Matrix

| Identity | Diagnostic field | Retain | Remove | Bound | Collapse |
|---|---|---|---|---:|---:|
| Cargo operation identities | `stderr` only | warnings/errors, locations, summaries needed to understand the successful run | download/compile/check/fresh progress proven non-semantic by reviewed fixtures | Per-operation freeze required | No initially |
| `tsc-v1` | `stdout` | file, location, diagnostic code/severity, message, relevant context and counts | decoration/squiggle/version-only noise proven by fixtures | 100 lines unless revised before implementation | No |
| Angular operation identities | `stdout` | affected files, warnings, errors, bundle/test/lint summaries useful to Claude | stable build boilerplate only | 80 lines unless revised | No |
| `eslint-v1` | `stdout` | every warning/error, file/location, rule ID and summary | blank/decorative noise only | 140 lines unless revised | No; warnings must never become `ok` |
| `node-build-v1` | `stdout` | all semantic build output | blank lines only in first version | 120 lines unless revised | No |
| `dotnet-build-v1` | `stdout` initially | warnings/errors, projects/artifacts, success summary | stable build boilerplate proven by fixtures | 40 lines unless revised | No |
| `dotnet-test-v1` | `stdout` initially | test counts, failures if ever present on a successful event, framework summary | stable runner boilerplate | 100 lines unless revised | No |
| `maven-build-v1` | `stdout` initially | module/build result, warnings, useful summary | download/plugin progress and stable boilerplate | 50 lines unless revised | No |

Every stream assignment after Cargo is provisional until that family slice
captures representative fixtures. If a producer needs stderr to remain
truthful, it must receive an explicit producer contract under the approved
field-authority policy; it cannot inherit Cargo's permission.

All filtering remains atomic with the existing normalize -> redact -> filter
order. Streams are never concatenated. An error in either field transformation
returns no replacement for the event.

## 7. Stream-Authority Decision

The governing invariant is:

> **Authority is operation-sensitive when the executable can cross semantic
> roles.**

An executable name selects only a producer family. The recognized operation
within that family determines whether the invocation is eligible, which output
field is diagnostic, and which reducer may run. An unknown, omitted,
ambiguous, or arbitrary-output operation grants no filtering authority.

For Cargo, the frozen authority derivation is:

```text
cargo check   -> CargoDiagnostic::Check   + stderr
cargo build   -> CargoDiagnostic::Build   + stderr
cargo clippy  -> CargoDiagnostic::Clippy  + stderr
cargo test    -> separately frozen Cargo test diagnostic boundary
cargo run     -> arbitrary executed-program behavior -> ineligible
other Cargo operation -> no authority -> ineligible
```

`cargo test` is intentionally not granted generic Cargo stderr authority by
this table. Before its implementation, tracked fixtures must distinguish
Cargo/compiler-owned diagnostic progress from test-runner and test-program
output and freeze whether a useful field-local transformation exists. If that
boundary cannot be established without merging streams or interpreting
arbitrary test output, `cargo test` remains ineligible.

The exact requested approval is:

```text
NativeDiagnosticTarget {
  producer_identity,
  operation_identity,
  field: Stdout | Stderr,
  filter_identity,
}
```

The type need not use this spelling, but production ownership must enforce the
same invariant: filter authority belongs to a recognized
producer/operation/field tuple, not to an executable or a global stream
switch.

Consequences:

- Approved Cargo build/check/clippy operations may target only `stderr` in the
  first Rust slice.
- `cargo test` stdout is not Cargo progress merely because the command began
  with `cargo`; neither stream receives filtering authority until its narrower
  diagnostic boundary is separately frozen.
- Normalization and redaction continue on both text fields under Slice 1.
- No producer may combine stdout and stderr for filtering.
- A field not named by the producer contract receives no diagnostic filter.
- Failure events remain unchanged.

This is a new native authority policy and requires explicit approval under the
repository's architectural gate.

## 8. Success-Collapse Findings

The existing catalog frequently collapses successful output to `tool: ok`.
That behavior is useful historical evidence but is not required to make the
supported-family filters useful.

Success collapse is therefore excluded from all four family slices initially.
Noise stripping, structural retention, and bounded truncation provide useful
reduction without replacing the whole successful result. This particularly
avoids the existing ESLint behavior that can collapse warning-only output and
hide model-useful warnings.

Any later collapse proposal is a separate semantic-authority decision with
producer-specific evidence and regressions. It must not enter as an incidental
port of TOML behavior.

## 9. Overlap and Arbitrary-Output Findings

The native path does not load the proxy catalog or inherit its ordering.

- `dotnet-build-v1` and `dotnet-test-v1` replace the overlapping generic
  `dotnet` interpretation. `dotnet run` is rejected.
- `tsc`, Angular, and ESLint identities take precedence by being exact and
  disjoint. They never fall through to `node-build-v1`.
- `node-build-v1` recognizes only explicit build/compile/bundle script forms.
  Generic `npm run`, install behavior, and unknown scripts are rejected.
- Maven recognizes only the reviewed build lifecycle forms, not arbitrary
  plugin goals.
- Cargo recognizes selected build/check/test/clippy operations, not `run`.

These producers are safer than `curl`, `ssh`, `kubectl`, or `gh` only where the
selected operation has a stable diagnostic/build/test contract. The command
name alone is not sufficient. Any operation capable of emitting arbitrary
user-requested or application data is excluded even when its executable also
has diagnostic modes.

## 10. Native Ownership

The verified Slice 1 ownership remains intact:

- `src/claude_native/bash.rs` owns the `BashSuccessV1` shape;
- `src/claude_native/pipeline.rs` owns field-local orchestration,
  reconstruction, validation, and atomic pass-through;
- `src/claude_native/facts.rs` owns non-sensitive event facts; and
- `src/native_text/` owns provider-independent command selection and text
  reducers.

The smallest implementation extension is:

1. add a conservative simple-command classifier under `claude_native` or a
   narrowly named native command-selection module;
2. add one producer-specific reducer module at a time under `native_text`;
3. generalize filter facts only enough to represent existing identifiers and
   reduction measurements; and
4. replace the Git-only conditional in the pipeline with explicit
   producer/field dispatch.

This does not justify a shell AST, universal filter registry, provider-neutral
tool result, dynamic TOML loader, or proxy dependency. If the explicit match
becomes unwieldy after all supported families are implemented, abstraction can
be reconsidered from measured duplication.

### Shared native reduction invariant

Native diagnostic filtering uses shared deterministic reduction mechanics with
producer-specific semantic policies:

```text
operation-sensitive classifier
  -> typed authorized producer/operation/field target
  -> producer-specific line retention semantics
  -> shared line-reduction kernel
  -> reduced text plus common FilterFacts
```

The shared kernel owns line and byte accounting, deterministic retained-line
assembly, bounded truncation, disclosure formatting, exactly-once disclosure
protection, changed/unchanged detection, and the common result shape. It has no
command, tool, lifecycle, stream, Git, Cargo, or provider knowledge.

Git continues to own patch-noise recognition and its 500-line policy. Cargo
owns build/check/clippy progress-noise recognition, operation labels, and its
100-line policy. Different producer semantics are not forced into the common
layer merely because they both operate on lines.

> **Authority is granted before reduction by the operation-sensitive
> classifier. Shared reduction infrastructure cannot infer or expand
> authority. Shared code executes authority; it never grants authority.**

## 11. Facts and Disclosure Contract

Slice 1's fact boundary is sufficient. Each reducer records only:

- stable producer/filter identity;
- targeted field name;
- original and reduced byte/line counts;
- reduction kind;
- truncation and collapse flags; and
- validation/pass-through outcome.

Facts never contain the raw command, output, diagnostic text, filenames,
secrets, or matched fragments.

The model-visible disclosure remains adapter-owned and is appended only when a
reducer changes content:

```text
§FILTERED <stable-filter-label>: <original> -> <reduced> lines
```

Each reducer recognizes only its own terminal disclosure marker. A second
application returns the text unchanged and emits no second marker. Disclosure
generation is shared behavior; diagnostic reduction remains producer-owned.

## 12. Tracked Regression Plan

All required tests remain under `src/tests/**` and are registered through the
repository's existing `#[path]` convention.

Common command-selection regressions:

- admit every exact supported simple form;
- preserve ordinary and quoted arguments;
- reject chaining, pipelines, redirection, substitution, subshells, groups,
  environment prefixes, path prefixes, wrappers, and deceptive substrings;
- prove that output content cannot select a producer; and
- preserve the verified compound-command pass-through behavior.

Per-family reducer regressions:

- use representative tracked fixtures for every approved operation;
- retain file/location, severity, message, context, warning, and summary data
  named by that producer contract;
- remove only frozen noise;
- prove deterministic bounds and exactly one disclosure when changed;
- prove idempotence/repeat protection; and
- prove unsupported operations remain unchanged.

Stream regressions:

- Cargo transforms only approved stderr and leaves stdout filtering absent;
- each later producer transforms only its frozen field;
- streams are never merged;
- existing normalization/redaction still apply independently;
- a transform/reconstruction/validation error produces no replacement; and
- failure events never produce replacement output.

Facts regressions recursively reject raw command, output, diagnostic, filename,
and secret sentinels. Existing Slice 1 behavior is tested only where the new
dispatch or eligibility gate could regress it.

## 13. Implementation Sequence

Implementation is authorized only after approval of the
producer/operation/field authority policy.

1. **Rust slice:** simple-command gate, Cargo operation identities, explicit
   operation-specific stderr targets for build/check/clippy, a separate
   `cargo test` boundary decision, conservative noise reduction, facts,
   disclosure, and tests.
2. **TypeScript/Angular slice:** `tsc`, operation-specific Angular, ESLint with
   warnings retained, then exact Node build scripts.
3. **C#/.NET slice:** specialized build/test identities with no generic
   fallback and no `dotnet run` filtering.
4. **Java/Spring slice:** evidence-backed Maven build lifecycle operations.
5. Only after all four slices pass their deterministic and applicable live
   gates may Python, Go, or other unsupported-language toolchains be proposed.

Each step begins with RED tracked tests and ends with the user-owned
authoritative verification gate. No step modifies or restructures the proxy.

## 14. Deferred Policy Classes

- failed-result mutation;
- arbitrary application or remote-command output;
- success collapse;
- stream merging;
- dynamic filter catalogs or registries;
- proxy/native deduplication;
- Python and Go toolchains;
- tools without current repository evidence, including speculative Java build
  systems;
- dev/watch/server processes; and
- broad package-install/deployment filtering.

## 15. Stop Conditions

Stop before implementation if any of these occurs:

1. the producer/operation/field authority policy above is not explicitly
   approved;
2. representative fixtures show a proposed field does not contain the
   diagnostic structure assumed here;
3. safe selection requires shell interpretation beyond the conservative gate;
4. a producer needs success collapse to be useful;
5. an operation emits arbitrary application/user-requested output;
6. deterministic selection would require ambiguous catalog precedence;
7. implementation would require proxy changes or a shared global registry; or
8. a family slice discovers a need to mutate failed results; or
9. executable identity is known but its operation is unknown, ambiguous, or
   crosses into arbitrary program execution.

In those cases, narrow the producer contract or open a separate architecture
decision. Do not broaden native authority as an implementation convenience.
The ninth condition is ineligible input, not permission to fall back to
family-wide authority.

## 16. Rust Slice Verification Record

On 2026-10-06, the repository owner reported all requested local verification
green. This closes deterministic verification for:

- the shared native line-reduction kernel;
- preserved `git-diff-v1` behavior through that kernel;
- operation-sensitive Cargo build/check/clippy selection;
- Cargo stderr-only diagnostic filtering;
- rejection of Cargo test/run/unknown/wrapper/compound invocations;
- producer and field isolation; and
- non-sensitive shared filter facts.

No Rust live-Claude claim is made. The real Claude hook, binary, pipeline, and
`updatedToolOutput` lifecycle was already established by Slice 1; Rust/Cargo is
not a representative live workflow for this deployment. Its new behavior is
therefore accepted through the tracked deterministic repository verification.

## 17. TypeScript/Angular Slice Implementation Record

The next supported-family slice implements stdout filtering for these exact
successful simple-command forms:

- direct `tsc`, plus the catalog-established `npx tsc` and `bunx tsc` wrappers;
- direct `ng build`, `ng test`, and `ng lint`;
- direct, `npx`, and `bunx` ESLint, plus the catalog-established
  `npm|pnpm|yarn [run] eslint|lint` scripts; and
- exact `npm|pnpm|yarn|bun [run] build|compile|bundle` scripts.

Explicit watch mode, informational/help/config-listing modes, unsupported
wrappers, package installation, arbitrary scripts, shell compounds, and
redirection remain ineligible. This prevents executable-family identity from
granting filtering authority when the requested operation has changed from
diagnostics to user-requested information. All four producer families retain
successful summaries and warnings; none uses success collapse.
Their existing catalog bounds become producer policy supplied to the shared
line-reduction kernel: TSC 100, Angular 80, ESLint 140, and Node build 120.

Authority remains structurally upstream of reduction:

```text
simple command
  -> typed TSC/Angular/ESLint/Node-build operation target
  -> stdout selection
  -> producer-specific noise semantics
  -> shared deterministic line reduction
```

Cargo-like or Git-like stderr remains untouched for these targets. Output text
cannot select a producer. Tracked tests cover command admission/rejection,
semantic retention, bounds, disclosure, producer isolation, and field
isolation. Verification results must not be recorded until supplied by the
repository owner.

The repository owner reported the requested TypeScript/Angular verification
green on 2026-10-06. This closes its deterministic local gate.

## 18. C#/.NET Slice Implementation Record

The .NET slice uses only specialized successful simple-command identities:

```text
dotnet build -> DotnetOperation::Build -> stdout -> dotnet-build-v1
dotnet test  -> DotnetOperation::Test  -> stdout -> dotnet-test-v1
```

The broad proxy `dotnet` rule is not imported. `dotnet run`, `dotnet watch`,
unknown operations, wrappers, shell compounds, help modes, build property,
item, or target-result queries, and test-listing modes are ineligible. This
prevents the executable family from gaining authority over application output
or explicitly requested informational output.

Both reducers preserve warnings, project/test identities, success summaries,
error/warning counts, test counts, and test timing. Neither operation uses
success collapse. Producer policy supplies distinct shared-kernel bounds: 40
total lines for build and 100 for test.

Tracked regressions cover specialized selection, rejected semantic roles,
boilerplate removal, warning/summary retention, distinct bounds and identities,
stdout-only authority, stderr isolation, and output-content non-authority.
Verification results must not be recorded until supplied by the repository
owner.

## Frozen Principle

> **Finish native filtering for the language ecosystems Clean-CTX actually
> supports before adding foreign toolchains. Give each producer authority only
> over its proven diagnostic field, preserve stream identity, and reject
> arbitrary-output modes.**
