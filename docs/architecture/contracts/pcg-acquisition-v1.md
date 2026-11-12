# Deterministic enrolled-evidence acquisition

Agent: **Agent MaxHeadRoom**. Current normative owner for the source propositions below, normalized in B3. IMPLEMENTED denotes an operative, source-described bounded contract, not independent validation or authenticated approval.

## Scoped standing and original numbering

C02 is reconciled through the source-attributed stage chain: [M2a-1](../../agent/PCG_175_M2A1_IMPLEMENTATION.md) explicitly attributes authorization to the corrected design for enrollment/persistence only; [M2a-2](../../agent/PCG_175_M2A2_IMPLEMENTATION.md) records its subsequent acceptance; [M3](../../agent/PCG_175_M3_IMPLEMENTATION.md) records separate projection authorization and verification; [M4a](../../agent/PCG_175_M4A_IMPLEMENTATION.md) records M3 acceptance and separate MCP authorization; [M4b reconciliation](../../agent/PCG_175_M4B_SCOPE_RECONCILIATION.md) explicitly attributes acceptance of all four named stages. The original design's withheld-approval banner remains historical and intact. These records support the operative bounded v1 contract, not blanket approval of every original proposal or independently certified execution.

Original numbered sections remain locatable through the design document's headings and B3_PROVENANCE.csv. Sections 2–6 and trusted-local security are owned by enrolled v1; §4.3 and §7 by acquisition; §8.1/8.3/8.4 by projection; §8.2 and the implemented MCP boundary by evidence MCP. Cross-section numbers retain those original meanings. Acceptance scenarios, pilot data and staged verification history remain evidence/design history, not new normative owners. Proposed oracle names are specifications, not a claim that B3 authored or executed those tests. Runtime approval, autonomous extraction/injection, embeddings, tenancy and the other original deferrals remain unapproved by these bounded stages.

### 4.3 Correction A: single mandatory-evidence closure

**ME-01 (normative):** `USES_EVIDENCE` is always descriptive. Mandatory evidence is the least fixed point below, computed independently of optional support/history and before token packing. No new manifest edge attribute or SQL column is added. `record.required` selects applicable decision/constraint roots under §7; it is not an instruction to pull every evidence record in the corpus, or to promote a descriptive target.

```text
mandatory_closure(required_roots, required_bindings, mode):
  M_records := required_roots; M_edges := required_bindings; M_evidence := empty
  repeatedly process each new required record/relationship once:
    record r -> add r.primary and each evidence_ref where required=true to M_evidence
    DEPENDS_ON/BLOCKED_BY from r -> add edge and target record (with its satisfaction check)
    incoming VALIDATES/FALSIFIES to r's exact proposition revision -> add assertion edge
      and its source record as required explanation, not as a true premise or approver
    required conflict/supersession/impact paths selected by the mode -> add their edges/records
    every edge/binding in M_edges -> add its provenance evidence to M_evidence
    never follow USES_EVIDENCE, PRESERVES or IMPLEMENTS as a mandatory edge
  deduplicate immutable evidence revisions and produce required source witness set
  validate each M_evidence revision once; encode/reuse its shared citation if deliverable
  evaluate optional refs/descriptive paths separately, without weakening M_* or its coverage
```

Required paths in the fourth step are precisely those establishing applicable controlling heads/conflicts or the requested reverse-impact result in §7; it is not an open-ended relevance rule. Provenance mode traces this mandatory proof graph; descriptive paths remain optional unless a record is explicitly requested as an additional required root. A cycle in descriptive/assertion explanation traversal terminates through visited sets without asserting premise satisfaction. Validation of an evidence revision means checking immutable source/span/hash/permission; §7's two working-file observations are separate source-freshness checks, not repeated evidence revision validation. Shared evidence bytes are reused only within the query.

| Witness or relationship | Mandatory treatment |
| --- | --- |
| Required record primary; explicit `required:true` ref | Always included, even if no USES_EVIDENCE edge exists. |
| Optional ref; USES_EVIDENCE to optional or `required:true` evidence record | Optional unless independently selected by this algorithm. To require its witness, put that exact evidence ID in the required record's `evidence_refs` with `required:true`. |
| Traversed required relationship/binding provenance | Always included, including provenance of incoming VALIDATES/FALSIFIES and required impact paths. Optional descriptive edge provenance is needed only if that edge is emitted. |
| Incoming VALIDATES/FALSIFIES | Both are enumerated for every required target revision; assertion source, its primary/required refs and edge provenance are mandatory. No silent removal of counterevidence. |
| Historical evidence supporting an active decision | Mandatory if selected above; current access and exact historical identity still required. Its old failure is evidence for the rejection, not a currently true prerequisite. `includeHistory:false` cannot remove it. |
| Missing/stale/inaccessible/unauthorized/invalid mandatory source or witness | `blocked` with safe diagnostics and evidence coverage false; confirmed projection row corruption is `unavailable`. Work exhaustion before determination is `incomplete`. No complete actionable bundle. |
| Mandatory evidence exceeds budget | `blocked/required_budget_exceeded`; no truncation of that witness into a complete answer. Minimal control result or explicit too-small-budget protocol error follows §8. |

