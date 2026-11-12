# Owner-enrolled immutable evidence authority

Agent: **Agent MaxHeadRoom**. Current normative owner for the source propositions below, normalized in B3. IMPLEMENTED denotes an operative, source-described bounded contract, not independent validation or authenticated approval.

## Scoped standing and original numbering

C02 is reconciled through the source-attributed stage chain: [M2a-1](../../agent/PCG_175_M2A1_IMPLEMENTATION.md) explicitly attributes authorization to the corrected design for enrollment/persistence only; [M2a-2](../../agent/PCG_175_M2A2_IMPLEMENTATION.md) records its subsequent acceptance; [M3](../../agent/PCG_175_M3_IMPLEMENTATION.md) records separate projection authorization and verification; [M4a](../../agent/PCG_175_M4A_IMPLEMENTATION.md) records M3 acceptance and separate MCP authorization; [M4b reconciliation](../../agent/PCG_175_M4B_SCOPE_RECONCILIATION.md) explicitly attributes acceptance of all four named stages. The original design's withheld-approval banner remains historical and intact. These records support the operative bounded v1 contract, not blanket approval of every original proposal or independently certified execution.

Original numbered sections remain locatable through the design document's headings and B3_PROVENANCE.csv. Sections 2–6 and trusted-local security are owned by enrolled v1; §4.3 and §7 by acquisition; §8.1/8.3/8.4 by projection; §8.2 and the implemented MCP boundary by evidence MCP. Cross-section numbers retain those original meanings. Acceptance scenarios, pilot data and staged verification history remain evidence/design history, not new normative owners. Proposed oracle names are specifications, not a claim that B3 authored or executed those tests. Runtime approval, autonomous extraction/injection, embeddings, tenancy and the other original deferrals remain unapproved by these bounded stages.

## 2. First-slice responsibility boundary

```text
Local owner profile (scope + exact approved manifest digest)
                      |
Authoritative Git artifacts <-- reviewed sidecar references exact bytes
                      |
             explicit operator import
                      v
       isolated, rebuildable SQLite generation
                      |
    explicit read query -> applicability/closure/witness checks
                      v
          cited, token-bounded MCP result

Canonical IR / source persistence / source publication clocks
          remain independent source-owned systems
```

PCG owns enrolled relationship representation, deterministic retrieval, revision provenance and truthful coverage of the named corpus. Artifacts own decision/evidence prose and its domain truth. The local owner owns enrollment and scope; approval attribution in an artifact remains an assertion of that source. SQLite owns only a derived projection, generation publication and indexes. Canonical IR owns structural source facts and typed endpoints. Existing MCP persistence owns per-file baselines/deltas/semantic restoration and its established deletion/recovery lifecycle. No PCG read invokes restore, replay, semantic hydration, source compilation, cache mutation or pending persistence flush.

First deployment: one OS account, one trusted profile, one canonical repository root, one corpus per query, no cross-corpus edges. Sources are locally available immutable Git blobs in that repository plus their current working-file witnesses. No network access occurs during import/query. The cloud commit supplies pilot identity; cloud GitHub is not a new runtime dependency. The first slice supports source-attributed continuity for other repositories/document subjects using the same format, without AF-PERF-specific evaluator branches.

Deferred: runtime ledger records/writes/approval, autonomous extraction, automatic injection, embeddings, enterprise tenancy, multi-root joins, arbitrary external URI ingestion and independent banking/reclosure execution. A source adapter may preserve descriptive reclosure/banking citations, but cannot issue either decision. No source-semantic/global cache, lock, retry, limit or publication change is part of this contract.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `fe5e338181020c67f96055c79893b5f8311544a48a57f493e02f25ed00a214eb`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 3. Source enrollment contract

### 3.1 Choice and trusted local profile

| Alternative | Identity/review/provenance/drift/security/parser tradeoff | First slice |
| --- | --- | --- |
| A: inline declarations | Strong artifact proximity, but changes AF-PERF authorities; fence parsing, source revision churn and declaration review required. Source text still cannot self-authenticate approval. | Defer. |
| B: reviewed sidecar | Stable IDs and exact byte references without rewriting evidence; adds explicit enrollment review and hash maintenance; strict JSON parser; digest pin prevents an agent from silently shrinking coverage. | **SELECT.** |
| C: generated proposal | Reduces initial authoring effort but uncertain applicability/authority; extraction/parser/model revisions and review remain necessary. Never confers approval. | Defer extraction; proposed output could later use B's review boundary. |

Trusted profile v1 is an owner-provided strict JSON object, selected for server reads by `CLEAN_CTX_PCG_PROFILE` containing an absolute path. Omitted variable means PCG is disabled. The profile is separate from evidence and cannot be selected/replaced by a tool request. Its required fields are `schema_version:1`, `partition_id`, `owner_label`, `workspace_id`, canonical absolute `workspace_root`, absolute `database_path`, absolute `manifest_path`, `approved_manifest_sha256`, `policy_revision`, `history_allowed` (boolean), and `limits` (positive integer ceilings `records`, `edges`, `evidence_bytes`, `token_budget`). Unknown fields reject. IDs are case-sensitive ASCII `[A-Za-z0-9][A-Za-z0-9._:-]{0,127}`. No caller-supplied owner/tenant field is accepted. The isolated DB and profile must not be source evidence or source artifacts indexed as claims.

The owner reviews the manifest and pins SHA-256 of its exact UTF-8 bytes in the profile. Editing either document does not cause automatic enrollment. Import requires the exact profile-approved digest; server reads pin a profile fingerprint and check its on-disk bytes/permissions at request start/end. A changed/revoked profile yields `unavailable/profile_changed`, requiring explicit owner restart/reimport as appropriate. Same-OS-account malicious profile replacement is outside the trusted-local threat model, not defeated by this digest. No source-declared `owner`, `approved` or `confidence` field overrides profile trust.

