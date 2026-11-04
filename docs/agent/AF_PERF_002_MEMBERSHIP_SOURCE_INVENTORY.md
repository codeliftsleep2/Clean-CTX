# AF-PERF-002: source-wide membership investigation ledger

Agent: Agent MaxHeadRoom
Status: expanded enumeration and initial inspected ledger; discovery INCOMPLETE.
Owner directive: exhaustive pattern discovery, selective remediation. No additional
production optimization approved. Declaration companion design remains proposed.

## Search accounting and boundaries

Expanded local enumeration includes **525 Rust source/support files** under src,
proxy/src and examples, plus build.rs/proxy/build.rs if present. **736 explicit
test/fixture Rust files** under tests/test_files directories are excluded and
recorded. Inline cfg(test) blocks remain in this conservative source superset.
Root Cargo.toml declares the main crate and proxy workspace member. Examples are
auxiliary targets, not evidence of default production reachability. Verification
Rust fixtures/drivers outside these roots are verification assets, not production.

The expanded lexical census records **8334 matching lines**, covering contains,
any/find/position/filter, retain/dedup, sorts/binary search, collection rebuilds,
keys/values iteration, manual loops and equality. This is not a count of distinct
AST sites or defects: comments, strings, tests and several operations on one line
can affect that number. Full output and per-source hashes are preserved under
target/af-perf-002/membership-expanded-lexical. The earlier 760-line/218-file search
was narrower and did not include the proxy; it is superseded for search scope,
not discarded as historical evidence.

The **28 operation records** in this ledger and its artifact supplement have named source bodies inspected and provisional
classifications. This count is not comparable to raw matching lines. Remaining
lexical matches are UNRESOLVED/uninspected unless explicitly covered by a record;
even a reviewed function can contain other unclassified paths. The number of
actual source-wide AST sites and uninspected AST sites is not yet available.
No claim of exhaustive inspection or finding closure is made.

CBM status was consulted and reports degraded/frontend-retired timeout. Local
source inspection is the available fallback; no type-resolved external call graph
is available. A tracked test-only Rust AST census is authored to cross-check the
file enumeration, identify function/line/byte locations, manual loops and raw call
edges, and preserve parser errors/unknown dispatch as review obligations. It uses
existing tree-sitter-rust, adds no dependency and does not change runtime behavior.
It is UNRUN and cannot supply site counts until operator execution.

## Inspected operation records

Notation: E input/relationship edges; U unique identities; O physical occurrences;
K publisher keys; A artifacts; F fields; V validator rules; B request blocks; M
stored map entries. Source-derived scaling is a hypothesis until measured at its
enclosing operation. Local lexical calls are not a type-resolved call graph.

