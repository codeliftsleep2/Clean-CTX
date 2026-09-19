# IR Architecture Lock-Down Implementation Plan

**Status:** Approved target defaults. The bounded `DefClass`/`DefMethod`/
`Param`/`Return` slice is user-verified and checkpointed. The bounded
`DefField`/`FieldType` slice is user-verified as the Phase 2 checkpoint; later
operation families remain pending. The bounded shared-validation slice was
user-verified as the Phase 3 checkpoint on 2026-09-18. Phase 4A completed the
shared validation contracts for all current operations and was user-verified
on 2026-09-18. Phase 4B implemented stable-identity method-fact projection and
hierarchical schema revision 2 and was user-verified on 2026-09-18. Phase 4C's
stable-identity class-fact projection and schema revision 3 were user-verified.
Phase 4D typed pattern targeting was implemented and user-verified.

**Date:** 2026-09-17

**Basis:**
[`IR_PROJECTION_RECONNAISSANCE_REPORT.md`](IR_PROJECTION_RECONNAISSANCE_REPORT.md)

**Operation contracts:**
[`CORE_OP_CONTRACT_MATRIX.md`](CORE_OP_CONTRACT_MATRIX.md)

## Approval record

On 2026-09-17, the typed-identity, checked-projection failure, preserve-by-
default multiplicity, corrected-binary, and green-test-rollout defaults were
approved. They are now target contracts and must not be reopened without direct
contradictory source evidence requiring architectural review.

Source inspection found that the current binary format already emits version
`0x03` and decodes `0x01` through `0x03`. Physical version `0x04` is approved
for the later corrected/versioned binary format. The binary migration is not
part of the `Param`/`Return` slice.

## 1. Objective

Make semantic identity, ownership, cardinality, and preservation enforceable
throughout the production IR lifecycle so that stream position, consumer
assumptions, lossy keys, or incomplete codecs cannot silently change meaning.

The plan follows the repository enforcement order:

1. Rust type system and visibility
2. Existing compiler and Clippy mechanisms
3. Exhaustive validation at runtime boundaries
4. Focused tracked tests under `src/tests/**`
5. Property and compile-fail tests only where simpler mechanisms cannot enforce
   the contract

This plan deliberately does not introduce a generic fitness-function
framework, invariant registry, runtime plugin system, or repository-wide gate
abstraction.

## 2. Success definition

The architecture is locked down when all of the following are true:

- every identity-bearing fact has a typed target contract;
- projection resolves facts by stable identity, never an implicit cursor;
- unresolved or mismatched identities produce structured diagnostics;
- every repeatable operation declares cardinality, ordering, and merge rules;
- transformations preserve identity or return an explicit remapping;
- delta and replay preserve every occurrence required by the contract;
- each wire format truthfully declares and tests its preservation level;
- language producers derive facts only from the owning syntax region;
- production MCP paths use the checked validator and projection;
- adding a new `CoreOp` forces contract work through exhaustive Rust matches;
- adversarial and property tests protect the remaining behavioral invariants.

Passing isolated unit tests is not sufficient. Completion requires tracing the
real production lifecycle from producer through MCP exposure.

## 3. Target architecture and package choices

| Layer | Responsibility | Primary enforcement | Package choice |
|---|---|---|---|
| Language capture | Extract only facts owned by a syntax node | Node kinds and bounded source ranges | Existing `tree-sitter-*` crates |
| Semantic identity | Distinguish class, method, field, parameter, and pattern IDs | Newtypes, private fields, checked constructors | Rust `std`; existing `serde` |
| Canonical IR | Represent lossless language-neutral facts | Exhaustive typed variants | Existing IR types; existing `serde` |
| Validation | Reject invalid or unresolved semantic relationships | Exhaustive matches and structured errors | Existing `thiserror` |
| Transformations | Preserve or deliberately remap identity | Pass-specific contract and tests | Existing pass infrastructure |
| Projection | Resolve identities into consumer views | Two-pass indexing and checked `Result` | `HashMap`, `BTreeMap`, `Vec` from `std` |
| Delta/replay | Preserve repeatable facts and ordering contracts | Typed keys mapped to collections | `BTreeMap` and `Vec` from `std` |
| Wire formats | Preserve a declared semantic level | Versioned schema and compatibility decoder | Existing `serde`; manual binary codec |
| Consumers | Expose only validated state | Checked production call path | Existing MCP layer |
| Behavioral fitness | Exercise permutations and multiplicity | Table-driven tests and generated cases | Existing `proptest` |
| Compile-time fitness | Prove a few forbidden constructions remain impossible | Compile-fail fixtures | Proposed dev dependency: `trybuild = "1"` |
| Public API compatibility | Detect unintended Rust API breaks at release time | Release-only compatibility analysis | Optional `cargo-semver-checks` |

