# PCG-175 Corpus 03: enrollment and real MCP evaluation

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11. Authority: owner-approved union enrollment and six MCP queries.
Disposition: **USEFUL**. Three architectural domains coexist in one complete
generation; the four new questions recover distinct mandatory evidence sets.
No production implementation or earlier verification campaign was reopened.

## Enrollment identity and preservation

Actual generation matches the approved prediction exactly:
`b4f290463df4a5134d05b03b699a24eb2bfba748bf3d42f53f5a62d82e4dc361`.

| Identity | Verified value |
| --- | --- |
| Source commit | `6add8449c1a036d0599283fd6a982009d495e954` |
| Manifest SHA-256 | `4417bed80bd8e0a6850ad33859000bf3ab533eb7c7eb48c5e96330bfff7a596c` |
| Profile SHA-256 | `e0422b0eb317f59a22d5267d498bf318cdbfff5c7f445797de19b6db531ea641` |
| Executed accepted binary SHA-256 | `0959d317624514e5cb9e1d79e36e5ba1c48587f7ac84496d31b7edec1d29f76c` |

Preflight rechecked digests, pinned source blob OIDs/bytes, excerpt hashes and
unchanged Corpus 02 definitions. The isolated workspace contains exact selected
Git bytes with detached HEAD at that commit, local alternate objects and the
existing provider/cache/source-persistence-disabled pilot startup configuration.
The approved separate PCG profile supplies the read-only query store. No network
fetch or original-checkout source modification was needed.

Explicit CLI import exited zero: 28 records, 64 evidence entries, 22 source
witnesses, 12 selectors and 63 relationships/bindings. It reported 64 evidence
validations, 543,319 Git bytes and 1,086,638 working-file bytes across both
22-source observation boundaries. A read-only SQLite inspection confirmed all
stored source, evidence, record, selector and edge revisions equal the proposal.
These are pilot operation receipts, not tracked tests or performance benchmarks.

All **158** inventoried Corpus 01/02 review, execution and storage artifacts retain
their original hashes. Their databases, profiles, accepted manifests, revisions
and reports were not modified. Source working-byte preflight hashes also remained
unchanged. Corpus 03 uses its own workspace and database under
`target/pcg-175/corpus03/`; import was a complete union, never an implicit merge.

## Six actual MCP results

A fresh production stdio session initialized with the approved profile. The
catalog exposed one read-only pcg_query_v1 descriptor. Six explicit tools/call
requests completed and the server exited zero. All six results report
`complete_within_registered_scope`, all five coverage checks true, START and
COMPLETION matched, required material fit, and no conflicts or missing/stale
requirements. No profile, budget or evidence definition was changed.

In the table, relationship and citation pairs are **mandatory / total emitted**.
Optional support records are separate from required_records.

| Question | Required records | Optional support | Relationships | Citations | Whole-result o200k tokens / 32,768 | Result bytes |
| --- | --- | --- | --- | --- | --- | --- |
| answer-authority | 13 | 2 | 17 / 19 | 29 / 36 | 28,745 | 111,786 |
| file-fidelity | 5 | 1 | 5 / 6 | 16 / 17 | 12,533 | 47,887 |
| declaration-absence | 9 | 2 | 9 / 11 | 17 / 24 | 19,428 | 76,273 |
| cycle-negative | 4 | 0 | 4 / 4 | 7 / 7 | 6,035 | 23,960 |
| Earlier IR question | 11 | 0 | 16 / 16 | 20 / 20 | 14,864 | 58,399 |
| Original CBM question | 7 | 0 | 15 / 15 | 18 / 18 | 13,647 | 50,776 |

All four new mandatory record, relationship and witness sets match the approved
prediction exactly. No mandatory citation was omitted. Their distinctness is
therefore substantive: one-file fidelity does not pull in the whole workspace
inventory, and indexed-only cycle reasoning does not inherit declaration absence
authority. The whole-result counts include optional material actually delivered.

Optional/diagnostic truncation is false; optional_omissions is `["not_requested"]`
for unrequested history. No budget block occurred. The broad response leaves
4,023 tokens of headroom; this is not a guarantee that future expansions fit.

