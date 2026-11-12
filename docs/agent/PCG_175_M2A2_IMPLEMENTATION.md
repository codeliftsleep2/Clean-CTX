# PCG-175-M2A2: deterministic evidence acquisition

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: owner's explicit M2a-2 authorization after M2a-1 closure.
Normative basis: [approved design freeze](PCG_175_DESIGN_FREEZE.md), especially
ME-01, INTEGRITY-01/02 and §§7/10.4. M3/M4, automatic injection, approval writes,
enrollment/persistence redesign, staging and publication remain unauthorized.

Disposition: **M2A2 ACCEPTANCE READY** following the explicitly authorized
M2A2-VERIFY mission. All 12 focused cases passed; changed-file formatting and
library-scoped Clippy passed. No functional defect or lint correction was needed.
Only formatting changed during verification. The accepted M2a-1 campaign remains
closed. Owner acceptance is the next gate; M3/M4 are not authorized.

**Subsequent owner disposition (2026-11-11):** M2a-2 accepted; its verification
and the M2a-1 campaign remain closed. The owner separately authorized M3 with
"proceed to m3". The original stage disposition above is historical; current M3
work and its unrun verification are recorded in
[PCG_175_M3_IMPLEMENTATION.md](PCG_175_M3_IMPLEMENTATION.md). M4 remains unauthorized.

## 1. Scope and call path

The new explicit Rust library entry is `pcg::query::acquire(profile_path, request)`:

```text
trusted owner profile + typed request
  -> existing store::reader (read-only SQLite transaction)
  -> exact generation/header and scope checks
  -> indexed selector/binding acquisition + three-valued applicability
  -> required roots / dependencies / gates / incoming support and refutation
  -> provenance, conflict, supersession and reverse-impact proof paths
  -> exact mandatory evidence and all corpus-current source witnesses
  -> required acquisition bundle
  -> final source/profile observations
  -> Ready / Incomplete / Blocked / Unavailable acquisition
```

This is a library acquisition capability, not a registered MCP tool. Its typed
`Ready` means the requested evidence acquisition succeeded within the named
registered scope and trusted-storage assumptions. `context_complete` is always
false: exact token accounting, final wire projection and MCP completeness are
M3/M4 work. Existing tools/responses and startup behavior are unchanged.

M3 must compose its packing before the COMPLETION witness boundary; simply
packing an already returned acquisition cannot claim that packing was inside
the two observations. No speculative callback/adapter framework is added now.

## 2. Changed files and ownership

| File | Responsibility |
| --- | --- |
| `src/pcg/query.rs` | Typed request, limits and acquisition coverage/state; trusted scope checks; orchestration and failure precedence; clears actionable bundle on incomplete/blocked/unavailable results. |
| `src/pcg/selectors.rs` | Exact conjunction/union including missing dimensions, implied entity file and half-open UTC intervals. TRUE dominates UNKNOWN for the same record; known mismatch dominates unknown conjuncts. |
| `src/pcg/query_rows.rs` | Parameterized p/c/g reads, indexed candidate/relationship acquisition, bounded row retrieval and exact accessed-row checks. No writer or full-generation hash scan. |
| `src/pcg/query_rows/objects.rs` | Lazy record/source/evidence metadata and exact revision reconstruction. Optional ref metadata participates in record-hash validation; optional source bodies are not fetched. |
| `src/pcg/closure.rs` | Mandatory least-fixed-point traversal; true premise/gate checks; incoming VALIDATES/FALSIFIES; conflict/supersession/impact paths and cycle detection. |
| `src/pcg/query_witnesses.rs` | One selected immutable witness set, START and COMPLETION working-file observations and retained historical permission/availability checks. |
| `src/pcg/provenance.rs` | Exact citation bytes and hashes, source-attributed record metadata and deterministic typed paths. No prose summaries or authority promotion. |
| `src/pcg/mod.rs` | Narrow module export/registration and focused test registration. |
| `src/pcg/git.rs` | Only widens the existing local Git helper to `pub(super)` for read-only historical availability probing. Its command body, source verification, import and persistence behavior are unchanged. |
| `src/tests/pcg/query.rs`, `query_pilot.rs` | Twelve focused cases plus pinned AF-PERF fixture construction. Existing M2a-1 tests/support files are untouched. |
| This document | Implementation evidence, scope and canonical question matrix. |

