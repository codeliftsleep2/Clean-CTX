# ANG-R2 production acceptance

Agent: **Agent MaxHeadRoom**. **UNRUN.** No current executable acceptance claimed.

The existing production knowledge executable remains the historical pinned binary
4e3da8ee0b160938a238fc273271ce24d1574348c60d16b5b047ae936260387f.
Its successful governed knowledge publication is separate from public resolver
registration capability. Library test compilation does not rebuild or verify the
production executable and cannot substitute for the required production witness.

After current focused verification passes, prepare the established issue-162
production-boundary witness against a separately authorized current executable
build. Record exact executable/source/configuration/fixture identity and raw stdio
receipts for cold forward/reverse records, keys sharing a target, exact ownership,
provenance, unchanged membership, refusal/siblings, replacement/removal and replay.
Full repository gates remain CI-owned. Claude-only testing remains the last field
gate after batching local deterministic and production-stdio evidence; no environment
switch or Claude pilot is requested at this stage. No runtime or build permission
is inferred from implementation authorization or an earlier operator command.

## Verified local build checkpoint (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/registration-build-20261115-103503-868`: scoped all-feature library/binary/test Clippy passed (66.4665054 seconds), all-feature production build passed (90.0229315 seconds), strict UTF-8 guard passed (29.0891902 seconds; 2869 files, zero BOMs), active-file guard passed (4.2010105 seconds; 48 active files, 12 untouched legacy oversized files). Cargo emitted no warnings. The file-size guard's stderr contains existing Git CRLF-to-LF notices, not compiler warnings or file-size violations. All receipt/log hashes, before/after source manifests and current source pins match.

Built executable: `target/debug/clean-ctx.exe`, SHA-256 `5d00fea65d44bf7522d123e10a543b46a92842c77a1e60caee918605ec1549bd`. A successful build establishes no runtime acceptance. The earlier historical knowledge executable identity is superseded only for this build checkpoint.

The new reusable `registration-stdio.mjs` is UNRUN. It reuses the established McpClient transport and synthetic package declarations, checks the exact successful build/source pins before and after, and captures raw RPC/stderr and hashed fixtures under `target/ang-r2/registration-stdio-*`. Three sessions cover independent cold forward/reverse roots plus durable restore, key-only/target replacement, root/source retirement, owner relocation and recreation, exact source/key coordinates, refusal sibling preservation and old unqualified membership/component edges. Runtime results remain pending. This harness is operator production-boundary evidence and cannot replace tracked Rust tests or CI. It does not establish edit/delta/replay/reset or additional-root coverage beyond its explicit scenarios.

Next operator command from repository root:

```powershell
node verification/live-acceptance/issue-162/ang-r2/registration-stdio.mjs target/ang-r2/registration-build-20261115-103503-868
```

This starts the pinned executable for three local synthetic stdio sessions; it performs no build and requests no Claude environment switch. JavaScript syntax and bounded encoding/whitespace inspection passed before handoff; the agent started no runtime. Full CI and remaining acceptance obligations stay separate.

## Production stdio failure retained (2026-11-15)

Agent MaxHeadRoom reviewed `target/ang-r2/registration-stdio-2026-11-15T15-43-41-046Z`: FAIL, 181 passing checks and one failing check (182 total). Independent cold forward/reverse sessions passed. All three server processes exited zero. The restore/mutation session is incomplete because `owner-recreated/unqualified-membership` failed. Current executable and harness-input hashes match the report.

Public registration checks passed cold/warm retrieval, durable restore, key-only replacement, target remap, root retirement, owner relocation, source retirement and recreation, including exact evidence/current target hashes. After deleting moved.ts and recreating one.ts, the separate old forward_edges query returned two ResolvedBy edges plus one RouteMapsTo edge from each owner, including the deleted moved.ts. The preceding registration response correctly returned only three records from one.ts. Do not relabel the overall harness PASS or weaken its old membership assertion.

