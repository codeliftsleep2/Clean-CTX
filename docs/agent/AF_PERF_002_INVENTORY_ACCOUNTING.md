# AF-PERF-002: closed candidate accounting, open investigation

Agent: Agent MaxHeadRoom
Status: owner completion boundary applied; accounting reconciles, classification
and coverage remain blocking. No production optimization approved or implemented.

## Original candidate denominator preserved

The accepted frozen membership-census-20261104-085055 packet is the denominator.
Offline reconcile-membership-census.py assigns C00001..C09763 to original records
in original order, retains every record and its SHA-256, records exact source
identity, and emits one accountability state for each original record. Its output
is investigation evidence, never runtime authority or a generic fitness registry.

Run metadata: target/af-perf-002/membership-accounting-002/{summary,files,operations}.json
and candidates.jsonl. Offline metadata reconciliation ran; no Rust/test/runtime
was started by the agent. Original census/log bytes were not rewritten.

| Accountable outcome | Original syntax records |
| --- | ---: |
| PRODUCTION OPERATION | 6 |
| INCIDENTAL/BOUNDED | 1 |
| TEST/SUPPORT ONLY | 0 |
| DUPLICATE SYNTAX | 6 |
| MACRO REVIEW | 3226 |
| REVIEW REQUIRED | 6524 |
| Total | 9763 |

Six production scan anchors plus the fixed opcode scan map to seven reviewed
logical operations. Six nested iterator records link to exact canonical candidate
IDs, preserving original source start and reviewed expression-prefix evidence.
This grouping applies only to the named manually inspected chains; no module-wide
or method-spelling exclusion is used. The single bounded outcome is the existing
source-defined primitive vocabulary, not a universal cap on production input.
No automatic test/support exclusion is claimed: inline cfg(test) needs structural
scope evidence before it can be removed from production classification.

**9750 original candidates remain blocking** through macro/review outcomes. Closed
accounting does not mean completed inventory, classified production paths or absence
of high-risk defects. Remaining operations and uncovered caller boundaries are
not excused by reconciliation or semantic test success.

## Source drift, parser and raw calls remain distinct obligations

Of the original 525 files, src/lib.rs differs from the frozen source hash due to
later test registration. Its old syntax stays in the ledger and drift is explicit;
new/changed source needs a separately linked supplement, never replacement of old
records. Source hashes bind evidence, not whole-file behavioral approval.

artifact.rs remains the sole parser-flagged source. The current census contains
some recovered calls, but exact error/missing-node spans and lost boundaries are
not established. A new tracked af002_artifact_parser_boundary diagnostic records
every ERROR/missing node byte/line interval, recovered function interval and impl
owner with the exact artifact source hash. It is authored UNRUN. Production code
is not modified for parser compatibility; recovery does not automatically accept
coverage. Manual source/caller reconciliation must follow the output.

Popper identified `all` as missing from candidate-method enumeration. A supplement
retains **96 raw .all call obligations** from the original 40863 raw call records,
keyed by original raw-call ordinal and kept REVIEW REQUIRED. These do not silently
change the original 9763 denominator. They demonstrate why textual/finite AST
method enumeration alone cannot establish discovery completeness. Other callback,
trait, alias and macro paths remain unresolved too.

## Agent Popper: independent scoped challenge

Owner explicitly requested independent adversarial review. Agent Popper inspected
the frozen census/source and returned concrete counterexamples:

- host.rs has Witness::current and Host::current under the same bare function name.
  Grouping by file/name alone would merge different currentness boundaries. Keep
  exact operation anchors and manually established owner; recover full impl/function
  byte intervals before general automatic grouping. The seven current groups use
  reviewed exact expressions/source starts, not a whole-function exclusion.
- Host::current calls witnesses.iter().all(Witness::current). Manual-loop depth zero
  cannot mean no repeated work; iterator/delegated callback multiplicity is absent.
- artifact.rs Option::filter is a scalar operation; iterator filter uses the same
  syntax spelling with a different work unit. Receiver inspection is mandatory.
- Overlapping iter/filter/collect nodes must retain original IDs after grouping.
- Source drift, parser failures and macro arguments/generated dispatch cannot be
  waived by naming, compilation or old performance tests.
- PageRank, unregister and subclass-query default reachability remains unproved;
  local complexity observations cannot become production acceptance.

Popper supports lossless accounting as a method, not final inventory acceptance.
The full first pass and subsequent adversarial closure review remain incomplete.
No inventory-complete statement is justified by this scoped review.

## Populated NgRx characterization: verified outcome

ngrx-noop-20261104-090950 passes two new tests (2.67s) plus nine af174 controls
(0.26s), zero failed/ignored. Clippy is clean (1m05s), format logs empty, encoding
passes 2039 files. All 17 receipt hashes match at inspection before the new parser
diagnostic edit; that edit requires its own affected gate, not a post-edit GREEN claim.

At 16 populated candidates and 16 empty calls: 16 rebuilds, 256 candidate visits,
256 retired refined edges, 256 export reads and parses; observed 82793us. Duplicate
declaration calls give the same work counts, observed 95166us. Candidate/call growth
dimensions vary independently over 1/2/4/8/16. These are descriptive small-fixture
instrumented times, not production latency attribution.

The source mutation control removes export while leaving the declaration name,
performs empty add_edges, and observes four dispatches retire. Repairing the export
and performing another empty call restores four supported callers. Each transition
performs one rebuild and four export reads/parses. **Empty input is not a semantic
no-op when consumed filesystem evidence changes.** Skipping rebuild merely because
no input edges were added would change this tested behavior. No skip, cache,
deferral or currentness reuse is approved. Populated map/switch binding joins and
unrelated adjacency dimensions still need separate characterization.

## Next narrow operator command

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-parser-boundary.ps1
```

One new tracked parser diagnostic, affected format/size/Clippy/encoding and source
hash receipt; no repeated source census or prior correctness campaign. The agent
only inspected syntax/encoding and reconciled existing metadata. After the output,
recover the precise parser boundary, add owner/interval/callback-aware grouping,
classify macros/test scopes with evidence, and continue production caller inspection.
No declaration representation change follows without separate owner disposition.

## Scoped parser recovery verified

[AF_PERF_002_PARSER_RECOVERY_RESULTS.md](AF_PERF_002_PARSER_RECOVERY_RESULTS.md)
records the GREEN one-test diagnostic, clean affected lint/format/encoding, 1282
matching receipt hashes, three exact error spans confined to cfg-qualified field
destructuring and all 23 recovered function/impl boundaries. Agent Popper accepts
manual recovery of this syntax boundary only; artifact caller/cost classification
and final inventory acceptance remain open. Original parser flags/record identities
are retained; the accounting-003 supplement attaches recovered intervals without
automatically clearing any candidate classification. No production changes or
additional user-run gate are required for this metadata/documentation return.
