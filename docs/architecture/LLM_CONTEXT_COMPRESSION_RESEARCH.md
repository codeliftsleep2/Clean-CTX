# LLM-context compression research

**Status:** presentation boundary (ARCH-003) implemented; SCHEMA-v5 density measured and language-dependent; task-based edit evaluation green (3/3)
**Date:** 2026-09-21  
**Scope:** the representation delivered to an LLM. Canonical IR and tool input contracts are unchanged. File-context content is being corrected to exclude unsolicited workspace graph snapshots; those facts remain available on demand through `workspace_query`. Structured file snapshots use a compact candidate only when it is safely cheaper than byte-exact raw source; otherwise raw is returned without a wrapper. Sections 1–13 preserve the pre-repair research findings; Section 14 records the approved production boundary and current checkpoint.

## Executive finding

Clean-CTX is not yet at a point where `CONTROL-PROD` can be used as the semantic baseline for a codec contest. The canonical state is stronger than the text currently placed in the model-readable MCP `content` channel. In particular, the checked hierarchy retains IDs, injection occurrences, structural calls, written call arity, and spread evidence, while `render_hierarchical_for_llm` omits injections and calls and generally renders declaration names instead of stable IDs. Framework and generic semantic edges are attached by `provide_code_context` under result `_meta`, not in `content`.

That is a pre-existing production-presentation gap, not a compression result. The valid research comparison is therefore:

```text
CONTROL-PROD = exact text models receive today in result.content
CONTROL-FULL = correctness-complete, intent-sensitive semantic envelope
candidate correctness = exact normalized CONTROL-FULL recovery
candidate production win = fewer model tokens than byte-exact raw source AND no reasoning regression versus raw
```

The strongest near-term hypothesis is that the typed architecture does permit materially denser output through scoped positional ownership, schema elision, columnar facts, and short local handles. It does **not** permit omitting an identity or relationship just because the current renderer omits it. No production codec should be selected until a model-visible correctness-complete envelope and its benchmark are approved.

## Evidence and terminology

Repository authorities used in this investigation:

- Canonical operations: [`src/ir/opcodes.rs`](../../src/ir/opcodes.rs)
- Checked hierarchical projection: [`src/ir/hierarchical.rs`](../../src/ir/hierarchical.rs)
- Current LLM renderer: [`src/ir/render_llm.rs`](../../src/ir/render_llm.rs)
- Generic call-edge projection: [`src/ir/semantic_projection.rs`](../../src/ir/semantic_projection.rs)
- Framework semantic model: [`src/layers/meta/semantic.rs`](../../src/layers/meta/semantic.rs)
- Registered handlers: [`src/mcp/tool_handlers/registry.rs`](../../src/mcp/tool_handlers/registry.rs)
- Production response assembly: [`src/mcp/tool_handlers/core/provide.rs`](../../src/mcp/tool_handlers/core/provide.rs), [`compress.rs`](../../src/mcp/tool_handlers/core/compress.rs), [`delta.rs`](../../src/mcp/tool_handlers/core/delta.rs), and [`restore.rs`](../../src/mcp/tool_handlers/core/restore.rs)
- Architectural separation rule: [`docs/ARCHITECTURAL_INVARIANTS.md`](../ARCHITECTURAL_INVARIANTS.md), ARCH-003

External references inform experimental design, not repository behavior:

- The [MCP low-level SDK guidance](https://py.sdk.modelcontextprotocol.io/v2/advanced/low-level-server/) says `content` is read by the model, `structured_content` is the typed form of that answer, and `_meta` is application-facing. It also warns that hosts decide what they render. This makes `_meta.semantic_edges` insufficient evidence that a model received those edges.
- OpenAI's [tiktoken token-counting guide](https://developers.openai.com/cookbook/examples/how_to_count_tokens_with_tiktoken) supports measuring the actual target encoding rather than characters.
- Anthropic exposes a model-specific [token counting API](https://platform.claude.com/docs/en/build-with-claude/token-counting); it should replace local approximation in the Claude arm of the experiment.
- The working-draft [TOON specification](https://github.com/toon-format/spec/blob/main/SPEC.md) is relevant evidence for schema-once/tabular encoding, but explicitly says it is best for uniform records and can lose its advantage on irregular nested data.
- [LLMLingua](https://arxiv.org/abs/2310.05736) intentionally removes low-information tokens and reports non-zero quality loss. That family is useful as a negative control but violates this project's exact correctness gate.

## 1. Exact production response and rendering trace

### Registered operations

The registry exposes four direct context-producing paths plus two lifecycle re-render paths:

| Operation | Primary purpose | Model-readable `content` on the relevant success path | Auxiliary wire fields |
|---|---|---|---|
| `provide_code_context` | preferred contextual read | full renderer text, raw source fallback, or a delta summary | full path: `_meta.semantic_edges`; delta path: delta and edges in `_meta` |
| `compress_code_context` | direct compilation/encoding | full renderer text | `ir`, `pretty`, `semantic_edges` as non-standard result siblings |
| `delta_code_context` | create/store IR baseline or delta | only “Baseline stored…” / “Cached IR…” / delta summary | baseline/edge/delta data as non-standard result siblings |
| `restore_context` | restore durable context | persisted compact output or fresh renderer text | hierarchical `ir`; edge **count**, not edge values, in `_meta` |
| `apply_delta` | apply accepted delta | freshly rendered compact text | version in `_meta` |
| `replay_history` | restore historical state | persisted compact output or fresh renderer text | hierarchical `ir`; no edge values in response |

`diff_code_context` returns a structural diff rather than the current semantic snapshot and is outside the full-context control. `save_context` returns an acknowledgement only.

### Normal full path

For `provide_code_context`, the production path is:

```text
tools/list + registry
  -> handle_provide_code_context
  -> compile_file_ir_focused
  -> checked_hierarchy_or_respond / try_ir_to_hierarchical
  -> render_hierarchical_for_llm_focused
  -> append "// ── <file-alias> (<resolved-path>) ──"
  -> append request-scoped §PATHMAP
  -> result.content[0].text

semantic edges
  -> WorkspaceIndex + durable/session ownership
  -> result._meta.semantic_edges (not content)
```

The exact full-text assembly is:

```text
<SCHEMA-v5 renderer text, trimmed>
// ── <alias> (<resolved path>) ──
§PATHMAP
  <alias> = <resolved path>
```

The path is therefore normally repeated twice. The schema legend is repeated on every full single-file response. Path scoping is request-local, which is safe but carries fixed cost.

### Renderer behavior

`render_hierarchical_for_llm[_focused]` is the renderer used by all full/re-render paths above. It:

- emits class names as comment boundaries and interfaces as `Q <name>`;
- establishes ownership by nesting/order, not by emitting owner IDs;
- emits field and method names, but not their IDs;
- emits parameter names/types, but not parameter IDs;
- uses `+N` only when a method name repeats under one owner; `N` is parameter count, not a stable method identity;
- flattens modifier, control-summary, pattern-fact, and legacy-flag occurrence groups;
- emits control/data flow, side effects, and execution contexts only at High fidelity;
- emits exact bodies only at Edit fidelity and never emits body byte spans;
- emits final `Pattern` operations as `P <name> <args...>`;
- does not read `HierarchicalIR.calls` or `ClassNode.injects` at all.

The call omission is deliberate current behavior: `TS-CALL25` asserts renderer output is byte-identical with or without call operations. The injection renderer test similarly asserts only that injection-bearing hierarchy renders without error. Both facts predate this research.

### Compact/hierarchical differences and channel caveat

`compress_code_context` returns three projections at once: renderer text in `content`, hierarchical wire in `ir`, and named/positional/tagged wire in `pretty`. By contrast, the preferred `provide_code_context` full path returns only renderer text to `content`; hierarchy is retained server-side and semantic edges are in `_meta`. These are materially different response shapes.

The server's JSON-RPC wire contains auxiliary fields, but model visibility is not established merely by wire presence. Under MCP's standard channel semantics, `_meta` is not the answer to the model. The repository's own MCP-001 invariant likewise identifies `content` as human/model-readable. Non-standard result siblings (`ir`, `delta`, `semantic_edges`) have no declared `outputSchema` on these legacy exceptions, so a portable client cannot be assumed to place them in model context. A host-level capture test is required before crediting any auxiliary field as LLM-visible.

## 2. Canonical-versus-production semantic-family matrix

In the table, “payload” means the normal High-fidelity `provide_code_context` full-path `content` text. Low and Medium omit additional High-only families. “ID” means stable canonical identity, not merely a display name.

| Semantic family | Canonical | Actual production payload | Exact current representation | Identity preserved? | Multiplicity/order preserved? | Required for reasoning? | Safe omission? | Future codec requirement |
|---|---:|---:|---|---|---|---|---|---|
| File | yes | yes | trailer plus `§PATHMAP` alias/path | yes, by path alias mapping | one response-local occurrence | yes | no | one file handle plus exact path map |
| Class | yes | yes | `// ── Name ──` | name only; no class ID | class order yes | yes | canonical spelling may be replaced by bijective local handle | distinct typed class handle |
| Interface | yes | yes | `// Q=interface`, then `Q Name` | name only; no interface ID | interface order yes | yes | no type-family collapse | distinct typed interface handle |
| Method | yes | yes | `M name` or `M name(+N)` | no method ID; `+N` is not identity | method order yes | yes | no; ID spelling itself may be elided if a bijection remains | unique method handle scoped to typed owner |
| Field | yes | yes | `F name:type` | no field ID | order yes | often | only when no question can address/compare fields; unsafe as global rule | unique field handle or lossless scoped ordinal |
| Parameter | yes | yes except Low non-overload | `p:name:type` | no parameter ID | order yes | yes for signatures/overloads | no in correctness-complete structural view | ordered ID/ordinal, name, type |
| Return type | yes | yes | `→ type` | attached by line position | one per method | yes | no | method-scoped value |
| Ownership IDs | yes | implicit | class/interface blocks and method line position | positional only | block order yes | essential | explicit parent repetition is safely elidable only after checked identity resolution | deterministic scope stack plus unambiguous handles |
| Class modifiers | yes, occurrence-grouped | yes | `cmod:` flattened | target by current class; groups lost | values/order/duplicates yes; group boundaries no | yes | group boundaries only if proven irrelevant | preserve typed values and required occurrence boundaries |
| Interface modifiers | yes, occurrence-grouped | yes | `imod:` flattened | target by current interface | same caveat | yes | same caveat | same as class modifiers |
| Method modifiers | yes, occurrence-grouped | yes | `mod:` flattened | target by current method line | same caveat | yes | same caveat | same as class modifiers |
| Extends | yes | yes | class-local `X parent` | child positional; parent textual/alias | one class parent | yes | no | typed child and honest target reference |
| Implements | yes | yes | class-local `I ref...` | child positional; targets textual/aliases | list order yes | yes | no | typed class→interface records |
| Interface extends | yes | yes | interface-local `X ref...` | context distinguishes it from class extends | list order yes | yes | no | typed interface→interface records |
| Injects | yes | **no** | absent from renderer | no | no | yes for DI reasoning | no | owner, ordered occurrence, dependency refs; retain framework provenance when available |
| Calls | yes | **no** | absent from renderer | caller/callee absent | no | yes for call reasoning | no | caller method ID, unresolved callee name/ref, ordered occurrence |
| Call arity | yes | **no** | absent; overload `+N` is declaration arity, not call evidence | no | no | yes | no | written argument count on each call occurrence |
| Call evidence | yes | **no** | semantic edge auxiliary only | not in content | no | yes to avoid false resolution claims | no | typed evidence separate from entity identity |
| Spread | yes | **no** | absent | no | no | yes because count ceases to be exact | no | explicit qualifier per call occurrence |
| Control summaries | yes, occurrence-grouped | yes | `ctl:` flattened | method positional | values/order/duplicates yes; groups lost | yes | not globally | preserve groups unless equivalence is proven |
| Pattern facts | yes, occurrence-grouped | yes | `pf:` flattened | method positional | values/order/duplicates yes; groups lost | yes | not globally | preserve typed facts and required grouping |
| Final pattern classification | yes | yes | `P name args...` before owner/member | args are untyped strings, sometimes IDs | pattern order yes | yes | no | typed class/method target plus pattern payload |
| Control flow | yes | High only | `cf:kind:target,...` | method positional | tuple order yes | often | intent-dependent, not globally | typed tuple records at required fidelity |
| Data flow | yes | High only | `df:direction:target,...` | method positional | tuple order yes | often | intent-dependent, not globally | typed tuple records at required fidelity |
| Side effects | yes | High only | `se:value,...` | method positional | order/duplicates yes | yes for side-effect questions | no in reasoning envelope | typed ordered values |
| Execution contexts | yes | High only | `ec:value,...` | method positional | order/duplicates yes | yes | no in reasoning envelope | typed ordered values |
| Imports | yes | yes | `$ alias module [named]` | import alias present | import order yes; named list is one string | language/task dependent | only with explicit intent proof | preserve alias/module/named distinctions |
| Type aliases | yes | yes | `T alias = original` | alias present | order yes | yes where types/meta use them | no | pair records; do not mix with unrelated meta facts |
| Bodies | Edit only, byte-exact | Edit only | raw body after method line | associated by preceding method | exact bytes/order | required for exact editing/deep source reasoning | yes in semantic mode; never in an edit request that needs it | separate verbatim segment keyed by method ID |
| Source spans | yes when Body carries them | **no** | absent from renderer | no | n/a | required by span-based editing, not ordinary reasoning | safe only when edit path reacquires authoritative spans server-side | non-LLM capability metadata or explicit edit receipt; never reconstruct from text |
| Provenance/asserting file | yes on semantic entities/state | partial | file footer; edge provenance absent | file yes, edge assertion no | edge occurrences unavailable | yes for cross-file/framework claims | no | file handle on each external assertion or inherited edge block |
| Framework/meta edges | yes in separate edge snapshot | not in `content` as a complete family | some facts may also appear as `T @...`; full edge array is `_meta.semantic_edges` | structured auxiliary identity is `(domain,type,name)`; content varies | `_meta` array preserves extraction order; WorkspaceIndex may deduplicate | yes | no | relation, typed subject/object, provenance layer/file, occurrence policy |

Two qualifications matter:

1. A canonical ID's literal spelling (`C1`, `M7`) is not irreducible. A codec may replace it with a shorter file-local ordinal if decoding/reasoning preserves a bijection and type family.
2. Canonical call facts do **not** claim resolved callee identity. They carry an exact caller method ID, the callee name as written, written argument count, and spread. A future codec must preserve that honest uncertainty rather than fabricate a declaration target.

## 3. Identity ambiguity findings

### Same-name methods and overloads

Block position lets a reader infer an enclosing class, but there is no stable method handle to cite. Two classes can each contain `save`; two same-owner overloads can both render `M find(+1)`. Parameters may visually distinguish the lines at Medium/High, but references elsewhere cannot point to one line unambiguously. Low hides parameters for unique names and shows them only for repeated names, making the identity grammar data-dependent.

### Same-name classes across files or namespaces

Each response has a file path, so a careful model can say “`Service` in α1,” but class references such as extends/implements and framework entity names are not consistently qualified by file or namespace. Concatenated multi-file context can therefore contain two visually identical class headers. The file boundary is necessary but not sufficient for relationship targets.

### Field and property collisions

Fields are name/type pairs under a visual owner. Repeated names under different owners are distinguishable only by nearby block position. No external fact can address `F1` because field IDs are dropped.

### Class/interface collisions

`Q` preserves the declaration family, so a class `Worker` and interface `Worker` remain distinguishable at their declarations. Relationship targets are still textual/alias values, so an untyped target spelling can be ambiguous unless the opcode/relation supplies the expected target kind.

### Calls

The hierarchy knows the exact caller ID but only an unresolved callee spelling. The LLM payload receives neither. Generic `SemanticEdge` projection resolves the caller ID to a **name** and defines entity identity as `(domain, entity_type, name)`, excluding file and owner. Consequently, two owners' `run` methods share the same projected generic method identity even though canonical IR distinguishes their caller IDs. This is safe for name-oriented discovery only; it is insufficient for exact caller ownership reasoning.

### Injections

Core injection facts retain the owning class ID, ordered operation occurrences, dependency ordering, and duplicates. The renderer drops the family. Some frameworks also produce typed `Injects`/`Autowired` edges with domain/type/name/provenance, but those are not a universal substitute: they are separate facts, may use a different identity model, and live outside `content` on the preferred path.

### Conclusion on current completeness

`CONTROL-PROD` is not correctness-complete for the requested call-chain, DI, exact-ownership, duplicate-occurrence, and framework-edge tasks. The contradiction does not prevent research design because `CONTROL-FULL` can be specified from existing checked state without choosing a production contract. It **does** block selecting or implementing a codec: first, maintainers must approve which required envelope becomes model-visible and through which MCP channel.

## 4. Representative semantic input

All formats below encode the same High-fidelity, body-free sample. It deliberately includes same-name overloads, distinct class/interface ownership, duplicate call occurrences, a spread call, DI, and a framework edge.

```text
file f1 alias α1 path /repo/orders.ts
class C1 OrdersController mods [[EXPORT]] extends BaseController implements [I1]
  inject occurrence 0 [C2]
  field F1 service:C2
  method M1 get; param P1 id:$s; returns Order; mods [[PUBLIC,ASYNC]]
    control [[IF,RET]]; facts [[OBSERVABLE]]; final pattern OBSERVABLE(C1,M1)
    cf [(if,missing),(return,Order)]; df [(reads,F1)]; se [io,async]; ec [async]
  method M2 get; param P2 ids:$s[]; returns Order[]; mods [[PUBLIC]]
class C2 OrdersService
  field F2 repo:Repository
  method M3 find; param P3 id:$s; returns Order; mods [[PUBLIC]]; se [io]
interface I1 OrderApi mods [[EXPORT]] extends [BaseApi]
  method M4 get; param P4 id:$s; returns Order
calls in order: (M1,find,1,exact), (M1,audit,1,spread), (M1,find,1,exact)
import (IM1,./models,Order); type alias (OrderId,$s)
meta edge: dotnet HasRoute Controller(OrdersController,f1) -> Route(GET /orders/{id},f1)
```

Bodies and spans are not present in this sample because they are intent-gated. Every candidate's schema requires an additive `BODY(method, exact bytes, start, end)` segment when Edit fidelity requests it. Measuring semantic mode without bodies is valid; claiming that bodies can be summarized in Edit mode is not.

## 5. Concrete representations

### CONTROL-PROD — exact current renderer family

This is the exact **shape** assembled by the current High renderer and file footer for the sample. Calls, injections, stable declaration IDs (apart from untyped pattern args), spans, and the framework edge do not appear.

<!-- TOKEN-SAMPLE:CONTROL-PROD -->
```text
// SCHEMA v5  @=meta X=extends I=implements F=field M=method $=import →=scope mod:=method-modifiers cmod:=class-modifiers ctl:=control-summary pf:=pattern-facts fl:=legacy-flags cl:=class-metadata P=pattern T=type-alias
// ── OrdersController ──
cmod: EXPORT
X BaseController
I I1
F service:C2
P OBSERVABLE C1 M1
M get(+1)  → p:id:$s → Order mod:PUBLIC,ASYNC ctl:IF,RET pf:OBSERVABLE cf:if:missing,return:Order df:reads:F1 se:io,async ec:async
M get(+1)  → p:ids:$s[] → Order[] mod:PUBLIC
// ── OrdersService ──
F repo:Repository
M find  → p:id:$s → Order mod:PUBLIC se:io
// Q=interface
Q OrderApi
imod: EXPORT
X BaseApi
M get  → p:id:$s → Order
$ IM1 ./models [Order]
T OrderId = $s
// ── α1 (/repo/orders.ts) ──
§PATHMAP
  α1 = /repo/orders.ts
```

The renderer itself would place a method pattern before its method; exact pattern availability depends on canonical instruction order/recognition. The sample fixes that order explicitly.

### CONTROL-FULL — correctness-complete named records

This is intentionally explicit and serves as the semantic oracle, not as a proposed production format.

<!-- TOKEN-SAMPLE:CONTROL-FULL -->
```text
schema: file(id,alias,path); class(id,name,modifier_occurrences,extends,implements); interface(id,name,modifier_occurrences,extends); field(id,owner,name,type); method(id,owner,name,modifier_occurrences,return); param(id,owner,name,type,ordinal); inject(owner,occurrence,deps); control(method,occurrence,values); facts(method,occurrence,values); pattern(kind,class,method,args); cf(method,ordinal,kind,target); df(method,ordinal,direction,target); side_effect(method,ordinal,value); execution(method,ordinal,value); call(ordinal,caller,callee_written,written_arg_count,spread); import(id,module,named); alias(name,type); edge(ordinal,relation,subject_domain,subject_type,subject_name,subject_file,object_domain,object_type,object_name,object_file,layer)
file id=f1 alias=α1 path=/repo/orders.ts
class id=C1 name=OrdersController modifier_occurrences=[[EXPORT]] extends=BaseController implements=[I1]
inject owner=C1 occurrence=0 deps=[C2]
field id=F1 owner=C1 name=service type=C2
method id=M1 owner=C1 name=get modifier_occurrences=[[PUBLIC,ASYNC]] return=Order
param id=P1 owner=M1 name=id type=$s ordinal=0
control method=M1 occurrence=0 values=[IF,RET]
facts method=M1 occurrence=0 values=[OBSERVABLE]
pattern kind=OBSERVABLE class=C1 method=M1 args=[]
cf method=M1 ordinal=0 kind=if target=missing
cf method=M1 ordinal=1 kind=return target=Order
df method=M1 ordinal=0 direction=reads target=F1
side_effect method=M1 ordinal=0 value=io
side_effect method=M1 ordinal=1 value=async
execution method=M1 ordinal=0 value=async
method id=M2 owner=C1 name=get modifier_occurrences=[[PUBLIC]] return=Order[]
param id=P2 owner=M2 name=ids type=$s[] ordinal=0
class id=C2 name=OrdersService modifier_occurrences=[] extends=- implements=[]
field id=F2 owner=C2 name=repo type=Repository
method id=M3 owner=C2 name=find modifier_occurrences=[[PUBLIC]] return=Order
param id=P3 owner=M3 name=id type=$s ordinal=0
side_effect method=M3 ordinal=0 value=io
interface id=I1 name=OrderApi modifier_occurrences=[[EXPORT]] extends=[BaseApi]
method id=M4 owner=I1 name=get modifier_occurrences=[] return=Order
param id=P4 owner=M4 name=id type=$s ordinal=0
call ordinal=0 caller=M1 callee_written=find written_arg_count=1 spread=false
call ordinal=1 caller=M1 callee_written=audit written_arg_count=1 spread=true
call ordinal=2 caller=M1 callee_written=find written_arg_count=1 spread=false
import id=IM1 module=./models named=Order
alias name=OrderId type=$s
edge ordinal=0 relation=HasRoute subject_domain=dotnet subject_type=Controller subject_name=OrdersController subject_file=f1 object_domain=dotnet object_type=Route object_name="GET /orders/{id}" object_file=f1 layer=dotnet
edit_fallback: BODY(method,byte_exact_source,start_byte,end_byte) is mandatory on demand
```

### COMPACT-A — conservative scoped text

Schema labels are declared once; IDs remain explicit; indentation supplies ownership but never creates identity.

<!-- TOKEN-SAMPLE:COMPACT-A -->
```text
S A1 f=id,alias,path c=id,name,mods,x,impl i=id,name,mods,x F=id,name,type M=id,name,mods,ret A=id,name,type J=occ,deps cs=occ,vals pf=occ,vals P=kind,target cf=ord,kind,target df=ord,dir,target se=ord,val ec=ord,val K=ord,caller,callee,argc,spread E=ord,rel,subj,obj,layer; BODY=id,start,end,bytes
f f1 α1 /repo/orders.ts
c C1 OrdersController [EXPORT] BaseController [I1]
  J 0 [C2]
  F F1 service C2
  M M1 get [PUBLIC,ASYNC] Order
    A P1 id $s
    cs 0 [IF,RET]; pf 0 [OBSERVABLE]; P OBSERVABLE M1
    cf 0 if missing; cf 1 return Order; df 0 reads F1
    se 0 io; se 1 async; ec 0 async
  M M2 get [PUBLIC] Order[]
    A P2 ids $s[]
c C2 OrdersService [] - []
  F F2 repo Repository
  M M3 find [PUBLIC] Order
    A P3 id $s; se 0 io
i I1 OrderApi [EXPORT] [BaseApi]
  M M4 get [] Order
    A P4 id $s
K 0 M1 find 1 0
K 1 M1 audit 1 1
K 2 M1 find 1 0
$ IM1 ./models Order
T OrderId $s
E 0 HasRoute dotnet/Controller/OrdersController@f1 dotnet/Route/"GET /orders/{id}"@f1 dotnet
```

Risk is low relative to the other candidates: the model still sees mnemonic family labels and explicit IDs. The main hypothesis is that one schema line plus scope-relative records beats repeated inline labels at corpus scale.

### COMPACT-B — columnar/grouped semantic tables

Each table declares its columns once. Owner IDs remain explicit across tables, and occurrence ordinals preserve duplicates and order.

<!-- TOKEN-SAMPLE:COMPACT-B -->
```text
S B1; f{id,alias,path}; c{id,name,mods,x,impl}; i{id,name,mods,x}; F{id,owner,name,type}; M{id,owner,name,mods,ret}; A{id,owner,name,type}; J{owner,occ,deps}; Z{family,owner,occ,values}; V{family,owner,ord,a,b}; K{ord,caller,callee,argc,spread}; E{ord,rel,sd,st,sn,sf,od,ot,on,of,layer}; BODY{id,start,end,bytes}
f|f1|α1|/repo/orders.ts
c|C1|OrdersController|EXPORT|BaseController|I1
c|C2|OrdersService|-|-|-
i|I1|OrderApi|EXPORT|BaseApi
F|F1|C1|service|C2
F|F2|C2|repo|Repository
M|M1|C1|get|PUBLIC+ASYNC|Order
M|M2|C1|get|PUBLIC|Order[]
M|M3|C2|find|PUBLIC|Order
M|M4|I1|get|-|Order
A|P1|M1|id|$s
A|P2|M2|ids|$s[]
A|P3|M3|id|$s
A|P4|M4|id|$s
J|C1|0|C2
Z|cs|M1|0|IF,RET
Z|pf|M1|0|OBSERVABLE
Z|P|M1|0|OBSERVABLE
V|cf|M1|0|if|missing
V|cf|M1|1|return|Order
V|df|M1|0|reads|F1
V|se|M1|0|io|-
V|se|M1|1|async|-
V|ec|M1|0|async|-
V|se|M3|0|io|-
K|0|M1|find|1|0
K|1|M1|audit|1|1
K|2|M1|find|1|0
$|IM1|./models|Order
T|OrderId|$s
E|0|HasRoute|dotnet|Controller|OrdersController|f1|dotnet|Route|GET /orders/{id}|f1|dotnet
```

This format should have low marginal entity/edge cost. Its chief reasoning risk is table hopping: answering one method question requires joining `M`, `A`, `Z`, `V`, and `K` by ID.

### ULTRA — dictionary-backed semantic bytecode text

This candidate is intentionally machine-oriented. Dictionary index is zero-based; record schemas are fixed by `U1`; `~` means absent; `*` means spread; repeated records remain repeated.

<!-- TOKEN-SAMPLE:ULTRA -->
```text
U1 f3 c5 i4 F4 M5 A4 J3 Z4 V5 K5 E11 B4
D|/repo/orders.ts|OrdersController|EXPORT|BaseController|OrderApi|BaseApi|OrdersService|service|$s|Order|get|PUBLIC|ASYNC|IF|RET|OBSERVABLE|missing|return|reads|io|async|ids|repo|Repository|find|audit|./models|OrderId|HasRoute|dotnet|Controller|Route|GET /orders/{id}
f|f1|α1|0
c|C1|1|2|3|I1
c|C2|6|~|~|~
i|I1|4|2|5
F|F1|C1|7|C2
F|F2|C2|22|23
M|M1|C1|10|11+12|9
M|M2|C1|10|11|9[]
M|M3|C2|24|11|9
M|M4|I1|10|~|9
A|P1|M1|id|8
A|P2|M2|21|8[]
A|P3|M3|id|8
A|P4|M4|id|8
J|C1|0|C2
Z|c|M1|0|13,14
Z|p|M1|0|15
Z|P|M1|0|15
V|c|M1|0|if|16
V|c|M1|1|17|9
V|d|M1|0|18|F1
V|s|M1|0|19|~
V|s|M1|1|20|~
V|e|M1|0|20|~
V|s|M3|0|19|~
K|0|M1|24|1|0
K|1|M1|25|1|*
K|2|M1|24|1|0
$|IM1|26|9
T|27|8
E|0|28|29|30|1|f1|29|31|32|f1|29
```

ULTRA tests the safe limit of “resolve by identity; render by position.” It retains typed handles and ordinals but substitutes dictionary positions for repeated values. It should be rejected if models confuse dictionary indexes, family codes, or joins even when deterministic decoding is perfect.

## 6. Token-cost anatomy and measurement plan

### Current fixed and marginal costs

Static inspection predicts the following dominant costs; actual corpus measurements must validate them:

- **Fixed per full response:** the long SCHEMA v5 legend, file boundary, `§PATHMAP`, and repeated path.
- **Per declaration:** repeated `F`, `M`, `p:`, `mod:`, `ctl:`, `pf:`, `cf:`, `df:`, `se:`, and `ec:` labels.
- **Repeated values:** types, method names, framework/domain names, and common semantic values.
- **Structural syntax:** comment rulers, arrows, commas, colons, brackets, and newlines; their cost is tokenizer-specific.
- **Graph density:** currently near-zero in `CONTROL-PROD` because calls/injections/edges are omitted. This is not a saving. `CONTROL-FULL` must establish the true marginal edge cost.
- **Source size:** skeleton cost follows entity/fact count more closely than source bytes; Edit cost is dominated by exact bodies.

The representative sample's exact o200k control anatomy is:

| Family | CONTROL-PROD | CONTROL-FULL | Why it changes |
|---|---:|---:|---|
| Schema/legends | 71 | 175 | FULL declares the complete oracle grammar; PROD includes SCHEMA v5 plus the interface legend |
| File identity/path | 28 | 14 | PROD repeats the path in the boundary and PATHMAP |
| Declarations/signatures/DI | 24 declarations + 10 fields + 109 method/behavior + 22 modifier/relation = 165 | 245 | FULL restores IDs, owners, parameter IDs, occurrence structure, and injection |
| Behavior facts | included in PROD's 109 method/behavior subtotal | 130 | FULL uses separate typed records and ordinals |
| Calls | 0 | 58 | omitted from PROD |
| Imports/type aliases | 15 | 22 | FULL uses explicit labels |
| Framework edge/provenance | 0 | 53 | omitted from model-readable PROD content |
| Edit fallback contract | 0 | 20 | FULL states the mandatory exact-body/span escape hatch |
| **Total** | **279** | **717** | totals use newline-preserving family slices and exactly match whole-payload counts |

### Measured sample results

Token counts below are populated by a read-only reproduction of the repository's `tiktoken-rs` BPE process using the vendored `cl100k_base` and `o200k_base` rank tables. They count only the marked code-block payload, not Markdown fences or prose. Claude and Llama values are not reported as actual tokens: the repository currently approximates Claude as cl100k × 1.0 and Llama 3 as o200k × 1.12, while the experiment should use model-native counters.

| Representation | Semantically complete? | cl100k tokens | o200k tokens | Reduction vs CONTROL-FULL (o200k) | Fixed schema/dictionary share |
|---|---:|---:|---:|---:|---:|
| CONTROL-PROD | no | 277 | 279 | invalid comparison | 66 / 23.7% |
| CONTROL-FULL | yes | 711 | 717 | baseline | 175 / 24.4% |
| COMPACT-A | yes by design | 388 | 389 | 45.7% | 106 / 27.2% |
| COMPACT-B | yes by design | 511 | 534 | 25.5% | 120 / 22.5% |
| ULTRA | yes by design | 483 | 504 | 29.7% | 114 / 22.6% |

On this sample, making the explicit named reference complete costs 438 o200k tokens over production (717 versus 279, +157.0%). That number is reported separately because it combines the missing semantic information with CONTROL-FULL's deliberately verbose oracle labels; it is not a lower bound on the cost of closing the gap. COMPACT-A is 110 tokens (39.4%) above deficient CONTROL-PROD while carrying the full designed envelope, and 328 tokens (45.7%) below correctness-complete CONTROL-FULL. Those are the valid directions of comparison.

The character result also demonstrates why visual minification is unreliable: COMPACT-A is 985 characters and 389 o200k tokens, while ULTRA is only 747 characters but 504 tokens. Punctuation, numeric dictionary lookups, and fragmented symbols erase the apparent byte advantage.

These one-sample counts are screening data, not a winner selection. Token counts must also be collected by semantic family, corpus size, entity count, edge count, and tokenizer. Schema share is measured by counting the schema/dictionary prefix separately; marginal cost is obtained by paired fixtures that add exactly one entity or edge. The local counter was checked against three published tiktoken examples (`antidisestablishmentarianism`, `2 + 2 = 4`, and `お誕生日おめでとう`) and reproduced the documented cl100k/o200k counts of 6/6, 7/7, and 9/8 respectively.

### Required corpus

Use checked canonical output from real Clean-CTX fixtures, serialized into a neutral oracle manifest before rendering candidates:

| Stratum | Required cases |
|---|---|
| Scale | tiny file; large class; many classes; dense graph; large exact body |
| Identity | same method under two owners; same-arity overloads; same class name in two files/namespaces; class/interface collision; repeated field/dependency names |
| Language | representative TypeScript, C#, Java, Rust |
| Semantics | modifiers; inheritance; interface extends/implements; DI; imports/aliases; control/pattern facts; final patterns; side effects; execution contexts |
| Occurrences | duplicate facts, duplicate calls, ordered calls, repeated injection occurrences |
| Calls | exact written arity; spread; unresolved external callee; several possible same-name targets; extension-call evidence |
| Framework | Angular/NgRx, Spring, and .NET edges with asserting-file/layer provenance |
| Cross-file | declarations and relationships spanning files; identical display names with distinct provenance |
| Editing | focused body; all bodies; UTF-8/CRLF; exact spans; request-more-source decision |
| Adversarial | every case where position/name coincidence suggests the wrong owner or target |

## 7. Semantic roundtrip methodology

1. Compile each fixture through the production checked pipeline into canonical `CompiledIR`, checked hierarchy, and complete semantic-edge snapshot.
2. Normalize these into a **test-only oracle manifest** that retains types, IDs, occurrence boundaries/order, duplicates, body bytes/spans, and edge provenance. This is not a new production model.
3. Encode the oracle with each candidate and decode back to the same manifest.
4. Require exact equality, not set equality. Compare ordered arrays and duplicate occurrences. For bodies compare bytes and start/end spans.
5. Validate reference integrity: every owner/caller/field/parameter handle resolves to exactly one entity of the required kind. Unresolved callees must remain explicitly unresolved.
6. Mutate one dimension at a time to measure marginal tokens: entity, edge, repeated string, duplicate occurrence, path, and body byte length.
7. Reject malformed/truncated candidate streams deterministically. Explicit row counts/checksums are worth testing but are not semantic substitutes.

Lossless recovery is necessary, not sufficient.

## 8. Reasoning-fidelity methodology

### Tasks

Build deterministic questions from the oracle, with exact expected answers and an abstention class for facts not canonically resolved:

- identify class/interface/method/field/parameter ownership by stable handle;
- distinguish same-name entities and same-arity overloads;
- trace calls in occurrence order and report written arity/spread;
- refuse to claim a resolved callee when only a written name is known;
- list callers/callees at the level actually supported by the facts;
- trace inheritance, interface extends, implements, and injections;
- identify modifiers, control summaries, pattern facts/classification, side effects, and execution contexts;
- preserve duplicate occurrences instead of deduplicating them;
- reason across files using provenance and distinguish equal display names;
- select the exact edit target or request the required body/source when compact facts are insufficient.

### Experimental controls

- Same model/version, system prompt, question order randomization, temperature, and output schema across formats.
- Blind format labels; include only the candidate's schema/legend and encoded data.
- Multiple runs per item for stochastic models; paired comparison against CONTROL-FULL.
- Separate syntax-learning warm-up from scored tasks. Measure both cold schema cost and amortized schema-known sessions, but never assume conversation memory unless the API contract guarantees it.
- Score exact answer, unsupported-claim rate, ownership errors, occurrence/order errors, and body-request correctness.
- Set non-inferiority margins **per task family**, with zero tolerance for identity fabrication, wrong edit target, or source corruption. Token savings never offset a failed family.

The candidate passes only if deterministic recovery is exact and reasoning is statistically non-inferior to CONTROL-FULL with no correctness-critical errors.

## 9. Technique assessment

| Technique | Hypothesis | Main risk | Priority |
|---|---|---|---:|
| One versioned schema per payload | removes repeated labels cheaply | cold schema overhead; version drift | 1 |
| Scoped positional ownership | safely removes repeated owner IDs after checked resolution | scope loss after truncation/reordering | 1 |
| Columnar family tables | lowers marginal fact/edge cost | multi-table joins degrade reasoning | 2 |
| Frequency-ranked dictionaries | helps repeated paths/types/names | index lookup cost; bad on small inputs | 2 |
| Short mnemonic opcodes | can be tokenizer-cheap and learnable | visually short may tokenize poorly | 1 |
| Punctuation/delimiters | replaces English labels | tokenizer variance and parse ambiguity | 2 |
| Hybrid layout | declarations scoped, graphs columnar, bodies verbatim | multiple grammars increase learning cost | 1 |
| File/scope-local dictionaries | lower overhead than global maps on small payloads | repeated cross-file values | 2 |
| Differential context | avoids resending stable schema/maps | stale or non-authoritative conversation state | 3 |
| TOON-like tabular rows | proven design pattern for uniform records | irregular nested semantic facts | 2 |
| Grammar-constrained decoder | prevents malformed codec streams | does not prove model reasoning | 2 |
| Learned/destructive prompt compression | large apparent savings | non-zero loss and unverifiable omissions | reject |

## 10. Risks and ambiguity traps

- Treating `_meta` or non-standard result siblings as model context without a host trace.
- Measuring CONTROL-PROD as though absent calls/injections/edges cost zero.
- Using parameter count as overload identity.
- Replacing unresolved callee names with guessed declaration IDs.
- Flattening occurrence groups or deduplicating repeated facts.
- Letting a dictionary index's position become semantic identity rather than a reversible rendering handle.
- Reusing one symbol for class and interface handles without a type tag or typed table.
- Dropping file/layer provenance because entity equality deliberately excludes file.
- Optimizing characters instead of actual target-model tokens.
- Amortizing a schema/dictionary across turns when the API/client does not guarantee its presence.
- Putting exact bodies through a lossy summarizer or reconstructing byte spans from rendered text.
- Comparing candidates at different fidelity levels.
- Reporting aggregate accuracy that hides a catastrophic identity/edit failure.

## 11. Recommended experimental sequence

1. **Transport visibility probe.** Capture what at least one real MCP host places in model context for `content`, `structuredContent`, `_meta`, and legacy extra result siblings. This resolves the current auxiliary-channel uncertainty without changing production.
2. **Oracle extractor.** In a research harness, export checked IR + hierarchy + semantic edges into the exact comparison manifest. Do not alter canonical production types.
3. **CONTROL-PROD corpus capture.** Record exact `result.content` for full, raw fallback, Edit, delta summary, restore, and replay paths. Break down SCHEMA, declarations, footer/path, and bodies.
4. **CONTROL-FULL reference encoder.** Encode the approved required envelope with explicit named records and prove exact roundtrip.
5. **COMPACT-A first.** Test schema elision and scoped ownership with explicit IDs. This isolates low-risk savings and provides the best chance of baseline-equivalent reasoning.
6. **Opcode micro-benchmark.** Exhaustively tokenize candidate markers/words under cl100k, o200k, native Claude, and at least one Llama tokenizer before fixing an alphabet.
7. **COMPACT-B.** Add columnar graph/fact tables; focus scoring on cross-table joins and duplicates.
8. **Dictionary threshold sweep.** Determine break-even points by file size, repetition, and graph density; allow “no dictionary” below threshold.
9. **ULTRA last.** Run only after the reasoning suite is sensitive enough to catch ownership, unresolved-target, and edit-request errors.
10. **Differential experiment.** Evaluate schema/dictionary reuse only on APIs/hosts with guaranteed prior-context continuity and explicit version/hash acknowledgement.
11. **Decision gate.** Reject every candidate below CONTROL-FULL fidelity. Among survivors, compare model tokens and complexity. “Current format is near the frontier” remains a valid outcome.

## 12. First hypotheses to test

1. Removing the repeated SCHEMA v5 prose and declaring a smaller versioned schema will produce immediate fixed savings without reasoning loss.
2. Explicit file/class/method IDs plus scoped positional ownership can be **more correct and still cheaper** than CONTROL-PROD once calls/DI/edges are added.
3. COMPACT-A will retain reasoning better than columnar formats and may capture most safe savings.
4. Columnar encoding will win on dense graphs but may lose on entity-centric questions because of joins.
5. Dictionaries will lose on tiny files and win only after a tokenizer-specific repetition threshold.
6. ASCII mnemonic opcodes will often beat unusual Unicode glyphs despite longer character length.
7. Full-body fallback will dominate Edit token cost; optimizing the semantic prefix matters far less there than focused body selection.
8. A hybrid—scoped declarations, columnar edges, separate exact bodies—will dominate one universal encoding.

## 13. Explicitly rejected techniques

- **Semantic omission:** invalid by definition, including preserving current omissions.
- **Perplexity/token deletion:** LLMLingua-style approaches allow quality loss; Clean-CTX's gate does not.
- **Summary in place of required body/source:** violates exact edit fidelity.
- **Name-only identity:** fails same-name owners, overload references, and cross-file cases.
- **Position as canonical identity:** truncation/reordering would change meaning. Position is permitted only as a reversible rendering handle after internal resolution.
- **Unversioned shared dictionary/schema:** a stale map silently changes meaning.
- **Opaque binary/base64 to the model:** mechanically compact in bytes but generally token-expensive and hostile to reasoning; binary remains appropriate for storage, not assumed LLM input.
- **One global dictionary for every response:** fixed overhead and lifecycle coupling can exceed savings; test scoped/adaptive variants.
- **Weighted “overall” score:** forbidden because token savings cannot compensate for correctness loss.

## 14. Approved production-mode envelope

The architecture gate is approved for the correctness repair only. Portable MCP `content` is authoritative; `_meta` remains application-facing. Production behavior classes come from the implemented mappings: overview→Low, debug/implement→Medium, refactor→High, edit→Edit, explicit fidelity overrides intent, and Verbatim is raw source. Focus is meaningful only at Edit and resolves through typed owner identity to canonical method IDs; ambiguous bare/qualified selectors fail rather than guessing. Structured modes never substitute a cheaper semantically incomplete raw payload for CONTROL-FULL. Exact-source escalation is explicit: Edit supplies all or resolved focused bodies; Verbatim supplies the document.

The benchmark corpus must exercise full provide/compress, selected and all-body Edit, delta with acknowledged baseline, apply, durable restore/replay, and multi-file semantic-edge provenance. Rows are recorded separately; no aggregate may conceal a mode failure. Token cells remain pending until the user-owned verification/baseline run because this research phase does not authorize the agent to run Cargo or binaries.

| Fidelity | Intent | Focus mode | Production path | Required semantic families | Body behavior | CONTROL-PROD tokens | CONTROL-FULL tokens | Candidate tokens | Reasoning result |
|---|---|---|---|---|---|---:|---:|---:|---|
| Low | overview | ignored | provide full | canonical Low hierarchy + calls + injections + all edges/provenance | none; request Edit/Verbatim when needed | pending | pending | n/a | pending user run |
| Medium | debug/implement | ignored | provide full | canonical Medium hierarchy + calls + injections + all edges/provenance | none; request Edit/Verbatim when needed | pending | pending | n/a | pending user run |
| High | refactor | ignored | provide/compress full | complete compiled reasoning envelope, IDs, groups, calls, DI/framework/generic edges | none; request Edit/Verbatim when needed | pending | pending | n/a | pending user run |
| Edit | edit | omitted | provide full | High structural envelope plus every exact method body/span | all method bodies exact | pending | pending | n/a | pending user run |
| Edit | edit | one/many typed selectors | provide full | same envelope; focus resolves to canonical method IDs | only resolved target set exact; overload family retained | pending | pending | n/a | pending user run; rejection cases are transport-only |
| Verbatim | any/explicit | ignored | provide/compress raw | exact source document; no compressed semantic claim | entire document exact | pending | n/a | n/a | excluded from semantic reasoning baseline |
| Low–Edit | mapped/explicit | inherited | provide/delta/apply | exact delta plus complete post-apply edge snapshot; acknowledged prior state required | follows effective fidelity/focus | pending | pending | n/a | pending user run |
| persisted fidelity | n/a | persisted | restore/replay | regenerate normalized CONTROL-FULL from durable checked IR + edges, render economical A1 or exact raw, never trust stale compact text | follows persisted body facts | pending | pending | pending | pending user run |

Registered-path verification, token capture, and the corrected 36-case reasoning run completed. Follow-up diagnostics found that the required DI provenance and occurrence facts were present and deterministically correct, while their organization reduced model reasoning reliability. The approved CONTROL-FULL v2 repair added stable typed navigation descriptors without altering canonical facts. COMPACT-A1 passed deterministic roundtrip, bounded reasoning, and production edge-case gates, but its initial 77% token result used verbose CONTROL-FULL as the denominator rather than raw source. That result proves oracle-encoding reduction only and is not production savings evidence. The corrected harness preserves capture-time raw bytes, measures raw-to-A1 economics, and runs paired raw/A1 reasoning only after the candidate clears that screen. Full snapshots retain the local raw ceiling; focused Edit forbids full-document raw fallback because it violates the requested body-disclosure boundary. CONTROL-FULL-DELTA v2 remains the acknowledged-state delta representation.

## 15. Current measurement checkpoint (2026-09-24)

The presentation boundary (ARCH-003) is implemented and mechanically enforced:
`content` is the SCHEMA-v5 presentation on every path, the reversible codec is
code-side (`result.ir` + persistence), and `content` falls back to byte-exact
raw source when the presentation is not safely cheaper under the local tokenizer
estimate — i.e. for small files at structural fidelity and for Edit all-bodies.
The measurement harness now splits into `codec/` (reversible wire) and
`schema-v5/` (model-visible presentation) under `verification/context-compression/`.

### Measured SCHEMA-v5 density (raw → presentation, o200k; cl100k is within ~1%)

| Fixture (language) | Low | Medium | High |
|---|---:|---:|---:|
| LargeService.ts (typescript) | 76.3% | 70.1% | 67.1% |
| UserManagementService.ts (angular) | 73.9% | 64.6% | 61.8% |
| OrderManagementService.cs (csharp) | 64.2% | 48.4% | 43.9% |

Density is **language-dependent**, and it is the high-fidelity annotation load,
not the schema, that drives the gap. C# is lowest because its idiom is
structurally verbose: every async method carries a
`CancellationToken cancellationToken = default` parameter and a
`Task<ActionResult<T>>` return plus `[FromBody]`/`[FromQuery]` attributes; at
High fidelity each async method is annotated
`mod:ASYNC ctl:… pf:OBSERVABLE cf:await… se:io`; and an
`IOrderService` interface mirrors the concrete service. This is a faithful
encoding of real facts, not a renderer defect — the gap is narrow at Low and
widens at High precisely where those per-method annotations land.

### Compression vs. accuracy

The High-fidelity annotations are the **reasoning payload**: they replace the
body. Cutting them to close the C# gap would remove the facts the model reasons
with, trading accuracy for size. The legitimate lever is collapsing *redundancy*,
not dropping facts. The first such collapse — the `async` triplicate
(`mod:ASYNC` + `se:async` + `ec:async`, all co-derived from the `async` keyword)
— is implemented and measured: it lifted High-fidelity density by **+1.6pp
(typescript), +1.3pp (angular), +2.6pp (csharp)** while preserving every fact —
the codec still stores all three, and the presentation reports `async` once at
the most structural level available (`mod:ASYNC`, else `se:async`). Whether a
further compressed encoding still preserves reasoning remains an empirical
question, answered by measuring size and task accuracy together, never size
alone.

### Task-based reasoning evaluation

A task-based edit evaluation replaces the comprehension quiz (which, on small
fixtures, measured raw-source reading rather than SCHEMA-v5): 3 edit tasks on
the large fixtures, graded deterministically (no LLM judge) by target resolution
plus find/replace checks, with an `apply_edit` round-trip as the byte-exact gate.
Initial result: **3/3 tasks** — the model targeted the correct method, reproduced
the focused body byte-exactly (modulo CRLF→LF line endings), and produced the
correct edit; the `apply_edit` round-trip (v2 grader) also passed **3/3**,
confirming the line-ending normalization is immaterial (`apply_edit` is
EOL-preserving and accepts the LF-normalized bodies against CRLF source). This
harness is the accuracy guardrail for any future
encoding-compression experiment: change the annotation set, then re-measure size
*and* task pass rate together.
