# PCG-175 Corpus 03: workspace-query answer authority

Agent: **Agent MaxHeadRoom**

Date: 2026-11-11. Authority: Corpus 02 accepted; Corpus 03 Phase A only.
Disposition: **READY FOR OWNER MANIFEST/PROFILE REVIEW — NOT IMPORTED**.
No production development or earlier verification campaign is reopened.

## 1. Architectural finding and proposed question scopes

**OBSERVED:** zero matches is not a completeness signal. The current contracts
separate authoritative observed facts, exhaustive source coverage and sufficient
semantic fidelity. A current supported file projection can certify file-scoped
emptiness; a partially indexed workspace cannot establish workspace absence.
`has_cycle=false` explicitly remains indexed evidence. Declaration absence has
its own proof-backed acquisition boundary. Generic automatic compilation is not
an approved substitute for these distinctions.

The requested engineering question is registered as `answer-authority`:

> When a workspace query returns no matches, how can Clean-CTX determine whether
> the answer is genuinely empty, the requested source universe is incompletely
> indexed, or the available semantic representation lacks sufficient fidelity?
> What must happen before the answer can be treated as authoritative?

Three narrower questions are proposed alongside it. Each uses exact conjunctions
of operation and task; each has its own coverage/applicability selectors. These
are PCG question labels, not changes to workspace_query or new MCP operations.

| Question | Exact operation / task | Necessary answer distinction | Predicted mandatory records / relationships / evidence |
| --- | --- | --- | --- |
| answer-authority | `workspace_query` / `explain_empty_authority` | All three failure/absence cases and their authority boundaries | 13 / 17 / 29 |
| file-fidelity | `entities_in_file` / `explain_empty_fidelity` | Is an empty result at this requested semantic fidelity valid for one explicit current file? | 5 / 5 / 16 |
| declaration-absence | `find_entities` / `explain_declaration_absence` | When can declaration mode prove absence, and why cannot occurrence-mode emptiness do so? | 9 / 9 / 17 |
| cycle-negative | `has_cycle` / `explain_negative_cycle` | What does a false cycle result actually establish without source acquisition? | 4 / 4 / 7 |

Workspace stays `cleanctx-cbm-cache-pilot01`; effective time is explicitly
`2026-11-11T12:00:00Z`. There are no file/entity selector guesses. The declaration
question is a reviewed conceptual scope covering both occurrence and declaration
modes; it does not add an unsupported `declarations_only` selector dimension to
PCG. That mode distinction is in the cited contract, not an invented v1 field.

These predicted sets are least required closures under the existing v1 rules.
They differ because file-local evidence does not require a workspace-wide
inventory, and an indexed-only cycle answer cannot inherit declaration absence
authority. No quote was shortened or required ref made optional to obtain this
selectivity. Token fit and runtime retrieval remain **UNVERIFIED**.

## 2. Accepted basis and complete-union strategy

Current committed HEAD: `6add8449c1a036d0599283fd6a982009d495e954`.
Tree: `d1540017265328e4bbb0b4ea31d6a8bc7172cb1a`.
Branch: `(fix)Performance-Regression-Issues`. No live remote-tip claim is made.

The Corpus 02 approved manifest/profile digests were rechecked. Read-only SQLite
inspection confirmed its accepted generation
`e304524614d3df0d6512efbaea0a30f31b94b1eaec54c7e6120c26be6f5f83f9`,
17 records and exact stored record revisions. Its retained MCP evidence remains
accepted historical pilot evidence; none of its queries were rerun.

The new manifest is a complete union, never an incremental-merge request. All
Corpus 02 source/evidence/record/selector/edge definitions and every corresponding
revision compare equal to the prior proposal. All 158 inventoried Pilot 01 and
Corpus 02 review/execution/storage files retain their hashes. Neither database
was modified. Original reports remain untouched.

The proposed physical workspace and DB are separate:
`target/pcg-175/corpus03/workspace/` and
`target/pcg-175/corpus03/storage/projection.sqlite`.
Neither has been created. Partition, corpus, workspace, owner and policy labels
are retained as stable logical IDs, including historical `pilot01` labels.
The importer would publish only the supplied complete generation; it does not
merge omitted records from an older head. Older approved stores/profiles remain
independent and usable. No supersession or source-truth replacement is proposed.

