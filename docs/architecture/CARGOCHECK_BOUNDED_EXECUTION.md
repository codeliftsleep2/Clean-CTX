# CargoCheck bounded execution and capture

This phase continues the frozen CargoCheck vertical slice after authority and
fixed invocation/environment preparation. It introduces no CLI or MCP adapter
and makes no Claude pilot claim. The approved numeric and operational policies
remain those in `CARGOCHECK_OPERATIONAL_POLICY_CALIBRATION_2026-10-07.md`.

## Implementation boundary

`execute_cargo_check` consumes a prepared invocation and a cancellation handle.
It revalidates both admitted authorities immediately before process creation.
The executable, directory, exact two arguments, and minimal environment remain
those established by invocation preparation. No shell or caller arguments are
introduced. Windows encodes only that fixed vector for its native process API.

Linux establishes a dedicated process group before exec. Root exit is observed
with `waitid(WNOWAIT)`: retaining the root until cleanup ends prevents PID/group
identity reuse between an observed exit and a later cancellation. The controller
observes live group members through `/proc`; zombies cannot execute or retain
pipe writers and are distinguished from live descendants. Intentional group
escape and controller death remain excluded from this Linux guarantee.

Windows creates an initially suspended process with an explicit three-handle
stdio inheritance list. A private Job has kill-on-close and no breakaway flags.
Assignment must succeed before the primary thread resumes. Failed assignment
terminates the still-suspended root; failed resume terminates the assigned Job.
Root signaled state and Job active-process accounting are separate observations.
Owned handles and the Job close deterministically on all ordinary error paths.
Other operating systems fail admission rather than borrowing a Linux claim.

## Capture and cancellation

Two nonblocking readers drain stdout and stderr concurrently. Their independent
admission limits are 8 MiB and 4 MiB, including delimiters; their sum enforces the
approved noncompeting 12 MiB aggregate. Frames accept at most 2 MiB excluding the
newline. Admission exhaustion does not terminate the producer: excess bytes and
frame lengths continue to be counted while raw storage remains bounded.

Capture reports byte admissions/discards, framing limits and admission cuts,
unterminated frames, EOF observations, interrupted drains, and numeric OS read
errors. Raw bytes are never logged, traced, spooled, or returned. Complete frames
feed the existing semantic compiler; incomplete/oversized admitted frames use a
withheld-evidence marker instead of exporting a raw prefix potentially cutting
through a credential. Empty frames are counted without allocating evidence items.

A separate bounded complete-line stderr sample continues after admission stops.
Oversized or unterminated terminal samples are withheld. At drain completion the
latest eligible sample is sanitized and retained inside the existing 48 KiB
stderr category: earlier head evidence gets at most half and the terminal sample
at most half. It does not enlarge the 96 KiB shared evidence budget. Sampling
facts distinguish late samples from physical-frame parser coverage. The next
phase now accounts for all observed frames, including discarded frames; see
`CARGOCHECK_INCREMENTAL_PARSING.md`.

Cancellation is first-request-wins with separate host/user sources and a
monotonic request offset. It immediately requests SIGKILL of the Linux group or
termination of the Windows Job, without a graceful interval or intrinsic
timeout. Nonblocking readers may close before EOF on forced cancellation;
that lost capture remains explicit. Root outcome, live-descendant quiescence,
forced mechanism, and control/cleanup uncertainty are separate result facts.
Reader startup/read failures also trigger owned termination, not silent success.

## Verification status and commands

Tracked fixtures reside under `src/tests/diagnostics` and are registered through
the normal `#[path]` convention. The test executable doubles as a private
synthetic producer before libtest argument handling, allowing the same owned
execution cases on Windows and Linux without launching a shell or compiling a
separate untracked helper. Fixtures cover independent OS/Cargo outcome authority,
both-pipe floods, root-before-descendant exit, cancellation, final authority
revalidation, framing boundaries, stream admission, late stderr, and redaction.

Linux verification completed after the owner's explicit per-run authorization:

- Focused CargoCheck suite: 34 passed, 0 failed, 0 ignored, and 3,481 filtered
  out, including all 16 new execution/capture tests. The final source-state rerun
  exited 0 and finished its tests in 11.07 seconds.
- Package-scoped all-target/all-feature Clippy: exited 0 with `-D warnings`.
  An unused import and the synthetic fixture's missing descendant wait were
  corrected before the final green runs; no warning suppression was added.
- Offline dependency resolution, focused formatting, and `git diff --check`
  passed. New/materially modified source files remain below 615 lines.

The implementation is Linux-verified and prepared for branch publication so a
Windows operator can pull this phase. The owner subsequently reported both
focused Windows checks GREEN on commit `88118b5`; that is owner-reported
verification, not a Windows run in this Linux environment. The full suite was not run.
Real-workspace CargoCheck behavior and adapters remain later work. Windows code
was neither compiled nor executed by the Linux checks. The owner's Windows GREEN
report completes the focused cross-platform verification for this phase.

From the checkout on either supported platform, the focused commands are:

```text
cargo test --locked -p clean-ctx --lib --all-features -j 4 diagnostics::cargo_check::
cargo clippy --locked -p clean-ctx --all-targets --all-features -j 4 -- -D warnings
```

In this cloud machine, first activate the installed Rust toolchain:

```bash
export CARGO_HOME=/workspace/.tooling/cargo
export RUSTUP_HOME=/workspace/.tooling/rustup
export PATH="$CARGO_HOME/bin:$PATH"
cd /workspace/Clean-CTX
```

The full suite is reserved exclusively for CI. No full-suite command belongs in
this phase's agent or local handoff instructions.

## Remaining work

Incremental Cargo JSON decoding and mixed/malformed/truncated coverage accounting
are implemented in the subsequent `CARGOCHECK_INCREMENTAL_PARSING.md` phase.
The framer remains the capture boundary, not final result-budget certification.
Subsequent semantic retention, fallback evidence hardening, final result budgets,
CLI exit-code projection, typed MCP integration, and production lifecycle trace
remain separate phases. The bundled Claude pilot remains last.
