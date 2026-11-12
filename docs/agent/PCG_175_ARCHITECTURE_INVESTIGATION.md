# PCG-175: persistent evidence-backed context continuity

Agent: **Agent MaxHeadRoom**

Mission: PCG-175-DISCOVERY / Issue #175

Date: 2026-11-09

Authorization: investigation and design only; owner review is the next gate.
Disposition: **PROCEED TO DESIGN FREEZE for the bounded read-only contract described below.**

## 1. Executive disposition and authorization boundary

**INFERRED:** Persistent decision/evidence continuity is an earned new responsibility composing existing Clean-CTX components (classification **B**), rather than another representation of source IR. Existing persistence survives sessions, but its unit of ownership is a file's compiled context and delta chain. It does not establish who may approve a decision, what proposition a test supports, or which policy revision controls a later request. [C1-C7]

**PROPOSED:** Adopt a staged **Option C, hybrid evidence-backed architecture**, beginning with source-local authoritative records and a rebuildable, isolated SQLite retrieval projection. Defer the runtime evidence ledger and every approval write until the authority principal, correction, retention and transaction contracts are frozen. A single explicit read surface is enough for the first useful slice. No embeddings, automatic memory extraction, automatic context injection, new global cache, or canonical IR changes are needed to establish this slice.

This recommendation freezes neither SQL DDL nor public tool names. It authorizes no implementation. It does not reopen delta, replay, Git, startup or other AF-PERF dispositions. No production Rust, production database, MCP registration, Git index, commit, branch, remote ref or PR was modified. Acquired cloud bytes and acquisition metadata are inspection evidence under `target/pcg-175/cloud/`; this Markdown report is the only intended tracked deliverable.

### Snapshot and evidence labels

| Repository | Examined cloud ref | Immutable commit |
| --- | --- | --- |
| `realworldbooks/GoalLine-Stand-2026` | `main` | `f48228537ac7c44b7c47a71b5cf19c099130e468` |
| `codeliftsleep2/Clean-CTX` | `(fix)Performance-Regression-Issues` | `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3` |

**OBSERVED:** GLS's pinned generated Markdown index reports **421 nodes / 2,402 edges**. This is an index observation, not an executed validator result or frozen requirement. The earlier 417/2,372 figure is historical. The initially inspected local GLS checkout was behind; its 296/1,519 observation and local modifications are excluded from the comparative evidence. Access was initially denied, then restored. All substantive GLS findings below use cloud bytes at the pinned commit. [G1-G3]

**OBSERVED:** Clean-CTX remote `main` was 230 commits behind the selected performance branch. The owner selected the performance branch. Its cloud tree was acquired, and the Git object identities of 41 inspected Clean-CTX files matched the clean local mirror before this report was created; that mirror was used for exact source reading. The proof is `target/pcg-175/cloud/cleanctx-mirror-proof.json`. No current-state claim is based on an unverified, older local GLS copy.

Labels apply to paragraphs/table groups: **OBSERVED** = inspected implementation or artifact assertion; **INFERRED** = architectural conclusion from that evidence; **PROPOSED** = future contract/example; **UNVERIFIED** = execution, authentication, scale or integration not established. Reading an artifact's PASS claim establishes that the artifact reports PASS, not a newly executed PASS.

## 2. GLS implementation trace

### 2.1 Declaration to derived index

**OBSERVED:** `scripts/architecture_graph.py` owns `KINDS`, `STATUSES`, `EDGE_FIELDS`, `NODE_ID_PATTERN`, parsing, validation and projection. `iter_markdown` scans `docs/**/*.md`, excluding generated `docs/architecture/`. `extract_blocks` recognizes top-level `architecture-graph` fences, skipping nested examples through `_skip_fence`. `parse_block` accepts a strict flat scalar/list subset, rejects duplicate/unknown fields and unsupported indentation, and does not ask a YAML loader to infer scalar types. An edge is declared on its source node. [G2]

`load_graph` retains declaration origin and line; duplicate IDs are findings, including malformed duplicates. `validate` checks identity grammar, kinds/statuses, required artifact existence, unknown endpoints, self-edges, selected target-kind constraints, prerequisites, blocker satisfaction, and supersession. It rejects dependencies on falsified/superseded propositions. A `SUPERSEDED` node needs exactly one incoming `SUPERSEDES` successor. `BLOCKED` with no blocker or with only satisfied blockers is stale. Accepted blocker-bearing states require blocker targets in `RECLOSED` or `BANKED`. These are representation/consistency checks, not theorem proving. [G2]

`render_index_yaml` and `render_index_markdown` order output deterministically. `check_index` compares exact UTF-8 bytes to both renderings and emits `E_INDEX_MISSING`/`E_INDEX_STALE`. `write_index` writes derived bytes only after semantic validation. A hand-edited index cannot become independent authority by agreeing with another summary. The YAML view is also derived; it is not an alternative declaration owner. [G2-G3]

Identity is deliberately independent of content: established mission IDs survive report edits; `AUTH.*`, `INV.*` and `DEBT.*` identify longer-lived propositions. The regex accepts both mission-style IDs and infrastructure IDs. A content hash cannot replace these stable identities. [G1-G2]

### 2.2 Dependencies, closure, validation and banking

**OBSERVED:** `Graph.unsatisfied_blockers` checks declared blocker statuses; `blocker_chain` follows unresolved blocker paths; `upstream` follows only `DEPENDS_ON`. Traversal has visited-path protection, but the validator does not generally prohibit multi-node dependency cycles or establish a quantitative request traversal budget. `outgoing` and `incoming` scan the edge list; this small repository implementation is not evidence of scalable indexed retrieval. [G2]

The six required non-equivalences are enforced partly by selected validator rules and partly by doctrine/consumer discipline:

| Distinction | Exact GLS meaning; consequence for PCG |
| --- | --- |
| `DEPENDS_ON` / `USES_EVIDENCE` | Necessary premise versus informative evidence. Failed historical evidence may inform a current conclusion without becoming its prerequisite. |
| `PRESERVES` / `DEPENDS_ON` | Later work respects earlier truth; preservation creates no prerequisite or closure claim. |
| `IMPLEMENTS` / `VALIDATES` | Realization of a contract versus evidence for a proposition. Implementation alone proves no successful test or independent closure. |
| `VALIDATES` / `RECLOSES` | Supporting validation versus an independent closure obligation aimed at a closable artifact. |
| `RECLOSES` / `BANKS` | Independent closure versus an explicit decision to bank the target. A reclosed node is not automatically banked. |
| `SUPERSEDES` / `FALSIFIES` | A newer controlling interpretation versus evidence refuting a proposition. Supersession does not manufacture a past failure; a falsifier can itself remain historical evidence. |

**OBSERVED:** These semantics do not establish authenticated independence. The parser has no authenticated author principal or signature check. `BANKS` and `FALSIFIES` do not each receive a complete dedicated transition/actor authorization proof in `validate`. The report/review/publication process carries responsibilities the graph schema alone cannot enforce. PCG must not mistake a valid declaration for permission to promote a claim. [G1-G2]

### 2.3 Real declaration-to-consumer traces

**OBSERVED, example 1 — P677P:** `docs/investigations/Football_DevelopmentalHumanMorphology_RuntimeSynthesisContract_677P.md` declares `P677P`, currently `RECLOSED`, with `DEPENDS_ON` P658P/P661P/P666P and retained `BLOCKED_BY` P658PR/P681PR. Its `PRESERVES` and `USES_EVIDENCE` links remain different. The declaration explicitly attributes current closure to P677PR and disclaims banking or discharge of other debts. `load_graph` parses the block; `validate` evaluates referenced status/edge rules; `describe` exposes origin/evidence and incoming/outgoing links. [G4]

`docs/investigations/Football_DevelopmentalHumanMorphology_RuntimeMorphologySynthesis_IndependentReclosure_677PR.md` declares `P677PR RECLOSES P677P`, uses P677P-XE as evidence and preserves authority/invariants. It reports independent closure with non-blocking debt. `mission_control_intelligence.analyze` consumes incoming `RECLOSES` evidence separately from incoming validation. A fresh reader must see the exact target and remaining debt, not infer that every future Human Development capability is complete. [G5, G9]

**OBSERVED, example 2 — retained failed science:** `P676P`'s causal-fidelity report preserves P675P's full-oracle failure and classifies its consequence as non-blocking scientific debt. It explicitly does not authorize runtime implementation. P675P's research declaration is `FALSIFIED` and `SUPERSEDES P674P`; P674P remains `SUPERSEDED`. The index records that history rather than treating P675P as a successful prerequisite. `HistoricalTruthTests` exercises this distinction, including the negative dependency controls. [G6-G7, G10]

