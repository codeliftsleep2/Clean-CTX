# AF-PERF-002 C2: alias matching and enclosing type-opcode work

Agent: Agent MaxHeadRoom
Status: characterization authored UNRUN; no alias algorithm/representation repair.
C1, A3 and NgRx collector remain protected. A2 decision is not reopened.

## Actual production caller trace

mcp/semantic_compilation/compile.rs compiles the requested source then calls
ir/type_aliases.rs::apply_type_aliases_to_ir using that request's exact config.
The pass visits IR, calls the existing matcher separately for FieldType, Return
and Param strings, then appends sorted reversible TypeAlias records. It precedes
artifact construction/publication. Matching setup therefore repeats for every
eligible operand, independent of full source size or emitted alias count.

compression/pipeline.rs also calls the matcher during body assembly. compress_text
is called by four language-layer implementations with aliases=None, so those
specific calls do not establish configured-alias work. compress_source is documented
as retired from workspace compression and retained for test stats; do not rank it
as a default public production hotspot. Other exported compress_file_* consumers
and compaction paths remain incompletely traced. Main MCP IR success is the measured
public path here, not a claim of exhaustive alias caller coverage.

## Test-only actual-boundary observations

compression/type_aliases.rs: matcher calls/input bytes; configured pairs inspected;
eligible pairs; actual sort-key evaluations; scan positions; candidate-pair probes;
full key lengths considered after length gate; successful substitutions; output body+
footer bytes; sum of temporary pair Vec capacities. Sort/priority/matching/UTF-8
advancement and footer logic remain unchanged. Key lengths considered are not actual
byte comparisons: memcmp may stop early. Capacity sums are neither concurrent peak
memory nor allocator allocation counts.

ir/type_aliases.rs: enclosing pass calls, actual IR instruction visits and operand
calls. Observations are cfg(test), synchronous/thread-local and never authorize
semantic reuse. No alias preprocessing, trie, index, cache, limits or generation change.
Timing includes counter updates; no uninstrumented speedup/regression claim.

## Controlled dimensions and exact oracles

Three tracked tests:

1. Production matcher: independent text/alias growth1/2/4/8/16/64/256;
   match density0/1/4/8/16; key bytes4/8/16/64/256; longest-key precedence;
   Unicode, prefix/suffix/$/underscore/bracket boundaries; empty config.
   Exact body assertions and used-footer observations preserve current behavior,
   including existing byte-oriented boundary semantics, not a new Unicode policy.
2. Whole IR transform: operand growth1/2/4/8/16/64/256 with16 configured aliases.
   Require exact ordered rewritten Return records plus one reversible TypeAlias.
   Setup counts characterize current repeated construction; they are not future
   optimization budgets or evidence of meaningful latency without measurement.
3. Actual public compress_code_context: alias count0/1/2/4/8/16/64/256 at
   fixed parameters, then parameter growth1/2/4/8/16/64/256 at fixed aliases. Assert
   captured operand growth and exact named-IR TypeAlias declaration. Report full
   operation time alongside actual operand count/size, setup and matching totals.
   Raw/candidate economics are unchanged; this test does not replace C1 oracles.