Ordinary queries compare the admitted manifest digest in the generation header with the trusted profile; they do not reparse/rehash the whole sidecar. An unapproved sidecar edit cannot replace the admitted mapping or revoke it implicitly; owner revocation/change is expressed through the trusted profile. Import and explicit audit read the exact approved sidecar and reject absence/digest mismatch. This avoids smuggling full-corpus manifest work into every read without introducing another authority owner or cache.

Proposed maintenance entry point: `clean-ctx pcg import --profile <absolute-path>`; it reads `manifest_path` and never edits it. `--rebuild` constructs the same projection in the same DB transaction after clearing derived rows. Mutually exclusive `--verify-only` performs the explicit read-only full-generation audit in §6.3, without importing or publishing. This remains one operator maintenance surface, not a retrieval MCP operation or scheduled background task. No command is implemented or run by this mission. Server startup opens an existing projection read-only; it never creates, migrates, imports or repairs it.

### 3.2 Closed manifest format

Manifest v1 is strict UTF-8 without BOM. Reject duplicate JSON member names, invalid UTF-8, unsupported version, unknown members, comments, nonfinite numbers and nonintegral offsets. No Unicode normalization. Object order is immaterial to normalized semantic hashes; arrays that represent sets are sorted/deduplicated as specified in §5. Owner approval pins raw bytes, so even whitespace changes require review of a new digest. All source/record/selector/evidence/edge IDs are unique in their own corpus namespace. No executable expressions or document-supplied filesystem roots exist.

| Required top-level field | Exact content |
| --- | --- |
| `schema_version`, `parser_version` | Integer `1`, string `pcg-json-v1`. A parser/schema mismatch rejects import; no best-effort projection. |
| `corpus_id`, `source_namespace` | Stable scoped IDs; namespace is an origin label, not a directory or CBM project. |
| `enrollment` | `{owner_label,policy_revision,complete}`; `complete` boolean is an owner-reviewed corpus declaration, not proof of all prose/world truth. Must match profile attribution/policy. |
| `sources` | Array of `{id,path,commit,blob,sha256,check}`. `path` is repo-relative slash-separated; `commit` and `blob` are full hex Git object IDs; SHA-256 covers exact blob bytes; `check` is `current` or `historical`. |
| `selectors` | Array of `{id,purpose,workspace,file?,entity?,operation?,task?,from,until}`; purpose `coverage` or `applicability`; exact scalar selectors, UTC half-open interval, null unbounded ends. Semantics in §7. |
| `evidence` | Array of `{id,source,start,end,sha256,heading?}`. Half-open UTF-8 byte span; nonempty, scalar-boundary aligned, quote hash exact; optional heading must occur exactly once in the span and start at its first byte. |
| `records` | Array of `{id,role,required,primary,authority,verification,lifecycle,subject,disposition,evidence_refs}`. Role `claim/decision/constraint/evidence`; primary evidence ID; authority `{class,attributed_to}`; verification `{outcome,subject,execution}`; lifecycle enum §4; subject/disposition are exact owner-reviewed labels, not a replacement decision body; evidence_refs array `{id,required}`. No free-form decision body field. |
| `edges` | Array of `{id,kind,from,to?,to_selector?,evidence,gate?}`. Exactly one target form; `gate` only for `BLOCKED_BY`; semantics §4. Provenance evidence ID is mandatory. |

`verification.execution` is `not_run`, `report_of_execution` or `executed_artifact`; the last requires the actual enrolled executable receipt artifact, not a report naming it. `subject` identifies the exact proposition evaluated, e.g. `equivalence` versus `counterexample_detection`. `disposition` is descriptive source vocabulary (`PROTECTED`, `DEFER`, `REJECT`, etc.), never an approval capability. `evidence_refs` need not repeat `primary`; primary is always required. All definitions/endpoints resolve at import, including optional ones; later loss of optional source evidence is handled by §4.3. `USES_EVIDENCE` has no required-edge attribute; mandatory witness selection is exclusively §4.3. A target record's `required` flag never promotes a descriptive edge into required closure.

Each corpus has at least one coverage selector. A binding is an explicit `CONSTRAINS` edge from a decision/constraint to an applicability selector; coverage selectors cannot be binding targets. Exact records queried by ID need not have a binding, but cannot thereby become applicable constraints. No inline expression evaluation, glob, regex, lexical ranking or implicit inheritance is admitted.

At least one source has `check=current`; all such sources share one exact enrollment commit, which becomes `generation.source_commit`. Historical source rows may name earlier commits. Import/resource ceilings debit raw manifest bytes, examined definitions and actual source bytes before unbounded parsing/reads; ceiling failure is explicit, never partial enrollment. Namespace and `enrollment.complete` are inert until the trusted profile's exact digest admits this manifest.

### 3.3 Complete proposed enrollment example and normalization

This is a complete **unimplemented one-record corpus example**, not the complete A-E pilot and not an owner-approved manifest. Hashes/locators are OBSERVED cloud witnesses (§9). Its record IDs and classification/mapping are PROPOSED. It contains no duplicated decision prose.