**OBSERVED, example 3 — banking:** P667FR's actual independent-reclosure report declares both `RECLOSES P667F` and `BANKS P667F`; the generated view carries the separate relationships. This is an explicit banking act, not a general consequence of any `VALIDATES` edge. [G8]

**OBSERVED, example 4 — a failed graph repair remains failed:** GRAPH-002-I retains its historical blocked verdict and a frozen historical attempt; GRAPH-002-XE remains `FALSIFIED`. The debt node's later `RECLOSED` state does not rewrite the old attempt as a success. A current status, an old verdict and the time/subject of their assertions need separate interpretation. [G11]

### 2.4 Git/CI admission and agent consumption

**OBSERVED:** `GitTreeSource` obtains one exact candidate tree's path set and batched blob bytes, rather than reading a dirty working tree. Non-file/symlink candidate entries fail admission. `check_tree` resolves external revisions; `check_staged_tree` obtains the staged tree from `git write-tree`; internal `_check_resolved_tree` validates and optionally synchronizes only deterministic projections. Final staged state is reread before acceptance. An external caller cannot supply a boolean or environment assertion to skip tree resolution. [G2]

`.githooks/pre-commit` runs the encoding guard and `--staged`; `pre-push` passes the Git-provided destination to `--pre-push`. `check_push` examines outgoing commits and the final tip; new refs use advertised accepted remote refs rather than trusting local tracking refs. `agent_commit.admit` invokes the same staged validator; `verify_head` invokes the same committed-tree validator. `publication_owner.PublicationOwner.transition` fences the publication slot generation atomically with candidate pushes. This publication machinery is separate from graph domain truth. [G12-G14]

The pinned PR workflow runs `verify_merge_integrity.py candidate`, rejects request envelopes, checks exclusive publication ownership, validates `--tree HEAD`, and invokes `run_architecture_tests.py --workers 4`. Document integrity runs separately against the candidate. The workflow is read-only; it does not repair failed declarations. Read source tests cover invalid stage/valid worktree, valid stage/invalid worktree, projection synchronization, unrelated unstaged preservation, semantic-failure nonmutation and CI topology. **UNVERIFIED:** hook activation, live branch protection, required-check settings and actual CI execution at this snapshot were not queried or run. [G12-G15]

**OBSERVED:** `AGENTS.md` requires locating the target, dependencies, blockers and successors before work. Beyond the basic `--show`/`--upstream` CLI, `mission_control_intelligence.py` provides concrete read-only consumption: `analyze` calls the canonical parser/validator/index checks, `prerequisite_paths` follows only prerequisites, `blocker_paths` preserves paths, and `report_context` extracts cited observations with artifact SHA-256. Fenced examples cannot discharge obligations. Its result is `advisory_only`; even a readiness candidate does not issue authorization. Report obligation coverage remains `UNKNOWN` because no exhaustive prose schema exists. Snapshot publication metadata cannot certify live GitHub state. Tests include conflicting rows, missing reports, cycles, stale indexes, no-authorization notices and acceptance-versus-reclosure confusion. [G9-G10]

### 2.5 Currentness and truth limits

**OBSERVED:** GLS validates artifact existence and declared consistency, not a general hash-bound proof that a report's conclusions remain true after production changes. Optional `commit` is metadata, not automatic reexecution. MCI hashes inspected reports, but does not authenticate authors or prove exhaustive document obligations. Contradictory prose requires adjudication; graph metadata yields to reports, production and executable evidence. Historical declarations and explicit supersession/reclosure preserve the record. This system earns reusable relationship/governance semantics, not automatic general-purpose memory authority. [G1-G2, G9]

## 3. Clean-CTX existing-system ownership map

**OBSERVED** unless explicitly labelled otherwise:

| Responsibility | Existing owner / production path | PCG boundary |
| --- | --- | --- |
| SQLite lifecycle | `McpState::new` opens configured `SqliteStore`, wrapped by `BufferedStore`; startup rebuilds available statistics. `SqliteStore::open` sets WAL and foreign keys and calls `migrate`. | Reuse SQLite technology and error conventions; database availability is not proof that a decision ledger exists. [C1-C2] |
| Schema / transactions | Migrations 1-6 cover contexts, deltas, symbols, sessions, source hashes, semantic snapshots, edit intents and compatibility identities. `save_context_transaction` writes canonical/semantic state atomically and rolls back on failure. | New evidence schema needs its own approved migration ownership. Do not extend a file-context row into a claim by convention. Migration code is not itself proof of whole-upgrade crash atomicity. [C1-C3] |
| Baseline/history | `ContextStore::save_context` replaces the file baseline and removes obsolete delta ownership; `load_context_with_deltas` restores retained history. `sessions` is documented dead schema; `list_sessions` enumerates contexts. | Destructive baseline replacement/purge is unsuitable for immutable decision history. Do not store claims in `pretty_text`, `metadata` or dead session rows. [C1, C4] |
| Canonical source IR | `PassPipeline::default_production`, compilation result, `ContextState`, physical/canonical identities, `persistence_ir::baseline`. | Source structural authority remains source-owned. PCG references typed source identities without injecting decisions into opcodes. [C5-C6] |
| Structural relationships | `SemanticEdge`, `SemanticDependency`, retained authority and `DurableSemanticSnapshot`; `WorkspaceIndex` holds entity/edge occurrences and reverse adjacency with asserting-file provenance. | Join to source endpoints using exact identity/currentness; never convert an LLM claim to an implicit-confidence-1 structural edge. [C6-C7] |
| Inferences | `InferenceLayer` explicitly separates confidence-scored structural/CBM/heuristic/AI inferences from core wire IR. | Useful conceptual separation; ephemeral confidence is neither durable verification nor approval. [C6] |
| Durable publication | `persist_read_baseline_with_authority` checks retained currentness and configured checkpoint requirements; `save_context_with_retained_semantics` persists exact source and compatibility identities. | A separate evidence transaction must not share source publication ownership accidentally. [C3, C8] |
| Restoration/replay | Registered `restore_context` and `handle_replay_history` load checked durable state, recover pending edits, validate compatibility and publish canonical/semantic projections. | Replay is source context restoration, not retrieval of conversation conclusions. Do not call it from a PCG read. [C4, C9] |
| History inspection | `handle_context_history` reads live/durable version and cache metadata without creating ownership. | Extend only if semantics remain explicit; it currently is not a claim-history API. [C10] |
| Source currentness | `source_cache` stores Arc-backed source with metadata invalidation; explicit edits/refresh invalidate it. Compatibility validation hashes source and checks config/producer/semantic evidence. | Metadata cache presence is not evidence-source verification. PCG needs its own source revision witnesses and must preserve both protected delta reads. [C7, C11] |
| Concurrency | Dispatcher uses worker execution; `McpState` documents lock ordering. File-owner `SemanticPublicationTicket::commit` checks generation/currentness and holds its clock through publication. | An in-process source clock is not cross-process evidence approval isolation. Do not borrow it as a PCG approval lock. [C2, C12] |
| Scoping | `WorkspaceScope::admits` restricts occurrences by asserting-file provenance over root sets and optional narrowing; path identity uses components. | Query scoping is useful, but a workspace path supplied by a caller is not an authenticated tenant/principal boundary. [C13] |
| MCP reachability | Stdio server -> dispatcher -> `router::dispatch` -> tools dispatch -> `create_default_registry` -> handler -> protocol response. Tool schema and handler parity matter. | A type, database or internal evaluator alone cannot establish PCG production reachability. [C14] |
| Presentation/economics | `presentation_document`, `select_complete_content` and local tokenizer economics choose code presentation, sometimes raw passthrough. | Reuse tokenizers; do not reuse code's raw fallback or treat compression savings as a mandatory-constraint completeness contract. [C15] |
| Secret filtering | Proxy `scrub_secrets` transforms selected text; `ProxyConfig` defaults scrubbing off. | No inspected evidence establishes mandatory secret scrubbing before canonical SQLite writes. A proxy setting cannot be the new ledger's security boundary. [C16] |

**INFERRED:** Immediate-consumer removal succeeds. Even without Clean-CTX engineering governance, a coding agent needs a decision's applicability; a policy assistant needs an effective approved version; research needs failed experiments and correction history; cooperating agents need non-self-promoting proposals. These consumers share evidence identity, revision, scope, authority and bounded projection. They do not require shared football mission names or automatic transcript capture. This establishes a reusable responsibility, not measured market demand or proven production workload frequency.

