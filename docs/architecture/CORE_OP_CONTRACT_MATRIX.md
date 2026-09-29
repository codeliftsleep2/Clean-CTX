# CoreOp Architectural Contract Matrix

**Status:** Final implemented contract for `0.8.0-rc`. Every current `CoreOp`
row is normative and was traced through the registered production lifecycle
during Phase 9.

**Date:** 2026-09-17

**Implementation note:** The first four normative rows were implemented and
user-verified in the first projection slice. The two field rows were implemented
and user-verified in the second bounded projection slice. The shared typed
validator for all six normative rows was implemented and user-verified in the
third bounded slice on 2026-09-18. Phase 4A implemented and user-verified the
shared validation contracts for the remaining rows on 2026-09-18. During that
verification, production evidence proved that `TypeAlias` is also the carrier
for repeated `Φ` metadata. Its ordered-many correction was explicitly approved
on 2026-09-18 and is recorded below. Phases 4B and 4C implemented and user-
verified the method- and class-fact hierarchy contracts. Phase 4D implemented
schema-derived typed pattern targeting and was user-verified on 2026-09-18.

**Related documents:**

- [`IR_PROJECTION_RECONNAISSANCE_REPORT.md`](IR_PROJECTION_RECONNAISSANCE_REPORT.md)
- [`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md)
- [`BINARY_V04_CONTRACT.md`](BINARY_V04_CONTRACT.md)
- [`IR_ARCHITECTURE_CERTIFICATION.md`](IR_ARCHITECTURE_CERTIFICATION.md)

## 1. Purpose

This matrix assigns an explicit producer, identity, ownership, cardinality,
ordering, projection, delta, and wire contract to every current `CoreOp`
variant in `src/ir/opcodes.rs`.

It is intentionally a document, not a runtime registry. The implementation
must enforce these contracts with Rust types, exhaustive matches, checked
validation and projection, and focused tests.

## 2. Approved architectural defaults

The following defaults were approved on 2026-09-17 and are normative:

1. Introduce internal `ClassId`, `InterfaceId`, `MethodId`, `FieldId`, and `ParameterId`
   types while preserving existing serialized string shapes initially.
2. Unresolved, duplicate, or kind-mismatched identities fail checked
   projection with structured errors. MCP maps failures to its existing error
   contract and never returns a partial hierarchy as success.
3. Preserve every occurrence by default. Deduplication, union, replacement,
   or reduction is permitted only when an operation contract explicitly
   authorizes it.
4. A corrected binary format is the target. Do not add a compatibility reader
   for an older format unless repository evidence proves persisted artifacts
   are a supported input.
5. Each failing architectural test lands with the repair that makes it green.
   The default branch is not knowingly left red to preserve RED history.

These defaults must not be reopened without direct contradictory source
evidence and architectural review.

## 3. Source facts affecting the contract

### 3.1 Identity namespaces

- `DefMethod` facts are later referenced using only `method_id`; method IDs
  must therefore be unique within a compiled IR, not merely within one class.
- `FieldType` references only `field_id`; field IDs must likewise be unique
  within a compiled IR.
- `Param` is identified by `(method_id, parameter_id)` in the current wire and
  delta shapes. `ParameterId` uniqueness is required within its owning method.
- Class, method, field, and parameter strings remain unchanged on serialized
  boundaries during the first migration.
- Phase 9 adds `InterfaceId` and explicit interface-owned method/field
  declarations. Existing serialized strings remain unchanged.
- Despite its historical name, `TypeAlias` is an occurrence fact used both for
  configured type substitutions and repeated `Φ` metadata markers. Its alias
  operand is not a unique definition identity.

### 3.2 External symbolic references

The referenced parent in `Extends`, interface in `Implements`, dependency in
`Injects`, original type in `TypeAlias`, and callee in `Call` can represent
symbols not defined in the same compiled IR. Where an operation has a local
owning identity, that owner must resolve; external symbolic operands and the
`TypeAlias` payload need not.

### 3.3 Binary version-number contradiction

Before Phase 8, the binary encoder emitted `VERSION = 0x03` and the decoder
nominally accepted `0x01`, `0x02`, and `0x03`. The corrected/versioned binary
format is the approved physical version `0x04`.

The binary migration is deferred to its later phase and is not part of the
bounded `Param`/`Return` implementation slice.

Phase 8A resolves the complete corrected layout in
[`BINARY_V04_CONTRACT.md`](BINARY_V04_CONTRACT.md). Source inspection found no
persisted binary fixture and no production writer of non-empty binary IR.
Physical `0x03` omits eight identity operands and raw `file_id`; the corrected
`0x04` target preserves the complete `CompiledIR`, instruction order, and all
occurrences. No lossy `0x01`–`0x03` compatibility decoder is authorized.

## 4. Contract vocabulary

### 4.1 Cardinality

| Term | Meaning |
|---|---|
| Unique definition | Exactly one definition for the identity; duplicates fail. |
| Optional singular | Zero or one fact per target; duplicates fail. |
| Ordered many | Preserve operation order and every occurrence. |
| Ordered payload | Preserve order and duplicates inside the operation payload. |

No current row authorizes last-write-wins behavior. No current repeatable row
authorizes deduplication or set union.

### 4.2 Delta identity

`occurrence` means the stable ordinal among otherwise equal operations in the
canonical stream. Including it is necessary because the approved default
requires exact duplicates to survive.

Definition and optional-singular facts do not use occurrence identity because
duplicates are invalid. Repeatable facts use their complete semantic payload
plus occurrence.

Corrected delta protocol `dv: 2` carries positional `insert`, `remove`, and
`replace` edits. Position controls placement; destructive edits also carry the
expected tuple and typed semantic identity plus occurrence ordinal. Replay
must reject a tuple or occurrence mismatch transactionally. Legacy `+ / ~ / -`
deltas remain decode-only compatibility input and ambiguous legacy targets
fail rather than selecting an arbitrary occurrence. This protocol version is
independent of the CoreOp binary format version.

### 4.3 Wire classifications

| Term | Meaning |
|---|---|
| Semantic | All operands and their meaning survive round trip. |
| Structural | Operation shape survives but some semantic operands do not. |
| Lossy | Known canonical meaning is discarded or normalized. |
| Unsupported | Encoding must return an error rather than fabricate data. |

The target positional JSON and corrected binary contracts are semantic for all
supported `CoreOp` variants. The hierarchical representation is a projection,
not automatically a semantic serialization of the canonical stream.

## 5. Per-operation identity and multiplicity matrix

| `CoreOp` | Producer authority | Owning identity and validation | Cardinality and order | Authorized merge |
|---|---|---|---|---|
| `DefClass(class, name)` | Structural compiler capture; trusted decoder | Defines `ClassId`; unique; non-empty ID; duplicate or cross-kind collision fails | Unique definition | None |
| `DefMethod(class, method, name)` | Structural compiler capture; trusted decoder | Defines globally unique `MethodId`; owner `ClassId` must resolve; duplicate or wrong-kind owner fails | Unique definition | None |
| `DefField(class, field, name)` | Structural compiler capture; trusted decoder | Defines globally unique `FieldId`; owner `ClassId` must resolve; duplicate or wrong-kind owner fails | Unique definition | None |
| `DefInterface(interface, name)` | Structural compiler capture; trusted decoder | Defines interface identity in its existing string namespace; duplicate or cross-kind collision fails | Unique definition | None |
| `DefInterfaceMethod(interface, method, name)` | Structural interface capture; trusted decoder | Defines globally unique `MethodId`; owner `InterfaceId` must resolve; duplicate or wrong-kind owner fails | Unique definition | None |
| `DefInterfaceField(interface, field, name)` | Structural interface capture; trusted decoder | Defines globally unique `FieldId`; owner `InterfaceId` must resolve; duplicate or wrong-kind owner fails | Unique definition | None |
| `Param(method, parameter, type, name)` | Signature compiler capture; trusted decoder | Defines `(MethodId, ParameterId)`; method must resolve; duplicate parameter ID within method fails | Ordered many per method; source parameter order is semantic | None |
| `Return(method, type)` | Signature compiler capture; trusted decoder | Targets `MethodId`; method must resolve | Optional singular per method; duplicate fails | None |
| `FieldType(field, type)` | Field compiler capture; trusted decoder | Targets `FieldId`; field must resolve | Optional singular per field; duplicate fails | None |
| `MethodModifiers(method, values)` | Language declaration layer; trusted decoder | Targets `MethodId`; method must resolve; typed non-empty payload | Ordered many operations; payload order and duplicates preserved | None |
| `ClassModifiers(class, values)` | Language declaration layer; trusted decoder | Targets `ClassId`; class must resolve; typed non-empty payload | Ordered many operations; payload order and duplicates preserved | None |
| `InterfaceModifiers(interface, values)` | Language declaration layer; trusted decoder | Targets `InterfaceId`; interface must resolve; typed non-empty payload | Ordered many operations; payload order and duplicates preserved | None |
| `ControlSummary(method, values)` | Core capture pipeline; trusted decoder | Targets `MethodId`; method must resolve; closed typed non-empty payload (`IF`, `LOOP`, `RET`, `THROW`) | Ordered many operations; payload order and duplicates preserved | None |
| `PatternFacts(method, values)` | Additive pattern producer; trusted decoder | Targets `MethodId`; method must resolve; closed typed non-empty payload with accessor properties structurally owned by `Getter`/`Setter` | Ordered many operations; fact order and duplicates preserved | None |
| `Flags(method, values)` | Legacy decoder/manual compatibility only | Targets `MethodId`; method must resolve; empty payload plus every typed semantic-family spelling is invalid | Ordered many legacy operations | None |
| `ClassFlags(class, values)` | Residual class-metadata producer; trusted decoder | Targets `ClassId`; class must resolve; empty payload and declaration-modifier spellings are invalid | Ordered many operations; payload order and duplicates preserved | None |
| `Extends(child, parent)` | Language layer; trusted decoder | Targets child `ClassId`; child must resolve; parent is an external-capable symbolic reference | Optional singular per child; duplicate fails | None |
| `InterfaceExtends(child, parent)` | Language layer; trusted decoder | Targets child `InterfaceId`; child must resolve; parent is an external-capable symbolic reference | Ordered many per interface; every occurrence preserved | None |
| `Implements(class, interface)` | Language layer; trusted decoder | Targets `ClassId`; class must resolve; interface is an external-capable symbolic reference | Ordered many per class; every occurrence preserved | None |
| `Injects(class, dependencies)` | Language or pattern layer; trusted decoder | Targets `ClassId`; class must resolve; dependencies are external-capable symbolic references | Ordered many operations; each payload is ordered and duplicate-preserving | None |
| `Import(alias, module, named)` | Import compiler capture; trusted decoder | Defines import alias in existing string namespace; duplicate alias fails | Unique definition per alias | None |
| `TypeAlias(alias, original)` | Type-alias and meta-marker producers; trusted decoder | Alias token must be non-empty; `original` is an external-capable payload, not a locally resolved identity | Ordered many; every operation and exact duplicate preserved | None |
| `Pattern(name, args)` | Approved pattern recognizers; trusted decoder | `args[0]` is `ClassId`; method-level schemas also require `args[1]` as `MethodId` owned by that class; both must resolve; pattern schema determines remaining operands | Ordered many; complete args and duplicate occurrences preserved | None |
| `Body(method, text, start, end)` | Edit-fidelity compiler capture; trusted decoder | Targets `MethodId`; method must resolve; span fields are both present or both absent; present span must be valid | Optional singular per method; duplicate fails | None |
| `DataFlow(method, direction, target)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; direction must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `ControlFlow(method, kind, target)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; kind must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `SideEffect(method, SideEffectKind)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; closed typed vocabulary (`pure`, `io`, `mutation`, `async`, `transaction`) | Ordered many; every occurrence preserved | None |
| `ExecutionContext(method, ExecutionContextKind)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; closed typed vocabulary (`sync`, `async`, `thread_bound`, `transaction_scope`, `realtime`) | Ordered many; every occurrence preserved | None |
| `Call(caller, callee, argc, spread)` | Structural call capture; trusted decoder | Targets caller `MethodId`; caller must resolve; callee is deliberately an unresolved textual name; `argc` is written-node count and is exact only when `spread` is false | Ordered many; every call-site occurrence preserved | None |

### 5.1 Pattern schema restriction

The current pattern representation embeds target identity in untyped arguments.
The named compressed method-pattern schemas validate their documented tuple
shape and class/method ownership. Other pattern names retain the established
generic class-pattern contract: `args[0]` is the resolved class target and all
remaining operands are opaque metadata. No validator or projector may use ID
prefixes such as `starts_with('M')` to decide target kind. Phase 4D introduces
an internal typed `PatternTarget`; it does not alter the four approved identity
newtypes or either serialized representation.

### 5.2 Flags restriction

Declaration modifiers, control summaries, and method-pattern facts are closed
typed families carried by their dedicated operations. Generic `Flags` rejects
all three vocabularies and remains only for legacy unknown payloads;
`ClassFlags` remains residual class metadata.

## 6. Projection, delta, and wire matrix

| `CoreOp` | Checked projection target | Delta identity | `0x03` baseline status | `0x04` contract |
|---|---|---|---|---|
| `DefClass` | One non-synthetic `ClassNode` | `ClassId` | Lossy: class ID omitted | Encode class ID and name |
| `DefMethod` | Method under resolved owner class | `MethodId` plus owner for consistency | Lossy: owner class ID omitted | Encode owner, method ID, and name |
| `DefField` | Field under resolved owner class | `FieldId` plus owner for consistency | Lossy: owner class ID omitted | Encode owner, field ID, and name |
| `DefInterface` | Explicit `InterfaceNode`; never normalize to class | `InterfaceId` | Lossy: interface ID omitted | Encode interface ID and name |
| `DefInterfaceMethod` | Method under resolved owner interface | `MethodId` plus interface owner | Not defined | Additive opcode 26; preserve all operands |
| `DefInterfaceField` | Field under resolved owner interface | `FieldId` plus interface owner | Not defined | Additive opcode 27; preserve all operands |
| `Param` | Parameter under resolved method, in source order | `(MethodId, ParameterId)` | Semantic for present operands | Preserve all operands |
| `Return` | Singular return type under resolved method | `MethodId` | Semantic for present operands | Preserve all operands |
| `FieldType` | Singular type under resolved field | `FieldId` | Semantic for present operands | Preserve all operands |
| `MethodModifiers` | Repeated typed modifier occurrences under resolved method | `(MethodId, complete values, occurrence)` | Semantic via additive opcode 22 under `0x03` | Preserve payload and occurrence order |
| `ClassModifiers` | Repeated typed modifier occurrences under resolved class | `(ClassId, complete values, occurrence)` | Semantic via additive opcode 23 under `0x03` | Preserve payload and occurrence order |
| `InterfaceModifiers` | Repeated typed modifier occurrences under resolved interface | `(InterfaceId, complete values, occurrence)` | Not defined | Additive opcode 28; preserve payload and occurrence order |
| `ControlSummary` | Repeated typed summary occurrences under resolved method | `(MethodId, complete values, occurrence)` | Semantic via additive opcode 24 under `0x03` | Preserve payload and occurrence order |
| `PatternFacts` | Repeated typed pattern-fact occurrences under resolved method | `(MethodId, complete values, occurrence)` | Semantic via additive opcode 25 under `0x03` | Preserve payload and occurrence order |
| `Flags` | Repeated legacy unknown occurrences under resolved method | `(MethodId, complete values, occurrence)` | Semantic for present operands | Preserve legacy payload and occurrence order |
| `ClassFlags` | Repeated flag occurrences under resolved class | `(ClassId, complete values, occurrence)` | Semantic for present operands | Preserve payload and occurrence order |
| `Extends` | Singular parent reference under resolved child | `ClassId` | Lossy: child ID omitted | Encode child and parent |
| `InterfaceExtends` | Repeated parent references under resolved interface | `(InterfaceId, parent, occurrence)` | Not defined | Additive opcode 29; preserve all occurrences |
| `Implements` | Repeated interface references under resolved class | `(ClassId, interface, occurrence)` | Lossy: class ID omitted | Encode class and interface |
| `Injects` | Repeated dependency payloads under resolved class | `(ClassId, complete dependencies, occurrence)` | Semantic for present operands | Preserve payload and occurrence order |
| `Import` | File-level import definition | Import alias | Lossy: alias omitted | Encode alias, module, and named export |
| `TypeAlias` | File-level ordered occurrence fact | `(alias, original, occurrence)` | Lossy: alias omitted | Encode alias, original payload, and occurrence order |
| `Pattern` | Class or method collection selected by validated schema | `(name, complete args, occurrence)` | Semantic for present operands | Preserve complete schema and occurrences |
| `Body` | Singular body under resolved method | `MethodId` | Semantic in current `0x03`, including optional spans | Preserve text and paired spans |
| `DataFlow` | Repeated data-flow facts under resolved method | `(MethodId, direction, target, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `ControlFlow` | Repeated control-flow facts under resolved method | `(MethodId, kind, target, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `SideEffect` | Repeated side effects under resolved method | `(MethodId, effect, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `ExecutionContext` | Repeated contexts under resolved method | `(MethodId, context, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `Call` | File-level call table after caller validation | `(caller, callee, argc, spread, occurrence)` | Semantic for present operands | Preserve all occurrences and qualifier |

### 6.1 Hierarchical destination changes implied by the matrix

Phase 4B implements the method-scoped subset of these changes:

- method `flags` stores an ordered collection of complete operation payloads;
- `side_effect` is an occurrence-preserving collection;
- `execution_context` is an occurrence-preserving collection;
- TypeScript `@Injectable` is class/metaclass DI metadata, not a method execution context or an `Injects` dependency edge. Phase 6E removes its invalid `CTX(class_id, "di_scope")` encoding; a replacement class-scoped family requires separate architectural review.
- `Body`, `DataFlow`, and `ControlFlow` resolve through the complete method-ID
  index rather than current-scope state;
- hierarchical wire output declares schema revision `"hs": 2` and its decoder
  retains the previously supported unmarked flat/scalar shape;
- unknown explicit hierarchical schema revisions fail decoding;

Phase 4C implements the class-scoped subset and was user-verified:

- `ClassFlags`, `Extends`, `Implements`, and `Injects` resolve through the
  complete typed class-ID index, independent of instruction order;
- repeated class flags remain separate ordered operation payloads;
- injection operation boundaries, payload order, and duplicates are preserved;
- optional-singular `Extends` relies on shared validation to reject duplicates;
- hierarchical wire output advances to schema revision `"hs": 3`;
- strict revision-2 flat class containers and unmarked legacy documents remain
  readable through decode-only compatibility adapters.

Phase 4D implements schema-derived typed pattern targeting without changing
the serialized `CoreOp::Pattern` or hierarchical shapes. Validation and
projection share the same `PatternTarget` parser; projection attaches patterns
through complete class and method indexes after definitions are materialized.
Pattern names, never identifier prefixes, determine target kind. The user-run
gate was green on 2026-09-18.

Phase 6C adds ordered typed `pattern_facts` (`pf`) under resolved methods and
advances the hierarchy to revision 6. This annotation migration never changes
`Body` text or spans; Edit-fidelity bodies remain byte-exact and continue to
force conservative pattern-compression decline where required by IRPAT-001.

Phase 6D types side effects as `SideEffectKind` without changing their named
wire spelling, binary opcode, hierarchical `se` strings, compact LLM `se:`
projection, hierarchical revision 6, LLM schema v5, or physical binary `0x04`.
Every occurrence, order, and duplicate remains authoritative. `SideEffect`
continues to block unsafe consumptive pattern compression. Phase 9 confirmed
the default compiler, checked projection, persistence, reload, and registered
MCP exposure for this family.

Phase 9 adds explicit `InterfaceNode` projection and interface-owned canonical
method and field declarations. Class and interface ownership remain distinct
through validation, physical `0x04`, persistence, hierarchy, and compact LLM
rendering.

The serialized hierarchical shape is externally observable. Its revision-2
method-fact contract and revision-3 class-fact contract were approved on
2026-09-18. They are distinct from the later corrected binary physical version
`0x04`.

## 7. Validation requirements

Validation must use an exhaustive `CoreOp` match and collect all definition
indexes before validating references, so valid pre-definition facts are
accepted.

Every diagnostic must contain:

- a stable error code;
- instruction index;
- operation name;
- offending identity and observed kind;
- expected identity kind;
- owner identity where applicable.

Minimum failure classes:

| Failure | Required result |
|---|---|
| Duplicate unique definition | Structured error |
| Duplicate optional-singular fact | Structured error |
| Unknown owning identity | Structured error |
| Identity exists in wrong namespace | Structured kind-mismatch error |
| Method owner class missing | Structured error |
| Field owner class missing | Structured error |
| Method owner inconsistent with pattern class | Structured error |
| Invalid pattern tuple shape | Structured error |
| Partial or invalid body span | Structured error |
| Unknown controlled-vocabulary value | Structured error |

Validation must not require external symbolic references to resolve locally
unless a future operation-specific contract explicitly changes that rule.

## 8. Transformation contract

Every transformation consuming canonical operations must be classified as:

- **identity-preserving:** all definitions and referenced identities survive;
- **identity-remapping:** a complete old-to-new mapping is returned and applied
  to every referencing fact;
- **identity-removing:** removal is explicitly authorized and no surviving fact
  refers to the removed identity.

Pattern recognition is currently required to be identity-preserving and
additive for declaration identity. Existing pattern identity tests remain
contract evidence. Stream reordering is allowed only when the operation matrix
says order is insignificant or the transformation preserves the declared
relative order.

## 9. Required first-slice tests

The first implementation slice should protect `DefClass`, `DefMethod`,
`Param`, and `Return` with tracked tests covering:

1. method facts after their definition;
2. method facts before their definition;
3. method facts after another class or method becomes current;
4. two classes with interleaved method facts;
5. unresolved method target;
6. a class identity used where a method identity is required;
7. duplicate class and method definitions;
8. duplicate return facts;
9. ordered repeated parameters;
10. MCP mapping of checked projection failure without partial success.

The test and corresponding repair land together under the approved rollout
policy.

## 10. Resolved review items and separately deferred work

Typed semantic-family vocabularies and language mappings were implemented in
Phase 6. Phase 9 introduced `InterfaceId`, explicit interface-owned member
operations, and distinct hierarchy/LLM representation.

Resolved on 2026-09-18: method flag occurrences, side effects, and execution
contexts use hierarchical schema revision `"hs": 2`; unmarked legacy
documents remain readable through the established compatibility path.

Resolved on 2026-09-18: class flag and injection occurrences use hierarchical
schema revision `"hs": 3`; strict revision-2 and unmarked legacy documents
remain readable through decode-only compatibility paths.

Import aliases are normatively file-wide because no downstream operation
carries a narrower owner. `TypeAlias` is not a definition identity; production
evidence established its ordered-many occurrence contract. Class-scoped
injectable metadata remains a separately reviewable future semantic family;
it is not represented as method `ExecutionContext` or as an `Injects` edge.

## 11. Required second-slice tests

The bounded field-identity slice protects `DefField` and `FieldType` with
tracked tests covering:

1. field definitions before their owner class;
2. field types before and after their field definition;
3. two classes with interleaved field facts;
4. preservation of field-definition order;
5. unresolved and wrong-kind field owners;
6. duplicate, empty, and cross-kind field definitions;
7. unresolved, empty, and wrong-kind field-type targets;
8. duplicate optional-singular field-type facts;
9. unchanged serialized string shapes;
10. MCP mapping of the field projection failure without partial success.

The implementation removes synthetic orphan-field recovery. It does not alter
the hierarchical schema's retained `synthetic` field because interfaces and
legacy decoded documents remain outside this bounded slice.

## 12. Review checklist

- [x] Every current `CoreOp` variant appears exactly once in each matrix.
- [x] Definition and optional-singular cardinalities are accepted.
- [x] Repeatable facts preserve every occurrence by default.
- [x] External symbolic references are distinguished from owning identities.
- [x] Delta identity preserves exact duplicates.
- [x] Projection destinations do not require silent loss or synthesis.
- [x] Current binary losses are accurately identified by Phase 8A.
- [x] The physical corrected-binary version and layout are resolved as `0x04`.
- [x] Each row has tracked contract and production-path evidence.