All 22 source rows use exact immutable blobs at the same commit, `check=current`.
Some original working files differ through CRLF checkout representation. Future
enrollment must reproduce the exact Git bytes in the isolated workspace; it must
not enroll the original checkout by silently normalizing or weakening witnesses.
Source/record revisions identify this pinned snapshot, not later source changes.

## 3. Source authority and exclusions

The primary fidelity document labels itself **implemented** for entities_in_file
despite its PROPOSAL filename. Its §§4–5 provide the normalization/coverage
decision; current WSC-006 and implementation qualify older prospective wording.
Its wider audit candidates and historical pre-fix empty response are not enrolled.
The separate cycle proposal was inspected for its implemented outcome, but current
WSC-007 suffices; obsolete pre-implementation relation-policy prose is excluded.

`SEMANTIC_AUTHORITY_ARCHITECTURE.md` labels itself CURRENT AUTHORITY with an older
audit baseline and explicitly defers to production/invariants. Selected spans
cover tuple-versus-declaration identity and compatible validity. Their presence
does not certify a new audit. The named invariant entries supply current
normative decisions; their embedded test/gate claims are attributed source text,
not freshly executed acceptance.

The source inventory adds two documents and seven implementation files to the
13 existing sources. The inventory/evidence tables below identify them exactly.
The code excerpts are necessary to distinguish proof-bearing source inventories,
strict proof acceptance, normalized request versus actual compilation fidelity,
current dependency checks and error responses. No historical audit, test receipt,
schema dump or entire transitive code tree is automatically enrolled.

New normative records use `verified_claim`, explicitly source-attributed;
`wq-failure-not-empty` uses `attributable_observation` from inspected code. All
are required active constraints with `supported` for the named proposition and
execution `not_run`. None authenticates an approver or claims current test PASS.
This corpus explains workspace-query guarantees. PCG's own
complete_within_registered_scope is not a certificate that a separate workspace
query acquired all its sources.

## 4. New constraints and required evidence

| Record | Primary | Required supporting evidence | Constraint |
| --- | --- | --- | --- |
| wq-observed-vs-complete | wq-facts-invariant | wq-discovery-invariant | Counts do not establish coverage; advisory discovery never fabricates semantic facts. |
| wq-source-scope | wq-scope-invariant | wq-authority-model | Declared primary/additional roots and narrowing use occurrence provenance; tuple identity is not exact declaration identity. |
| wq-source-universe | wq-completeness-invariant | wq-inventory-fingerprints, wq-source-enumeration, wq-source-reading, wq-coverage-units, wq-proofs | Every required provider-neutral unit needs a compatible exhaustive proof with no truncation/limitations. |
| wq-proof-certificate | wq-completeness-invariant | wq-certificate-code | Only bounded scope/generation/evidence satisfaction permits exact absence claims. |
| wq-file-fidelity | wq-file-fidelity-invariant | wq-fidelity-decision-contract, wq-semantic-fidelity-code, wq-current-projection-code, wq-publication-fidelity | Current sufficient normalized semantic coverage or atomic query-only recompilation, including empty replacement. |
| wq-index-only-negative | wq-cycle-invariant | wq-entity-mode-coverage | No indexed cycle/matches does not establish workspace absence; operation-specific lower bounds remain explicit. |
| wq-semantic-omissions | wq-completeness-invariant | wq-certificate-code, wq-publication-fidelity | Exactness is limited to supported producer patterns; declared omissions are not absent semantics. |
| wq-failure-not-empty | wq-file-preparation-code | wq-publication-code, wq-file-publication-code, wq-error-envelope-code, wq-single-outcome-code | Source/compile/dependency/publication failure stays an error, not a successful empty array. |
| wq-diagnostics-not-proof | wq-detail-invariant | none beyond primary | Coverage detail explains processing/proof barriers; it cannot strengthen the exact/lower-bound tuple. |
| wq-history-independent | wq-history-invariant | none beyond primary | Warm state changes cost, not certified-domain truth; generic families retain lower bounds. |
| wq-direct-declarations | wq-direct-declarations-invariant | wq-entity-mode-coverage, wq-authority-model | Current source proves declaration existence independently of derived relationship survival. |