Source trace: the new operation consumes its explicit current filesystem nominated-owner set; old adjacency queries consume an indexed bucket. Existing candidate eligibility excludes absent files, so acquisition alone does not retire their prior indexed rows. This identifies the differing consumption paths, not a demonstrated pre-ANG-R2 baseline regression or an approved global lifecycle repair. The old edge query source and shared candidate eligibility are unchanged by ANG-R2.

A tracked legacy-only control is now authored in src/tests/mcp/angular_deleted_owner_membership_issue_162.rs, registered in the existing test module. It uses only old forward_edges queries through initial acquisition, relocation, deletion and recreation, never resolver_registrations. It is UNRUN. Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-deleted-owner-regression.ps1`. Preserve either result, including failure; no production code or frozen witness changed. This test-only addition does not require a new production build yet.

CBM remains degraded and this host advertises no pcg_knowledge_v1 tool; architectural graph coverage is unavailable, not repaired or inferred. Any repair that changes shared cross-query deletion/retirement policy needs the applicable owner architectural disposition. Actual production acceptance remains blocked pending this finding's resolution/disposition, with CI and other lifecycle/field obligations separate.

## Confirmed tracked RED and shared-lifecycle decision point (2026-11-15)

Agent MaxHeadRoom verified `target/ang-r2/deleted-owner-regression-20261115-105355-728`: compilation succeeded without warnings; one tracked test ran and failed at the deletion assertion (line 34), exit 101. Enclosing duration 63.1784525 seconds; test duration 6.86 seconds. Receipt log hashes match. The relocation count assertion passed, but it asserted only count, not relocated provenance; do not claim relocation correctness from it. The recreation assertion was not reached.

After moving one.ts to moved.ts and deleting moved.ts, the old forward_edges query still returned one ResolvedBy edge asserted from now-absent one.ts. No resolver_registrations query occurs in this test. This establishes a current legacy-path behavioral RED independent of invoking the new public operation; it does not establish that the defect predates ANG-R2, since the producer/publication internals are the current implementation. The actual stdio capture separately established stale moved.ts membership after recreation.

Architectural decision requiring owner disposition under AGENTS.md Architectural Approval Gate:

1. Decide whether ANG-R2 includes correcting deleted asserting-source retirement at the existing shared semantic publication/index lifecycle boundary. No implementation choice or permission is inferred from the failure.
2. ANG-R2 preservation verification now fails and its tracked regression is RED. New registration output alone cannot establish mission acceptance while that failure remains unresolved.
3. A shared-boundary repair can affect forward/reverse edges and other indexed semantic consumers across query types. Persistent/session ownership, cache behavior and disappearance versus transient acquisition failure must remain explicit; no new store, global cache or broad policy is proposed.
4. Alternatives: authorize a bounded shared-boundary repair with focused deletion/recreation/sibling tests; authorize a narrower existing boundary if investigation proves equivalent ownership/currentness guarantees; or hold ANG-R2 and handle the defect separately. Holding is not a waiver, green gate or completed delivery. A resolver-only response filter is not recommended because it leaves shared lifecycle authority inconsistent.
5. Recommended: authorize investigation and the smallest correction at the existing shared lifecycle boundary, limited to confirmed deleted asserting owners, preserving source identity, supported in-memory/session sources, failure distinction, unaffected siblings and existing lifecycle semantics. Any new cross-cutting policy discovered during that work must return for a separate disposition. Preserve this tracked RED and rerun it after correction, then batch affected controls and production stdio. Do not reopen accepted P0 batching or unrelated optimizations.

No further test run or production modification is requested before this disposition. The frozen witnesses, failed receipts and admitted authority candidates remain unchanged. Host PCG coverage remains unavailable as previously recorded; no graph or baseline approval is inferred.

## Focused shared repair selection GREEN; production handoff (reviewed 2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/retirement-green-20261115-115618-261`: six new invocations passed, selecting 33 tests (hard links 3, resolver registrations 17, old membership 6, original collision 1, historical replay 5, failed-publication retry 1). The batch correctly retained the exact frozen retirement witness (1) and shared retirement controls (9), giving 43 passing tracked tests across the pinned captures. No compiler warnings were found. All six current log hashes and both prior receipt references match. The captured before/after source manifests are identical. An agent attempt to refresh every current source hash became unexpectedly slow and was stopped; a new current per-file source-hash verification is not claimed from that interrupted command. The next operator build batch captures current source pins separately.

