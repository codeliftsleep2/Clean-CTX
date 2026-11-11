# PCG-175-M3: token-bounded context projection

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11 (America/New_York).
Authority: owner instruction **"proceed to m3"**, following acceptance of M2a-2.
Normative basis: [approved design freeze](PCG_175_DESIGN_FREEZE.md) §§4.3, 7, 8,
10.1 and M3 in §10.4. M2a-1 and M2a-2 verification remain closed.

Disposition: **M3 ACCEPTANCE READY** following owner-authorized M3-VERIFY.
All eight focused tests passed; seven-file formatting/check and library-scoped
Clippy passed. No functional or lint correction was necessary in this verification
mission; only formatting changed. M2a-1/M2a-2 verification remains closed.
No broad gate, operator campaign, application/server or M4 integration was run.
Owner acceptance remains distinct from executed verification; M4 is unauthorized.
## 1. Implemented boundary

The explicit Rust entry is
`pcg::projection::project(profile_path, acquisition_request, tokenizer, token_budget)`.
It returns the exact serialized MCP result string, its measured token count and
operation-local diagnostic counters. Token count/counters are outside the counted
result; they are not inserted into its metadata. Only local exact `cl100k` and
`o200k` tokenizers are accepted; initialization errors and unsupported tokenizers
have fixed errors with no raw-text fallback.

```text
trusted profile + acquisition request + tokenizer/budget
  -> existing read-only generation acquisition and mandatory closure
  -> START source witnesses and exact mandatory citation materialization
  -> bounded optional enrichment using already witnessed sources
  -> canonical fragments + cached MCP-escaped citation fragments
  -> pack/count entire MCP result; remove optional material first
  -> COMPLETION source/profile observations inside the same read transaction
  -> release the exact counted result bytes, or discard draft and meter control
```

The whole counted result is one text block plus `isError:false`. The text block
contains canonical payload JSON. Its quotes, slashes, Unicode and metadata are
counted in their actual escaped MCP representation. There is no second
`structuredContent`, reported used-token field inside the counted object, JSON-RPC
request ID/framing, or source/code raw fallback. M4 must emit these exact result
bytes, rather than wrap or reserialize a different representation.

Packing assembles a draft serialized representation for exact counting before
COMPLETION. Final emission reuses those bytes after the observations; no evidence
is escaped again or additional metadata inserted after counting. An observed
COMPLETION failure discards the entire draft and separately serializes/meters a
safe control response. This leaves the explicit final-emission TOCTOU gap and
non-atomic filesystem assurance unchanged; it does not promise continuous truth.

The existing `query::acquire` entry retains its original acquisition behavior.
Its new internal packing callback is a no-op for existing callers. M3 invokes the
callback after mandatory materialization and before COMPLETION, while the reader,
transaction, profile and work owner remain alive. There is no second import,
query/reacquisition retry, SQLite writer or source-semantic publication.

## 2. Changed files

| Path | Responsibility |
| --- | --- |
| `src/pcg/projection.rs` | Explicit library entry; exact tokenizer/budget validation; in-transaction packing orchestration; discard-on-COMPLETION-failure and fixed control errors. |
| `src/pcg/projection/payload.rs` | Closed v1 payload shaping, omission codes, deterministic selection, optional-first packing and bounded required-overflow controls. |
| `src/pcg/wire.rs` | Canonical JSON and escaped text-string fragments; reuse one citation fragment per evidence revision across packing candidates. |
| `src/pcg/optional.rs` | Direct descriptive support and optional evidence refs; history permission checks, explicit omissions and reservation of mandatory COMPLETION byte work. |
| `src/pcg/query.rs` | Narrow internal packing seam; existing public acquisition calls it with a no-op. No replacement of applicability, closure, witness or error policy. |
| `src/pcg/mod.rs` | Register the focused modules; expose the explicit projection library entry. |
| `src/tests/pcg/projection.rs` | Eight focused tracked test definitions, registered normally from the projection module. |
| M2a-2 report | Record subsequent owner acceptance and separate M3 authorization without rewriting its historical execution claims. |
| This report | Source attribution, canonical questions, implementation limits and scoped verification handoff. |

