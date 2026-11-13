# Graph preservation checkpoint

Agent: **Agent MaxHeadRoom**. Baseline: 218 identities, 221 typed edges,
326 components, generation 80c1195a7a9df253e4221635bd3feb8201d15ac4e0fbc328425e59cbe04b03e5.
Exact baseline descriptors, source bytes/hashes and edges remain captured under
target/pcg-b10/baseline.json and original production/host packages.

Actual production-01 verifies preservation of those identities/edges/components,
all metadata, exact unaffected source revisions/boundaries and evidence association
membership. All eleven accepted closure memberships/edges/witnesses are unchanged.
Only these source-owned revisions advance for authorized contract/workflow edits:

| Identity | Previous revision | Current revision |
|---|---|---|
| AUTH.PCG_DOCUMENTS | ad66a89666dc743121558c7ecf8b51adfb12b5893549e2221e2e2890f217a09e | 4a91082a514377408cf126819c387263d1d76dd1a90139bbc6791f18d235779a |
| AUTH.PCG_GLS | a89faa37f12a89c251363f7f76c5f3a8c868d9ef14da91bf8c2035c3dc06a2e8 | d605ac3a0762d19d8af49b1ad8f5a59581ca8da493681333c3124dba41380bc7 |
| AUTH.PCG_PREAUTHORING | e76250f95b5f65a7a9b13c6de1244107b916cd6574d1c1ebe1a976f47f97f7b0 | c2f65671e8b5d5788c302f808a93a08fffaa8d5543a4f92306d63d352b572e3b |

Generation is 684f1ca847aaec8398506ca3e8d0ebd3bf8f5bbaacb26ba88ede062fb17c6d70.
Their old pinned components still reconstruct byte-exactly; original whole
authoring source hashes to its original baseline SHA. This verifies those retained
historical witnesses, not an exhaustive audit of every old database publication.

No schema migration, identity deletion, relationship change, selector change,
component taxonomy or FTS semantic change is implemented. get/context replace
content projection with the existing descriptor rows; traversal remains unchanged.
Historical document_source/document_revision/member/publication rows are retained.

Two governing/workflow source files are intentionally amended, so their source
revisions and any actual source-attributed evidence dependencies may advance.
Actual-root reconciliation and the identity/edge/component plus selected history
comparisons passed as reported above. This is not an exhaustive database history
audit. Frozen B7 files and all accepted B10
captures are not changed or reused as proof of the corrected workflow.
