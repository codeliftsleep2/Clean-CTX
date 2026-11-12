# Conditional exact enrolled-evidence MCP transport

Agent: **Agent MaxHeadRoom**. Current normative owner for the source propositions below, normalized in B3. IMPLEMENTED denotes an operative, source-described bounded contract, not independent validation or authenticated approval.

## Scoped standing and original numbering

C02 is reconciled through the source-attributed stage chain: [M2a-1](../../agent/PCG_175_M2A1_IMPLEMENTATION.md) explicitly attributes authorization to the corrected design for enrollment/persistence only; [M2a-2](../../agent/PCG_175_M2A2_IMPLEMENTATION.md) records its subsequent acceptance; [M3](../../agent/PCG_175_M3_IMPLEMENTATION.md) records separate projection authorization and verification; [M4a](../../agent/PCG_175_M4A_IMPLEMENTATION.md) records M3 acceptance and separate MCP authorization; [M4b reconciliation](../../agent/PCG_175_M4B_SCOPE_RECONCILIATION.md) explicitly attributes acceptance of all four named stages. The original design's withheld-approval banner remains historical and intact. These records support the operative bounded v1 contract, not blanket approval of every original proposal or independently certified execution.

Original numbered sections remain locatable through the design document's headings and B3_PROVENANCE.csv. Sections 2–6 and trusted-local security are owned by enrolled v1; §4.3 and §7 by acquisition; §8.1/8.3/8.4 by projection; §8.2 and the implemented MCP boundary by evidence MCP. Cross-section numbers retain those original meanings. Acceptance scenarios, pilot data and staged verification history remain evidence/design history, not new normative owners. Proposed oracle names are specifications, not a claim that B3 authored or executed those tests. Runtime approval, autonomous extraction/injection, embeddings, tenancy and the other original deferrals remain unapproved by these bounded stages.

### 8.2 Proposed request schema

Draft 2020-12; `additionalProperties:false` applies at every object. The exact operation condition is enforced by the `oneOf`: constraints uses only subject; closure/impact/provenance also require records. Generation, policy, corpus IDs and references are never inferred from a local namespace. Byte paths are repo-relative and validated beyond JSON schema as §3 specifies.

```json
{
 "$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,
 "required":["workspaceRoot","corpus","generation","mode","subject","tokenizer","limits","includeHistory"],
 "properties":{
  "workspaceRoot":{"type":"string","minLength":1},"corpus":{"$ref":"#/$defs/id"},"generation":{"type":"string","pattern":"^[0-9a-f]{64}$"},
  "mode":{"enum":["constraints","closure","impact","provenance"]},
  "subject":{"type":"object","additionalProperties":false,"required":["workspace","at"],"properties":{
   "workspace":{"$ref":"#/$defs/id"},"file":{"type":"string","minLength":1},"operation":{"$ref":"#/$defs/id"},"task":{"$ref":"#/$defs/id"},"at":{"type":"string","pattern":"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"},
   "entity":{"type":"object","additionalProperties":false,"required":["kind","file","source_sha256","start","end","name","declaration_kind"],"properties":{
    "kind":{"const":"declaration"},"file":{"type":"string","minLength":1},"source_sha256":{"type":"string","pattern":"^[0-9a-f]{64}$"},"start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":1},"name":{"type":"string","minLength":1},"declaration_kind":{"type":"string","minLength":1}
   }}
  }},
  "records":{"type":"array","minItems":1,"uniqueItems":true,"items":{"$ref":"#/$defs/id"}},
  "tokenizer":{"enum":["cl100k","o200k"]},"includeHistory":{"type":"boolean"},
  "limits":{"type":"object","additionalProperties":false,"required":["records","edges","evidence_bytes","token_budget"],"properties":{
   "records":{"type":"integer","minimum":1},"edges":{"type":"integer","minimum":1},"evidence_bytes":{"type":"integer","minimum":1},"token_budget":{"type":"integer","minimum":1}
  }}
 },
 "oneOf":[{"properties":{"mode":{"const":"constraints"}},"not":{"required":["records"]}},{"properties":{"mode":{"enum":["closure","impact","provenance"]}},"required":["records"]}],
 "$defs":{"id":{"type":"string","pattern":"^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$"}}
}
```

Semantic request validation: real UTC calendar/time, `end>start`, entity/file consistency, unique IDs, configured limits and profile root; no caller-supplied corpus/partition expansion. `includeHistory:true` with history disallowed is a generic permission error, not silently ignored. Unknown record IDs return `incomplete/record_not_enrolled` after authorized scope validation, not complete emptiness.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `8f05a849a48333401795347f660cac7204917b2e39309f746228b8ae0b256c47`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 2. Production lifecycle and exact-byte boundary

```text
owner configures CLEAN_CTX_PCG_PROFILE before server startup
  -> McpState -> PcgState -> pinned Trusted profile witness
  -> initialize / tools/list advertises pcg_query_v1 when configured
  -> stdin JsonRpcRequest -> Dispatcher -> router::dispatch
  -> tools::dispatch_tools_call -> enabled handler registry -> pcg::handle
  -> closed typed arguments -> Session::project -> existing M3 projection
  -> existing acquisition/store/closure/packing/COMPLETION observations
  -> protocol::send_pcg_result adds JSON-RPC framing only
  -> existing shared stdout mutex -> UTF-8 line + flush -> MCP client
```

