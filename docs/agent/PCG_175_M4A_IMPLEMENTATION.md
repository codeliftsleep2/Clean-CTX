# PCG-175-M4A: explicit read-only MCP integration

B3 source disposition: Current normative request and transport clauses are owned by [evidence MCP](../architecture/contracts/pcg-mcp-v1.md). The remaining sections retain M4a implementation/verification chronology. Subsequent scoped acceptance is attributed in [M4b reconciliation](PCG_175_M4B_SCOPE_RECONCILIATION.md); this is not authorization for runtime ledger work.

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: owner acceptance of M3, explicit M4a implementation and focused
verification authority, and the additional instruction to commit a clean M3
baseline first. No M4b, production enrollment, push or publication is authorized.

Disposition: **M4A ACCEPTANCE READY**. Five focused integration cases and four
directly affected existing compatibility tests passed. Changed-file formatting,
library-scoped Clippy and the all-feature server binary build passed. Two fresh
real-binary stdio sessions passed: PCG enabled and disabled. The enabled client
received the exact M3-metered result bytes. No prior-stage campaign was reopened.

## 1. Accepted baseline checkpoint

Before M4a source edits, committed all accepted PCG work through M3 as one honest
checkpoint, rather than fabricating separate historical phase commits:

- Commit: `b65561ed6169ce860a7f07c32a08382de5b5e725`.
- Tree: `0447afbb8249580b3d223333d34906b77fcc8d2d`.
- Branch: `(fix)Performance-Regression-Issues`.
- Subject: `feat(pcg): checkpoint accepted enrollment, acquisition and M3 projection`.
- Scope: 43 accepted PCG source/test/definition/report files, including the narrow
  existing lib/main wiring; 9,034 additions and one deletion.

The normal commit hooks passed strict UTF-8 inspection of 2,353 text files and
the active-file-size check (34 active files; 12 untouched legacy debt files
reported). Hooks were retained. A subsequent status inspection was clean before
M4a changes. These hook receipts are text-policy checks, not a full test/CI gate.
Git required sandbox elevation for index/commit writes; both authorized actions
completed. No branch switch, remote operation or push occurred.

Only that baseline was committed. All M4a changes below remain unstaged and
uncommitted; no second commit or publication was made.

## 2. Production lifecycle and exact-byte boundary

