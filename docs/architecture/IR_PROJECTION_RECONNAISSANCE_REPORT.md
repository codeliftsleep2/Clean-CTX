# IR Projection Reconnaissance Report

**Status:** Historical reconnaissance baseline. Its findings describe the
pre-migration source state and are not a current status dashboard. Current
contracts and certification are recorded in
[`IR_ARCHITECTURE_CERTIFICATION.md`](IR_ARCHITECTURE_CERTIFICATION.md).

**Date:** 2026-09-17

**Companion plan:**
[`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md)

**Operation contracts:**
[`CORE_OP_CONTRACT_MATRIX.md`](CORE_OP_CONTRACT_MATRIX.md)

**Approval update:** On 2026-09-17, the report's recommended typed-identity,
checked-projection failure, preserve-by-default multiplicity, corrected-binary,
and green-test-rollout defaults were approved as target contracts. The
companion plan and operation matrix record the binding details and the one
binary version-number contradiction discovered during source inspection.

**Implementation update:** The bounded `DefClass`/`DefMethod`/`Param`/`Return`
projection repair was user-verified and checkpointed. The bounded
`DefField`/`FieldType` repair was user-verified as the Phase 2 checkpoint on
2026-09-18. The shared validation authority for all six normative operations is
implemented and user-verified as the bounded Phase 3 checkpoint on 2026-09-18.
Phase 4A implemented and user-verified the remaining shared operation-validation
contracts on 2026-09-18. Phase 4B implements identity-driven projection for
method `Flags`, `Body`, `ControlFlow`, `DataFlow`, `SideEffect`, and
`ExecutionContext`, plus occurrence-preserving hierarchical schema revision 2;
user-run verification was green on 2026-09-18. Phase 4C implemented and user-
verified typed class-fact attribution plus hierarchical schema revision 3.
Phase 4D implemented shared schema-derived typed pattern targeting and was
user-verified on 2026-09-18. The projection cursor and ID-prefix inference are removed.
Phase 9 P9-01 implemented explicit canonical interface ownership and a
distinct hierarchy/LLM projection; it was user-verified. User-run verification proved that
`TypeAlias` carries repeated `Φ` metadata; its ordered-many correction was
approved, implemented, and verified on 2026-09-18.
Phase 6A separates declaration modifiers into typed method/class operations
across producers, validation, patterns, projection, rendering, and wire
formats; it was user-verified on 2026-09-18. Phase 6B implements typed control
summaries while keeping the LLM-facing `ctl:` projection independently compact;
it was user-verified and checkpointed as `3360512` on 2026-09-19.
Phase 6C implements typed method-pattern facts with a distinct compact `pf:`
projection; it was user-verified and checkpointed as `6d9b621` on 2026-09-19.
Phase 6D typed side-effect values were user-verified on 2026-09-19. Phase 6E
typed the five approved method execution contexts while preserving wire strings
and the compact `ec:` projection and was user-verified on 2026-09-19. Phase 7
implemented the corrected `dv: 2` positional delta protocol and was user-
verified on 2026-09-19. Phase 8A inventories physical `0x03` and defines the
complete semantic `0x04` contract. Phase 8B implements the codec and tracked
contract coverage and was user-verified on 2026-09-20. Phase 8C connects
canonical `0x04` baselines and corrected `dv: 2` history to buffered SQLite
persistence and registered MCP replay; it was user-verified on 2026-09-20.

**Production-integration status:** Phase 9 traced every registered operation
and migrated family through the real production lifecycle, repaired findings
P9-01 through P9-27, and was user-verified on 2026-09-21. The original findings
below remain preserved as reconnaissance history.

## 1. Executive summary

At reconnaissance time, the repository had sound local protections for
semantic identity but did not enforce them consistently through the lifecycle.
Phases 1-9 repaired and production-traced those gaps; this section retains the
original problem statement rather than redefining the baseline retrospectively.

The original highest-risk defect was positional attribution in the
hierarchical projection. The bounded method-fact families are migrated in
Phase 4B and the projection cursor is removed. Phase 4C addressed class-fact
attribution and multiplicity. Phase 4D addressed pattern targeting. Phase 9
P9-01 repaired canonical and projected interface ownership through production
MCP persistence and reload.

Related risks reinforce the same failure mode:

- unresolved identities can be dropped or replaced with synthetic owners;
- singular destination fields accept facts that may be emitted repeatedly;
- delta indexing collapses multiple operations onto one string key;
- the binary wire format omits identity operands for structural operations;
- several language producers infer declaration modifiers from descendant text;
- tests sometimes protect implementation shape or acknowledged lossy behavior
  instead of the intended semantic contract.

The recommended response is not a generic fitness-function framework. The
architecture should be enforced in layers: Rust types first, exhaustive
validation second, checked identity-driven projection third, and focused
contract/property tests at the remaining behavioral boundaries.

## 2. Scope and method

This reconnaissance covered the following lifecycle:

```text
language capture
  -> canonical IR production
  -> validation
  -> transformation
  -> hierarchical projection
  -> delta and replay
  -> wire encoding and decoding
  -> MCP consumption
```

The investigation used source inspection and existing tracked tests. It did
not run builds, tests, Clippy, format checks, binaries, or servers. Findings
are therefore code-level evidence, not verification results.

The governing architectural principles are:

1. Identity-bearing facts are attributed by stable semantic identity.
2. Producers own facts; consumers do not reinterpret producer contracts.
3. Cardinality, ordering, and merge behavior are declared per operation.
4. Transformations state what semantic information they preserve.
5. Projection does not silently discard canonical semantic information.
6. Wire-format round-trip claims say exactly what survives.
7. Tests enforce intended contracts rather than accidental implementation
   shape.

## 3. Original architecture map (reconnaissance baseline)

This map records the architecture observed before the migration checkpoints.
It is retained as investigation evidence, not as a current integration claim.

### 3.1 Producers

Language-specific capture code creates canonical `CoreOp` instructions. The
producer boundary is the correct place to determine which syntax node owns a
fact and to assign stable class, method, field, and parameter identities.

Relevant implementation areas include:

- `src/languages/rust.rs`
- `src/languages/typescript.rs`
- `src/languages/java.rs`
- `src/languages/csharp.rs`
- the language dispatch and capture integration in the IR compiler

### 3.2 Canonical IR

The canonical instruction stream is the semantic interchange boundary. It
contains definitions and facts such as flags, body information, control flow,
data flow, side effects, execution contexts, and patterns.

Some operations carry stable target identifiers, but the type system does not
consistently distinguish identity kinds. Multiple semantic families also share
string-based representations, allowing consumers to flatten or reinterpret
them.

### 3.3 Validator

`src/ir/validator.rs` validates a useful subset of references, including
several method-scoped operations and calls. It is not yet an exhaustive
semantic boundary. Important definition ownership, field, pattern, body,
class-flag, and duplicate-identity cases are incomplete or absent.

### 3.4 Hierarchical projection

`src/ir/hierarchical/encode.rs` converts the canonical stream into the
hierarchical representation. Some operation handlers consult positional state
such as the current class or method. This makes stream position an undeclared
source of semantic authority.

### 3.5 Delta and replay

The original delta engine indexed instructions with one operation per string
key, and replay assumed uniqueness where repetition is valid. Phase 7 retains
that shape only for compatibility input. Production emits versioned positional
sequence edits with typed identity and occurrence ordinals; corrected replay
checks expected tuples and applies transactionally without discarding order.

### 3.6 Wire formats

Physical `0x03` omits identity operands for eight operations and raw
`CompiledIR.file_id`; decoding substitutes empty IDs and the literal `"bin"`.
Phase 8A records the exact losses, malformed-input gaps, absent compatibility
evidence, production persistence gap, and normative physical `0x04` layout in
[`BINARY_V04_CONTRACT.md`](BINARY_V04_CONTRACT.md).

Phase 8C repairs that persistence gap: canonical production writers persist
non-empty `0x04` bytes under durable file identity, applied corrected deltas
are retained and replayed in order, and replacement/reset/deletion own their
state transitions. The non-canonical Angular template renderer no longer
creates an empty canonical-IR row.

### 3.7 Production consumption

MCP tool handlers call hierarchical projection in the production path. A
projection defect can therefore alter externally observed results even when
the canonical compiler output is correct.

## 4. Principal findings

### F-01: Projection still uses positional attribution

The original affected family included parameters, returns, method and class
flags, inheritance, implementations, injections, body facts, control flow,
data flow, side effects, and execution context. The implemented slices now
resolve those facts through stable typed identity. Phase 4B removes the
obsolete projection cursor, Phase 4C completes the class-fact subset, and
Phase 4D implements the separate typed-schema pattern repair.

Consequences:

- a pre-definition fact can be lost;
- an interleaved fact can attach to the wrong class or method;
- a transformation that preserves IDs but changes order can change meaning;
- the projection can appear successful while returning incorrect data.

Required boundary: projection must index definitions by stable ID and resolve
facts through that index.

### F-02: Unresolved references are not uniformly diagnosable

The validator covers several references but does not make every identity-
bearing `CoreOp` participate in an exhaustive contract. Projection can also
drop information or synthesize owners rather than returning a structured
failure.

Required boundary: every target-bearing operation must have an exhaustive
validation rule, and checked projection must report unresolved or mismatched
targets.

### F-03: Cardinality is implicit

Some hierarchical fields use `Option<String>` although multiple producers or
repeated facts can supply values. In other places, repeated flags are combined
without an operation-level declaration of whether the collection is ordered,
set-like, multiset-like, or reducible.

Required boundary: every operation declares cardinality, ordering, and merge
semantics. `Option<T>` is used only when singularity is proven by the producer
contract and validator.

### F-04: Semantic families are flattened

The flags family can contain declaration modifiers, control summaries, and
inferred pattern classifications. Their meanings, owners, and merge behavior
are different, but string-based consumers can treat them as one category.

Required boundary: represent distinct semantic families with distinct Rust
types or enum variants.

Phase 6A resolves declaration modifiers, Phase 6B resolves control summaries,
Phase 6C resolves method-pattern facts, and Phase 6D resolves side effects.
Phase 6E types the five method execution contexts. It removes TypeScript's
invalid `CTX(class_id, "di_scope")` path because injectable status is
class/metaclass DI metadata, not because the information is unnecessary.
`Injects` remains the dependencies consumed by a class; replacement DI
metadata remains a separate architectural decision.

### F-05: Delta keys lose multiplicity

At the original baseline, the delta index used `BTreeMap<String, CoreOp>` and
collapsed repeated same-target facts. Phase 7 isolates that representation as
legacy compatibility, adds corrected protocol `dv: 2`, and makes position,
expected tuple, typed semantic identity, and occurrence ordinal authoritative.
Ambiguous legacy modifications fail structurally instead of being reinterpreted.

### F-06: Wire-format claims exceed observed preservation

Repository documentation describes supported wire formats as preserving the
canonical stream. The binary format deliberately omits identifiers for several
structural operations and recreates empty values during decoding. Hierarchical
round-trip tests also tolerate normalization and semantic loss.

Required boundary: each format declares semantic, structural, lossy, or
unsupported behavior. Tests must verify the declared level rather than only
the opcode sequence.

Phase 8A classifies physical `0x03` as lossy and fixes `0x04` as a semantic,
exact-`CompiledIR` format. Phase 8B replaces opcode-only and selective-field
test authority with complete equality and structured malformed-input coverage.

### F-07: Declaration ownership differs by language

C# modifier extraction was already bounded to the declaration head. Phase 5
routes Rust, TypeScript, and Java modifier extraction through a shared lexical
head boundary. Equivalent adversarial fixtures cover all four languages;
the user-run gate was green on 2026-09-18.

Implemented boundary: declaration modifiers derive from a bounded declaration-
head span for every supported language, never descendant source text.

### F-08: Some tests protect defects or implementation shape

The suite contains valuable identity and hierarchical regression tests, but
some delta tests explicitly accept overwriting because another stage was
assumed to merge facts. Some hierarchical tests accept fewer decoded
instructions and document lost semantics as expected behavior.

Required boundary: classify tests as contract, migration, compatibility, or
implementation-shape evidence. A test that captures a known defect must not be
treated as architectural authority.

## 5. Original reconnaissance invariant assessment (baseline)

This table records the source state observed at the start of the migration. It
is not a current-state status dashboard. The implementation updates at the top
of this report and the phase records in the companion plan supersede baseline
findings that have since been repaired. The Phase 9 audit and final
certification document supersede this table for current-state assessment.

Classification meanings:

- **Green:** enforced at the relevant production boundary.
- **Partial:** meaningful enforcement exists but does not cover the complete
  lifecycle.
- **Broad gap:** the contract is absent, contradicted, or routinely bypassed.
- **Unknown:** insufficient evidence.

| # | Architectural invariant | Status | Evidence summary |
|---:|---|---|---|
| 1 | Identity-bearing facts are authoritative | Partial | Stable IDs exist, but downstream consumers do not always honor them. |
| 2 | Target attribution uses stable IDs | Broad gap | Hierarchical projection still uses positional current-scope state. |
| 3 | Repeatable operations declare merge semantics | Broad gap | Behavior is spread across producers, projection, and delta code. |
| 4 | Singular fields have a singularity proof | Broad gap | Some repeated semantic facts project into `Option<T>`. |
| 5 | Producers own facts | Partial | C# is bounded; other language paths inspect broader text. |
| 6 | Transformations declare preservation | Partial | Some focused regressions exist; no complete per-pass contract. |
| 7 | Pre- and post-scope facts are handled explicitly | Broad gap | Correctness depends on instruction order. |
| 8 | Unresolved references are diagnosable | Broad gap | Coverage is incomplete and projection can silently recover or drop. |
| 9 | Semantic families remain distinct | Broad gap | Heterogeneous meanings share strings and flag channels. |
| 10 | Projection preserves semantic information | Broad gap | Identity and multiplicity can be discarded. |
| 11 | Round-trip claims are precise | Broad gap | Documentation and current binary behavior disagree. |
| 12 | Delta keys preserve multiplicity | Broad gap | Single-value map entries overwrite repeated facts. |
| 13 | Consumers cannot redefine producer contracts | Broad gap | Positional and string-based consumer assumptions alter meaning. |
| 14 | Cross-language behavior is covered | Partial | Coverage exists but ownership invariants are uneven. |
| 15 | Every operation has a contract matrix | Broad gap | No complete target/cardinality/order/wire matrix exists. |
| 16 | Adversarial cases are tested | Partial | Several focused regressions exist; permutation coverage is incomplete. |
| 17 | Correctness has priority over compression | Partial | Some codecs preserve compactness by omitting semantic operands. |
| 18 | Invariants are executable | Partial | Valuable tests exist, but enforcement is not lifecycle-complete. |

At the original reconnaissance baseline, no invariant was classified as fully
green across the complete production lifecycle.

## 6. Existing evidence worth preserving

The following tracked tests represent useful architectural assets and should
be retained or strengthened during migration:

- `src/tests/ir/pattern_identity.rs`
- `src/tests/ir/pattern_identity_downstream.rs`
- `src/tests/ir/hierarchical_flags.rs`
- nested ownership and call-attribution tests
- named and string-table equality tests
- validator tests for rules that the validator actually enforces

The production implementation remains the behavioral reference for unrelated
behavior during migration. Existing behavior that contradicts an approved
identity or preservation invariant should be recorded as a defect, not
preserved silently.

## 7. Risk assessment

| Risk | Likelihood | Impact | Reason |
|---|---|---|---|
| Cross-class or cross-method fact attribution | High | High | Position is used where stable identity is available. |
| Silent loss of unresolved facts | High | High | Projection does not uniformly fail on unresolved targets. |
| Repeated-fact loss in delta/replay | High | High | One map entry represents potentially many operations. |
| False round-trip confidence | High | High | Opcode-only assertions miss identity and operand loss. |
| Cross-language modifier false positives | Medium | Medium | Ownership boundaries differ among language producers. |
| Accidental public/wire break during repair | Medium | High | Correcting types and codecs changes shared contracts. |
| Over-engineered enforcement framework | Medium | Medium | A generic registry could duplicate compiler and test capabilities. |

## 8. Architectural decisions requiring approval

Implementation should not begin on cross-cutting portions until the following
decisions are explicit:

1. **Semantic family model:** which current flag-like facts become declaration
   modifiers, control summaries, patterns, side effects, and execution
   contexts.
2. **Projection failure contract:** how structured projection failures reach
   internal callers and externally visible MCP responses.
3. **Per-operation cardinality:** singular, optional, ordered-many, set,
   multiset, or explicitly reducible.
4. **Binary compatibility:** version bump, compatibility decoder, migration
   window, and behavior for unrepresentable operations.
5. **Gate rollout:** whether initial failing contract tests are kept on a
   migration branch or introduced only with the corresponding repair.

The companion implementation plan recommends resolving these decisions in
narrow slices rather than approving one repository-wide rewrite.

## 9. Conclusion

The repository already has the core ingredient needed for correct attribution:
stable semantic identity. The architectural weakness is that downstream layers
do not consistently treat that identity as authoritative.

The highest-value correction is a checked, two-pass, identity-driven
hierarchical projection preceded by exhaustive validation. Typed semantic
families, multiplicity-aware delta/replay, truthful wire contracts, and focused
fitness tests then close the remaining routes by which identity or meaning can
be lost.

This report is an investigation artifact. The proposed target architecture,
work sequence, and acceptance criteria are defined in the companion plan.
