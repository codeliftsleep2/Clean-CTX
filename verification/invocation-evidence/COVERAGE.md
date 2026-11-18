# Observable invocation boundary coverage

Acceptance criterion: every fact established internally that is observable through
an existing MCP surface is delivered correctly, consistently and freshly. This
matrix scopes **everything changed in the invocation-boundary phases**, not every
unrelated Clean-CTX feature. Existing definitions below are coverage authorities,
not a claim that those harnesses were rerun in this task.

## Existing verification reviewed and reused

| Existing definition | Assertions already owned there | What this expansion adds |
| --- | --- | --- |
| `context-compression/codec/scripts/Verify-Captures.ps1` and schema-v5 verifier | Preserved declarations, duplicate calls, arity, spread, unresolved calls, fidelity/focus errors, exact bodies, restore/replay, cross-file provenance | Structural invocation edge cases and corrected multiline RxJS payload delivery |
| `context-batch/scripts/Verify-ProvideContextBatchLive.ps1` | Schema, cap, ordering, isolated errors, mirrored/structured/indexed routing, single parity and repeat cache behavior | Actual RxJS, C#, Karma and Vitest payload parity in all three routes, mixed fidelities/files |
| `workspace-query/scripts/Verify-BatchQueriesLive.ps1` | Heterogeneous graph query batching, discovery reuse, isolated invalid item and legacy single parity | Mixed fresh canonical TS/C# calls plus Angular graph questions, missing source isolation, same-length edit, mirrored answer equality |
| `workspace-query/scripts/Verify-IdentityResolutionLive.ps1` / `Verify-EntitiesInFileLive.ps1` | Identity/physical-file ownership and entity publication | Same-named call owners in different source files; alternating fresh call queries |
| `workspace-query/scripts/Verify-CSharpConstructorDependenciesLive.ps1` | Constructor type dependency production lifecycle and stale edge removal | No duplicate DI suite: invocation evidence does not change that relation |
| `live-acceptance/angular_template_linkage_live_acceptance.mjs` | Angular template/selector linkage, completeness and lifecycle | No duplicate template suite: this migration changes invocation/RxJS authority, not template interpretation |

Generic protocol tests do not establish correctness of a new semantic payload.
For that reason payload-specific single/batch parity is exercised here even
though the batch routing itself is already tested elsewhere.

## Phase suites

`Verify-InvocationEvidenceLive.ps1 -Phase N` selects the phase; omit `-Phase` for
all. Initialization/tool readiness applies to every selection. The summary records
phase, case identity, result, planned and selected counts. Missing/partial runs
fail. All requests and replies are captured. The case files are direct extensions
of the existing stdio driver, not a new model-host/configuration mechanism.

| Phase | Observable contracts checked | Case families |
| --- | --- | --- |
| 0 preservation (2 cases) | Both Angular testing source styles still expose describe/it/TestBed, Vitest spy and cross-file Tests→Component | `angular-*-testbed-preservation` |
| 1 lifecycle boundary (2) | Source identity isolation, same owner name in TS/C# sibling files, alternating and repeated queries | `phase1-*-source-isolation` |
| 2 TS collection (9) | Free/member generic written calls; spread and whole rich argument counts; receiver/argument nesting order; await/return; conditional duplicates; dynamic and lexical exclusions; field calls not assigned to method; bound-arrow ownership; framework-neutral negative control | `phase2-ts-*`, `typescript-canonical-calls` |
| 3 C# collection (8) | Named/generic calls; rich object/array/ref/out/in argument count; nested receiver/argument; local initializer; repeated/control/UTF-8 decoys; field/property exclusion; exact overload and interface/file isolation | `phase3-cs-*`, `csharp-*` |
| 4 generic consumer proof (3) | Shared TS/C# canonical summaries agree, callback-contained written calls preserved; every calls helper compares text/structured surfaces | `phase4-cross-language-*`, `phase4-*-deferred-*` |
| 5 domain consumer (13) | Commented/spaced pipe terminals; whole rich arguments; nested callback suppression versus receiver chains; import gate; dynamic/non-pipe exclusions; threshold; owner policy; High multiline arguments and Unicode cutoff; Low/Medium parity; disabled configuration | `phase5-rx-*`, `rxjs-*`, `*-config` |
| 6 freshness (5) | Same-length TS/C# edits, empty bodies, delete/error/restore without changing caller identity; changed/empty/deleted/restored Rx source; same-length Rx operator edits reach canonical and provided text; complete multiline markers survive durable save/server restart/restore | `phase6-*`, `changed-source-*` |
| 7 composition (17) | Compression/provide parity at Medium/High/Edit; exact verbatim; Rx batch routing; mixed C#/Rx/testing files and fidelities; interleaved questions; cross-file TS/C# written-reference truthfulness; same-source Angular/NgRx/Rx composition; cross-layer edit isolation; mixed graph/call query batch with local failure and freshness | `phase7-*`, `provide-context-*`, `verbatim-*` |