Against the retained M2a-2 final-source inventory, only `mod.rs` and `query.rs`
changed among previously inventoried source files. Existing M2a-1 persistence,
enrollment, identities, security and import files are untouched. Existing M2a-2
tests are untouched. No schema, migration, dependency, MCP registration, tokenizer
implementation, canonical IR, replay, delta, source cache, publication or global
limit/retry/cache policy changed.

## 3. Required evidence and optional enrichment

Mandatory records, relationships and citations are retained in full. Required
spans are never sliced or replaced with summaries. Required citations stay shared
across primaries, refs and relationship provenance. Optional target `required`
flags do not promote descriptive edges into mandatory closure.

Optional enrichment offers direct `USES_EVIDENCE`, `PRESERVES` and `IMPLEMENTS`
targets, plus explicit optional evidence refs. It is not an exhaustive history
search or recursive relevance traversal. Optional history requires request and
profile permission. It uses only source bodies already acquired by the mandatory
witness set, including all corpus-current bodies and selected historical bodies.
Off-path optional historical bodies are omitted with `unavailable`; callers can
explicitly request a historical record through normal acquisition when permitted.
This optional-body limitation does not certify absence or weaken required closure.

Optional source/access loss records an omission. Confirmed accessed-row or quote
corruption fails closed, including when discovered while offering optional
material. Secret-denied material is not emitted. Optional work exhaustion stops
enrichment and records `work_limit`, preserving completed mandatory coverage.
The first current observation's byte count is reserved before optional reads so
optional evidence cannot consume the normal unchanged-source COMPLETION budget.
A concurrent source expansion can still exhaust the owner ceiling; observations
and work limits are not an atomic filesystem snapshot.

Packing removes optional history, support records and standalone optional
citations before declaring required overflow. Every candidate counts the actual
whole result including omission metadata. Remaining records/edges cite only
present citation fragments. When the required result cannot fit, no partial
actionable required records, relationships or citations are returned:
`blocked/required_budget_exceeded`, with `required_material_fit:false`.
If the control envelope itself does not fit, the library returns fixed
`token_budget_too_small`. M4 will own mapping fixed library errors into its
approved protocol errors; no tool is registered by M3.

## 4. Canonical questions and consumer audit

| Question | Authority and acquisition | Answer / adversary / public boundary |
| --- | --- | --- |
| Which exact enrolled decision revision is being conveyed? | Existing exact generation, scoped record revisions and source witnesses | Closed typed records and exact source/span/hash citations; no generated decision prose. Public MCP reachability remains M4. |
| Are all mandatory constraints and proof paths retained? | M2a-2 mandatory evidence closure, without caller-selectable weaker edges | Full required bundle or explicit undelivered control; optional support never discharges a premise. |
| Why is a decision protected, rejected or contradicted? | Exact required record and counterevidence citations from acquisition | Source-attributed authority and proposition-specific execution category; no detector-PASS/equivalence promotion. |
| Does this fit the actual model-facing result budget? | Exact cl100k/o200k counter over escaped whole MCP result | Result metadata/wrapper/citations all count; approximate tokenizer, mandatory overflow and too-small control refuse explicitly. |
| Did source permission or bytes change while packing? | Same transaction, START witnesses, COMPLETION after draft packing | Discard complete draft on observed failure; never assert atomic/continuous currentness or detection of unobserved ABA. |
| Does omission mean no other evidence exists? | Named enrolled required scope, optional omission categories | Complete required context can omit optional material; no optional absence proof, history count disclosure or enterprise corpus claim. |
| Can a real MCP client consume it now? | Requires M4 registry/schema/handler and exact-byte response consumption | **Not yet integrated.** This stage exposes a Rust library response only; no stdio, Claude or legacy-response execution is claimed. |