This establishes the named focused repair/preservation results, not current executable behavior, CI, field acceptance or governed admission. The prior production executable/failed stdio captures predate the shared repair. No producer/schema/version or approved resolver contract was changed by this verification checkpoint.

Next operator command:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-build-stdio.ps1
```

The wrapper delegates to the existing scoped all-feature Clippy/build/UTF-8/file-size capture, requires exactly one newly completed build receipt, then runs the existing pinned synthetic three-session registration stdio witness with that build. It performs one production build, preserves raw logs and hashes, and stops on any failure. The stdio witness must resolve the earlier deleted-owner membership failure, not weaken it. PowerShell syntax and bounded UTF-8/whitespace inspection passed before handoff; no agent Cargo, guard or runtime was started.

After actual production results are reviewed, substantive findings, owner disposition and exact verification associations still require the appropriate governed document, actual admission checker and resolution of applicable blockers before push. Protected grants remain owner-only. CI and any applicable final field gate remain separate; no Claude switch is requested now.

## Current production boundary verified; governance candidate prepared (2026-11-16)

Agent MaxHeadRoom verified `target/ang-r2/registration-build-20261116-093721-400`: scoped all-feature library/binary/test Clippy passed (200.996687 seconds), production build passed (149.4825576 seconds), UTF-8 guard passed (40.4572944 seconds), active-file guard passed (4.8517157 seconds). No compiler warnings. Build raw log hashes and unchanged captured source manifests match. Source manifest SHA-256 is `25b8e03826e8bab856d21d24a596f9178f83904627aa53091491842e5c088f56`, the same pinned focused-test source.

Current executable SHA-256: `93f1431f8be130ae2f764b31a5d6e0d579955c8274c0d0b207d54ee20bdc0ffa`. Production stdio `target/ang-r2/registration-stdio-2026-11-16T14-44-02-327Z` passed all 185 checks in cold-forward, cold-reverse and restore-and-mutate sessions; all three processes exited zero. Exact trace/log/report/binary identities are retained. The formerly failing legacy membership check after deleted-owner recreation passed. Wrapper `target/ang-r2/retirement-production-20261116-094400-628` exited zero in 31.4023024 seconds; its raw log and binary-manifest hashes match. This establishes those production scenarios, not CI or Claude field certification. Historical failed stdio and regression receipts remain unchanged.

The existing governed evidence source `docs/agent/ISSUE_162_ROUTE_RESOLVER_ACCEPTANCE.md` (DOC-270) now has a dated continuation for the exact finding/root causes, owner disposition 02, shared implementation/affected consumers, bounded-work declaration, exact RED-to-GREEN and 43-test/185-check receipts, and acceptance limits. Historical generation-78 prose remains intact. Source-owned knowledge components and typed dependencies bind these substantive sections to the existing ownership/publication/persistence constraints. No new governed corpus identity, protected selection or membership was created. This is a candidate, not claimed admitted knowledge.

Next operator command: `pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-retirement-governance-review.ps1`. It pins the current executable/profile/source/predecessor, runs actual Rust document describe validation and the normal knowledge-context admission checker, and captures complete diagnostics/descriptor. It neither creates nor submits a protected grant. If protected authorization is required, the exact returned descriptor must be reviewed before the owner is asked to authorize its Rust digest. No final publication/push, current governance acceptance, CI or field success is claimed.
