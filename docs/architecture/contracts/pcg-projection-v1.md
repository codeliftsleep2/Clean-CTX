# Required evidence and exact token projection

Agent: **Agent MaxHeadRoom**. Current normative owner for the source propositions below, normalized in B3. IMPLEMENTED denotes an operative, source-described bounded contract, not independent validation or authenticated approval.

## Scoped standing and original numbering

C02 is reconciled through the source-attributed stage chain: [M2a-1](../../agent/PCG_175_M2A1_IMPLEMENTATION.md) explicitly attributes authorization to the corrected design for enrollment/persistence only; [M2a-2](../../agent/PCG_175_M2A2_IMPLEMENTATION.md) records its subsequent acceptance; [M3](../../agent/PCG_175_M3_IMPLEMENTATION.md) records separate projection authorization and verification; [M4a](../../agent/PCG_175_M4A_IMPLEMENTATION.md) records M3 acceptance and separate MCP authorization; [M4b reconciliation](../../agent/PCG_175_M4B_SCOPE_RECONCILIATION.md) explicitly attributes acceptance of all four named stages. The original design's withheld-approval banner remains historical and intact. These records support the operative bounded v1 contract, not blanket approval of every original proposal or independently certified execution.

Original numbered sections remain locatable through the design document's headings and B3_PROVENANCE.csv. Sections 2–6 and trusted-local security are owned by enrolled v1; §4.3 and §7 by acquisition; §8.1/8.3/8.4 by projection; §8.2 and the implemented MCP boundary by evidence MCP. Cross-section numbers retain those original meanings. Acceptance scenarios, pilot data and staged verification history remain evidence/design history, not new normative owners. Proposed oracle names are specifications, not a claim that B3 authored or executed those tests. Runtime approval, autonomous extraction/injection, embeddings, tenancy and the other original deferrals remain unapproved by these bounded stages.

### 8.1 Public choice and token contract

| Interface | Candidate source fit / risk | Decision |
| --- | --- | --- |
| Evidence mode in `workspace_query` | Closed source operation enum, source scope/coverage schema and batch hydration/publication path would need non-source branches. Metadata resembling source completeness could mislead consumers. | Do not extend in v1. |
| Dedicated `pcg_query_v1` | One separately versioned read contract; no source hydration, explicit corpus generation and distinct coverage/authority semantics. Requires normal catalog/registry/schema/annotation parity. | **SELECT.** |

Annotations: read-only, non-destructive, idempotent, closed-world hint for the enrolled local corpus; hints are not authorization. Tool accepts no writes/approval/import/maintenance operation. Existing tool requests/responses, dictionary/cache hints, default compression, canonical IR and source SQLite stay unchanged. Only the enabled new catalog entry is added; when PCG is disabled there is no entry. A configured but unavailable projection leaves the entry discoverable and returns unavailable. No automatic context injection or source-facing footers are added.

Required payload is exactly §4.3's mandatory closure: selected controlling constraints/decisions, complete cited directive spans/conditions/exceptions, required proof/counterevidence/provenance, blockers/conflicts, authority/verification/lifecycle/freshness and coverage. Reuse one citation text body per immutable evidence revision. Optional refs/descriptive links/history cannot enlarge that mandatory set implicitly. Record omitted optional categories through `budget.optional_omissions`; `optional_truncated` marks actual known optional packing/work omissions, not a changed mandatory completeness class. Withheld/unrequested history is reported as a category policy, regardless of whether hidden records exist; never disclose their IDs/counts. Source spans must include complete conditions; unverified LLM summaries cannot replace them.

Order: required roots then required dependencies/counterevidence by logical ID; edges by `(kind,from,to,id)`; citations by `(source-id,source-revision,start,end)`; optional support then history by logical ID/revision. No model ranking. Escape source excerpts as JSON data and label them untrusted evidence; never concatenate them into system/tool instructions.

Response uses one MCP text block containing canonical JSON payload; no duplicated `structuredContent`. Publish `inputSchema`; do not advertise MCP `outputSchema` requiring a structured channel absent from this v1 contract. The documented payload schema below governs the JSON inside the text block. Normal tool result is exactly `{"content":[{"type":"text","text":PAYLOAD_JSON}],"isError":false}`. JSON-RPC framing/request ID is not context material; **the entire serialized MCP result**, including escaped payload JSON, wrapper, all metadata/citations and isError, is counted with exact local `cl100k` or `o200k`. Approximate tokenizers and tokenizer initialization failure return a control error; no raw/source fallback.