```json
{
  "schema_version":1,"parser_version":"pcg-json-v1","corpus_id":"af-perf-175","source_namespace":"codeliftsleep2:Clean-CTX",
  "enrollment":{"owner_label":"local-owner","policy_revision":"local-source-attribution-v1","complete":true},
  "sources":[
    {"id":"delta-report","path":"docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md","commit":"8ce0a6865ff8a2b55866895c6a239f6bcc3245a3","blob":"606e9da096cb7c272252246296d2d8f8cfa882f3","sha256":"0177375cf4b24b6ddf4b741da1ceff1d94ed840c00a7a0ae60cf4349b30dea51","check":"current"},
    {"id":"delta-owner","path":"src/mcp/tool_handlers/core/delta.rs","commit":"8ce0a6865ff8a2b55866895c6a239f6bcc3245a3","blob":"9b1849de120228c7786ff0124bfcb89c40f32d81","sha256":"b50b5fca0fd8550df1be1910e35e6c536b759b92872819cfcc00dfee1aa46722","check":"current"}
  ],
  "selectors":[
    {"id":"delta-coverage","purpose":"coverage","workspace":"cleanctx","file":"src/mcp/tool_handlers/core/delta.rs","operation":"delta_code_context","task":"modify","from":null,"until":null},
    {"id":"delta-modify","purpose":"applicability","workspace":"cleanctx","file":"src/mcp/tool_handlers/core/delta.rs","operation":"delta_code_context","task":"modify","from":null,"until":null}
  ],
  "evidence":[{"id":"delta-final","source":"delta-report","start":22798,"end":25013,"sha256":"3c95b9640d858ac6963dc5dbb910bf0a41c0c61b272dad29a506d75f0c575e0f","heading":"## Final composed acceptance: CLOSED / FIX / PROTECTED"}],
  "records":[{"id":"delta-capture","role":"decision","required":true,"primary":"delta-final","authority":{"class":"explicitly_approved_decision","attributed_to":"owner approval reported by Agent MaxHeadRoom"},"verification":{"outcome":"supported","subject":"delayed_capture_contract","execution":"report_of_execution"},"lifecycle":"active","subject":"delta_previous_authority_capture","disposition":"PROTECTED","evidence_refs":[]}],
  "edges":[{"id":"delta-capture-binding","kind":"CONSTRAINS","from":"delta-capture","to_selector":"delta-modify","evidence":"delta-final"}]
}
```

Normalized representation is exactly: partition from profile; corpus/name/version; a source revision for each `(source-id,commit,blob,sha256,path,check)`; evidence revision for the primary span; one record revision of `delta-capture`; selectors `delta-coverage` and `delta-modify`; one binding revision; and a generation fingerprint. `primary_text` is derived from bytes `[22798,25013)` of the report, not authored in the manifest. `delta-owner` is a **corpus currentness witness**, not a source body stored by default: all declared sources are validated at import; query witness scope is §7.2. Only enrolled excerpts are persisted. Normalized authority includes `assurance="source_attributed"` regardless of the source's approval label. Freshness is `not_checked` until query validation. Evidence execution remains `report_of_execution`; the 110-pass receipt itself is not magically enrolled.

Reject absolute/traversing paths, `.git` traversal, symlinks/reparse-point escapes, sources outside the pinned repository, Git fetch/promisor fallback, missing blobs, hash mismatch, ambiguous heading, wrong byte range, conflicting IDs, unknown edge vocabulary, invalid intervals or unsupported selectors. Git object reads use argument arrays and literal validated OIDs/paths; never shell-built revspecs or hooks. Current working sources must also be regular files beneath the profile root; Git LF/CRLF conversion is **not** normalized away. A mismatch requires reviewed re-enrollment or historical-only interpretation, not a rewritten hash. Initial/replacement invalid import publishes no generation.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `7b52118ac760893b1a56f0a8846c089dd973214fd97087f6292b0c8ca73f7594`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 4. Semantic and relationship model

### 4.1 Roles and independent dimensions

| Dimension | Stored source-attributed values | Query treatment |
| --- | --- | --- |
| Role | `claim`, `decision`, `constraint`, `evidence` | Determines endpoint/selection rules; evidence records identify propositions about actual artifacts, not an extra authority layer. |
| Origin/authority | `hypothesis`, `unverified_extraction`, `attributable_observation`, `verified_claim`, `explicitly_approved_decision` plus attributed actor | Stored classification; always expose assurance `source_attributed`. No authenticated runtime approval exists in v1. Only reviewed profile enrollment permits reliance on this attribution. |
| Verification | `unknown`, `supported`, `contradicted`, `inconclusive`; exact subject and execution category | Stored as what the source says; not recomputed by executing tests. `supported/equivalence` and `supported/counterexample_detection` are distinct. |
| Lifecycle | `active`, `superseded`, `retracted`, `falsified` | Stored source assertion, validated with links. `falsified` retains historical result; it does not erase evidence. |
| Freshness | No persisted “current truth” flag | Derived each read: `current`, `stale`, `missing`, `inaccessible`, `invalid`, `not_checked`. Required old decisions remain visible as blocked obligations when stale. |

`required` means necessary to the selected operation's context, not permission to act. A required applicable decision/constraint must be emitted with complete source span/conditions or the result cannot be complete. Optional records cannot override controlling requirements. Proposal/claim origins cannot satisfy approval gates. Independent reclosure/banking have no executable v1 status shortcut: preserve their actual evidence citations and distinct subjects; `VALIDATES` cannot establish either.

### 4.2 Relation contracts

All edges have source evidence and immutable endpoint revisions; applicability is checked before root selection. Once required closure selects an edge, its target is included even if its own selector differs: a prerequisite cannot disappear through applicability filtering. An explicit out-of-corpus/root prerequisite is unsupported and blocks import rather than being silently severed.

