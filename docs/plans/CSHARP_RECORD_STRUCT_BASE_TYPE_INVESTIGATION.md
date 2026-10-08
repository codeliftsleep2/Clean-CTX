# C# Struct and Record Base-Type Investigation

**Author:** Agent MaxHeadRoom
**Date:** 2026-10-08
**Status:** Implemented; focused RED→GREEN regressions passed

## Question

Does the cross-file C# base-type defect fixed for classes in v0.9.1 also affect structs or records, and what is the smallest architecturally correct fix?

## Finding

Yes. C# structs and records with base-list entries do not currently project those entries into semantic edges. The class fix cannot simply be reused because the canonical IR erases declaration kind while the public builtin graph preserves Struct and Record identity.

The smallest correct fix is a C#-scoped, capture-backed semantic projection at the compiler semantic-projection boundary:

- capture each parser type node directly under a C# base_list;
- associate it with the nearest enclosing struct.root or record.root capture;
- emit builtin/Struct/<owner> --HasBaseType--> builtin/TypeRef/<written> or builtin/Record/<owner> --HasBaseType--> builtin/TypeRef/<written>;
- leave classes on the existing canonical BaseTypeRef path;
- do not infer Extends or Implements.

This preserves the neutral written-type fact without inventing a false class identity or guessing whether a record base-list entry is a class or interface.

## Relationship to v0.9.1

Treat the shipped v0.9.1 C# class `BaseTypeRef` → `HasBaseType` projection as the authoritative existing behavior.

This task extends that neutral written-base-type capability to C# `Struct` and `Record` owners where the canonical `BaseTypeRef` path cannot preserve the correct public owner identity.

Do not alter or replace the v0.9.1 `Class` path.

`CSharpSemanticProjection` already exists as the C#-scoped semantic compatibility producer introduced by v0.9.1. Increment its generation for this behavior change; do not introduce another producer key.

## Code evidence

### Parsing already distinguishes every declaration kind

src/queries.rs captures class, interface, struct, enum, and record roots separately. The C# tree-sitter grammar also exposes base_list beneath class and struct declarations and aliases record_base to the same named base_list node. Individual entries are structured type nodes, including qualified and generic types.

Consequence: the parser already provides a reliable structural boundary. A raw-text comma/newline parser is unnecessary and less correct.

### Canonical lowering erases the owner kind

src/ir/pipeline/core.rs lowers class.root, struct.root, enum.root, trait.root, and record.root to CoreOp::DefClass.

That makes the canonical instruction stream insufficient to distinguish a C# Struct owner from a Record or Class owner at semantic projection time.

### The builtin graph preserves the owner kind

src/layers/meta/builtin.rs registers declarations with distinct builtin kinds: Class, Interface, Struct, Enum, Trait, and Record.

src/tests/mcp/workspace_query_builtin.rs already verifies public C# Struct and Record registration. Any new relationship must use those existing identities.

### Current C# relationship extraction is class-only

src/ir/layers/csharp.rs calls relationship extraction for class.root. The struct.root, enum.root, trait.root, and record.root branches do nothing.

Before this implementation, a struct interface list and a record base list produced neither canonical BaseTypeRef operations nor neutral HasBaseType facts.

### BaseTypeRef is class-owned throughout the canonical pipeline

BaseTypeRef is documented and validated with a ClassId owner in src/ir/opcodes.rs and src/ir/identity/contracts.rs. Hierarchical encoding stores it under ClassNode, and the generic semantic projector hardcodes builtin Class as its subject.

Emitting BaseTypeRef for a struct or record without a wider identity change would create builtin/Class/Foo relationships alongside the already-correct builtin/Struct/Foo or builtin/Record/Foo declaration. That is semantically false.

### Semantic projection is an established separate authority

src/ir/compiler.rs builds meta edges and then appends semantic facts at the compilation boundary. It has both the instruction stream and the original captures available, and publishes the resulting edges with semantic provenance.

Using C# captures here is a focused extension of an existing boundary, not a second graph or new global indexing policy.

## Recommended implementation

### 1. Add structured captures

Extend the C# query with captures for the individual type nodes that are direct entries of a base_list. Avoid capturing the entire textual list.

### 2. Resolve the owner from capture spans

During C# semantic projection, associate each base type capture with the nearest enclosing struct.root or record.root span. Use the same declaration-name extraction authority used by builtin declaration registration.

Nearest enclosing ownership matters for nested declarations. File order alone is not sufficient.

### 3. Emit neutral owner-local facts

Emit HasBaseType from the preserved public owner kind to TypeRef using the normalized written type:

- Struct/<name> -> TypeRef/<written>
- Record/<name> -> TypeRef/<written>

