# AF-PERF-002: binary encoder reverse-map reuse design investigation

Agent: Agent MaxHeadRoom
Status: test-only comparison executed GREEN; internal lookup reuse FIX PROPOSED.
No production replacement authorized or implemented. Vector-borrow repair remains CLOSED / FIX /
PROTECTED; all previous protected optimizations stay closed. SQLite unused text is
the next independent candidate. Dense same-signature equality remains deferred.

## Production boundary and retrieval authority

The same current persisted compression/checkpoint entry points documented in
AF_PERF_002_DEFAULT_OPERATION_DISCOVERY.md call binary_wire::encode. After interning
all instruction tuple strings and file identity, encode writes the ordered strings
and constructs HashMap<&str,u64> by enumerating them. That additional map serves
only encode_operand. Current StringTable already owns HashMap<String,usize> with
the same mapping, but has no direct key-to-index accessor. Its lookup(usize) is
the opposite direction; encode_op returns an allocated full operand-index vector
and has a different panic message, so neither is a drop-in substitute.

Fresh Clean-CTX admission still reports degraded frontend_retired timeout. No
fallback search/read tools are exposed here; bounded known-source host reads
establish this trace. Provider failure does not imply absence. No server launched.

## Proposed smallest retrieval design, not implemented

An eventual crate-private read-only StringTable::index_of(&str)->Option<usize>
could return self.indices.get(s).copied(). Encoder maps the successful usize to
u64 and retains EXACT expect("canonical binary operand missing from string table")
at its current operand boundary. Remove only the additional reverse-map construction.
Do not use intern on lookup: that mutates the dictionary after emission. Do not
substitute lookup(index), encode_op, normalization, hashes/digests or a new index.
The existing owned-string index remains operation-local; no lifetime/cache change.
An accessor is a proposed narrow internal API addition requiring owner approval.

Inductive equivalence: for empty table, map/vector are empty. Existing-key intern
returns its original index without mutation. New-key intern captures strings.len()
as i, appends that exact string and inserts i for its exact String key. Thus every
reachable entry satisfies indices[s]=the unique original vector position of s.
from_instructions and from_json both construct through intern. Clone preserves
both collections. Empty strings are ordinary keys; Unicode uses exact Rust string
equality with no normalization. Duplicate input occurrences reuse the first index.
Fields are private and the proposed accessor cannot mutate the invariant.

The reconstructed map enumerates exactly those same positions, converting i as u64.
Direct reuse applies the same conversion to the same i, including on architectures
where casting behavior matters. Both query by borrowed str; std String supports
Borrow<str> with consistent Hash/Eq. Missing keys return None, then the same expect
panics with the same message. Valid typed input interns all written operands, so
missing-key tests are explicit faulted lookups, not claimed reachable public inputs.
Dictionary emission, opcode order, version and file index never depend on randomized
map iteration. No decoder or binary-policy change is involved.

## Complete cost dimensions and measurement boundaries

Let N be instruction count, A total tuple-string occurrences, U unique interned
strings including file id, Q written operand references, and L corresponding
offered key/payload lengths. Both paths still construct tuples, intern A+1 times,
retain the existing ordered strings/owned reverse index, write all payloads and
perform Q operand queries. Current reconstruction adds U key insertions and one
borrowed-key map with spare capacity, plus destruction. Reuse removes that setup
and changes query container from borrowed-str/u64 map to owned-String/usize map.
Same expected hash complexity does not establish equal CPU/query costs: capacity,
load factor, randomized seeds, layout and collisions may differ. Full construction
and teardown must be compared rather than assuming U removed entries are net gain.

Two tracked design tests use a copied StringTable and a copied encoder writer only
under src/tests. Both compared modes share the same table-building and observation
sites; the sole retrieval branch selects rebuilt map versus proposed index accessor.
Both assert exact output against ACTUAL protected production and the all-opcode
fixture, empty/duplicate/Unicode/long payload cases, and39 complete binary payloads
from the40 frozen public records. These are public-derived payload comparisons,
not registered public integration of a new production implementation.

The table law is checked incrementally against independently enumerated first
positions, duplicate/empty/missing keys, clone/JSON reconstruction and invalid JSON.
Both explicit missing-key modes must panic with the existing exact encoder message.
All existing decoder/error contracts remain production-owned and unchanged.

