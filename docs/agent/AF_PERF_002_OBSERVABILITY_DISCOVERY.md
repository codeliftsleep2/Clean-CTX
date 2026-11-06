# AF-PERF-002: observability entry points and public statistics format work

Agent: Agent MaxHeadRoom
Status: format-gated session-wide stats text construction FIX / PROTECTED.
Original RED, corrected semantic/static GREEN, isolated mutation RED/restored GREEN
and final composed gate verified. Discovery remains incomplete.
Historical authored-status entries retained below.
Replay packet remains CLOSED / NO REPAIR; protected scopes are unchanged.

## Production entry points and distinct work

observability/tracing.rs installs a filtered stderr writer at server startup; JSON
and text formatting depend on configuration. No stdout transport or logging policy
change is authorized. startup.rs checks an opt-in environment path before assembling
JSON/opening/appending/flushing each event. event_detail converts its supplied detail
before that gate, but traced callers are mostly lifecycle/error boundaries, not a
proven high-volume growing hot path. Startup IO cadence and disabled-path cost remain
separate observations, not reasons for a new buffer or logging abstraction.

MetricsRegistry is instantiated in McpState. server.rs optionally spawns the configured
periodic snapshot/OTLP-like JSON exporter; category normalization bounds record_error
keys to seven enum variants through that API. Public errors DashMap and public Histogram
bounds permit other callers, so no universal cardinality invariant follows. Histogram
record scans bounds (default10), snapshot clones buckets, and export serializes a
complete snapshot. Current first-party metrics_registry references are construction
and exporter; do not mistake comments about context_stats integration for its actual
consumer path. Full producer instrumentation/export usefulness remains unverified.

Actual context_stats uses mcp/tool_handlers/stats/mod.rs, SessionStats, optional SQLite
rebuild_stats, optional proxy fetch and cache metrics. It clones session and merged
snapshots before rendering. DB/proxy/cache work and owner locks are independent costs.
The response must remain observational: no stats read may flush queues or publish.
Existing tracked context_stats read-only lifecycle tests govern those behaviors.

## Concrete format-dependent candidate

The session-wide branch unconditionally calls render_dashboard_text(&merged), decorates
its String with tokenizer/persistence/cache text, and only then checks format==json.
JSON returns separately rendered JSON; the earlier text is unused. render_dashboard_text
summarizes file statistics, collects all file references, sorts by savings and formats
every file row. Thus request format and retained file count are independent dimensions.
This is a source-grounded unused-result lead, not executed waste evidence or repair.

File-specific JSON similarly builds a text summary before selecting JSON, but that
is a different bounded path, not the growing full-dashboard work measured first.
JSON also emits root/session/combined dashboards separately; public duplicated payload
fields cannot simply be removed as an optimization. No output contract change proposed.

SessionStats is an oversized legacy file and is unchanged. Observation at the existing
handler call records initial merged-text render invocations, input file counts and
returned base text bytes (before decorations). This exact boundary measurement does
not need moving rendering code or weakening the semantic boundary. It excludes sort
comparison counts, other renderer calls, decorated bytes and allocator/peak memory.
No production algorithm or public API changes; counters exist only under cfg(test).

## Public packet and exact controls