Do not emit Extends or Implements. For records, syntax alone does not safely classify every written entry without resolution. HasBaseType deliberately records only the source fact.

### 4. Preserve the class path

Do not emit capture-backed class facts. Existing Class-owned BaseTypeRef instructions already project HasBaseType. Keeping one authority per owner kind avoids duplicate edges and divergent normalization.

### 5. Version the C# semantic producer

Increment CSharpSemanticProjection generation so stale snapshots cannot silently retain the older incomplete semantics. Non-C# producers remain unchanged.

## Required RED regressions

Add tracked tests under src/tests before implementation:

1. C# struct interface entry is queryable forward from Struct and reverse from TypeRef.
2. C# record base list exposes every written entry through Record.
3. record struct uses Record identity, never Struct or Class.
4. Qualified and generic written types produce one intact TypeRef each.
5. Multiline base lists behave identically to single-line lists.
6. Nested declarations associate entries with the nearest enclosing owner.
7. Recompilation after removing or replacing a base entry retracts the old fact.
8. Repeated publication is stable and does not duplicate edges.
9. Struct and record base entries never create a Class subject.
10. No Extends or Implements fact is fabricated.
11. Older C# semantic-projection generations are rejected while non-C# generation behavior is unchanged.

## Options and tradeoffs

### Option A — Capture-backed C# semantic projection (recommended)

Benefits:

- smallest change that preserves correct Struct and Record identity;
- uses parser structure rather than fragile text parsing;
- avoids canonical wire, delta, hierarchy, and storage migrations;
- keeps language-specific ambiguity inside the existing C# semantic producer;
- supports deterministic replacement and provenance through the current semantic edge lifecycle.

Costs:

- these facts are derived semantic edges rather than canonical CoreOps;
- owner association must be span-aware and carefully tested;
- class and struct/record facts originate through different internal paths, although they converge on the same public relation.

### Option B — Add a typed canonical owner-kind base fact

Introduce a canonical base-type operation whose owner carries declaration kind.

Benefits:

- one canonical representation for all declaration kinds;
- stronger regeneration and serialization symmetry.

Costs:

- requires changes across opcode contracts, validation, named and binary encoding, hierarchical storage, delta/control-full paths, compatibility tests, and semantic projection;
- creates a public compatibility and persistence migration for a narrowly scoped missing fact.

This is architecturally coherent but disproportionate for the current defect.

### Option C — Add DefStruct and DefRecord canonical operations

Preserve declaration identity throughout the canonical IR.

Benefits:

- resolves the deeper type-erasure issue comprehensively;
- future language semantics can rely on typed canonical owners.

Costs:

- broad identity migration across the compiler and persistence stack;
- much larger regression surface and release risk;
- not required to expose the neutral base-list fact correctly.

This may be a future architectural direction, not the smallest fix.

### Option D — Reuse the raw class relationship parser

Rejected. The current string parser stops at newlines and splits on commas, which is unsafe for multiline and generic types. It also cannot repair erased owner identity.

### Option E — Normalize structs and records to Class

Rejected. The public graph already promises distinct Struct and Record identities, backed by tests. Creating Class aliases would make query results internally inconsistent.

## Adjacent issue kept out of scope

record.root currently falls through the generic CoreOp capture-text path, so its DefClass payload may retain raw declaration text while builtin registration independently extracts the correct Record name. This is a separate canonical record-name normalization defect. It should receive its own investigation and regressions rather than being silently bundled into the semantic relationship fix.

## Architectural decision

Option A was approved and implemented. It changes externally visible C# query completeness for Struct and Record base lists within this authorized scope:

- C# Struct and Record owners only;
- neutral HasBaseType facts only;
- no Extends/Implements inference;
- no canonical opcode or public identity migration;
- generation bump limited to CSharpSemanticProjection.

## Implementation outcome

The approved capture-backed projection is now the production path. The implementation:

- appends a semantic-only C# capture query to the existing single IR parse, leaving the shared compression/diff query unchanged;
- selects the nearest enclosing declaration across all C# type kinds, then emits only for Struct and Record owners;
- handles aliased record base-list children and positional-record base constructors structurally by capture span;
- preserves record, record class, and record struct public names through the shared declaration-name authority;
- leaves the v0.9.1 canonical Class projection unchanged;
- increments the existing CSharpSemanticProjection generation from 1 to 2.

Tracked production-path regressions established RED on the unfixed implementation and passed unchanged after implementation. They cover forward/reverse queries, multiline and generic record entries, nested ownership, replacement retraction, repeat-publication stability, neutral relation semantics, correct owner identity, and producer generation.

## Release-note correction

The v0.9.1 documentation now describes queryable neutral C# class, struct, and record base types, matching the implemented release behavior.
