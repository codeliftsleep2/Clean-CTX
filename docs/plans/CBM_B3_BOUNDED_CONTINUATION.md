# B3 bounded typed-provider continuation

## Approved policy and actual scope

The owner explicitly approved plan §19: the existing query_timeout_ms is one
logical collection deadline, and 4 MiB is the maximum aggregate retained encoded
provider payload. Existing per-frame protection remains in place. No page-count
cap, enlarged provider output budget, new public schema or full-suite run is
introduced. A0–A5 admission, ownership and replay classification remain frozen.

Only consumed typed surfaces collect: query_graph, structural search_graph,
trace_path, and check_index_coverage over exact supplied candidate paths.
Architecture has no consumed paging contract. list_projects is raw-only and is
not implicitly crawled. Semantic search, coverage scopes and detect_changes
snapshot collection have no current typed collection consumer and are not added.
Raw proxy requests retain their requested single page and format.

## Production trace

**PROVEN BY CURRENT CODE:** existing MCP typed handlers → GraphBridge query
owner → CbmClient typed wrappers → collect → existing serialized call/recovery
owner → fixed requests on the admitted frontend. Native candidate acquisition
uses the same collector for exact-path coverage after serving-root verification.
Collected values go through the B1 decoders and B2 exactness validation before
returning to existing bridge/cache consumers. Failure returns through the
existing ToolError/Timeout/native-fallback surface, never a successful partial
Vec/table. Private page observations retain limitation fields, number of pages,
encoded payload accounting and partial failure, including after retirement.

Query and trace use only opaque provider-emitted next/next_cursor as cursor.
Query's live next_offset is never used as fallback after a stale/missing cursor.
Search uses a validated next_offset = previous offset + returned rows. Exact-path
coverage uses its independent path_next_offset with the original input set.
Semantic arguments, project and requested filters remain fixed. Column/QN rules,
totals, generation/coverage metadata and explicit project identity must remain
consistent. Duplicate query rows and cell types remain untouched. Trace traversals
retain provider ordering; only proven hop-1 edges are exposed by the existing
normalizer, without invented deeper parents.

Continuing pages must provide positive progress and a nonrepeating cursor or
advancing offset. Lower-bound/unknown totals, contradictory totals, stale cursor,
projection drift, malformed frames, unsupported continuation and unresolved
provider truncation fail explicitly. Current page_limit truncation can be resolved
through supported continuation. Output/engine/window budget truncation is retained
as failure; budgets are not raised. Terminal aggregate results still require the
B2 exact totals/relations and no unresolved truncation. Search offset exhaustion
is not a snapshot binding or all-source completeness proof.

The collector counts UTF-8 bytes of each original encoded inner JSON response
before admitting that page to accumulated payload. Decoding/merging and final
metadata are checked against the logical deadline. The collector keeps partial
metadata privately and never caches partial data as success. The existing one
retry budget is shared across pages, including entry-time half-open recovery;
replacement admission and receive waits are constrained by the same deadline.
Outside a collection, existing A single-response timeout/retry behavior remains.
No changes are made to native persistence or HydrationReport::is_complete.

## Deterministic verification

**PROVEN BY EXISTING TEST:** initial 10 tracked cbm_continuation_ cases ran on
unchanged B2 production: 9 intended behavioral failures, 1 raw-proxy baseline
pass. Regression-only files/registrations were stashed before implementation,
then restored unchanged. First GREEN: 10 passed.

Preservation checks passed: cbm_provider_state_ (23), cbm_current_wire_ (14),
cbm_a4_ (24), cbm::tests::query_wire::deterministic (16). Package Clippy passed
with all targets/features and -D warnings at the preservation checkpoint.
The expanded authorized audit ran 14 cbm_continuation_ cases: all passed,
including shared retry, zero progress, changed totals and partial evidence after
retirement. Final Clippy after audit adjustments passed with all targets/features
and -D warnings. B3 local implementation is green.
UTF-8/BOM, active-file size and whitespace checks passed.

**NOT ESTABLISHED:** pinned live paging/snapshot acceptance, Windows acceptance,
model-host consumption and B4/B5 final acceptance. Source-derived fake frontends
exercise actual client/bridge ownership, not live provider data. B5 must include
real query/trace cursors and stale-snapshot failures, search offsets, exact-path
coverage, and raw single-page versus typed collection comparison. The full test
suite remains CI-only. No #142 aggregate cold/warm fix is claimed.
