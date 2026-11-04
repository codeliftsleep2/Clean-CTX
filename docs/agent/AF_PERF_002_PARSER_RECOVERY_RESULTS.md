# AF-PERF-002: parser boundary recovered, operation review remains open

Agent: Agent MaxHeadRoom
Status: diagnostic and affected gates verified GREEN; scoped independent manual
syntax recovery accepted by Agent Popper. Inventory discovery remains incomplete.

## Verified operator evidence

parser-boundary-20261104-091946: one tracked diagnostic PASS, zero failed/ignored,
0.02s. All-feature/all-target warning-fatal Clippy is clean (22.28s); format logs
empty; encoding passes 2041 text files. All 1282 receipt file hashes match current
files at inspection. artifact-parser-boundary.json binds the exact artifact bytes,
error fragments and recovered function/impl intervals. This is parser inspection
evidence, not a work-count budget, optimization RED or whole-source acceptance.

No production source or test code changed after that verification in this return.
Only offline reconciliation metadata/script and documentation changed. No agent
test/build/lint/runtime was started.

## Exact error regions

All three ERROR nodes lie inside SemanticCompilationArtifact::into_parts, recovered
as lines 121..172. Byte intervals are half-open against the source SHA-256 recorded
in the diagnostic; line intervals are human-readable inclusive locations.

| Start byte | End byte | Source lines | Error fragment |
| ---: | ---: | --- | --- |
| 4728 | 4738 | 129 | `let Self {` |
| 4794 | 5020 | 131..138 | `typescript_authority` through `dotnet_testing_enabled` field bindings |
| 5078 | 5089 | 140..141 | trailing comma and closing pattern brace |

The complete source region is cfg-qualified Self field destructuring. Manual
inspection finds only bindings/punctuation in these error fragments, with no
concealed calls, loops, joins or collection construction. The later ArtifactIdentity
and ProducerGenerationFingerprint destructuring, field drops and returned values
remain inside the recovered into_parts boundary and were read separately.
This localizes the parser limitation; it does not claim that removing one cfg
attribute would repair the grammar, because that experiment was not performed.
Rust/Clippy already accepts the real production source, which is unchanged.

All 23 source fn headers match recovered function intervals. Twenty-two belong
to SemanticCompilationArtifact; ProducerGenerationFingerprint::current is the
other owner. Diagnostic slices and error fragments match exact source bytes.
Matching headers is boundary evidence, not proof that all function bodies/call
graphs have been semantically classified. Artifact clone/projection/publication
costs remain review work.

## Real callers and manual coverage supplement

from_compiler is invoked by semantic_compilation/compile.rs. into_parts is reached
from semantic dependency preparation, edit, compress, delta, provide evaluation
and query handlers. published_semantics is reached from hydration publication and
entity querying. It clones semantic edges, dependencies and source hash; this cost
is separate from the parser's destructuring limitation and remains unmeasured.
Do not use successful syntax recovery to grant artifact freshness, publication
authority or acceptable cloning costs.

The offline reconciler now accepts a source-bound parser supplement. It validates
all diagnostic fragment bytes, preserves original candidate IDs/parser flags,
and attaches recovered impl/function intervals to 13 original artifact candidate
records that fall inside the recovered functions. Module/type/attribute records
outside those intervals stay explicitly ungrouped. Candidate accountability and
blocking classification are unchanged.

Evidence: target/af-perf-002/membership-accounting-003. All 9763 original records
still reconcile: 6 production anchors, 1 bounded anchor, 6 duplicate syntax links,
3226 macro review records and 6524 review-required records. 9750 remain blocking;
96 supplemental all-callback obligations remain outside the original denominator.
src/lib.rs drift remains explicit. Whole operation coverage is not accepted.

## Independent Agent Popper finding

Popper independently verified all three error fragments and all 23 recovered
function slices against source. Scoped manual syntax recovery is acceptable,
provided the original parser flag/error evidence is retained. Popper found no
erroneous merge among the reconciler's seven exact reviewed anchors: exact
source/path/expression/start linkage avoids the prior bare-name collision there.

This is scoped methodological acceptance, not final inventory acceptance. Popper
retains concrete blockers: missing impl/function intervals elsewhere, incomplete
iterator/delegated callback coverage beyond all, unresolved production reachability,
macros and source drift. Duplicate syntax links account for records; they do not
create additional independent inspections.

## Next investigation boundary

Continue owner/interval-aware candidate grouping and verified inline test/support
scope classification, preserving every original record and separately supplementing
new raw-call sinks/source drift. Evidence-based exclusions must remain challengeable;
no function-name, macro-name, fixed-method vocabulary or whole-file safety shortcut.
The narrow parser-error syntax coverage is recovered manually; artifact operation
classification remains REVIEW REQUIRED. Full membership inventory and broader
AF-PERF-002 audit remain incomplete. No new operator run is required solely for
these documentation and offline metadata changes.

## Owner reporting correction

The accepted parser syntax boundary is closed; its original flags remain historical
evidence, not an active blocker. Artifact operations remain separately reviewable.
See AF_PERF_002_OPERATION_PROGRESS.md for independent syntax-accounting and
production-operation indicators, four new artifact/public-caller traces and an
unrun real-query characterization. No raw syntax progress is equated to behavioral
coverage. No production optimization or declaration representation change followed.
