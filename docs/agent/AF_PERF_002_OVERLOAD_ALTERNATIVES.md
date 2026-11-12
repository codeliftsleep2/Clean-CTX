# AF-PERF-002: overload retrieval alternative investigation

Agent: Agent MaxHeadRoom
Status: comparison executed GREEN; TRADEOFF REVIEW / no candidate accepted.
No production algorithm change; no remaining FIFO closure gate executed.
Current production FIFO remains unaccepted, retained only as a comparison candidate.
Original RED/oracle/public budget, FIFO semantic GREEN and cost receipts stay frozen.
No remaining FIFO closure, mutation or composed gate is requested.

## Owner boundary and discrimination versus equality

The current design's tradeoff review is not resolved by its wide-group win. Owner
authorizes comparison of signature partitions, sorted exact indices and full-equality
hashing, with all original order contracts retained. No thresholds, cross-call reuse,
adaptive dispatch, persistent cache or public behavior change is introduced.

The comparison separates cheap signature discrimination from declaration/marker/body
payload equality or ordering. It records signature string comparisons, payload string
comparisons, sequence comparisons and offered lengths separately. Hash input bytes
are bytes supplied to Hasher::write, including structural tags/lengths; they are not
allocator traffic, method payload length or CPU-inspected bytes. Standard string
operations may short circuit or use optimized instructions; offered lengths do not
claim actual byte reads. Exact ordering and exact equality use distinct operations.

## Candidate implementations (tracked tests only)

All candidates borrow the existing current group. A test-only complete comparator
adapter copies the frozen original action/control flow and replaces only its two
selection sites with candidate retrieval. It does not call or alter production FIFO
retrieval. Original pairing traces, used flags, action timing and remainder code are
retained. No production caller can select these candidates.

1. Signature-partitioned occurrence arrays: sort current indices by exact visible
   signature then original index. Binary-search the equal-signature range; exact
   matching scans only that range, skips used indices and compares full payload
   equality in original occurrence order. After the complete global exact pass,
   reuse the signature array with a per-range cursor to consume first unused
   occurrences. Unrelated signatures never trigger payload equality. Dense identical
   signatures can still require repeated exact candidate work; this is measured,
   not hidden or claimed solved by partitioning alone.
2. Sorted full-exact indices: sort borrowed current indices by all equality fields,
   with original index as tie-breaker. Binary-search each baseline's complete-key
   range and consume its cursor. Drop exact storage after the global exact phase;
   then build a sorted signature-index array from unmatched current occurrences.
   No per-identity FIFO allocation. Full payload ordering may still be expensive.
3. Collision-checked full-equality hashing: private borrowed method keys hash every
   equality field, including exact optional/sequence semantics. Each key owns only
   its occurrence-index Vec/cursor, not payload. Hash lookup checks full equality;
   a digest is never sufficient. Drop this map before unmatched signature-array
   construction. A deterministic standard test hasher makes fixture comparison
   reproducible; production hash/security/seed policy is not proposed by this test.
   A separate forced-zero-finish collision mode proves correctness when every key
   collides. That adversarial mode is semantic coverage, not a normal cost sample.

No hybrid, fallback threshold or automatically selected winner is implemented.
Signature range order is always original current order, since sorting ties by
original index. Exact-key ranges likewise consume lowest indices among equal keys.
Hash buckets append original indices and retain all duplicates. Every design
finishes the entire exact phase before signature consumption; no early signature
match can steal a later baseline's exact occurrence. Output never iterates a map.

## Reference symmetry and semantic controls

The frozen original oracle is unchanged. A second TEST-ONLY observed-original
adapter keeps its find predicates, flags and output code, replacing direct method/
signature equality with field-level observers using the same short-circuit equality
rules. It is checked against the frozen original before use. Thus the original
does not omit the payload probes charged to the candidates. The existing FIFO is
included with its original diagnostic sites; it is a reference design, not closure.
All implementations keep diagnostic calls while updates are disabled during timing.
Residual per-probe call overhead remains a limitation; no release-speed claim is
made, and comparisons cannot identify CPU costs below that overhead.

Tracked alternative semantics compare complete selected indices/actions/summaries
and all five fidelity outputs across every sequence through length two over nine
method variants, class and orphan paths, plus all fourteen larger fixtures and the
exact-reservation adversarial case. Variants include duplicate multiplicity,
None/Some(empty), bodies, markers, declaration evidence, Unicode and signatures.
Hash collision mode runs the same semantic cases. The original frozen exhaustive
length-three evidence remains independent; new results are not claimed yet.

Forty-five frozen ORIGINAL public rows are tracked as a fixture. Candidate actions,
pair traces, exact formatted response projection, snapshot content and source hash
are checked against those rows at every fidelity. This is frozen public-oracle
consistency for test-only candidates, not live dispatch through a new production
implementation. Actual public candidate reachability/cache admission would require
the separately approved production correction. No such integration is claimed.