## 4. GLS-to-Clean-CTX semantic mapping matrix

**PROPOSED** mapping based on the observed distinctions above:

| GLS concept | Reuse / adapt / omit | Minimal Clean-CTX interpretation |
| --- | --- | --- |
| Source-local declaration and derived indexes | Reuse | Evidence source owns its assertions; retrieval projection is rebuildable. |
| Stable mission IDs | Adapt | Preserve imported IDs under a source/partition namespace; runtime records receive stable IDs independent of content. |
| MISSION / RESEARCH / IMPLEMENTATION / EXPERIMENT | Adapt | Artifact role metadata; do not require four universal node classes. An experiment receipt is an evidence revision, not automatically a verified claim. |
| AUTHORITY | Adapt | Authenticated owner/capability and source policy, separate from a domain authority proposition. |
| INVARIANT | Adapt | Constraint record with explicit applicability and required evidence. |
| DEBT | Adapt | An unsatisfied obligation/claim with explicit blocking relationship; debt need not block all consumers. |
| Twelve overloaded statuses | Omit as universal enum | Separate authority, verification/outcome, lifecycle and freshness. Preserve imported GLS status as source metadata. |
| Thirteen edges | Adapt selectively | Preserve each non-equivalence; import unsupported lifecycle semantics as typed descriptive links, never executable approval. |
| `DEPENDS_ON` / `BLOCKED_BY` | Reuse with a policy | Required premise versus unresolved gate; satisfaction predicate belongs to the contract, not universal GLS status names. |
| `USES_EVIDENCE` / `VALIDATES` / `FALSIFIES` | Adapt | Citation/support/refutation plus exact verification subject and receipt outcome. Support does not promote authority. |
| `PRESERVES` / `IMPLEMENTS` | Retain when needed | Respect/realize a revision; neither means verified, approved or prerequisite. |
| `RECLOSES` / `BANKS` | Domain-specific profile | Explicit independent review versus protected closure decision. No need for those labels in every enterprise/research workflow. |
| `SUPERSEDES` | Reuse | New controlling revision/record points to older, with explicit scope/effectivity and authorized transition. |
| `DEEPENS`, produces/consumes authority | Optional profile | Preserve imported meaning; defer universal executable semantics until a real consumer needs them. |
| MCI advisory citations and unknown coverage | Reuse | Report declared evidence and uncertainty; never infer authorization or complete prose coverage. |
| Git hooks/publication slot/request envelopes | Omit from core PCG | Repository-specific admission adapters, not general memory locks, task scheduling or approval authentication. |

## 5. Five general-purpose use cases

**PROPOSED:** All five use the same evidence/record/revision core. These are acceptance scenarios, not implemented workflows.

| Requirement | UC-1: coding continuity | UC-2: cross-session memory |
| --- | --- | --- |
| Input / trigger | Explicit pre-edit query with exact repository, file/entity and task. | User explicitly requests retention of a bounded statement or decision; later exact-topic query. |
| Authority source | Approved design/invariant, source and executable receipts with exact subjects. | Attributable user/owner statement; external factual assertion needs its own evidence. A user preference and a universal fact have different scopes. |
| Persistence | Stable decision/constraint IDs, source/record revisions, citations and verification obligations. | Minimal consented record, provenance and correction chain; no automatic entire-transcript storage. |
| Retrieval | Applicable constraints plus prerequisites, rejected approaches and exact tests. | Current authorized revision, its attribution and corrections; no transcript needed. |
| Freshness | Source/config/receipt witness drift; changed applicability requires revalidation. | Expiry, changed source, explicit correction/retraction; superseded text stays historical only where permitted. |
| Security / isolation | Authorized repository/root set; evidence ACL checked independently of source visibility. | User/principal partition, consent and historical-access permission; no cross-user dedup leak. |
| Projection | Exact directive, scope, reason, citation and required test; optional history separately. | Attributed statement, epistemic category, effective period, citation and uncertainty. |
| Failure | Missing/stale required authority blocks an actionable complete pre-edit result. | An unverified model extraction is a proposal; correction conflict returns unresolved, not a fabricated fact. |
| Existing contribution | Source identity, structural graph, file context, tokenizer and persistence primitives. | SQLite and protocol primitives only; source-context history does not satisfy durable memory authority. |

| Requirement | UC-3: multiple agents | UC-4: enterprise knowledge | UC-5: long research |
| --- | --- | --- | --- |
| Input / trigger | Agent A submits evidence/proposal; B queries the same scoped task. | Query policy/contract for subject, jurisdiction, version and effective date. | Record hypothesis/experiment; later compare evidence and conclusions. |
| Authority source | Raw artifact/receipt; separate authorized reviewer for promotion. | Owner-approved policy/contract revision and authenticated approval. | Experiment data/protocol and attributed interpretation; scientific support does not imply governance approval. |
| Persistence | Proposal immutable revision, author identity, approval/rejection events. | Source revisions, effectivity, ACL, owner and signed/host-authenticated approval evidence. | Protocol/data hashes, outcomes, counterexamples, hypotheses and revision history. |
| Retrieval | Pending versus approved records visibly distinct. | Applicable current approved policy; conflicting approved records exposed. | Supports/refutes paths plus current interpretation and retained failed findings. |
| Freshness | Expected revision/generation for promotion; agent session termination removes no accepted evidence. | Source version/permission/effective-date checks; revoked policy cannot remain controlling via a summary. | Changed dataset/protocol invalidates dependent verification, not the fact that an old run occurred. |
| Security / isolation | Agent identity and capabilities supplied by trusted host, not agent text. | Tenant/project/source ACL intersection, residency, retention and historical permissions. | Project embargoes, participant data restrictions and dataset licenses. |
| Projection | Who proposed/approved what, exact evidence and remaining gates. | Controlling clauses with citations/effectivity; historical alternatives labelled. | Hypothesis, experiment subject/outcome, contrary evidence and limits of conclusion. |
| Failure | Self-approval rejected; competing authorized updates conflict transactionally. | Missing permitted controlling authority yields blocked/incomplete; no leakage of hidden titles/counts. | Conflicting evidence retained; no silent erasure or averaging into a confident answer. |
| Existing contribution | Registered tool boundary and source-currentness patterns, not authenticated approval. | Scoped source querying is useful but does not prove enterprise tenancy or approval identity. | Confidence/source separation is conceptually useful; no inspected durable research ledger. |

## 6. Persistence architecture alternatives

**PROPOSED:** The following compares ownership alternatives, not three implementations or measured benchmarks.

| Dimension | A: source-local + derived SQLite | B: SQLite-native evidence ledger | C: hybrid |
| --- | --- | --- | --- |
| Source of truth | Authoritative source artifacts; DB only projection. | DB owns recorded evidence/claims/revisions; external documents retain domain authority. | Explicit source-local owner or runtime-ledger owner per record, never both. |
| Write owner | Repository/document owner and admission adapter. | Authenticated ledger service/transaction owner. | Source adapter for imports; ledger boundary for runtime assertions. |
| Stable identity | Source namespace + declared ID, independent of path/content. | Partition + stable record ID and immutable revision ID. | Shared reference format with explicit origin namespace. |
| Transaction | One immutable source generation imported atomically. | Evidence/claim/approval/edges/head revision in one SQLite transaction. | Each source generation and ledger transaction atomic independently; read manifest pins the composition vector. |
| Crash / restart | Discard incomplete projection; reimport immutable sources. | WAL/transaction recovery; acknowledged durable commits survive reopen. | Rebuild projection from both owners; ledger backup required for runtime history. |
| Concurrent writers | Source system resolves edits before projection. | Expected-revision compare-and-swap; approval serialization within partition/subject. | Same plus generation checks across source adapters. |
| Duplicate ingestion | Same source revision + declared ID is idempotent. | Scoped idempotency key; same content in a different source is not same authority. | Stable mapping per origin and partition, not global hash deduplication. |
| Rebuildability | Full if authoritative source revisions remain available. | Index rebuildable; lost ledger content is not reconstructable from its index. | Source projection rebuildable; runtime ledger remains durable non-derived data. |
| Migration | Reparse versioned declaration schema into new projection. | Transactional ledger migrations, backups, fault controls and recovery. | Independent versioned adapters/ledger/projection; no source IR migration dependency. |
| Schema evolution | Source schema negotiation; unknown required fields fail visibly. | Versioned record encoding and migration policy. | Imported original representation retained alongside normalized semantics. |
| Immutability | Exact source revision is immutable; working source can change. | Revisions/events logically immutable except authorized erasure. | Same distinction; no rewriting imported history from ledger. |
| Supersession / deletion | Source-owned revisions/tombstones; archive access policy required. | Authorized supersession/retraction and retention-aware erasure. | Explicit origin owner controls lifecycle; unified query does not own deletion rights. |
| Source drift | Hash/revision mismatch invalidates applicability projection. | External evidence references can still drift or disappear. | Independent freshness witnesses; source owner controls domain truth. |
| Token / storage cost | Small graph index; may need permitted source fetches. | Stores selected evidence payloads/revisions; greater retention cost. | Avoid duplicate bodies; retain exact excerpts/manifests only when necessary/authorized. |
| Query complexity | Indexed typed traversal + source validation. | Indexed traversal + authority/revision resolution. | Same with origin and snapshot-vector checks. |
| Workspace / tenant isolation | Partition every imported identity; repository path alone insufficient. | Partition keys/FKs plus trusted principal and per-source authorization. | Apply both at ingress, traversal, projection and history. |
| Existing compatibility | Reuses SQLite primitives; separate projection lifecycle. | Existing `contexts` replacement/purge semantics cannot be reused as ledger semantics. | Can compose without changing canonical schema or publication clocks; more owner boundaries to test. |