| ID / exact file and function | Enclosing caller and collection/identity | Growth / scan frequency / complexity | Classification / disposition | Coverage and smallest proposal |
| --- | --- | --- | --- | --- |
| D1 workspace/index/declarations.rs::register_declaration occurrence scan | add_edges self-Defines branch; Vec<EntityRef>, exact entity.file within EntityKey; ordered persistent owner | O occurrences across calls; one growing scan per declaration; measured n(n-1)/2 | MEASURED DEFECT / FIX proposed; latency consequence beyond fixture unresolved | Characterization n<=256 GREEN; budget RED and lifecycle/mutation proof required before separate two-companion repair |
| D2 same function publisher-key scan | same add_edges branch; file_declarations Vec<EntityKey>, asserting file distinct from physical occurrence | K unique declarations in one file; one scan per append; measured n(n-1)/2 | MEASURED DEFECT / FIX proposed | Same packet independently isolates dimension; no endpoint companion may grant declaration authority |
| S1 workspace/index.rs::resolve_selector | angular_templates::selector_candidates -> resolve_selector; subject_keys Vec<EntityKey>, first encounter order | E matching assertions, U subjects; per HasSelector edge growing scan, O(E*U) upper structure | STRUCTURAL RISK / MONITOR | Existing selector semantics are not a work budget; measure scoped public template query and fan-in/duplicate dimensions; consider local seen set only after justification |
| T1 cbm/client.rs::extract_trace_edges | CbmClient::trace_path -> decoder::trace, also collection decode; Vec<Value> exact from/to pair, first-seen provider order | E decoded direct edges/U pairs; scan accumulated output per pair, O(E*U) | STRUCTURAL RISK / MONITOR | trace_wire semantics exist, counts absent; characterize through actual decoder including rejected hops/duplicates; possible local pair set, no contract change |
| C1 angular_meta/config_composition.rs::project output dedup | retained config-composition slot and producer projection; Vec<SemanticEdge>, exact canonical subject/object pair | owners/arguments/U pairs; one output scan per proved candidate; proof scans add independent work | STRUCTURAL RISK / MONITOR | AF batching bounds whole projection calls, not inner pair comparisons; measure argument growth, forged/partial behavior; local canonical pair set only if existing identity supports exact equality/hash |
| F1 angular_meta/reactive_forms_normalize.rs::normalize_artifacts | AngularMetaLayer extraction -> reactive_forms extraction -> normalize; Vec<FormArtifact>, form name or field name+structural kind | A inputs/U compatible artifacts; position scan per drained artifact O(A*U) | STRUCTURAL RISK / MONITOR | Need artifact-density/depth production extraction counts; preserve incompatible evidence and first output position; no global form registry |
| F2 same file::merge_fields | normalize -> merge_artifact -> merge_fields, also reactive_forms merge call; Vec<FieldDecl>, name+kind | F incoming and accumulated siblings; iter_mut/find per field plus recursive merges | STRUCTURAL RISK / MONITOR | Flat width and nested depth independent; measure recursion/merge multiplicity before considering local lookup |
| F3 same file::merge_unique | merge_field -> validators; Vec<String>, exact rule spelling, first occurrence order | V existing/incoming; contains per incoming, O(V*U) structure, persists only through enclosing merge | STRUCTURAL RISK / MONITOR | Counts absent; local seen set plausible but not approved; allocations/key bytes and actual list sizes matter |
| F4 angular_meta/reactive_forms_validators.rs::push_rule | validators_from_value/body -> collect_validator_expression -> push_rule; Vec<String>, exact callable/normalized rule text | expressions and U rules; scan per accepted rule | STRUCTURAL RISK / MONITOR | Declaration and form semantics not transferable authority; characterize independent rule count/duplicates first |
| N1 dotnet_meta/general.rs::extract_all_class_names | extract_general -> extract_services -> extract_all_class_names; Vec<String>, literal extracted names | source bytes, matched declarations and U names; fixed pattern passes plus growing contains | STRUCTURAL RISK / MONITOR | Many pattern passes can see one declaration; identify actual source-sized caller slice and duplicate density; possible local set while preserving order |
| O1 compression/opcodes.rs::is_primitive_opcode | SymbolDictionary::format_footer; fixed PRIMITIVE_OPCODES vocabulary, opcode equality | one fixed vocabulary scan per lookup, no growing collection | JUSTIFIED WORK / ACCEPT for this membership pattern only | Source explicitly documents bounded table tradeoff; do not add a set merely to ban any; caller repetition remains a separate work unit |
| P1 intelligence/pagerank.rs::compute_pagerank | public compute API; default production caller not established; HashMap score inputs and normalized borrowed-key maps | whole-map maxima/normalization passes once, then keyed gets per symbol; expected O(input facts) excluding hashing/strings | JUSTIFIED WORK / ACCEPT for local membership; reachability UNRESOLVED | No per-key growing map scan observed in read body; do not label all values()/keys() iteration defective or claim default integration |
| X1 proxy/src/cache.rs::inject_breakpoints | proxy server request transformation -> inject; JSON tools/system/message blocks, client breakpoint and last eligible position semantics | request B blocks; multiple semantic passes, no growing dedup Vec found in inspected body | JUSTIFIED WORK / ACCEPT for this local antipattern; request total uncharacterized | Client-sent breakpoints early-return; scans are not per-element growing membership. Do not alter transport policy; measure request multiplication separately |
| K1 cache.rs::forget_file | durable_semantics retirement -> cache_write().forget_file; registry/baseline maps retained by normalized path | one narrow retirement scans three maps; F retirements may cause O(F*M) | STRUCTURAL RISK / DESIGN REVIEW | Need enclosing retirement counts, alias/fidelity ownership and lock scope; no path normalization or invalidation redesign approved |
| G1 workspace/index/traversal.rs::build_node_index | transitive_dependencies_with_scope and graph consumers -> build_node_index; HashSet identity union -> sorted Vec + HashMap | all workspace nodes plus sort, even potentially small reachable component; Q repeated traversals repeat build | STRUCTURAL RISK / MONITOR | Not a growing contains defect; collection reconstruction may dominate selective query; preserve scope/order and characterize unrelated-growth before proposal |
| R1 ir/symbol_table.rs::GlobalSymbolTable::register | CoreIRPass::run class/interface captures -> symbol_table_mut().register; symbols HashMap and file_members Vec<String>, alias identity | fresh alias uses keyed lookup; overwrite scans old file Vec with retain then appends; repeated overwrites can multiply by file symbol count | STRUCTURAL RISK / MONITOR, specifically overwrite workload | Determine table lifetime and alias reuse before claiming default recompilation amplification. Preserve version_first/version_last, reverse name ownership and order; no companion selected |
| R2 same type::unregister | public table mutation; actual live enclosing caller not established in current search; symbol removal -> file_members retain | removed aliases D and surviving file members S; repeated removal can scan shrinking list, O(D*S) structure | UNRESOLVED / MONITOR | Standalone API cost is known; production exposure/frequency unresolved. Do not infer ContextState removal uses this table merely from method names |
| R3 diff/differ.rs::diff_methods | MCP tool_helpers snapshot diff and gitdiff/engine -> diff_snapshots -> diff_class -> diff_methods; name-group Vecs of CapturedMethod | baseline and current same-name overload counts; exact-equality pass and signature pass each search unused current candidates, O(B*C) per name group plus method equality cost | STRUCTURAL RISK / MONITOR | Existing overload semantics are not a scaling budget. Characterize unchanged/reordered/duplicate/ambiguous methods at real diff boundary; exact matching precedes signature matching, so a name-only set would be incorrect |
| R4 ir/query.rs::IRQueryEngine::find_subclasses | public query engine API; MCP/default reachability not established; child_ids Vec<&str> then graph.nodes filtering | one edge pass plus contains scan per node: O(E+N*C) source-derived, C matching child IDs | STRUCTURAL RISK / MONITOR | Repeated call multiplicity and duplicate Extends facts unmeasured. A local child-ID set might preserve node output order, but needs evidence and approval; no global graph index proposed |
| R5 dictionary/path.rs::PathDictionary::remove_path_alias | durable_semantics::forget_dictionary_path -> dict_lock -> removal; forward alias/path check and reverse HashMap retain | one validated alias retirement scans all reverse entries; repeated retirements may multiply dictionary size | STRUCTURAL RISK / DESIGN REVIEW | Existing reverse keyed lookup does not by itself prove one removable key: validate all historical normalization/alias writers and compatibility before choosing direct removal. Lock-held map work, not new filesystem canonicalization, is the observed concern |
| R6 ir/string_table.rs::StringTable::intern | from_instructions -> intern -> binary_wire::encode and string-table encoding; ordered strings Vec plus HashMap<String,usize> | one keyed probe per intern, two owned string allocations only for new string; expected-linear intern count excluding hashing/key bytes | JUSTIFIED WORK / ACCEPT for membership only | Existing representation is a concrete counterexample to a repository-wide absence of ordered membership reuse. Preserve first appearance/index identity; copies during encoding remain a separate memory question |
| R7 cbm/client/decoder.rs::grouped | provider collection/wire decode -> grouped -> table for header and each group; column Vec<String>, row JSON materialization | fixed required-field list searches C supplied columns; per group rebuilds header/row table then constructs per-row maps, dimensions G/C/R/JSON bytes | STRUCTURAL RISK / MONITOR | Required-column scan alone is fixed-factor linear, not quadratic growing membership. Repeated column construction/JSON cloning across G groups may matter; validate malformed/duplicate columns and empty groups before any reuse proposal |
| R8 proxy/src/filter_registry.rs::select_for_command | transform.rs tool result -> selection on first line, then full original text only on no match; HashMap filter values -> matching Vec | all enabled regexes tested, matching regexes searched again for longest span; negative fallback can repeat selection on larger text | STRUCTURAL RISK / MONITOR, regex/envelope cost not membership dedup | Most-specific-match and priority require inspecting eligible filters. Do not ban values iteration or change tie/override policy. Characterize filter count, pattern/input cost and fallback before removing temporary collection |
| R9 ir/binary_wire.rs::encode | IR persistence/transport encoding API -> StringTable::from_instructions -> strings().to_vec | instruction operands interned, then entire unique-string Vec cloned before writing; independent operand/unique-string/byte dimensions | STRUCTURAL RISK / MONITOR, allocation rather than growing membership | Default serializer callers and peak live bytes still need accounting. Do not infer an unnecessary copy or change durable bytes without tracing how the cloned table is consumed |

