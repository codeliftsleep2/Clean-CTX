# Diagnostic Architecture Decision-Leverage Falsification

**Date:** 2026-10-07  
**Status:** High-leverage synthetic and resolution probes completed; live MCP secrecy/selection and cross-platform process-lifetime evidence remain blocking.  
**Starting architecture:** `NATIVE_DIAGNOSTIC_CONTEXT_ARCHITECTURE_INVESTIGATION_2026-10-07.md`  
**Observed starting commit:** `46a6acd3` (`docs(architecture): define native diagnostic context architecture`)  
**Observed working-tree state before this report:** clean  
**Scope:** Decision-leverage falsification only. No production implementation, production test modification, Cargo execution, MCP runtime startup, or process-supervisor implementation.

## Evidence vocabulary

- **PROVEN** — established by repository state or completed controlled evidence.
- **DOCUMENTED** — stated by an authoritative platform or producer source.
- **OBSERVED** — measured in a bounded experiment without claiming a universal guarantee.
- **INFERRED** — reasoned from proven or documented evidence.
- **UNRESOLVED** — insufficient evidence; a specified experiment remains necessary.

---

## 1. Verdict

**INSUFFICIENT EVIDENCE — SPECIFIC OWNER/ENVIRONMENT EXPERIMENT REQUIRED**

The repository owner subsequently reported that the complete baseline gates were green. That statement is accepted as **OBSERVED owner evidence** for baseline commit `46a6acd3`; the agent did not run or independently inspect the gate outputs.

Three bounded, non-Cargo falsifiers were then performed:

1. semantic compilation versus disciplined terminal filtering;
2. Windows executable-resolution substitution and absolute-path stability; and
3. truthful degradation under mixed, malformed, and truncated Cargo-style evidence.

None hit its kill condition. The architecture therefore survives these three gates provisionally, with one required modification: executable resolution must produce and validate an absolute producer path before changing to or executing within the repository workspace. Bare-name lookup and inherited `PATH` are not acceptable authority boundaries.

The architecture has not yet survived the full high-leverage series. Cross-platform process-tree ownership remains a core blocker. Live MCP selection and secret-boundary experiments also remain necessary before MCP can honestly be called the primary agent interface or the direct-MCP secrecy advantage can be claimed.

---

## 2. Decision-Leverage Ranking

The ranking separates architectural leverage from execution order. Once the baseline exists, experiments should be ordered by the cheapest reliable opportunity to kill the most expensive assumption.

| Rank | Unresolved assumption | Architecture at risk | Decision leverage | Cheapest reliable falsifier |
|---:|---|---|---|---|
| 1 | Semantic diagnostic compilation materially improves decision-quality/context economics over disciplined filtering | Canonical `DiagnosticResult`, producer parsers, semantic rendering, much of the new product domain | Critical | Bounded synthetic Cargo evidence rendered raw, filtered, and semantically; compare preserved decisions and context, not only bytes |
| 2 | Restricted producer resolution can exclude repository-controlled substitution without arbitrary executable acceptance | Entire authoritative execution core | Critical | Platform-specific resolution probe using benign local shims, manipulated `PATH`/`PATHEXT`, trusted installation, and configured absolute path |
| 3 | Clean-CTX can own and terminate descendant process trees without becoming a general process manager | Entire authoritative execution core | Critical | Minimal parent → child → grandchild fixture using OS-native ownership primitives; normal, graceful, and forced termination |
| 4 | Claude reliably selects one typed MCP diagnostic tool for obviously matching tasks | MCP-primary UX, not necessarily the execution core | High | Small non-production MCP fixture across four availability/guidance conditions and multiple neutral task phrasings |
| 5 | Mixed or malformed structured evidence can reduce semantic confidence without corrupting process authority | Canonical result truthfulness and parser boundary | High | Pure bounded Cargo-style fixture with valid, non-JSON, malformed, truncated, and completion records |
| 6 | Direct MCP execution keeps raw producer output outside inspected Claude-owned result surfaces until sanitization | A major security advantage over `PostToolUse` and architecture scoring | High | Synthetic secret through investigative MCP execution; inspect response, transcript, enabled debug logs, and documented surfaces only |
| 7 | Host hook input remains complete at relevant successful-output sizes | Compatibility fallback quality | Medium | Deterministic head/middle/tail output fixture through existing hook path |
| 8 | Large concurrent output can be handled within acceptable memory/spool bounds | Production hardening and global resource policy | Medium | Bounded stream-volume fixture after execution architecture survives |
| 9 | Structured artifacts remain useful after crash/cancellation | .NET/Maven producer design | Medium–later | TRX/Surefire/MSBuild abnormal-termination study |