**INFERRED recommendation:** A alone is the smallest useful coding/governance pilot but cannot preserve an explicitly approved runtime decision that has no source-local home. B alone unnecessarily centralizes repository evidence and invites a competing document truth. C preserves both legitimate ownership patterns. Implement A's read-only/import slice first within C's explicit origin contract; add ledger writes only when UC-2/3 acceptance and authenticated ownership justify them. Avoid a generic plugin/adapter framework: one source adapter and one direct query composition suffice initially.

## 7. Recommended authority and provenance model

**PROPOSED:** A claim has four independent dimensions; model confidence is optional descriptive metadata and cannot change any of them.

| Dimension | Values / responsibility |
| --- | --- |
| Origin / authority | Model hypothesis; unverified extraction; attributable observation; verified claim with exact verifier/subject; explicitly approved decision with approver capability and policy revision. |
| Verification outcome | Unknown, supported, contradicted, inconclusive; include method, subject revision and executed receipt. Passing a counterexample detector supports divergence, not equivalence. |
| Lifecycle | Active, superseded, retracted/falsified; immutable transition history with effective interval. |
| Evidence availability/currentness | Current, stale, missing, inaccessible, invalid/corrupt, or not checked in this read. These do not erase historical observation. |

Only the authoritative owner can decide that a scope-bound proposal becomes controlling. A document declaration saying `approved: true`, an agent identity in prose, or a high confidence score is insufficient. For source imports, trust policy must name the accepted repository/source and approval evidence; the initial importer may report `source-declared approved` without certifying authenticated approval. Runtime approval requires an authenticated capability from outside model-controlled arguments.

Approval binds exact record revision, evidence revisions, scope/effective interval, policy revision and expected current head. Approval is not a permission to approve future revisions. Contradictory approved decisions with overlapping applicability block action pending owner disposition; timestamp, rank and confidence cannot choose the winner. A later effective policy controls only through an authorized supersession/effectivity rule. Observation provenance can be certain while its interpretation remains unverified.

## 8. Proposed logical schema and relationship semantics

**PROPOSED, logical only:** No SQL production schema or Rust types are added here.

| Concept | Owned responsibility / minimum content |
| --- | --- |
| Partition/scope identity | Trusted owner/tenant + project/workspace + explicit audience; applicability selectors and effective interval. Not another source entity type. |
| Evidence revision | Origin system, source stable key, immutable revision/hash, exact locator/section/span, media/parser version, availability and authorized sanitized payload/reference. Owns provenance, not the asserted conclusion. |
| Record revision | Stable ID + revision, role (`claim`, `decision`, `constraint`), content, applicability, origin and lifecycle/verification dimensions. A hypothesis is a claim, not another table. An invariant is a constraint. |
| Relation assertion | Typed directional source-revision -> target-revision link, assertion evidence and scope. Owns relationship provenance; must not silently retarget to the latest endpoint revision. |
| Authority event | Attributable proposal/verification/approval/rejection/supersession/retraction event, exact subject/evidence revisions, policy and authenticated actor. Runtime ledger owns these; imported events remain attributed external assertions. |
| Projection generation | Source revision vector + ledger high-water mark + parser/policy versions, published atomically after validation. Owns retrieval snapshot coherence and rebuild state, not truth. |

Entities remain typed references to existing canonical source endpoints or scoped external subjects; no parallel code-symbol registry. Scope can be a common key/validated value rather than an elaborate hierarchy of tenant/workspace/session tables. Session IDs identify an interaction, not durable truth ownership.

Constraints: composite partition keys on every durable identity and foreign key; unique `(partition, origin, source_key, declared_record_id)` mapping; immutable revision identity; unique scoped ingestion idempotency key; one controlling head per explicitly single-valued decision subject/effective region. Nonidentical payload under the same idempotency key is an error. Identical content under different policy sources remains distinct. Preserve original imported IDs; rename via an explicit source mapping, never title similarity. Index forward/reverse relation endpoints, applicability selectors, record heads and evidence source references.

Minimal executable relations: `DEPENDS_ON` (record -> required premise), `BLOCKED_BY` (action/decision -> obligation), `CONSTRAINS` (constraint -> subject/scope), `USES_EVIDENCE` (record -> evidence), `VALIDATES` and `FALSIFIES` (verification evidence/claim -> exact proposition), `SUPERSEDES` (new -> old). Preserve `PRESERVES` and `IMPLEMENTS` descriptively when importing the dogfood corpus. `RECLOSES` and `BANKS` remain explicit profile events/links with independent actor requirements; they never collapse into validation. Optional relations need no executable policy until an acceptance scenario earns it.

Cycle rules: reject supersession cycles and conflicting controlling heads; report dependency strongly connected components as unresolved required closure unless the governing policy explicitly defines a valid fixed-point meaning. Evidence-use/history cycles may exist; visited sets terminate traversal without asserting prerequisite satisfaction. Endpoints and applicability are validated before projection publication.

## 9. Retrieval and context-projection design

**PROPOSED:** Begin with explicit retrieval. Automatic pre-edit invocation is a later, separately approved client integration. Lexical lookup and exact source/entity/subject selectors discover candidates; structured applicability and graph closure establish required context. Embeddings are unnecessary for the first slice and could only suggest candidates later. Similarity cannot establish scope, completeness or approval.

1. Resolve the trusted caller partition and authorized source set; reject caller attempts to widen it. Resolve exact requested subjects, source entities and applicability/effective date. Ambiguous identity returns unresolved.
2. Pin a projection generation and its source/ledger revision vector. Enumerate all matching constraints from the registered authoritative corpus, independently of lexical top-k results. Unknown applicability is a potential required constraint and blocks an actionable complete result.
3. Traverse the declared prerequisite/blocker closure and necessary approval/evidence references with stable edge/ID ordering, a visited set and explicit per-request work limits. Use indexed adjacency. Validate a distinct evidence revision once per operation, not once per candidate.
4. Recheck permissions, source witnesses, approved-head/conflict state and policy revisions. For mutable source systems without immutable snapshots, use source version/ETag checks or report the read boundary as unverified. Do not claim a cross-origin global transaction.
5. Build the required authority bundle: exact directive/constraint, applicability, authority/verification/lifecycle, mandatory exception/condition, unresolved blockers and citations. Then include optional support and history in deterministic priority order.
6. Count the complete serialized envelope with the selected local tokenizer, including citations/metadata. Preserve mandatory meaning with exact clauses or an owner-approved lossless structured representation. Arbitrary LLM summarization cannot certify it. Fit optional content only after the required bundle.

The completeness boundary is **all applicable registered declarations and their required closure in the named partition/corpus generation under the named authority policy**, not all knowledge in the workspace or all prose in a document. Corpus enrollment and negative-lookup coverage are explicit. A source missing from the enrolled corpus cannot be silently certified absent; a natural-language discovery query alone returns discovery results with unknown completeness. Recursive impact queries need separately closed incoming-edge acquisition, not only an indexed outgoing neighborhood.