**EVIDENCE-01/02:** every emitted required record has its primary and explicit required witnesses accounted for; every emitted required relationship has its provenance accounted for. If the full required bundle is not deliverable, return no partial `required_records`/required relationships as actionable authority; use permitted diagnostic IDs instead. **EVIDENCE-03/04:** descriptive evidence grants neither dependency nor authority, and historical failure stays distinct from premise truth. **EVIDENCE-05:** inaccessible or missing mandatory evidence forbids complete actionable context. **EVIDENCE-06:** validate each immutable evidence revision once and serialize it once per explicitly necessary representation: one canonical citation JSON fragment and its one MCP text-string escaped form. Reuse those operation-local fragments through budget candidates/final emission; do not reserialize the same quote per reference or packing attempt. Only one shared citation occurs in the emitted payload. **EVIDENCE-07:** omission of optional source evidence is observable through §8's omission codes, cannot downgrade required completeness, and cannot promote optional facts. Known projection corruption is a projection failure, not mere optional evidence omission.

Positive examples: (1) replay rejection's primary C-accepted plus explicit required refs C-reject/W-replay are mandatory; `USES_EVIDENCE replay-counterexample` adds no prerequisite. Even if its optional record is omitted, C-reject's required span demonstrates divergence; detector PASS never proves equivalence. (2) two required records and their required edge share one evidence revision: primary, required ref and edge provenance all resolve to one validation/citation; an incoming historical FALSIFIES assertion adds its distinct source witness without requiring the refuted proposition to be true.

Proposed negative oracles (not authored tests): `required_primary_missing_blocks`, `required_ref_missing_without_uses_edge_blocks`, `descriptive_required_target_does_not_promote`, `required_edge_provenance_missing_blocks`, `incoming_falsifier_cannot_be_omitted`, `optional_evidence_omission_preserves_required_completeness`, `shared_evidence_once_per_query`, `divergence_pass_is_not_equivalence`. G3-G5/G8 in §10.1 own these specifications.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `672326ccc38136668c00cab7474be03c176e2b07e619c1e28fd3c8aad98c7119`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 7. Applicability, closure and completeness

### 7.1 Scope and exact selectors

Request supplies `workspaceRoot` and corpus ID; root must equal the configured canonical workspace root, not enlarge it. Corpus/partition/owner/policy derive from the profile. The caller must name the exact registered generation; mismatch gives `unavailable/generation_mismatch` rather than silently selecting another tree. Scope is a named enrolled source universe, not every document in a repository or all knowledge relevant to a task.

Subject = `{workspace,file?,entity?,operation?,task?,at}`. File is a validated repo-relative path; operation is an exact MCP operation label (or a source-owned noncoding operation label), task is a reviewed category such as `modify`, `explain`, `investigate`. No lexical synonyms. `at` is explicit canonical UTC `YYYY-MM-DDTHH:MM:SSZ`, interpreted within `[from,until)`; v1 has no implicit wall-clock default. Historical effective-date queries do not bypass current source permissions.

A selector is conjunction of supplied dimensions; several bindings/coverage selectors form a union. Missing selector field means unrestricted on that dimension within this workspace. Missing **request** dimension needed by a selector yields UNKNOWN, not false. A known mismatching dimension makes the conjunction false even if another field is unknown. Coverage requires at least one definitely matching coverage selector AND `enrollment.complete=true`; an unknown potentially applicable required binding still prevents complete output. Coverage describes the reviewed enrolled universe only; it does not certify exhaustive prose extraction.

Evaluate the union per record: any TRUE binding makes that record applicable even if another binding is UNKNOWN; UNKNOWN blocks completeness only when no binding for that same record is TRUE. A FALSE coverage selector cannot certify its scope. These are ordinary three-valued AND/OR rules, not a heuristic confidence threshold.

Canonical entity selector v1 supports **declarations only** using `{kind:"declaration",file,source_sha256,start,end,name,declaration_kind}` mapped exactly to the candidate's `CanonicalDeclaration`. Resolve against existing source-owned current `WorkspaceIndex` canonical endpoints under a read guard, with matching source bytes/hash/span/name/kind; normalize physical path through configured root mapping. Do not invent an entity from a namespace or caller label. No available matching typed endpoint (including after a fresh process before source indexing), ambiguity, or unsupported invocation/route/string endpoint yields `incomplete/identity_unavailable`; no hydration/compilation fallback. File-level bindings still enumerate from the file carried by the entity reference. This explicitly limits first-slice entity readiness while supporting deterministic file/operation/workspace requests across restart without IR acquisition.

