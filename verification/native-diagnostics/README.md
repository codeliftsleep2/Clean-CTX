# C#/TypeScript/Angular native-hook operator verification

This PowerShell driver runs real diagnostic producers in an isolated copy of
tracked fixtures, then replays their exact stdout/stderr through the built
`clean-ctx claude-hook post-tool-use` adapter. It follows the existing operator
process/capture pattern, with concurrent stream draining and actual exit codes.
It never starts Claude, a watch/dev server, or the repository test suite.

## Prerequisites

Use PowerShell 7, Node.js compatible with Angular 21, a .NET 10 SDK, and the
already-built all-feature Clean-CTX binary. Install tools outside the checkout;
`NodeToolsRoot` is the directory containing their `node_modules`:

```powershell
npm install --prefix <tools-directory> --no-audit --no-fund --save-exact typescript@5.9.3 eslint@9.39.1 @angular/cli@21.0.0 @angular/build@21.0.0 @angular/core@21.0.0 @angular/compiler@21.0.0 @angular/compiler-cli@21.0.0 @angular/common@21.0.0 rxjs@7.8.2 tslib@2.8.1
pwsh -NoProfile -File verification/native-diagnostics/scripts/Verify-NativeDiagnosticsLive.ps1 -NodeToolsRoot <tools-directory>
```

`-BinaryPath`, `-DotnetPath`, and `-NodePath` select exact executables. On Linux
the cloud setup installs these tools below `/workspace/.tooling`; npm's cache
also uses that writable root. Windows uses a directory junction for the isolated
workspace's dependency directory; Linux uses a symbolic link.

The Angular runner cases additionally require:

```powershell
npm install --prefix <tools-directory> --no-audit --no-fund --save-exact @angular/platform-browser@21.0.0 @angular/platform-browser-dynamic@21.0.0 vitest@4.0.8 jsdom@27.2.0 karma@6.4.4 karma-jasmine@5.1.0 karma-chrome-launcher@3.2.0 karma-coverage@2.2.1 jasmine-core@5.12.0 @types/jasmine@5.1.13
```

Pass `-ChromePath` (or set `CHROME_BIN`) for Karma's browser. The harness
discovers common Chromium/Chrome command names on Linux. Its isolated Karma
launcher uses container-compatible headless flags. `-CaseNames` can select
individual named cases; unknown names fail rather than selecting zero cases.

## Assertions

- A successful .NET build retains real CS1030 warning text, artifact/build
  output, and warning/error totals while reducing approved boilerplate.
- ESLint retains the unused-variable name, rule, and warning summary.
- Angular retains its real initial-bundle budget warning and completion summary.
- Each reduction uses the expected stdout filter, leaves stderr and unrelated
  metadata unchanged, and emits exactly one disclosure.
- Successful empty TSC output stays empty.
- A real two-test MSTest run checks the installed .NET 10 runtime and loaded
  test assembly; its clean 2/2 VSTest summary collapses under `dotnet-test-v1`.
  An intentional 1/2 failure receives no replacement and keeps its diagnostic.
- A real TS2322 failure maps to `PostToolUseFailure` and receives no replacement.
- Informational .NET and TypeScript commands receive no replacement.
- Classic `@angular/build:karma` runs two Jasmine/TestBed tests in Chromium.
- The new `@angular/build:unit-test` builder also runs two Jasmine/TestBed tests
  with its Karma runner and two TestBed tests with Vitest/jsdom. Both spy APIs
  actually execute. Runner selection and includes are explicit; watch is off.
- Karma's exact completed counts and Vitest's file/test summaries survive the
  hook with the `angular-test-v1` stdout authority. The checks account for real
  ANSI wrapping in the raw Karma output without changing the hook input.

Every invocation creates a distinct directory under
`target/native-diagnostics-verification/`. Per-case evidence includes producer
streams and exit code, the replay envelope, hook output, and hook facts. The
aggregate report records the binary hash and all passed/failed observations;
the command fails if any case fails. A 120-second per-process operator deadline
cleans up the process tree on failure; it does not change product policies.