Proposed envelope fields: `scope`, `snapshot`, `authority_policy_revision`, `requested_subjects`, `required_records`, `optional_support`, `historical_records`, `conflicts`, `missing_or_stale_requirements`, `coverage`, `budget`, `citations`, `result_state`. Each citation binds origin, immutable source revision/hash, exact section/span and record revision. Derived summary text identifies itself and its source revision; it never replaces evidence.

Result states: `complete_within_registered_scope`, `incomplete`, `blocked`, `unavailable`. Coverage independently reports corpus acquisition, identity/applicability, required traversal, evidence validation and projection. Optional history truncation is explicit and does not falsify required completeness. If required authority exceeds the token budget, return `blocked`/`incomplete`, the known requirement IDs that can safely fit, and a permitted required-token estimate; never claim that the emitted subset is enough to proceed. If even the control envelope does not fit, return a bounded protocol error. Pagination may transport an immutable required bundle, but no page alone grants completeness; the caller must acknowledge the same complete bundle before a complete-context assertion.

Source-hash drift does not revoke a protected decision automatically. It invalidates any claim of current applicability supported only by the old witness; return the protected historical instruction with its uncertainty when permitted, and block the affected action until the owner revalidates. Missing mandatory evidence also blocks. Historical contradictory findings can remain visible with warnings without overriding the currently applicable approved decision.

## 10. MCP/API integration alternatives

**OBSERVED:** The real public path is server/dispatcher/router/tool registry/handler/response, and current tools' context restoration changes source-owned state. Reusing a tool name is not proof of semantic compatibility. [C9-C10, C14]

**PROPOSED alternatives:** (1) extend `workspace_query` with an explicit evidence-context operation, sharing transport/batching but keeping a separate authority evaluator; (2) expose one versioned read-only context-evidence query supporting constraint selection, provenance and bounded impact; (3) inject PCG context into every `provide_code_context` response. Recommend evaluating (1) against the actual query schema during design freeze; choose (2) if adding non-source subjects overloads existing query semantics. Defer (3): it changes observable code output, token economics and omission semantics. Do not add six tools merely because six operation names were suggested.

| Boundary | Initial exposure / permission |
| --- | --- |
| Retrieval and trace | Explicit read-only query; no persistence flush, source restore, IR mutation or approval side effect. |
| Evidence ingestion | Separate idempotent source import or runtime evidence write; attribution, scrub and ACL validation; never implicit approval. |
| Claim proposal | Separate non-authoritative write, returning pending revision. |
| Authority approval | Trusted host/admin control path bound to exact revisions; absent from ordinary agent capabilities by default. |
| Supersession | Authorized lifecycle transaction with conflict checks and retained historical provenance. |
| Impact | Read mode with incoming-closure coverage stated explicitly; no inferred closed-world answer from a partial index. |
| Maintenance | Separate migration/rebuild/retention owner; unavailable to normal retrieval/ingestion callers. |

Preserve existing MCP response bytes/contracts when PCG is not explicitly requested. Add schemas, dispatch parity, content-channel and compatibility tests only in an authorized implementation campaign. Reuse local tokenizers through a separate evidence projection boundary; leave code's raw-passthrough economics, canonical publication, cache hints, replay rendering and delta source checks unchanged.

## 11. Security, concurrency and invalidation contracts

**PROPOSED:** The first deliverable is a trusted local, single-owner workspace profile. This is a deliberate deployment limit, not enterprise security certification. Even there, source/document content is untrusted data. Multi-tenant hosting and runtime approval require a trusted principal/ACL adapter before exposure.

| Threat / interruption | Required behavior |
| --- | --- |
| Cross-tenant/project/root leakage | Authorize partition/source scope before candidate discovery and on every edge join/output. No hidden names, snippets, hashes, counts or timing diagnostics that reveal forbidden records. Cross-root links require explicit permitted scope. |
| Historical disclosure | Recheck permissions on every historical retrieval; former access does not entitle current access. Hidden mandatory authority produces a generic blocked result, not an explanatory leak. |
| Secrets | Scrub/minimize before durable payloads, indexes, logs and projections; recheck at retrieval. If sanitization destroys required meaning, refuse authoritative ingestion. Optional proxy scrubbing does not satisfy this. Treat hashes and locators as potentially sensitive too. |
| Prompt injection | Typed records and quoted excerpts are data; retrieved text cannot alter tool permissions, actor identity, approval state or system policy. No content-driven tool execution. Client instruction compliance remains a client boundary, not something SQLite can prove. |
| Unauthorized promotion | Reject caller-supplied role/approval flags without trusted capability. A valid source declaration establishes attribution only until source approval trust is verified. |
| Conflicting approvals | Compare expected subject head/effectivity; one winner for a single-valued subject. Multiple authorized overlapping conflicting records block projection pending adjudication. |
| Concurrent approval/supersession | Transaction covers exact revisions, authorization policy check, edges, controlling-head CAS and generation increment. No success before durable commit; stale generation fails or requires explicit retry. |
| Source drift / deletion / inaccessible evidence | Required current authority blocks; permitted old observations remain historical, marked stale/missing/inaccessible. Never substitute a cached summary as current proof. |
| Interrupted transaction / process crash | SQLite rollback/recovery restores a whole committed state; no partial approval or orphan endpoint. Source imports publish only complete generations. Crash tests must verify actual reopen behavior. |
| Corrupt DB / failed migration | Fail closed for authority retrieval/writes; quarantine/diagnose without leaking data. Rebuild only derived projections. Ledger loss needs validated backup, not invention from an index. |
| Duplicate ingestion | Scoped idempotency; conflicting repeated key rejects. No cross-tenant content dedup or status promotion from repeated observations. |
| Cycles / traversal explosion | Deterministic visited traversal, cycle diagnostics, explicit limits and incomplete signaling. Hitting a bound never proves absence or discharges an obligation. |
| Excessive context expansion | Required bundle first; complete token accounting; blocked/incomplete if mandatory material cannot fit. |
| Retention / deletion | Owner-approved retention applies to source payload, excerpts, projections, WAL/backups and logs. Logical immutability does not defeat lawful erasure. Retain only a non-disclosing tombstone if permitted. |

Source-local ingestion pins a source generation and swaps a validated projection in one transaction. Runtime ledger ingestion/approval commits its event and projection generation atomically, or marks projection pending and refuses complete retrieval until caught up. There is no proposed distributed transaction spanning Git and SQLite. A unified read pins a vector of independently committed origins and reports that boundary. ACL changes are checked outside stale cached projections. No new retry/global cache policy is authorized by this design.

## 12. AF-PERF dogfood examples

**OBSERVED corpus:** the pinned Clean-CTX documents and active source/test files below exist on the selected cloud branch. Executed results are reported by those artifacts; generated owner-run receipts were not reexecuted or independently rehashed in this mission. **All node IDs, edges, queries and context excerpts in this section are PROPOSED representations, not production PCG data.**

**PROPOSED admission prerequisite:** AF-PERF reports do not already contain PCG declarations. The owner must enroll an exact, reviewed source-section-to-record mapping and its trust policy before these examples can drive authoritative retrieval. Automatic prose extraction may propose that mapping, never approve it. A sidecar import manifest identifies exact report revisions and sections; it must not become a separately maintained decision body or manufacture missing executable receipts. Until approved enrollment, return source-attributed discovery with unknown required-constraint coverage.

### A. Protected delta previous-authority capture

Proposed records: `decision:delta-previous-capture@1` (source-declared approved, CLOSED/FIX/PROTECTED), `constraint:delta-same-guard@1`, source evidence revisions for the repair report and `src/mcp/tool_handlers/core/delta.rs`, and verification records for original RED, restored-current GREEN, mutation and composed acceptance. Link the decision `USES_EVIDENCE` to each exact receipt reference, `CONSTRAINS` the delayed capture site, and `DEPENDS_ON` the same canonical guard/currentness invariant. Receipt hashes/subjects belong to evidence revisions, not free-text PASS tags. [C17]

Query: constraints for `delta_code_context`, exact `delta.rs` owner, proposed task "move previous-authority capture". Projected context: "Protected delayed capture: cached returns must perform zero unused previous snapshots. Capture remains in the original canonical guard; changed-source capture, required cached-output construction and both source reads remain required. Evidence: repair report, final composed acceptance section; source revision 8ce0a686… . No universal latency improvement is claimed." Expected agent behavior: preserve the correction and request explicit owner disposition before contradicting it; do not optimize merely from syntax.

### B. Deferred second delta read