Budget limit and tokenizer appear in the payload; actual used count is deliberately not inserted into the same counted object (avoids self-referential accounting). Tests/operators count the emitted result independently. Drop optional complete items, remeter the final result, never slice mandatory text. If required bundle does not fit, return `blocked/required_budget_exceeded`, `required_material_fit:false`, coverage and bounded permitted IDs/diagnostics; `required_records` may be empty and the bundle is explicitly undelivered. Diagnostic ID lists may truncate with `diagnostics_truncated:true`; never truncate the required bundle and call it delivered. If even the minimal control result cannot fit, return `-32602/token_budget_too_small` outside the evidence-result budget; this protocol error is no usable context or completeness claim. No pagination/resumption in v1.

Under EVIDENCE-01/02, undelivered mandatory closure leaves `required_records` and its required relationships empty rather than emitting partial actionable authority. Safe diagnostic IDs may describe the deficit. Optional omissions use unique sorted category codes `not_requested`, `budget`, `unavailable`, `work_limit`, with no hidden-source identifier/count. Required omission instead changes result state/coverage/fit; it is never disguised as an optional omission. The count includes the added omission/assurance fields and any reclassified control result.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `de275cf370456c03ef07f87fa77741c0ef62238674329791d15865f5566a7a1b`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

### 8.3 Proposed payload schema

The following schema defines all payload fields and closed record/citation/issue shapes. `required_records` and `support/history` carry typed record objects, not executable instructions. A canonical empty `required_records` never alone establishes complete state. Evidence text occurs once in `citations`; each required record cites its primary and required reference IDs. All arrays are present, even when empty; no null array masquerades as not-checked.

