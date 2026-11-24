# E4 bounded direct identity evidence

Agent MaxHeadRoom. Opt-in `written_source_observations` feature only. No public
v2 tool, canonical registration, persistence or authoritative projection consumer.

Rust API: `clean_ctx::ir::source_observations::direct_identity::capture` consumes a
reference to an independently admitted E2 observation, its exact record index, the
same E3 Request, explicit authorized roots and explicit retained-work envelopes.
The observation stays owned by its caller. Returned Evidence has private proof;
`validate` charges final witness validation to that request. Accessors alone are
snapshot inspection, not fresh authority. Classification reads no filesystem.

Supported direct envelope: module-level value declarations, named direct imports
including aliases, direct named re-exports/local export aliases and one imported
namespace member. Existing Unit import/export extraction and physical module
resolution are reused. Function/block/namespace shadow scopes refuse; arbitrary
alias evaluation, star export closure, merging and contextual/ambient closure are
not implemented. Imported endpoints always retain a contextual-contributors
obligation. A physical package export route is not a proven full TypeScript binding.
Direct support also conservatively refuses consumed sources containing Unicode
escape sequences until equivalent escaped binding names are certified. This can
refuse an otherwise harmless escaped string; earlier E2 observations are retained.

Direct request-bound Host reads reserve expected support bytes plus one growth
detection byte before reading, and cap the read to that reservation. Directory
enumeration checkpoints and charges each entry to the same edge/memory ledger.
Ordinary v1 hosts have no attached direct budget and retain existing paths.
Source parsing, binding hops and physical resolution have separate reservations.
Retained parser/result envelopes are caller declarations, not exact heap accounting.
Existing final witness validation is a non-interruptible quantum and can exceed
expected work if a dependency or directory grows concurrently; a changed proof
is refused. No hard deadline or allocation interception is claimed.

Final reproduction commands used from PowerShell 7 at the repository root:

```powershell
$env:OPS_E1_RUN='e4-build-r5-20261123'
python verification/ops-r1/e1/measure.py build
$env:OPS_E4_BUILD='e4-build-r5-20261123'
$env:OPS_E4_RUN='e4-complete-20261123'
python verification/ops-r1/e4/measure.py tests
python verification/ops-r1/e4/measure.py perf
python verification/ops-r1/e4/measure.py controls
python verification/ops-r1/e4/measure.py lint
python verification/ops-r1/e4/analyze.py
```

The recorded paths are frozen; choose fresh output names for future authorized
runs. Builds/lint are externally limited to 600 seconds/8 GiB aggregate job committed
memory. Test/measurement children use 30 seconds/512 MiB. The Windows Job Object
capture's post-spawn assignment/watchdog window remains a documented limitation.
Check child receipt exit codes, not just Python wrapper completion.

Five measurement children each resolve actual local/imported source plus missing
and exhausted-budget cases. These execute real Unit parsing/module resolution and
proof validation, not synthetic scheduling sleeps. Original fixture bytes/hashes
are copied under each receipt's fixtures directory; recorded original physical IDs
refer to temporary workspaces that are later retired, not to those copied paths.
Receipts are historical observations after retirement. E4 source/target identity
cannot be inferred from fixture-copy paths. Synthetic malformed @types/node has
version 0.0.0-invalid and is not an exact real Node grammar reproduction.

Regression evidence is behavioral and counter-based for this new staged API, not
a claimed existing-v1 defect RED/GREEN repair. Preserve E2's earlier RED/GREEN
separately. No workload here is originating-repository or field acceptance.