| Kind / direction / endpoints | Meaning, required closure and lifecycle | Negative / cycle control |
| --- | --- | --- |
| `DEPENDS_ON`: decision/constraint/claim -> decision/constraint/claim | Necessary premise. Required source forces target and recursive required closure. Target must be active, current, noncontradicted, and have at least source-attributed observation authority; hypotheses/unverified extraction cannot discharge a premise. | Descriptive evidence does not substitute. Dependency cycles are import-invalid; no fixed-point semantics. |
| `BLOCKED_BY`: decision/constraint/claim -> claim/decision/constraint | Unresolved gate. Gate is exactly `{outcome:"supported",subject:<exact>,authority:<class>}`, with class one of `attributable_observation`, `verified_claim`, `explicitly_approved_decision`. Satisfaction requires matching target verification subject, active/current target and authority at least that class in this explicit order. Target and gate evidence are required. | No status-name or confidence shortcut; hypothesis never satisfies. Combined dependency/blocker cycles reject import. |
| `CONSTRAINS`: decision/constraint -> applicability selector | Declares applicability binding, not an extra record prerequisite. Every matching active `required:true` bound record is a required root. | No bound inferred from title, proximity or namespace; unknown matching blocks completeness. Not a record-cycle edge. |
| `USES_EVIDENCE`: any record -> evidence record | Always descriptive. Neither this edge nor its target's `required` flag creates mandatory closure. Required witnesses must be named by primary/explicit `evidence_refs.required=true`, or be required independently by §4.3. The target record/edge may be offered as optional support. | A failed experiment may be cited; no premise, verification or approval promotion. Descriptive cycles terminate by visited IDs. |
| `VALIDATES`: evidence/claim -> claim/decision/constraint | Source asserts support for exact target revision/subject. Incoming verification must be reported when its target is required, including its primary/required evidence. | Does not mutate target state, authorize implementation or reclose/bank; a detector may validate divergence rather than equivalence. |
| `FALSIFIES`: evidence/claim -> claim/decision/constraint | Source asserts counterevidence to target; incoming falsifiers are mandatory for a required target. Active contradictory support must be exposed, not ranked away. | No automatic erasure or approval; a falsified target can remain history. Cycles never imply refutation is resolved. |
| `SUPERSEDES`: decision/constraint/claim -> same role | Source claims current control over same `subject`; old target must be `superseded`, effective applicability must explicitly cover the relevant overlap, and successor attribution must meet profile policy. | Exactly one applicable active head for a single-valued subject; cycles/forks/conflicting overlap reject import. History is never a required premise merely because superseded. |
| `PRESERVES`, `IMPLEMENTS`: descriptive record -> record | Imported respect/realization relationship with evidence. Optional history unless provenance of a required relation/condition. | Neither is a prerequisite, successful verification, reclosure or banking. Cycles allowed with visited traversal. |

No caller may choose a weaker closure edge set. `constraints`, `closure`, `impact` and `provenance` all collect mandatory incoming validation/falsification and explicit conflict peers, then their required evidence. Opposing `VALIDATES`/`FALSIFIES` assertions on the same subject are an unresolved conflict unless a source-backed lifecycle/supersession explicitly resolves them. Distinct decision subjects can coexist; same-subject/disposition records retain their distinct logical IDs while identical citation payloads are emitted once. No record receives independent approval merely because a second source repeats it.

Required **historical evidence** is not a required **true premise**: a falsified claim may be emitted to explain a rejection without blocking that rejection. Only controlling active authority and DEPENDS_ON/BLOCKED_BY satisfaction require the corresponding active/noncontradicted state. A historical detector result can be valid evidence of past divergence while the equivalence proposition remains falsified.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `3587cf546ed91ae38bf556d279c1cf0bb11fcb1a193ed15c21b1ce3eedfe01cc`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 5. Identity, revisions and provenance

Hash function `H` is SHA-256 of strict UTF-8 canonical JSON: object keys sorted by UTF-8 byte order, no insignificant whitespace, no Unicode normalization, JSON strings escaped only as required for valid JSON, integers decimal, booleans/null standard, and set-valued arrays sorted by their canonical encodings. Source byte hashes never use this JSON transform. These are versioned v1 hash rules, not existing source identity rules.

| Identity | Definition and ownership |
| --- | --- |
| Logical record | `(partition_id,corpus_id,record_id)`; owner assigns stable ID, independent of title/path/hash. |
| Source identity | `(partition,corpus,source_namespace,source_id)`; source ID survives a reviewed rename. |
| Source revision | `H({source_id,path,commit,blob,sha256,check})`; source OID and SHA-256 both verified. |
| Evidence revision | `H({evidence_id,source_revision,start,end,quote_sha256})`; citation is exact half-open bytes; line numbers/headings are orientation, not identity. |
| Record revision | `H({record_id,role,required,primary_evidence_revision,authority,verification,lifecycle,subject,disposition,sorted_evidence_refs})`; no edge recursion in this hash. |
| Selector/binding revision | Selector hash covers exact dimensions/interval/purpose; `CONSTRAINS` revision covers source record revision, selector hash and provenance evidence revision. |
| Relationship assertion revision | `H({edge_id,kind,from_record_revision,to_record_revision,gate,provenance_evidence_revision})`; endpoints resolve within one candidate generation. |
| Generation | `H({partition,corpus,raw_manifest_sha256,policy_revision,parser_version,sorted_source_revisions,sorted_record_revisions,sorted_edge_or_binding_revisions,sorted_selector_revisions})`. No timestamp participates. |

Idempotency key is `(partition,corpus,manifest_sha256,policy_revision,parser_version)`; same key/same normalized fingerprint is a no-op, different fingerprint is invalid/nondeterministic import. A parser upgrade changes the key and generation and requires reviewed version support, never reinterprets old rows silently. The same source body at different origins/partitions stays distinct. Same-name records in different corpora remain distinct; no cross-project hash deduplication.

