# A3 pinned frontend admission — prepared live acceptance

Status: **NOT RUN**. These cases supplement deterministic admission fixtures;
they do not establish live acceptance until captured against the pinned artifact.
Use the existing PowerShell MCP-session pattern documented in this directory and
`verification/context-compression/scripts/McpSession.ps1`. Do not substitute
terminal graph commands for Clean-CTX's registered MCP boundary.

## Preconditions and capture

Obtain fresh authorization before builds, servers or live checks. Record the
Clean-CTX commit, provider executable absolute path, SHA-256, source revision
`82b1ec908f975354ceb4bac8a8bfc594942a9bc4`, build provenance, OS/account,
primary and additional roots, and effective canonical cache root. A packaging
version alone does not prove the artifact. Stop if provenance is unknown.
Capture bounded per-case initialize/tool responses, stderr, direct frontend
PID/lifetime, bridge status and whether Clean-CTX indexing began. Keep evidence
under `target/`; redact unrelated account/project data before sharing.

Use an operator-controlled account and repositories. Do not force activation,
change another session's cache root, kill a daemon or create a private cohort.
Each case has a 90-second observation deadline; on expiry record incomplete,
close the owned session and stop. This is a harness observation budget, not a
new production timeout or a guarantee that synchronous pipe writes are bounded.

| Case | Setup and observation | Acceptance |
| --- | --- | --- |
| Compatible existing daemon | With a provenance-verified compatible daemon, start Clean-CTX and invoke registered graph/proxy tools against the admitted fixture. | Initialization and complete minimum capabilities precede Clean-CTX indexing/tool requests; usable status and nonempty expected evidence. |
| First frontend | In an operator-owned account with no daemon, launch the verified frontend through Clean-CTX. | Provider bootstrap succeeds; same admission sequence and truthful result. No Clean-CTX account repair. |
| Second compatible frontend | Keep the first session open; admit another session with the same canonical cache root. Close the second. | Both work; closing one frontend leaves the other usable. Daemon reuse/lifetime is observed, not inferred from PID ownership. |
| Incompatible startup refusal | Only in an isolated operator-controlled account, use a deliberately incompatible provider/cache setup approved separately. | Refusal cause reaches degraded status; no Clean-CTX background indexing or successful-empty provider result; unrelated compatible session is unaffected. |
| Frontend exit during admission | Terminate only the recorded direct frontend before admission completes. | Unusable status, explicit error and direct-child retirement. If a structured refusal was emitted, preserve its cause. Never kill the shared daemon. |
| Canonical root behavior | Capture effective cache/project roots with the normal account configuration and distinct repository roots. | Root admission is consistent with provider canonicalization; no automatic cache override; project requests route to the intended roots. |
| Exact artifact provenance | Reconcile executable hash and trusted build/source evidence with the pinned source before interpreting the above. | Provenance established externally; initialize packaging version is not claimed as build/ABI/cache proof. |

Required registry: `index_repository`, `search_graph`, `query_graph`,
`trace_path`, `get_architecture`, `list_projects`, `detect_changes`.
Presence/basic schema validation does not establish Phase B argument/result
compatibility, watcher freshness, cold/warm completeness or daemon recovery.
