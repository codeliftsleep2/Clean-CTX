---

## [0.5.2] - 2026-08-29

### Fixed

- **CBM transport pipeline deduplication — unified write/read/flush/timeout/disconnect state machine.** Two separate implementations of the same low-level pipe I/O loop existed in `call_tool_inner()` and `call_tool_raw_inner()`, causing stale-pipe divergence, retry inconsistency, and dead-subprocess recovery asymmetry. Extracted the shared transport into `send_and_receive_raw()` — the single method both entry points delegate to. `call_tool_inner()` remains the parsed-`Value` semantic entry point; `call_tool_raw_inner()` remains the raw-response-text entry point. Retry loops in `call_tool()` / `call_tool_raw()` are unchanged. (`src/cbm/client.rs`)

---