The certificate and source-universe records share a normative excerpt but model
different propositions: evidence acquisition obligations versus permission to
emit an exact envelope. One shared citation identity prevents quote duplication.
These records add query-specific authority conditions rather than duplicate
binary, replay, persistence or CBM ownership records. The existing
`cbm-semantic-authority` and `ir-projection-authority` are reused unchanged.

## 5. Prerequisites and cross-domain relationships

| Required prerequisite | Provenance | Why necessary |
| --- | --- | --- |
| wq-source-universe -> wq-source-scope | wq-completeness-invariant | Completeness needs an admitted declared scope before source units have a bounded meaning. |
| wq-proof-certificate -> wq-source-universe | wq-completeness-invariant | Workspace absence authority requires all required coverage units, not merely observed matches. |
| wq-file-fidelity -> wq-source-scope | wq-file-fidelity-invariant | File exactness is for the authorized canonical file, not a same-named occurrence elsewhere. |
| wq-file-fidelity -> ir-projection-authority | wq-file-fidelity-invariant | Semantic coverage cannot be established by confusing rendered/Edit/Verbatim presentation with certifiable semantic compilation. |
| wq-observed-vs-complete -> cbm-semantic-authority | wq-discovery-invariant | Only Clean-CTX compiled semantics count as WorkspaceIndex facts; CBM paths/relationships cannot discharge this condition. |

Two additional links are descriptive `PRESERVES`:
`wq-proof-certificate -> ir-compatible-adoption` cites wq-authority-validity;
`wq-direct-declarations -> ir-typed-identity` cites wq-authority-model. They respect
the existing compatibility/identity boundaries without making durable restore or
IR binary transport prerequisites of every in-memory query. They may remain
unreturned optional links under v1's mandatory-only response behavior.

No causal dependency is inferred from common names or document order. In
particular, a directly proven declaration does **not** depend on a complete
workspace inventory to exist; the declaration-absence question separately binds
the certificate/inventory records because absence has a stronger obligation.
No statement makes file-local auto-compilation exhaustive workspace hydration.
No VALIDATES/approval edge or runtime reclosure is fabricated.

## 6. Fidelity, universe and failure interpretation

Request Edit/Verbatim normalize to High for file semantic compilation. Actual
Edit/Verbatim compilations do not automatically certify High semantic coverage:
`SemanticFidelity::from_compilation` refuses that promotion. Reuse checks source
hash, minimum semantic fidelity and current dependencies/retained authority.
Replacing a stale projection must remove old contributions even if the new set
is empty. Stronger internal acquisition does not silently upgrade presentation.

The source universe is the admitted registry-supported, policy-filtered inventory
over declared roots, not every path in the repository or every configured language
implementation. Enumeration honors built-in/configured exclusions, Git ignore
policy and no-follow-links; traversal/read failures affect completeness. Coverage
units are producer/capability/generation bound. Unknown producer capability must
not be silently removed from obligations. Exact proof validation checks target,
scope, inventory/admission identities, predicate/contract versions and exhaustive,
untruncated, unlimited disposition. No provider readiness or cache count suffices.

Current entities_in_file implementation includes conditional framework contributor
hydration before its file projection. The older proposal's single-file rationale
must not be read as a blanket claim that the current handler never hydrates any
contributors. It also has a source-bound Angular-template branch. The code
witness preserves these conditions; unsupported semantics remain explicitly
limited rather than inferred absent.

Some comments in the current handler still describe an older minimal-zero scope
response, but executable control flow returns `QueryFailure` for that condition.
The selected failure record follows the actual code. Compile/contributor/typed
publication failure propagates before successful query preparation. Unsupported
file type is -32602; compilation/internal failure uses -32603 here. The single
response boundary preserves errors rather than wrapping them as empty success.
No new error behavior or broader generic-family failure policy is proposed.

## 7. Proposed identities and local review material

