# PCG-175 PILOT 01: CBM cache architectural continuity

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: owner-approved Phase B explicit enrollment and Phase C real-use MCP
evaluation of the reviewed corpus/profile. PCG v1 development and verification
remain closed. No new product feature, production enrollment, commit or publication.

Assessment: **USEFUL — for this owner-curated, pinned architectural question.**
PCG returned the complete reviewed constraint/proof/evidence set in a fresh
production MCP session. Its returned bundle supports the qualified answer below
without reopening the original source files. This demonstrates retrieval and
continuity value after curation, not net engineering-time savings, autonomous
architectural reasoning, live CBM correctness or enterprise efficiency.

## 1. Approval, source identities and explicit enrollment

The [Phase A proposal](PCG_175_PILOT01_PROPOSAL.md) supplied the independent answer
oracle before execution. The owner approved these exact, unchanged artifacts:

| Item | Actual identity |
| --- | --- |
| Pinned source commit | `6add8449c1a036d0599283fd6a982009d495e954` |
| Manifest SHA-256 | `a570c20629da16c422d60698ff4e7aae738d29855033dfa5a447e93f6360870d` |
| Profile SHA-256 | `4b33647c87fff76759c24bf7b13a2e0c7c437653da83196ef0f25bb7872cc499` |
| Verified server binary SHA-256 | `0959d317624514e5cb9e1d79e36e5ba1c48587f7ac84496d31b7edec1d29f76c` |
| Actual enrolled generation | `9cd113849a41cc4edc60304105b1fffa2fdf55df67365cd25c97f855ea16ee69` |

The actual generation **exactly matched** the proposed static prediction. No
manifest/profile/source/record identity was changed to make enrollment succeed.
All seven selected Git blob OIDs/full hashes and all 18 excerpt hashes/ranges
were rechecked against the pinned commit before import. Original selected working
files still matched their Phase A working-byte hashes, including their existing
newline representation.

Created `target/pcg-175/pilot01/workspace` as an isolated minimal Git repository,
with detached HEAD at the pinned commit and all seven files written from exact
blob bytes. Its immutable objects are read through a local alternate pointing to
the original repository's object store; no fetch, network or new source commit
was used. The original repository's files, index and refs were not changed.
All seven manifest sources use `check=current` against this exact-byte snapshot,
not historical flags or normalization to bypass original-checkout CRLF differences.

Copied the accepted non-provider M4a fixture startup configuration into the
isolated snapshot: CBM disabled, proxy auto-start disabled, ordinary source
persistence disabled and cache hints disabled, with no additional roots. No
existing accepted fixture was modified. This config is not an enrolled source.

Executed the existing binary's explicit import command:

```powershell
./target/debug/clean-ctx.exe pcg import --profile C:\Users\MNasty\Desktop\RustContextLayerAI\target\pcg-175\pilot01\review\profile.proposed.json
```

Exit 0, operation `import`; process wall time 2.316s. Importer diagnostics reported
7 current sources in each observation, 331,332 Git bytes, 662,664 working-file
bytes across both observations, 31,048 evidence bytes, 18 evidence validations,
34 definitions and 1,023,974 total accounted bytes. This was the authorized derived
projection enrollment, not a runtime ledger or approval write.

Storage is isolated at `target/pcg-175/pilot01/storage/projection.sqlite`.
Enrollment/source receipts are retained in
[execution/](../../target/pcg-175/pilot01/execution/): `preflight.json`,
`snapshot.json`, `import-process.json`, `import.stdout.json`, `import.stderr.txt`
and `enrollment-identity.json`. No enrollment or witness failure occurred.

## 2. Fresh production MCP execution

Launched the unchanged verified `clean-ctx.exe` once with:

- `CLEAN_CTX_PROJECT_ROOT` = the isolated snapshot root.
- `CLEAN_CTX_PCG_PROFILE` = the exact approved profile path.
- Fresh stdin/stdout MCP session, initialize/initialized handshake and tools/list.
- Explicit `pcg_query_v1` calls for constraints, closure, provenance and impact.

Each request used the actual enrolled generation and registered subject:
workspace `cleanctx-cbm-cache-pilot01`, operation `graph_query`, task
`explain_project_cache`, effective time `2026-11-11T12:00:00Z`, o200k,
`includeHistory:false`, records/candidates 512, edges/refs 1,024, evidence bytes
4,194,304 and whole-result token budget 32,768. Closure/provenance named
`cbm-refresh-failure`; impact named `cbm-canonical-identity`, exactly as proposed.
The effective time is an explicit applicability input, not an execution timestamp.