The existing SQLite schema, import/rebuild/admission algorithms, profile format
and source identities are consumed unchanged. No edits to `store.rs`, enrollment,
identity hashing, main/lib wiring, MCP dispatch/registry/schema, canonical IR,
replay, delta, source cache, publication or tokenizer are required by this stage.
No Cargo dependency, migration, default limit or retry policy was added.

The starting HEAD/tree are the accepted immutable baseline:
`8ce0a6865ff8a2b55866895c6a239f6bcc3245a3` /
`10d8ecadd0a151ea0638e2a066bd50b40ad022e0`, branch
`(fix)Performance-Regression-Issues`. The implementation remains uncommitted.
CBM reported available; code-context acquisition returned mostly skeletons, so
exact known source ranges were read natively for bodies. No branch/ref was changed.

## 3. Semantics and completeness

- Active applicable required decisions/constraints become roots. Explicit IDs
  add required context; they do not create implicit applicability bindings.
- DEPENDS_ON and BLOCKED_BY pull exact target revisions and evidence. Gates
  retain exact outcome/subject/authority requirements. Failed or inactive true
  premises cannot be satisfied by descriptive evidence or a confidence score.
- Required primary witnesses, explicit required refs, all required relationship
  provenance and incoming support/refutation are mandatory. USES_EVIDENCE,
  PRESERVES and IMPLEMENTS never enlarge mandatory closure.
- Historical explanation is distinct from premise truth. Mandatory history
  cannot be removed by `include_history=false`, but it still requires profile
  permission. Direct historical roots require explicit history selection.
- A retracted historical refutation remains explanation; active unresolved
  refutation/support of controlling authority blocks the acquisition.
- Conflict peer acquisition is reused per subject/role within one operation.
  Disjoint, definitely nonmatching peers are not manufactured into UNKNOWN
  overlap. No rank, confidence or timestamp arbitrates a conflict.
- Exact empty answers require definite coverage, completed enrollment,
  applicability and required acquisition. Missing scope dimensions, unenrolled
  roots and limits return incomplete; they never establish absence.
- Every corpus-current witness is still checked, including uncited source
  files. Required immutable evidence is checked once per revision, independently
  of two current-file observations. All text remains untrusted source data.
- Known projection corruption is unavailable; observed required source/access
  failure blocks; incomplete work remains incomplete unless a known blocker
  already has precedence. No partial actionable required bundle is returned.

Only exact declaration-shaped entity input is accepted. No source-owned canonical
declaration proof is supplied by this standalone library entry yet; entity
requests therefore return `Incomplete/identity_unavailable`. It does not compile,
hydrate or invent an endpoint. File/operation/workspace requests can acquire
complete evidence across fresh processes independently of source indexing.
No claim of a working entity-index/MCP adapter is made.

Optional support/history packing is not introduced. Optional metadata needed to
verify a selected record's immutable hash is not optional body retrieval; its IDs
are removed from emitted required refs. Work counts are suppressed on non-ready
or history-restricted results (`counts_disclosed=false`), rather than exposing
hidden history counts or describing redacted zeros as measured zero work.

INTEGRITY-02 and FRESHNESS-01/02 limits remain explicit: accessed-row verification
does not certify intact off-path membership, and two observations are neither
continuous currentness nor an atomic multi-file snapshot. All acquisition results
retain those assurance limits. No global cache, lock, file snapshot or retry was
introduced to conceal them.

## 4. Focused test definition inventory

The 12 new registered cases are:

| Case | Contract / original negative controls |
| --- | --- |
| `applicability_three_valued` | Exact TRUE/FALSE/UNKNOWN, known mismatch, union and exclusive interval end. |
| `covered_empty_and_unknown_scope` | Covered empty versus unenrolled; disjoint opposite policy; TRUE plus UNKNOWN bindings for one record; missing task; enrollment incomplete. |
| `scope_generation_identity_and_record_refusal` | Wrong root, generation, missing ID and unavailable canonical entity; no source acquisition fallback. |
| `mandatory_refs_and_incoming_counterevidence` | Required refs without descriptive promotion, retained historical refutation, shared citation and unresolved active support/refutation. |
| `descriptive_evidence_never_promotes` | Even a required=true evidence target of USES_EVIDENCE remains descriptive; source approval/execution attribution stays distinct. |
| `gates_and_historical_truth` | Exact gate subject/outcome/authority, failed lifecycle premise, historical explanation and revoked history permission. |
| `reverse_impact_and_exact_provenance` | Incoming dependency closure and outgoing proof paths; exact cited subject/hash. |
| `drift_corruption_and_no_read_side_effects` | Read-only main DB bytes, persistent same-size source drift, controlled mutation between witness boundaries and accessed row corruption. |
| `shared_witness_fanout_and_bounded_work` | N/2N/4N shared-evidence fanout, one validation/citation, explicit source-byte work, unapproved sidecar edits not changing admission and bounded incomplete results. |
| `query_fixture_worker` | Test-only subprocess entry; performs actual acquisition assertions when selected by parent. Normal standalone entry has no child mode. |
| `fresh_process_file_acquisition` | Fresh test process reads existing admitted generation through the same library API without source indexing or import. |
| `af_perf_frozen_dispositions` | A/B protected delta and deferred second read; C rejection/refutation and required regression source; separate D deferrals; E failed/completed packet subjects remain distinct. |

The test count and expected scope were reported before registration. Small local
Git/SQLite cases ran in the focused verification below; the AF-PERF case checks multiple
queries over a pinned corpus and explicitly pays all-current-witness costs.
There is no new CI job, ignored required test or separate regression framework.
All 12 registered cases passed in the authorized focused run below.

The AF-PERF case reads exact objects at the accepted commit into a temporary
workspace and asserts the frozen span hashes before import. A local Git alternate
provides those immutable objects; no fetch/network fallback exists. Missing
baseline objects are fixture acquisition failure, not semantic RED. Existing CI
checkout explicitly uses `fetch-depth: 0`; this is not a workflow modification.
The manifest/profile are test-owned and do not enroll or approve a production
corpus. Fixture scopes match the separate file/operation/task bindings; no global
AF-PERF completeness or measured enterprise benefit is invented.

## 5. Canonical question matrix and consumer audit

| Question | Required authority/acquisition | Entry / answer / adversary |
| --- | --- | --- |
| Which exact constraints apply here? | Trusted profile, admitted generation, complete candidate universe and three-valued bindings | `query::acquire(Constraints)`; required records and exact citations; wrong root/name/generation and unknown dimensions refuse or remain incomplete. |
| What premises and gates are necessary? | ME-01 typed required closure and exact gate subject/authority | `Closure`; deterministic proof edges; USES_EVIDENCE cannot satisfy a gate or pull optional targets. |
| What depends on this exact record? | Generation-scoped incoming dependency/blocker index and recursive consumers | `Impact`; exact paths plus applicable roots; incomplete acquisition cannot certify no consumers. |
| Why is a decision protected/rejected? | Immutable primary/required refs, incoming support and counterevidence | `Provenance`; attributed outcome and source spans; detector success is not equivalence and report-of-execution is not a new receipt. |
| Has the evidence changed? | All corpus-current files plus required historical Git/span identities at stated boundaries | Observed stale/missing/access failure blocks; unobserved ABA and serialization gaps remain outside guarantees. |
| Are there no applicable constraints? | Registered coverage and complete enrollment/applicability/required acquisition | Empty Ready only in that scope; absence of canonical proof or work limit cannot establish a complete negative. |
| Can the LLM consume a token-complete public response? | M3 packing and M4 registry/stdio integration | **Not exposed by this stage.** Rust acquisition is reachable; no MCP completeness, token fit or live client integration is claimed. |