### Why semantic value ranks first

If semantic compilation does not materially improve context quality or size, the proposed canonical domain is unjustified even if process execution can be made safe. The execution core might still support better filtering, but the most expansive architectural claim would be dead. A synthetic comparison can falsify that claim without first implementing a runner.

### Why resolution and lifetime outrank MCP selection

MCP selection can falsify MCP-primary UX while leaving the restricted execution core and CLI useful. Unsafe executable resolution or unreliable process-tree cleanup invalidates the authority core itself. Those assumptions therefore carry greater architectural leverage.

### Why parser truthfulness follows core feasibility

Parser truthfulness is mandatory, but a bounded producer-specific result can likely represent incomplete coverage without a universal schema. It should be tested before design freeze, after the cheaper semantic-value test and alongside—not ahead of—core execution safety.

---

## 3. Experiments Actually Performed

The following prerequisite inspection was performed and must not be reported as an experiment:

```text
Inspection: Git starting state
Observed commit: 46a6acd3
Observed branch: feature/claude-native-tool-integrations
Observed initial working tree: clean
Classification: PROVEN for Git state only
```

This inspection proves only that the tracked working tree was clean at the observed commit. It does not prove compilation, tests, Clippy, file-size enforcement, encoding, live Claude behavior, process security, or any Gate A–F claim.

The repository owner then reported the complete baseline gate green. Exact command transcripts and environment-version fields were not provided, so the report records the baseline as owner-observed rather than independently proven.

### Experiment A — Semantic compilation decision value

```text
Question:
Does semantic diagnostic rendering materially improve context over disciplined
terminal filtering without losing the next coding decision?

Hypothesis:
A producer-specific semantic rendering will preserve diagnostic identity,
location, severity, outcome, and unparsed evidence with less irrelevant context.

Architecture assumption under test:
Diagnostic facts earn a canonical producer-specific domain boundary.

Fixture/setup:
Disposable synthetic Cargo evidence under target/tmp/diagnostic-falsification.
The evidence contained one E0308 error, one unused-variable warning, locations,
codes, failed completion, Cargo progress, and one non-JSON build-script line.

Observed result:
Raw terminal:       957 bytes, 26 lines, ~240 chars/4 estimated tokens
Filtered terminal:  793 bytes, 20 lines, ~198 chars/4 estimated tokens
Semantic rendering: 575 bytes, 13 lines, ~143 chars/4 estimated tokens

Evidence classification:
OBSERVED on a bounded synthetic fixture.

What it proves:
For this fixture, semantic rendering preserved the same immediate coding
decision—repair E0308 at src/lib.rs:27:22 by supplying a u32—plus the warning,
codes, failed outcome, and unparsed build-script evidence, while using about
27.5% fewer bytes and 35% fewer lines than disciplined filtering.

What it does NOT prove:
It does not prove model performance, real Cargo parser completeness, savings
across repositories, or that every producer needs a shared diagnostic IR.

Architecture consequence:
The semantic-domain kill condition was not met. Producer-specific semantic
results remain justified for design-freeze consideration; a universal IR does not.
```

The chars/4 values are explicitly estimates, not tokenizer measurements. No appropriate already-available tokenizer was used.

### Experiment B — Windows executable-resolution substitution

