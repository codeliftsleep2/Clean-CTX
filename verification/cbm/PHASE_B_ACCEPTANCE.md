# Phase B live MCP acceptance coverage

Run `scripts/Invoke-PhaseBAcceptance.ps1` using the README command. This is an
operator verification harness over actual production binaries and registered
MCP tools, not a mock, terminal Cargo test or Codex/Claude selection pilot.
B1–B4 deterministic regressions protect internal contracts; the harness checks
whether their observable consequences survive the real provider/adapter boundary.

| Phase | Live cases and assertions |
| --- | --- |
| B1 wire contracts | Actual provider and Clean tools/list; indexed identity; grouped QN/file → typed graph_search; valid empty search; consumed architecture table counts → existing modules/dependencies |
| B2 independent state/identity | Index job succeeded separate from serving ready and best-effort coverage; source-root binding; queries do not change the observed job; distinct additional-root identity and explicit primary routing; same-named C# interface/class files in both roots remain reachable through native context |
| B3 collection | 225 generated functions across nine TS files: provider query cursor oracle at 17 rows/page versus typed graph_query; default-limit typed search reaches every expected name/file on first and cached query; 225-call cross-file hub establishes direct cursor pages and a typed target selected beyond the default first page; deeper-parent invention remains pinned by deterministic tests |
| B5 native preservation | Explicitly disabled provider; high-fidelity C# and TypeScript context remains reachable through live MCP |
| B3/B4 snapshot errors | A real emitted query cursor becomes stale after reindexing only the owned fixture; direct provider and raw proxy both retain the error |
| B4 proxy truth | Single raw page versus direct provider exact JSON facts; current bridge availability; intentional tree whitespace; errors remain errors; private index_status remains excluded from the public proxy whitelist |

The source manifest is an independent oracle for name/file/count truth. The direct
provider is additionally observed for canonical identities, schemas and cursor
contracts. Failures are saved before assertions; a missing tool/index/body does
not silently become a pass or zero-result verdict. Case deadlines are observation
bounds, not production policy. Every temporary-index gate attempt is saved;
compiler/antivirus/admission failures remain distinct from collection failures.
Cleanup closes only owned sessions via EOF and falls back to their own process
termination, never shared-daemon/account repair.

## What these cases do not establish

- No fake captures or fabricated malformed provider frames are labeled live.
  B1/B4 tests cover malformed/unknown shapes, null/false/empty values, colliding
  shorthand keys, multiple content blocks and independent structuredContent.
- B2 tests cover running/failed/unknown job replies, prior serving evidence,
  retirement and wrong roots; this harness observes real successful jobs and
  best-effort coverage. It does not sabotage shared jobs to manufacture failure.
- B3 tests cover stale cursors, repeated cursors, zero progress, changed totals/
  columns, 4 MiB exhaustion, deadlines and shared retry ownership. The live
  harness establishes real supported paging and direct-provider/raw-proxy stale
  rejection after an owned-fixture generation change. It does not claim live
  typed mid-collection generation-change/restart acceptance.
- Private observations not exposed by existing tools are checked through their
  actual consumer consequences or retained direct-provider evidence. No public
  graph schema is added solely to make tests inspect internals.
- Additional roots do not imply those projects are watched or share one atomic
  source generation. Offset exhaustion does not certify all-source completeness.
- Native C#/TypeScript controls are preservation checks, not proof that every
  Angular, NgRx, .NET or Issue #133–#141 acceptance criterion is resolved.
- A successful stdio run proves the MCP response boundary, not model tool
  selection, structured-channel consumption or VS Code rendering.

B5 acceptance must record the actual run, classify upstream/provider failures
versus adapter failures, and retain declared Windows/host/live limitations. No
overall Phase B acceptance is claimed merely because this script was written.
