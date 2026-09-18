# CoreOp Architectural Contract Matrix

**Status:** Approved architectural defaults. The `DefClass`, `DefMethod`,
`Param`, `Return`, `DefField`, and `FieldType` rows are normative target
architecture. Other per-operation rows remain a Phase 0 draft for later
review.

**Date:** 2026-09-17

**Implementation note:** The first four normative rows were implemented and
user-verified in the first projection slice. The two field rows were implemented
and user-verified in the second bounded projection slice. All other rows remain
planning targets and were not broadened into these slices.

**Related documents:**

- [`IR_PROJECTION_RECONNAISSANCE_REPORT.md`](IR_PROJECTION_RECONNAISSANCE_REPORT.md)
- [`IR_ARCHITECTURE_LOCKDOWN_PLAN.md`](IR_ARCHITECTURE_LOCKDOWN_PLAN.md)

## 1. Purpose

This matrix assigns an explicit producer, identity, ownership, cardinality,
ordering, projection, delta, and wire contract to every current `CoreOp`
variant in `src/ir/opcodes.rs`.

It is intentionally a document, not a runtime registry. The implementation
must enforce these contracts with Rust types, exhaustive matches, checked
validation and projection, and focused tests.

## 2. Approved architectural defaults

The following defaults were approved on 2026-09-17 and are normative:

1. Introduce internal `ClassId`, `MethodId`, `FieldId`, and `ParameterId`
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
- `DefInterface`, import aliases, and type aliases are real identities but are
  outside the four approved first-slice newtypes. They remain validated string
  identities until a separately justified typed migration.

### 3.2 External symbolic references

The referenced parent in `Extends`, interface in `Implements`, dependency in
`Injects`, original type in `TypeAlias`, and callee in `Call` can represent
symbols not defined in the same compiled IR. Their owning source identity must
resolve locally; the external symbolic operand need not.

### 3.3 Binary version-number contradiction

The current binary encoder emits `VERSION = 0x03` and the decoder already
accepts `0x01`, `0x02`, and `0x03`. The corrected/versioned binary format is
therefore assigned the approved physical version `0x04`.

