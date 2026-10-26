# Issue #162: route provider approval-to-acceptance map

Agent: Agent MaxHeadRoom
Status: bounded P1.R1 local acceptance GREEN; generation-66 scoped Rust/lint/guards and route binary PASS (305 + 332 preservation checks); broader repository/field gates open.

This reconciles the owner's 43 verification items for P1.R1. It does not close
P1, AQ12/AQ13, W04, the repository gate or the final field gate. The supplied
approval remains authoritative; this map neither narrows it nor adds semantics.

## Generation-66 binary acceptance inspected PASS

Inspected route-providers PASS: 305 checks, and provider-imported-owners
preservation PASS: 332 checks. Both receipts record zero failed checks/errors,
three FINISHED sessions and the 500-check maximum. They share binary SHA-256
b06e07bef41ea377c25023b4c9e8616b58f1ba941ae0dfd7389d5e203e162abf,
mtime 2026-10-26T21:35:04.237Z, and input hash
e24125aa3ad218adb7997b3c44f07fb52d787e19d5ed3ed8dc3a15182edc5df4.
Counts/report hashes are retained at target/issue-162/route-provider66-binary-receipt.json.
Full logs: target/issue-162/route-provider66-binary-20261026-173424/.
The all-feature build finished without warnings/errors; syntax preflight passed
29 JavaScript files.

This supersedes the authored binary UNRUN checkpoint for these two selected
suites. Together with the scoped Rust/lint/guard receipts, it satisfies bounded
P1.R1 local acceptance for the approved declared-root/object/provider forms.
Same-path and empty-path public multiplicity survives the tested binary lifecycle;
no public field/selector/equality change was required. Known-owner reacquisition
after external root recreation remains explicit; automatic removed-identity
rediscovery is not certified. No other binary suite, full repository gate,
activation/injector answer, AQ12/AQ13, broader P1/W04/#162 or final field acceptance
is inferred. No production/harness change accompanied inspection.

## Evidence classes

- Original two unchanged REDs: src/tests/mcp/angular_route_provider_issue_162.rs.
- Initial integrated controls: src/tests/mcp/angular_route_provider_controls_issue_162.rs.
- Exit controls: src/tests/mcp/angular_route_provider_exit_issue_162.rs.
- Preservation: provider/view/string/module/helper/routing/compatibility filters,
  fmt/Clippy and encoding/size guards; 172 scoped passing executions including
  preserved historical acquisition-control failure and corrected one-test rerun.
- Counts/hashes: target/issue-162/route-provider66-preservation-receipt.json.
- Operator binary definition: verification/live-acceptance/issue-162/route-providers.mjs,
  selected through suites/route-providers.mjs. This is authored, not runtime evidence.

## Required controls mapped

| Approval items | Existing tracked evidence | Remaining binary boundary |
| --- | --- | --- |
| 1–3: direct/local/imported composition | Both original controls; alias_namespace_roots_composition_and_malformed_siblings; proof_forgery_serde_and_shared_strategy_policies | Shared imported alias/barrel/namespace entries plus repeated direct local entries |
| 4–6: exact native root, alias and type-only anchor | native_root_children_same_path_and_refusals and alias_namespace_roots_composition_and_malformed_siblings | Type-only aliased Routes root over real stdio acquisition |
| 7–10: fake root, wrong package/export, arbitrary name/object | Native-root refusal control; alias/namespace refusal table; stale export/config control | Fake annotated type and wrong public export/config retirement without claiming a generic type evaluator |
| 11–17: multiple/child/same/empty paths, same token, distinct same-named tokens, mixed entries | Native-root/children control; complete SQLite restore; independent retirement; full private identity roundtrip | Exact forward and reverse source/path/token multisets across two owners; repeated same-source empty-path occurrences |
| 18–21: source/hash/span/object/root/container proof | Initial forgery batch and crossed_object_field_and_private_identity_refusal | Public source/endpoint privacy and correct retirement; private evidence itself stays a Rust/SQLite assertion |
| 22–25: malformed/competing entries, strategy limits, cycles and siblings | Shared-strategy initial control; alias/namespace malformed-sibling table; all decorated-owner provider and helper preservation | Mutable, type-only, cyclic and deleted contributor behavior while independent direct/child/empty entries survive |
| 26–27: complete SQLite restore before queries, stale refusal/reacquisition | complete_sqlite_restore_same_path_empty_path_and_retirement; complete_private_identity_roundtrip_before_public_queries; both stale batches | Three fresh processes, real save/restore, stale restore refusal before current semantic queries |
| 28–37: root/object/field/contributor/token/config/package mutation, replacement/deletion and repair | Both stale batches and independent_fields_empty_objects_contributor_and_owner_retirement | Batched source/field/object/contributor/token/config/package changes; owner recreation uses explicit known-root acquisition |
| 38: exact same-path persistence/index multiplicity | Complete private endpoint equality before any public query, exact retained-index authority and same/empty-path public counts | Exact public occurrence multisets after restart and independent field/object retirement |
| 39–40: decorated owners and Component views | 85 provider, 6 view-provider, 6 helper preservation tests, plus related string/module filters | Existing provider-imported-owners suite re-run against the same rebuilt generation-66 binary |
| 41: preserve existing routing without target upgrade | 26 routing tests; original RouteMapsTo prerequisite | No eager/lazy/guard/resolver authority upgrade claimed by route-provider assertions |
| 42: private identity serde/compatibility | Complete SQLite/serde endpoint equality; annotation/root proof removal refusal; current/prior generation compatibility filters | Existing binary/input receipt hashes and real restart transport |
| 43: zero warnings and rebuilt binary | Scoped fmt/Clippy and guards already GREEN | Logged all-feature rebuild, route-providers and affected owner preservation PASS still pending |

## Public-API guardrail and acquisition limit

The passing Rust controls demonstrate that distinct same-path/empty-path objects
can survive current private identity, SQLite, index and public serialization as
separate occurrences. They do not create an exact route selector. The binary suite
must independently preserve those multiplicities through the actual stdio path.
If it exposes conflation or cannot express the admitted lower-bound answer, stop
and present the concrete public-contract decision before adding fields/selectors.

After an external edit removes the queried route identity entirely, automatic
rediscovery on later source recreation is not certified. The corrected root-edit
control and binary suite explicitly reacquire the known source through
compress_code_context. Unchanged-owner contributor/token/barrel/config/package
repairs still use semantic queries alone. This limit is retained in the failure
archive and cannot be erased by the corrected passing control.

Imported route roots, route-object spreads, RouterModule/provideRouter/bootstrap
roots, activation, active application selection and injector resolution remain
outside this approved slice. Existing RouteMapsTo is preserved as lower-bound
legacy behavior, not certified as a canonical eager/lazy target answer.

## Major integrated handoff

run-route-provider66-checkpoint.ps1 performs syntax preflight, one all-feature
binary build and the two selected suites. It saves full logs in a timestamped
target/issue-162/route-provider66-binary-* directory. Each suite owns its three
sessions, binary/input hashes, RPC/stderr/report captures and hard 500-check
ceiling; exceeding the ceiling fails rather than truncating checks. No full
inventory or final Claude pilot is requested by this handoff.
