# PCG-175 Corpus Expansion 02: canonical IR contracts

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11. Authority: Phase A, bounded inspection and proposal only.
Disposition: **READY FOR OWNER CORPUS/PROFILE REVIEW — NOT ENROLLED**.
PCG v1 and Pilot 01 remain closed and unchanged. No product change is proposed.

## 1. Generation strategy: complete union, never implicit merge

**OBSERVED:** `src/pcg/import.rs::prepare` normalizes the supplied manifest;
`finish` inserts that complete generation and atomically publishes its head.
`src/pcg/store.rs::publish` replaces the `(partition,corpus)` active pointer.
`preserve_decisions` protects controlling metadata only for IDs present in both
generations. It does not restore or reject omitted records. Importing an IR-only
manifest would therefore replace active coverage with that manifest, not extend
the prior graph. Query access is generation-scoped; retained older generations
and `includeHistory` do not provide a cross-generation union.

**PROPOSED:** import one complete union manifest containing every Pilot 01
definition plus the reviewed additions. Use a separate Corpus 02 workspace,
profile and database. Do not rebuild, replace or repoint the original Pilot 01
database/profile. This preserves the original runnable pilot and its evidence.
The implementation supports this workflow without an adapter or code change.
Publishing a union into the original database would also require the complete
union manifest and a reviewed profile digest change; that is not this proposal.

Stable partition/corpus/workspace IDs and policy labels are retained, including
their historical `pilot01` names. They are local scoped identifiers, not CBM
slugs or fabricated canonical entity identities. The new physical root and
database location do not participate in record revision hashes. All original
source, evidence, record, selector and edge definitions/revisions are identical.
The manifest digest and generation necessarily change.

## 2. Immutable basis and preservation

Commit: `6add8449c1a036d0599283fd6a982009d495e954`.
Tree: `d1540017265328e4bbb0b4ea31d6a8bc7172cb1a`.
Branch: `(fix)Performance-Regression-Issues`. No remote-tip assertion is made.

Read-only SQLite inspection confirmed Pilot 01 active generation
`9cd113849a41cc4edc60304105b1fffa2fdf55df67365cd25c97f855ea16ee69`,
with seven records, 18 evidence rows, eight relations and seven bindings.
The original approved manifest/profile digests still match their reviewed values.
The preservation receipt records hashes of existing review, execution and storage
files before/after preparation. No PCG audit/import or application was executed.

All proposed sources use exact Git blob bytes at the same commit, `check=current`.
Some original checkout files have CRLF differences from the enrolled Git bytes.
The future isolated workspace must reproduce the Git bytes exactly; current
original-checkout bytes must not be substituted or silently normalized. This is
a pinned-snapshot architectural corpus, not proof about future/live source tips.

## 3. Scope, source selection and authority

Pilot 01's seven CBM records remain bound to their original question unchanged.
The new reviewed scope is workspace `cleanctx-cbm-cache-pilot01`, operation
`canonical_ir_lifecycle`, task `explain_preservation`, explicit time
`2026-11-11T12:00:00Z`. These are exact noncoding question labels, not new MCP
operations. No file/entity selector or canonical entity adapter is needed.

The two new selectors declare coverage/applicability for this question only.
`enrollment.complete=true` proposes completeness of this reviewed registered
contract set. It does not certify all repository prose, every implementation
path, all possible IR questions, or correctness of the software. All ten IR
constraints and existing `cbm-semantic-authority` are mandatory roots in this
scope. Old CBM queries still select their original roots; the new descriptive
outgoing link does not silently enlarge their mandatory closure.

All six candidate documents contribute bounded spans. Only two implementation
files are additionally enrolled: `src/ir/binary_wire/decode.rs` for explicit
version refusal, and `src/mcp/compatibility/validator.rs` for distinct canonical
and semantic admission checks, exact producer equality and dependency freshness.
Their excerpts are implementation observations, not execution results.
`sqlite_store/edit_intent.rs::recover_edit_intent` was also inspected: byte/hash
reconciliation, target identity/version checks and retained compatibility explain
why completing recovery is distinct from live compatible adoption. Its whole
file is not necessary evidence for the selected normative recovery contract and
is not enrolled. Historical audits and their test receipts are not enrolled.

The certification's release/Phase 9 PASS language is historical, not a current
v0.10.0 gate result. The lockdown approval span is a source-attributed historical
approval assertion, supporting still-normative matrix defaults. It is not an
authenticated approver or new permission. Every new record is an active
`constraint`, `required:true`, authority `verified_claim` **source-attributed**,
verification `supported` for its named contract, execution `not_run`. Here support
means the source states the contract; it does not mean tests were executed.

