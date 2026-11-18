# B2 provider limits, project identity and index observations

Authority: Phase B continuation and current pinned upgrade plan §§17–22,33.
Pinned provider: `82b1ec908f975354ceb4bac8a8bfc594942a9bc4`.
A0–A5 lifecycle/replay policy remains frozen. No live provider or full suite
is authorized or run in this slice.

## Production integration

**PROVEN BY CURRENT CODE:** typed CbmClient calls retain separate private
observations keyed by requested project and tool. Index jobs, serving status,
coverage and query pages cannot overwrite one another. QueryRows additionally
carries its private page observation. Outer soft errors are recorded before
existing ToolError classification, preserving failed-job details and prior
serving-index evidence. Transport retirement clears observations; a new attempt
removes that tool's old observation so an RPC failure cannot promote stale
status into current evidence. No new polling, watcher or session is introduced.

Typed search/query/trace and architecture consumers validate limitations before
returning data to existing Vec/table/overview consumers. More pages, truncation,
gte/unknown totals, inconsistent counts and opaque continuation prevent a
full-looking successful result. Unknown exactness in search/query/trace is an
error rather than a successful empty result. Search's pinned structural schema
has an exact total without total_relation; query requires eq; trace uses each
requested leg's eq total against all returned traversal rows. Only hop-1 edges
are projected. Root and nested limitation fields remain private evidence.
This is single-response validation, not B3 collection: no cursor is followed.

GraphBridge startup, manual, active/non-active lazy and file-specific indexing
use a project-bound synchronous request with existing mode and explicit name.
Only `status=indexed` for that project counts as successful terminal indexing.
Running/queued/unknown/degraded replies cannot mark startup Complete or advance
dirty generations. Supplied root and configured project identity remain distinct
from serving-index generation; no successful search upgrades job observation.
A successful job is not a source-universe or native semantic completeness claim.

Project-explicit native candidate acquisition (search and inbound-reference
paths) verifies index_status's project and canonical serving root against the
existing admitted map. Coverage requests contain only the exact candidate paths,
no inferred scopes or filesystem crawl. Returned path identities/counts and
limitation metadata are validated. The provider's 128-path input contract is
honored without inventing batching or collection limits; larger input fails to
existing native fallback. Best-effort clean/metadata-match/unknown observations
are retained independently and do not certify native source adequacy. Missing
optional diagnostics fails that advisory acquisition; no new public MCP tool or
required admission-tool list is introduced.

The existing detect_changes path no longer interprets a nonexistent graph_version
as a freshness authority. It retains the existing optional None behavior; provider
diff/snapshot evidence is not a native source hash. Its request/replay shape is
unchanged under the frozen A4 policy.

Graph cache epoch advances from v1 to v2 because v1 entries can contain results
accepted before limitation validation. Old entries are ignored without deletion.
Native persistence, Binary0x04 and HydrationReport::is_complete are unchanged.

## Evidence and pending verification

**PROVEN BY EXISTING TEST:** initial 12 cbm_provider_state_ cases observed
11 intended behavioral failures and one terminal-index baseline pass on the
unchanged B1 implementation. An earlier fixture-constructor compilation failure
was corrected before observing RED and is not counted as RED. Regression-only
files/registrations were stashed for implementation, then restored unchanged.
First GREEN: 12 passed. Eight additional contract cases cover independent running/
failed observations, preserved serving evidence, retirement, unknown pages,
opaque continuation and production candidate/root/coverage paths: 20 passed.

The B1 empty-search, B1 trace and A4 normal-response fakes now include their pinned
producer totals/relations. Assertions and A4 recovery behaviors remain unchanged;
missing limitation metadata is no longer a valid exact-page fixture.

| Authorized preservation filter | Passed |
| --- | ---: |
| cbm_current_wire_ | 14 |
| cbm_b0_ | 1 |
| cbm::tests::cache_ownership | 3 |
| cbm::project_search::tests | 7 |
| cbm::tests::regression::freshness | 10 |
| cbm_a4_ | 24 |

Clippy initially found a unit-value binding left by the indexing refactor; the
binding was removed without suppressing warnings. Final package Clippy passed
with --all-targets --all-features -j 4 -- -D warnings. UTF-8/BOM, active-file
size and git diff whitespace checks also passed.

## Final audit follow-up

**PROVEN BY EXISTING TEST:** the three dispatcher regressions observed RED:
running and degraded replies incorrectly cleared cached native source; indexed
publication correctly refreshed it. Regression-only test files were stashed
before changing the callback, then restored without assertion changes.

**PROVEN BY CURRENT CODE:** raw manual MCP index_repository still passes through
cbm_proxy with its original response. The native-refresh callback now additionally
requires a parsed pinned synchronous status=indexed reply. Running, queued,
degraded, malformed and unknown results do not establish publication and cannot
trigger native refresh. Raw response format/compression remains B4 scope.

Final proxy GREEN: 3 passed. Combined cbm_provider_state_: 23 passed, zero
failed. Final all-target/all-feature package Clippy passed with -D warnings.
B2 local implementation is green. Full live acceptance remains a B5 gate.

**NOT ESTABLISHED:** pinned live B2 acceptance, Windows acceptance, model-host
consumption and the remaining B3/B4/B5 gates. Fake frontend tests trace real
client/bridge/dispatcher paths but are not a substitute for live acceptance.
The final operator harness must exercise current status, coverage, paging and
raw/typed consistency. No #142 aggregate cold/warm completion fix is claimed.