Actual context_stats dispatch varies retained file count and invocation count over
1/2/4/8/16/64/256 plus16x16, comparing JSON/text forms. Fixture session events are set
up outside observations; persistence is disabled, proxy port0, CBM disabled. There is
no upstream traffic. JSON oracles check root/session/combined exact file identities,
counts, token totals and selected stable fields; text checks both account scopes and
file rows. Session file/domain data must remain exactly unchanged. File-specific JSON
serves as a separate control. Session duration is time-dependent; no full response-
identity claim is made by this characterization. Existing lifecycle controls supply
separate DB/queue/read-only evidence, not work attribution for every configuration.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-stats-work.ps1
```

Expected six tracked executions: one new public characterization plus five existing
read-only stats lifecycle tests; then main-package all-feature/all-target warning-
fatal lint, format, active sizes and strict encoding. Preserve logs/source hashes.
All UNRUN; agent performed only bounded source, UTF-8/size and wrapper syntax review.
No runtime, binary, exporter or server was started.

## Disposition and breadth

Public unused text candidate DESIGN REVIEW pending actual counts and exact output/
side-effect proof. No lazy helper, caching, snapshot sharing, synchronization,
registry, response or metrics-export change is authorized. Any future conditional
must keep required text rendering outside the cache-metrics lock as currently ordered,
rather than moving growing work under that lock while rearranging format branches.

Other observability/startup diagnostics, full disabled trace cost, error cardinality,
periodic export, SQLite/proxy stats acquisition and concurrent observations remain open.
No whole-file/subsystem safety claim. Syntax accounting stays9763 preserved/13 scoped/
9750 pending plus96 callback supplements. Production-path tracing is not census
classification progress; final independent architectural coverage review is outstanding.


## Executed public stats characterization and narrow design return

stats-work-20261105-201544 passes six tracked executions: new public format/file
characterization1.41s, five existing read-only lifecycle controls23.77s. Zero failed/
ignored. Main all-feature/all-target warning-fatal Clippy30.52s, empty format-check,
size18 active files and strict encoding2121 text files pass. All 1309 source
hashes match at inspection before documentation updates. inspection.json retains28
format rows and14 file controls. Agent inspected evidence only; no runtime started.

### Session-wide JSON, one invocation

| Files F | Unused initial text renders | Input files at render boundary | Base text bytes constructed |
| ---: | ---: | ---: | ---: |
|1|1|1|1475|
|2|1|2|1551|
|4|1|4|1701|
|8|1|8|2001|
|16|1|16|2605|
|64|1|64|6207|
|256|1|256|20611|

For Q1/2/4/8/16/64/256 with F1, JSON makes Q unused initial text renders, passes Q
file entries to them and constructs1475*Q base bytes. Q256 totals377600 unused base
text bytes. Joint F16/Q16 gives16 renders,256 input-file entries and41680 base bytes.
Text requests take the same initial boundary but consume its result. All14 file-
specific JSON controls make zero initial full-dashboard renders; other file-path
setup/formatting costs are not zero or certified by that counter.

The rendered input counts are not instrumented sort comparisons; source shows the
complete file list is collected/sorted/formatted. Base byte counts exclude tokenizer/
persistence/cache decorations and other dashboard renders, so they understate total
unused text allocation. They are not peak memory or allocator events. JSON outputs
still perform necessary serialization and cloned snapshot work.

Public JSON checks root/session/combined file identity sets, token totals, selected
stable fields and disabled persistence; text checks account scopes/file rows. File
and domain session accumulators remain byte-equivalent after queries. Existing tests
also pass queued-work/read-only/persisted accounting invariants. Full response equality
including dynamic durations and concurrent data snapshots is not established here.

Descriptive Q1/F256 JSON83147us versus text6159us; Q256/F1 JSON253018us versus
text81479us. Formats have different response workloads and cold initialization;
these observations cannot attribute differences to unused text or imply a measured
post-repair speedup. The demonstrated waste is the unused growing rendering itself.

### Smallest correction candidate, owner disposition required

In only the session-wide stats branch, construct the existing text dashboard and
text-only tokenizer/persistence decorations only when format != json. Keep text
construction in its current position before cache_metrics_lock for text requests.
After acquiring that same lock, build cache text only for text requests; JSON still
uses the existing render_cache_json under the same snapshot guard. Leave all JSON
rendering/accounting fields, session/DB/proxy acquisition and merge behavior untouched.

One explicit conditional around existing local construction is sufficient; an empty
String for JSON has no heap payload. No helper, cache, registry, snapshot protocol,
public API, normalization or new synchronization is needed. File-specific formatting
is excluded. Do not move whole growing text rendering under the cache lock while
rearranging branches. Do not skip DB/proxy/cache observation because output is JSON.

Source proof: render_dashboard_text reads the already-owned merged snapshot and
formats local temporaries; it does not mutate session/index/persistence authority.
Token/config summaries likewise read immutable config or owned db_stats. The cache
text helper's behavior must be reviewed explicitly before implementation; preserve
its lock/observation ordering and all JSON cache fields. Exact text/JSON and read-only
controls remain required; source proof alone is not acceptance.

Four criteria: actual unused work grows with F and Q; its removal avoids a complete
file-list text render with no new construction; format-gated pure rendering preserves
output subject to exact controls; local condition adds little maintenance. Classify
MEASURED UNUSED WORK / FIX PROPOSED. No production repair has been implemented.

If approved: author zero-unneeded-text public budget and demonstrate genuine RED
against current source, preserve characterization separately, apply only the approved
format guard, require unchanged GREEN. Compare exact normalized response fields and
text, cache/persisted/read-only behavior, populated and empty cases, text demanding
work/timing, finite actual-sink protection and isolated production eager-render
mutation RED/exact restored GREEN. Do not retroactively call this packet RED.

Broader statistics acquisition/snapshot clones, generic metric exporter, startup and
other observability work remain independent. Original syntax accounting unchanged;
no discovery-complete claim follows from42 observed format/file rows.


## Owner-approved format guard: original budget authored, UNRUN

Agent: Agent MaxHeadRoom. Owner authorizes only skipping session-wide initial
text construction for JSON; no production guard has been applied yet.

Source proof: render_dashboard_text reads the supplied immutable SessionStats,
computes a local summary and formats a local String. Its elapsed-duration read
is presentation data, not validation or currentness state. render_cache_text
likewise reads CacheMetrics and returns a local String/None; it performs no IO,
mutation or registration. JSON uses separate dashboard/cache serializers and
never consumes either text result. Persistence reconstruction, proxy observation,
merged cache metrics and all locks remain required. The future guard must retain
initial text rendering before the final cache lock for text requests, retain that
lock acquisition for JSON, and retain both existing explicit release sites.

New tracked stats_format_budget_af_perf_002.rs exercises actual context_stats:
empty/single/multi-file populations through 256, independent request growth through
256, and 16x16 joint growth. JSON requires zero initial merged text renders/input
files/bytes; text requires exactly one initial merged render per invocation.
Additional session/persisted text renderings are distinct required accounting
scopes, not claimed redundant. File-specific rendering is outside repair scope.
All rows print before aggregate assertion, so intended RED preserves full counts.

Complete response rows are retained for subsequent before/after comparison.
Only elapsed duration is normalized (validated numeric JSON duration_secs and
text duration line); JSON file arrays are sorted by exact path to compare their
existing unordered HashMap enumeration. No response field is removed. Fixture
savings are distinct so text ordering is deterministic. These are pending frozen
oracles, not an already executed equivalence proof. Authoritative file snapshots
must remain unchanged. Existing five read-only tests retain separate observation/
failure authority; final static/lock placement review and mutation proof remain
pending. No claim of byte-identical volatile clock observations is made.

Canonical boundary: public dispatch -> context_stats -> immutable session/merged
snapshots -> selected serialization -> captured JSON-RPC response. No new persistent
owner, cache, lifecycle state or authority is proposed. Scope is representation
construction only; standalone renderers and other callers stay unchanged.

Run the Before packet below. It must fail only the intended unused-render budget;
compile/fixture failures are not genuine RED. It retains source/test bytes and
hashes. Production is not edited until that evidence is inspected.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-budget.ps1 -Phase Before
```