All ACCEPT entries are scoped local inspection dispositions, not blanket performance
approval of their subsystem. No allocation budget is inferred from logical probes.
Memory/owned keys, downstream proof work and external call multiplicity remain
independent dimensions. Where default callers are unknown, that gap remains open.

## Confirmed findings and next repair boundaries

Declaration D1/D2 repeat the ordered-storage-as-membership mistake independently
of the three repaired endpoint/publisher dimensions. The measured enclosing packet
has 2 characterization + 9 af174 passes; exact table and source hashes are in
AF_PERF_002_ENCLOSING_RESULTS_AND_DECLARATION_DESIGN.md. At n=256 each isolated
declaration series makes 32640 comparisons. This supports a separate small repair
design, not a generalized index rewrite or a shared collection abstraction.

Rank measured membership defects by available evidence, not unmeasured latency:
the repaired original dimensions have bounded fitness and isolated mutation proof;
D1/D2 are the remaining measured growing-membership defects in the inspected set.
NgRx full rebuilding is independently measured but is a different mechanism.
S1/T1/C1/F1/F2 are prioritized candidates for enclosing characterization because
they can scale with public input or semantic density. No equivalently severe
production latency claim is justified until those work counts exist.

Smallest independent proposals remain per-owner: declaration companions require
separate approval; local stable-output seen sets may suit trace/selector/form
accumulators if consequential counts justify them. Persistent state must cover all
writers/clone/removal/reset and exact Vec/set invariants. Never reuse endpoint
presence as declaration proof. No cache, generic registry, key normalization,
publication policy or synchronization change is proposed by the census.