## Construction-inclusive comparison and limits

Reuse all fourteen original comparator fixtures, unchanged axes/source generation:
empty/singleton/small, independent width, all-exact/body, duplicates, same signature,
reordered, sparse, long common payload and many singleton groups. Each operation
includes existing grouping, candidate setup/query, action construction and teardown.
Common snapshot input preparation and test-design selection are outside timing.
Three warmup rounds precede 24 retained samples; execution order rotates across
original, FIFO, partition, sorted and hash to avoid fixed original-first ordering.

Enabled work observations are collected separately before timed samples. Original
predicate visits/equalities, signature discrimination, payload comparison, hashing,
selection visits, entry/lookups, retained hash keys/queues, hash capacity and actual
index/cursor capacities are retained. Array designs use the existing queue_capacity
slots to report index-plus-cursor capacity; the log/document identifies the storage,
so those numbers must not be labeled allocated FIFO queues. Counts are accumulated
across sequential groups/phases; they are not simultaneous peak memory.

Layout rows report actual target sizes of indices, signature-array headers, hash
key/value/map headers and lookup state. Hash control bytes, allocator overhead,
allocation events/RSS and exact peak storage remain unmeasured. No allocator win
may be claimed from capacities alone. Full-payload hash construction and repeated
long-prefix ordering remain real candidate costs, not free identity operations.

