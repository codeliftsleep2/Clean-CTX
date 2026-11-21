# OPS-R1 machine-generated test census and effectiveness method

Agent MaxHeadRoom. This census separates source existence from real execution.

## Reproducible offline acquisition

Operator command, from the tool repository root:

```powershell
python .\verification\ops-r1\census.py
```

It invokes only Git's tracked-file listing, scans tracked Rust source, consumes
retained run-414 evidence through the existing CI parser, and writes evidence.
No Cargo/test/server/build invocation. After the first source-scanner pass exceeded
the short-check expectation it was interrupted; the scanner was tightened and the
full pass handed to the operator. Do not claim that interrupted pass completed.

Outputs: target/ops-r1/census/summary.json, source-directory inventory shards,
source-test shards, exact CI identity/outcome shards and family-summary.csv;
verification/ops-r1/test-census-evidence.zip plus the member manifest and digest
recorded in the summary. Generated source facts must remain attributable to the
generator and file hashes. The archive is evidence, not executable regression tests.

The source scanner masks comments/strings and extracts attributed test functions,
body hashes, direct assert/panic/unwrap/loop facts and adjacent cfg/ignore markers.
It is lexical, not a Rust AST or cfg evaluator. Macro-generated/parameterized tests,
module-level feature gates, helpers and registration paths are explicit limitations.
A direct-assert count of zero does not mean a test lacks assertions through helpers.
A count of many assertions does not establish meaningful architectural protection.

CI names come from actual registered/executed tuples (package, target, executable,
test identity). Function-name-to-source matches are candidates; ambiguous matches
are not silently resolved. Preserve the boundary between compiled identities and
source candidates. The census never reports invented coverage percentages.

## Completed source/retained-execution census (2026-11-20)

The operator completed the generator before descriptor refusal in
target/ops-r1/checkpoint-20261120-103100. It scanned 1,483 tracked Rust files and
identified 5,518 lexical test functions in 850 source files. Retained CI membership
contains 5,637 identities: 5,610 passed and 27 ignored. Function-name mapping gives
5,586 unique-name candidates and 51 ambiguous/unresolved cases. These are different
populations: shared source registration, targets and generated tests prevent
subtracting these totals to infer missing tests. No coverage percentage is inferred.
The generated evidence archive and member manifest are pinned by
verification/ops-r1/test-census-index.json. The failed descriptor does not erase
the successful census or preceding source guard results.

## Verified retained execution baseline

Run 414 at 2ada188e1d6fb981d7db8bc605ae1fe9101a3fdb retained 5,637 registered
and attributed outcomes, no membership errors, 56 deterministic fitness groups.
Library: 5,260 passed, 27 ignored; workspace/proxy/targets contribute the remaining
identities. The nine declared legacy ignores and 18 explicit measurements are
different categories; mandatory measurement companions remain ordinary execution.
Exact archive: verification/ci-verify-r1/runtime-accounting-414-evidence.zip.
The census checks retained registration/execution hashes against their receipt.

Later run 415 is successful at 84bb200d; its two exact job identities and raw logs
are retained in runtime-415-evidence.zip. Warm total 11m12s was accepted by the owner.
This does not make run-414 tuples current execution of OPS-R1 source changes.
No fresh Cargo listing or suite was started for this audit.

## What the reviewed families actually protect

| Reviewed family / categories | Contract and negative evidence | Execution/value and remaining gap |
| --- | --- | --- |
| Core query/capture — semantic equivalence, architectural work, concurrency | Complete definitions/captures/instructions, warm query compilation count, independent parse/cursor, custom fallback and concurrent registered use. | Unique architectural protection with historical RED. Does not count TypeScript project acquisition. |
| Resolver registration — integration, identity, lifecycle, authority | Cold forward/reverse, key multiplicity, exact sources/spans, omission attribution, aliases, persistence and lifecycle. | Valuable real production dispatch tests. Fixed-owner record growth misses unrelated owner/project axes. |
| Shared retirement — lifecycle, authority, work fitness | Established deletion, acquisition error, reappearance, independent supports, unrelated families and once-per-owner/batch probes. | Historical exact RED and counterexamples; must be preserved in any shared repair. Does not prove all-query termination. |
| Restore config — authority/currentness, integration | Config enablement distinct from optional identity, native scope and current/history; stale/forged/missing evidence refusal. | Detects historical boundary defect. Historical source changes can be valid H1, not current H2. |
| AF-PERF AST — architectural structure/work | Direct and helper-mediated repeated sinks; actual retained-admission entry/module/sink list. Unknown dispatch/macros produce review findings. | Valuable scoped structural prevention. Unlisted TS project acquisition and executor batch lifetime are not automatically inspected. |
| CI membership — verification infrastructure | Actual Python verifier is run on valid, missing, duplicate, failed, ignored, empty, measurement and interleaved/nested evidence fixtures. | Tests execution integrity rather than production semantics. Omitted arbitrary test obligations need golden membership to become mandatory. |
| Feature-native evidence — integration/structure | Native TS ownership without Angular; isolated compile/registration exposes cross-feature imports and cfg mistakes. | Focused operator GREEN exists; allfeature/default CI does not establish independently isolated wiring. |
| Dispatcher shutdown — concurrency/progress, stated contract gap | shutdown_stops_new_requests never calls shutdown; assert accepts Ok or Err. shutdown_handles_stuck_workers has timeout call/assertions commented and sleeps 10s. | Executed presence is not shutdown proof. shutdown_completes_inflight_requests proves a delayed request completed, not shutdown completion. No removal authorized. |
| Encoding — functional/infrastructure | Byte-exact Unicode canary, strict UTF-8/BOM/corruption scanning. | Useful integrity protection, not graph admission or semantic/currentness certification. Some script/Rust guard overlap is deliberate independent enforcement. |
| Empirical integrated costs — performance measurement plus assertion companions | Existing frozen outputs/work and preserved growth dimensions; 18 explicit repeated measurements paired with nonignored companions. | Measurement is not useless or retired. Ordinary CI need not repeat timing campaigns to protect functional assertions. |
| Proxy integration — integration | Production proxy executable/mock-server transport boundary in proxy/tests/integration_test.rs; root semantic tests do not replace it. | Separate package/target membership preserved. Not an Angular authority or workspace-scale resource test. |
| B11 admission — authority/infrastructure | Accepted authority survives candidate policy edits; exact diagnostics and immutable accepted head on rejection. | Actual checker remains needed at publication. Passing Rust tests do not prove an author acquired/admitted this new packet. |

## Completion limits

Repository-wide source and CI identity census is automated. Deep semantic-value
classification here covers explicitly inspected major families, not every one of
5,000+ bodies or every helper transitively. Source-only category flags cannot
complete that audit by themselves. Remaining families must be marked unreviewed
in the exposure/coverage backlog, not assigned confidence from names.

No existing test is removed, weakened or deduplicated by this design. Equivalent
body hashes and overlapping outputs are leads, not proof fixtures/helpers/features
protect the same contract. Cost decisions require exact unique verification value.
Historical mutation runs and a current isolated-feature inventory remain operator-
owned and pending where not already established by retained receipts.