The existing counter measured the exact serialized MCP result extracted from
JSON-RPC framing, including escaped JSON text and the complete content/isError
wrapper. JSON-RPC framing is excluded by v1. Counter SHA-256 remains
`92ca8701af027627e51f87a02d566a65241d859e1174d57e034ad98a35864382`.
It was reused without a rebuild; this is binary provenance, not a newly certified
counter build. This approved pilot exercised o200k only.

## Observed difference from proposal and oracle

The first independent assessment expected mandatory relationship/citation sets
to equal total output, and stopped on extra material. Inspection showed optional
PRESERVES support fitted the remaining budget. The exact-total expectation was
overly strict; the approved mandatory closure itself was correct.

answer-authority and declaration-absence additionally return the descriptive
links to ir-compatible-adoption and ir-typed-identity, those two records in
`support`, and seven supporting/provenance citations. file-fidelity additionally
returns the existing ir-projection-authority PRESERVES cbm-semantic-authority
link, that optional support record and its overview citation. cycle-negative
has no optional addition.

The assessment now separately verifies exact mandatory sets and the specific
optional additions, including all their record revisions, links and witnesses.
No approved oracle/manifest/profile was edited, no assertions about required
closure were relaxed, and no product correction or MCP rerun was needed.
A support record can retain stored `required:true` while being optional in this
question's traversal: its placement in `support`, rather than the stored flag
alone, determines its role in this response. Consumers must preserve that
distinction and must not promote PRESERVES into a prerequisite.

## Independent engineering-answer assessment

The semantic oracle was read independently of future-result predictions. Returned
source text supports all its required distinctions:

1. **Empty observation versus absence:** zero or nonzero counts do not establish
   coverage. Authoritative observed facts may still form only a lower bound.
   Scope/generation/evidence requirements, producer support and current
   dependencies are necessary before a bounded certificate permits exactness.
   Citations: wq-facts-invariant, wq-completeness-invariant, wq-certificate-code.
2. **Declared source universe:** admitted roots, exclusions, source-language and
   producer/capability units define the universe. Inventories retain exact source
   hashes; proof acceptance rejects target/scope/inventory/admission/version
   mismatch, nonexhaustiveness, truncation or limitations. Traversal/read failures
   cannot silently disappear into a successful empty inventory. Citations:
   wq-scope-invariant, wq-coverage-units, wq-inventory-fingerprints,
   wq-source-enumeration, wq-source-reading, wq-proofs.
3. **Current file fidelity:** Edit/Verbatim query requests normalize to High
   semantic compilation; actual Edit/Verbatim compilations are not automatically
   semantic supersets. Reuse needs current source/dependency/retained authority
   and sufficient normalized fidelity. Otherwise compile and atomically replace
   the file contribution, including empty projections. Supported-pattern omissions
   still limit absence claims. Citations: wq-file-fidelity-invariant,
   wq-semantic-fidelity-code, wq-current-projection-code, wq-certificate-code.
4. **Occurrence versus declarations:** ordinary find_entities stays indexed
   lower-bound; declarations_only may earn a scoped SourceDeclarations certificate.
   Independently proven declaration existence does not depend on a derived
   relationship surviving, nor can an exact declaration be guessed from a shared
   semantic tuple. Citations: wq-entity-mode-coverage,
   wq-direct-declarations-invariant, wq-authority-model, wq-certificate-code.
5. **Negative cycles:** false means no eligible retained-index cycle, not no
   workspace cycle. Only Injects/ImportsModule are eligible; witness provenance
   and ambiguity remain explicit. The operation performs no implicit discovery,
   hydration or source compilation and retains source_complete=false. Citation:
   wq-cycle-invariant. PCG's complete delivery of this contract does not upgrade
   that separate workspace-query result to source-complete.
6. **Failure propagation:** unsupported inputs and source, compilation,
   contributor or typed-publication failure take the existing error path before
   a successful query answer. They do not become empty-success results. Query-only
   compilation does not manufacture rendered-context/durable-baseline claims.
   Citations: wq-file-preparation-code, wq-publication-code,
   wq-file-publication-code, wq-error-envelope-code, wq-single-outcome-code.
