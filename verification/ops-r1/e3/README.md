# E3 request-local reservation and cancellation increment

Agent MaxHeadRoom. The API is nested under the existing opt-in
`clean_ctx::ir::source_observations::request_budget` module. It has no public tool,
canonical projection, persistence or default-profile integration.

All request clones share one ledger. Cost dimensions are cumulative source/support
bytes, units/edges, validation bytes/checks and response bytes; retained bytes and
active operations are live reservations. Optional work cannot spend unused protected
finalization capacity. Cancellation blocks subsequent optional work, while independent
finalization still requires its own remaining time, costs and currentness proof.
Concurrency denial returns immediately; there is no hidden retry, global thread pool
or service policy. Callers decide whether to stop, queue or retry at a later boundary.

`run_optional` reserves before invoking the closure and provides that same request
to nested work. Its returned `Reserved<T>` keeps the declared memory charged until
the result is dropped. `map` transforms a result without releasing its reservation.
Do not clone underlying allocations, drop permits early or create a new Request to
evade accounting. Operations must declare accurate costs and checkpoint between
bounded quanta. This is a cooperative API, not allocator/filesystem interception.

The E2 API remains available unchanged, so callers bypassing the new wrapper also
bypass its accounting. `Request::admit` reserves expected validation costs; callers
must reserve capture and retained costs before producing the candidate. Validation
may overrun expected byte cost if source grows concurrently; E2 rejects the changed
hash. No claim of hard process or actual heap bounding follows from the ledger.

Reproducing the final authorized local run from PowerShell 7:

```powershell
$env:OPS_E1_RUN = 'e3-build-r4-20261123'
python verification/ops-r1/e1/measure.py build
$env:OPS_E3_BUILD = 'e3-build-r4-20261123'
python verification/ops-r1/e3/measure.py tests
python verification/ops-r1/e3/measure.py controls
python verification/ops-r1/e3/measure.py perf
python verification/ops-r1/e3/measure.py lint
python verification/ops-r1/e3/analyze.py
```

Recorded output paths are immutable; future executions need fresh paths in the
runner and corresponding analyzer. Builds/lint use 600 seconds and 8 GiB aggregate
job committed memory; focused tests use 30 seconds/512 MiB; performance children
15 seconds/512 MiB. Existing external capture has a post-spawn assignment window;
these limits are experiment controls, not product defaults. Check each receipt's
child exit code rather than relying on the Python wrapper's exit status.

Performance work uses explicitly sized synthetic buffers and a two-millisecond
non-interruptible quantum, not TypeScript closure. One and two workers share exactly
64 KiB/eight optional units, following a separately charged source capture and
independent E2 admission. Deadline overshoot is measured with a ten-millisecond
experiment deadline and a 25-millisecond quantum. Cancellation is injected during
an active operation using barriers. Those durations are experimental stimuli, not
latency objectives or calibrated production budgets. No field acceptance is claimed.
