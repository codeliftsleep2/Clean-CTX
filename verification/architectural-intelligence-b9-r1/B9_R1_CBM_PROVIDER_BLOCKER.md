# B9-R1 installed CBM missing-project response blocker

Agent MaxHeadRoom inspected the operator-captured evidence on 2026-11-13.

## Observed boundary

The isolated tracked test in
`target/pcg-b9-r1/failure-recheck-20261113-125956-163/tests.log` failed after
the valid-project empty-result control completed. Its first missing-project
`query_graph` call timed out at the existing 30-second deadline. The test did
not skip. Clean-CTX returned the timeout rather than a false successful empty
result, but the required semantic-error response was not established.

The independent raw-provider session in
`target/pcg-b9-r1/cbm-error-20261113-131047-565/` reproduced that timeout without
the Clean-CTX bridge or Rust client. Initialization and tools/list returned;
request 3, tools/call query_graph, did not produce a correlated response within
30 seconds. Captured stderr is empty. The request uses fields advertised by
the provider's captured query_graph input schema.

Installed executable:
`C:\Users\MNasty\AppData\Local\Programs\codebase-memory-mcp\codebase-memory-mcp.exe`

SHA-256:
`7edcd3807ebcfd85ec1968985964080f2589748da2fc3c7ce9261eebab31ff04`

Exact request arguments:

```json
{
  "query": "MATCH (n:Function) WHERE n.in_degree = 0 AND n.is_entry_point = false RETURN n.name, n.file_path",
  "project": "definitely-not-a-real-project-xyz",
  "format": "json"
}
```

## Disposition and limits

The missing-project response failure reproduces outside Clean-CTX. This isolates
the observed failure to the installed provider/session path; it does not identify
whether the defect resides in the provider frontend, account daemon, database,
or another provider-internal component. No evidence supports increasing a
timeout, inventing a project-not-found error, treating timeout as an empty result,
or weakening/skipping the live regression.

The provider executable is outside this repository's repair scope. No provider
binary, account daemon, database, cache, configuration, or installed version was
modified. The diagnostic sent no index request and retired only its own frontend.
Original receipts remain preserved. A provider-side correction and an unchanged
passing live regression are required before claiming this obligation satisfied.

The original 80 non-live failures have passing focused reruns. The broadened
72-test C# suite and scoped Clippy passed in
`target/pcg-b9-r1/failure-recheck-20261113-124828-813/`. These results do not
waive the provider failure or establish a full repository gate. The next full
gate must retain this failure if it recurs, alongside all other actual outcomes.
