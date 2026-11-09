# AF-PERF-002: broad default-operation discovery after SQLite closure

Agent: Agent MaxHeadRoom
Status: characterization executed GREEN; bounded startup design investigation recommended.
All previous scopes remain CLOSED / FIX / PROTECTED. Dense equality deferred.

## Acquisition and selection

Fresh Clean-CTX status is degraded, frontend_retired timeout. No provider launched;
known-path host reads supply this bounded trace, not inferred absence. Registry
create_default_registry exposes list_sessions and refresh_workspace alongside the
existing core operations. This pass examines an uncovered construction lifecycle
and uses a real registered read operation as its necessary-work comparison.
No completed encoder/SQLite conversion/overload/formatting correction is reopened.

## Provisional candidates, not measured performance ranks

1. McpState::new -> committed BufferedStore::sqlite -> SqliteStore::list_contexts
   with i64::MAX limit -> physical_identities.register_durable_owner(file_path).
   Startup consumes only file_path but query constructs summaries containing token
   counts, fidelity, timestamp and per-context indexed COUNT(deltas), sorts all
   contexts and allocates the result vector. Potential avoidable work grows with
   context/delta population on every enabled-persistence state construction. Existing
   ordering and metadata conversion errors can affect physical-owner registration;
   narrower reads are NOT automatically equivalent. No new query or policy selected.
2. Registered list_sessions -> same list_contexts(200). Here counts/fidelity/tokens/
   timestamps/order are real output requirements. Work may scale beyond200 due to
   sorting/selection, but a query plan/VM measurement must distinguish that from
   indexed necessary count work. No SQL/index rewrite is proposed. This is a control
   and a source candidate, not presumed second defect.
3. refresh_workspace -> hydration filesystem scan -> root_inventories filters and
   clones combined.entries for each root. Root-by-entry work and overlapping scoped
   payload copies are visible in source; per-root manifests may require that work.
   No measurements or avoidability proof here; retain as later uncovered hypothesis.

The first candidate is the provisional next DESIGN question because an actual
consumer discards most returned fields. Implementation risk is higher than a local
copy removal: startup currently skips all durable-owner initialization on metadata
read error, and order can choose the first durable spelling for physical aliases.
No new global index/cache/synchronization is needed to ask the bounded question.
Any proposed narrower reader requires separate error/ordering/physical-identity
analysis and owner approval. Do not silently replace it with SELECT file_path.

## Characterization boundary

Two tracked tests seed ordinary committed ContextStore rows in a temporary file
SQLite store, varying0/1/16/128/256 contexts and0/16/128 delta rows per context across
12 populations. Delta setup is transactional and outside measurement. These are
controlled database fixtures, not compiled enterprise workloads or real frequency
estimates. Each runs actual McpState construction with persistence enabled and CBM/
proxy disabled, then actual registered list_sessions. Full response is retained and
checked against committed summary formatting/population. Pending count and semantic
edge population remain zero; no history is promoted into live authority.

SQLite VM/fullscan counters and EXPLAIN QUERY PLAN are collected from a separate
statement using the SQL text extracted VERBATIM from actual list_contexts source,
for startup unlimited and listing200 limits. This is source-exact standalone query
work, not instrumentation of the actual production statement or CPU/public latency.
The normal schema already has deltas(context_id,edit_sequence) index; do not falsely
claim repeated full delta-table scans. Query planner decisions, indexed entry work,
fullscan steps and rows are separate facts. A fullscan counter alone omits index
steps; VM instructions are SQLite operations, not machine instructions.

Single actual startup/listing elapsed observations include their full local operation
and response but are descriptive, unpaired and not causal attribution to the query.
No replacement is implemented or timed. Statement setup/materialization/drop occur
in the observed standalone query; connection/fixture setup is excluded. There is
no allocation/peak-memory/engine-copy measurement. Large-loop counters are not CPU.

The malformed-fidelity case establishes that actual list_contexts errors even when
path is valid; startup source catches this with if let Ok and continues construction.
A successful construction is not proof that durable owner registration occurred.
No fabricated live alias or canonical ownership assertion is introduced. Future
physical-alias/error-oracle controls would be required for a design proposal.

Existing registered_list_sessions_observes_only_committed_rows independently protects
read-only pending-write behavior. No expensive composed gate or old repair revalidation.

