# Acquisition verification checkpoint

Agent: **Agent MaxHeadRoom**. Focused tracked tests, production acquisition and
the bounded actual-host workflow are verified. Final evidence changes await commit;
CI and the external CBM blocker remain separate. Latest production/host receipts:
`target/pcg-b9-r1/delivery-refresh-20261113-134605-318` and
`target/pcg-b9-r1/host-refresh-20261113-142039`. The earlier checkpoints below
retain the original RED/GREEN provenance.

Current update: verification-02 establishes tracked GREEN (2 original regressions,
39 document tests and 10 registered MCP tests), scoped zero-warning Clippy and
all-feature build. production-01 verifies all 326 explicit selected references,
430 requested/qualification occurrences, individual acquisition, ordered duplicate/
invalid outcomes, exact historical components/full source and stale-current refusal.

Original tracked reference-first run compiled and failed both tests at the intended
assertion: graph retrieval must not acquire component text. Receipt:
target/pcg-b10/b9-r1-reference-red.log. Exact regression file SHA-256:
fbe348e3d0ccdb200d300fd5374cbcc66c598d64e4dfcd7bedd092cbf74e21e2.
Narrow stash 458ea8e81392f3e66fab7aa5d11874d5d3ef4620 preserved that file and
parent registration before the production projection change, then restored them.
Additional existing parent tests are subsequently migrated to explicit acquisition;
the two observed RED assertions/file remain unchanged.

New supplemental tracked tests cover selected content/qualification bytes,
ordered batches, duplicate behavior, invalid exact descriptors, source drift,
retained historical components/full source, invalid current declaration versus
history, malformed batches and deterministic shared-source work counts. Existing
search/closure/currentness/rollback/no-token-limit/source tests remain and their
text assertions now explicitly acquire content where appropriate. Registered MCP
and flat descriptor tests are updated for structural get and explicit acquire.

All executions above were user-owned. The agent performed static receipt/source
comparison, not test/runtime execution. Full repository verification and the
representative actual-host graph-to-acquisition witness remain separate obligations.
