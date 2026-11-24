# Gated E2 source-observation verification

Agent MaxHeadRoom. Internal Rust capability only; no public v2 operation or persistence.

Feature `written_source_observations` is opt-in and implies TypeScript. It is absent
from Cargo default features. The public Rust path is
`clean_ctx::ir::source_observations`; this is not an MCP/public query integration.
Candidate capture reads one authorized disk source, extracts written object shapes,
and retains exact source/hash/spans and typed witnesses. `admit` validates only those
inputs. `Admitted::is_current` must be called at a later use boundary; admission is
not a perpetual freshness certificate. Configuration is not consumed and does not
affect this purely syntactic claim. No canonical target identity is provided.

Recorded execution commands, from PowerShell 7 at repository root:

```powershell
$env:OPS_E1_RUN = 'e2-red-20261123'
python verification/ops-r1/e1/measure.py build
python verification/ops-r1/e2/measure.py red
# Frozen regression stashed; source-only correction applied; same bytes restored.
$env:OPS_E1_RUN = 'e2-green-20261123'
python verification/ops-r1/e1/measure.py build
python verification/ops-r1/e2/measure.py green
python verification/ops-r1/e2/measure.py extra
python verification/ops-r1/e2/measure.py perf
python verification/ops-r1/e2/measure.py lint
python verification/ops-r1/e2/analyze.py
```

These output directories are frozen and cannot be reused. Future runs require
fresh output paths in the runner. Execution requires repository-policy authorization.
Builds use all features, a 600-second watchdog and 8 GiB aggregate committed-memory
ceiling. Focused children use 30 seconds/512 MiB, performance children 15 seconds/512
MiB. Job assignment occurs after spawn; existing watchdog enforcement limitations
remain recorded. Inspect child receipt exit codes, not only wrapper exit status.

The RED adapter exercised existing TypeScriptAuthority capture through the new
mechanical interface; its isolation assertion found one program acquisition.
GREEN removes that prerequisite without changing v1. The exact same regression
file hash is preserved. This is RED/GREEN for the new isolation contract, not proof
that existing v1 was obliged to provide source-only answers.

Source-only measurements reuse frozen E1 fixtures, including deliberately invalid
synthetic dependencies, and do not reproduce a particular real Node grammar gap.
Thirty samples per each of eight fixtures are warm-cache sequential observations
inside fresh test processes. They are not independent cold enterprise benchmarks.
Raw receipts, executable/source manifests, full records and timing/counter data are
under target/ops-r1/e2-implementation-20261123 and the red/green build directories.