Tests, formatter and other runtime gates are UNRUN by the agent. Bounded text/
script review only. After verified RED: smallest guard, unchanged budget GREEN,
exact normalized response comparison, focused read-only regressions, static known
sink/lock-placement protection, isolated unconditional-render mutation RED and
restored GREEN, then final source-bound verification. Earlier protected repairs
and syntax accounting remain unchanged.


## Genuine original RED verified; narrow guard authored, After UNRUN

Agent MaxHeadRoom inspected stats-budget-before-20261105-231919: one intended
budget failure, zero ignored, 1.64s; all 1310 recorded source hashes matched before
editing. Thirty work rows and thirty full normalized response rows are retained.
JSON constructed one unused initial dashboard per request, including the empty
case; F256/Q1 produced 20611 initial text bytes and F1/Q256 produced 377600.
Text controls passed their one-initial-render invariant before the final aggregate
JSON budget failure. This is correctness/work RED, not a compilation failure.

Production correction wraps only initial merged text rendering and its tokenizer/
persistence decorations in format != json, and separately guards cache-text
formatting after the existing cache lock. The same lock acquisition and explicit
release sites remain. JSON serialization, file-specific branches and final text
assembly are unchanged. No new helper, state owner or authority exists; the empty
String has no allocated payload. Original strings are preserved byte-for-byte.

