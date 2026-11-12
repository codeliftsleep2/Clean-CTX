# AF-PERF-002: registered diff_commits discovery and replay apparatus retirement

Agent: Agent MaxHeadRoom
Status: cleanup and discovery executed GREEN; measured Git candidates DEFER.
No production optimization or repository publication authorized.

## Rejected replay apparatus retirement

Owner accepted REJECT for earlier-Candidate reuse and authorized retaining a focused
regression while retiring candidate comparison/timing machinery. The production
replay selection is now a single original expression in both release and tests;
there is no Candidate reuse switch or alternate implementation. Discovery counters,
comparison modes, timing tests and their two operator drivers have been removed.
Only one cfg(test,typescript) one-shot hook remains immediately after economics.

The replacement tracked test is replay_footer_observation_af_perf_002.rs, registered
from replay.rs. It runs actual registered compression/replay and three controlled
mutations: unrelated alias insertion, exact alias removal and registered deletion.
It requires the complete original response with the exact later footer, text-cache
equality and unchanged held canonical tuples/version/hash. The expected text is
derived from the first candidate by removing ONLY the exact requested alias suffix
when the alias is removed; no response normalization or alternative is executed.
Unrelated insertion must leave the complete response unchanged. A RAII reset clears
the hook on unwind. The test protects the later rendering observation and does not
create a new lifecycle guarantee for successful replay after deletion.

Both investigation reports and original executed receipts remain. Seven retired
source artifacts were copied before removal into the existing design receipt's
retired-source-evidence directory, with exact hashes and matched-execution flags in
inventory.json. These are historical source evidence, not active test definitions,
portable CI coverage or newly executed results. Existing historical report commands
are retired; do not rerun them. The replacement tracked test is the active contract.
No rejected source was silently converted into production behavior.

## Next registered production trace

create_default_registry binds diff_commits to tool_handlers/gitdiff.rs. The handler
resolves a trusted workspace root, validates refs, checks Git work-tree admission,
parses fidelity and passes existing configured file-count/size limits to
gitdiff_workspace. The engine invokes collect_changed_files; that function uses
git diff --name-status --find-renames and, only for working-tree requests, git
ls-files --others --exclude-standard. It validates paths and sorts changes before
the engine's existing file cap. No order, caps, ref security or partial/skip policy
is changed by this investigation.

The engine retrieves blobs through show_file_bytes/git show for committed content,
or std::fs::read for working-tree current content. NUL/non-UTF8 admission remains
binary; size limits retain original checks/order. Added/deleted/renamed/type-change
branches remain distinct. Compressible modified/renamed pairs build both snapshots,
call the unchanged protected diff/matching/formatting helpers, then copy
format_diff(...).trim_end() into another owned String. That owned body is consumed
by marker counting and manifest assembly; the hypothesis concerns its redundant
ownership, not deleting output, parsing or the marker pass.

For working-tree enumeration, collect_changed_files constructs an owned String
HashSet of all already-discovered paths before iterating ls-files output. This
happens even when untracked output is empty. The set enforces duplicate suppression;
its necessity and ownership must not be weakened simply because a fixture has no
untracked entries. Borrowing directly from a Vec that is appended to changes the
lifetime constraint; no borrowing correction is proposed from syntax alone.

The registered handler constructs full public manifest/metadata, comparison-loss
declarations and existing cache hints/tokenizer work. Manifest/cache-breaker/output
work remains necessary. Per-request Git process repetition is observable but does
not authorize batching, caching, fewer currentness reads or rewritten failure policy.

## Source hypotheses, not measured findings

| Candidate | Current status | Required evidence/boundary |
| --- | --- | --- |
| Owned diff body after trim_end | UNMEASURED | Count real body copies/logical bytes on registered modified/renamed calls; preserve exact Unicode trimming, text/headers/fidelity/errors and protected helper work |
| Working-tree owned duplicate-suppression path set | UNMEASURED | Count set entries/path bytes versus actual untracked output; preserve duplicates, deterministic ordering and Vec ownership; no global index/cache |
| Repeated Git acquisition and two AST snapshots | Required work pending characterization | Both comparison sides, subprocess errors, binary/UTF8 admission and limits have real consumers; repeated calls alone are not redundancy |

No serious candidate receives INVESTIGATE/FIX until the actual-path packet executes.
Recommend at most one bounded design after owner review of measured benefit and
semantic risk. Replay reuse remains REJECTED; no attempt to rescue it is included.

## Actual work populations and verification

Tracked tests use isolated temporary Git repositories. Fixture setup creates local
commits ONLY inside those generated repositories, using explicit test identity and
core.autocrlf=false. No repository staging, committing or pushing is performed by
the agent or operator driver. No hooks/network/remote interaction is needed.