Record `decision:delta-second-read@1` has disposition DEFERRED, not CLOSED/FIX. Its constraint points to the second `state.read_source` under `ir_context_lock` and the currentness rationale; it `USES_EVIDENCE` from the delta design/repair reports. Do not infer a duplicate acquisition from two textual reads: metadata invalidation, hashes and lock boundaries matter. [C11, C17]

Query: constraints for task "remove the second source read". Projected context: "Deferred: the second delta read protects currentness at the guarded cached-return boundary. Removing it requires a separately authorized equivalence/currentness design. Prior capture repair did not authorize this change." Expected behavior: leave the read in place; no implicit reauthorization from the completed capture correction.

### C. Rejected earlier replay presentation reuse

Records: `proposal:replay-earlier-candidate@1`, rejected decision, registered-deletion counterexample evidence, and `constraint:replay-later-footer@1`. The counterexample `FALSIFIES` the exact equivalence claim. The focused active regression `src/tests/mcp/replay_footer_observation_af_perf_002.rs::af002_replay_footer_observes_dictionary_after_economics` `VALIDATES` the required later observation; retired comparison machinery is historical evidence only. The rejection does not define a new successful replay-after-deletion lifecycle. [C18-C19]

Query: constraints for `replay_history`, task "reuse economics candidate instead of rendering again". Projected context: "REJECTED: registered deletion changes the later dictionary/footer state. The second render observes state the earlier owned string cannot preserve. Active regression exercises unrelated insertion, exact-alias removal and registered deletion. Passing the counterexample detector demonstrates divergence, not semantic equivalence. Historical cost medians show no consistent benefit." Expected behavior: retain the later render; no new snapshot/lock/retry policy to rescue the proposal.

### D. Deferred Git ownership candidates

Use two decision records: `decision:gitdiff-body-copy@1` and `decision:gitdiff-empty-untracked-set@1`, both DEFERRED, with shared measured evidence revision but distinct subjects. The report records 88 body-site copies / 6,084 logical bytes across thirty requests, and fifteen set constructions / 48 entries / 436 logical path bytes. These are fixture logical-work observations, not allocator, latency or enterprise-frequency results. [C20]

Query: constraints/previous decisions for `diff_commits`, trim-to-owned body or empty-untracked duplicate-set construction. Projected context: "Measured DEFER: observed bodies are 26..965 bytes/file; largest set eight entries/72 bytes. Construction-inclusive performance benefit is unproved. Preserve Unicode trim behavior, duplicate suppression, ordering, refs/path admission and required snapshots. No correction is recommended by this discovery." Expected behavior: do not reopen from count reduction alone; a larger credible workload and owner authorization would be prerequisites for a future design.

### E. Historical failed and incomplete packets

Each packet receives its own evidence revision with execution state (`incomplete`, `failed`, `completed`), exact tested proposition, command/environment, source identities and receipt locator. Link the later complete report to the earlier packet with descriptive follow-up evidence, not a claim that the old run passed. A compiled test's successful detection of a counterexample has outcome PASS for detection and contradiction for equivalence. [C17-C20]

Query: "What proves current replay equivalence/delta closure/Git discovery?" Projected context includes the final applicable packet plus the distinction: Git's first warm-cache equality packet failed on its fixture assumption; the later packet separately reports completion. Delta's failed/interrupted runs remain distinct from final 110 tracked passes across 55 groups. Replay comparison tests can pass while disproving reuse. Expected behavior: cite the correct subject and source revision, distinguish tracked tests from operator evidence and historical source archives, and never present an older partial packet as a complete gate.

These records prevent accidental reopening only if applicability is enrolled and the client actually retrieves and respects the bundle. They cannot stop an arbitrary non-cooperating agent from editing files. Source/action enforcement would be a separately approved integration, not a property of graph storage.

## 13. Executable acceptance and measurement plan

**PROPOSED, UNRUN:** Implement required Rust regressions under tracked `src/tests/**` with normal `#[path]` registration only after authorization. Reusable stdio drivers/fixtures belong under `verification/**`; generated databases/receipts under `target/**`. No scratch graph example or live harness substitutes for regression/CI coverage.

| # | Executable acceptance | Required negative control |
| --- | --- | --- |
| 1 | Separate process writes approved fixture, exits, fresh process queries exact revisions. | Kill before commit/ack; reopened DB exposes no partial record or false durable success. |
| 2 | Repeated ingestion returns same stable record and revision. | Same key/different payload rejects; same body/different origin stays distinct. |
| 3 | Every emitted citation resolves exact source revision/span/hash. | Retarget locator or hash; required evidence becomes stale/invalid. |
| 4 | Trusted reviewer capability promotes exact revision under exact policy. | Approval of revision N cannot approve N+1 or another scope. |
| 5 | Ordinary agent can propose without approval. | Forged role, high confidence, `approved=true` and retrieved injection all fail promotion. |
| 6 | Authorized supersession selects new head and retains old evidence. | Unapproved supersession/cycle/conflicting head rejects; erasure obeys separate retention policy. |
| 7 | Changed source bytes invalidate dependent current verification. | Same-size/mtime-preserving change cannot pass an exact witness check. |
| 8 | Deleted/unreadable required evidence returns blocked/incomplete. | Cached summary cannot make the query complete; optional historical loss is labelled. |
| 9 | Overlapping contradictory approved decisions are returned as conflict. | Higher rank/confidence/newer timestamp cannot silently win. |
| 10 | Two DB connections race promotion/supersession using barriers; exactly one valid head commits. | Stale writer loses CAS; rollback leaves no detached relation/event. |
| 11 | Identical IDs/content in two partitions remain isolated end to end. | Traversal, counts, search, error text and history expose no forbidden source. |
| 12 | Insertion-order permutations yield identical traversal/envelope ordering. | Cycles terminate deterministically and do not satisfy prerequisites. |
| 13 | Complete required dependency/blocker closure yields explicit paths. | `USES_EVIDENCE`, `PRESERVES`, `VALIDATES` cannot discharge prerequisites/reclosure. |
| 14 | Exact serialized envelope token accounting respects request budget. | Required constraint larger than budget blocks; citation/metadata tokens count too. |
| 15 | Truncation/acquisition failure/unknown applicability reports incomplete. | Empty lexical results or a traversal cap cannot certify no constraints. |
| 16 | PCG reads leave canonical tuples, source hash/version, semantic projection, dictionary and durable source rows unchanged. | Run around protected delta/replay interleavings; detect any publication/cache-owner mutation. |
| 17 | Existing tools retain complete responses when PCG is absent; new read path is registered and schema-valid. | Unsupported mode/capability fails explicitly; no accidental code-output injection. |
| 18 | Each supported schema upgrades/reopens/rebuilds the same normalized graph. | Interrupted upgrade/corrupt ledger fails closed; projection rebuild cannot invent missing runtime evidence. |
| 19 | Malicious source prose remains quoted data with unchanged capability/approval state. | Fake declarations in fenced examples, forged authorization text and hidden ACL material cannot promote or leak. |
| 20 | Fresh sessions query AF-PERF A-E with exact expected scope, disposition, citations and active regression. | Remove enrollment/evidence, swap failed/final packet or equate detector PASS with equivalence; acceptance must fail visibly. |

Additional lifecycle controls: read at approval/source/ACL generation change, permission revocation between discovery and emission, source rename/restoration, source parser upgrade, source-policy revocation and legal deletion including old projection copies. A read can return its explicitly pinned historical source snapshot, but it cannot assert current permission after revocation. Public stdio tests must exercise the real registry and durable reopen, not a custom-only pipeline.

### Canonical question matrix for the bounded slice

| Question class / question | Required proof and public path | Answer / completeness / adversary |
| --- | --- | --- |
| Identity: which decision constrains this exact source owner? | Stable scoped record + canonical endpoint/applicability; proposed explicit read query. | Record/revision/source tuple; same-name different-root negative. |
| Forward/composition: what required premises and blockers apply? | Registered authority universe + deterministic required closure. | Typed paths and unresolved obligations; cycle/missing edge/budget controls. |
| Reverse/impact: which decisions depend on changing this evidence? | Complete incoming-reference acquisition for named corpus generation. | Impact paths with exact-versus-incomplete coverage; unacquired dependent control. |
| Why/provenance: why is this protected or rejected? | Exact approved revision + cited evidence/verification subject. | Attributed conclusion, reason, receipt and constraints; forged PASS control. |
| Scope/visibility: does this policy apply here and now? | Trusted partition/source authorization, effective interval and exact selectors. | Applicable/current/conflict state; foreign root, hidden history and overlapping policy controls. |
| Currentness/lifecycle: what changed or superseded this result? | Revision chain + current evidence/policy witnesses. | Current and historical assertions separated; same-size source drift and stale approval controls. |
| Completeness/negative: are there no applicable constraints? | Exhaustively enrolled scope and closed required acquisition, not text search. | Complete only within named registered universe; unenrolled corpus returns unknown. |