Nonsemantic edits still change a source witness and dependent record revision if cited bytes/source revision changes; a reviewer may preserve logical IDs but must approve a new manifest digest. A changed decision requires an explicit revised source assertion, correct lifecycle/supersession and reviewed enrollment; it is never a summary overwrite. Rename preserves source/record logical IDs but changes source revision, locators and exact file selectors; old path history remains historical. New missing/deleted sources cause failed import or blocked query, never automatic deletion of a constraint. Removing an enrolled record or coverage rule requires a reviewed replacement manifest; the prior generation is retained only under owner policy, not used as silent current fallback.

For a substantively changed controlling conclusion, enroll a distinct successor record ID plus SUPERSEDES to the predecessor, both represented in the new generation with their exact source revisions. A prior generation's original lifecycle assertion remains unchanged. Same-ID reimport may update nonsemantic witnesses while preserving the conclusion, but must not hide substantive replacement. `includeHistory` exposes only historical/superseded records explicitly enrolled in the requested active generation; it never scans retained older generations under an obsolete approval policy. Missing unenrolled history is not reconstructed.

For permissions, history means superseded/retracted/falsified records or retained `check=historical` source revisions, not a current report's still-valid observation that a past run failed. `includeHistory` controls optional history and explicit historical roots; direct requests for those roots require it true plus profile permission. Required historical evidence supporting a current conclusion is included even when optional history is false, but still requires profile history permission. Withheld required history blocks the current bundle; a caller cannot bypass history policy by naming a record ID or following an edge.

Currentness witnesses: immutable commit/blob membership, source and excerpt hashes, observed profile bytes/permissions, admitted manifest digest compared to profile/header, policy identity, current-file observations for `check=current`, and source-owned canonical endpoint proof when an entity selector is requested. These are identity and boundary observations as §7.2 defines, not continuous-currentness proof. Required `historical` sources use retained local immutable blobs and current history permission; absence means missing history, not inferred truth. A stored old excerpt cannot prove a currently missing source. Supersession preserves prior source assertions; lawful deletion can remove historical projection data because the authoritative source retention policy, not SQLite, owns that obligation.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `69e9d030f74cb865447430140a6f4149cb040584e97012c64c2f41cdd62cd495`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

## 6. SQLite persistence contract

### 6.1 Isolation and proposed DDL

Choose an **isolated PCG database** supplied by the trusted profile. Sharing the existing DB under new table names saves a connection but couples schema versioning, purge/backup, corruption and source-context lifecycle. An isolated file avoids touching `ContextStore`, source baseline replacement, durable semantic transactions and publication clocks. Initial schema version is `1`; SQLite owns derived rows only. Writer uses `foreign_keys=ON`, `journal_mode=WAL`, `synchronous=FULL`, `busy_timeout=0`; no implicit retry. Server opens `SQLITE_OPEN_READ_ONLY`, `query_only=ON`, foreign keys enabled; never use SQLite immutable-file mode against a live WAL DB.

Concrete proposed DDL follows. `p,c,g` mean partition/corpus/generation. JSON fields are strictly validated and canonicalized by the importer; no dependency on SQLite JSON extensions. All declared FKs are enabled. SHA fields are lowercase 64 hex validated by importer as well as length checks; Git OIDs use the repository's verified object format. Cross-table role/endpoint/lifecycle rules in §§3-4 are admission checks in the same transaction, not implied by SQL alone.

