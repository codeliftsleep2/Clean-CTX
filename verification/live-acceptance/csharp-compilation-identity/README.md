# C# compilation identity: Issues #153/#154 live acceptance

Agent: Agent MaxHeadRoom

This operator-run harness drives a rebuilt Clean-CTX binary over real MCP
stdio. It checks externally exposed declaration/framework facts for the
compilation identity repair. It is not a Rust test, CI gate, or replacement for
the established RED/GREEN regressions under `src/tests/**`.

Status: maintainer reported all handed-off checks passing on 2026-10-20,
including the local binary/stdio harness. Exact assertion counts and the run
directory were not supplied. NOT RUN by the agent. Original-workspace Claude
field acceptance remains pending.

## Run from the repository root

Build and run explicitly; the driver never selects an arbitrary debug/release
binary automatically:

```powershell
cargo build --all-features --bin clean-ctx
node verification/live-acceptance/csharp_compilation_identity_live_acceptance.mjs ./target/debug/clean-ctx.exe
```

On a non-Windows host, use `./target/debug/clean-ctx` for the binary argument.
An already-built release binary is accepted when explicitly supplied and fresh
against the source/build inputs. A missing/stale binary fails before startup.

Optional static syntax validation, also operator-run:

```powershell
node --check verification/live-acceptance/csharp_compilation_identity_live_acceptance.mjs
node --check verification/live-acceptance/csharp-compilation-identity/client.mjs
node --check verification/live-acceptance/csharp-compilation-identity/fixtures.mjs
node --check verification/live-acceptance/csharp-compilation-identity/oracles.mjs
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
```

The binary must support C# and .NET. Required tools are checked with
`tools/list`; missing framework support also fails the positive oracles.
The harness itself requires only Node built-ins; no package installation,
credentials, network service, or proprietary repository is needed.

## What the run establishes

The reusable generator defines 72 synthetic C# files and 38 checked identities:

- Twelve controllers: constructor at beginning/middle/end, empty/nonempty
  body, forward/reversed action order. Every variant requires the exact five
  action identities; an extra constructor or a missing final action fails.
- One overloaded-action controller, with exact public action names. The MCP
  action surface identifies these by method name; it does not prove distinct
  overload attribute ownership. The Rust regression remains that authority.
- Twenty-four framework chains: Controller, Hub, Profile, and DbContext, each
  with ordinary non-generic classes containing multiple generic methods and
  generic overloads; cross-file partials with generic methods in either part;
  genuine generic classes of arity 1/2/3 containing a method of arity 4.
  Cross-namespace leaves inherit through two admitted-source hops. Oracles
  require the exact leaf role and action/hub/mapping/entity payload.
- One MSTest class with a constructor and a generic method, plus an ordinary
  source lookalike which must retain its builtin Class without a Controller role.

For each checked identity, `find_entities` must expose exactly one matching
file-provenanced framework occurrence. `forward_edges` must expose the exact
expected target names and object types with matching subject/object file
provenance. Complete relationship sets are compared for repeated observations;
duplicate relationship facts fail. Self-Defines is an occurrence-registration
carrier at the index boundary, so the harness does not demand a fabricated
Defines adjacency for MSTest. DbContext uses an actual HasEntity payload.

The transport treats RPC/tool errors, missing response arrays, malformed
stdout, timeout, premature exit, and incomplete sessions as failures. Empty
or malformed output cannot satisfy a negative assertion by defaulting to `[]`.

## Session and lifecycle matrix

1. Fresh process/workspace A, with provider disabled and all fixture files
   explicitly published in forward order. Observe cold semantic queries and
   then warm repetitions; compare normalized facts exactly.
2. In A, rename the last action, refresh, explicitly republish all files, and
   require the exact changed action set. Restore and compare original facts.
3. In A, remove the external base from one cross-file partial root for each
   framework. Refresh and republish in reverse order. Descendant roles and
   payloads must retract; builtin source declarations must remain. All other
   checked identities must preserve their facts.
4. Restore all original sources in A, refresh, republish, and require the full
   original snapshot.
5. Fresh process/workspace B, with the same baseline sources published in
   reverse order. Repeat cold/warm assertions and compare B with A's original
   normalized snapshot. Physical workspace prefixes are normalized; declared
   identity, relation, and relative file provenance are preserved.

Every relevant tool call supplies `workspaceRoot` explicitly. Each synthetic
workspace has its own `.clean-ctx.json` with CBM disabled, proxy auto-start
disabled, and high default fidelity. There are no provider startups or real
repository discovery requests. Each request has a 120-second deadline; each
session has a 20-minute deadline. These are operator-harness bounds, not new
Clean-CTX production limits. The client closes stdin and applies bounded child
termination on exit. Ctrl+C records interruption as failure and stops the child.

## Evidence and interpretation

Each invocation creates a unique directory under:

```text
target/live-acceptance/csharp-compilation-identity/<UTC timestamp>-<pid>/
  report.json
  forward-publication/
    rpc.jsonl
    stderr.log
    cold-snapshot.json
    workspace/                  generated fixtures, config, runtime state
  reverse-publication/
    rpc.jsonl
    stderr.log
    cold-snapshot.json
    workspace/
```

`report.json` includes the binary hash/mtime, Rust source/build-input hashes,
fixture manifest/hashes, exact mutation definitions, completed assertions, and
errors. Input hashes are rechecked at completion. `rpc.jsonl` retains every
request and raw response, including notifications, with timestamps and request
labels. Evidence and generated workspaces are retained even on failure; the
driver does not recursively delete directories. It never modifies tracked
sources or fixtures. Only a complete run with every assertion true reports
PASS and exit code 0. An interrupted or partial run is not a pass.

## Proof boundaries and final field gate

"Cold" here means a fresh MCP process and initially empty synthetic workspace
index **followed by explicit admission of the known fixture files**. It does
not prove automatic cold dependency acquisition. That is Issue #155 and is
deliberately excluded. Workspace coverage metadata is retained verbatim;
these fixture oracles do not promote a lower-bound response to global
source-completeness proof.

SignalR per-method realtime IR IDs and constructor-ID preservation are verified
by the tracked Rust tests. This harness checks the publicly exposed Hub role
and method payload; it does not infer internal execution contexts from those
edges. It also cannot establish that a particular Claude installation loaded
the rebuilt executable or that the original proprietary workspace is fixed.

After this local binary/stdio batch passes, consolidate any remaining questions
into one final Claude pilot of the original action/ancestry scenarios. Keep
company source local; report sanitized results only. Do not close #153/#154
based solely on harness authorship or an incomplete run.