## User-owned execution

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-broad-startup-discovery.ps1
```

Expected three tracked passes,12 complete operation rows with24 source-exact query
observations/plans, warning-fatal all-target/all-feature Clippy, format/size/encoding
and source hashes. Startup/state/SQLite/encoder/matcher/edit protected identities
must remain unchanged. Only new test files and test registration are formatted.
Full logs stay under target; console emits short summaries. All runtime UNRUN.
Agent performed bounded source/script/UTF-8 review only. No stage/commit/push.

Return after execution: rank observed avoidable work, exposure, complexity and risk;
recommend at most one next bounded design/correction slice, without implementation.
If data show necessary/indexed work dominates or no consequential opportunity, say
so rather than manufacture a defect. Root manifests remain unmeasured. Dense equality
stays deferred; census unchanged13/9763 scoped,9750 unresolved plus96 supplements.
## Executed characterization and ranked return

Agent MaxHeadRoom inspected broad-startup-discovery-20261108-214510: three tracked
passes, zero failed/ignored. Two discovery tests115.30s and existing registered
committed-only listing2.00s pass. All12 populations/24 source-exact query observations
complete. Warning-fatal all-target/all-feature Clippy34.09s, both format logs empty,
size19 active files and strict encoding2261 text files GREEN. All1389 source hashes
and eleven protected identities match at inspection, before this documentation
update. inspection.json retains full rows and checked counts. No agent runtime,
production change, composed rerun, stage, commit or push.

SQLite VM steps from the SOURCE-EXACT standalone query, not a production-statement
probe or CPU count:

| Contexts / deltas per context | Startup rows / VM steps | Listing rows / VM steps |
| --- | ---: | ---: |
| 0 / 0 | 0 / 11 | 0 / 11 |
| 1 / 0 | 1 / 44 | 1 / 44 |
| 16 / 0 | 16 / 539 | 16 / 539 |
| 128 / 0 | 128 / 4235 | 128 / 4235 |
| 256 / 0 | 256 / 8459 | 200 / 8179 |
| 1 / 16 | 1 / 91 | 1 / 91 |
| 16 / 16 | 16 / 1306 | 16 / 1306 |
| 128 / 16 | 128 / 10378 | 128 / 10378 |
| 256 / 16 | 256 / 20746 | 200 / 20466 |
| 1 / 128 | 1 / 427 | 1 / 427 |
| 16 / 128 | 16 / 6682 | 16 / 6682 |
| 128 / 128 | 128 / 53386 | 128 / 53386 |

Plans show SCAN c, CORRELATED SCALAR SUBQUERY, SEARCH d USING COVERING INDEX
idx_deltas_context(context_id=?) and temporary B-tree ORDER BY. Context fullscan
steps are N-1 for populated multirow cases, independent of deltas; index count
work increases VM steps without increasing that fullscan counter. This is indexed
counting, not N full delta-table scans. Output limit200 does not bound all query
work:256 contexts still show255 fullscan steps, with similar VM growth. These facts
do not prove how many delta/index entries the engine inspected or allocate CPU
percentages to sorting/counting. No query rewrite or new index was tested.

Actual startup observations range1.824..9.620ms in nonempty populations; actual
registered listing0.276..4.525ms. Empty startup6.005ms illustrates fixed/cold effects.
These are SINGLE complete-operation observations with fixture preparation excluded;
not paired benefit or public-latency improvement measurements. Real user database
populations and startup/request frequency remain unknown. Necessary physical-owner
registration, connection/schema work, metadata extraction, sorting and response
formatting coexist; causal time attribution is unavailable.

The malformed row produces exact actual reader error InvalidColumnType(1,
"fidelity", Blob). State construction still completes because its source uses
if let Ok and skips the registration loop on read failure. This confirms a real
acceptance boundary; it does not independently inspect the internal registry or
prove all alias/lifecycle behavior. Valid list responses are exact for seeded
committed populations, sorted by distinct timestamps with expected delta/token/
fidelity data. No pending writes or live semantic edges are created.

### Ranked engineering shortlist, with evidence limits

1. **Startup full-summary consumption: DESIGN REVIEW recommended.** Actual default
   construction calls the reader unbounded and consumes only file_path. Complete
   metadata projection/owned summary vector and indexed delta-count work are
   source-demonstrated; measured query work grows with both context and delta axes.
   Benefit remains a hypothesis until a semantically equivalent alternative is
   measured. Exposure is enabled-persistence startup, not every tool request.
   Complexity/risk are moderate: retain physical-owner order/hard-link spelling,
   complete-read failure behavior, NULL/type/conversion/schema failures and committed
   ownership. A small operation-local reader might suffice, but SELECT file_path
   alone is NOT established equivalent. No new cache/global index is warranted.
2. **Listing bounded-output query work: measured scaling, no repair selected.**
   Actual registered list_sessions needs all emitted summary fields and counts.
   Source-exact query visits/sorts more contexts than output at256, but removing
   that work safely requires a separate SQL/order/error argument. Indexed COUNT
   work is necessary for listed rows. No demonstrated semantically removable work
   or simpler safe correction yet; priority below the discarded startup consumer.
3. **Per-root inventory filtering/cloning: unmeasured structural hypothesis.**
   Source root_inventories considers combined entries per root and clones selected
   entries. Scoped manifests may require these copies; no avoidability or benefit
   conclusion. Keep for later characterization rather than claim CPU rank from
   source complexity. This pass does not certify broad operation coverage complete.

Recommend only one next slice for owner review: a bounded DESIGN investigation of
startup durable-owner enumeration, comparing complete current summary query against
an operation-local alternative only if it preserves failure/ordering authority.
First enumerate every consumed error and physical-identity dependency. Preserve
original scope; do not change SQL/index/schema, bypass malformed-row refusal, split
fail-closed behavior, add persistent state or optimize implementation under this
recommendation. If exact equivalence requires a policy change, return that decision
instead of silently adopting it. No production correction is authorized here.

This shortlist ranks engineering opportunity/risk, not measured CPU dominance.
There is no comparable evidence ranking dense same-signature equality above these
new paths, so it remains deferred. All protected repairs stay closed. No further
runtime is requested merely to repeat these single observations or seek better
numbers. Census unchanged; repository publication remains separately unauthorized.