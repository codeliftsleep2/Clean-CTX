# Phase 9 operator verification

This directory is an **operator-only live MCP harness**. It drives a freshly
built `clean-ctx` binary through its registered stdio MCP surface. It is not a
tracked test, not CI evidence, not RED→GREEN evidence, and never substitutes
for the regression tests under `src/tests/**` or the repository verification
gate.

The harness covers these live scenarios:

- A: physical `0x04` baseline persistence and restart restore;
- B: production `dv:2` generation, application, persistence, and replay;
- C: full-body edit fidelity and byte-exact structural `apply_edit`;
- D: deterministic prior-byte and target-byte interrupted-edit recovery;
- E: irreconcilable recovery failure isolation;
- F: registered context deletion without source deletion;
- G: observational history, statistics, and persisted-context listing;
- H: purge isolation;
- I: cross-file source isolation, including CRLF plus UTF-8 BOM bytes;
- J: legacy fallback quarantine through registered
  `inspect_legacy_fallbacks`.

## Prerequisites

Build the binary yourself first. The agent does not run builds or binaries:

```powershell
cargo build --all-features
```

Node.js and Python 3 must be on `PATH`. On Windows, the harness uses `py -3`
by default. Set `PYTHON` to another executable name if needed.

## Run

From the repository root:

```powershell
node .\verification\lifecycle\phase9\scripts\run-phase9.mjs
```

To select an exact binary instead of the freshest debug/release artifact:

```powershell
node .\verification\lifecycle\phase9\scripts\run-phase9.mjs .\target\debug\clean-ctx.exe
```

Reusable fixtures and scripts are tracked here. The harness recreates only
`target/phase9-verification/state/workspace`, uses that isolated workspace's
`.clean-ctx/persistence.db`, and prints a PASS/FAIL line for each operator
observation. A final `Operator scenarios: PASS` proves only that the operator
observed those scenarios through the built binary. Run the tracked repository
gate separately for verification.

The recovery seeder directly prepares deterministic SQLite crash states solely
so the real registered MCP recovery paths can be exercised after restart. It
does not call Rust internals and is not itself evidence of correctness.