Manifest SHA-256:
`4417bed80bd8e0a6850ad33859000bf3ab533eb7c7eb48c5e96330bfff7a596c`.

Profile SHA-256:
`e0422b0eb317f59a22d5267d498bf318cdbfff5c7f445797de19b6db531ea641`.

**STATIC PREDICTED — NOT IMPORTED** generation:
`b4f290463df4a5134d05b03b699a24eb2bfba748bf3d42f53f5a62d82e4dc361`.

Union: 22 sources, 64 evidence entries, 28 records, 12 selectors and 63 edges
(42 bindings, 18 prerequisites, three descriptive preservation links). Additions
are nine sources, 27 excerpts, 11 records, eight selectors and 31 edges. Source
bytes total 543,319; new excerpts total 52,848; union excerpts total 118,020.
Actual source-check work includes complete current witnesses twice; narrow output
selection does not imply subcorpus source-freshness costs.

The profile keeps the approved predecessor's finite ceilings: 512 records, 1,024
edges, 4,194,304 evidence bytes, 32,768 tokens; history disabled. These are local
proposal values, not new product defaults. No tokenizer was run and no result fit
is claimed. A later approved run must retain budget blocks rather than silently
raise limits or weaken mandatory evidence. Exact absolute paths require profile
review if relocated. No source attribution or stable policy label grants
automatic enrollment of this new digest.

Review artifacts under `target/pcg-175/corpus03/review/`:

- [Complete-union manifest](../../target/pcg-175/corpus03/review/manifest.proposed.json)
- [Isolated profile](../../target/pcg-175/corpus03/review/profile.proposed.json)
- [Source catalogue](../../target/pcg-175/corpus03/review/source-catalog.json)
- [Exact excerpt catalogue](../../target/pcg-175/corpus03/review/new-evidence-spans.json)
- [All proposed revisions](../../target/pcg-175/corpus03/review/proposed-revisions.json)
- [Four proposed requests](../../target/pcg-175/corpus03/review/queries.proposed.json)
- [Exact per-question proof sets](../../target/pcg-175/corpus03/review/question-proof-sets.json)
- [Independent semantic oracle](../../target/pcg-175/corpus03/review/independent-oracle.md)
- [Prior-artifact preservation](../../target/pcg-175/corpus03/review/prior-preservation.json)
- [Static inspection receipt](../../target/pcg-175/corpus03/review/static-inspection.json)

Request artifacts wrap each approved PCG argument object with a local `id` label;
only its `arguments` object is the pcg_query_v1 request. No new request field or
transport adapter is implied. The ignored one-off preparation/inspection scripts
are static review aids, not production ingestion adapters or tracked tests.

## 8. Independent expected-answer oracle and review gate

The separate semantic oracle is specified before any Corpus 03 response exists.
It requires the broad answer to distinguish scoped supported absence, unproved
coverage and insufficient/unsupported semantic evidence. It requires the narrow
answers to preserve these differences:

1. File-local: current authorized sufficient projection or explicit compile and
   atomic replacement, no stale facts surviving an empty update, correct fidelity
   normalization and no failure-as-zero substitution. No workspace-wide inventory
   obligation is invented for a single explicit file.
2. Declaration absence: occurrence mode stays lower-bound; declarations_only
   earns SourceDeclarations completeness only through satisfied proof units,
   supported producer patterns, valid dependencies and publication. Direct
   declaration existence remains independent of derived edges.
3. Negative cycle: `false` means no eligible retained-index witness; only Injects
   and ImportsModule participate, provenance and ambiguity remain visible, no
   implicit compilation occurs, and source_complete remains false.

A future authorized evaluation should check the exact mandatory sets, byte-exact
citations, proof paths, omissions, result state and whole-result token counts for
these four questions. PCG corpus completeness does not upgrade the workspace
query's own lower-bound answer. Existing CBM/IR question bindings remain intact
and may be checked for preservation only when execution is separately authorized.
No future output is prelabelled USEFUL or complete.

Bounded static inspection passed reference resolution, prerequisite acyclicity,
unique role/subject controls, exact UTF-8 excerpt hashes and finite deny signatures
over proposed persisted metadata/excerpts. The four mandatory record sets are
distinct. All 158 prior artifact hashes remained unchanged. These inspections
are not actual PCG schema admission, executable acceptance, arbitrary-secret
certification or performance evidence. The full immutable fingerprints remain
predictions until the approved importer returns them.