Every row is **PROPOSED / UNIMPLEMENTED / UNRUN**. This matrix must be expanded into exact public query schemas, tracked test names and consumer audit before an implementation campaign can claim the capability is integrated.

### Metrics and AF-PERF discipline

**PROPOSED:** Measure exact expected required-record/edge recall against a frozen owner-reviewed corpus; false-authority promotions, missed mandatory constraints, wrong-scope disclosures and false-complete answers must be zero in the acceptance populations. Report discovery precision separately from authority/completeness. Token savings = comparable full baseline tokens minus complete envelope tokens, including citations/metadata; do not compare a partial projection against a complete transcript and call the omission a saving.

Record cold/restart/warm end-to-end latency distributions, ingestion/approval/rebuild latency, peak memory and DB/WAL/storage growth per evidence/record revision. State tokenizer, hardware, corpus identities, ACL distribution, revision depth and request population. Include construction, source checks, permission checks, serialization and cleanup. No measured PCG speedup, storage efficiency or enterprise token saving is claimed in this report.

AF-PERF-001 scope is one import generation or one query over a pinned authorized corpus vector. Expensive complete-set work comprises applicability enumeration, required closure, distinct evidence validation and complete envelope tokenization. Expected indexed work is proportional to examined applicability candidates plus visited vertices/edges, distinct evidence bytes validated and output bytes, with indexed lookups/sorting costs disclosed. Reuse each distinct exact revision's validated result only within the operation. Invalidation boundary is source/policy/ACL/ledger generation, not a global memo cache. Deterministic tracked counters must prove no per-candidate reconstruction of the same authority set across controlled N/2N/4N growth, and include enclosing-operation costs.

AF-PERF-002 requires evidence-backed FIX/MONITOR/ACCEPT for actual performance candidates. Controlled growth can establish scaling failure; fewer counted operations alone cannot establish overall benefit. Large realistic permitted corpora are needed before enterprise claims; isolated fixtures are for correctness/scaling controls only. Existing AF-PERF corrections remain protected.

## 14. Incremental implementation milestones — planning only

**PROPOSED:** Preserve the useful sequence but reduce initial write/API scope.

| Milestone | Bounded deliverable / exit evidence |
| --- | --- |
| M1: design freeze | Owner accepts responsibility/origin model, registered completeness boundary, trusted local profile, authority dimensions, identity and lifecycle rules. Decide exact read API and runtime-approval prerequisites before code. |
| M2a: source-owned persistence prototype | One AF-PERF source adapter into isolated SQLite projection; idempotency, rebuild, migration/reopen and drift negatives. No runtime approvals, real user migration or canonical IR changes. |
| M3: deterministic retrieval | A-E query oracle, applicability/closure/conflict/permission checks and explicit blocked budget results with tracked work counts. |
| M4a: read-only MCP integration | One approved explicit read surface; full server->registry->handler->store->projection->response trace, compatibility tests and fresh-process stdio acceptance. |
| M2b/M4b: earned runtime ledger | Only after authenticated owner/approval, consent, retention and correction contracts; demonstrate UC-2/3 without self-promotion. No automatic transcript extraction. |
| M5: optional context integration | Measure complete bundle economics. Automatic source-context attachment requires separate public-contract approval and exact legacy-response controls. |
| M6: broader workflows | Validate research and non-coding policy fixtures first; enterprise hosting only after actual identity/ACL/retention integration. Batch remaining relevant Claude field questions after local deterministic gates. |

For any later implementation, trace creation -> production call path -> returned data -> durable owner -> restart/edit/deletion lifecycle -> consumer -> MCP exposure -> live reachability. A populated index or unit-test-only query is not integration. Local tracked regressions and operator/live evidence remain separate; Claude is the last-resort field gate, not the first experiment.

## 15. Owner decisions still required

**PROPOSED decision points, not enacted architecture:**

1. **Responsibility/storage:** approve hybrid ownership and an isolated PCG projection/ledger lifecycle. Needed because file-baseline replacement cannot retain decisions. Affected: persistence configuration, migration and retention. Alternative: source-only A permanently, sacrificing runtime-only continuity. Recommend C staged through A; more boundaries, less duplicate truth.
2. **Authority principal:** choose the trusted local owner and eventual host/admin approval adapter, imported-approval trust policy, independence rules and revocation semantics. Affected: every promotion/supersession/write API. Alternative: all imported approval remains source-declared advisory. Recommend advisory imports first; runtime authority writes remain disabled until authenticated capability exists.
3. **Completeness/applicability:** approve enrolled-corpus completeness, mandatory constraint selectors and fail-closed required budgets. Affected: new PCG response semantics only. Alternative: discovery-only memory search with no completeness guarantee. Recommend explicit required closure; no changes to existing tool completeness.
4. **Public API:** select a versioned `workspace_query` operation or one separate read tool after schema fit review. Affected: MCP schemas/dispatch/clients. Recommend the smallest explicit read contract; defer automatic injection and all six proposed tool names.
5. **Deployment/security:** approve single-owner local scope versus a funded multi-tenant service contract; choose scrubbing, retention, backup and lawful erasure policies. Affected: evidence storage/history/logs. Recommend local-first; do not advertise enterprise isolation from workspace path checks.
6. **Conflict/time rules:** define single-valued decision subjects, effective intervals and policy-specific authorized precedence. Affected: controlling-head resolution. Recommend conflict-visible blocking unless explicit supersession/effectivity resolves it; never highest confidence/latest timestamp by default.

These decisions have externally observable and cross-cutting effects. This mission permits proposing them, not implementing them. No global limit, cache, authority boundary or API is changed by this report.

## 16. Explicit final recommendation

**PROCEED TO DESIGN FREEZE**, limited to: evidence remains authoritative in its owning source; stable scoped identities and immutable referenced revisions; independent authority/verification/lifecycle/freshness dimensions; deterministic explicit retrieval; enrolled-scope completeness; mandatory-constraint fail-closed budgeting; source/canonical IR ownership preservation; read-only first integration; and A-E executable acceptance obligations.

The hybrid target is recommended, with source-owned projection first. Runtime ledger writes/approval, enterprise identity, exact SQL/API contracts, retention policy and automatic integration remain owner decisions. If the owner requires immediate multi-tenant authoritative writes, **INVESTIGATE FURTHER** on trusted identity/ACL/approval integration before freezing that expanded design. No evidence supports treating SQLite existence or a graph status as sufficient authority.

The investigation is static and design-only. No tests, builds, lints, formatters, benchmarks, validators, migrations, application binaries or servers were launched. Existing reports' execution receipts were read as attributed evidence, not rerun acceptance. No new performance result or production integration is claimed.

For this documentation-only return, the user-owned text gates are:

```powershell
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-file-sizes.ps1
pwsh -NoProfile -ExecutionPolicy Bypass ./scripts/check-utf8.ps1
```

Both remain **UNRUN** by the agent. Bounded byte-level UTF-8/BOM inspection and `git diff --check` are appropriate static inspection; their actual result is recorded separately at delivery. A future implementation must run the authoritative Final Verification Gate in `docs/agent/verification.md`; no speculative PCG test command is presented as existing coverage.

## Source/evidence appendix

