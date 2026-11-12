# B8 actual verification and remaining gates

Agent: **Agent MaxHeadRoom**, 2026-11-12. The consolidated owner authorization
explicitly authorized these focused tests, scoped Clippy, production build and
production witnesses. These were run by the agent. Actual commands and outcomes
are recorded in `B8_VERIFICATION.csv`; they are not a claim that the complete
repository gate passed.

## Tracked tests and compilation

29 tracked Rust tests passed: 15 document-module tests, 6 registered MCP tests and
8 separate enrolled projection tests. All used the all-feature library executable
compiled by the recorded Cargo test command. Test executable SHA-256:
`5d2dd9b7565e354ac11096f47d9a981a4897594bd81231bc3c4528801de29b63`.
Root-package all-target/all-feature Clippy passed with zero warnings.
The all-feature production binary build passed. Changed Rust formatting passed.

Full repository `cargo fmt --all -- --check` failed on the three untouched B7
files `src/pcg/mod.rs`, `src/pcg/security.rs`, and `src/tests/pcg/persistence.rs`.
Their normalized bytes equal the B7 Git blobs. They were not modified to expand
this mission. The raw formatter result is `target/pcg-b8/full-fmt-check.log`.
The complete Final Verification Gate in `docs/agent/verification.md` remains
UNRUN. This is draft delivery, not release acceptance or a green full gate.

## Actual production verification

Accepted baseline executable SHA-256:
`b81789484379020d8bb96f540058f32e34bf765e28eca645f369ea8b69e2affa`.
It reproduced the exact B7 requests before replacement: 3 ready / 7 budget-refused
searches, and 1 ready / 5 budget-refused required context responses. Original
captures are retained under `target/pcg-b8/before`; the accepted executable is
preserved as `target/pcg-b8/accepted-b7-clean-ctx.exe`.

B8 production executable SHA-256: `d6af36341dc15b96548b9c3e404e96f7af215dea082490650bc792cafb4f2dcc`.
Unchanged actual profile SHA-256: `0c1958e1302bd6d1e4a82dd644b03a2e322c687e9349bc038e2835283c1a40e9`.
No profile resource ceiling was raised. `token_budget=65536` remains in that
profile as retired compatibility metadata. Profile-byte provenance and rechecks
remain effective; changing any pinned profile bytes mid-session still refuses
for currentness, not token economics.

All 33 stdio requests completed ready/current. The ten original FTS requests are
unchanged and all ready. All six mandatory complete contexts are ready in both
formats: DOC-157 24 nodes / 25 edges / 24 witnesses; PERF-003-OVERLOAD 22 / 23 / 22;
PERF-004-DOCTRINE 17 / 18 / 17. Replay provides an additional complete historical
context, 8 / 7 / 8. Four independent format pairs reconstruct exactly, including
node/revision identities, evidence associations, relationships and witnesses.
Both formats are complete even where descriptive counts exceed the old ceiling.
No retained structural/operational limit prevented the required retrievals.

The read-only capture checker independently verifies selected FTS/get/related
membership, declared closure sets, typed edge sets and valid witness reachability.
It checks exact current filesystem source/evidence hashes and the original stored
revision bodies. It is operator verification, not tracked regression coverage,
CI, independent agent reasoning evaluation or authenticated authorization.

Final generation: `abb04e766d381da87b1dcd7ec18c7b22d6c98949276d83d523f209f0f6c5e0da`.
Membership/topology: 218 nodes / 221 typed relationships, exactly preserved.
216 current revisions are unchanged; AUTH.PCG_DOCUMENTS and AUTH.PCG_GLS advance
only through the changed controlling source. Every baseline revision remains
stored, with unchanged original body/declaration/hash. Predecessor chains retain
the original revisions. `B8_GRAPH_PRESERVATION.csv` records each member and edge.

## Repeatable operator commands

Use fresh output/report roots for new captures. The summarizer requires the
preserved B7 baseline snapshot, exact B7 requests and original B8 before capture.
Do not overwrite accepted receipts or launch independent B7 evaluations.

```powershell
pwsh -NoProfile -File verification/pcg-175/run-b8-document-regression.ps1 -OutputRoot target/pcg-b8/after-new-run
python verification/pcg-175/summarize-b8-document-capture.py --capture-root target/pcg-b8/after-new-run --report-dir target/pcg-b8/after-new-run/reports
```

The driver and checked-in JSON request sets are reusable operator definitions.
Generated requests, raw results, tokenizer counts and process identities remain
under `target/pcg-b8/after/**`. The tracked tables preserve the accepted capture's
identities and outcomes. Existing token measurements are descriptive compatibility
evidence, not a token-reduction objective or efficiency claim.

## Source and repository guards

All 36 frozen B7 package files match their recorded hashes. Actual original graph
revision bodies and current source hashes are preserved as recorded above.
Strict UTF-8/no-BOM guard passed for 2,673 text files, with no unexplained mojibake
or letter-substitution signatures. The active-file size guard passed for 31 active
text files and reported 12 untouched legacy oversized files. `git diff --check`
passed. Normal configured pre-commit guards also run at the delivery commit.

## Outstanding obligations

Run the governing full gate before repository acceptance, resolve upstream formatter
debt through its proper owner, and let CI verify the delivered stack. All 16 frozen
independent A/B runs remain UNRUN because of memory contamination; all 36 frozen
files are hash-identical. Host source-reference integration, uncontaminated evaluation
infrastructure and upstream merges remain separate. No Claude pilot, unrelated
historical producer/performance campaign, independent certification, global closure
or banking is claimed. The next representation mission requires separate owner
approval; only its source-supported proposal is delivered here.
