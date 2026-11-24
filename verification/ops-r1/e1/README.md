# E1 stage-cost experiment

Agent MaxHeadRoom. Measurement-only authorization for issues #190/#191.
No public v2 query, production change, persistence migration or field acceptance.

The Rust experiment definitions live in `src/tests/ir/ops_r1_e1_measurement.rs` and
`ops_r1_e1_metrics.rs`. Production-file instrumentation is exclusively cfg(test).
The runner requires Windows Job Objects; no third-party Python package is needed.
New files and instrumentation remain unstaged and uncommitted for owner review.

From the repository root, use PowerShell 7 and a fresh evidence directory name:

```powershell
$env:OPS_E1_RUN = 'e1-review-20261123'
python verification/ops-r1/e1/measure.py build
python verification/ops-r1/e1/measure.py controls
python verification/ops-r1/e1/measure.py matrix
python verification/ops-r1/e1/measure.py regressions
python verification/ops-r1/e1/measure.py cold-controls
python verification/ops-r1/e1/analyze.py
```

Execution requires authorization under AGENTS.md. These are the commands used
with `OPS_E1_RUN=e1-r2-20261123`, not commands newly requested for the owner to run.
The runner refuses source/executable drift before matrix execution. Existing
fixtures and run directories are never overwritten. The build command is
`cargo test --lib --all-features --no-run --message-format=json`, bounded by
600 seconds and an 8 GiB aggregate job committed-memory ceiling.

The matrix is four dependency environments times two sizes times three deadlines
times two worker counts: 48 isolated libtest processes and 72 worker observations.
The larger size contains 256 synthetic 8 KiB declaration units. Application and
configuration bytes are identical across all environments. The synthetic package
named @types/node has version `0.0.0-e1-synthetic`; it is not an exact Node grammar
reproduction. Deadline values 1/2/5 seconds are owner-selected experimental probes,
not calibrated product requirements. Matrix child watchdog is deadline plus two
seconds; every test child has a 512 MiB job committed-memory ceiling.

Raw output: `target/ops-r1/<OPS_E1_RUN>/`. Each run contains command/limit receipts,
working-set samples, stage counters and evidence, acquisition-start markers and
completed results. Manifests pin application/configuration/dependency bytes.
Build identity pins the archived executable and instrumented source manifest.
Watchdog termination is not semantic RED. Inspect each receipt: the Python wrapper
can finish normally after terminating a test child; its exit status alone is not
a passing test result.

Important limitations: elapsed budgets are checked between stages; blocking work
can overrun. Job assignment and watchdog installation occur after process spawn;
startup is not covered by the same hard boundary. Report measured launch/termination
overshoot. Working set, committed memory, cached source bytes and serialized witness
bytes are different quantities. The prototype does not enforce request byte budgets,
produce canonical registrations or publish into any production owner/index.
Its local observation recognizer supports only the deliberately narrow fixture
shape; it is not a generally sound route-recognition replacement.

The first pass is preserved at `target/ops-r1/e1-20261123`; its
`retained_source_bytes` field incorrectly counted hash text. Do not use that field
for memory attribution. The corrected pass is `e1-r2-20261123`, including a shared
request clock and measured active-stage peak. Do not blend their measurements.