### 3.1 Dependencies not recommended

- **No ArchUnit-like framework:** Rust module privacy, types, exhaustive matches,
  and ordinary tests provide stronger local enforcement.
- **No `syn` for producer parsing:** it only parses Rust and would create a
  different ownership model from TypeScript, Java, and C#.
- **No `insta` for wire contracts:** explicit field-level fixtures make
  semantic loss easier to review than broad snapshots.
- **No `rstest`:** ordinary table-driven tests are sufficient and avoid adding
  macros to the architectural contract surface.
- **No `indexmap` initially:** deterministic `Vec` plus `BTreeMap` composition
  is sufficient until a concrete requirement proves otherwise.

## 4. Core architectural contracts

The authoritative per-operation detail lives in [`CORE_OP_CONTRACT_MATRIX.md`](CORE_OP_CONTRACT_MATRIX.md); its 23 rows define
identity ownership, cardinality, ordering, delta identity, projection behavior,
and wire preservation without creating a runtime registry.

The implementation-level rules are:

- use private typed IDs and checked boundary conversions;
- make validator matches exhaustive and diagnostics structured;
- resolve projection targets in two passes by stable identity;
- preserve every repeatable occurrence unless a matrix row authorizes a
  reducer;
- require identity-changing transformations to return a complete remapping;
- use occurrence-aware delta keys and collections;
- version formats that claim semantic round trip and reject unrepresentable
  operations;
- derive declaration-owned facts from Tree-sitter nodes or bounded declaration
  heads.

The current hierarchy's singular side-effect and execution-context fields,
synthetic-owner fallback, pattern prefix inference, and current-scope
attribution do not satisfy the approved target. Their required replacements
are specified in the matrix.

## 5. Fitness-test portfolio

All test artifacts that satisfy the gate must be tracked under `src/tests/**`
and connected through the existing `#[path = "..."]` convention.

Recommended focused modules:

- `src/tests/ir/hierarchical_identity.rs`
- `src/tests/ir/hierarchical_cardinality.rs`
- `src/tests/ir/validator_identity.rs`
- `src/tests/ir/transformation_preservation.rs`
- `src/tests/ir/delta_multiplicity.rs`
- `src/tests/ir/wire_contracts.rs`
- language-specific ownership tests in the existing language test areas
- optional compile-fail cases below `src/tests/ui/`

Names may be adjusted to fit existing module ownership. Do not create a central
test registry merely to collect them.

### 5.1 Deterministic adversarial cases

Every identity-bearing operation should be exercised in these shapes where
applicable:

1. fact after definition;
2. fact before definition;
3. fact after leaving its lexical scope;
4. two owners with interleaved facts;
5. repeated facts with equal targets and different values;
6. duplicated identical facts when multiset behavior matters;
7. unresolved target;
8. target of the wrong semantic kind;
9. transformation reorder with identity preserved;
10. encode/decode through every supported format.

### 5.2 Property tests

Use the existing `proptest` dependency for properties that benefit from input
generation and shrinking:

- shuffle independent facts and preserve projected semantics;
- interleave two valid class/method streams without cross-attribution;
- generate repeated facts and preserve multiplicity through delta/replay;
- encode/decode arbitrary supported operations;
- verify identity-preserving transformations over valid streams.