The unchanged tracked budget must now pass. After driver also requires all thirty
complete normalized responses to match the frozen original rows, including text
output. Duration and previously unordered JSON arrays remain the only declared
normalizations. This does not certify persistence/proxy/failure paths alone; the
existing read-only suite and static/mutation/focused gates remain pending.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-budget.ps1 -Phase After
```

No After test/build/lint/formatter execution was started by the agent. Only bounded
UTF-8, line-count, diff and PowerShell parsing checks were performed. Status remains
implemented / acceptance pending, not FIX / PROTECTED.


## Repaired public budget GREEN; final protection authored UNRUN

stats-budget-after-20261105-232638 passes the unchanged budget (one test, zero
failed/ignored, 1.81s). All 1310 receipt hashes matched before further acceptance
asset authoring. All thirty complete normalized JSON-RPC response rows match
frozen Before rows exactly. Six empty/single/four-file JSON/text rows are now
retained as tracked immutable fixtures, checked by the public budget test.
Normalization is limited to elapsed duration and unordered JSON file arrays.

| Files / requests | JSON initial renders before / after | Initial text bytes before / after | Public JSON us before / after |
| --- | --- | --- | --- |
| 0 / 1 | 1 / 0 | 1396 / 0 | 14347 / 5388 |
| 256 / 1 | 1 / 0 | 20611 / 0 | 511242 / 491547 |
| 1 / 256 | 256 / 0 | 377600 / 0 | 306312 / 463496 |
| 16 / 16 | 16 / 0 | 41680 / 0 | 124669 / 119221 |

These are single instrumented public-operation samples including serialization,
response capture/output and observation overhead; no overall latency speedup is
established. All text controls retain their initial render count and bytes; no
allocator-byte estimate is inferred. JSON still performs its required serializers.

Finite AST protection classifies the initial merged dashboard and cache-text
sinks, requires format gating, checks initial rendering before the existing final
cache lock and cache formatting after it, and retains the two explicit releases.
No blanket rendering prohibition is introduced. Static fixture sensitivity rejects
an unconditional initial-render guard. Runtime isolated mutation changes that real
production guard to unconditional execution, requires budget/static RED while
characterization remains GREEN, restores exact bytes and requires three GREEN.
The active workspace is not mutated by that driver. Existing five read-only
controls exercise persistence separation, pending saves/clears, proxy snapshot
isolation and semantic/pending-edit ownership; their coverage is not relabeled an
exhaustive proof of every failure or concurrent schedule.

The cache guard uses an equivalent short-circuit let-chain to avoid a nested-if
lint; all new protected source needs fresh lint/format/encoding authority. No runtime
was started by the agent. Authored scripts parse and edited text is strict UTF-8
without BOM, under active code size limits. Existing mandatory fitness composition
now includes the two new protected stats tests (expected total at least 61).

Run sequentially:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-mutation.ps1
```

Focused expects eight tracked passes plus warning-fatal all-feature/all-target
Clippy, format, size and encoding. Mutation expects clean three-test baseline,
two intended one-test failures, one preservation pass and exact restored three-test
GREEN. Both remain UNRUN. Final composed gate follows successful inspection;
FIX / PROTECTED is not yet claimed. Syntax accounting and other dispositions
remain unchanged.


## Encoding gate rejected focused packet; exact Git recovery, rerun required

stats-focused-20261105-235537 executed eight tracked passes (three stats 1.98s,
five read-only 18.23s), warning-fatal Clippy 53.16s and format/size gates, then
FAILED encoding. It is not a completed GREEN receipt. The four reported signatures
were corrupted em dashes in handler comments; byte inspection against HEAD also
found damaged existing arrow/box-drawing literals that the signature gate did not
report. An earlier Python read_text without explicit UTF-8 decoded this source
through the Windows default charset before a UTF-8 write. This agent-caused boundary
error is corrected using explicit UTF-8 reads/writes and exact lines from Git.

Six source lines (two comments and four existing presentation literal lines) are
restored exactly from HEAD:src/mcp/tool_handlers/stats/mod.rs, preserving guards
and test registrations. Every non-ASCII production line now matches a known-good
Git line, ignoring indentation only. No redaction/normalization or new spelling
was substituted. Recovery provenance is retained in the failed run's
encoding-recovery.json. Original receipts and raw output remain frozen.

Earlier full response equality compared two snapshots with the same damaged
presentation literals. It remains evidence of guard behavior on that snapshot,
but cannot certify preservation of intended typography. The thirty original
response rows are now retained as tracked expectations with only the precisely
identified tokenizer/persistence literals recovered from the Git source. The
public test verifies all thirty corrected complete responses, not only six small
cases. JSON response contents require no encoding recovery. Existing declared
elapsed-duration/unordered-JSON-array normalization remains unchanged.