Phase 4 introduces a private test consumer, not a production generic-consumer
tool. Its MCP cases prove existing canonical preservation, not direct access to
that private reader. Phase 7 is composition/acceptance, not a new producer.

## Every frozen contract (T1–T24)

| Contract | MCP-observable consequence / live authority | Internal-only remainder |
| --- | --- | --- |
| T1 member call | Phase 2/3 terminal and arity | Receiver/full byte spans and callable context |
| T2 nested argument | Nested calls and ordered arities; deferred callback cases | Immediate parent identity/index |
| T3 receiver chain | Ordered canonical calls; Rx receiver-chain markers | Structural receiver containment |
| T4 generic | Generic calls retain terminal and arity | Type argument spans; no resolved generic meaning |
| T5 spread | TS generic/spread case and existing codec spread oracle | Spread span |
| T6 named C# | Named whole-argument count | Name qualifier/span |
| T7/T8 await/return | Phase 2 await/return call preservation | Explicit absence of await/return ancestry |
| T9 initializer | Field/property calls excluded from selected method; Rx initializer owners | Private declaration context; no new initializer query |
| T10 callback | Written nested calls; Rx rich and nested callback policy | Callback ancestry deliberately absent |
| T11/T12 repeated/control | Ordered repeated and conditional calls | Distinct private span identities; control ancestry absent |
| T13 framework-neutral | Framework-neutral FakeStore negative marker control; no resolution field in written calls | No framework semantics in evidence types; tracked generic-consumer proof |
| T14 unsupported target | TS/C# dynamic call exclusions and Rx dynamic terminal negative | Unsupported root remains private |
| T15 UTF-8 | TS/C# UTF-8 decoys and actual Rx Unicode summary; verbatim equality | Exact private byte-span round trip |
| T16 edit | Same-length calls and Rx operator replacement; mixed query batch edit | SHA-256 rebinding rejection |
| T17 deletion | Missing source produces error, restored source fresh | Old evidence cannot bind missing source |
| T18 canonical preservation | All call suites plus existing codec and graph verification | CallEvidence internal projection tests |
| T19 Rx parity | Ordered owner/operators, receiver/nested policies, all fidelities/config | Full standalone RxShape equivalence tests |
| T20 lexical | Fake pipe and fake call strings/comments excluded | Capture-level occurrence exclusion |
| T21 rich boundaries | Objects/arrays/commas/templates in TS arity, rich Rx operator summaries and C# arguments | Exact untruncated whole private slices |
| T22 standalone API | Not an MCP entry point | One-pass source-only compatibility tests |
| T23 single parse | Not wire observable | Production parse-counter regressions |
| T24 shared parity | TS/C# canonical call records identical for equivalent calls | Shared private fields/qualifiers and language-neutral type model |

## Limits that must remain explicit

These runs observe server JSON-RPC `content` and applicable `structuredContent`,
not proof of a particular Claude/Codex host consuming every channel. Raw fallback
is required to be exact current source; it is not relabeled as semantic marker
rendering. No .NET/Angular application compilation or Karma/Vitest runtime runs.
Calls are written source facts: cross-file source references must not be promoted
to resolved implementations or inline another file's body. No new public
occurrence schema was added. Private records are compilation-local and are not
persisted; generic binary persistence/restore remains the existing harness's
contract, complemented here by the corrected RxJS multiline marker save/reload. Full repository tests remain CI-only.