```sql
CREATE TABLE pcg_schema(version INTEGER PRIMARY KEY CHECK(version=1));
INSERT INTO pcg_schema VALUES(1);
CREATE TABLE corpus(
 p TEXT NOT NULL,c TEXT NOT NULL,namespace TEXT NOT NULL,
 PRIMARY KEY(p,c)
);
CREATE TABLE generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL CHECK(length(g)=64),
 manifest_sha TEXT NOT NULL CHECK(length(manifest_sha)=64),policy TEXT NOT NULL,
 parser TEXT NOT NULL,source_commit TEXT NOT NULL,fingerprint TEXT NOT NULL,
 enrollment_complete INTEGER NOT NULL CHECK(enrollment_complete IN(0,1)),
 PRIMARY KEY(p,c,g),UNIQUE(p,c,manifest_sha,policy,parser),
 FOREIGN KEY(p,c) REFERENCES corpus(p,c) ON DELETE CASCADE
);
CREATE TABLE active_generation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,
 PRIMARY KEY(p,c),FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g)
);
CREATE TABLE source_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,sid TEXT NOT NULL,srev TEXT NOT NULL,
 path TEXT NOT NULL,git_commit TEXT NOT NULL,blob TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64),
 check_mode TEXT NOT NULL CHECK(check_mode IN('current','historical')),
 PRIMARY KEY(p,c,g,sid,srev),UNIQUE(p,c,g,sid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE evidence_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 sid TEXT NOT NULL,srev TEXT NOT NULL,bstart INTEGER NOT NULL CHECK(bstart>=0),
 bend INTEGER NOT NULL CHECK(bend>bstart),quote_sha TEXT NOT NULL CHECK(length(quote_sha)=64),
 quote_utf8 BLOB NOT NULL CHECK(length(quote_utf8)=bend-bstart),
 PRIMARY KEY(p,c,g,eid,erev),UNIQUE(p,c,g,eid),
 FOREIGN KEY(p,c,g,sid,srev) REFERENCES source_revision(p,c,g,sid,srev) ON DELETE CASCADE
);
CREATE TABLE record_revision(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 role TEXT NOT NULL CHECK(role IN('claim','decision','constraint','evidence')),
 required INTEGER NOT NULL CHECK(required IN(0,1)),eid TEXT NOT NULL,erev TEXT NOT NULL,
 authority TEXT NOT NULL CHECK(authority IN('hypothesis','unverified_extraction','attributable_observation','verified_claim','explicitly_approved_decision')),
 attribution TEXT NOT NULL,verification TEXT NOT NULL CHECK(verification IN('unknown','supported','contradicted','inconclusive')),
 verification_subject TEXT NOT NULL,execution TEXT NOT NULL CHECK(execution IN('not_run','report_of_execution','executed_artifact')),
 lifecycle TEXT NOT NULL CHECK(lifecycle IN('active','superseded','retracted','falsified')),
 subject TEXT NOT NULL,disposition TEXT NOT NULL,
 PRIMARY KEY(p,c,g,rid,rrev),UNIQUE(p,c,g,rid),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE evidence_ref(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,rid TEXT NOT NULL,rrev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,required INTEGER NOT NULL CHECK(required IN(0,1)),
 PRIMARY KEY(p,c,g,rid,rrev,eid,erev),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE selector(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,
 purpose TEXT NOT NULL CHECK(purpose IN('coverage','applicability')),
 workspace TEXT NOT NULL,file_path TEXT,entity_json TEXT,operation TEXT,task TEXT,
 effective_from TEXT,effective_until TEXT,normalized_json TEXT NOT NULL,
 PRIMARY KEY(p,c,g,xid),UNIQUE(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g) REFERENCES generation(p,c,g) ON DELETE CASCADE
);
CREATE TABLE binding(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,bid TEXT NOT NULL,brev TEXT NOT NULL,
 rid TEXT NOT NULL,rrev TEXT NOT NULL,xid TEXT NOT NULL,xrev TEXT NOT NULL,eid TEXT NOT NULL,erev TEXT NOT NULL,
 PRIMARY KEY(p,c,g,bid),
 FOREIGN KEY(p,c,g,rid,rrev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,xid,xrev) REFERENCES selector(p,c,g,xid,xrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE TABLE relation(
 p TEXT NOT NULL,c TEXT NOT NULL,g TEXT NOT NULL,aid TEXT NOT NULL,arev TEXT NOT NULL,
 kind TEXT NOT NULL CHECK(kind IN('DEPENDS_ON','BLOCKED_BY','USES_EVIDENCE','VALIDATES','FALSIFIES','SUPERSEDES','PRESERVES','IMPLEMENTS')),
 src TEXT NOT NULL,src_rev TEXT NOT NULL,dst TEXT NOT NULL,dst_rev TEXT NOT NULL,
 eid TEXT NOT NULL,erev TEXT NOT NULL,gate_json TEXT,
 PRIMARY KEY(p,c,g,aid),
 CHECK((kind='BLOCKED_BY' AND gate_json IS NOT NULL) OR (kind<>'BLOCKED_BY' AND gate_json IS NULL)),
 FOREIGN KEY(p,c,g,src,src_rev) REFERENCES record_revision(p,c,g,rid,rrev) ON DELETE CASCADE,
 FOREIGN KEY(p,c,g,dst,dst_rev) REFERENCES record_revision(p,c,g,rid,rrev),
 FOREIGN KEY(p,c,g,eid,erev) REFERENCES evidence_revision(p,c,g,eid,erev)
);
CREATE INDEX record_subject ON record_revision(p,c,g,subject,lifecycle);
CREATE INDEX selector_scope ON selector(p,c,g,purpose,workspace,file_path,operation,task);
CREATE INDEX binding_selector ON binding(p,c,g,xid,rid);
CREATE INDEX relation_forward ON relation(p,c,g,src,kind,dst);
CREATE INDEX relation_reverse ON relation(p,c,g,dst,kind,src);
CREATE INDEX evidence_source ON evidence_revision(p,c,g,sid,srev);
CREATE INDEX evidence_required ON evidence_ref(p,c,g,rid,rrev,required,eid,erev);
CREATE INDEX source_check ON source_revision(p,c,g,check_mode,sid);
```

`source_commit` is the current repository enrollment anchor, with individual historical source commits retained in source rows. Import metadata/fingerprint/parser/policy/source rows are the rebuild manifest; an additional authoritative ledger or audit-event table is unnecessary. `CONSTRAINS` is stored as `binding`, not a fake subject record or nullable-FK relation. One source ID has one selected revision per generation; enrolling multiple historical revisions of one artifact uses explicit distinct source IDs referencing the same origin path and immutable commits. Identity continuity remains in the origin namespace/record IDs, not path alias guessing.

### 6.2 Transaction and lifecycle algorithms

Import: validate trusted profile, raw manifest digest, syntax, paths, exact Git source bytes/excerpts, secret policy, selectors, records, edges, cycles and conflicts in bounded operation-local memory; capture all source witnesses. Open writer; `BEGIN IMMEDIATE` (busy fails explicitly); validate schema and the active head expected at preparation; insert corpus/generation and source/evidence/record/selector/binding/edge rows; verify all constraints, complete normalized fingerprints/membership and FK consistency; reread each distinct current file once; verify profile/manifest digest again; publish `active_generation` in the same transaction; `COMMIT`. An observed mismatch rejects import, but an unobserved change-and-restore does not. Filesystem observations neither freeze working files through commit nor establish atomic multi-file currentness. Success is returned only after commit.

