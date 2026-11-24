# E6-B internal integration verification

Agent MaxHeadRoom. This operator harness exercises the opt-in internal Rust
operation, not MCP, canonical publication, persistence or a proprietary workspace.
It does not establish field acceptance for #190/#191.

Run from the repository root, with explicit owner authorization for these runs:

```powershell
$env:OPS_E1_RUN='e6b-build-REVIEW-RUN'
python verification/ops-r1/e1/measure.py build
$env:OPS_E6_BUILD=$env:OPS_E1_RUN
$env:OPS_E6_RUN='e6b-verification-REVIEW-RUN'
python verification/ops-r1/e6b/run.py tests
python verification/ops-r1/e6b/run.py controls
python verification/ops-r1/e6b/run.py red
python verification/ops-r1/e6b/run.py lint
python verification/ops-r1/e6b/run.py measure
python verification/ops-r1/e6b/audit.py $env:OPS_E6_RUN
```

Use fresh names. The deliberately failing RED subprocess must exit 101; the
runner checks that outcome. Every other subprocess must exit zero with registered
tests. The existing E1 launcher is reused solely for its build identity recording,
Windows Job Object ceiling, watchdog and sampling; no historical E1 campaign runs.
Bounds: test/measurement subprocesses 60 seconds and 512 MiB aggregate committed
memory; build/library Clippy 600 seconds and 8 GiB. Assignment follows process
creation, leaving a documented preassignment window. Working-set sampling at
10 ms can miss short peaks and covers the launched process, while the Job Object
committed-memory ceiling covers descendants too. Neither is allocator attribution.

`tests` includes one measurement entry point that returns without measuring when
`OPS_E6_OUTPUT` is unset. `measure` explicitly enables that entry point. Historical
E2-E5 controls include their own disabled measurement entry points; report their
pass counts separately from executed performance cases.

The build records all Rust/Cargo/build-script identities. The response-schema
oracle included by the tracked test is additionally pinned by `audit.py`, which
checks exact equality with the reviewed preparation artifacts. It validates all
actual DTOs and records every fixture file hash. Python `jsonschema` is required
for the operator audit, not the Rust product.

The malformed Node fixture is synthetic, version `0.0.0-synthetic`. It demonstrates
that an irrelevant dependency body is not traversed; it does not reproduce an
unknown field package version or explain every historical acquisition failure.
Response withholding retains established counts and predicate qualifications.
No interpretation becomes a canonical registration.

`review_diff.py` uses the frozen preimplementation snapshot under
`target/ops-r1/e6b-20261124`; it writes an independent diff, exact file identities
and an in-memory byte-roundtrip proof. It never applies a patch or resets the
worktree. Preserve those evidence files to reproduce this rollback audit.

Full repository verification remains a separate CI/owner gate. Public exposure,
server-authorized ceilings, aggregate service pressure and field acceptance need
their own authorized increments.