Current normative owner: [AUTH.PCG_MCP_V1](../architecture/contracts/pcg-mcp-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## 3. Exact changed-file inventory

| Path | Change |
| --- | --- |
| `src/mcp/pcg_state.rs` | Environment-selected enabled/disabled session capability. |
| `src/pcg/session.rs` | Pinned trusted profile witness; read-only calls to the existing M3 entry. |
| `src/mcp/state.rs` | One PCG field and focused initialization; remains below the line ceiling. |
| `src/mcp/mod.rs` | Register the PCG state module. |
| `src/mcp/tool_handlers/pcg.rs` | Strict request adapter, approved fixed error mapping and exact-result sender call. |
| `src/mcp/tool_handlers/pcg/schema.json` | Exact request schema copied from approved freeze §8.2; no new tool arguments. |
| `src/mcp/tool_handlers/mod.rs` | Register the handler module. |
| `src/mcp/tool_handlers/registry.rs` | Conditional registry extension composed from the unchanged default registry. |
| `src/mcp/tool_dispatch.rs` | Select enabled or default registry from the session capability. |
| `src/mcp/tools.rs` | Session-aware catalog extension; original `tool_list()` remains unchanged. |
| `src/mcp/handlers.rs` | tools/list consumes the session-aware catalog. |
| `src/mcp/tool_annotations.rs` | Read-only, non-destructive, idempotent, closed-world hints for the new tool. |
| `src/protocol.rs` | PCG-only verbatim result framing and opt-in test wire capture. |
| `src/pcg/mod.rs` | Session module and normally registered test fixture bridge. |
| `src/pcg/query.rs` | Classify projection-open failure as projection unavailable, not source-witness failure; demonstrated correction in §5. |
| `src/tests/mcp/pcg_query_v1.rs` | Five normally registered focused integration cases. |
| `src/tests/pcg/mcp_fixture.rs` | Test-only bridge to the accepted fixture, without exposing private enrollment/store types. |
| `verification/pcg-175/run-mcp.ps1` | Small real-stdio client using one retained fixture, enabled and disabled sessions, raw streams and receipt retention. |
| `docs/agent/tooling.md` | Document conditional explicit invocation and response interpretation. |
| This report | Authority, canonical questions, corrections and measured verification evidence. |

Enrollment, persistence/schema, record identities, evidence closure, tokenizer
implementation, token packer, source witnesses and freshness ordering remain
unchanged. No Cargo dependency/lock change, canonical IR, replay, delta,
source-cache, publication owner or global policy change was made. The closed
M1/M2/M3 regression definitions and their operator drivers are untouched.

## 4. MCP request and response contract

Current normative owner: [AUTH.PCG_MCP_V1](../architecture/contracts/pcg-mcp-v1.md). The original section remains available at commit `187d0096`; this location preserves navigation and historical attribution.

## 5. Demonstrated corrections and negative controls

Initial focused compilation exposed a test bridge error: making the old shared
fixture module crate-visible exposed private `Prepared`, `Report` and `Trusted`
types, causing one compile error and three visibility warnings. Reverted that
visibility expansion. A new test-only bridge uses the existing fixture internally
and exports only ordinary fixture data and its generation string. The old helper,
production enrollment/store types and tests remain unchanged. This was compilation
setup failure, not behavioral RED; no warning was suppressed.

After compilation, the first five-case run had four passes and one genuine
behavioral failure: removing the fixture projection DB returned blocked with
`source_inaccessible`, while the test required unavailable. The source was not
missing; projection open reused a generic file-access error. The narrow correction
maps `store::reader` failure to `projection_unavailable` at the acquisition call
site. The store and source-witness error handling are untouched. The exact
unavailable assertion stayed unchanged and passed in the affected-case rerun.
This is an integration-discovered classification correction, not a redesign or
reopening of earlier stage campaigns.

The final focused run retained the original negative controls and added an
ordinary legacy `list_sessions` response check to the same compatibility case.
The raw test sink was made opt-in so unrelated tests do not retain wire copies.
The stdio client waits for initialization before normal calls and for responses
before closing stdin; it drains stderr concurrently. Its time bounds apply only
to this disposable probe, not production tool policy.

## 6. Tracked verification

All commands below were executed under this mission's explicit authority.
Logs are retained under `target/pcg-175/m4a-verify/`.

| Command / selection | Actual outcome | Log |
| --- | --- | --- |
| `cargo test --package clean-ctx --lib --all-features mcp::tool_handlers::pcg::tests::` (initial compilation) | Private test bridge type error / three visibility warnings; corrected without suppressions. | `focused-tests-initial.log` |
| Same five-case filter, first executable run | 4 passed / 1 failed / 0 ignored; 15.82s, compilation 1m 44s. Demonstrated missing-projection classification RED. | `focused-tests.log` |
| Exact `...::profile_is_pinned_and_unavailable_database_stays_discoverable -- --exact` | 1 passed / 0 failed / 0 ignored; 6.93s, compilation 1m 08s. Original assertion preserved. | `missing-projection-green.log` |
| Final five-case filter | **5 passed / 0 failed / 0 ignored / 0 measured**, 5,151 filtered out; 14.69s, compilation 48.69s. | `focused-tests-final.log` |
| `cargo test --package clean-ctx --lib --all-features protocol::tests::` | **2 passed**, 0 failed/ignored; 0.00s. Existing capture isolation and poison-tolerant response gate. | `protocol-compatibility.log` |
| Exact `mcp::tools::tests::p3_21_tool_names_match_tool_list_and_registry` | **1 passed**, 0 failed/ignored; 0.01s. | `registry-compatibility.log` |
| Exact `mcp::tools::tests::every_registered_tool_has_conservative_standard_annotations` | **1 passed**, 0 failed/ignored; 0.01s. | `annotations-compatibility.log` |
| Changed-file rustfmt, edition 2024, `skip_children=true`, then final `--check` over all 16 changed/new Rust files | Exit 0 for formatting and final check; no recursive formatting of closed-stage modules. | Empty stdout/stderr |
| `cargo clippy --package clean-ctx --lib --all-features -- -D warnings` | **Passed, zero warnings**, 23.57s. | `library-clippy.log` |
| `cargo build --package clean-ctx --bin clean-ctx --all-features` | **Passed**, 1m 38s. | `binary-build.log` |

The five new registered cases are:

1. `catalog_registry_and_disabled_legacy_parity`: conditional catalog/registry,
   strict descriptor, unchanged legacy definitions, ordinary legacy tool and
   unknown-method dispatch with PCG enabled and disabled.
2. `strict_arguments_and_protocol_errors`: missing/unknown/null/fractional members,
   mode/record rules, invalid generation/time, profile limits, tokenizer/tiny-budget
   controls and foreign-root refusal.
3. `registered_dispatch_delivers_exact_metered_bytes`: actual router/registry/
   handler/protocol path, both tokenizers and all four modes; direct M3 bytes equal
   emitted result bytes, independent token counts and one text result.
4. `states_fail_closed_and_leave_source_owners_unchanged`: incomplete scope/entity,
   unavailable generation, required overflow, stale source, empty failed bundles,
   unchanged DB bytes and untouched source-context/cache owners.
5. `profile_is_pinned_and_unavailable_database_stays_discoverable`: no principal
   replacement through profile edits; unavailable DB remains discoverable without
   evidence leakage or repair.

No ignored/conditional required tests, new suite framework, benchmark population
or prior-stage suite was introduced. Library Clippy excludes other targets;
test compilation emitted no warnings in the final run. No all-target/workspace
lint or repository-wide tests were run or claimed green.

## 7. Real MCP client / exact-byte operator evidence

Executed once:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/pcg-175/run-mcp.ps1 -Binary ./target/debug/clean-ctx.exe
```

Outcome: exit 0. This is **operator/stdio evidence**, distinct from the tracked
test results. The driver launches two fresh production server processes against
one retained tiny fixture. Each performs initialize/initialized, tools/list,
explicit PCG query, unknown-method and ordinary `list_sessions` dispatch. Each
exited 0; both receipts report succeeded with no error. Normal server stderr is
retained, not misreported as compiler warnings or assumed empty.

The enabled session advertised the legacy 26 tool definitions unchanged plus
the one PCG tool. Disabled mode advertised exactly the legacy definitions and
returned `-32601` for PCG. Both preserved legacy method/tool behavior. The enabled
query used o200k, token budget 8192, exact fixture generation
`fcdbc612825185014dcb0d626995a0528050d0a786dbe8fcfeebfafc126eb013`.

The client extracted the literal result substring from the actual stdout JSON-RPC
line without parsing/reserializing it, then compared it ordinally to the strict
UTF-8 M3 output retained by the passing integration test. Its **2,730 UTF-8 result
bytes** matched exactly; result SHA-256:
`29f0188c4e0c54d404c91dc65e67a4d6d80232c789a522abe3dec442927efa10`.
The tracked test independently counts actual delivered result bytes for both
tokenizers; equal byte sequences preserve M3's count at the real stdio boundary.
No additional provider tokenizer or JSON-RPC framing is claimed inside the budget.

Artifacts:

- `probe-input.json`: arguments, trusted fixture profile location, expected M3
  bytes and legacy catalog; SHA-256 `fd44cd96b3bcf32d6140f39b95723d3f268284b50c0c01466b0a5ca030c1c15f`.
- `stdio/receipt.json`: binary identity and both process exits/statuses;
  SHA-256 `f67e4b3f018bfa27eb116d94ea7c346d35d4bd210d6d1e65a47f468c44765742`.
- `stdio/{enabled,disabled}.stdout.jsonl` and `.stderr.txt`: retained real process
  streams, five response lines per session.
- `stdio-probe.log`: driver completion output.

Executed binary: `target/debug/clean-ctx.exe`, SHA-256
`0959d317624514e5cb9e1d79e36e5ba1c48587f7ac84496d31b7edec1d29f76c`.
The build completed against the source inventory below; source bytes were checked
unchanged through lint, build and probe, and the driver checked the executable
hash before/after its sessions. This binds this binary and probe to the final
identified implementation, rather than relying on a file timestamp.

## 8. Canonical questions and consumer audit

| Question | Public path / proof | Answer and refusal / evidence |
| --- | --- | --- |
| Which enrolled decision constrains this exact owner? | `pcg_query_v1`, constraints + explicit root/corpus/generation/subject | Source-attributed revision and exact citations; foreign scope rejects, unsupported entity proof stays incomplete. Both dispatch and real stdio are exercised. |
| What premises and gates compose its context? | Closure mode through the existing M2/M3 entry | Mandatory proof paths survive unchanged; callers cannot select a weaker edge set. Four-mode exact-byte tests exercise public reachability. |
| What depends on this record? | Impact mode over the named generation | Existing reverse impact paths, same completeness and budget rules; no new graph traversal in handler. |
| Why is it protected/rejected? | Provenance mode -> existing cited proof | Exact attributed source/execution subjects, not a new approval or rerun receipt. Metered bytes retained unchanged through transport. |
| Does it apply now and is it visible? | Session-pinned profile + exact subject/time/history policy | Revoked/replaced trust fails closed; no caller profile/partition or fabricated canonical endpoint. |
| What happens when evidence, DB or budget fails? | Existing M3 controls + the corrected DB-open classification boundary | Incomplete/blocked/unavailable remain explicit; no actionable partial bundle, raw fallback, repair or hidden import. |
| Does the MCP client receive exactly what was counted? | Raw sender under production stdout lock | Real enabled-session result is byte-identical to M3; disabled legacy catalog/dispatch stays unchanged. |

Producer is owner-enrolled Git/sidecar evidence, durable owner is the existing
isolated SQLite generation, and the session holds only the trusted profile witness.
Queries open read-only state and close it after explicit processing. Deletion or
permission loss refuses as above; no automatic recovery changes ownership. The
consumer is now a real MCP tool caller, exercised with a fresh binary/client,
rather than only an internal evaluator or test-only registry.

## 9. Source identity, AF-PERF and limitations

Final M4a changed-source/definition inventory:
`target/pcg-175/m4a-verify/source.sha256`, SHA-256
`0de354231f559154dc040eda0c5ea8205f509ce6b24850cc5f42e6f7ed299c56`.
Scope: the 18 changed/new source, test, schema and operator-definition files under
`src/` and `verification/` relative to the baseline commit. Definition: ordinal
slash-path order, lowercase file SHA-256, two spaces, path and LF per entry,
final LF included, SHA-256 over strict UTF-8 bytes. Documentation is excluded.
The immutable baseline commit supplies all unchanged dependencies/source owners.
Inventory bytes were rechecked after the passing focused run, lint, binary build
and successful probe; no intervening source drift was observed.

Selected log SHA-256 identities:

- Final five cases: `b2ed899bc3a774c62f08e97399c846069cf4ba752da51b8f6bca1c44fc6b3312`.
- Demonstrated classification RED: `7665ee5dc6168c88b337a560be40c8228f018eb2f139aae93a2d35b5299e3b02`.
- Affected-case GREEN: `181a269302e720d9cce2bcda9e1e318bfc0ec6c1744bfaf6b5341cb3e3dbab0e`.
- Library Clippy: `a5dd4048ab8280cf7902db318399586e6190a1cb75fb9d8e8ab8dd410eea4270`.
- Binary build: `dcccaebc383fb74c70fbf472b2db55751639be8a92baf4746af6ede77bb2fed9`.

Operation scope is one explicit MCP request over one existing projection. The
adapter decodes input once; profile pin/rechecks cost profile bytes, and raw
framing/output cost the returned result bytes plus request-ID serialization.
The handler does not enumerate the corpus, rebuild closure, rehash authority per
candidate or re-escape citations. Existing M3 work/counters/ceilings remain the
semantic work owner. No new graph/profile result cache, retry or global cap was
added. Timings above are scoped execution receipts, not a throughput, latency
improvement, enterprise efficiency or scale claim.

Strict UTF-8/no BOM, replacement-character, whitespace, PowerShell AST and active
code line-ceiling inspection passed; changed Rust files are <=615 lines. The
largest changed state file remains 597 lines including final split line. Git's
normal CRLF checkout notices are not compiler warnings. No formatting of untouched
large source files or repository-wide decomposition occurred.

Remaining limits: trusted single-owner local profile/storage, admitted-generation
integrity assumptions, source-attributed approval, observed-boundary freshness
with ABA/TOCTOU limits, unavailable canonical entity proof and optional historical
enrichment limits inherited from M3. The real stdio run covers one tiny fixture,
not every security/crash invariant or a real enterprise corpus. Proxy transport
and Claude field behavior were not tested or claimed. Deterministic local evidence
is complete for this integration slice; no Claude-only question requires another
pilot now. M2a-1/M2a-2/M3 campaigns, full CI, workspace tests, all-target Clippy and
benchmarks remain unrun in this mission, not failed or silently green.

**Recommend M4A ACCEPTANCE READY.** No remaining demonstrated integration blocker.
Leave M4a changes uncommitted for owner review. Stop before M4b, automatic
injection, production enrollment, further commit, push or publication.

```architecture-graph
node: PCG-175-M4A
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Conditional read-only evidence MCP implementation
track: REPOSITORY
artifact: docs/agent/PCG_175_M4A_IMPLEMENTATION.md
implements:
  - AUTH.PCG_MCP_V1
note: Historical implementation and scoped reported verification; B3 does not rerun or independently certify these witnesses.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