```json
{
 "$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","additionalProperties":false,
 "required":["version","scope","generation","authority_policy","subjects","required_records","support","history","relationships","conflicts","missing_stale_requirements","coverage","budget","citations","result_state","reasons"],
 "properties":{
  "version":{"const":1},"scope":{"type":"object","additionalProperties":false,"required":["partition","corpus","workspace"],"properties":{"partition":{"type":"string"},"corpus":{"type":"string"},"workspace":{"type":"string"}}},
  "generation":{"type":["string","null"]},"authority_policy":{"type":"string"},"subjects":{"type":"array","items":{"type":"string"}},
  "required_records":{"$ref":"#/$defs/records"},"support":{"$ref":"#/$defs/records"},"history":{"$ref":"#/$defs/records"},
  "relationships":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","revision","kind","from","to","citation"],"properties":{"id":{"type":"string"},"revision":{"type":"string"},"kind":{"enum":["DEPENDS_ON","BLOCKED_BY","CONSTRAINS","USES_EVIDENCE","VALIDATES","FALSIFIES","SUPERSEDES","PRESERVES","IMPLEMENTS"]},"from":{"type":"string"},"to":{"type":"string"},"citation":{"type":"string"}}}},
  "conflicts":{"$ref":"#/$defs/issues"},"missing_stale_requirements":{"$ref":"#/$defs/issues"},"reasons":{"$ref":"#/$defs/issues"},
  "coverage":{"type":"object","additionalProperties":false,"required":["enrollment","identity","applicability","required_closure","evidence","currentness_boundary","freshness_guarantee","continuous_currentness","atomic_filesystem_snapshot","observations","projection_integrity"],"properties":{
   "enrollment":{"type":"boolean"},"identity":{"type":"boolean"},"applicability":{"type":"boolean"},"required_closure":{"type":"boolean"},"evidence":{"type":"boolean"},"currentness_boundary":{"const":"pinned_revision_two_observations"},
   "freshness_guarantee":{"const":"observed_boundaries_only"},"continuous_currentness":{"const":false},"atomic_filesystem_snapshot":{"const":false},"projection_integrity":{"const":"admission_and_accessed_rows"},
   "observations":{"type":"object","additionalProperties":false,"required":["working_start","working_completion"],"properties":{"working_start":{"enum":["matched","failed","incomplete","not_checked"]},"working_completion":{"enum":["matched","failed","incomplete","not_checked"]}}}
  }},
  "budget":{"type":"object","additionalProperties":false,"required":["limit","tokenizer","accounting","required_material_fit","optional_truncated","diagnostics_truncated","optional_omissions"],"properties":{"limit":{"type":"integer","minimum":1},"tokenizer":{"enum":["cl100k","o200k"]},"accounting":{"const":"serialized_mcp_result_v1"},"required_material_fit":{"type":"boolean"},"optional_truncated":{"type":"boolean"},"diagnostics_truncated":{"type":"boolean"},"optional_omissions":{"type":"array","uniqueItems":true,"items":{"enum":["not_requested","budget","unavailable","work_limit"]}}}},
  "citations":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","source","commit","blob","source_sha256","start","end","quote_sha256","text","freshness","execution"],"properties":{"id":{"type":"string"},"source":{"type":"string"},"commit":{"type":"string"},"blob":{"type":"string"},"source_sha256":{"type":"string"},"start":{"type":"integer","minimum":0},"end":{"type":"integer","minimum":1},"quote_sha256":{"type":"string"},"text":{"type":"string"},"freshness":{"$ref":"#/$defs/freshness"},"execution":{"enum":["not_run","report_of_execution","executed_artifact"]}}}},
  "result_state":{"enum":["complete_within_registered_scope","incomplete","blocked","unavailable"]}
 },
 "$defs":{
  "freshness":{"enum":["current","stale","missing","inaccessible","invalid","not_checked"]},
  "issues":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["code","records"],"properties":{"code":{"enum":["profile_changed","projection_unavailable","generation_mismatch","policy_mismatch","record_not_enrolled","enrollment_incomplete","scope_not_enrolled","identity_unavailable","applicability_unknown","work_limit","source_stale","source_missing","source_inaccessible","source_invalid","premise_unsatisfied","gate_unsatisfied","required_cycle","authority_conflict","required_budget_exceeded","source_changed_during_read"]},"records":{"type":"array","items":{"type":"string"}}}}},
  "records":{"type":"array","items":{"type":"object","additionalProperties":false,"required":["id","revision","role","required","authority","verification","lifecycle","freshness","subject","disposition","citations"],"properties":{
   "id":{"type":"string"},"revision":{"type":"string"},"role":{"enum":["claim","decision","constraint","evidence"]},"required":{"type":"boolean"},
   "authority":{"type":"object","additionalProperties":false,"required":["class","attributed_to","assurance"],"properties":{"class":{"enum":["hypothesis","unverified_extraction","attributable_observation","verified_claim","explicitly_approved_decision"]},"attributed_to":{"type":"string"},"assurance":{"const":"source_attributed"}}},
   "verification":{"type":"object","additionalProperties":false,"required":["outcome","subject","execution"],"properties":{"outcome":{"enum":["unknown","supported","contradicted","inconclusive"]},"subject":{"type":"string"},"execution":{"enum":["not_run","report_of_execution","executed_artifact"]}}},
   "lifecycle":{"enum":["active","superseded","retracted","falsified"]},"freshness":{"$ref":"#/$defs/freshness"},"subject":{"type":"string"},"disposition":{"type":"string"},"citations":{"type":"array","minItems":1,"items":{"type":"string"}}
  }}}
 }
}
```

Additional semantic invariants: returned IDs/revisions/hash/byte ranges must equal normalized admitted records; citations must resolve and hash their exact text; no dangling citation/relationship; repeated IDs reject; complete state requires every coverage bit true plus `required_material_fit=true`, no required issues/conflicts and no required omission. `unavailable` has no citations/records/history/relationships. A known source failure sets evidence=false; incomplete enumeration sets the corresponding coverage bit false. `blocked` budget overflow may retain true acquisition coverage while required material fit is false. Unknown result-state strings or future schema versions reject at the client boundary.

Here “coverage bit” means the five boolean checks enrollment/identity/applicability/required_closure/evidence, not the deliberately false continuous/atomic assurance fields. Complete additionally requires both working observation states `matched`. A phase is matched only when all required current-file/permission observations in that phase match; an observed failure takes `failed` precedence over unfinished observations, otherwise exhaustion is `incomplete`, and an unattempted phase is `not_checked`. Known projection corruption sets unavailable; `projection_integrity` never means a full-storage audit was performed. Every result exposes boundary-only freshness and admission/accessed-row integrity limitations, including a complete-negative result. These added v1 design fields require no manifest or SQL schema change.