Fresh focused and isolated mutation runs are required after this correction.
The isolated unconditional-render mutation must satisfy the same recovered text
oracles as the guarded source, independently proving the guard preserves text.
No completed stats mutation receipt was present at inspection; no mutation GREEN
is claimed. No test/lint/runtime gate was started by the agent. Bounded byte/line
and PowerShell parsing checks passed; repository encoding gate remains user-owned.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-focused.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-stats-format-mutation.ps1
```

Run sequentially; the focused command must complete successfully before mutation.
FIX / PROTECTED remains pending, and the failed packet is retained as failure
evidence rather than relabeled GREEN.


## Corrected focused and mutation receipts verified; composed gate pending

Agent MaxHeadRoom inspected stats-focused-20261106-000051: three format budget/
static/characterization tests pass 2.64s and five existing read-only tests pass
19.64s, zero failed/ignored. Warning-fatal all-target/all-feature Clippy passes
25.35s, format logs are empty, size checks pass 25 active files, strict encoding
passes 2127 text files. All 1312 receipt hashes match at inspection. Thirty tracked
complete response expectations pass with the Git-restored presentation literals.
This supersedes the encoding-failed packet for current focused acceptance.

stats-format-mutation-20261106-000243: clean three-test baseline GREEN 2.91s;
real unconditional initial-render production mutation gives exactly the intended
budget RED (one failure, 2.49s) and static RED (one failure, 0.02s), while existing
characterization passes 0.64s. Exact restoration gives three passes 2.62s, zero
failed/ignored. All 2134 recorded inventory hashes match at inspection. Active,
frozen-before and restored isolated production bytes are identical. No production/
test edits follow these receipts; this return edits documentation only.

JSON observes zero initial full text renders/input-files/bytes on all axes. Text
retains one initial render per invocation. F256/Q1 JSON remains zero versus original
one render/256 input files/20611 bytes. Q256/F1 remains zero versus 256 renders/
377600 bytes. Measured current public JSON times are 559935us and 313916us
respectively; original samples 511242us and 306312us show why no full-operation
speedup is claimed. These samples include response logging/capture/serialization
and instrumentation, not isolated rendering CPU. The correction eliminates proven
unused construction with no new maintained state or semantic policy.

The exact production difference from the known-good Git implementation is the
format guard around the initial text construction/decorations and the cache-text
format guard, plus test-only observations and registrations. All existing Unicode
literals are restored exactly, JSON response logic and lock sites stay unchanged.
Per-file paths remain excluded. Complete whole-subsystem/error/concurrent-schedule
coverage is not inferred from the finite protected scope.

Next existing mandatory composed gate (user-owned, UNRUN in this return):

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-semantic-work-fitness.ps1
```

Expected at least 61 tracked passes, including new stats budget/static protections.
Focused and mutation acceptance is verified; final FIX / PROTECTED disposition
awaits composed GREEN/source identity. No agent runtime started, no additional
historical campaign or live Claude gate requested. Syntax census remains 9763
preserved / 13 scoped outcomes / 9750 pending plus 96 callback supplements.


## Final composed GREEN: FIX / PROTECTED

Agent MaxHeadRoom verified af-perf-001-20261106-001356: **61 tracked passes across
15 groups, zero failed/ignored**. All **224 recorded hashes match** at inspection;
inspection.json retains group counts/times and exact production SHA-256:

`cf56788e4689da49cbbe9ed18b1bc75578f8e7fb188d41255df71f2a435592a9`

Production authority: src/mcp/tool_handlers/stats/mod.rs. New permanent tracked
protections: stats_format_budget_af_perf_002.rs, stats_format_static_af_perf_002.rs
and fixtures/stats_format_responses.json, registered and selected by the existing
mandatory semantic-work fitness gate. Corrected focused eight-test/lint/encoding
receipt and isolated real mutation receipt remain separately authoritative for
those gates. No production/test changes follow final composed verification; only
this completion documentation is updated.

Final disposition: **FIX / PROTECTED**, limited to skipping the initial complete
text representation and cache text for session-wide JSON requests. Initial text
work drops to zero while all thirty exact normalized response expectations pass;
text retains its required rendering, bytes, ordering and existing lock placement.
Zero new maintained state/cache/authority or public schema change. The investigated
encoding incident and failed packet remain documented, not erased. Original budget
RED remains genuine for the work invariant, and corrected isolated unconditional
rendering independently reproduces budget/static RED with restored GREEN.

This is not whole-workspace/binary/field certification, universal latency benefit
or whole-observability acceptance. Persistence/proxy/read-only controls are finite;
transport/concurrency/failure schedules, other rendering consumers, A1/A4 and broad
source inventory remain independent gaps. Original syntax accounting is unchanged.
No additional runtime command, binary rebuild or live Claude run is needed to
close this bounded repair. No commit or push performed.