Selected invariant excerpts include their enforcement/gate descriptions. Those
are attributed source claims only. PERSIST-001's current auto-save distinctions
qualify the older certification's broad durable-before-publication language:
ordinary manual-mode reads remain session-only; accepted mutations/Edit
authority retain their durability requirements. The selected overview span
excludes its adjacent broad `RwLock` description; no global-lock claim is added.
Historical binary 0x03 inventory and earlier decoder-branch commentary are also
excluded; the selected final policy explicitly rejects lossy 0x01–0x03 input.

## 4. Proposed graph

| New required record | Primary evidence; required supporting evidence | Controlling requirement |
| --- | --- | --- |
| `ir-typed-identity` | ir-approved-defaults; ir-identity-namespaces, ir-operation-matrix, ir-approval-history | Typed owner namespaces, global method/field identity and method-scoped parameters; structured refusal of invalid/ambiguous identity. |
| `ir-occurrence-preservation` | ir-cardinality; ir-operation-matrix | Operation-specific singularity, ordered facts/payloads and exact duplicates; no implicit last-write-wins, deduplication or union. |
| `ir-transform-preservation` | ir-validation-transform; ir-semantic-boundaries | Preserve identity or apply complete remapping/authorized removal; no orphaned references; preserve declared order and edit-fidelity bodies/spans. |
| `ir-projection-authority` | ir-projection-separation; ir-semantic-boundaries | Canonical IR and minimized LLM rendering are separate fidelity/authority contracts. |
| `ir-binary-preservation` | ir-binary-equality; ir-binary-legacy, ir-binary-failures, ir-binary-decoder | Physical 0x04 exact semantic round trip or structured failure, including malformed/trailing data; no invented legacy identity. |
| `ir-delta-replay` | ir-replay-invariant; ir-cardinality, ir-semantic-boundaries | dv:2 positional typed occurrence checks and transactional rejection; server-owned target semantic snapshot; binary/delta versions independent. |
| `ir-durable-publication` | ir-checkpoint-invariant; ir-owner-publication | Policy-aware checkpointing, one-owner coherent capture and ordering, aligned durable state before required live publication; reads do not flush others. |
| `ir-compatible-adoption` | ir-compatibility-invariant; ir-compatibility-checks | Decode/source/fidelity/configuration/producer/dependency checks precede respective adoption; canonical and derived semantic validity remain separate. |
| `ir-edit-recovery` | ir-durable-recovery; ir-owner-publication, ir-compatibility-invariant | File-scoped exact-byte recovery/preflight; completing an older transaction never relabels it as currently compatible. |
| `ir-compatibility-quarantine` | ir-quarantine; ir-binary-legacy, ir-compatibility-invariant | Decode-only legacy hierarchy/delta, unsupported lossy binary and inspection-only fallback never manufacture authoritative semantics. |

All required evidence is primary/explicit `evidence_refs.required=true`; no
descriptive link is being used as a hidden mandatory-evidence rule. Shared matrix
and compatibility excerpts retain one evidence identity for query deduplication.

Five prerequisites are proposed: transform requires typed identity and declared
occurrence preservation; exact delta replay requires occurrence preservation;
edit recovery's authoritative adoption requires compatible adoption; compatible
adoption requires structurally valid semantic binary. These encode explicit
necessary conditions in the cited contracts, not historical implementation order.
Each edge has exact provenance. No timing, causal or approval relationship is
inferred merely because two documents mention each other.

The sole new cross-topic relation is
`ir-projection-authority PRESERVES cbm-semantic-authority`, citing the existing
overview authority span. It records respect for CBM discovery versus
WorkspaceIndex semantic authority, without making CBM cache state a prerequisite
for canonical IR. The old semantic-authority record is separately bound as a
required root for the new question. No claim says a fresh CBM cache proves
compatible persisted IR, or that an IR restore refreshes CBM project caches.

## 5. Review artifacts and identities

Artifacts are local ignored review material under `target/pcg-175/corpus02/review/`.
They are proposals, not a new ingestion mechanism, enrolled owner truth, or test
fixture. The one-off preparation script reads immutable Git bytes and applies
published v1 hashes; it never invokes the importer or accepted binary.