**Owner review gate:** accept/reject the exact union manifest/profile digests,
the four declared scopes, source classifications and prerequisite/preservation
mapping. Stop before creating the workspace/DB or importing. No tests, builds,
Clippy, tokenizer runs, PCG queries, operator campaigns, CI, benchmarks, commits,
pushes or publication were performed in this preparation.

## Appendix A. Added source inventory

The following tables give exact immutable identities and half-open byte witnesses.
All sources are at the source commit above. Existing 13 source definitions are
unchanged and remain in the linked full catalogue.

| Source ID | Path | Git blob | Source SHA-256 |
| --- | --- | --- | --- |
| wq-fidelity-decision | docs/architecture/WORKSPACE_QUERY_FIDELITY_AUTO_COMPILE_PROPOSAL.md | 1fb3e5d3e0ae3c2d98fe0fbdf893a7e8d4591b94 | c31598bb3a905326eff9873b1a6514fa7bb008aac76760224274f17372a403a2 |
| wq-authority-doc | docs/architecture/SEMANTIC_AUTHORITY_ARCHITECTURE.md | 37f16e702ea1ca74e4cdd8a0d016a9da5fd62264 | 441146aaa665342b4ae9b595a6f42fa6c812f84f956f4a3985bf7d555e78e05c |
| wq-entities-code | src/mcp/tool_handlers/query/entities.rs | 59d62095121e27768bb6631fde515365a4f1bbb6 | 4ae137fae809845c2d360f5d934d151056c19a89d618d4ba115cd5e631554a03 |
| wq-coverage-code | src/mcp/tool_handlers/query/coverage.rs | 6843d88c38657c0f3116a3f7d3398d8d205118d5 | 56deb91457727a2eb21db7fc435856926577a393bb0c6239c386d65301c81d94 |
| wq-outcome-code | src/mcp/tool_handlers/query/outcome.rs | 71ececa22df70c9e92316d8f732276eb872379c3 | c025be457cd8b21e9311408dd4990f95867206f0b5e3ed2d51cfefb1ce9634a4 |
| wq-index-coverage-code | src/workspace/index/coverage.rs | ffe301f037aa1f87ed2e82a2ac27bfb2ad5b0aab | e95d9f37c7ab949666abc9e10e75a556656d9a6335e8282c47ef1f440d3fddaf |
| wq-proof-code | src/mcp/acquisition/proof.rs | 4b3b28bb995a63007bad8a18dd7f4c040e2a4732 | 660f1f65b0c1ee7634fba039d61db9b692010985ca06805f0a14e43a7ac5d2ea |
| wq-inventory-code | src/mcp/tool_handlers/hydration/filesystem.rs | dd71cf050c6e8ee34bc72365fdfec4edc9a5a3a6 | 28c8066a33acdc07f476904507f06977c1d84269aef267daaf15c6f2c5fc9474 |
| wq-requirement-code | src/mcp/acquisition/requirement.rs | 0894d210e780ad70c3f664a6cf0a7a5a93312a01 | 9afea4ba5e46666453c0c8b81e88c227b018c98fb50dabc1a90176a3cbd39708 |

## Appendix B. Added evidence spans

