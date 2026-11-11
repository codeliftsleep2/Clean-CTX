# PCG-175 M2a-1 operator verification

Agent: **Agent MaxHeadRoom**

This definition exercises the actual `clean-ctx pcg import` CLI in fresh
processes. It creates only synthetic local Git repositories, reviewed fixture
profiles, manifests, derived SQLite files and receipts under `target/pcg-175/`.
It does not start MCP, enroll the AF-PERF production corpus, access a real user
database, fetch Git objects or create runtime approval authority.

Run from the repository root after the focused tracked tests:

```powershell
cargo build --all-features --bin clean-ctx
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/pcg-175/run-cli.ps1 -Binary ./target/debug/clean-ctx.exe -BaseRecords 1 -SingleCase
```

These are **user-run commands** under AGENTS.md Rule #1. No execution is claimed
by checking in this definition. The binary suffix/path can be adjusted by the
operator for a non-Windows build.

`-SingleCase` runs only the requested `BaseRecords` population; the command above
uses one record, one shared evidence revision and one distinct current source.
Without that switch, the existing N/2N/4N mode remains available (default 8/16/32)
for separately authorized scaling work. It is not part of this acceptance handoff.
For each selected population the driver invokes
initial import, idempotent reimport, explicit rebuild and read-only full audit in
separate processes. It requires identical generation identities within that
population, correct idempotency/source accounting, unchanged main-DB bytes across
audit and unchanged binary bytes at its two observations. This is not a continuous
binary/file snapshot guarantee. Each saved profile pins that case's exact raw
manifest digest; fixtures never change an owner profile.

Each native phase retains its executable/arguments, exit code (null if no process
started), stdout, stderr, parsed result when available, and success/failure status
under `n-<size>/phases/`. Nonzero exit, launch failure or invalid CLI JSON retains
the receipt before the error propagates. Completed phases survive a later failure.
The final `operator-receipt.json` is written on normal success or exception and
identifies the selected populations, binary and completed case reports. These
files do not claim crash-durable logging if the driver itself is forcibly killed.

`operator-receipt.json` retains each completed case's actual
counts, durations and DB/WAL sizes. CLI durations exclude final output; operator
process duration includes startup, output and shutdown. The footprint reported by
writers is sampled after commit while their connection is open. It is not peak
disk usage. No enterprise speedup, allocation saving, peak-memory measurement or
statistical latency advantage follows from these tiny synthetic fixtures.

This is **operator CLI verification**, separate from tracked Rust test coverage.
One successful fixture proves only the four-phase CLI lifecycle exercised; it
does not independently establish the security or crash invariants covered by the
previously owner-reported focused tests. Binary hashes identify executable bytes;
the owner-run build evidence must separately bind those bytes to final source.
The 17 registered cases under `src/tests/pcg/` include storage subprocess crash
and reopen controls; they do not substitute for this real clap/main path. There
are no ignored required tests or new CI jobs. The authoritative final gate remains
[docs/agent/verification.md](../../docs/agent/verification.md).

For actual enrollment, the local owner must supply a private profile and review
the exact manifest/digest under the [approved design](../../docs/agent/PCG_175_DESIGN_FREEZE.md).
Neither the driver nor this fixture is the owner-approved A-E pilot. No production
pilot manifest, digest or profile has been invented by this campaign.