## User-owned investigation execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-overload-alternatives.ps1
```

Expected three tracked investigation tests: frozen semantic comparison including
forced collisions, 45-row public-oracle consistency, and fourteen paired cost rows
plus one layout row. Wrapper runs warning-fatal all-target/all-feature Clippy,
format/size/encoding and freezes source hashes. All are UNRUN. Agent performed
bounded source/text/PowerShell syntax review only; no test/build/formatter/lint/
runtime process was started. No stage/commit/push.

Return measured candidate costs, semantic results, container footprints and smallest
defensible recommendation. If none resolves the regressions, report that evidence;
do not pick a winner from one favorable axis. If one does, propose the bounded
production replacement and acceptance plan for owner review: preserved original
RED/oracle identity, unchanged budget/public GREEN, finite phase/full-key/static
protection, independent real restored-scan mutation RED/semantic GREEN/restored
GREEN, focused checks and mandatory composed fitness. No production replacement
is authorized by this investigation packet. Prior protected repairs stay closed;
census remains 13/9763 scoped, 9750 unresolved plus 96 supplements.

## Executed comparison: semantics GREEN; no complete cost winner

Agent MaxHeadRoom inspected overload-alternatives-20261107-152608: three tracked
tests pass in 13.60s, zero failed/ignored. Exhaustive bounded pairing/action/all-
fidelity controls, forced-collision semantics and 45 frozen public-oracle rows pass.
Fourteen cost rows and one layout row complete. Warning-fatal all-target/all-feature
Clippy passes 58.51s, both format logs empty, size passes 18 active files and strict
encoding passes 2189 text files. All 1344 recorded source hashes match at inspection.
inspection.json preserves parsed comparison rows. No agent runtime was started.

The table uses microseconds. Original is the field-observed original adapter,
checked against the unchanged frozen oracle. FIFO is the explicitly unaccepted
current design. Measurements from this run are not compared causally with older
cost runs using less detailed original instrumentation.

| N / width / mode / payload repetitions | Original | FIFO | Signature arrays | Sorted exact | Full hash |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 / 1 / exact / 0 | 1.6 | 1.5 | 1.6 | 1.6 | 1.6 |
| 1 / 1 / exact / 0 | 11.5 | 13.8 | 12.5 | 11.3 | 14.3 |
| 8 / 8 / body / 0 | 31.8 | 43.6 | 34.9 | 35.6 | 47.3 |
| 128 / 1 / body / 0 | 672.4 | 901.7 | 777.3 | 830.0 | 1042.2 |
| 128 / 8 / body / 0 | 422.2 | 601.4 | 476.3 | 500.7 | 712.8 |
| 128 / 128 / body / 0 | 1815.9 | 739.4 | 506.0 | 650.2 | 725.5 |
| 128 / 128 / exact / 0 | 475.0 | 462.9 | 360.5 | 462.2 | 545.3 |
| 128 / 128 / duplicates / 0 | 468.6 | 245.0 | 434.1 | 553.1 | 364.9 |
| 128 / 128 / same-signature / 0 | 3578.7 | 869.6 | 3070.2 | 885.3 | 637.1 |
| 128 / 128 / reordered / 0 | 939.4 | 457.9 | 459.6 | 555.6 | 639.6 |
| 128 / 128 / sparse / 0 | 2473.3 | 728.0 | 488.3 | 606.1 | 737.5 |
| 64 / 64 / duplicates / 4096 | 346.6 | 382.4 | 329.2 | 1402.6 | 11173.8 |
| 64 / 64 / same-signature / 4096 | 1554.2 | 1739.4 | 1461.8 | 1831.0 | 20073.6 |
| 512 / 1 / exact / 0 | 2810.1 | 3363.6 | 3171.4 | 3359.5 | 4390.0 |

These are rotating-order medians from 24 retained-in-memory debug samples after
warmup. Logs preserve medians/work, not raw timing distributions. Residual probe
calls remain while updates are disabled. No statistical confidence, release speed,
allocator saving or public latency improvement follows. Still, construction-gate
regressions cannot be dismissed simply because semantic tests pass.

### Retrieval discrimination versus payload equality

At N128/width128/all-body with distinct signatures, the observed original makes
16384 exact and 8256 signature candidate visits, with 16512 signature string
comparisons and 128 payload string comparisons. Signature arrays make 5139
signature comparisons, the same 128 payload comparisons and 256 selection visits.
The lookup filters by signature before full equality, giving the best result in
this fixture. It avoids ordering bodies merely to discriminate unrelated signatures.

For N64/all-same-signature/long bodies, original and signature arrays both perform
4096 payload string comparisons, offering approximately 201.4MB of total strings.
Signature partitioning does not eliminate repeated searches WITHIN that identical
signature family. Its improvement here is modest (about 6%); do not call the dense
exact-search problem solved. Equality can reject unequal string lengths before
inspecting payload, whereas lexicographic ordering may traverse a common prefix.
Actual inspected byte counts are not measured.

Sorted exact performs 1270 payload comparisons offering about 62.5MB in that long
fixture, but is slower than original. On long identical duplicates it offers about
47.2MB versus original 3.15MB and is roughly four times slower. Fewer selection
visits do not imply cheaper full-key construction/query.

Full hashing offers only nine payload equality comparisons in the long changed
fixture, but feeds 5808308 bytes (including structural encoding) to the hasher and
takes about 20.1ms, roughly 13 times original. Long identical duplicates feed
3150080 hash bytes and take about 11.2ms. Full hashing successfully discriminates
identity and passes forced-collision correctness, but its complete construction
cost is not acceptable evidence of an optimization here.

FIFO's legacy string counters do not separate discrimination/payload categories;
zeros in those NEW category fields mean unavailable attribution for that design,
not absence of string work. Its existing comparison/length totals remain visible.

### Storage and complexity

Executed target sizes: signature-index header 48 bytes (two Vec headers), borrowed
hash key 8, hash value 32 (Vec plus cursor), hash-map header 32, lookup state 120,
index 8. At 512 singleton groups signature arrays accumulate 2560 index/cursor
capacity slots (20480 bytes) over sequential groups, versus sorted exact 1024
slots (8192 bytes); these are not simultaneous peak storage. Signature construction
currently collects a filtered iterator, which retains Vec spare capacity on tiny
groups. Exact-capacity array construction is a concrete investigation refinement,
not a measured benefit or production correction. Hash per-key Vecs and table/control
storage remain additional to their reported capacities. No allocator counts exist.

Signature arrays are simplest and best separate cheap discrimination from necessary
equality, but retain dense same-signature scans and have about 16%/13% regressions
at N128 width one/eight and 13% at 512 singleton groups in this run. Sorted exact
adds binary ranges/cursors and deep ordering, regressing long payloads. Hashing
adds field-hash maintenance, collision handling and per-key vectors, with decisive
long-payload costs. None demonstrates the required benefit without meaningful
common-group or long-payload regressions. No hybrid has been measured or accepted.

## Returned owner decision point

Disposition remains TRADEOFF REVIEW, not FIX / PROTECTED. No production replacement,
threshold, container switch or fallback policy is authorized or implemented. The
unaccepted FIFO is still in the working tree; no stage/commit/push performed.

Recommendation: use signature-partitioned arrays as the basis for a separately
reviewed refinement, not authorize it for publication yet. A concrete next design
comparison could remove avoidable tiny-array spare capacity and evaluate an explicit
structural one-current-occurrence case without building retrieval storage, retaining
the full global exact phase and original remainder policy. That is an additional
algorithm path and requires an explicit reviewed scope; it is not silently added
as a threshold. Dense same-signature discrimination needs its own low-cost proposal
if eliminating that repeated equality remains required. Payload-length/shape keys
can only reject impossible matches; they cannot become equality authority.

The original RED, frozen semantics, FIFO GREEN and both cost packets remain intact.
If a revised candidate resolves the measured regressions, return its exact bounded
production proposal and original-budget/semantic/static/mutation/composed plan for
owner authorization. No further execution is requested merely to reconfirm these
results. All previously protected scopes remain closed.

```architecture-graph
node: DOC-178
kind: RESEARCH
status: INVESTIGATING
title: Historical overload alternatives and unaccepted tradeoff comparison
track: REPOSITORY
artifact: docs/agent/AF_PERF_002_OVERLOAD_ALTERNATIVES.md
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