Generators should create valid semantic graphs first, then derive instruction
orders. Arbitrary strings alone will mostly test parser rejection rather than
architectural preservation.

### 5.3 Compile-fail tests

Add `trybuild` only after typed IDs and visibility make meaningful misuse
impossible. Limit it to high-value boundaries such as:

- constructing a method-scoped fact with a `ClassId`;
- bypassing validation to invoke checked projection;
- mutating an identity-bearing field outside its owning module.

Do not use compile-fail tests for ordinary wrong-type expressions already
obvious from local unit tests and compiler diagnostics.

## 6. Phased implementation

Each phase is a reviewable migration slice. Existing unrelated behavior remains
the reference unless a change is explicitly approved.

### Phase 0: Finalize operation contracts

Deliverables:

- review and finalize the drafted `CoreOp` contract matrix;
- approve semantic-family members and language mappings not covered by the
  approved preserve-by-default rule;
- record physical version `0x04` for the later corrected binary phase;
- classify existing contract and defect-pinning tests.

Exit criteria:

- every operation has an owner, target, cardinality, delta, projection, and
  wire decision;
- all cross-cutting decisions are recorded before code changes depend on them.

### Phase 1: Establish focused failing contracts

Add the smallest deterministic regressions for the observed failures:

- stable-ID projection after scope movement;
- pre-definition and interleaved facts;
- unresolved references;
- repeated side effects, execution contexts, flags, and patterns;
- delta key collisions;
- binary identity loss;
- descendant-text modifier leakage.

Exit criteria:

- each test names one approved invariant;
- defect-pinning expectations are clearly identified;
- no scratch artifact outside `src/tests/**` is counted as verification.

Gate strategy must follow the approved rollout policy. Do not merge a knowingly
red default branch merely to demonstrate RED.

### Phase 2: Introduce typed identities

Deliverables:

- typed identity newtypes and target enum;
- checked construction at compiler and decoder boundaries;
- compatibility adapters for existing external shapes;
- narrowly scoped compile-fail cases if approved.

Exit criteria:

- identity kinds cannot be mixed by ordinary internal callers;
- serialization behavior is unchanged unless separately approved;
- no new generic identity framework is introduced.

### Phase 3: Complete validation

Deliverables:

- exhaustive `CoreOp` validation match;
- definition, owner, target-kind, duplicate-ID, and cardinality checks;
- stable structured diagnostic codes;
- validation integrated into the real compilation path.

Exit criteria:

- invalid identity graphs cannot reach projection through production entry
  points;
- adding a new `CoreOp` requires an explicit validator decision.

Implementation status: Phase 4A completes these deliverables for all 21
current operations through the shared typed validation authority and was
user-verified on 2026-09-18.

### Phase 4: Replace hierarchical projection

Deliverables:

- two-pass definition indexing and fact resolution;
- checked projection result;
- removal of current-scope attribution;
- removal of synthetic-owner and prefix-inference recovery;
- migration of all production callers.

Exit criteria:

- fact ordering does not change attribution;
- unresolved targets fail diagnostically;
- production MCP handlers use the checked path;
- legacy projection paths have no production callers.

This is the first phase that closes the principal demonstrated violation.

Implementation status: Phase 4B moves method-scoped `Flags`, `Body`,
`ControlFlow`, `DataFlow`, `SideEffect`, and `ExecutionContext` to the typed
`MethodId` index and preserves occurrences. Hierarchical output emits `"hs":
2`; unmarked scalar/flat input remains readable and unknown revisions fail.
The projection cursor is removed. Phase 4C moves class-scoped `ClassFlags`,
`Extends`, `Implements`, and `Injects` to the typed `ClassId` index. Revision 3
preserves repeated class-flag and injection payloads; strict revision 2 and
unmarked legacy documents remain readable through decode-only adapters. Typed
pattern targeting now uses a shared schema-derived `PatternTarget`, attaches
after definition indexing, and retains existing serialized shapes. Interface
representation remains a later Phase 4 slice. The user-reported Phase 4B and
4C gates were green on 2026-09-18; Phase 4D was also green on that date.