The binary migration is deferred to its later phase and is not part of the
bounded `Param`/`Return` implementation slice.

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
| `Param(method, parameter, type, name)` | Signature compiler capture; trusted decoder | Defines `(MethodId, ParameterId)`; method must resolve; duplicate parameter ID within method fails | Ordered many per method; source parameter order is semantic | None |
| `Return(method, type)` | Signature compiler capture; trusted decoder | Targets `MethodId`; method must resolve | Optional singular per method; duplicate fails | None |
| `FieldType(field, type)` | Field compiler capture; trusted decoder | Targets `FieldId`; field must resolve | Optional singular per field; duplicate fails | None |
| `Flags(method, values)` | Language layer and additive pattern classifiers; trusted decoder | Targets `MethodId`; method must resolve; empty payload is invalid | Ordered many operations; payload order and duplicates preserved | None until semantic families are separated |
| `ClassFlags(class, values)` | Language layer; trusted decoder | Targets `ClassId`; class must resolve; empty payload is invalid | Ordered many operations; payload order and duplicates preserved | None |
| `Extends(child, parent)` | Language layer; trusted decoder | Targets child `ClassId`; child must resolve; parent is an external-capable symbolic reference | Optional singular per child; duplicate fails | None |
| `Implements(class, interface)` | Language layer; trusted decoder | Targets `ClassId`; class must resolve; interface is an external-capable symbolic reference | Ordered many per class; every occurrence preserved | None |
| `Injects(class, dependencies)` | Language or pattern layer; trusted decoder | Targets `ClassId`; class must resolve; dependencies are external-capable symbolic references | Ordered many operations; each payload is ordered and duplicate-preserving | None |
| `Import(alias, module, named)` | Import compiler capture; trusted decoder | Defines import alias in existing string namespace; duplicate alias fails | Unique definition per alias | None |
| `TypeAlias(alias, original)` | Type-alias compiler/runtime assignment; trusted decoder | Defines type alias in existing string namespace; duplicate alias fails | Unique definition per alias | None |
| `Pattern(name, args)` | Approved pattern recognizers; trusted decoder | `args[0]` is `ClassId`; method-level schemas also require `args[1]` as `MethodId` owned by that class; both must resolve; pattern schema determines remaining operands | Ordered many; complete args and duplicate occurrences preserved | None |
| `Body(method, text, start, end)` | Edit-fidelity compiler capture; trusted decoder | Targets `MethodId`; method must resolve; span fields are both present or both absent; present span must be valid | Optional singular per method; duplicate fails | None |
| `DataFlow(method, direction, target)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; direction must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `ControlFlow(method, kind, target)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; kind must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `SideEffect(method, effect)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; effect must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `ExecutionContext(method, context)` | Language semantic layer; trusted decoder | Targets `MethodId`; method must resolve; context must be from the declared vocabulary | Ordered many; every occurrence preserved | None |
| `Call(caller, callee, argc, spread)` | Structural call capture; trusted decoder | Targets caller `MethodId`; caller must resolve; callee is deliberately an unresolved textual name; `argc` is written-node count and is exact only when `spread` is false | Ordered many; every call-site occurrence preserved | None |

### 5.1 Pattern schema restriction

The current pattern representation embeds target identity in untyped arguments.
The first migration must validate the documented tuple shape and stop using ID
prefixes such as `starts_with('M')` to decide target kind. A later typed
`PatternTarget` representation is desirable but is not part of the four
approved first-slice identity newtypes.

### 5.2 Flags restriction

The current `Flags` payload contains declaration modifiers, control summaries,
and additive classifications. Until those families are separated, no consumer
may deduplicate, reorder, reduce, or reinterpret repeated `Flags` operations.
Preservation is the safe interim contract; it is not an endorsement of the
flattened model.

## 6. Projection, delta, and wire matrix

| `CoreOp` | Checked projection target | Delta identity | Current binary status | Corrected binary target |
|---|---|---|---|---|
| `DefClass` | One non-synthetic `ClassNode` | `ClassId` | Lossy: class ID omitted | Encode class ID and name |
| `DefMethod` | Method under resolved owner class | `MethodId` plus owner for consistency | Lossy: owner class ID omitted | Encode owner, method ID, and name |
| `DefField` | Field under resolved owner class | `FieldId` plus owner for consistency | Lossy: owner class ID omitted | Encode owner, field ID, and name |
| `DefInterface` | Explicit interface representation or unsupported projection; never normalize silently to class | Interface identity | Lossy: interface ID omitted | Encode interface ID and name |
| `Param` | Parameter under resolved method, in source order | `(MethodId, ParameterId)` | Semantic for present operands | Preserve all operands |
| `Return` | Singular return type under resolved method | `MethodId` | Semantic for present operands | Preserve all operands |
| `FieldType` | Singular type under resolved field | `FieldId` | Semantic for present operands | Preserve all operands |
| `Flags` | Repeated flag occurrences under resolved method | `(MethodId, complete values, occurrence)` | Semantic for present operands | Preserve payload and occurrence order |
| `ClassFlags` | Repeated flag occurrences under resolved class | `(ClassId, complete values, occurrence)` | Semantic for present operands | Preserve payload and occurrence order |
| `Extends` | Singular parent reference under resolved child | `ClassId` | Lossy: child ID omitted | Encode child and parent |
| `Implements` | Repeated interface references under resolved class | `(ClassId, interface, occurrence)` | Lossy: class ID omitted | Encode class and interface |
| `Injects` | Repeated dependency payloads under resolved class | `(ClassId, complete dependencies, occurrence)` | Semantic for present operands | Preserve payload and occurrence order |
| `Import` | File-level import definition | Import alias | Lossy: alias omitted | Encode alias, module, and named export |
| `TypeAlias` | File-level type-alias definition | Type alias | Lossy: alias omitted | Encode alias and original type |
| `Pattern` | Class or method collection selected by validated schema | `(name, complete args, occurrence)` | Semantic for present operands | Preserve complete schema and occurrences |
| `Body` | Singular body under resolved method | `MethodId` | Semantic in current `0x03`, including optional spans | Preserve text and paired spans |
| `DataFlow` | Repeated data-flow facts under resolved method | `(MethodId, direction, target, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `ControlFlow` | Repeated control-flow facts under resolved method | `(MethodId, kind, target, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `SideEffect` | Repeated side effects under resolved method | `(MethodId, effect, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `ExecutionContext` | Repeated contexts under resolved method | `(MethodId, context, occurrence)` | Semantic for present operands | Preserve all occurrences |
| `Call` | File-level call table after caller validation | `(caller, callee, argc, spread, occurrence)` | Semantic for present operands | Preserve all occurrences and qualifier |

### 6.1 Hierarchical destination changes implied by the matrix

The current hierarchy cannot satisfy the matrix without these changes:

- `side_effect: Option<String>` becomes an occurrence-preserving collection;
- `execution_context: Option<String>` becomes an occurrence-preserving
  collection;
- repeated class flags remain distinguishable until a reducer is authorized;
- optional-singular fields reject duplicates rather than overwrite them;
- definitions and facts resolve through complete ID indexes;
- unresolved targets produce structured errors;
- synthetic classes are not created to hide unresolved owners;
- interfaces are represented explicitly or rejected as unsupported;
- patterns use a validated target schema, never prefix inference.

The serialized hierarchical shape may be externally observable. Its migration
requires the checked projection and MCP error policy already approved, plus a
separate compatibility review for field-shape changes.

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

## 10. Open review items not covered by the approved defaults

These items do not reopen the five defaults:

1. Decide whether the hierarchical external schema changes repeated
   `side_effect` and `execution_context` fields in place or introduces a new
   hierarchical schema version.
2. Approve the eventual semantic-family enum members and their language-
   specific mappings. Preservation remains mandatory in the meantime.
3. Decide whether `DefInterface` gains a dedicated typed ID in a later slice;
   the first slice retains its validated string identity.
4. Confirm whether import and type-alias identities are globally unique within
   a compiled file or scoped more narrowly. The draft assumes file-wide
   uniqueness because downstream operations do not carry a narrower owner.

None of these items blocks the first typed class/method identity, exhaustive
validation, or checked projection slice.

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

- [ ] All 21 current `CoreOp` variants appear exactly once in each matrix.
- [ ] Definition and optional-singular cardinalities are accepted.
- [ ] Repeatable facts preserve every occurrence by default.
- [ ] External symbolic references are distinguished from owning identities.
- [ ] Delta identity preserves exact duplicates.
- [ ] Projection destinations do not require silent loss or synthesis.
- [ ] Current binary losses are accurately identified.
- [ ] The physical corrected-binary version is resolved.
- [ ] Each row has an identified tracked test owner before implementation.
