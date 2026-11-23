# OPS-R1 Directive 33 — originating-machine comparison

Agent: **Agent MaxHeadRoom**. Prepared 2026-11-23. Incident #190 stays OPEN.
Current disposition: **historical/superseded as an active handoff**. The owner
confirmed the actual workspace uses `types: []` and requires direct testing in
that repository. These frozen binaries predate the unified effective-input path.
Do not use this packet as validation of the current source or as a #190 fix.
The original procedure below is retained for provenance only.
Owner disposition: retain the correction for review; require matched field evidence.

## What this packet establishes

One baseline query followed by one candidate query against the same affected
Angular workspace, exact effective JSON configuration and absent route selector.
Both use the real public dispatcher in fresh isolated libtest processes with full
acquisition telemetry. This is real-workspace diagnostic evidence, not production
stdio, Claude consumption, CI, leak proof or automatic incident closure.

`resolver_registrations` remains acquire-then-query. Both binaries nominate and
prepare eligible owners before consuming admitted evidence. Empty output must keep
`source_complete=false`, `omitted_possible=true`, `result_semantics=lower_bound`.
Do not convert a refusal, incomplete capture or resource stop into empty success.

## Preparation on the originating machine

Keep all proprietary sources, fingerprints, responses, logs and memory samples on
that machine. Return only an owner-sanitized summary. Nothing is sent automatically.
Pause editors, generators, dependency installs and repository mutations for the
pair. Do not run this through the owner's active Claude/server session.

Unpack `field-packet-33.zip` into a disposable existing Clean-CTX Git checkout on
the work computer. It overlays only the named verification definitions and
`target/ops-r1/cost-33` evidence/artifacts. Do not overwrite unrelated local files.
The Python launcher needs a Clean-CTX checkout for its host-source drift check;
that checkout's current files are NOT the compiler identity of the two executables.
Their separate frozen compiler manifests/build logs are pinned in
`BUILD_IDENTITIES_33.json`.

The packet contains explicitly built, all-feature Windows x64 test executables.
They must match both SHA-256 pins below; do not select a newest executable or
substitute the connected production server. No rebuild is needed when these exact
artifacts run on the work machine. If incompatible or unavailable, stop before
querying and report the prerequisite; rebuilding requires new actual binary pins.

| Case | Executable SHA-256 |
| --- | --- |
| Baseline | `7844e4662d673152aceb22373f389f87b9c396e0ff32994d79c58a433cad2bad` |
| Candidate | `3af4e4523adafd58b7746cbf2363949c640e8c13847fd6fdb350d5bf468fb01c` |

Supply the exact affected root and an explicit effective configuration JSON file.
Include existing semantic options, exclusions and additional roots; every
`additional_roots` entry must be an absolute directory path. Do not replace the
real configuration with an empty/default JSON merely to get a successful run.
The diagnostic intentionally disables CBM, persistence and proxy startup in both
processes while preserving semantic options. Record those differences from the
normal production configuration when interpreting results.

## One command from that Clean-CTX checkout

PowerShell 7; Python 3.12 or newer. These prompts acquire only local paths/text:

```powershell
python .\verification\ops-r1\field\check_pair_preflight_33.py
if ($LASTEXITCODE -ne 0) { throw 'Preflight failed; do not run a field query' }
$affectedRoot = Read-Host 'Exact affected Angular workspace root'
$effectiveConfig = Read-Host 'Absolute path to effective configuration JSON'
$absentRoute = Read-Host 'Exact route selector established to be absent'
python .\verification\ops-r1\field\compare_cost_33.py --workspace $affectedRoot --config $effectiveConfig --route $absentRoute
```

The packet uses one fresh process and `McpState` per case, with persistence
disabled. That is application-cold state. It does NOT clear OS caches; complete
input fingerprinting reads the files before each case and warms filesystem caches.
Do not describe these measurements as cold-disk performance.

Before and after each query, a separate child fingerprints every file/directory
under every configured source root, including dependency contents and metadata;
there are no silent directory exclusions. Config bytes are included. The four
fingerprints must match. Internal links are recorded and their in-root referents
are covered by traversal; external link targets refuse preflight. Fingerprint
children have 120-second deadlines, sampled 512 MiB working-set ceilings,
a 200,000-entry ceiling and a 20 GiB read
ceiling. Hitting a limit is incomplete acquisition: stop, preserve the receipt,
and do not trim the manifest to manufacture equivalence. Fingerprinting cannot
prove absence of a mutate-and-revert event between observations; the freeze
precondition and query's native witness/currentness checks still matter.

Each query has an independent 60-second deadline, sampled 512 MiB working-set
ceiling, 100 ms memory samples and five-second termination wait. A separate
120-second launcher deadline stops only its owned process tree if needed.
No product timeout, budget, concurrency or completeness policy changes.

A baseline resource stop may be followed by the one candidate query only after
the child is confirmed terminated and inputs remain identical. Monitoring errors,
identity drift, input drift, unexpected refusal, unconfirmed termination or a
candidate resource stop end the pair. No retries or warm follow-up queries.

## Evidence and return

The command prints `target/ops-r1/cost-33/field-pair-<timestamp>/`. Preserve this
directory locally, including `pair.json`, four complete fingerprints and both
case subdirectories. Each case retains available stages, source identity, exact request,
response when completed, process identity, wall time and memory series.

`PAIR_CAPTURED_REQUIRES_REVIEW` means the bounded comparison was captured, not that
the fix is accepted. Review nominations/compilations, acquisition bases, actual
acquisitions/shares, concurrency, proof/job/result counters, query completion,
elapsed time, sampled memory and unchanged unknown-absence semantics. When both
queries complete, structured answers must be exactly equal. A stopped baseline has
no full answer for equivalence; do not invent one.

Return a sanitized summary of those observations, executable/config identities,
input-fingerprint equality, stop/termination outcomes, and local evidence location.
Nonempty automatic types can keep acquisitions directory-specific; this correction
does not promise improvement outside its proven empty-discovery boundary.

After review, batch any remaining production stdio/Claude-only questions into one
final isolated field session. This packet alone does not exercise that gate.
No commit, push, merge, publication or #190 closure is authorized here.
