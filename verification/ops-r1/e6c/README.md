# E6-C controlled public-boundary verification

Agent MaxHeadRoom. All measurements are synthetic local MCP stdio evidence,
not field acceptance for #190/#191. No production budget default is selected.

## Activation boundary

The Cargo feature `resolver_evidence_v2` is not a default feature. It implies
the accepted written-observation and Angular support. Public v2 additionally
requires the absolute `CLEAN_CTX_RESOLVER_EVIDENCE_PROFILE` environment path at
server startup. Its closed JSON object is:

```json
{"schema":"clean-ctx/resolver-evidence-operator-v1","ceiling":{"elapsed_ms":10000,"source_bytes":2097152,"support_bytes":2097152,"retained_bytes":67108864,"active_expensive_tasks":2,"traversal_units":20000,"traversal_edges":4000,"validation_bytes":8388608,"validation_checks":20000,"response_bytes":1048576,"finalization":{"elapsed_ms":2000,"validation_bytes":4194304,"validation_checks":10000,"response_bytes":1048576}}}
```

These numbers define the **larger lab candidate only**. The focused lab candidate
is recorded in `stdio.py` and the generated operator file. Both require owner
review before any deployment. Missing, relative, unreadable or invalid profiles
leave v2 disabled and unadvertised. Configurations are session-pinned; restart
after operator changes. Caller budgets cannot supply internal allocation formulas.

Effective fields are bounded by both caller request and operator ceiling. Initial
within-request execution is sequential (effective active expensive tasks = 1).
Protected finalization remains inside those totals. Cross-request scheduling is
the existing dispatcher; this increment adds no global admission policy.

## Reproduction

Use fresh evidence directory names and explicit owner authorization for runs.
From the repository root:

```powershell
$env:OPS_E1_RUN='e6c-build-REVIEW'
python verification/ops-r1/e1/measure.py build
$env:OPS_E6C_BUILD=$env:OPS_E1_RUN
$env:OPS_E6C_RUN='e6c-verification-REVIEW'
python verification/ops-r1/e6c/run.py tests
python verification/ops-r1/e6c/run.py controls
python verification/ops-r1/e6c/run.py lint
python verification/ops-r1/e6c/run.py binary
$env:OPS_E6C_BINARY_RUN=$env:OPS_E6C_RUN
$env:OPS_E6C_RUN='e6c-full-REVIEW'
$env:OPS_E6C_PROFILE='full'
python verification/ops-r1/e6c/run.py stdio
python verification/ops-r1/e6c/audit.py $env:OPS_E6C_RUN
$env:OPS_E6C_RUN='e6c-focused-REVIEW'
$env:OPS_E6C_PROFILE='focused'
python verification/ops-r1/e6c/run.py stdio
python verification/ops-r1/e6c/audit.py $env:OPS_E6C_RUN
```

Run timings after builds, lint and guards finish. Test/stdio watchdogs: 60 seconds,
512 MiB aggregate committed memory. Build/library Clippy: 600 seconds, 8 GiB.
The common Job Object launcher has a documented assignment-after-spawn window.
The stdio client samples the actual server working set at 10 ms, while the outer
launcher samples its Python parent and limits the whole descendant process tree.
Client and server memory are reported separately. Sparse samples can miss peaks.

The client preserves initialization, tools/list, actual tools/call request/response
bytes, per-request timings, configuration and fixture identities. It verifies
both channels against one DTO and the exact approved schemas. It asserts that
the launch configuration parsed, persistence stayed disabled and no CBM child
started. The correct configuration tokenizer spelling is `cl100k`.

`run.py red` targets the archived **pre-correction** memory-finalization executable
only and expects exit 101. It is not a passing-test command for the final binary.
Final GREEN uses the same tracked assertion with transport memory prepaid.

## Accounting and interpretation

One top-level ledger serves every v2 batch item; there are no item-local budgets.
The public adapter reserves both response-channel capacity and conservative
DTO-copy/wire-buffer memory before optional expansion. An owned response credit
distinguishes pending capacity from completed serialization and refunds unused
capacity. Serialized byte charges include the internal DTO and full public
JSON-RPC envelope, escaping, duplication and a conservative framing margin.
Large response maxima therefore consume a larger retained reservation envelope.
This coupling is visible in measured peaks, not a hard allocator guarantee.

Entire records can be withheld while their established counts and predicate
qualifications remain. An open empty result is unknown. Canonical collections
remain empty and are not queried or populated by v2. A proven written mapping or
physical declaration endpoint is not a proven TypeScript runtime binding.

The malformed Node package is version `0.0.0-synthetic`; the synthetic Angular
declarations are version 20.0.0. Neither reproduces the exact #191 field bytes.
The legacy comparison's cross-worker collector is test-only because hydration
uses Rayon; caller-thread E1 counters alone cannot count its acquisitions.
Public v2 runs the accepted synchronous Session path on its request worker.

## Full integration gate

The existing authoritative CI entry points remain `.github/workflows/ci.yml`,
`docs/agent/verification.md` and `scripts/check-semantic-work-fitness.ps1 -Consolidated`.
Do not replace the consolidated full gate with these focused filters.

Prepare an isolated candidate matching the recorded source hashes, preserve the
absence of a production opt-in profile, then run the existing CI procedure:

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
pwsh -NoProfile -File scripts/check-tree-sitter-versions.ps1
pwsh -NoProfile -File scripts/tests/check-file-sizes.tests.ps1
pwsh -NoProfile -File scripts/check-file-sizes.ps1
pwsh -NoProfile -File scripts/check-utf8.ps1
pwsh -NoProfile -File scripts/check-semantic-work-fitness.ps1 -Consolidated
```

The existing workflow also owns cargo check, dependency audit and archived fitness
evidence. No workflow is triggered by this harness. Full acceptance remains pending
those gates and the separately reported inherited formatting blocker, including
the frozen E2 source identity. Do not apply a formatting exemption or alter frozen
source bytes to bypass that blocker. The supplied normalization diff is unapplied.