Creation -> acquisition -> generation read owner -> packing -> source recheck ->
returned bytes is present in source. SQLite persistence remains the existing
derived generation owner. No new durable projection/cache is written. The
future consumer is the separately authorized M4 handler; it is absent now.
Restart/session behavior still derives from explicit acquisition, not hidden
runtime memory. Entity requests retain their existing `identity_unavailable`
limitation; source-attributed approval is not authenticated runtime approval.

## 5. AF-PERF scope and work

Operation scope is one explicit projection over one pinned acquisition. Necessary
complete-set work remains applicability, mandatory closure and both source
observations. Optional reads use remaining request work with a COMPLETION byte
reservation; no persistent memo cache or changed corpus freshness policy exists.

Required and offered citation revisions are validated once by acquisition or
optional materialization. One canonical citation fragment and its one escaped
MCP text-string form are encoded once per revision and reused through all packing
candidates. Execution attribution is indexed once from offered record primaries,
not reconstructed by scanning the authority set for each citation.

Let U be optional items, K the examined selected record/edge/citation references,
C the distinct citation bytes, and P the cumulative bytes passed to the tokenizer.
Fragment construction covers C once. This simple optional-first packer may build
and meter U+1 candidate envelopes and one control envelope, with selection work
up to O(U*K) plus cumulative output assembly/tokenization P. P can exceed final
output size substantially; no final-token-only cost bound is claimed. Mandatory
acquisition costs and full source bytes remain separate from projection work.
No optimization, latency reduction, allocation saving or enterprise benefit is
asserted without measurements.

Diagnostic counters expose tokenization calls, cumulative tokenizer-input bytes
and citation encodings outside the counted evidence result. The new focused tests
assert citation sharing and repacking counts. These are now executed focused assertions (§8),
not measured evidence or an additional scaling population.

## 6. Focused verification handoff

Eight new tracked cases under `pcg::projection::tests::`:

1. `exact_serialized_result_and_unicode`: both exact tokenizers; independently
   count emitted bytes, retain Unicode/escapes/hashes and wrapper coverage.
2. `optional_first_dropping_preserves_required_material`: preserve complete
   mandatory material; drop optional support/citations; reuse encoded citations.
3. `mandatory_overflow_blocks_and_tiny_control_rejects`: independently size a
   control envelope, require blocked undelivered authority and tiny-budget error.
4. `completion_failure_discards_counted_draft`: change source after packing;
   require a separately metered blocked response with no evidence payload.
5. `optional_work_limit_preserves_required_completeness`: exhaust only optional
   record acquisition; keep mandatory evidence and completed observations.
6. `exact_fit_and_deterministic_repeated_projection`: identical bytes across
   repeated reads and an independently calculated exact-fit boundary.
7. `approximate_tokenizer_and_profile_budget_refuse`: no approximate fallback
   and no budget enlargement beyond the trusted profile.
8. `shared_required_witness_encoded_once`: three required records share one
   citation and exactly one citation encoding.

Original bounded handoff commands, subsequently executed under explicit M3-VERIFY
authority as recorded in §8, without reopening prior campaigns:

```powershell
$pcgM3Files = @(
    'src/pcg/mod.rs', 'src/pcg/query.rs', 'src/pcg/projection.rs',
    'src/pcg/projection/payload.rs', 'src/pcg/optional.rs', 'src/pcg/wire.rs',
    'src/tests/pcg/projection.rs'
)
rustfmt --edition 2024 --config skip_children=true @pcgM3Files
rustfmt --edition 2024 --check --config skip_children=true @pcgM3Files
cargo test --package clean-ctx --lib --all-features pcg::projection::tests::
cargo clippy --package clean-ctx --lib --all-features -- -D warnings
```