### 8.4 Seven representative public examples

Metavariables here identify **proposed example values**, not executed MCP data: `G` is the generation computed from the reviewed full pilot manifest, `ROOT` the owner profile root, and `BASE` the common request fields `{workspaceRoot:ROOT,corpus:"af-perf-175",generation:G,tokenizer:"o200k",includeHistory:false,limits:{records:256,edges:1024,evidence_bytes:1048576,token_budget:8192}}`. These finite fixture values must be admitted by the test profile; they are not global/default caps. Each row is one call to `pcg_query_v1` with BASE plus the shown fields, and specifies the expected payload fields in addition to schema-required scope/generation/policy/subjects/empty arrays.

`subjects` in v1 is exactly one string: canonical JSON of the normalized request subject, including its explicit effective time and any resolved declaration reference. Relationship `from` is a logical record ID; `to` is a record ID except for `CONSTRAINS`, where kind unambiguously identifies a selector ID. Citations use evidence IDs scoped by the returned generation. Protocol errors are JSON-RPC `-32602` with fixed non-disclosing reason for invalid request/unauthorized scope/unsupported tokenizer/too-small control budget; tokenizer creation or denied secret material uses `-32603` with fixed `pcg_control_unavailable` and no source detail. These errors are not evidence results or complete-negative answers.

| Example | Request addition | Expected result / public contract |
| --- | --- | --- |
| 1 Exact source owner | `mode:"constraints", subject:{workspace:"cleanctx",file:"src/mcp/tool_handlers/core/delta.rs",operation:"delta_code_context",task:"modify",at:"2026-11-09T23:00:00Z"}` | Required A/B records, their full source citations/conditions; assurance source-attributed, no edit approval; complete only if full reviewed pilot coverage and all witnesses fit. |
| 2 Dependency closure | `mode:"closure",records:["delta-capture"],subject:<example 1>` | A plus a separately enrolled same-guard/currentness constraint if linked by DEPENDS_ON; typed edges/citations and gate status, no history-as-premise. |
| 3 Reverse impact | `mode:"impact",records:["delta-currentness"],subject:<example 1>` | Incoming DEPENDS_ON/BLOCKED_BY consumers A/B and transitive paths, plus all applicable required constraints. Reverse index completeness is for G only; missing corpus enrollment => incomplete. |
| 4 Provenance | `mode:"provenance",records:["replay-reuse"],subject:{workspace:"cleanctx",file:"src/mcp/tool_handlers/persistence/replay.rs",operation:"replay_history",task:"explain",at:"2026-11-09T23:00:00Z"}` | Rejection, registered-deletion counterexample and exact active regression references; report-of-execution stays such. Complete only if the whole required bundle fits; otherwise example 7's rule applies. No claim of equivalence. |
| 5 Complete negative | constraints on fixture `unconstrained.ts` with an explicit matching reviewed coverage selector, all dimensions supplied, no bound records | `result_state:"complete_within_registered_scope"`, all coverage bits true, `required_records:[]`, `citations:[]`, `reasons:[]`, required material fits. Without the explicit coverage selector this example is not complete. |
| 6 Incomplete enrollment | Same request as 5, but approved fixture manifest has `enrollment.complete:false` | `incomplete`; enrollment=false, reason `enrollment_incomplete`; empty results are not absence proof. An unapproved manifest edit is rejected earlier, never used as this fixture. |
| 7 Required overflow | Example 1, fixture token budget chosen above minimal-control size but below independently counted required envelope | `blocked`; `required_material_fit:false`, reason `required_budget_exceeded`, no truncated actionable authority bundle. All emitted metadata still fits the limit. Below control size => protocol error, not a complete empty result. |


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `c00805aa3f5f973ec206013ac222e1025281afa069041b72a3e9377b4d763fd6`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.


```architecture-graph
node: AUTH.PCG_PROJECTION_V1
kind: AUTHORITY
status: IMPLEMENTED
title: Required evidence and exact token projection
track: REPOSITORY
artifact: docs/architecture/contracts/pcg-projection-v1.md
depends_on:
  - AUTH.PCG_ACQUISITION_V1
note: Operative source-described contract; status does not authenticate approval or certify implementation.
```
