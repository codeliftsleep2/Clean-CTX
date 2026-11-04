# AF-PERF-002: AST census result and remaining inspection queue

Agent: Agent MaxHeadRoom
Status: first AST packet verified GREEN; source-wide classification incomplete.
Generation 78 semantics remain baseline; no new production optimization authorized.

## Verified operator execution

membership-census-20261104-085055 passes one AST enumeration test (5.33s), two
enclosing characterizations (0.03s) and nine af174 controls (0.24s). Zero failures
or ignored tests. All-feature/all-target warning-fatal Clippy completes cleanly
(1m21s); format logs empty; encoding passes 2036 text files. All 525 recorded Rust
source hashes match current files at inspection, before subsequent new test-only
registration described below. The receipt records discoveryComplete=false.

Evidence: target/af-perf-002/membership-census-20261104-085055/receipt.json and
census/{files,sites,calls,scope}.json. Counts below are syntax, not syscall/work
measurements, independent defects, or human classification completeness.

| Census unit | Count |
| --- | ---: |
| Enumerated source/support Rust files | 525 |
| Candidate syntax sites | 9763 |
| Raw call expressions | 40863 |
| Collection/string operation syntax | 3922 |
| Manual loops | 1455 |
| Equality syntax inside manual loops | 1160 |
| Unexpanded macro review records | 3226 |

These include inline cfg(test), string operations and unrelated fixed-size work;
no type-resolved or macro-expanded graph is claimed. Source path enumeration agrees
in count with the prior independent filesystem enumeration. Candidate-dense files
include angular_meta/template.rs (151), proxy/src/transform.rs (147), C# compilation
(102) and dotnet semantic extraction (96). Density is a review scheduling aid,
not an impact ranking or justification to optimize them first.

## Exact classification accounting

The existing 24 operation records have differing inspection/caller limitations.
A separate source-hash-bound inspected-site-classifications.json now classifies
13 syntax records in this receipt: four MEASURED DEFECT records for the two
declaration scan chains, seven STRUCTURAL RISK records for selector/trace/forms
accumulators, and two JUSTIFIED WORK records for fixed primitive vocabulary scans.
Nested iterator and terminal method records are distinct syntax sites for one
logical scan; four syntax records do not imply four measured declaration defects.

**9750 syntax records remain UNRESOLVED/unclassified**, including macro review
records and known incidental/test-only code not yet individually separated. These
are not 9750 confirmed expensive operations. classification-progress.json records
both denominators. No whole-file safety approval follows from source hashes.

## Parser boundary: explicit REVIEW REQUIRED

src/mcp/semantic_compilation/artifact.rs is the sole file with root.has_error=true.
The Rust Clippy gate passes, so this flag must not be reported as a compiler failure.
Its exact grammar/error-node cause is not established by the current census output.
The file uses cfg-qualified struct fields/patterns, but those are not asserted to
be the cause without error-node evidence. Raw call owners include from_compiler,
into_parts and take_retained_authority; their presence does not prove all nested
boundaries were recovered. Keep artifact construction/consumption caller coverage
REVIEW REQUIRED and manually trace it before closure. Do not omit the file, strip
syntax, change production code for the parser, or infer absence from partial AST.

## Next independently authorized characterization: populated NgRx no-ops

The earlier empty/duplicate call packet had zero candidates. New tracked
index_ngrx_noop_af_perf_002.rs adds two tests at actual add_edges:

- Independently vary populated candidate count and empty/duplicate invocation count
  over 1/2/4/8/16. Record candidate visits, rebuilds, retired edges and actual export
  reads/parses, preserving independently supported caller output.
- Change a consumed on-disk action from exported to non-exported with the same name,
  call add_edges with no input edges, require old refined dispatches to retire, then
  repair the bytes and require support to return. This tests whether an index-level
  no-op is also a semantic no-op; it does not preapprove skipping revalidation.

No production algorithm, currentness or reconciliation policy is changed. These
tests are authored UNRUN. The currentness mutation uses export removal, not a new
const-versus-let eligibility policy. Object-member binding join counts and full
map/switch candidate families remain separate missing characterization.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-ngrx-noop.ps1
```

The packet runs only two new tests and nine existing membership controls, then
focused all-feature lint/format/size/encoding; it does not repeat the AST census or
historical campaign. Agent performed only bounded inspection and driver syntax/
encoding review, not Cargo/runtime execution. New registration changes source
identity; previous GREEN remains a frozen pre-edit result, not a post-edit claim.

Broad first-pass classification, declared caller and behavioral-dimension coverage,
blocking drift enforcement beyond finite known paths, and independent Agent Popper
review remain outstanding. The declaration repair design still awaits disposition.

## Owner completion boundary and populated characterization return

See [AF_PERF_002_INVENTORY_ACCOUNTING.md](AF_PERF_002_INVENTORY_ACCOUNTING.md)
for lossless reconciliation of all 9763 original records, seven reviewed logical
operations, 9750 blocking macro/review outcomes, 96 supplemental all-callback
obligations and explicit src/lib.rs drift. No automatic test-only exclusions or
blanket file-level approvals were used. Closed accounting is not completed review.

Agent Popper independently found bare-name owner collisions, omitted iterator
multiplicity and scalar-versus-iterator ambiguity; methodological scoped review
is recorded, final adversarial acceptance is outstanding. Populated NgRx packet
verified 2+9 passes, clean lint/format/encoding and export retirement/byte repair.
Empty input alone cannot justify skipping reconciliation. Exact parser-error
span/impl/function diagnostic is authored UNRUN under a narrow operator handoff.
No production optimization or declaration representation change was made.

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