All GLS links below resolve against [the pinned GLS snapshot](https://github.com/realworldbooks/GoalLine-Stand-2026/tree/f48228537ac7c44b7c47a71b5cf19c099130e468); all Clean-CTX links resolve against [the selected performance snapshot](https://github.com/codeliftsleep2/Clean-CTX/tree/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3). Local acquisition copies are ignored inspection receipts, not a new authoritative repository or reusable test fixture.

| Ref | Repository path and inspected authority |
| --- | --- |
| G1 | GLS [`docs/ARCHITECTURE_EVIDENCE_GRAPH.md`](https://github.com/realworldbooks/GoalLine-Stand-2026/blob/f48228537ac7c44b7c47a71b5cf19c099130e468/docs/ARCHITECTURE_EVIDENCE_GRAPH.md): governing distinction, seven kinds, twelve statuses, thirteen directional relations and agent duties. |
| G2 | GLS [`scripts/architecture_graph.py`](https://github.com/realworldbooks/GoalLine-Stand-2026/blob/f48228537ac7c44b7c47a71b5cf19c099130e468/scripts/architecture_graph.py): `load_graph`, `validate`, `Graph`, render/check functions, exact-tree and outgoing-push admission. Blob `732474a72e93eca07a862996bf0321f361c83f7a`. |
| G3 | GLS `docs/architecture/ARCHITECTURE_EVIDENCE_GRAPH.{yaml,md}`: generated snapshot; YAML blob `30529e576908902699042dde6aea7b562843a93a`, Markdown blob `ab86c6cc890399550b378f52c4fbfdeaf21c8647`. Count not independently revalidated. |
| G4 | GLS `docs/investigations/Football_DevelopmentalHumanMorphology_RuntimeSynthesisContract_677P.md`, final declaration and GRAPH-002 reconciliation. |
| G5 | GLS `docs/investigations/Football_DevelopmentalHumanMorphology_RuntimeMorphologySynthesis_IndependentReclosure_677PR.md`, final answers and `RECLOSES` declaration. |
| G6 | GLS `docs/investigations/Football_DevelopmentalHumanMorphology_CausalFidelityBoundary_FootballRelevanceReassessment_676P.md`, §§15-17 and declaration; failure preserved, implementation not authorized. |
| G7 | GLS `docs/research/Football_DevelopmentalHumanMorphology_JPPS_NonGaussianJointPrior_CalibrationFalsification_675P.md`, historical failure/supersession declaration. |
| G8 | GLS `docs/investigations/Football_Field_SameSegmentBlockOriginProvenance_IndependentReclosure_667FR.md`, explicit reclosure/banking declaration. |
| G9 | GLS [`scripts/mission_control_intelligence.py`](https://github.com/realworldbooks/GoalLine-Stand-2026/blob/f48228537ac7c44b7c47a71b5cf19c099130e468/scripts/mission_control_intelligence.py): `report_context`, `node_fact`, `prerequisite_paths`, `blocker_paths`, `analyze`, `render_text`. Blob `3b012a205a270f395a34ab861d81f9fae9c31c63`. |
| G10 | GLS `scripts/tests/test_architecture_graph.py` and `test_architecture_graph_intelligence.py`: parsing, identity, blockers, history, drift, no-promotion, coverage and cycle negative tests; source inspection only. |
| G11 | GLS `docs/integrations/Architecture_Evidence_Graph_DebtRetirement_Implementation_GRAPH-002-I.md`; historical verdict versus current declaration/debt. `docs/experiments/Repository_ArchitectureEvidenceGraph_CurrentRepositoryConvergence_GRAPH-001R-X.md` is a separately attributed historical convergence report. |
| G12 | GLS `.githooks/pre-commit`, `.githooks/pre-push`, `.github/workflows/architecture-graph.yml`; `scripts/tests/test_architecture_graph_admission.py` and `test_architecture_graph_ci_topology.py`. |
| G13 | GLS `scripts/agent_commit.py`: `admit`, `verify_head`, `publish`; canonical admission composition. |
| G14 | GLS `scripts/publication_owner.py`: `PublicationOwner.transition`, generation/transaction checks and atomic publication fence. No publication invoked. |
| G15 | GLS `AGENTS.md`, `docs/DOCUMENT_GRAPH_PUBLICATION_CONTRACT.md`, `docs/investigations/Repository_ArchitectureEvidenceGraph_GovernanceDoctrine.md`; repository-specific authority and publication boundaries. |
| C1 | Clean-CTX [`src/mcp/sqlite_store.rs`](https://github.com/codeliftsleep2/Clean-CTX/blob/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3/src/mcp/sqlite_store.rs): `open`, `migrate`, `save_context`, `list_contexts`, purge/replacement lifecycle; `sqlite_store/compatibility_schema.rs` v6. |
| C2 | Clean-CTX `src/mcp/state.rs::McpState::new` and lock ordering; `src/mcp/buffered_store.rs` explicit lifecycle-owned buffering and failed-pending behavior. |
| C3 | Clean-CTX `src/mcp/sqlite_store/semantic_state.rs::save_context_transaction`, `append_delta_with_semantics`, `load_durable_context`; `sqlite_store/edit_intent.rs` recovery. |
| C4 | Clean-CTX `src/mcp/sqlite_store/replay.rs::load_context_with_deltas`; `src/mcp/context_store.rs`; baseline ownership and retained delta history. |
| C5 | Clean-CTX `src/ir/pipeline.rs::PassPipeline::default_production`; `src/mcp/persistence_ir.rs`; source structural/durable ownership. |
| C6 | Clean-CTX `src/ir/inference_layer.rs`, `src/layers/meta/semantic.rs`; structural versus inferred edges and source/entity identity. |
| C7 | Clean-CTX `src/mcp/durable_semantics.rs`, `src/mcp/compatibility/validator.rs`; exact retained semantic/source compatibility and currentness boundaries. |
| C8 | Clean-CTX `src/mcp/tool_handlers/core/provide_persistence.rs::persist_read_baseline_with_authority`. |
| C9 | Clean-CTX `src/mcp/tool_handlers/persistence/replay.rs::handle_replay_history`; checked load, economics, later render, publication and response. |
| C10 | Clean-CTX `src/mcp/tool_handlers/context/mod.rs::handle_context_history`; metadata inspection without semantic ownership. |
| C11 | Clean-CTX `src/mcp/state/source_cache.rs`; `src/mcp/tool_handlers/core/delta.rs`, guarded second read and delayed capture. |
| C12 | Clean-CTX `src/mcp/state/publication.rs::SemanticPublicationTicket::commit`; per-owner generation/currentness serialization. |
| C13 | Clean-CTX `src/workspace/{index.rs,scope.rs,path_identity.rs}`; provenance-scoped source relationships and component-wise root identity. |
| C14 | Clean-CTX `src/mcp/{server.rs,dispatcher.rs,router.rs,tools.rs}`, `src/mcp/tool_handlers/registry.rs::create_default_registry`. |
| C15 | Clean-CTX `src/mcp/tool_handlers/core/content.rs`, `src/mcp/content_economics.rs`; complete code selection and local tokenizer economics. |
| C16 | Clean-CTX `proxy/src/config.rs`, `proxy/src/transform.rs::scrub_secrets`; optional proxy transform, default disabled. |
| C17 | Clean-CTX [`docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md`](https://github.com/codeliftsleep2/Clean-CTX/blob/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3/docs/agent/AF_PERF_002_DELTA_PREVIOUS_REPAIR.md), final CLOSED/FIX/PROTECTED section; `AF_PERF_002_DELTA_PREVIOUS_DESIGN.md`. Reported receipt `af-perf-001-20261109-161615`; not rerun. |
| C18 | Clean-CTX `docs/agent/AF_PERF_002_REPLAY_RENDER_{DISCOVERY,DESIGN}.md`; registered-deletion counterexample and REJECT recommendation. |
| C19 | Clean-CTX [`src/tests/mcp/replay_footer_observation_af_perf_002.rs`](https://github.com/codeliftsleep2/Clean-CTX/blob/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3/src/tests/mcp/replay_footer_observation_af_perf_002.rs), `af002_replay_footer_observes_dictionary_after_economics`; active registration in replay handler. |
| C20 | Clean-CTX [`docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md`](https://github.com/codeliftsleep2/Clean-CTX/blob/8ce0a6865ff8a2b55866895c6a239f6bcc3245a3/docs/agent/AF_PERF_002_GITDIFF_DISCOVERY.md): cleanup, failed first packet, final reported discovery and two DEFER decisions; receipt `gitdiff-discovery-20261109-190157`, not rerun. |

Inspection limitations: no full GLS validator/fitness execution, no new original test receipt verification, no live MCP/Claude run, no production DB reads/migrations, no enterprise identity/ACL integration test, no private enterprise corpus, no whole-repository absence proof for every possible memory feature, no live GitHub protection/CI verification. Separate feature branches and later commits are outside this pinned design basis. Source-level tests show intended assertions, not their current execution result. A future design freeze must record any changed source baseline before implementation.

```architecture-graph
node: PCG-175-DISCOVERY
kind: RESEARCH
status: INVESTIGATING
title: PCG architecture investigation and bounded read-only proposal
track: REPOSITORY
artifact: docs/agent/PCG_175_ARCHITECTURE_INVESTIGATION.md
note: Source record and its attributed disposition only; no new execution, authenticated approval, independent certification, reclosure or banking.
```