| Mode | Actual state | Whole-result o200k tokens | UTF-8 result bytes | Observed request wall time |
| --- | --- | --- | --- | --- |
| constraints | complete_within_registered_scope | 13,642 / 32,768 | 50,776 | 1.339s |
| closure | complete_within_registered_scope | 13,642 / 32,768 | 50,776 | 1.435s |
| provenance | complete_within_registered_scope | 13,642 / 32,768 | 50,776 | 1.366s |
| impact | complete_within_registered_scope | 13,642 / 32,768 | 50,776 | 1.361s |

All five coverage booleans were true; START and COMPLETION both reported matched.
`required_material_fit:true`, `optional_truncated:false`, no required issues,
conflicts or mandatory omissions. Optional omission code was `not_requested`
for unrequested history; optional support/history arrays were empty. No budget
overflow, stale witness, incomplete acquisition or protocol error occurred.

All four result byte sequences were identical, SHA-256:
`db024b9e4bb1966387bf7d6b3f429d7c6145b2e1e7f12d4e12ea0f091ff8b99b`.
This is expected from seven constraints all mandatory in the registered scope:
the explicit roots/modes do not remove other applicable mandatory roots. The
additional modes did not produce smaller or differently focused packets here.
These are four requests for one question, not independent workload samples.

Server exit was 0. The main SQLite file SHA-256 was identical before and after
MCP reads: `1616fc626ebb42b786a7ecd81f699248f93e2a123d46bfa7b2209bc783983c8a`.
This is a main-file nonmutation observation, not physical/WAL erasure or a claim
that every OS-side file was monitored.

## 3. Actual required bundle and citation checks

Every mode returned exactly these seven required records, with exact proposed
record revisions, `authority.assurance=source_attributed` and execution `not_run`:

1. `cbm-canonical-identity`
2. `cbm-project-owned-cache`
3. `cbm-target-invalidation`
4. `cbm-refresh-failure`
5. `cbm-errors-not-empty`
6. `cbm-semantic-authority`
7. `cbm-query-route`

All 15 expected relationships were retained: seven CONSTRAINS bindings and eight
DEPENDS_ON assertions. Exact kind/from/to/provenance/revision checks matched the
approved manifest. Recoverable proof paths include:

```text
refresh-failure -> target-invalidation -> project-owned-cache -> canonical-identity
refresh-failure -> errors-not-empty
query-route -> canonical-identity
query-route -> errors-not-empty
```

These paths are read from the returned typed edges; no separate server-generated
path narrative is claimed. Semantic authority remains independently required
context rather than an invented causal premise of invalidation.

All 18 expected citation IDs appeared once, shared across records and relation
provenance. For every citation, decoded UTF-8 text matched the exact pinned Git
span and excerpt SHA-256; commit, blob, source hash/path and byte range were
checked independently against approved inspection bytes. Every record's primary
and required evidence-ref citation set matched. There were no dangling or missing
required witnesses. Verification categories were not promoted to executed receipts.

Machine-readable checks are in
[integrity-assessment.json](../../target/pcg-175/pilot01/execution/integrity-assessment.json).
[returned-evidence.md](../../target/pcg-175/pilot01/execution/returned-evidence.md)
is a decoded readability view of the actual captured citations, not another MCP
response, new source authority or product summary. Original result bytes remain
in each `<mode>.result.json`; full JSON-RPC lines remain separately captured.

## 4. Independently assessed answer to the engineering question

