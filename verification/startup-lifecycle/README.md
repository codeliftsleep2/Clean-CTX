# Issue #144 bounded startup/lifecycle acceptance

This operator workflow verifies that optional CBM admission no longer delays
Clean-CTX MCP readiness and that reconnect/host exit do not retain an abandoned
Clean process or directly owned CBM frontend. It does not run graph queries or
semantic verification.

## Build and configure

Build the current all-feature release binary using the repository's normal
user-owned build command:

```powershell
cargo build --release --all-features
```

Configure the host's Clean-CTX MCP environment with an absolute trace path that
contains the literal `{pid}` placeholder:

```text
CLEAN_CTX_STARTUP_TRACE_FILE=C:\path\to\target\issue-144-startup\startup-{pid}.jsonl
```

Create the parent directory before launching the host. Tracing is completely
disabled when the variable is absent. Each process writes one JSONL file, so a
failed Clean A and successful Clean B cannot interleave records.

## Verify once

1. Ensure no intentionally retained Clean-CTX session is running.
2. Start the application and confirm its initial Clean-CTX MCP connection
   succeeds without the former approximately 30-second timeout.
3. Immediately capture the connected process snapshot:

```powershell
pwsh -NoProfile -File ./verification/startup-lifecycle/Capture-Issue144Processes.ps1 `
  -OutputPath ./target/issue-144-startup/processes-after-connect.json
```

4. Use the application's normal manual reconnect once. It must reconnect
   successfully without leaving two Clean processes alive.
5. Capture the second snapshot:

```powershell
pwsh -NoProfile -File ./verification/startup-lifecycle/Capture-Issue144Processes.ps1 `
  -OutputPath ./target/issue-144-startup/processes-after-reconnect.json
```

6. Exit the application normally and capture a final snapshot using the same
   script with `processes-after-host-exit.json`.

After evidence capture, clean up any unexpected retained process through the
normal host/process-management path and report the failed acceptance condition.

## Evidence interpretation

- `clean_initialize_response_flushed` must precede `cbm_spawn_begin` and
  `cbm_admit_initialize_begin` for an auto-launched provider.
- The initial host connection must succeed even when CBM admission remains in
  progress after the initialize response.
- `stdin_eof` followed by no `process_exit` identifies Clean-side shutdown work.
- A reconnect snapshot containing two Clean processes fails lifecycle
  acceptance.
- `stdout_error` proves Clean learned about output transport loss but currently
  did not make it terminal.
- After host exit, no Clean process from the session may remain.

Generated JSONL and process snapshots belong under `target/**` and are field
evidence, not tracked tests or CI results.