The existing20 compression-alias and11 IR-alias semantic tests remain unchanged,
protecting footer reversibility, collisions, deterministic output and supported
IR transformations independently of the new counters.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-c2-alias-work.ps1
```

Expected34 tracked executions (three characterization plus31 existing controls),
then all-target/all-feature warning-fatal lint, formatting, active sizes and strict
encoding. Exact source hashes/logs retained. No agent formatter/Cargo/test/build/
runtime started. Only bounded script parsing and UTF-8/no-BOM/line-count checks run.

## Decision and remaining coverage

Return setup versus matching work and enclosing dimensions before recommending
FIX/MONITOR/ACCEPT. A nested T*A scan can be necessary to preserve priority and
boundaries; a new matching structure is not justified by notation alone. Repeated
per-operand sorting may be a significant setup opportunity, but sharing setup would
introduce a representation/API tradeoff requiring independent design approval.
No correction is proposed as already justified or authorized by this packet.

Public branch/error/currentness dimensions, large Unicode key population, lower
capture/compaction exports, rendering traversal, proxy/transport and remaining
subsystems stay open. Original9763 syntax records remain preserved with13 scoped
outcomes/9750 pending plus96 callback supplements. New measured operations are not
additional syntax dispositions or whole-system architectural fitness certification.


## First execution: fixture failure, corrected public growth axis

Operator c2-alias-work-20261104-155212 ran the three new characterization tests:
two passed and the public test failed, zero ignored. The wrapper stopped before
existing controls and lint gates. This is a fixture failure, not optimization RED.
Captured TypeScript fields emitted DEF_F records without FieldType operands; only
the method parameter and return reached alias substitution. Field population was
therefore not a valid alias-operand growth dimension in this fixture.

The corrected test varies captured method parameters instead, requires their exact
ordered SIG names/types and the exact RET type, and retains the TypeAlias oracle.
No production behavior or counter changed. Original execution logs remain frozen.
The corrected packet is UNRUN; rerun the same command above. Expected 34 tracked
executions and subsequent gates remain pending until actual successful execution.


## Verified execution and disposition

Operator c2-alias-work-20261104-160845 completes GREEN: three new characterization
tests (16.12s), twenty compression-alias controls and eleven IR-alias controls;
34 passes, zero failed/ignored. Warning-fatal all-target/all-feature Clippy completes
in24.46s; format-check log is empty; size gate passes; strict encoding passes2087
text files. All 1299 receipt source hashes match at inspection. Original
logs and receipt remain under target/af-perf-002/c2-alias-work-20261104-160845.
Earlier fixture and lint failures remain separate historical evidence, not RED
against an optimization budget. No agent runtime was started.

| Measured boundary/dimension | Actual work |
| --- | --- |
| Negative text8..2048 bytes,16 aliases |128..32768 probes, exactly text positions times aliases |
| Negative text128 bytes, aliases1..256 |128..32768 probes |
| Match density0..16,16 aliases |2048..272 probes; matches skip consumed text |
| IR operands1..256,16 aliases |16..4096 setup-entry visits;30..7680 sort-key evaluations |
| Public four parameters, aliases1..256 |5 operands;5..1280 setup visits;5 matching probes |
| Public parameters1..256,16 aliases |2..257 operands;32..4112 setup visits;60..7710 sort-key evaluations |

The public path verifies exact ordered SIG parameter names/types, RET type and
reversible TYPE declaration. Matcher controls cover longest-key priority, matching
and negative cases, key lengths4..256, Unicode and token boundaries. Existing31
semantic controls pass. The public no-alias case makes zero matcher/setup calls.
Field-type capture remains a fixture limitation; parameter growth supplies the
legitimate captured operand dimension instead.

Public parameter growth times are895902us at one parameter and2314636us at256;
these include compilation, persistence, rendering and counting, so they do not
attribute latency to alias setup. Direct IR-transform samples are122us at one
operand and2434us at256, including observation overhead. Capacity sums count
transient reference slots across calls, not peak memory or allocated bytes.

Disposition: C2 STRUCTURAL RISK / MONITOR. Growing nonmatch probes and repeated
per-operand setup are measured. Eliminating setup would require separating prepared
pairs from the current matcher interface and preserving stable longest-key order,
eligibility and footer semantics. No construction-inclusive alternative has been
measured; no production correction is authorized or justified by these counts
alone. Keep repeated setup as a concrete design-review candidate, without adding
an index, trie, cache or new abstraction. Lower pipeline callers and full public
failure/currentness coverage remain open. C1 and earlier repairs stay protected.

Syntax accounting remains9763 preserved/13 scoped/9750 pending plus96 callback
supplements. These measurements advance production-operation coverage only.