```text
Question:
Can a Cargo-first operation avoid repository-controlled producer substitution?

Hypothesis:
Bare-name/PATH resolution is unsafe, but an explicitly resolved and validated
absolute cargo.exe path remains stable under repository PATH manipulation.

Architecture assumption under test:
Restricted producer identity can be bounded without accepting arbitrary executables.

Fixture/setup:
Disposable repository-local cargo.cmd and cargo.bat markers were placed first
in a process-local manipulated PATH/PATHEXT. No marker or Cargo executable ran.
PowerShell resolution was inspected before and after manipulation.

Observed result:
Normal cargo.exe: C:\Users\MNasty\.cargo\bin\cargo.exe
Bare cargo candidates after manipulation: repository cargo.cmd, repository
cargo.bat, then the real cargo.exe.
Explicit cargo.exe after manipulation: the real cargo.exe.
Previously resolved absolute path: exists and is outside the repository.

Evidence classification:
OBSERVED for this Windows environment; DOCUMENTED for Rust Command lookup behavior.

What it proves:
PATH-sensitive bare resolution is not trustworthy. A Cargo-first policy can
instead accept a user-authorized configured absolute path or resolve a trusted
toolchain path before entering repository-controlled cwd, canonicalize it,
validate its file type/location, record it, and spawn that absolute path.

What it does NOT prove:
It does not establish publisher identity, file-signature policy, Unix behavior,
or policy for ecosystems where project-local tools are intentionally authoritative.

Architecture consequence:
The Cargo-first resolver kill condition was not met. Absolute pre-resolution is
a required architectural constraint, not an optional hardening step.
```