- `manifest.proposed.json`: complete old-plus-new manifest, not an IR delta.
- `profile.proposed.json`: proposed isolated root/database and exact digest pin.
- `source-catalog.json`: commit/blob/SHA-256, byte counts and checkout differences.
- `new-evidence-spans.json`, `excerpts/`: exact half-open byte spans and hashes.
- `proposed-revisions.json`: all revisions and static predicted generation.
- `queries.proposed.json`: four explicit constraints/closure/provenance/impact calls.
- `pilot01-preservation.json`: prior identities and unchanged artifact hashes.
- `proposal-summary.json`: digest/count summary.

Proposed manifest SHA-256:
`58f334bedb47ad26e2decbcbc23a7940f0c10bca758e709bde8766300a73afe5`.

Proposed profile SHA-256:
`cc5501390d5443e20eb38369b20cdd9754f61df2f7d8a14efc9b6ddbe1ca3896`.

**STATIC PREDICTED, NOT IMPORTED** generation:
`e304524614d3df0d6512efbaea0a30f31b94b1eaec54c7e6120c26be6f5f83f9`.

Union: 13 sources, 37 evidence revisions, 17 records, four selectors and 32 edges
(18 bindings, 13 prerequisites, one descriptive preservation). Original definitions
and all original constituent revisions compare equal. New evidence is 34,124
bytes; union excerpts total 65,172 bytes; complete source witnesses total 435,106
bytes. These are static byte counts, not measured retrieval/token savings.

The profile keeps limits 512 records, 1,024 edges, 4,194,304 evidence bytes and
32,768 tokens, history disabled. They are finite pilot limits, not product
defaults. Import and every query must still debit actual work, including all
current witnesses at both boundaries. Token fit has **not** been measured;
escaping, metadata and proof paths count too. If required material does not fit,
the later authorized run must retain its blocked response, not remove mandatory
clauses, silently raise the budget or weaken enrollment. The new workspace and
database have not been created. Absolute profile paths require fresh review if
moved; no tool request can select a replacement trusted profile.

## 6. Subsequent evaluation oracle and review gate

After owner approval only: create an isolated exact-blob workspace at the pinned
commit, explicitly import this complete manifest using the existing binary,
retain actual generation and compare with the prediction. Stop on mismatch.
Use a fresh profile-pinned MCP session and the four proposed requests. No current
session's CBM project identity substitutes for the PCG workspace ID.

Independently assess the engineering answer against these source obligations:

1. Typed identities/owners and operation cardinality survive transforms; lawful
   remapping is complete and dangling/ambiguous facts fail structurally.
2. Canonical order, repetitions, metadata, full bodies/spans and semantic operands
   survive semantic transport; minimized presentation is not durable authority.
3. 0x04 binary and dv:2 replay have separate exactness/version contracts; tuple,
   occurrence and target semantic snapshot requirements cannot be omitted.
4. Checkpoint policy differs from accepted-mutation durability; one owner captures
   aligned IR/source/fidelity/edges/identities before required durable publication.
5. Restored/replayed/recovered bytes must independently pass relevant compatibility
   and source/dependency checks before live adoption; recovery is not approval.
6. Legacy missing identities, lossy binary, ambiguous legacy deltas and incomplete
   fallbacks cannot be promoted. Inspection or structural decoding proves neither
   current compatibility nor semantic authority. Compatible reuse stays supported.
7. CBM discovery/cache freshness and WorkspaceIndex semantic authority remain
   distinct; no invented cache-to-IR prerequisite or authenticated approval appears.

Expect the new constraints mode to select ten IR roots plus the old semantic
authority root and all their required witnesses. Closure/provenance must preserve
the stated prerequisites and exact provenance. Impact from
`ir-occurrence-preservation` should expose transform and replay consumers while
retaining the applicable roots. Old CBM question selectors remain unchanged.
Compare actual IDs, proof paths, citation bytes, completeness, omissions and whole
MCP-result tokens, then assess whether the answer can be recovered without opening
source files. A successful call alone is not a usefulness verdict.

**Owner review requested:** the exact complete-union manifest, its profile digest,
the bounded question coverage, prerequisite/preservation mapping and isolated
generation strategy. Enrollment and runtime evaluation remain withheld until
approval. No tests, builds, checks, Clippy, PCG binaries, servers, operator campaigns,
benchmarks, migrations, commits, pushes or publication were performed. Bounded
static preparation, exact Git/blob/hash inspection and read-only pilot metadata
inspection are the evidence categories for this mission.

