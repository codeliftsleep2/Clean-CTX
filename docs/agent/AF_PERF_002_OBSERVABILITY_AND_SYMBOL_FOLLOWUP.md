# AF-PERF-002: exporter, transport and symbol-path follow-up

Agent: Agent MaxHeadRoom
Status: owner ACCEPTS source-tracing report without repair; no further work queued
on these evaluated candidates. Broader architectural coverage remains open.
Owner closes session-wide stats format guard FIX / PROTECTED. That scope stays
closed. This document records separate acquisition/caller distinctions.

## Reconciled baseline and repository state

Final stats composed receipt remains af-perf-001-20261106-001356: 61 passes and
224 matching source hashes at its recorded inspection. No new execution is claimed.
The working tree still contains the verified repair and earlier discovery packets;
latest commit at this inspection is 50e26a19 (scrubber prefilter correction).
No commit/stage/push was requested or performed. Verification completion and Git
publication are separate facts. Existing dirty files are preserved.

CBM acquisition returned degraded frontend_retired timeout. Discovery used the
documented host rg/native-read fallback. A provider timeout is not evidence that
a symbol or caller is absent. The bounded source search described below supplies
its own explicitly limited caller evidence.

## Remaining metrics acquisition is not the stats handler

Actual shipped path: mcp/server.rs condition on config.observability.export_metrics
-> spawn named metrics-exporter -> sleep configured interval -> registry.snapshot()
-> format_otlp_json() -> stderr. config.rs defaults export_metrics to false.
No server, exporter thread or profiler was launched for this investigation.

McpState owns Arc<MetricsRegistry>. Its snapshot loads four histogram snapshots,
scalar counters/gauges and the current error entries, then sorts errors by count.
Default histograms have 10/10/10/7 bounds: 37 bucket loads and 37 copied bound
values per snapshot, derived from source, not measured runtime counts. Recording
scans bucket bounds until a match, so its documented O(1) description relies on
fixed default populations; arbitrary public Histogram::new inputs need their own
bound if ever made consequential. No replacement is proposed.

The typed record_error API has seven categories, but the registry exposes its
DashMap field publicly: do not claim a global seven-entry cardinality invariant.
Targeted source search found no production record_error calls or registry counter
updates beyond construction/export in the inspected src tree. Existing unit tests
exercise them; helper presence does not prove normal acquisition integration.
Comments claiming context_stats exposes this registry do not describe the current
handler, which uses SessionStats. This is a documentation/reachability discrepancy,
not authorization to wire new producers or change telemetry behavior.

format_otlp_json consumes selected aggregate values, not every cloned histogram
bound/count or sorted error entry. Thus there is a possible snapshot overcollection
on the optional exporter path. It is not a measured FIX: fixed default histogram
work is small, normal producers were not found, export is opt-in, and no measured
latency/allocation pressure or worthwhile correction has been established.
Disposition: MONITOR for acquisition/representation coverage; no optimization.
Avoid turning a source observation into a new registry or metrics integration task.

startup.rs checks CLEAN_CTX_STARTUP_TRACE_FILE before allocating JSON/opening its
trace file. event_detail converts its argument before that gate, and caller-side
formatting can occur even while disabled. Current inspected call sites are startup,
transport admission/indexing or error events, not a demonstrated source-size hot
loop. Separate opt-in IO/currentness and error behavior must remain intact. No
filesystem observation, flush, environment policy or caching change is proposed.

## Protocol transport: distinguish laboratory capture from production

protocol.rs::send_response_checked has mutually exclusive cfg(test)/cfg(not(test))
branches. Tests clone the Value into their capture queue and serialize/print it;
production acquires the stdout mutex and stdout lock, serializes once, writes and
flushes. That test capture clone is not evidence of a production response clone.
Serialization under the lock may matter under contention, but no contention or
failure measurements exist here. Moving serialization across lock acquisition
would be an independent observation/ordering change, not an incidental follow-up
to format guarding. No lock or transport changes are queued.

## Symbol dictionary: concrete local allocation candidate, caller-limited

dictionary/symbol.rs::encode has two word-flush sites, for a delimiter and the
final word. Both evaluate:

`self.forward.get(&word).cloned().unwrap_or(word.clone())`

Source proof: unwrap_or evaluates its default eagerly, so word.clone executes on
both a dictionary hit and a miss. On a hit, the cloned fallback is unused while
cloned() separately owns the replacement. Output needs an owned result String,
but not necessarily those intermediate owned strings. This is a source-proven
unused intermediate allocation candidate, not measured allocator traffic or a
production optimization approval. Matching, Unicode word classification, opcode
assignment/order, punctuation and footer policy are independent semantic contracts.

Actual caller: compression/symbol_compression.rs invokes dictionary.encode only at
Low fidelity; other fidelities return an owned body directly. Pipeline file/text
assembly calls that pass. layers/language implementations delegate compile to
compress_text, while default successful MCP IR compilation uses the distinct
ir::layers and IRCompiler path. Same language names do not make those entry points
equivalent. No production caller of the layers::language compile methods was found
in the targeted src search; this is a finite source result, not external library
usage telemetry. Public library pipeline APIs remain real entry points but their
existence does not establish default MCP request reachability.

Disposition: DESIGN REVIEW / low-priority candidate pending actual caller demand
and hit/miss byte/allocation measurements. Do not label this a default-handler FIX,
move it ahead of consequential default production paths, or apply a borrowed-string
rewrite without its own narrow equivalence/budget evidence and owner disposition.

The default IR GlobalSymbolTable is a distinct owner. pipeline/core.rs registers
class/interface captures through generated aliases; symbol_table.rs only scans
an old file-members vector during alias overwrite/unregister. A scan inside that
conditional does not establish quadratic fresh registration. Overwrite population,
file membership and lifecycle reachability would require separate measurement.
The closed replay no-repair decision does not authorize reopening it through this
symbol-table trace. No symbol identity, reverse-index or version policy is changed.

## Coverage and next priorities

This pass distinguishes four actual production/library boundaries without adding
performance tests for low-priority or unproven scenarios. It advances source-path
coverage only. Measurements, allocator counts, exact semantic characterization and
whole-subsystem acceptance are not claimed. No new runtime command is handed off
merely to validate these notes.

Prefer the next consequential default-operation investigation, including the still
open A1 publication-copy boundary or A4 dependency obligations, over an exporter
or legacy/library rewrite. Any copy-volume characterization must count its actual
boundary, ownership and dependent publications; the previous zero accessor count
is not proof of zero copying. Broader compaction/rendering/diff/edit and subsystem
paths remain open, and investigation does not imply optimization.

Syntax accounting is unchanged: 9763 original records, 13 scoped dispositions,
9750 unresolved plus 96 callback supplements. None of the source observations in
this document classify a frozen syntax record, establish a new performance
invariant or substitute for exhaustive source-wide reconciliation.
