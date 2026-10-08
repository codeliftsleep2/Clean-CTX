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
exact raw passthrough when declared, one `TestArtifact --Tests--> DiagnosticProbe`
edge per spec, and a fresh-session restore plus workspace query retaining that
edge. A generic spec with comment/string-only fake TestBed calls must produce
no Angular Tests edge. All these MCP observations passed on Linux. Generated
evidence lives under `target/angular-testing-meta-verification/<unique-run>`.

The source checks do not claim a specialized Jasmine `Φspy` marker: the current
spy marker extractor is Vitest-specific. Both complete spy source forms are
preserved in the observed model content, and both styles produce TestBed edges.

This proves the built adapter handles real producer output. It does not prove
Claude accepts or consumes the replacement, establish VS Code rendering or
host persistence, constitute tracked regression-test evidence, or replace CI.
The representative host-consumption gate remains separate.