The focused filter selects the eight M3 cases, not the M2a-2 campaign. Library
Clippy cannot select an internal Rust module in isolation; its scope is one
package/library and dependencies, excluding test/example/binary/proxy targets.
Compilation cost is cache-dependent; the prior focused library-test build took
5m 53s, so no fast build promise is made. Formatting is limited to the seven
listed files with child traversal disabled.

Repository documentation still describes broad final gates and a commit between
migration phases. The owner explicitly authorized M3 with earlier work uncommitted
and explicitly closed prior verification; that instruction controls this mission.
No commit was made. Broad tests, all-target/workspace Clippy and final CI remain
unrun, not failed or implicitly green. The earlier full suite exceeded five hours
and is unnecessary for this bounded handoff; no broad execution is requested.

## 7. Source identity and static inspection

HEAD: `8ce0a6865ff8a2b55866895c6a239f6bcc3245a3`.
Tree: `10d8ecadd0a151ea0638e2a066bd50b40ad022e0`.
Branch: `(fix)Performance-Regression-Issues`.
These remain the local uncommitted implementation basis, not a live remote claim.
No branch/ref/index/commit/push/publication action occurred.

The retained prior verified source inventory is
`target/pcg-175/m2a2-verify/final-source.sha256`, fingerprint
`309ba06bab6d87b0aa35884be567b18fc77a03d7c3efb23e533ca672949a16f2`.
It is historical execution attribution, not a PASS for the changed query seam.
The M3 source inventory is retained at
`target/pcg-175/m3-implementation/source.sha256`, fingerprint
`1d7f7348fb0153e61dceb2755e7b5391acf87d7418fbe641d09b6f5251090d16`.
Scope and encoding match the prior inventory: lib/main plus recursive
PCG production/test files, ordinal slash paths, lowercase file SHA-256, two spaces
and LF per entry, final LF included. Reports are excluded to avoid self-reference.

Bounded native source/hash review and byte-level strict UTF-8/BOM,
replacement-character, trailing-whitespace and code-line-ceiling inspection were
performed. All seven new/modified code files remain below 615 lines.
`git diff --check` reported no whitespace errors; its normal lib.rs CRLF checkout
notice is not a compiler warning. These checks do not establish compilation,
formatting, lint cleanliness or runtime behavior.

M3 focused verification is closed by §8; owner acceptance is the remaining gate.
M4 registration, automatic injection, production enrollment, approval writes,
enterprise hosting and publication remain outside authority.

### Historical owner verification follow-up

The owner reported compilation failing with E0599 at three calls to nonexistent
`Fixture::save` in the new projection tests. Corrected those calls to the existing
`Fixture::approve`, which writes the updated manifest, pins its exact digest and
writes the fixture profile. Shared fixture support and production code are
unchanged. This is a test-definition compilation failure, not behavioral RED.
No rerun result is supplied and no execution was launched by the agent.
The inventory/fingerprint above identifies the earlier delivered source; this
three-call correction supersedes that test-file revision.

The owner then reported `optional_work_limit_preserves_required_completeness`
returning incomplete. Source tracing showed its limit of three was exhausted
during mandatory work: two selectors, one binding and one required record debit
four record-budget units. Bindings debit both the record and edge budgets. The
fixture now allows exactly four units and first asserts that ordinary mandatory
acquisition is Ready with that exact count. Optional target acquisition must then
exhaust the remaining capacity. Original complete-result, required-record,
COMPLETION and optional work-limit omission assertions are unchanged. Production
code is untouched. This corrects a misconfigured negative fixture, not a proved
production completeness defect. The revised case remains unrun by the agent.

The owner subsequently reported **GREEN** for the handed-off exact case:
`cargo test --package clean-ctx --lib --all-features pcg::projection::tests::optional_work_limit_preserves_required_completeness -- --exact`.
This is owner-reported focused tracked-test evidence for the corrected fixture;
no detailed output or duration was supplied. The agent did not run it. No complete
eight-case M3 result, final formatting outcome or library Clippy result is inferred
from this confirmation. Prior-stage verification stays closed.