### Phase 5: Correct producer ownership

Implementation status: Rust, TypeScript, and Java modifier extraction now uses
a shared lexical declaration-head boundary. C# retains its established bounded
head extraction. Equivalent tracked adversarial fixtures cover all four
languages; the user-run gate was green on 2026-09-18.

- declaration-head or syntax-node extraction for Rust, TypeScript, and Java;
- confirmation that C# follows the same semantic contract;
- equivalent adversarial fixtures for every supported language.
- descendant source text cannot define a parent's declaration modifiers;
- language differences are explicit semantic differences, not extraction
  accidents.

### Phase 6: Separate semantic families and cardinalities
Phase 6A typed modifiers were user-verified on 2026-09-18. Approved Phase 6B
makes canonical `ControlSummary` ordered and duplicate-preserving while the
distinct compact LLM projection uses `ctl:`; hierarchical revision 5 and
additive `0x03` opcode 24 carry it. Physical `0x04` remains Phase 8.

Remaining families migrate one at a time:
1. patterns;
2. side effects;
3. execution contexts.
For each slice, migrate producer, IR representation, validator,
transformation, projection, wire format, consumer, and tests before beginning
the next family.

Deferred follow-up: after canonical repairs, audit LLM token efficiency where
whole bodies are required; presentation changes must preserve complete meaning.

Exit criteria:

- consumers cannot confuse unrelated semantic families;
- every destination field represents its declared cardinality;
- reducers are named and directly tested.

### Phase 7: Repair delta and replay

Deliverables:

- typed delta keys;
- occurrence-aware index collections;
- per-operation equality and ordering behavior;
- deterministic and property-based replay coverage.

Exit criteria:

- repeated operations never disappear through map overwrite;
- delta/replay algebraic properties hold for all supported operations;
- old assumptions about upstream merging are removed or made real contracts.

### Phase 8: Version and tighten wire formats

Deliverables:

- truthful preservation classification per format;
- approved version change;
- complete identity operand encoding;
- compatibility decoder if required;
- semantic rather than opcode-only round-trip assertions.

Exit criteria:

- no supported decoder fabricates empty semantic IDs;
- format documentation matches observed, tested behavior;
- lossy formats reject or explicitly document unsupported semantics.

### Phase 9: Production integration audit

Trace the real lifecycle for each migrated semantic family:

```text
producer
  -> production compiler
  -> validation
  -> transformations
  -> result boundary
  -> persistent workspace owner
  -> lifecycle updates and deletion
  -> consumer
  -> MCP response
```

Exit criteria:

- default production entry points invoke the new boundaries;
- recompilation, deletion, reset, and workspace replacement are covered;
- externally visible behavior is tested where the change can reach it;
- no obsolete production path remains.

### Phase 10: Documentation and final gate

Deliverables:

- updated `docs/ARCHITECTURAL_INVARIANTS.md`;
- final operation contract matrix;
- removal or reclassification of stale defect-pinning comments and tests;
- final file-size and encoding review;
- complete user-run verification gate from `docs/agent/verification.md`.

Exit criteria:

- implementation, tests, and documentation describe the same architecture;
- all modified files satisfy the active-file ceiling;
- the user reports the complete verification results;
- no unrun gate is reported as passing.

## 7. Approval gates

### Gate A: Semantic schema — partially approved

Approved: internal `ClassId`, `MethodId`, `FieldId`, and `ParameterId` with
unchanged initial serialized string shapes.

Remaining review: semantic-family enum members and language mappings. The
matrix's per-operation cardinalities are normative; preserve every occurrence
unless its row explicitly declares a singular contract.

Tradeoff: more explicit conversion code in exchange for compiler-enforced
boundaries.

### Gate B: Projection failure policy — approved

