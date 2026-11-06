# AF-PERF-002: Spring default compilation and inner annotation work

Agent: Agent MaxHeadRoom
Status: first production Spring characterization executed GREEN; MONITOR.
No production optimization. Historical authored-status sections retained below.
Protected earlier scopes remain unchanged; source-wide discovery remains incomplete.

## Actual producer and consumer path

IRCompiler::compile_focused constructs PassContext and runs default_production.
MetaLayerPass builds ordered class capture texts and paired entries, constructs
MetaLayerContext, then evaluates applicable registry layers. Spring's override
performs one combined marker/semantic evaluation. Existing tracked tests already
protect one Spring applicability detection and one combined evaluation per compile;
this investigation does not reopen or reimplement those repairs.

Spring applicability calls is_spring_file. With no @ character the detector skips
its own AST parse; otherwise it creates a Java parser and parses source for annotation
queries, then may fall back to text detection. This is separate from core Java
capture parsing. The pipeline already owns captures; whether reusing that evidence
is correct across every fallback is a separate design question, not an optimization
approved by observing another parse. Detector comments and fallback behavior differ;
comment/string/invalid-source cases need independent semantic characterization.

Within evaluation, marker extraction loops class captures and invokes annotation
helpers. Semantic extraction loops the same capture list separately. Both may call
collect_method_annotations on class body text; configuration semantic projection can
request additional method extraction for Bean facts. Field collectors, head parsing,
brace scans, path parsing and string allocation have separate costs. Do not describe
all duplicate traversal as unnecessary without a complete shared-result proof.

MetaLayerPass attaches caller-supplied canonical file provenance to semantic endpoints;
compiler returns semantic edges. Real MCP provide then publishes index facts consumed
by workspace_query. The packet includes the existing public Autowired publication
control separately; new work timings stop at compile_focused and do not measure
public transport, source acquisition, index replacement or persistence.

## Independent dimensions and exact oracles

New production compilation characterization varies class count and annotated method
population independently over1/2/4/8/16/64/256 plus16x16. Low/High each assert exact
ordered Spring EndpointMapsTo identities, layer/domain, relation and supplied file
provenance; Low expects no endpoint facts. Actual captured semantic output is a
prerequisite, not assumed from generator size. One plain-Java/no-annotation control
requires no Spring semantic facts. These fixtures do not establish all Spring roles.