Conflict key is source-declared `(subject,role)` for decisions/constraints, with overlap evaluated on selectors/effective intervals. Different active dispositions or verification contradictions in overlapping applicable scope block an actionable result; missing dimensions yield unknown overlap, not permission to ignore a peer. No timestamp/confidence/ranking arbitration. Reviewed explicit supersession with full controlling scope may resolve the conflict. Source-only approval attribution remains visible even when the local owner trusts enrollment.

### 7.2 Deterministic evaluator pseudocode

```text
query(q):
  validate closed request schema; authorize exact root/corpus against trusted profile
  recheck profile bytes/permissions; open read-only SQLite transaction
  pin p,c,g; compare schema/policy/manifest/header identities under INTEGRITY-01
  resolve requested subject and optional source-owned canonical declaration proof
  evaluate coverage selectors and ALL potentially relevant applicability bindings
    SQL may prefilter only definitely unequal dimensions; absent dimensions remain candidates
    verify accessed selector/binding/conflict rows; never full-generation-rehash here
  mark roots: matching active required decisions/constraints; retain unknown required matches
  seed closure/provenance/impact roots from exact record IDs when mode requires them
  compute mandatory_closure from §4.3, with complete required impact paths for impact mode
    verify every accessed required record/edge/ref hash and expected revision; debit once
    detect corrupt required cycles, missing endpoints and conflicting heads
  START witness phase: all enrolled current sources plus required historical sources
    check exact immutable identity/required excerpts once per revision, and permissions
    read/hash each distinct current working file once; record only what was observed
  evaluate gates, lifecycle, currentness, verification subjects and conflicts
  enforce ceilings before expansion/read; incomplete work never becomes absence proof
  order full required bundle; try optional support/history only from remaining work/budget
    descriptive paths cannot enlarge required closure; record optional omission codes
  build/count draft whole result, dropping optional items only
  COMPLETION witness phase: recheck profile and each distinct current file/permission once
    required historical source permission/availability rechecked, immutable bytes reused
  if any required observation now fails: discard draft authority bundle, classify failure
    build/count safe control result; do not retry or serialize the old complete draft
  otherwise classify with explicit observation/integrity limitations and required fit
  serialize/emit exactly counted final result; close transaction
```

`limits.records` debits each visited record plus each examined selector/binding/conflict candidate once; `limits.edges` debits each inspected relation/binding/ref once; `limits.evidence_bytes` debits actual bytes offered by Git/filesystem validation in the prescribed phases, not merely excerpt/output lengths. Read bounded chunks and stop before crossing the ceiling; never read a whole oversized source first. A limit hit during mandatory work sets the affected coverage false and `incomplete/work_limit` unless a known higher-priority block applies. Optional-only exhaustion records omission without changing completed mandatory coverage. No hidden depth/top-k cutoff exists. Ordered IDs, explicit stacks/queues and visited sets determine traversal; SQLite row order is never assumed.

All `check=current` sources remain required **corpus freshness witnesses**, including witness-only code files, regardless of `includeHistory`. This retains the existing corpus-wide freshness invariant: enrollment makes no finer relevance proof allowing an arbitrary witness to be skipped for one record. Their observed drift blocks current completeness even when their quote is not selected. Historical sources are mandatory only through §4.3; optional historical loss is recorded as omission. Imports validate all declared historical sources; ordinary queries do not hash off-path optional historical sources. The cost of checking every corpus-current witness is explicit in §10.3 and may dominate retrieval; targeted row verification must not be advertised as eliminating it. Narrowing this source policy would need an earned relevance contract/owner decision, not an implicit optimization.

**FRESHNESS-01 (normative):** “Currentness was observed against the enrolled source witnesses at the specified retrieval boundaries. This does not establish continuous immutability or an atomic multi-file snapshot.” Immutable Git/source/excerpt identity proves which historical bytes were cited; it does not prove that current working files stayed unchanged. START is after required closure discovery, before source interpretation; COMPLETION is after draft projection/token packing, before final serialization. Source/evidence identity is validated once per immutable revision; current working paths are deduplicated by observed canonical physical source identity and observed once per phase. Conflicting current witnesses for one physical path reject admission; distinct historical revisions of that path remain legal. A detected alias change/escape is invalid. Permission/availability is checked at each applicable phase, not inferred from old SQLite access.