Unresolved, duplicate, and kind-mismatched identities fail checked projection.
Production MCP maps the structured failure to its existing error contract and
does not return a partial hierarchy as success.

Tradeoff: callers receive an explicit failure where current behavior may return
incomplete data.

### Gate C: Multiplicity default — approved

Preserve every occurrence by default. A per-operation contract must explicitly
authorize any deduplication, union, replacement, or reduction. The matrix's
approved singular cardinalities fail on duplicates rather than replacing them.

Tradeoff: potentially larger IR and deltas in exchange for correctness and
reversible transformations.

### Gate D: Binary compatibility — approved

Approved: a corrected format is the target, and no V1 compatibility reader is
added without evidence that persisted V1 artifacts are supported input.

Versions `0x01`, `0x02`, and `0x03` already exist. Physical version `0x04` is
the approved identifier for the corrected/versioned format. That later phase
must not add a V1 compatibility reader without evidence that persisted V1
artifacts are supported input.

Tradeoff: compatibility code increases maintenance cost; an immediate break
requires coordinated consumers.

### Gate E: Test rollout — approved

Land each failing architectural contract test with the repair that makes it
green. Do not knowingly merge a red default branch merely to preserve RED
history.

Tradeoff: this keeps the default branch green but does not preserve a standalone
failing commit in shared history.

### Gate F: Hierarchical schema revision — approved

Approved on 2026-09-18: hierarchical output emits `"hs": 2`; method `fl` is
an ordered array of operation payloads and `se`/`ec` are ordered occurrence
arrays. Unmarked flat/scalar input remains readable; unknown revisions fail.
The binary contract and future physical version `0x04` are unchanged.

### Gate G: Hierarchical class-fact schema revision — approved

Approved on 2026-09-18: hierarchical output advances to `"hs": 3` so class
`fl` and `ij` store ordered arrays of complete operation payloads. Revision 2
remains readable with its strict method-fact shapes and flat class containers;
unmarked legacy documents retain their established compatibility adapter.
Unknown revisions fail. The LLM text schema and binary contract are unchanged.

## 8. Review and completion checklist

### Contract review

- [ ] Every `CoreOp` appears exactly once in the contract matrix.
- [ ] Every identity-bearing operation declares its target kind.
- [ ] Every repeatable operation declares cardinality and ordering.
- [ ] Every reducer has a semantic name and direct tests.
- [ ] Every format declares its preservation level.

### Implementation review

- [ ] No projection handler uses a current-scope cursor as semantic authority.
- [ ] No unresolved fact is silently discarded or assigned a synthetic owner.
- [ ] No delta key maps multiple permitted occurrences to one value slot.
- [ ] No decoder fabricates an identity required by the canonical contract.
- [ ] No producer scans descendant text to define a declaration-owned fact.
- [ ] No transformation changes identity without a declared mapping.

### Test review

- [ ] Pre-definition, post-scope, and interleaved facts are covered.
- [ ] Wrong-kind and unresolved targets are covered.
- [ ] Repeated and duplicated facts are covered.
- [ ] Cross-language ownership cases are equivalent.
- [ ] Property tests exercise permutation, delta/replay, and round trip.
- [ ] Compile-fail tests are limited to meaningful public/module boundaries.
- [ ] Every required test is tracked under `src/tests/**`.

### Production review

- [ ] The checked path is used by default compilation.
- [ ] Projected state survives the result and ownership boundaries.
- [ ] Workspace recompilation, deletion, and reset remain coherent.
- [ ] Real consumers use the migrated fields.
- [ ] MCP responses expose the intended behavior.
- [ ] Relevant live scenarios are handed to the user when field verification is
      required.

## 9. First implementation slice

The approved bounded slice implements only these contracts:

1. internal `ClassId`, `MethodId`, and `ParameterId` types at the projection
   boundary, with unchanged serialized strings;
2. complete first-slice definition indexing before reference validation;
3. stable-ID attachment of ordered `Param` facts and singular `Return` facts;
4. structured duplicate, unresolved, and wrong-kind failures with stable
   diagnostic codes;