| Evidence ID | Source ID | Byte range | Quote SHA-256 |
| --- | --- | --- | --- |
| wq-facts-invariant | invariants | [76318,78350) | 2235c39d7dade3f2ee9e35720abadebd36bc385156854959be259171534ab59e |
| wq-discovery-invariant | invariants | [78350,82870) | b4baea4ba7bd6175b69e491c274943eb6bb1d00dadef3b8b27da38b30c3d7be7 |
| wq-scope-invariant | invariants | [89276,92922) | 5c450c41d4106353e83f1a4821e760881c33c889ee9f9bb506c9a00f7eecfb97 |
| wq-file-fidelity-invariant | invariants | [95354,97235) | 6b8fb5c532ea584aeef87e33037bf11487d9e7e36e3b39e35524fcf330b92a95 |
| wq-cycle-invariant | invariants | [97235,99808) | 66db881506ace44d206fac235a575f42da39a3114106107170e04a1a6c1b57a4 |
| wq-history-invariant | invariants | [119268,120060) | dbb696693b18c7792f6bf441c3c126c47d73f603f60ab993f97b9df66b1bf825 |
| wq-completeness-invariant | invariants | [120060,121928) | b86b3f934b8124911325b3bde3d84956d427562ae72aa637741d997aa299c3d1 |
| wq-detail-invariant | invariants | [121928,123506) | e12dc80635382f7d8c16abf42e3ac117251db580e41203aaa0f26dc572e1170b |
| wq-publication-fidelity | invariants | [124005,124444) | df2935bfa962162b54c2a019c44e77c2f7092791364eafb77263e60720e05aa7 |
| wq-direct-declarations-invariant | invariants | [139490,141663) | 15168c44eb031b5a5873e25f0dd325e739c6807d607ecb643d03133ae80a26b6 |
| wq-authority-model | wq-authority-doc | [1361,2003) | df211ede0570b2d6f95103166fa783bc85ba540642afc65d852de39508980d11 |
| wq-authority-validity | wq-authority-doc | [4206,4770) | e26a0e34cad9cbb4fef40ba722cca154936f3d224cfe03b6693ccac052659a52 |
| wq-fidelity-decision-contract | wq-fidelity-decision | [6179,9319) | ea4cbd8c32b27d6a6ff3c9449c083cbd2de5a37defe74f82178b477dd7bf07cd |
| wq-proofs | wq-proof-code | [4202,5836) | 2fce45302356df439c58945bc2eea58018e67168eb9706329a6f3971598f8323 |
| wq-inventory-fingerprints | wq-proof-code | [168,2406) | 5ec73a2fd3e373fd93d86433a0b94183c6f4a1d4a10336c8cbc85942ff9f14c2 |
| wq-source-enumeration | wq-inventory-code | [8465,10354) | 11d0fe6e3d24c035a22f676bf3bad33909c29620db606bdb13ff95191da68ce0 |
| wq-source-reading | wq-inventory-code | [10354,11630) | 6bef5a6b438f4a62626860f94ece83c08825b4d2ea25d9e742c554d8253b8047 |
| wq-coverage-units | wq-requirement-code | [961,4876) | e25fb5e193d1c0dd47684f4b6dbe563a5e826a94666723a3bc663076674c2558 |
| wq-certificate-code | wq-coverage-code | [959,3651) | ec11dae29b759594f891ba4a5d016bc818bb6a616f06616e45e2aadb711893a3 |
| wq-entity-mode-coverage | wq-entities-code | [5657,6825) | 99bef40db144f50fc5bc080dd37660875bc289a3bdf3dccb6c1879611093e408 |
| wq-semantic-fidelity-code | wq-index-coverage-code | [390,1114) | b420e390fb164ca16622da1a2ec0a676aacb7473195647f2f3cdf284d1ed7e66 |
| wq-current-projection-code | wq-index-coverage-code | [6498,7506) | 70c3f468cbfd7be3073efdb585fe8aca31a37feea03216e3e20828109ba46c16 |
| wq-file-preparation-code | wq-entities-code | [7190,13185) | cbfec33778a5a489aca96647eb06e2ae293c1164afa60a982d36a31a943dd884 |
| wq-publication-code | wq-entities-code | [19203,19711) | 868a9f457bc3942a9096d48685f0b3423b7005176f7500cb8050df2bdc365ec1 |
| wq-file-publication-code | wq-entities-code | [19712,21114) | 206193c80ef1c15cdbeea6da3661782b0a5d3b42a65cbe6272fb02282cc46e6d |
| wq-error-envelope-code | wq-outcome-code | [494,2310) | eb66ed7e52c703383b979be61b4f9e5ecdd3c49c18c35b23396110b12ce5331d |
| wq-single-outcome-code | wq-outcome-code | [2310,3045) | b555f92003d0e1502140ea7039875ba5ec56a49833b1990440252ad8bba554f8 |