Twelve cost fixtures vary0/1/8/64/256 methods, unique versus duplicate identities,
and0/64/4096 repetitions of six-byte Unicode text. Record intern calls/offered key
lengths, newly retained string population/length, table/vector capacities, added
reverse entries/offered lengths/capacity, operand lookup counts/offered lengths,
common/extra container header sizes and output bytes. Actual production byte/work
controls must agree with rebuilt-copy entry/query counts. Actual Hash/Eq probe or
string-comparison CPU bytes are NOT observed; offered lengths are not inspected
bytes. Capacities/header sizes are not allocator events, allocator overhead or RSS.

Only the two symmetric test copies are timed:24 alternating samples after3 warmup
rounds, including whole table construction, map/query/write and output/table/map
teardown. Common prepared IR is outside timing. Observation updates are disabled
for BOTH timed copies, with identical shared sites and residual flag checks. Added
setup sites exist only where added setup executes. Actual production has different
diagnostic sites and is deliberately NOT a timed speed reference, avoiding an
instrumentation-asymmetric comparison. No source form/code-layout effect is proved.
The common test-only mode branch also limits extrapolation to an integrated source
substitution. No public/release latency or allocation-saving claim is established.

## User-owned comparison

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-binary-reverse-design.ps1
```

Expected two tracked tests GREEN,12 complete paired cost rows, all-target/all-feature
warning-fatal Clippy, formatting, size, encoding and source receipts. Full samples
remain in logs; console prints short summary only. Eight protected production/oracle
identities are checked unchanged. Only new test files and lib registration are
formatted. No composed gate, mutation closure or production optimization RED is
requested for design investigation. All runtime gates UNRUN; agent ran bounded
source/script/UTF-8 checks only. No staging, committing or pushing.

Return after execution: exact source identities, semantic/key-law controls and
measured construction/query tradeoff. If evidence supports reuse, propose only the
read-only internal accessor and local encoder retrieval substitution with separate
genuine original RED/frozen semantics/static/mutation/focused/composed acceptance.
Otherwise return the measured limitation. Do not expand to alternate containers,
caches or adaptive paths. Current vector clone and its verified elimination stay
protected. Census unchanged13/9763 scoped,9750 unresolved plus96 supplements.

## First comparison attempt: test-copy documentation lint

binary-reverse-design-20261108-154555 stops at warning-fatal Clippy before either
tracked design test runs. Removing decoder-only helpers from the test copy left
their doc comments dangling above op_to_index, triggering empty_line_after_doc_comments.
Removed only those obsolete copied comments. No suppression, algorithm, observation
site or production edit. Rerun the same design command; no runtime GREEN is claimed
from this failed attempt. Agent performed bounded source/UTF-8 review only.

## Executed comparison and construction-inclusive recommendation

Agent MaxHeadRoom inspected binary-reverse-design-20261108-154747: two tracked
tests GREEN4.40s, zero failed/ignored, all12 cost rows and24 retained samples per
mode/case. Warning-fatal all-target/all-feature Clippy50.79s, empty format logs,
size23 active files and strict encoding2233 text files GREEN. All1373 recorded
source hashes match at inspection before this documentation update. Eight protected
identities match; no production optimization, composed or mutation gate executed.
inspection.json preserves complete work/sample rows and checked identities.

Microsecond medians below are rebuilt-map TEST COPY / reused-index TEST COPY,
including common construction, queries, binary writing and full teardown:

| Methods / Unicode repetitions / duplicate identity | Rebuild us | Reuse us |
| --- | ---: | ---: |
| 0 / 0 / no | 9.6 | 7.1 |
| 1 / 0 / no | 21.4 | 16.2 |
| 8 / 0 / no | 82.6 | 63.2 |
| 64 / 0 / no | 485.3 | 400.8 |
| 256 / 0 / no | 2028.8 | 1669.1 |
| 1 / 4096 / no | 430.7 | 340.9 |
| 16 / 64 / no | 246.6 | 188.0 |
| 16 / 4096 / no | 8558.3 | 6024.5 |
| 64 / 4096 / no | 36920.4 | 26136.7 |
| 8 / 0 / yes | 49.0 | 43.9 |
| 256 / 0 / yes | 866.7 | 864.0 |
| 64 / 4096 / yes | 11387.9 | 11400.2 |

Zero methods still has one class declaration, not empty IR. Duplicate fixtures
repeat exact method ids/names/bodies so unique key count stays small. No key-count
or payload claim substitutes for these actual populations.

At256 distinct methods, both modes perform1796 intern calls and1283 operand queries
over774 unique strings/3029 retained string bytes. Reconstruction adds774 entries
and896 capacity slots; reuse adds zero entries/slots. At64 long distinct methods,
both perform452 intern calls/323 operand queries with198 unique strings/1573505
logical retained string bytes; reconstruction additionally offers those1573505 key
bytes across198 entries/capacity224, while reuse adds no map. Both modes retain the
common ordered vector and owned StringTable index and still write/hash/query long
payloads where necessary. Their setup and operand observations agree, and actual
production entry/query counts agree with the reconstruction copy.

The extra map header measures48 bytes on this target, excluding bucket storage,
control bytes, allocator metadata and all common/output containers. Capacity is a
slot count, not allocated bytes. Both common capacities and header observations
are retained in the receipt. No allocator events, peak live bytes or actual Hash/Eq
probe/comparison instructions were measured. Key lengths are offered bytes only.

Distinct long cases show favorable separated descriptive samples:64-method long
reconstruction indices6/12/18 are36.618/36.920/37.458ms, versus reuse25.545/26.137/
27.255ms. This supports the local design comparison, not a release/public benefit
or statistically established causal attribution. At256 duplicate methods, intervals
overlap (855.9..870.9us versus857.7..866.2us). Long duplicates likewise overlap:
11281.7..11632.8us versus11204.9..11661.2us. Their+12.3us median difference is
about0.11%, not evidence of universal nonregression or a consequential demonstrated
regression. The data do not justify a duplicate-specific fallback or threshold.

Both design modes pass complete binary equality against actual protected production
for all-opcode/empty/duplicate/Unicode/long cases and39 frozen public-derived binary
payloads. Key-law, original first-position, cloned/from_json table reconstruction,
missing-key None and exact panic-message controls pass. These are bounded executed
controls plus the source-level induction above; they are not new public integration
or a complete future repair acceptance. Debug observer flag checks and the common
mode dispatch remain in timed test code. Actual production is an untimed control;
its different observation costs are not silently compared against disabled copies.

Exact unchanged production identities:

- src/ir/binary_wire.rs:
  413316666b5bc7fafa455c194ec83fddcaa086fed973c82bc2a338dd7313d4a8
- src/ir/string_table.rs:
  8ff1239840e87c363bf29edf0022a5be4271157ac51cb4aca723452134f9eca0

**Recommendation: FIX PROPOSED for owner approval.** Add only a crate-private
read-only index_of returning Option<usize> from the EXISTING StringTable map.
Replace encoder's reconstructed map retrieval with index_of(s).map(|i|i as u64),
retaining its exact expect message and all output logic. Drop only the redundant
map setup. Keep the protected borrowed vector, existing intern/ordering/duplicate
behavior and all necessary lookup/output costs. Complexity is a small accessor and
local substitution, without another container, cache, threshold or identity policy.

Before implementation, freeze current encoder/table and exact binary/public identity
oracles; author an independent actual-site zero-reconstruction budget and execute
genuine original RED with binary/public semantics GREEN. Do not relabel this passing
design comparison as optimization RED. Preserve explicit common StringTable setup,
operand query and replacement observations; zero old map entries is not zero work.
After approval and genuine RED, require unchanged-budget and exact-byte/public GREEN,
missing-key/first-position/source-currentness coverage, finite accessor/index/full
writer protection, isolated real original-map restoration budget/static RED with
semantics GREEN, exact restored GREEN, focused warning-fatal checks and mandatory
composed acceptance with source identities. Integrated cost observations must be
reported as their own evidence; test-copy timings do not certify the integration.

The closed clone guard currently protects the entire encoder body. A future approved
reverse-map correction must reconcile that finite guard to the newly authorized
retrieval change while independently retaining clone prohibition, binary equivalence
and required work sites. No guard change or closed-scope retesting is made merely
for this design return. SQLite unused pretty_text stays next and independent; dense
same-signature equality remains deferred. No production edits, stage/commit/push.