Twelve independently controlled populations cover 0/1/8 files, 1/8/32 methods,
4096 six-byte Unicode body repetitions, 1/4 repeated requests, modified/added/deleted/
renamed/text/binary/invalid-UTF8 entries. Both committed-ref and working-tree requests
run through the actual registered tool: thirty complete public/work rows. A working
rename is staged only inside its temporary fixture so Git reports the intended
rename instead of separate deletion/untracked addition. Ignored fixture files must
not enter discovery. Repeated complete responses must remain equal, including cache
hints; diff requests must create no session aliases or persistence owner.

Three additional registered limit controls retain total file count while exercising
zero/one file caps and oversized content. Error controls preserve missing required
argument, unsafe ref and non-repository failure. Existing workspace/runner/engine
regressions and public comparison-metadata regression run separately. No alternative
is compared and no new optimization budget RED is claimed.

Observations are cfg(test,typescript) and record actual Git command attempts/results,
stdout/stderr lengths, failures, changed-file counts, constructed duplicate-set
entries/logical path bytes, untracked output bytes, snapshot attempts/offered source
lengths, text-admission calls/bytes and actual trim-to-owned body counts/lengths.
They do not count internal Git/SQLite work, filesystem disk reads versus cache hits,
parser CPU, allocator events or peak live bytes. Snapshot attempts are not successful
parse counts. Output bytes are logical String bytes, not copied source. No timing
or public latency claim follows these instrumented discovery runs.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-gitdiff-discovery.ps1
```

The user-owned gate runs warning-fatal all-target/all-feature Clippy, the focused
replay cleanup regression, three new Git discovery tests, existing public metadata
and Git workspace/runner/engine controls, formatting, size, strict encoding and
matching source identities. Require thirty public work rows and three limit rows.
Exact tracked pass totals are reported by the executed groups, not guessed from
legacy test counts. All 359 protected closure identities remain checked when the
local receipt is present. No mutation or composed closure is required for this
discovery-only packet.

Runtime gates remain UNRUN. Agent performed bounded source/parser/UTF-8 review only;
no tests, builds, formatter, lint, benchmark, binary or server launch. Protected
corrections stay closed. Second delta read/startup projection/dense equality remain
deferred; startup count removal and replay reuse remain rejected. Replay IR/save
snapshot hypotheses stay unmeasured. No unrelated investigation, production
optimization, staging, commit or push. Census unchanged.

## First attempt: first-emission versus warm cache hints

gitdiff-discovery-20261109-184618 reaches repeated eight-file requests, then fails
the fixture's unconditional complete-response equality assertion. Production emits
baseline cache_hints only on first emission and deduplicates the exact region/breaker
on later calls; the warm response legitimately lacks that field. Manifest and other
comparison metadata agree. cache_hints.rs explicitly documents and implements this
session-level policy. This is a fixture assumption, not a production discrepancy.

The test now requires a complete first response with cache hints, constructs the
exact expected subsequent response by removing only that first-emission field from
the EXPECTED value, and compares every actual warm response in full. Actual responses
remain untouched in receipts. Miss count must stay one and hit count equal the
repeat index. Cache remains enabled; no output is normalized away or policy changed.
The incomplete packet stays preserved and is not full discovery acceptance.

Rerun the same Git discovery command. Agent performed bounded log/source/encoding
inspection only; no tests/builds/formatter/lint/runtime started. Production remains
unchanged; cleanup and full discovery gates still require complete user execution.

## Executed cleanup and discovery return

Agent MaxHeadRoom inspected gitdiff-discovery-20261109-190157: 48 tracked passes
across six groups, zero failures/ignored. The focused replay regression passes in
11.24s; three Git discovery controls pass in 40.00s; public metadata control passes;
existing workspace/runner/engine groups contribute 13/6/24 passes. All thirty complete
public work rows and three limit rows are present. Warning-fatal all-target/all-feature
Clippy passes in 29.85s; both formatting logs empty; size passes seven active files;
strict encoding passes 2312 text files. All 1411 recorded source hashes match at
inspection before documentation update. inspection.json retains checked totals and
ranking. Prior failed discovery remains separate.

### Replay cleanup verified

The replacement tracked test executes unrelated insertion, direct exact-alias removal
and registered deletion at the post-economics boundary. Complete expected response,
later footer, cache equality and held canonical tuples/version/hash pass. The rejected
Candidate branch, modes and benchmarks are no longer compiled or active; historical
reports, receipts and seven archived source snapshots remain. No production behavior
change or new replay-after-delete authority policy follows cleanup. This regression
protects the second renderer's live dictionary observation, not a performance target.

### Production-path work evidence

| Population / operation | Observed work per request |
| --- | --- |
| One modified method/file | One owned diff-body copy, 44 logical bytes; two snapshot attempts |
| Eight modified files, one method each | Eight body copies, 352 bytes; sixteen snapshot attempts |
| One modified file, 32 methods | One body copy, 965 bytes; two snapshot attempts |
| One modified file, eight methods with 4096 Unicode body repeats | One body copy, 247 bytes; two snapshot attempts |
| Eight modified files, four repeated requests | 32 body copies / 1408 bytes and 64 snapshot attempts per ref mode |
| Unchanged rename | One body copy, 26 bytes; two snapshots |
| Working-tree eight modified files, no untracked output | One owned duplicate set, eight entries / 72 logical path bytes; empty ls-files output |
| Same committed-ref comparison | No working-tree duplicate set or ls-files call |
| Untracked added file | Positive ls-files output (10 bytes); new file admitted and one snapshot attempted |
| Binary/invalid-UTF8/text/deleted populations | No semantic diff-body copy; original branch-specific admission/retrieval retained |

The long source fixture does NOT imply a large formatted diff body. The unchanged
protected formatter emits method-change summaries at this fidelity/path; copying
247 output bytes must not be relabeled as copying hundreds of kilobytes of source.
Across thirty requests, actual body-site totals are 88 owned copies / 6084 logical
bytes. Working-tree totals are fifteen set constructions, 48 entries / 436 logical
path bytes. These aggregate fixture figures are not real-workspace request frequency.

Git subprocess work is separate: committed eight-file modification uses rev-parse,
diff and sixteen show commands; working-tree uses rev-parse, diff, ls-files and eight
show commands plus current-file filesystem acquisition. Every repeated request still
performs the recorded work. There is no measurement showing that these processes
or necessary snapshots are removable, or attributing CPU cost to ownership versus
Git/parse/presentation work. Host filesystem cache versus disk I/O is unmeasured.

Limits preserve total fileCount=8: zero-file cap skips eight and performs no blob
show/snapshot; one-file cap skips seven, retrieves two blobs and attempts two
snapshots; one-byte size cap skips eight after eight baseline show commands and
performs no snapshots. Existing short-circuit/error order is not rewritten. Ignored
files stay out, rename classification is controlled, binary/UTF8 handling and security
regressions pass. Cold responses emit existing cache hints, warm responses deduplicate
them with exact hit/miss transitions; full actual responses remain in evidence.
No session alias or persistent context ownership is created by the diff requests.

### Ranked disposition and owner recommendation

1. **DEFER — trim-to-owned diff-body copy.** A real redundant ownership candidate
   exists, and an in-place Unicode-equivalent trim might be a narrow future design.
   Observed copies are only 26..965 bytes per file in these populations; source-body
   growth did not translate into large output. No construction-inclusive alternative,
   allocation or timing advantage was measured. Do not authorize a correction from
   the presence of this copy alone.
2. **DEFER — working-tree owned duplicate set with empty untracked output.** Empty
   output still triggers set construction, but the largest measured set is eight
   entries/72 logical bytes. Duplicate suppression, sorting and mutable Vec lifetime
   remain real constraints when untracked paths exist. No borrowed-set or early-skip
   implementation is justified without larger realistic exposure and exact error/
   ordering controls. No broader indexing/cache change is proposed.
3. **REJECT unsupported omission — required Git acquisition, admission and snapshots.**
   Comparison sides, ref/path security, binary/UTF8 classification, baseline-size
   short-circuit and public skip semantics have consumers and error boundaries.
   Measurement of repetition is not evidence that these contracts can be removed.

**No bounded design investigation or production correction is recommended now.**
This is a measured DEFER return, not FIX/PROTECTED or a new syntax-census disposition.
No extra populations/timings were added to manufacture a favorable optimization.
Replay reuse remains REJECTED; previous protected repairs and deferred/rejected
boundaries remain unchanged. No new investigation follows automatically. Agent
only inspected user-run receipts and edited documentation; no agent-run runtime,
repository staging, commit or push.

```architecture-graph
node: DOC-165
kind: RESEARCH
status: INVESTIGATING
title: Replay retirement and measured deferred Gitdiff ownership candidates
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md
uses_evidence:
  - DOC-188
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```

```architecture-graph
node: PERF-001-REPLAY
kind: MISSION
status: FROZEN
title: Owner no-repair decision for earlier Candidate replay reuse
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md
uses_evidence:
  - DOC-188
preserves:
  - AUTH.TOKEN_ECONOMICS
note: FROZEN imports explicit owner no-repair disposition only; not a universal replay prohibition or new execution authority.
```