## 8. M3-VERIFY: actual results and final source

Authority: owner explicitly authorized changed-file formatting/check, the eight-case
M3 filter and production library Clippy, plus necessary in-scope corrections.
No earlier investigation or closed-stage test/operator campaign was repeated.

| Actual command | Result | Retained log under `target/pcg-175/m3-verify/` |
| --- | --- | --- |
| `rustfmt --edition 2024 --config skip_children=true @pcgM3Files` | Exit 0; formatted exactly the seven paths in §6, with child traversal disabled. No stdout/stderr. | `format.log` (empty output) |
| `rustfmt --edition 2024 --check --config skip_children=true @pcgM3Files` | Exit 0; formatting check passed. No stdout/stderr. | `format-check.log` (empty output) |
| `cargo test --package clean-ctx --lib --all-features pcg::projection::tests::` | Exit 0; **8 passed, 0 failed, 0 ignored, 0 measured**, 5,143 filtered out. Compilation 1m 10s; test execution 8.57s. | `focused-tests.log` |
| `cargo clippy --package clean-ctx --lib --all-features -- -D warnings` | Exit 0; **zero warnings**, finished in 23.95s. | `library-clippy.log` |

The exact eight case names and negative controls are listed in §6; all passed.
This includes independent whole-result counts for both exact tokenizers,
mandatory preservation under optional dropping, required overflow/tiny control,
after-packing source mutation with draft discard/control remetering, optional-only
work exhaustion, deterministic/exact-fit output, profile budget refusal and shared
citation encoding. The suite ran once after formatting. No case failed, so no
additional affected-case rerun or repeated full focused pass was necessary.
The earlier fixture corrections remain historical; this mission made no semantic
source change, test population change, assertion weakening or lint suppression.

Final formatted, tested and linted source fingerprint:
`7052fd8a44e3ed5eee8052dbee00bf73b0cf3dffea6b317d1d9b098b7f8084a8`.
Inventory: `target/pcg-175/m3-verify/source.sha256`, using the exact scope/encoding
in §7. It includes the fixture corrections and seven-file formatting and
supersedes the initial-delivery fingerprint for current verification attribution.
No source file was edited between inventory capture, the passing test run and
passing Clippy. HEAD/tree/branch remain those in §7. Comparison to the M2a-2
inventory still identifies only `mod.rs` and `query.rs` as changed existing files;
all other previously inventoried source/test files remain byte-identical.

Log SHA-256:

- `focused-tests.log`: `74e1f042654d7ff0938d5470ebf8e79ac3e487de3a9c99619501d8cb0a45f328`.
- `library-clippy.log`: `1f70953da900c09d5a2fb9cc08081fb126ed3a22cba6da312001dea9e3514017`.
- Each empty formatting log: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.

Static inspection passed strict UTF-8/no BOM, replacement-character,
trailing-whitespace and line-ceiling checks. Formatted code lengths including
final empty split line: mod 59, query 325, projection 133, payload 343, optional
168, wire 52 and projection tests 328; all below 615. `git diff --check` passed;
the existing lib.rs CRLF checkout notice is not a compiler warning.

The passing assertions establish the stated deterministic projection contracts
for these fixtures. Shared-citation and tokenization counters were asserted by
tests; no additional raw counter population, benchmark or efficiency measurement
was collected. Test/compile/lint durations are execution receipts, not projected
runtime performance or enterprise benefit.

The packing-before-COMPLETION order and exact-byte M4 handoff are unchanged.
Boundary-only freshness, trusted local storage, optional off-path history and
entity-identity limitations in §§1/3/4 still apply. No MCP registration, server,
Claude run, production corpus enrollment, broad suite, all-target Clippy, new
benchmark or prior-stage campaign was launched. Those unrun activities are not
failures and are not claimed green. No staging, commit, ref change or publication
occurred. **Recommend M3 ACCEPTANCE READY; stop before M4.**