Test-only counters record actual detector parse attempts/input bytes and method-
collector calls/input bytes. Existing detection/evaluation counters remain unchanged.
Bytes represent complete inputs passed to these boundaries, not actual character
comparisons or allocator bytes. Other nested collectors and core Java parsing remain
outside these counters. Timing covers complete compile_focused plus observations.
No hypothetical future budget, cache or authority reuse is introduced.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-spring-work.ps1
```

Expected13 tracked executions: two new work/negative tests, two protected Spring
evaluation/detection controls, eight existing Spring semantic tests, one existing
public provide/query Autowired publication control. Then main-package all-target/
all-feature warning-fatal lint, format, size and strict encoding; retain logs/hashes.
Agent performed bounded source/UTF-8/size/script checks only; no runtime or formatter.

## Disposition and accountable gaps

STRUCTURAL RISK / MONITOR pending actual counts. No production algorithm, semantic
eligibility, detection fallback, registry or producer generation changes. Sharing
annotation intermediates or compiler AST evidence requires a separate bounded design
with construction-inclusive benefit and exact semantics; it is not implied approval.

Configuration/Bean multiplicity, field and property density, comment/string and failure
behavior, nested classes/records/interfaces, ambiguous capture attribution, public
query lifecycle, negative fidelity, allocation and error costs remain open. Existing
helper e2e test names do not themselves establish public production reachability.
Properties/YAML, framework graph consumers and other Java language work are separate.

Syntax accounting remains9763 preserved/13 scoped/9750 pending plus96 callback
supplements; these named source paths are not newly disposed census records. Broader
audit and final independent coverage review remain incomplete. Traversal DESIGN REVIEW,
C2 MONITOR and protected scrub/hierarchy/C1/NgRx/A3 scopes remain independent.


Initial spring-work-20261105-165334 stopped at compilation: the new oracle called
as_str on EntityRef domain/entity_type, which already are static str references.
Corrected those four test expressions to compare the borrowed fields directly;
owned names still use String::as_str. No test executed or measurement RED claimed.
Production unchanged; corrected packet remains UNRUN.


## Actual Spring production characterization verified

spring-work-20261105-171440 completes13 tracked executions: two new work/negative
controls1.45s, two protected once-detection/evaluation controls0.04s, eight Spring
semantic controls0.01s, one actual provide/query Autowired publication control0.78s.
Zero failed/ignored. Main-package all-feature/all-target warning-fatal Clippy2m02s,
empty format-check output, active size6 files and strict encoding2112 text files pass.
All 1306 source hashes match at inspection before documentation updates.
inspection.json preserves28 parsed positive rows. No agent runtime was started.

### Independent class growth, one annotated method per class (High)

| Classes | Detector parses | Method helper calls | Helper input bytes | Endpoint facts | Compile us |
| ---: | ---: | ---: | ---: | ---: | ---: |
|1|1|2|112|1|21623|
|2|1|4|224|2|20572|
|4|1|8|448|4|16736|
|8|1|16|896|8|18759|
|16|1|32|1804|16|24710|
|64|1|128|7276|64|63800|
|256|1|512|29476|256|162589|

### Independent method growth, one class (High)

| Methods | Detector parses | Method helper calls | Helper input bytes | Endpoint facts | Compile us |
| ---: | ---: | ---: | ---: | ---: | ---: |
|1|1|2|112|1|21623|
|2|1|2|216|2|17326|
|4|1|2|424|4|21190|
|8|1|2|840|8|18466|
|16|1|2|1696|16|20626|
|64|1|2|6880|64|43326|
|256|1|2|28240|256|118031|

Shared baseline row is one fixture, not another independent execution. Joint16x16
observes one detection parse,32 helper calls,27328 helper-input bytes,256 endpoint
facts and110146us. Every Low fixture still detects/parses/evaluates once, but records
zero method-helper calls/bytes and no endpoint facts. Plain annotation-free Java
records one applicability invocation, zero detector parses/evaluations/helper calls,
and no Spring facts. Existing negative work avoidance must not be presented as absent.

All28 positive fidelity rows pass exact ordered relation/domain/endpoint/class/file
oracles; High endpoints equal C*M. One detector parse consumes the complete source
once per fixture (91..24100 bytes in the class axis). These counts cover the detection
parser boundary, not the compiler's own Java capture parsing. Helper input bytes are
summed request payload, not exact CPU scans, allocations or all nested brace/field
operations. The public publication control reaches actual provide/query but does not
supply a public whole-operation work-count attribution for these growth fixtures.

### Evidence-backed disposition

Keep Spring inner scanning STRUCTURAL RISK / MONITOR; no correction proposed from
this packet. Measured helper calls are2*C, with larger method bodies increasing bytes
and supported output. This is constant repeated projection work, not a measured
C*M complete-set candidate-selection defect. Timings include core compilation,
validation, output construction and observations, so increased latency cannot be
assigned solely to annotation rescanning. No alternative's allocation/API cost or
wall-time benefit is measured.

One detection/evaluation per compilation remains the protected baseline, not a new
repair opportunity. Reusing capture AST or sharing marker/semantic analysis would
need separate semantic/fallback and construction-inclusive proof. Current fidelity
short-circuits and plain-Java negatives already avoid inner work. Doing nothing for
this bounded slice is justified until consequential avoidable work is demonstrated.

Configuration/Bean paths (including potentially additional helper calls), field and
property density, comment/string fallback, records/interfaces/nesting, all error and
public currentness boundaries remain uncharacterized here. Continue those and other
uninspected subsystems independently. This report does not close Spring system-wide
coverage or source accounting:9763 original records preserved,13 scoped outcomes,
9750 pending plus96 callback supplements unchanged. No new runtime gate is required
solely to record this result; protected repairs are not reopened.