The real owner is the existing derived SQLite generation, pinned in its existing
read transaction. Nothing restores or publishes source-semantic state. Repeated
queries start new operation-local readers; cached objects never survive a query.
Restart preserves file-level acquisition because its proof comes from the admitted
projection and local immutable sources, not an ephemeral code index. Typed entity
availability remains explicitly incomplete until an actual source-owned adapter
is integrated under the approved boundary.

## 6. AF-PERF accounting and verification boundary

Operation scope is one acquisition over one p/c/g snapshot. Complete-set work is
candidate enumeration, required proof closure, accessed-row integrity, required
evidence verification and both corpus-current observations. Immutable record/ref
sets use shared operation-local `Rc` ownership; full sets are not cloned/rehashed
per incoming edge. Source and evidence metadata are validated lazily once, and
each required evidence span is materialized once. Distinct source bytes still
dominate some workloads; this stage does not advertise a subgraph-only byte cost.

Expected cost is indexed candidate/proof/ref work plus accessed metadata bytes,
all selected immutable/current-source bytes and deterministic ordering. A selected
record's hash includes all its ref metadata, so those exact optional ref revisions
must be examined once even though optional bodies remain unacquired. This necessary
integrity work is counted rather than hidden behind a required-ref-only claim.
Conflict groups are acquired once per examined subject/role. No unconditional
full-generation row/byte hashing or repeated complete authority-set reconstruction
is placed in ordinary open/query.

`Counts` records visited records, selectors, bindings, relations/refs, metadata
bytes, evidence validations and accounted witness bytes when disclosure is
permitted. No latency, storage saving, allocator or enterprise-efficiency result
is claimed. No tokenizer or packing work is counted as if already implemented.

Verification authority remains owner-controlled and change-scoped. Only the
explicitly authorized focused test filter, changed-file formatting and production
library Clippy were executed. No broad suite, all-target lint, operator campaign,
benchmark or application/server execution was added.

## 7. Focused verification and acceptance evidence

Actual commands and outcomes:

| Verification | Result | Retained evidence under `target/pcg-175/m2a2-verify/` |
| --- | --- | --- |
| `cargo test --package clean-ctx --lib --all-features pcg::query_tests::` | 12 passed, 0 failed, 0 ignored; 5,131 filtered out. Compilation 5m 53s; test execution 56.58s. The nested fresh-process worker separately passed 1 test in 3.13s. | `initial-tests.log` |
| `rustfmt --edition 2024 --check --config skip_children=true <11 stage files>` | Initial check found formatting differences only. Formatted exactly the 10 paths in `formatted-files.txt`, then the same 11-file check passed. Existing `git.rs` needed no formatting. | `format-check.log`, `formatted-files.txt`, `final-format-check.log` |
| `cargo clippy --package clean-ctx --lib --all-features -- -D warnings` | Passed with no warnings; finished in 1m 32s. | `library-clippy.log` |

The 11-file formatting scope was `src/pcg/{mod,git,query,selectors,query_rows,closure,query_witnesses,provenance}.rs`,
`src/pcg/query_rows/objects.rs` and `src/tests/pcg/{query,query_pilot}.rs`.
`skip_children=true` prevented recursive formatting of accepted M2a-1 modules.
No functional RED occurred, no negative control was weakened and no production
logic was corrected during verification. Tests ran before formatting; final
library Clippy ran after formatting. No test rerun was added solely for whitespace.