**Target B must own the query's identity and cache partition, even when A was
active on entry.** Resolve B through the canonical root/project mapping, not a
guessed directory-derived ID. An exact configured-root basename can be an alias
that resolves to the canonical slug; it is not itself the canonical identity.
An unknown literal is left for CBM to reject, not assigned an invented disk root.
Supporting returned evidence: [inv-identity](../../target/pcg-175/pilot01/execution/returned-evidence.md#inv-identity),
[overview-identity](../../target/pcg-175/pilot01/execution/returned-evidence.md#overview-identity),
[plan-owners](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-owners).

**Memory and disk ownership follow B.** Typed memory keys carry B, and disk
lookup/write-through uses B's registered root in `project_paths`, never A's
active root. Unregistered projects have no invented disk partition. A successful
B refresh removes all and only B's graph-cache entries/partition; switching
selection does not justify discarding unrelated safely owned results. The
invalidation helper itself does not advance generations, change the active
project or contact CBM. Evidence: [inv-cache](../../target/pcg-175/pilot01/execution/returned-evidence.md#inv-cache),
[cache-lookup](../../target/pcg-175/pilot01/execution/returned-evidence.md#cache-lookup),
[cache-invalidation](../../target/pcg-175/pilot01/execution/returned-evidence.md#cache-invalidation),
[plan-invalidation](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-invalidation).

**Freshness is a per-project, gated success boundary.** Edits remain lazy and
coalesced. Capture B's dirty generation, call the repository-scoped fast refresh,
advance the indexed generation only on success, invalidate B's cache, then allow
the gated query. Refresh failure propagates before advancement/invalidation-as-
success and leaves B dirty. Retained pre-refresh cache bytes are not authorization
to return stale success. Existing circuit/readiness policies govern later retry;
no new retry, TTL, schema, file-scoped indexing API or global lock is licensed.
The existing bridge guard serializes this orchestration. Evidence:
[plan-decision](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-decision),
[plan-refresh](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-refresh),
[index-target](../../target/pcg-175/pilot01/execution/returned-evidence.md#index-target),
[plan-controls](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-controls),
[plan-approval](../../target/pcg-175/pilot01/execution/returned-evidence.md#plan-approval).

**Public and internal scoped calls require different wording.** The pinned public
`graph_query(project=B)` handler selects B before checking the active readiness/
freshness gate and invoking the query. It does not promise that A stays active
after the request. Internal `query_graph_scoped` resolves B and keys the cache by B
without itself selecting B, but its cache lookup is not an autonomous freshness
gate: its caller must establish the appropriate gate. The non-active gate also
has an additional-root-not-started branch that consults primary readiness before
starting that root. Therefore an unconditional claim that A can never affect any
B startup readiness would be too strong. Evidence:
[handler-target](../../target/pcg-175/pilot01/execution/returned-evidence.md#handler-target),
[handler-query](../../target/pcg-175/pilot01/execution/returned-evidence.md#handler-query),
[query-owner](../../target/pcg-175/pilot01/execution/returned-evidence.md#query-owner),
[index-target](../../target/pcg-175/pilot01/execution/returned-evidence.md#index-target).

**Failure is not valid empty data.** Some low-level graph query methods carry
failure through `last_error` alongside an empty carrier. The actual MCP handler
consumes that error and emits an error result; an empty carrier cannot establish
a successful empty graph. A refresh error exits the freshness gate before the
normal graph-query/cache path. Evidence:
[inv-errors](../../target/pcg-175/pilot01/execution/returned-evidence.md#inv-errors),
[query-owner](../../target/pcg-175/pilot01/execution/returned-evidence.md#query-owner),
[handler-query](../../target/pcg-175/pilot01/execution/returned-evidence.md#handler-query).

**CBM discovery is not semantic authority.** CBM may discover candidates;
`WorkspaceIndex` remains the live semantic-answer authority, with canonical IR
and source evidence retaining their separate owners. A cached CBM result or
discovery completion cannot promote itself into authoritative semantic truth.
Evidence: [overview-authority](../../target/pcg-175/pilot01/execution/returned-evidence.md#overview-authority).
Startup statements also retain the [CBM-LIFE-001 qualification](../../target/pcg-175/pilot01/execution/returned-evidence.md#inv-lifecycle).

These conclusions are source-defensible constraints/observations, not observed
behavior of two running CBM projects. No real A/B provider refresh was induced.

## 5. Usefulness and limits

The independently prepared oracle's eight claim groups were supported, with the
alias, public/scoped route, primary-startup and error-carrier qualifications
preserved. The assessment used the captured bundle plus pre-enrollment oracle
and exact independently retained spans. It was not a blinded second-reviewer
experiment. The answer above was assembled from the returned evidence; original
repository files did not need reopening to supply its citations or conditions.

The practical value demonstrated is one explicit fresh-session retrieval that
recovers the reviewed policy set, provenance and prerequisites together, rather
than another navigation pass across three large documents and four source files.
It is useful architectural continuity after curation. The tool did not generate
the human answer or adjudicate the documentation itself.

Important limits:

- Corpus discovery, classification, dependency selection and conflict qualification
  required expert manual preparation. This cost was not compared with later
  reuse, so no net effort/time saving is established.
- The packet is 13,642 tokens and still requires interpretation. All four modes
  were identical in this mandatory scope; they offered no smaller focused view.
- Completeness is within this registered source universe, not all policies in the
  repository or every possible CBM scenario. An unenrolled obligation could be
  missed by the curator; typed graph consistency cannot prove exhaustive prose.
- Freshness describes the pinned isolated snapshot at two observations, not the
  original working tree, live A/B indexing state or continuous/atomic currentness.
  The snapshot depends on retained local Git objects through its alternate.
- Source approval remains attributed, not authenticated runtime approval. A source
  report's execution claims were not rerun or independently certified.
- One pinned code comment contains a pre-existing encoding artifact (code points
  U+00E2/U+2020/U+2019). It was confirmed in returned source bytes and preserved
  exactly, not repaired or normalized. A console display attempt initially hit
  Python's cp1252 output boundary; switching that display to UTF-8 changed no
  captured data. Exact citation integrity does not guarantee clean source prose.
- The token count was obtained with existing `measure.exe count o200k <file>` on
  the literal result substring, including escaped text/wrapper/metadata and
  excluding JSON-RPC framing/newline. Counter SHA-256 is
  `92ca8701af027627e51f87a02d566a65241d859e1174d57e034ad98a35864382`;
  its tracked source uses the existing tokenizer factory. It was not rebuilt or
  newly source-bound in this pilot; its binary identity and output are retained.
  No character estimate or newly implemented tokenizer was substituted.
- Timings are four observations for one question in one session, not latency
  distributions or a benchmark. No token-saving percentage is inferred by
  comparing entire source files with a selected evidence packet.

No retrieval, citation, enrollment-identity or budget defect was observed. No
repair, weakened witness, altered approved artifact, new adapter or product change
was needed. The use case succeeded within the stated scope; broader effort and
deployment benefits remain unmeasured.

## 6. Retained evidence and final state

Execution directory: `target/pcg-175/pilot01/execution/`.

- `mcp.stdout.jsonl`, `mcp.stderr.txt`, `tools-list.json`: actual fresh process streams/catalog.
- `<mode>.request.json`, `.response.json`, `.result.json`, `.payload.json`: exact
  requests, response framing, counted result bytes and decoded payloads.
- `<mode>.token-count.{stdout,stderr}.txt`: actual independent counter output.
- [mcp-results.json](../../target/pcg-175/pilot01/execution/mcp-results.json): per-request state, coverage, IDs, sizes, counts and timings.
- [mcp-session.json](../../target/pcg-175/pilot01/execution/mcp-session.json): binary/counter identities, process exit, unchanged main DB and approved digests.
- [usefulness-assessment.json](../../target/pcg-175/pilot01/execution/usefulness-assessment.json): human oracle judgments, evidence IDs and limitations.

Selected receipt SHA-256 values:

| Receipt | SHA-256 |
| --- | --- |
| enrollment-identity.json | `fbf14532cea5da01725e417f13b67bc3b1703bf9863eb632b8bcfb3a30cd76d4` |
| mcp-results.json | `261e8aa4e2f41a112b46f9bede75476ea081d6db6f4c616e4335ba37558647ee` |
| mcp-session.json | `ed46323c31552b43aa4170b4e7f9b11599a1929dcafb93bc9837abf63242da09` |
| integrity-assessment.json | `a86500bf2a175199761ccf8f74d6698e747012da9cea07e5ec058cc37d427d1f` |

Local source HEAD remains `6add8449c1a036d0599283fd6a982009d495e954`.
Approved manifest/profile digests, original selected-source working hashes and
the accepted M4a source/definition inventory are unchanged. Only pilot receipts,
the isolated approved snapshot/projection and this result report were created.
No accepted tests, fixture definitions, production source, original Git index/ref,
or production enrollment changed. No commit, push, PR or publication occurred.

No closed verification campaign, build, lint, benchmark, CI gate, runtime ledger,
approval write, automatic injection or entity adapter was introduced or run.
This is operator real-use evidence, not a new tracked regression or CI result.
**Final judgment: USEFUL for the bounded curated-context objective; stop here.**