5. checked production MCP projection mapped through the existing IR error
   response contract;
6. tracked interleaving, pre-definition, ordering, duplicate, unresolved,
   wrong-kind, and MCP mapping regressions.

That first slice deferred method metadata, patterns, delta/replay, and binary
`0x04`; Phase 4B now addresses the method-metadata portion. The first slice was
user-verified and pushed before the second bounded slice began.

## 10. Second bounded implementation slice

The approved field slice implements only these contracts:

1. internal `FieldId` at the projection boundary, with unchanged serialized
   strings;
2. globally unique, non-empty `DefField` identities owned by a valid
   `ClassId`;
3. optional-singular `FieldType` facts targeting a valid `FieldId`;
4. stable-ID field placement and type attachment independent of legal
   instruction ordering;
5. structured duplicate, unresolved, empty, and wrong-kind failures;
6. removal of synthetic orphan-field recovery from canonical projection;
7. production MCP mapping through the existing checked projection error path;
8. tracked ordering, identity, failure, serialized-shape, and MCP regressions.

The slice does not remove the hierarchical schema's `synthetic` property or
change hierarchical decoding because interfaces and existing documents remain
outside its approved scope. User-run verification was reported green on
2026-09-18 before this checkpoint was committed.

## 11. Third bounded implementation slice

At the time it landed, the approved Phase 3 checkpoint centralized validation
for the six then-normative operations without approving the remaining draft
operation rows:

1. `ClassId`, `MethodId`, `FieldId`, `ParameterId`, `IdentityKind`, and the
   structured identity errors move to one shared IR identity module;
2. `DefaultValidator` and hierarchical projection consume the same typed
   identity graph and diagnostics;
3. all definition indexes are collected before owner and target validation;
4. the production `ValidationPass` rejects invalid approved identity graphs;
5. every current `CoreOp` has an explicit validator match arm, so a new variant
   requires a compiler-visible validation decision;
6. existing E001-E011 behavior is preserved for previously validated paths;
7. unapproved semantic-family, vocabulary, alias-scope, pattern-schema, and
   cardinality rules remain explicitly deferred.

The checkpoint established the single validation authority. Its gates were
green before commit; Phase 4A subsequently superseded its row-level deferrals.

## 12. Approved Phase 4A implementation slice

Phase 4A makes every current operation row normative and completes the
validation prerequisite without changing hierarchical projection shapes:

1. the shared two-pass validator covers every definition, owner, target kind,
   duplicate definition, and optional-singular cardinality;
2. `DefInterface` and import aliases retain their serialized string
   representation while gaining duplicate and empty-identity checks;
3. `TypeAlias` retains every ordered occurrence because production uses the
   operation for both configured type substitutions and repeated `Φ` metadata;
4. flags reject empty payloads without reducing repeated occurrences;
5. body spans enforce both-or-neither presence and non-reversed ranges;
6. data flow, control flow, side effects, and execution contexts enforce their
   declared vocabularies;
7. patterns use their declared schema, resolved class and method identities,
   and verified method ownership rather than ID-prefix inference;
8. the production `ValidationPass` and checked hierarchical projection consume
   the same authority;
9. legacy validation codes E001-E011 remain stable for their existing operation
   families while projection retains structured error classifications.

The `TypeAlias` correction followed production evidence of repeated metadata;
producer deduplication would discard distinct payloads. Phase 4A did not alter
projection, producers, delta/replay, or binary `0x04`; its gate was green.

## 13. Verification ownership

Agents must not initiate the repository's long-running build, test, Clippy,
format, audit, binary, or server commands. During implementation, the agent may
perform only fast bounded inspections and must hand the exact applicable
commands from `docs/agent/verification.md` to the user.

Results must be reported by category:

- tracked Rust tests actually run by the user;
- compile, Clippy, format, file-size, and encoding gates;
- optional untracked live harness output;
- production or field scenarios.

An untracked harness never substitutes for a tracked regression test, and no
unrun gate may be reported as passing.