Rust documents that relative/bare `Command` programs use platform-specific lookup, recommends an absolute path to avoid surprises, and on Windows resolves `.exe` names through an ordered search. It also documents that non-`.exe` extensions are not found when omitted. [Rust `std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html)

### Experiment C — Parser truthfulness under mixed evidence

```text
Question:
Can malformed and mixed records reduce parser confidence without erasing process authority?

Hypothesis:
Valid records can contribute facts while plain, malformed, and truncated lines
remain bounded evidence and mark coverage incomplete.

Architecture assumption under test:
The first producer does not require a universal schema to preserve truth.

Fixture/setup:
One JSON-lines fixture containing two valid compiler messages, one plain line,
one malformed JSON-looking record, one truncated JSON-looking record, and one
valid failed build-finished record. Authoritative process exit was supplied as 101.

Observed result:
Valid records: 3
Parsed diagnostics: 2
Completion success: false
Plain evidence lines: 1
Malformed/truncated JSON-looking lines: 2
Coverage: incomplete
Unparsed lines preserved: 3
Authoritative outcome: failure from process exit 101

Evidence classification:
OBSERVED on a bounded synthetic fixture.

What it proves:
Process outcome, parsed facts, producer completion, parser coverage, and raw
fallback evidence can remain separate without a universal diagnostic abstraction.

What it does NOT prove:
It does not prove production parser recovery, streaming behavior, memory bounds,
or coverage of future Cargo record shapes.

Architecture consequence:
The parser-truthfulness kill condition was not met. The result contract must
represent incomplete coverage and preserve unparsed evidence explicitly.
```

No Cargo command, test, build, runtime, MCP server, or process-lifetime fixture was started.

---

## 4. MCP Selection Result

**UNRESOLVED**

MCP-primary UX remains a provisional hypothesis, not a supportable conclusion.

The required experiment needs a live Claude environment and an investigative MCP fixture. The fixture needs only one clearly named typed diagnostic-like operation and deterministic harmless output; it must not implement the production execution architecture.

Minimum conditions:

```text
A. Bash available; diagnostic MCP tool available
B. Bash available; diagnostic MCP tool + concise project guidance
C. Bash available; CLI guidance only
D. Bash-only baseline
```

Use multiple neutral phrasings such as “check this Rust workspace and report the diagnostics” rather than explicitly naming the tool. Record the first selected tool, fallback, retry, intervention, and description/guidance condition.

### Kill condition

If a clear tool description plus concise project guidance cannot produce reliable selection for obviously matching tasks, MCP must be demoted from assumed primary interface. The execution core and CLI may survive.

---

## 5. Executable Resolution Result

**SURVIVES FOR THE CARGO-FIRST SLICE WITH REQUIRED MODIFICATION**

The controlled Windows probe confirmed that unqualified `PATH` lookup is not an authority boundary. Repository-controlled `.cmd` and `.bat` candidates appeared ahead of the real Cargo executable after process-local `PATH`/`PATHEXT` manipulation.

The already resolved absolute `cargo.exe` path remained stable, existed, and was outside the repository. Rust's documented `Command` behavior supports using an absolute path to avoid platform-specific lookup.

A viable policy would need to distinguish:

| Category | Trust interpretation |
|---|---|
| OS executable resolution | Platform mechanism only; not inherently trusted |
| Ecosystem-approved project-local tooling | Repository-controlled but sometimes semantically required; must be explicit per operation |
| User-configured producer | User authority if absolute, validated, and recorded |
| Repository-controlled executable | Untrusted by default |
| Trusted toolchain installation | Candidate authority after platform-specific identity and path validation |

The completed probe covered repository `.cmd`/`.bat`, manipulated `PATH`/`PATHEXT`, the normal installed Cargo path, explicit `.exe` selection, and a configured absolute path. A repository-local spoof `.exe`, publisher identity, and Unix cases remain unresolved.

The required Cargo-first policy is:

1. obtain the producer from user configuration or a reviewed trusted-toolchain resolver;
2. resolve and canonicalize it before applying repository cwd;
3. require an absolute regular executable path;
4. reject a path within the repository unless that producer contract explicitly authorizes project-local tooling;
5. record the resolved producer identity in the result;
6. spawn only that absolute path.

Remaining Windows/Unix probes should use harmless marker executables/scripts and test:

- a repository-local producer-named `.cmd`;
- a repository-local producer-named `.exe`;
- manipulated `PATH`;
- relevant `PATHEXT` ordering;
- normal toolchain resolution;
- configured absolute producer path.

The Unix companion should test manipulated `PATH`, a repository-local executable, symlink resolution, expected toolchain installation, and configured absolute path.

### Kill condition

If the remaining platforms or producers require arbitrary executable acceptance or opaque resolution behavior that cannot be surfaced truthfully, restricted execution is not acceptable as the primary authority boundary. That condition was not observed for Cargo on this Windows environment.

---

## 6. Process-Lifetime Result

**UNRESOLVED**

No process fixture was started. Documentation that Windows Job Objects and Unix process groups exist is insufficient to prove Clean-CTX can use them with the required semantics across supported build tools.

The smallest adequate fixture is:

```text
controller
└─ parent
   └─ child
      └─ grandchild
```

Every process should write deterministic start, graceful-shutdown, and final-exit markers to an isolated temporary directory. Required cases:

- normal completion;
- graceful cancellation;
- forced cancellation after a bounded grace period;
- parent exit while child remains active;
- nested descendant cleanup.

Windows must exercise an explicit descendant-ownership primitive such as a Job Object. Unix must exercise a dedicated process group or session and graceful/forced group signaling.

### Kill condition

If descendants cannot be reliably owned and terminated without broad general-purpose process management, restricted execution requires architectural reconsideration.

---

## 7. Semantic Compilation Result

**SURVIVES ON BOUNDED SYNTHETIC EVIDENCE**

The comparison was performed without Cargo using a bounded fixture derived from Cargo's documented evidence shapes.

The fixture must contain:

- at least one error with file, line, column, severity, code, and message;
- at least one warning where practical;
- build completion evidence;
- ordinary progress/noise;
- non-JSON evidence;
- enough causal detail to support a concrete next coding decision.

Compare:

```text
A. raw producer representation
B. disciplined current-style filtered terminal text
C. structured semantic rendering + bounded unparsed evidence
```

Evaluate:

- bytes, lines, and token count when an appropriate tokenizer is available;
- diagnostic identity and code;
- primary and related locations;
- severity;
- authoritative outcome;
- unparsed evidence;
- omitted information and disclosure;
- ambiguity in the next coding decision.

The semantic rendering preserved the same immediate coding decision and required authority while reducing disciplined-filtered output from 793 to 575 bytes and from 20 to 13 lines. Its chars/4 estimate fell from 198 to 143 tokens. This is a material relative reduction in the bounded fixture, although the absolute sample is small and no model decision trial was run.

### Kill condition

The kill condition was not met. A producer-specific semantic result remains justified for design freeze, but the evidence does not justify a universal cross-tool diagnostic IR.

---

## 8. Parser Truthfulness Result

**SURVIVES ON BOUNDED SYNTHETIC EVIDENCE**

The truth model was exercised successfully:

```text
authoritative process outcome
+ parsed producer facts
+ parser coverage
+ bounded sanitized unparsed evidence
```

The bounded fixture included, in one stream:

- valid structured records;
- an ordinary non-JSON line;
- a JSON-looking malformed record;
- a truncated record;
- valid completion evidence.

The result must demonstrate:

| Evidence | Required authority behavior |
|---|---|
| Process exit status | Preserved independently of parsing |
| Valid records | Contribute structured facts |
| Non-JSON records | Preserved as categorized raw evidence |
| Malformed/truncated records | Reduce parser coverage; never disappear silently |
| Completion record | Contributes producer evidence but does not override actual process status |
| Rendering | Discloses incomplete parsing and omitted evidence |

### Kill condition

No universal schema was required. The experiment separated process outcome, parsed producer facts, producer completion, coverage, and unparsed evidence. The kill condition was not met.

---

## 9. Secret-Boundary Result

**UNRESOLVED**

No secret-bearing fixture or MCP result was executed. The following claim remains an inference:

> A direct MCP diagnostic runner can keep raw producer output within the producer/Clean-CTX boundary and return only a sanitized result to Claude.

The resumed experiment must use synthetic secrets and report each boundary independently:

| Boundary | Current classification |
|---|---|
| Before Clean-CTX | Not protected by the proposed architecture |
| Before Claude host receives the MCP result | INFERRED, requires experiment |
| Before local persistence | UNRESOLVED and surface-specific |
| Before model visibility | INFERRED, requires transcript/result inspection |
| Before provider egress | INFERRED for the sanitized result; host behavior still requires evidence |

Only authorized and technically inspectable surfaces may support conclusions. Absence from the MCP response does not prove absence from debug logs, transcript state, telemetry, temporary files, or uninspected host internals.

### Kill condition

If raw producer output enters Claude-owned lifecycle state before Clean-CTX returns the sanitized MCP result, a major advantage over `PostToolUse` disappears and the architecture must be rescored.

---

## 10. Falsified Assumptions

The following assumption was falsified:

> Inherited or bare-name executable lookup can participate in the trusted producer boundary.

It cannot. Producer identity for the authoritative path must resolve to an absolute validated path before repository-controlled execution context is applied.

One procedural assumption is also rejected:

> A clean committed repository state is sufficient to begin controlled architecture experiments.

It is not. The recorded owner-run verification gate is the experimental control.

The semantic-domain and parser-truthfulness assumptions survived bounded synthetic falsification. This does not elevate them to universal proof.

---

## 11. Surviving Architecture

The architecture has survived three high-leverage bounded falsifiers, but not the full series.

The smallest architecture still eligible for testing—not yet justified—is:

```text
typed diagnostic request
        │
closed producer operation
        │
bounded direct execution candidate
        │
producer evidence
        │
either:
  disciplined text rendering
or
  producer-specific semantic result
```

The smallest surviving candidate is:

```text
typed CargoCheck request
        │
absolute pre-resolved trusted cargo.exe
        │
direct execution candidate
        │
authoritative process outcome + Cargo JSON evidence
        │
producer-specific semantic facts
+ explicit incomplete coverage
+ bounded sanitized unparsed evidence
        │
concise rendering
```

The following components remain hypotheses:

- MCP as primary agent interface;
- Clean-CTX as safe process owner;
- canonical `DiagnosticResult`;
- direct-MCP secrecy advantage.

The CLI and `PostToolUse` compatibility roles remain plausible because they do not depend on all four hypotheses simultaneously, but they were not retested.

---

## 12. Remaining Unknowns

### Baseline status

- Green gate: owner reported green at the baseline commit.
- Exact command transcripts and environment-version fields: not supplied.
- Classification: sufficient to resume bounded investigation; incomplete as a durable reproduction record.

### Blocks first vertical-slice design freeze

- Reliable process-tree ownership and cancellation primitives.
- Direct-MCP secret-boundary evidence.
- MCP selection result if MCP is to remain the primary UX.
- Unix executable-resolution behavior and policy.
- Windows repository-local `.exe` substitution behavior under the proposed resolver.

### Does not necessarily block a first core slice

- Perfect MCP selection reliability, if the design freeze explicitly treats MCP as experimental and CLI as the initial controlled adapter.
- Artifact completeness for .NET and Maven.
- Global caching policy.
- Long-term evidence persistence.
- Full large-output spool policy, provided the first slice uses an explicitly bounded experiment-only fixture and makes no production claim.

### Later concern

- Hook-input completeness at very large successful-output sizes.
- Cross-producer artifact unification.
- Additional supported toolchains beyond Cargo check.
- Provider-egress integration.
- Result caching and replay.

---

## 13. Implementation Authorization Recommendation

**DO NOT IMPLEMENT — BLOCKING EXPERIMENT REMAINS**

The baseline blocker is cleared by the owner's green-gate report. The immediate architecture blocker is process-tree ownership across Windows and Unix. Live MCP secrecy and selection evidence also remains required for the proposed MCP-primary interface.

The next stage is not production implementation. It is:

```text
green baseline (owner-reported)
→ semantic-value falsifier: survived bounded fixture
→ Windows Cargo resolution falsifier: survived with absolute-path requirement
→ parser-truthfulness falsifier: survived bounded fixture
→ run cross-platform process-lifetime falsifier
→ stop immediately on any kill condition
→ run secrecy/MCP-selection gates only while relevant
→ return for architecture review
```

---

## 14. Proposed First Slice

No implementation slice is authorized by this report.

If the architecture survives the resumed falsification phase, the candidate for design freeze remains Cargo check with:

- a closed operation;
- direct trusted Cargo resolution;
- no shell;
- Cargo JSON evidence;
- authoritative process outcome;
- producer-specific parsing;
- bounded sanitized unparsed evidence;
- CLI and typed MCP renderings over one result.

Whether the slice should canonically return semantic facts or disciplined filtered text depends on the semantic-compilation experiment. Whether MCP is the preferred first adapter depends on the selection and secrecy experiments.

---

## 15. Deferred Experiments

| Experiment | Status | Why deferred |
|---|---|---|
| Semantic compilation versus filtering | Completed bounded synthetic probe | Kill condition not met; real/model validation remains later |
| Executable resolution | Windows Cargo probe completed; Unix and spoof `.exe` deferred | Cargo survives with mandatory absolute-path policy |
| Process lifetime | Required before design freeze | Can kill the execution core; needs OS-specific lifecycle fixture |
| Parser truthfulness | Completed bounded synthetic probe | Kill condition not met; streaming production behavior remains later |
| Secret timing | Required before claiming MCP advantage | Needs live investigative MCP path and inspectable Claude surfaces |
| MCP selection | Required for MCP-primary UX | Needs live Claude repetitions; failure demotes interface rather than necessarily killing core |
| Hook-input completeness | Later | Does not decide the proposed primary core while fallback behavior is already bounded truthfully |
| Large-output memory/spooling | Later design-freeze concern | Global limits require separate approval; not needed to test semantic value first |
| Artifact completeness | Producer-family phase | Relevant to .NET/Maven, not the Cargo-check first boundary |
| Broader context economics | Later | The bounded semantic comparison supplies the first decision-quality signal |

---

## 16. Baseline Record and Remaining Owner Handoff

The owner reported all baseline gates green. The exact logs and version record below remain desirable for durable reproduction, but their absence no longer blocks the bounded synthetic investigation recorded here.

Run the authoritative commands from repository root at the exact commit intended as the experimental baseline:

```powershell
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets --all-features
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/tests/check-file-sizes.tests.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
cargo test --all-features encoding
```

Return:

```text
commit:
branch:
working tree before gate:
operating system:
rustc version:
cargo version:
Claude Code version:
relevant producer versions:
Clean-CTX configuration identity:
command 1 outcome:
command 2 outcome:
command 3 outcome:
command 4 outcome:
command 5 outcome:
command 6 outcome:
known pre-existing limitations:
timestamp:
```

Do not report the baseline as green if any command failed, was interrupted, or was not run. Once the complete record is available, resume from the semantic-compilation falsifier rather than mechanically executing experiments in their original numbering.