7. **Authority separation:** CBM contributes advisory discovery, not WorkspaceIndex
   semantic truth or exhaustiveness. Canonical semantic state, semantic coverage
   and model-facing fidelity are separate. Diagnostics explain barriers but cannot
   mint completeness; warm state changes work rather than certified-domain truth.
   Citations: wq-discovery-invariant, overview-authority,
   ir-projection-separation, wq-publication-fidelity, wq-detail-invariant,
   wq-history-invariant.

A reader can derive these answers from the returned excerpts without opening
source files. This is assessment of retrieved architectural evidence, not a live
exercise of workspace_query failure/compile behaviors or a new regression result.

## Citation integrity, preservation and usefulness

Independent artifact checks passed for every required and optional record
revision, edge kind/endpoints/revision/provenance, primary/required witness and
citation. Every citation's UTF-8 text equals the exact pinned blob slice, with
matching offsets, quote/source hashes, commit, blob and source path. Evidence IDs
are unique in each output; shared required witnesses are not duplicated.

Both earlier-domain payloads equal their accepted retained counterparts after
removing only the generation field. No old record, relationship, clause,
coverage or budget metadata changed. Their small token-count changes arise from
the new serialized generation value, not altered evidence.

Corpus 03 main DB hash is identical before and after all six queries:
`1abb0759de2f9224ed817fb884b34c51491e4ca67de1772dd3faf59b433db7aa`.
Binary/counter hashes also remained unchanged. This is main-file/read-only
evidence, not a complete filesystem-side-effect or physical-erasure certificate.

**USEFUL:** all three domains coexist, both old answers remain retrievable, and
four source-justified questions select distinct complete mandatory closures at
different measured context sizes. The exercise demonstrates selective retrieval
without weakening evidence; it does not measure human time saved or enterprise
performance. No future design change is required to close this evaluation.

Limitations: manual registered scope and exact operation/task labels; attributed
source authority; explicit supported-pattern omissions; pinned historical commit;
boundary-only freshness and admission/accessed-row integrity. Narrower output
still checks all 22 corpus-current witnesses. Optional graph support increases
actual output beyond mandatory counts, so tooling/oracles must distinguish them.
No continuous/atomic freshness, authenticated approval or automatic semantic
discovery is claimed.

No production code, accepted definitions, prior stores or reusable verification
assets changed. No builds, tests, CI, benchmarks, commits, pushes or publication
occurred. Only ignored Corpus 03 execution aids/evidence and this report were
created for the authorized run.

## Retained evidence

- [Enrollment identity](../../target/pcg-175/corpus03/execution/enrollment-identity.json)
- [Import output](../../target/pcg-175/corpus03/execution/import.stdout.json)
- [Session/binary/counting identity](../../target/pcg-175/corpus03/execution/mcp-session.json)
- [Six measured summaries](../../target/pcg-175/corpus03/execution/mcp-results.json)
- [Independent assessment](../../target/pcg-175/corpus03/execution/independent-assessment.json)
- [Broad answer payload](../../target/pcg-175/corpus03/execution/answer-authority.payload.json)
- [File fidelity payload](../../target/pcg-175/corpus03/execution/file-fidelity.payload.json)
- [Declaration absence payload](../../target/pcg-175/corpus03/execution/declaration-absence.payload.json)
- [Negative cycle payload](../../target/pcg-175/corpus03/execution/cycle-negative.payload.json)
- [IR preservation payload](../../target/pcg-175/corpus03/execution/ir-preservation.payload.json)
- [CBM preservation payload](../../target/pcg-175/corpus03/execution/cbm-preservation.payload.json)
- [Prepared semantic oracle](../../target/pcg-175/corpus03/review/independent-oracle.md)
- [Approved proposal](PCG_175_CORPUS03_PROPOSAL.md)

Each query also retains its request, raw JSON-RPC response, exact serialized MCP
result and independent token-counter stdout/stderr under the same execution root.