Initial tested-source fingerprint:
`34b71fd3fed8d1119852662152cd767e6d1ce3b96f5ce8242fe8468a9aa49200`.
Final formatted/linted-source fingerprint:
`309ba06bab6d87b0aa35884be567b18fc77a03d7c3efb23e533ca672949a16f2`.
Both inventories are retained as `initial-source.sha256` and `final-source.sha256`.
Their scope is `src/lib.rs`, `src/main.rs` and all files recursively beneath
`src/pcg/` and `src/tests/pcg/`. Definition: ordinal-path-sorted lowercase SHA-256,
two spaces, slash path and LF per entry, final LF included; SHA-256 over those
UTF-8 bytes. The difference is the explicitly bounded formatting above.
HEAD/tree/branch remain those recorded in §2; no source revision switch occurred.

Log SHA-256 identities:

- `initial-tests.log`: `2739d9252eec89bb3cad16798432f7b33581abf266b2d84b09df97aa72f328e7`.
- `format-check.log`: `754d928273bf72b0270dc8553aa62433df7844807e8bdfa2b8a472b1bb3b2dd1`.
- `final-format-check.log`: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` (empty successful check output).
- `library-clippy.log`: `6a9b467fe830963f17f7bc2b745c42707d188f1ac4d950b22acbeb5e5f2ff809`.

Executed shared-witness assertions cover existing fixture populations 4/8/16:
required record counts 5/9/17 respectively, one shared citation and one evidence
validation in each query, and 804 accounted witness bytes for the 201-byte source
(immutable source plus the prescribed working observations/accounting). These
are asserted deterministic counts, not separately printed benchmark samples.
Other candidate/relationship counter values were not retained as raw output and
are not reconstructed. No additional population or scaling campaign ran.
The frozen AF-PERF dispositions and original negative controls passed.

Repository-wide tests, workspace/all-target Clippy, CI and live MCP/Claude gates
remain **UNRUN** in this mission; that is neither failure nor a claimed PASS.
No enterprise latency, allocation, storage benefit or integrated MCP capability
is established. Entity proof remains explicitly unavailable; `Ready` is an
acquisition result, never a token-complete public context result. The integrity
and boundary-only currentness limits in §3 remain unchanged.

The accepted enrollment/persistence design and previous operator campaign stay
closed. No production corpus/profile is enrolled, no runtime approval is written,
no M3 token projector or M4 tool is registered, and no stage advances automatically.
### Static delivery inspection

Strict UTF-8/BOM, replacement-character, trailing-whitespace and current line-size
inspection passed for all new/modified stage files. All 12 `#[test]` definitions
are registered under the existing test-tree convention. `git diff --check` reports
no whitespace errors. Existing accepted source hashes match the acceptance
inventory except the explicitly described module wiring and Git-helper visibility
changes; protected source subsystems and M2a-1 persistence files remain unchanged.

Independent bounded Git-object byte inspection confirmed the five full-source
hashes and all nine exact UTF-8 spans used by the AF-PERF pilot against the frozen
catalogue at `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`. The working delta report
differs bytewise from that pinned witness; it was not substituted into the fixture
or used to rewrite its hashes. The fixture explicitly reads raw pinned blobs.
This is static witness inspection, not a Rust test or an executed PCG query/import.

The static inspection above preceded the focused execution recorded in §7.
Those scoped results support M2a-2 owner acceptance; no operator or broad gate
result is inferred and the completed M2a-1 campaign is not reopened.

```architecture-graph
node: PCG-175-M2A2
kind: IMPLEMENTATION
status: IMPLEMENTED
title: Deterministic evidence acquisition implementation
track: REPOSITORY
artifact: docs/agent/PCG_175_M2A2_IMPLEMENTATION.md
implements:
  - AUTH.PCG_ACQUISITION_V1
note: Historical implementation and scoped reported verification; B3 does not rerun or independently certify these witnesses.
knowledge_scope: evidence
knowledge_content:
  - statement self#whole
```