## Observation — 2026-10-08

The original seven cases and all three Angular runner cases passed on Linux
using .NET SDK 10.0.401, TypeScript 5.9.3, ESLint 9.39.1, and Angular 21.0.0.
Both Karma configurations executed 2/2 tests successfully; Vitest executed
2/2 successfully. These pinned versions reproduce this
observation; they are not product support limits. No production code changed.

## Testing source/meta-layer MCP boundary

The existing context-compression measurement helper must already be built:

```powershell
pwsh -NoProfile -File verification/context-compression/scripts/Build-MeasureHelper.ps1
pwsh -NoProfile -File verification/native-diagnostics/scripts/Verify-AngularTestingMetaLive.ps1
```

This second driver uses the same Karma/Jasmine and Vitest specs through
registered `provide_code_context`, the existing read-only SQLite exporter, and
the production Binary0x04 decoder. It checks model-visible suite/spy source,
exact raw passthrough when declared, the exact compressed `T @spy` target
when compression is selected, one `TestArtifact --Tests--> DiagnosticProbe`
edge per spec, and a fresh-session restore plus workspace query retaining that
edge. A generic spec with comment/string-only fake TestBed calls must produce
no Angular Tests edge. All these MCP observations passed on Linux. Generated
evidence lives under `target/angular-testing-meta-verification/<unique-run>`.

Both Jasmine and Vitest now produce specialized `Φspy` markers. The driver
requires their exact `fixture.componentInstance.renderLabel` spy target once
in the decoded durable metadata, alongside the TestBed edges and separate
model-visible source checks. Jasmine `createSpy`, `createSpyObj`, `spyOn`, and
`spyOnProperty` use the existing lexical guards and sorted/deduplicated marker
projection. The marker expansion names both frameworks. Three tracked
regressions (two observed RED before the fix), all 33 focused testing checks,
package Clippy, and the updated MCP driver passed. Only the Angular marker
producer generation advances again, to 3; semantic generation remains 2.

## C# framework/meta-layer MCP boundary

```powershell
pwsh -NoProfile -File verification/native-diagnostics/scripts/Verify-CSharpMetaLive.ps1
```

This driver reuses the existing `src/test_files/dotnet` fixtures, stdio session
helper, read-only SQLite exporter, and production Binary0x04 decoder. It copies
only four fixture files into an isolated runtime and checks:

- ASP.NET `UserController`: exact `api/[controller]` route and all five actions;
- EF Core `AppDbContext`: `User`, `Order`, and `Product` entity relations;
- SignalR `ChatHub`: `SendMessage` and `SendToUser` hub-method targets;
- AutoMapper `UserProfile`: source and destination entity relations;
- an ordinary class: no fabricated .NET framework relations.

Each expected relation must have the correct typed owner and exact target name
in the decoded durable oracle and in a workspace query after fresh-session
restore. The driver also verifies model-visible owner/method evidence and
byte-exact source when raw passthrough is declared. All four framework cases
and the negative control passed on Linux on 2026-10-08. Evidence is retained
under `target/csharp-meta-verification/<unique-run>` with the binary hash.
The driver does not compile these framework fixtures, infer runtime .NET DI
registrations, or establish actual model-host consumption.

## Angular application/meta-layer MCP boundary

```powershell
pwsh -NoProfile -File verification/native-diagnostics/scripts/Verify-AngularAppMetaLive.ps1
```

This driver preserves the existing Angular fixture tree, including HTML/SCSS
companions and NgRx/routing relative paths. Its assertions cover legacy
decorated inputs/outputs and constructor injection, generic signal-based
inputs/outputs and `inject(UserService)` token metadata at High fidelity,
NgRx dispatch/selection and effect actions,
and route/component/guard/resolver relations. An ordinary TypeScript class
must not manufacture Angular or NgRx facts. Required edges are scoped to the
asserting source file, including when fixture classes share a display name.

All five positive cases and the negative control passed on Linux against the
branch containing main's 0.9.1 C# base-type fix. Complete Binary0x04 decoding,
aligned snapshots, model content, and fresh-session queries are checked with
the same existing helpers. Evidence belongs beneath
`target/angular-app-meta-verification/<unique-run>`.