Reimport: identical key/fingerprint returns the same generation without replacing rows; a different manifest or source replacement produces a wholly separate generation. CAS of the active head must match the head observed before preparation; a stale importer reports `import_head_changed` with no retry. A profile-approved manifest change makes the old generation ineligible for complete queries even before import; there is no fallback to outdated approved evidence. Invalid source replacement/semantic validation rolls back; old rows remain historical but cannot masquerade as the newly requested corpus.

Crash/interruption: before commit, all inserted rows/head changes roll back; after commit, a new process sees the complete committed generation. Tests must kill real fixture import processes before/after commit. A process-local flag or successful SQL call before commit is not durable acceptance. WAL read transactions may continue on a previously pinned valid generation; they report its identity and never claim the newest head. Source/profile changes detected during a read prevent a complete result.

Restart: no runtime ledger recovery. Validate database-open metadata and exact active generation identity as §6.3 specifies, then apply targeted query checks. This is not a full-generation rehash at restart or connection-open. Missing DB/schema, unknown version or detected corrupt rows produce `unavailable`; never auto-create/repair. Explicit rebuild: under one writer transaction delete active pointer and derived corpus generations in FK-safe order (relations/bindings/refs/records/selectors/evidence/sources/generations), rebuild from approved retained sources, publish head and commit. Failure rolls back the entire rebuild, leaving old committed DB intact. Readers pin their old SQLite snapshot; new reads see only the whole new head. Rebuild does not recover missing authoritative artifacts.

Migration: v1 has no implicit upgrade from any existing Clean-CTX schema or unknown PCG version. Future schema changes require a separately approved migration. Because this DB is derived, the initial supported upgrade route is explicit transactional rebuild into a supported schema after owner backup/permission review; never invoke existing `SqliteStore::migrate`. Corruption must be handled by closing affected connections, returning generic unavailable diagnostics and explicit operator replacement/rebuild after readers stop; no automatic database deletion, WAL removal or success fallback. Projection backup is optional recovery convenience; the retained approved manifest/profile and authoritative source revisions are essential.

Concurrency: independent read-only connections use read transactions; imports serialize through SQLite's single writer and head CAS, with zero application retries. Read transactions pin one generation; query SQL always includes `p,c,g`. No read updates metrics/timestamps or flushes persistence. Import/resource ceilings come from the trusted profile and are specific to PCG, not new global query caps. Immutable evidence is validated once per revision; distinct current files are observed once per specified phase, never per referencing record. Only accessed row hashes are recomputed during retrieval. Persistent semantic memoization is not introduced.

### 6.3 Correction C: admission-validated generation, targeted reads

**INTEGRITY-01 (normative): choose Option B.** Fully validate the generation at admission and publish it atomically; each query validates identity plus all accessed applicability/proof rows and required evidence. Do not recompute the full-generation fingerprint on ordinary retrieval or hide that scan in database-open validation. Admission/application checks establish the semantic invariants; SQLite transactions preserve their all-or-nothing publication, not their semantic truth. Generation IDs identify admitted content and do not attest that every stored row remains intact.

| Responsibility | Exact boundary / required checks |
| --- | --- |
| Import | Validate entire manifest/source/record/selector/relationship universe, exact memberships, hashes, closure/cycle/conflict rules and FK consistency; compute complete fingerprint. Reject any failure before publication. |
| Transactional publication | Insert immutable rows and head within one transaction; compare expected head; acknowledge only committed generation. Never edit published generation rows in place. |
| DB open / restart | Read schema version, required table/index definitions, active-head/header, approved manifest/policy/parser/partition compatibility and SQLite errors encountered opening/accessing them. No implicit `integrity_check`/full table scan. Metadata mismatch refuses service; success is not a corpus-integrity certificate. |
| Per-request generation identity | Pin one SQLite read transaction and exact `(p,c,g)`; compare requested generation, active header, profile-approved manifest digest, policy/parser/schema and stored fingerprint identity. Do not recompute all constituent hashes here. |
| Accessed-row verification | Recompute hashes and validate typed structure/partition/references for every examined applicability selector/binding/conflict candidate, required closure record, traversed assertion/ref and selected citation. Confirm endpoint revision/FK references when traversed. Known malformed/missing required rows or hash mismatch are projection corruption -> unavailable. Any optional row actually accessed is checked too. |
| Source/evidence witnesses | Apply §§4.3/7.2; exact immutable required evidence once, all corpus-wide `check=current` witnesses at both working-file boundaries, and required historical sources with permission. No full off-path historical evidence rehash. |
| Explicit full audit | Owner invokes proposed `--verify-only`: read-only pinned generation, complete row/membership/fingerprint/FK/semantic verification against approved manifest and retained immutable evidence. Report all limitations/failures; no repair, new generation, periodic worker or authority-cache receipt. Audit does not grant continuous filesystem freshness. |
| Corruption/rebuild | Any detected projection integrity failure refuses the affected read/handle pending operator recovery. A separate audit failure requires owner stop/disable affected readers and explicit rebuild; no cross-process failure notification is claimed. Rebuild re-admits exact authoritative sources; no automatic success fallback. |

| Alternative | Guarantees / detection / cost / lifecycle |
| --- | --- |
| A: full verification every query | Recomputes all row memberships/hashes against the expected generation and detects off-path changes visible in the pinned snapshot, if verification itself completes. O(total generation data) even for a small query; restart and concurrent imports still use one pinned generation. It does not freeze files or authenticate an approver. |
| B: admission plus targeted verification (**selected**) | Same full admission/atomic publication; per-request proof-row checks, full applicable-universe evaluation and required witnesses. Restart has metadata/targeted checks, not a hidden O(N) audit. Concurrent imports cannot mix generation rows within the read transaction. Unaccessed corruption/tampering may remain undetected until an audit or affected access; no persistent validation cache is introduced. |