| Observation | Classification and consequence |
| --- | --- |
| Current bytes differ from enrolled witness | `stale`; required corpus freshness fails -> blocked/source_stale. Historical Git blob availability does not cure it. |
| Required path/blob deleted, or permission lost | `missing` / `inaccessible`; blocked with non-disclosing source_missing/source_inaccessible diagnostics. Profile authority loss itself is unavailable. |
| Invalid UTF-8, unsafe path/type, malformed source/span or incompatible identity | `invalid`; blocked/source_invalid, unless the fault is confirmed projection corruption (unavailable). |
| Mandatory observation not completed within work limits | `not_checked`; incomplete, never a successful freshness observation. |
| Failure found after draft retrieval but in COMPLETION | Invalidate complete draft; clear actionable required bundle, classify observed failure (source_changed_during_read plus specific source reason), remeter control result. Do not emit the old draft or loop until it appears stable. |
| Change occurs and restores identical bytes/permissions between reads | May be undetected; both observations can match and a bounded complete result is permitted with the explicit limitation. No deterministic negative oracle may require its detection. |

**FRESHNESS-02 (TOCTOU limit):** changes after a file's last observation, including during final serialization/emission, can escape detection. Two files can each match at their own observation times without ever forming one jointly atomic snapshot; even each individual read is not a filesystem snapshot against an uncooperative concurrent writer. Return the identity of bytes actually hashed and the boundary-only assurance, never continuous/atomic currentness. A future stronger requirement would need a source-provider snapshot/version guarantee or coordinated ownership mechanism; neither is designed or added here. No change to existing Clean-CTX source-currentness, replay, delta or publication mechanisms is authorized.

Required proposed controls and exact expectations: `source_modified_before_start_blocks` (persistent mismatch detected); `source_modified_between_observations_blocks` (mismatch present at COMPLETION detected); `source_change_restore_between_observations_not_claimed_detected` (barrier-controlled unobserved ABA may complete, with continuous/atomic flags false); `source_deleted_during_query_blocks` (absence present at COMPLETION); `source_permission_revoked_during_query_blocks` (revocation effective at an attempted check; restored/unobserved permission changes may escape); `independent_source_observations_are_not_atomic` (detect whichever mismatches are observed, never certify a joint instant); `historical_blob_does_not_cure_stale_working_source`; and `completion_failure_discards_complete_draft`. These are design oracles only, not newly written/executed tests.

### 7.3 Result states and precedence

| State, in highest-first precedence | Exact condition |
| --- | --- |
| `unavailable` | Trusted scope/config/policy permission cannot be established, projection/schema/generation unreadable/incompatible, approved head mismatch or corruption. Emit no evidence payload. Unauthorized root is rejected before lookup with generic `-32602` instead of disclosing corpus existence. |
| `blocked` | Known applicable required authority is stale/missing/inaccessible/invalid, a required premise/gate fails, required cycle/conflict is detected, or the known required serialized bundle cannot fit. Preserve only safe diagnostic context; never imply permission to proceed. Other unknown coverage remains reported. |
| `incomplete` | No known higher-priority block, but enrollment/coverage, identity, applicability, required traversal or evidence checks cannot be completed within limits. Empty/partial results cannot certify absence. |
| `complete_within_registered_scope` | All named corpus/identity/applicability/required closure/observed-freshness/policy checks succeed and the full required bundle fits. Empty result requires exhaustive enrolled-scope applicability evaluation. The assurance assumes intact admission-validated generation membership under INTEGRITY-02 and establishes only FRESHNESS-01 observations, never continuous/atomic currentness or full-storage integrity. It proves no fact outside that boundary and grants no execution authorization. |

An unsatisfied condition discovered before a work limit can yield `blocked` with incomplete coverage; it does not certify all blockers were found. Optional stale history is warning-labelled and omitted if unauthorized; it cannot downgrade a valid complete required bundle unless its contradiction is itself required. `USES_EVIDENCE` never converts evidence into a mandatory prerequisite. Negative controls include a missing task field, omitted required binding, unenrolled file, unknown entity, removed source, hidden history, cycle, stale generation, exact empty covered scope and early traversal exhaustion.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `e900c0ca823be9455de3807657897b04d0984428c10cea6e7ec2024abdc08d05`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.


```architecture-graph
node: AUTH.PCG_ACQUISITION_V1
kind: AUTHORITY
status: IMPLEMENTED
title: Deterministic enrolled-evidence acquisition
track: REPOSITORY
artifact: docs/architecture/contracts/pcg-acquisition-v1.md
depends_on:
  - AUTH.PCG_ENROLLED_V1
note: Operative source-described contract; status does not authenticate approval or certify implementation.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