The first modern-input observation exposed missing generic signal fields,
`?` names from whitespace, and comment-only injection evidence. Two unchanged
tracked regressions in `src/tests/angular_meta/decorators.rs` failed before the
extractor fix and passed afterward. Angular marker and semantic producer
generations advance to 2 so the existing durable compatibility checks reject
pre-fix persisted facts. The focused Angular meta-layer checks passed 428 tests;
main's two cross-file C# query regressions, 15 compatibility identity tests,
and package Clippy also passed. No full repository suite was run.

This proves the built adapter handles real producer output. It does not prove
Claude accepts or consumes the replacement, establish VS Code rendering or
host persistence, constitute tracked regression-test evidence, or replace CI.
The representative host-consumption gate remains separate.

The injection-token regression now checks both markers and semantic extraction:
`service = inject(UserService)` names `UserService`, while computed/empty token
expressions are omitted. Two tracked regressions failed before the fix and
passed unchanged afterward. The existing modern fixture and app driver require
`Injects -> UserService` in complete Binary0x04 evidence and restored queries.
Current Angular marker generation is 4 and semantic generation is 3.
Focused verification passed 433 Angular tests, 15 compatibility tests, package
Clippy, and the existing five-case application MCP driver plus its negative
control. Builds used the locked dependency graph; no full suite was run.


## Local phase closure — 2026-10-08

| Active phase | Result |
|---|---|
| Real C#/.NET, TypeScript and Angular producer output | Passed: 12 isolated cases, including real VSTest success and intentional failure |
| Classic Karma/Jasmine, unit-test Karma/Jasmine and Vitest/TestBed | Passed: 2/2 tests in each runner; required inputs/models and typed outputs execute |
| C# framework metadata and source-file identity | Passed: four framework fixtures, negative control, and base/constructor dependency queries |
| Angular application and testing metadata | Passed: required inputs/models, spaced/nested generics, DI tokens, NgRx, routing and both spy APIs |
| Physical baseline and restored-query boundary | Passed: complete Binary0x04 decoding, aligned snapshots, exact raw passthrough, compressed spy target and fresh-session queries |
| Focused tracked verification and repository guards | Passed: 600 tests, all-target/all-feature package Clippy, formatting, active-file sizes and strict UTF-8 |

Three unchanged tracked `required_signal_regression_` cases observed two
intended failures and one passing negative control before the fix, then all
three passed. Angular marker generation is now **5** and semantic generation
**4**, so pre-fix durable Angular facts must be regenerated. Existing modern
fixtures and both real TestBed runners exercise the corrected syntax.

Final review also corrected two operator expectations rather than product
contracts: compressed test metadata names the exact spy target instead of
retaining literal `spyOn` syntax; and main's neutral C# base-list relation is
`HasBaseType`, distinct from `HasConstructorParameterType`. The constructor
harness checks all four returned relations and both declaring-file identities,
then verifies only the appropriate constructor edge disappears after editing.
Workspace queries return all relations; the harness selects each relation in
its captured evidence and does not claim server-side relation filtering.

The old .NET warning regression had lost its removable boilerplate in a prior
fixture edit while still expecting a disclosure. That fixture now includes
restore boilerplate; every original warning/summary/disclosure assertion stays
intact. A separate tracked case requires exact unchanged output and no filter
facts when a warning-bearing input has nothing removable. No production .NET
filter policy changed.

These are completed **local implementation/protocol phases**, not a claim that
all external acceptance gates passed. The full repository suite stays CI-only;
this cloud's read-only GitHub Actions request returned `Forbidden`, so CI status
is unobserved. No authenticated Claude host is available here (CLI absent and
neither supported API-key nor OAuth binding present). Actual Claude hook
acceptance, model consumption and host persistence/resume remain external
acceptance evidence; Codex output cannot certify those Claude-specific surfaces.
The CargoCheck host pilot remains paused under the owner's priority change.