**INTEGRITY-02 (limitation):** B does not detect all off-path row corruption, valid-SQL deletion of an otherwise required binding/edge, forged selector prefilter fields, index omissions or tampering that leaves header metadata unchanged. Such damage can affect an absence/closure answer without being discovered by that query; a matching generation ID is not a proof of intact membership. `complete_within_registered_scope` is conditional on the trusted-local, admission-validated generation remaining immutable outside the admitted writer. It is not a cryptographic or full-storage-integrity certificate. Source hashing cannot repair missing graph membership. Stronger untrusted-storage/tamper guarantees would require owner review, not a silent completeness claim or performance exemption. This restriction is exposed as `coverage.projection_integrity="admission_and_accessed_rows"` in §8.

The accepted trusted-local deployment allows only the admitted importer to write the projection; same-account malicious mutation is outside that trust assumption, while accidental/off-path damage remains an explicit detection limit. The evaluator must still exhaust the registered applicability candidates; B authorizes no intentional top-k, skipped binding or partial-index completeness shortcut. If that deployment assumption is unacceptable for the implementation, stop for owner disposition rather than advertise B as tamper-proof.

No schema tables/record/edge attributes change. The two additional design-text indexes in §6.1 bound required-ref lookup and enumeration of current witness sources; existing selector_scope, binding_selector, forward/reverse relation, primary-key and evidence_source indexes remain necessary. Positive controls: original admission/rebuild fingerprint equality, repeated targeted reads with unchanged immutable generation, and pinned old/new readers under concurrent import. Negative controls: `targeted_required_row_corruption_unavailable`, `off_path_corruption_detected_on_audit_or_access`, `removed_binding_not_certified_by_generation_id`, `restart_does_not_rehash_generation`, `concurrent_import_never_mixes_generation`. Off-path tests must record that targeted reads may not detect damage; they must not pretend B provides A's guarantees.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `e788f53c4a406eeeef1f0ddea26365175b7df2aa1b349ec608c47cbc30a31b53`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.

### 10.2 Trusted-local security and data handling

Profile and manifest are local owner-controlled files with OS permissions restricting untrusted modification. Tool arguments, source text and SQLite excerpts are untrusted data. Workspace paths are locators, not tenant principals. Reject unknown corpus/root before discovery; all joins and errors respect exact p/c/g. Local read access to source is required at each query phase; inaccessible evidence does not inherit old access through a cached excerpt. History requires explicit policy permission and currently readable authoritative revision; permission loss returns generic blocked/unavailable without sensitive locators/hashes. No enterprise multi-tenant security claim exists.

Secret minimization: enroll only necessary reviewed spans and witness hashes, no full source bodies, transcripts, arbitrary prose metadata or source credentials. The first-slice policy is exactly `reviewed-public-repository-evidence-v1`: only owner-reviewed nonsecret repository artifacts, plus deterministic deny checks on **all to-be-persisted or emitted text/metadata**. The deny set is Rust-regex syntax: `-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----`, `(?:gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|AKIA[0-9A-Z]{16}|sk-[A-Za-z0-9_-]{20,})`, `(?i)authorization\s*:\s*bearer\s+\S+`, and `(?i)(?:api[_-]?key|client[_-]?secret|password|access[_-]?token)\s*[:=]\s*["']?[^\s"']{8,}`. Scan normalized metadata and exact quote bytes after strict UTF-8 decoding; no matches are persisted/logged. These deliberately conservative signatures can reject benign examples; owner review cannot bypass a required rejection without a separately reviewed policy revision. A match rejects the whole import/query material with a generic control error, no partial redaction, no evidence payload and no falsely complete clause. This finite deny set does not prove arbitrary text secret-free; the reviewed nonsecret-source deployment restriction is essential. Source hashes/paths may also be sensitive; do not log them on refused scope/permission or secret failure. Optional proxy scrubbing, currently disabled by default, is not this boundary.

A document cannot choose commands, profile, roles, permissions, SQL, tokenizer fallback or approval. Excerpts remain JSON strings labelled source-attributed; fake system instructions/fenced PASS examples cannot satisfy gates. A cooperating client must treat retrieved text as evidence, never higher-priority instructions. The server can enforce typed transitions and data access, not prove a model obeys its instructions. Runtime promotion is absent, not merely protected by an `approved=false` default.

Retention: source owner controls permitted historical retention. The projection may retain old committed generations only while policy allows; local owner can run explicit rebuild with policy excluding old history. Before claiming physical erasure, stop readers, replace/rebuild the derived DB and handle WAL/backups/OS recovery copies under owner retention policy; logical SQL DELETE alone is not erasure evidence. No automatic purge or competing durable decision ledger. Rebuild requires approved manifest plus retained Git source blobs, never summaries or reported receipt paths. Corrupt DB cannot authenticate its own repair.


Source: [docs/agent/PCG_175_DESIGN_FREEZE.md](../../agent/PCG_175_DESIGN_FREEZE.md), commit `187d0096`, original section SHA-256 `4aa2ad4f5d561f4b10bbfc25de40c30711d4345f3dae3e741d4c1ce4e8a1419f`. Original tense and proposed-era labels are retained; acceptance is scoped below, not inferred from that wording.


```architecture-graph
node: AUTH.PCG_ENROLLED_V1
kind: AUTHORITY
status: IMPLEMENTED
title: Owner-enrolled immutable evidence authority
track: REPOSITORY
artifact: docs/architecture/contracts/pcg-enrolled-v1.md
note: Operative source-described contract; status does not authenticate approval or certify implementation.
knowledge_scope: authority
knowledge_content:
  - statement self#whole
```