Static preparation checks passed for reference resolution, prerequisite acyclicity,
exact excerpt hashes/UTF-8 boundaries and the frozen finite deny signatures over
decoded proposed metadata and excerpts. All 72 inspected Pilot 01 review/execution/
storage files retained their hashes. The report is strict UTF-8 without BOM or
trailing whitespace; `git diff --check` passed for tracked differences. These
checks do not execute PCG admission or certify arbitrary text secret-free.
`static-inspection.json` retains their category and results. The only new visible
worktree path is this report; the two Pilot 01 untracked reports remain untouched.

## Appendix: exact new evidence locators

The following table is generated from the proposed byte-span catalogue. Line
numbers orient reviewers; byte offsets and hashes are identity. All paths resolve
at the pinned commit; the catalogue additionally records each full path.

| Evidence ID | Source ID | Byte range | Lines | Quote SHA-256 |
| --- | --- | --- | --- | --- |
| ir-approved-defaults | ir-matrix | [1790,2876) | 38-58 | 2a2dbc8cb05b2f637b7fb23538b80b32341ac522152819d85cc92c7cd793c617 |
| ir-identity-namespaces | ir-matrix | [2919,3816) | 61-76 | a68f8e7943da4875126b35d19c2c9577bd55f937d427a051bf9faf0967287ad1 |
| ir-cardinality | ir-matrix | [4999,6962) | 101-145 | 768949712e76e425781988fd2a840636a6dea6fd18b0fb736e2da6139ce3e4fa |
| ir-operation-matrix | ir-matrix | [6962,15344) | 146-198 | 621aaf38a24f4d44d228bbeb294bcfeb26f416fe2ec9524d100f1b4a101057f6 |
| ir-validation-transform | ir-matrix | [23883,25766) | 289-337 | 7449001807e36bb6475dc77f742e7400fd8078cdc1024da60299f576d6db8598 |
| ir-semantic-boundaries | ir-cert | [959,1861) | 32-46 | c34a59ae1cc0abc1684a298e32576f63af4ce4423c6b14a7bf1e8ebd23a11678 |
| ir-durable-recovery | ir-cert | [1861,2791) | 47-63 | b36ba6a9a43f4248d767ed056cf05fbbba3dbc951e92ad7b9c1dd2b26cc50dc1 |
| ir-quarantine | ir-cert | [2791,3294) | 64-73 | 827366f8aef42257fa857388b1e7e8f0c67314aba7fb67774436e1f0438f0766 |
| ir-approval-history | ir-lockdown | [535,1066) | 16-26 | 4012e0c9afa89b815100c1587fc294d5ea9c0e8b361e10300ad70cc7335f5938 |
| ir-binary-equality | ir-binary | [504,1095) | 15-37 | 8c049b161cc8fca24fdd0e5c0d0b5847f1343d54b1f52b4e662d9e3aa4b842a2 |
| ir-binary-legacy | ir-binary | [4394,4732) | 115-120 | aa2574d92a92b505bb5ede7f8b397854454f225efcab3fb66f6a358d8d918ec2 |
| ir-binary-failures | ir-binary | [7962,8644) | 193-212 | cda20669cb120ebbf68d5f0795e60240c7600561607d310f23eb4fce1787fdee |
| ir-projection-separation | invariants | [5700,8621) | 101-115 | e8264f6a1847ddda5cd8b92f95d09d8524cf9f2784f31e6d2739387b8fb80e18 |
| ir-replay-invariant | invariants | [58330,59762) | 379-390 | d969d0fdeca660f02fde3a4d738ef6eae5940046b80034fedb71125c9034c5df |
| ir-checkpoint-invariant | invariants | [108953,111211) | 675-687 | aa6d6e67f8ecc05e3f841bbd6e8d92cc46d59a21eb4bf8aeae275669726f8010 |
| ir-compatibility-invariant | invariants | [111211,115034) | 688-703 | cccbc27410c185514443b09f95840f986da9b880bc147de7f15f4c1b3394fe47 |
| ir-owner-publication | overview | [12886,14782) | 246-266 | 33d61be3c20115698610b3770607b2cfe80fc8e02fddb4dac5cd167343437184 |
| ir-binary-decoder | ir-decode-code | [4377,4788) | 104-114 | b317ef55008b7a3eb32e5ee96b5cbe8622b0662dbc64ec1368a15aa236fcc3d1 |
| ir-compatibility-checks | ir-compat-code | [7599,10294) | 180-249 | bd410e3510208dcbfe657709adafcf4e974d8cd53aebf4557de7e3693089e52c |