The environment variable is read once for the session. An omitted variable
disables PCG: the legacy catalog and registry remain unchanged and direct PCG
invocation returns the existing tool-not-found error. A configured profile adds
exactly one descriptor and handler. The enabled registry is the existing default
registry plus that handler; all original handlers remain the same functions.
The profile path cannot be selected or replaced by a tool argument.

The session retains the existing `Trusted` byte/permission witness and rechecks
it around projection. Replacing/revoking it does not silently establish a new
principal: the request fails closed and restart is needed. Startup reads the
profile only; projection DB open/header checks remain lazy and read-only on an
explicit query. No import, migration, repair, source hydration, graph extraction,
source publication, pending persistence flush or context injection is added.

Ordinary `send_response` still serializes its existing `Value` responses normally.
The PCG-only sender copies `Response.serialized_result` directly into JSON-RPC's
`result` field. It serializes the request ID, never the metered result. It adds
no cache hints, metadata, whitespace, `structuredContent`, duplicate text block
or result re-escaping. Both paths use the same stdout lock in the production
binary. JSON-RPC framing and its newline are outside the design's result budget.

Test capture of serialized lines is thread-local and opt-in, then drained and
disabled after each call. Ordinary test suites do not accumulate a second raw
response history. Existing parsed-response capture semantics remain intact.


Source: [docs/agent/PCG_175_M4A_IMPLEMENTATION.md](../../agent/PCG_175_M4A_IMPLEMENTATION.md), commit `187d0096`, original section SHA-256 `7d8a186047eb25e433fe945fc13bc64e7feafa5670f11545b2c4da2c780f9115`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 4. MCP request and response contract

Tool: **`pcg_query_v1`**, explicit read only, single request (no new batch mode).
Its `inputSchema` is Draft 2020-12, closed at every object. No `outputSchema` is
advertised because the approved response has one text block, not structuredContent.

Required arguments: `workspaceRoot`, `corpus`, exact 64-hex `generation`, `mode`,
`subject`, `tokenizer`, `limits`, `includeHistory`. Modes are `constraints`,
`closure`, `impact`, `provenance`. Constraints rejects `records`; the other modes
require a nonempty unique record-ID array. Subject requires workspace and explicit
UTC effective time, with optional exact file/entity/operation/task selectors.
Scope bindings remain enrolled source assertions; callers cannot submit bindings,
partitions, principals, profile paths or approval flags. Entity-shaped requests
without source-owned proof retain `incomplete/identity_unavailable`; there is no
invented identity or hydration fallback.

Limits are positive `records`, `edges`, `evidence_bytes` and `token_budget`, bounded
by the trusted profile. Only `cl100k` and `o200k` are supported. Typed decoding
rejects unknown/missing members, inappropriate nulls, unsupported modes, fractions
and invalid argument shapes; existing acquisition validation enforces identities,
calendar time, paths, exact scope, selectors and policy. There is no fidelity,
tokenizer default or raw-text fallback borrowed from source-facing tools.

Success is the exact M3 result:

```json
{"content":[{"type":"text","text":"<canonical PCG v1 payload JSON>"}],"isError":false}
```

The JSON-RPC result payload retains M3 states `complete_within_registered_scope`,
`incomplete`, `blocked` and `unavailable`. Internal acquisition Ready corresponds
to a complete result only after M3 packing/fit and COMPLETION checks. No incomplete,
blocked or unavailable response gains partial actionable required authority.
Required overflow remains blocked with an empty authority bundle; a result whose
minimal control cannot fit becomes `-32602/token_budget_too_small` instead.

Fixed errors: unsupported tokenizer -> `-32602/unsupported_tokenizer`; malformed,
unauthorized scope/policy or invalid request -> `-32602/pcg_invalid_request`;
tokenizer/control/trusted-profile initialization or session-pin failure ->
`-32603/pcg_control_unavailable`. These protocol errors contain no evidence result
or source detail and are outside the evidence-result budget. A configured,
valid-profile but unavailable DB keeps the tool discoverable and returns a
metered unavailable result. An unreadable/invalid or changed trusted profile
cannot establish scope and returns the fixed control error instead of evidence.


Source: [docs/agent/PCG_175_M4A_IMPLEMENTATION.md](../../agent/PCG_175_M4A_IMPLEMENTATION.md), commit `187d0096`, original section SHA-256 `07afdfcfbf6fce62f0997d0b78a14af9d063d5df9ddaef3ac9aa3ddb24cf818f`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.


```architecture-graph
node: AUTH.PCG_MCP_V1
kind: AUTHORITY
status: IMPLEMENTED
title: Conditional exact enrolled-evidence MCP transport
track: REPOSITORY
artifact: docs/architecture/contracts/pcg-mcp-v1.md
depends_on:
  - AUTH.PCG_PROJECTION_V1
note: Operative source-described contract; status does not authenticate approval or certify implementation.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