Each consequential confirmed repair needs a real production-path growth budget,
enclosing-operation counts, exact semantic/ownership/currentness controls, finite
AST drift rule and an isolated real mutation RED/restored GREEN. The prior #174
mutation certifies its finite pattern, not all collection operations in this ledger.

## AST enumeration handoff and search-completeness evidence still missing

Tracked membership_census_af_perf_002.rs walks every Rust file in the named roots,
records hashes, AST candidates, raw call expressions, lexical function names and
manual-loop depth. It emits unknown macros and parse-error flags, retains inline
test code for later separation, and does not pretend to resolve trait dispatch,
aliases, helper ownership or iterator closure multiplicity. Every emitted site
starts UNRESOLVED/NOT INSPECTED. Detector output is discovery, not a semantic gate.
Raw call edges enable subsequent caller review but are not complete resolved chains.

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./verification/live-acceptance/issue-162/run-af-perf-002-membership-census.ps1
```

This runs 1 AST census + 2 unchanged enclosing characterizations + 9 af174 controls,
then affected all-feature warning-fatal Clippy, format/size/encoding. It preserves
source/harness hashes and census/logs in one timestamped packet. Authored UNRUN;
only PowerShell syntax and bounded file inspection were performed by the agent.

After results, reconcile independent file enumeration with AST files, classify
parser errors/inline test-only sites, map each candidate to a reviewed work unit,
trace helper/caller chains, and explicitly count unresolved/uninspected sites.
Source-path and behavioral-dimension coverage must both be recorded. Manual loops
and unsupported generated/macro/trait dispatch remain review work, never silent
absence. Adversarial Popper review follows complete first-pass enumeration and
must supply counterexamples or source-grounded acceptance. It has not run.

The source-wide family inventory is NOT complete. The broader filesystem/SQLite/
witness/memory/concurrency/lifecycle audit continues independently; this search
does not replace it. No production repair follows from this directive alone.

## AST packet verified; source classification remains open

See [AF_PERF_002_CENSUS_RESULTS.md](AF_PERF_002_CENSUS_RESULTS.md) for verified
12 tracked passes, lint/format/encoding and exact hash inspection. The frozen
525-file AST census contains 9763 syntax candidates and 40863 raw calls. Thirteen
source-bound syntax records are classified; 9750 remain unresolved. Existing 24
operation records and syntax record counts are different units. artifact.rs has
a parser error flag and stays REVIEW REQUIRED; Clippy passes the source, so no
compiler defect is inferred. Discovery and independent Popper review are incomplete.

New populated NgRx empty/duplicate/currentness controls are authored UNRUN under
separate focused handoff, with no production optimization or policy change.

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

## Two independent progress indicators; artifact production trace

[AF_PERF_002_OPERATION_PROGRESS.md](AF_PERF_002_OPERATION_PROGRESS.md) separates
syntax accounting from actual production-operation coverage and adds four named
artifact/query work units (A1-A4) to the 24 prior records. Parser syntax recovery
is accepted and no longer an active blocker. Artifact clone/projection/dependency
costs remain independent review obligations. New real public-query characterization
is authored UNRUN; no declaration repair or production optimization is approved.

## Actual artifact work return

AF_PERF_002_ARTIFACT_WORK_RESULTS_AND_A3_DESIGN.md records the completed authorized
artifact-work packet: three passes and clean affected gates. Independent C/D axes
measure direct filtering C*D; current warm generation avoids all instrumented
projection/resolution work. These are new production observations, not more syntax
classifications. The per-projection borrowed-bucket repair remains proposed only.
A1 publication-copy volume, A4 peer joins, extensions/ambiguity and A2 owner-count
growth remain unmeasured; no whole-system fitness or completed inventory claimed.
