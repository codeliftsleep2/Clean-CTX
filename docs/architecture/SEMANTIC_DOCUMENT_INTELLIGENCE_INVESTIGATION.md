# Semantic Document Intelligence for Markdown Workflows

**Status:** Investigation and phased roadmap; no production contract is
authorized by this document
**Recorded:** 2026-09-29
**Scope:** General Markdown discovery, reading, writing, editing, and
verification across authorized workspaces
**Initial laboratory:** Clean-CTX's own Markdown corpus, used as test material
rather than as the product boundary

## 1. Executive position

Clean-CTX should investigate treating Markdown engineering artifacts as a
first-class semantic input. The target is not merely better access to
Clean-CTX documentation. The target is lower-cost and safer Markdown work in
arbitrary repositories whose live engineering processes depend heavily on
architecture documents, plans, requirements, decisions, investigations,
runbooks, handoffs, changelogs, policies, and similar artifacts.

The central question is:

> Can Clean-CTX transform large Markdown documents into compact, structured,
> task-relevant context before an LLM reasons over them, and use that same
> structure to make document generation and editing safer?

The recommended architectural direction is a sibling document compilation
path:

```text
Repository
    |
    +-- source files ------------------------+
    |      tree-sitter -> CompiledIR         |
    |                                        +--> context selection
    +-- Markdown files ----------------------+         |
           Markdown parser -> DocumentIR               v
                    |                            model-facing context
                    +--> advisory semantics             |
                                                       v
                                                proposed mutation
                                                       |
                                                       v
                                             structural/semantic checks
```

`DocumentIR` should not be represented as source-code `CoreOp` instructions.
The two representations may share lifecycle patterns and converge at context
selection, but they describe different semantic domains.

## 2. Product scope

The production capability is intended to help whenever an authorized
workspace uses Markdown, including:

- architecture and design documents;
- investigations and implementation plans;
- requirements and specifications;
- ADRs and other decision records;
- repository instructions and agent policies;
- migration and operational runbooks;
- handoff, status, and validation reports;
- changelogs and release records;
- READMEs and API documentation;
- security and performance constraints; and
- unfamiliar user-created Markdown whose vocabulary was not present in the
  Clean-CTX corpus.

The complete workflow is:

```text
DISCOVER -> READ -> WRITE -> EDIT -> VERIFY
```

These operations share document structure but do not necessarily share
ranking, mutation, or verification mechanisms.

### 2.1 Read

Find and compile only the sections needed for a task while retaining enough
ancestry, provenance, and related context to interpret them correctly.

### 2.2 Write

Retrieve applicable constraints, decisions, templates, related artifacts, and
repository conventions before generating a new document. Validate the result
after generation without automatically rewriting it.

### 2.3 Edit

Resolve precise document units, preserve unrelated bytes, revalidate the full
document after a splice, and detect suspicious loss or reversal of protected
concepts.

### 2.4 Cross-document work

Combine relevant evidence from several documents. A real task may require a
decision from one file, a constraint from another, and current validation
evidence from a third.

## 3. Current limitation

Clean-CTX's structural context path currently supports source files. The
documented fallback for non-code files is a raw file read. Large Markdown files
therefore reach the model before deterministic structure or relevance can
reduce them.

The repository already provides useful implementation patterns, but its
canonical types are source-specific:

- `CompiledIR` is an ordered stream of source-code operations.
- `HierarchicalIR` models classes, interfaces, methods, imports, and calls.
- `UnitTable` resolves source declarations and method bodies.
- `apply_edit` validates results with the selected source-language grammar.
- persistence aligns canonical source IR, source hashes, semantic edges, and
  fidelity.

Markdown headings must not be disguised as classes, and sections must not be
disguised as methods. Reusing those identities would create an attractive but
incorrect abstraction.

## 4. Authority model

The architecture must preserve three distinct forms of evidence.

### 4.1 Authoritative structural facts

These come from deterministic parsing and validation:

- source bytes and source hash;
- node and section byte spans;
- heading levels, literal titles, and hierarchy;
- ordered blocks and list structure;
- explicit links and destinations;
- fenced-code boundaries and declared languages;
- explicit front matter and recognized explicit metadata; and
- exact before/after structural differences.

### 4.2 Explicit repository authority

A repository may deliberately establish stronger meaning through an approved
contract, for example:

- front-matter status or authority fields;
- an explicit supersession link;
- protected-section metadata;
- repository configuration identifying governing documents; or
- a recognized instruction-file contract.

No such syntax is authorized merely by being proposed here. Each externally
observable authority mechanism requires a separate decision.

### 4.3 Advisory semantic interpretation

These signals help retrieval and review but do not create facts:

- embeddings and similarity scores;
- inferred semantic roles;
- inferred status or authority;
- inferred supersession or staleness;
- classifier confidence;
- semantic drift scores; and
- LLM interpretation.

A classifier saying that a section resembles an architectural invariant does
not establish an architectural invariant.

### 4.4 Governing instructions

`AGENTS.md` and comparable instruction or policy files require a stricter
boundary. Document intelligence may help locate, navigate, or explain them,
but advisory retrieval must never decide which governing instructions are
injected into an agent. The host's policy and instruction-loading boundary
remains authoritative.

## 5. Candidate architecture

The smallest coherent architecture has six responsibilities:

```text
Markdown source
    |
    v
Document parser
    |  authoritative syntax tree and byte spans
    v
DocumentIR builder
    |  authoritative section ownership and addresses
    +---------------------+
    |                     |
    v                     v
Deterministic index   Advisory enrichment
links/terms/status    embeddings/roles/drift
    |                     |
    +----------+----------+
               v
        task-specific selector
               |
               v
         document renderer
               |
               v
              LLM
```

Mutation adds a separate path:

```text
tracked DocumentIR + current source
               |
               v
         resolve exact unit
               |
               v
      optimistic concurrency check
               |
               v
          byte-exact splice
               |
               v
       full-document deterministic parse
               |
               v
 structural diff + preservation checks
               |
               v
         atomic publication
```

## 6. Minimum `DocumentIR`

The first canonical form should contain structural facts only.

```text
DocumentIR
|- canonical file identity
|- source hash
|- parser/dialect identity
|- document byte span
|- front matter
|- ordered blocks
|- ordered sections
|- explicit links
|- reference definitions
|- fenced code blocks
`- diagnostics
```

### 6.1 Document record

The top-level record should minimally carry:

- canonical path or an authorized path identity;
- source hash;
- parser and dialect version;
- source length and line-ending information;
- optional front-matter span and parsed representation;
- root-level block identifiers;
- root-level section identifiers;
- parse diagnostics; and
- a canonical structural version independent of any model-facing schema.

### 6.2 Section record

Each section should minimally carry:

- emitted structural ID;
- literal heading text;
- normalized heading text for search only;
- heading level;
- heading span;
- content span;
- full section span;
- parent section ID;
- ordered child section IDs;
- ordered directly owned block IDs; and
- same-title occurrence ordinal within its structural scope.

Heading titles are not unique. A title-only address such as
`Document.Lifecycle.Recovery` is unsafe when headings repeat. The emitted ID
must distinguish structural path and occurrence, while mutation must also be
pinned to the tracked source identity.

### 6.3 Block record

Block records should preserve:

- block kind;
- exact byte span;
- owning section;
- source order;
- essential typed fields, such as a code fence's info string or a link's
  destination; and
- child ownership where the Markdown AST nests blocks.

The canonical IR should reference exact source spans rather than copying all
prose by default. Renderers can slice text from the verified source.

### 6.4 Advisory annotations

Semantic annotations should be stored separately and versioned by their
producer:

```text
SectionAnnotations
|- section ID and source hash
|- candidate roles with confidence
|- lexical score components
|- embedding model/version
|- vector or vector reference
|- inferred status/authority
`- semantic drift evidence
```

Annotations become stale whenever their source hash or producer version no
longer matches.

## 7. Section hierarchy and ownership

Heading hierarchy is deterministic but Markdown block nesting introduces
important edge cases:

- skipped heading levels;
- duplicate headings;
- setext headings;
- headings inside blockquotes or list items;
- raw HTML containing heading-like text;
- headings inside code fences;
- documents with preamble content before the first heading;
- generated tables of contents;
- front matter that resembles a thematic break; and
- malformed or unclosed constructs.

The parser AST determines whether text is a heading. The `DocumentIR` builder
then establishes section ownership using heading level and source order.
Regular expressions must not be the structural authority.

Preamble content should belong to an explicit root/preamble unit, not to the
first heading implicitly.

## 8. Library candidates

### 8.1 Markdown parser

The leading candidate is
[`markdown-rs`](https://github.com/wooorm/markdown-rs). It provides an owned
MDAST, CommonMark/GFM support, extensions, and byte-offset positions.

Two comparison candidates should be included in the parser spike:

- [`markdown-syntax`](https://docs.rs/markdown-syntax/latest/markdown_syntax/),
  which provides an owned AST, byte spans, diagnostics, validation, and
  serialization but is currently young and does not promise byte-preserving
  serialization; and
- [`comrak`](https://github.com/kivikakk/comrak), a mature CommonMark/GFM
  implementation with a mutable arena-backed AST.

The established `tree-sitter-markdown` grammar should not be the correctness
authority. Its own project describes accuracy limitations and recommends it
primarily for editor-oriented syntactic information.

The parser decision must follow the Phase 1 conformance evidence. This
document does not lock a dependency.

### 8.2 Lexical retrieval

The first retrieval baseline should be small and deterministic. Options are:

- an internal BM25 implementation over sections;
- the lightweight `bm25` crate; or
- a simpler scored inverted index if that is sufficient for the benchmark.

[`tantivy`](https://github.com/quickwit-oss/tantivy) is a credible later
option for persistent BM25, phrase queries, fielded search, and much larger
corpora. It should not be introduced before measurements demonstrate that the
smaller boundary is insufficient.

### 8.3 Embeddings and reranking

[`fastembed`](https://github.com/Anush008/fastembed-rs) is a candidate for
local dense embeddings, sparse embeddings, and reranking. It uses model files
and ONNX Runtime for many supported models.

It must remain optional during investigation because adopting it raises
separate decisions about:

- model identity and versioning;
- first-use downloads and offline operation;
- cache placement and lifecycle;
- native runtime and binary-size impact;
- CPU/thread policy;
- accelerator support;
- license and distribution review; and
- reproducibility across platforms.

### 8.4 Lightweight classifiers

`linfa-logistic` is suitable for testing one-vs-rest logistic classifiers over
section embeddings. A production runtime might instead store trained weights
and perform the small inference calculation directly, avoiding a permanent ML
training dependency.

### 8.5 Existing Clean-CTX dependencies

Useful existing building blocks include:

- `serde` and `serde_json` for typed IR and wire representations;
- `sha2` for source identity and fingerprints;
- `tiktoken-rs` for measured token economics;
- `rayon` for bounded parallel processing when measurements justify it;
- `ignore` for repository-aware Markdown discovery;
- `rusqlite` for a future approved persistence design; and
- existing tree-sitter grammars for optional analysis of fenced source code.

No vector database is required for the initial corpus sizes. A linear cosine
scan over a few thousand section vectors is an appropriate baseline.

## 9. Retrieval model

Retrieval should combine independent evidence rather than treating vector
similarity as the answer.

Candidate signals include:

- exact phrase and token matches;
- heading matches;
- lexical/BM25 relevance;
- explicit links and backlinks;
- file path and document type;
- section ancestry;
- explicit status and authority metadata;
- task intent;
- semantic similarity;
- advisory role confidence;
- repository-local relevance; and
- freshness or supersession evidence.

The initial ranking formula must remain an experiment. Introducing global
weights, thresholds, candidate limits, or truncation behavior requires an
explicit decision after benchmark evidence exists.

### 9.1 Context assembly

A selected section may be unintelligible without nearby structure. Assembly
should be able to include:

- the document identity and status;
- the section heading path;
- required ancestors;
- directly relevant descendants;
- definitions reached through explicit links;
- preservation constraints; and
- an honest disclosure that other sections were omitted.

The renderer should never imply that a partial selection is the whole
document.

### 9.2 Cross-document references

Explicit Markdown links are deterministic graph edges. Resolved relative paths
must pass the same root and path authorization principles as other Clean-CTX
operations.

Unlinked textual references and inferred relatedness remain advisory. A title
or filename mention must not be silently promoted into an authoritative link.

## 10. Semantic roles

The initial role taxonomy is a research vocabulary, not a closed schema:

- architecture decision;
- architectural invariant;
- requirement;
- implementation detail;
- root cause;
- bug reproduction;
- validation or test result;
- known limitation;
- deferred work;
- non-goal or scope boundary;
- design rationale;
- rejected alternative;
- migration step;
- configuration;
- command;
- example;
- API contract;
- security constraint;
- performance constraint;
- historical context; and
- status.

Roles must be multi-label and allow `unknown`. Heading labels provide weak
training evidence but cannot serve as ground truth.

Clean-CTX's corpus is useful because it contains many repeated headings and
large documents. Training and evaluation must be split by document, not by
random sections, to avoid measuring memorization of templates and neighboring
text.

## 11. Writing workflow

Document generation should use semantic intelligence before and after the LLM,
without attempting to replace language generation.

```text
task
  -> retrieve applicable constraints and related evidence
  -> assemble a generation brief with provenance
  -> LLM generates Markdown
  -> deterministic parse and structural validation
  -> explicit link/metadata checks
  -> advisory consistency review
```

The investigation must distinguish:

- writing a new document;
- adding a section to an existing document;
- updating a generated or templated artifact;
- recording a decision or result in an append-only ledger; and
- modifying a governing instruction document.

These operations have different authority and preservation requirements.

## 12. Editing workflow

The first safe operations should be section-oriented and byte-preserving:

- `replace_section_content`;
- `replace_section`;
- `insert_section_before`;
- `insert_section_after`; and
- `delete_section`.

Names are descriptive only; no public tool contract is approved here.

Each mutation should eventually require:

1. an authorized canonical path;
2. a tracked document baseline;
3. an unambiguous structural unit ID;
4. matching current source identity;
5. exact expected old text where content is replaced or deleted;
6. non-overlapping operations;
7. preservation of the file's line endings;
8. a byte-exact splice that leaves unrelated content untouched;
9. successful full-document parsing after the splice;
10. structural before/after analysis;
11. explicit preservation checks; and
12. atomic filesystem and state publication.

Whole-document AST serialization should not be the normal edit path because
it can reformat unrelated content. It may be useful for an explicitly
requested formatting operation, which is a different contract.

## 13. Semantic preservation and drift

Three levels of checking should remain distinct.

### 13.1 Deterministic preservation

Examples include:

- a protected section was deleted;
- an explicit status field changed;
- a required link disappeared;
- heading ownership changed unexpectedly;
- a code fence became unbalanced;
- an explicitly identified invariant block changed; or
- unrelated bytes changed outside the authorized spans.

These can reject a mutation when the underlying preservation rule is itself
authoritative.

### 13.2 Advisory drift

Embedding movement, role changes, contradiction scores, or classifier output
may flag a suspicious change. They should produce evidence and escalation,
not automatic rejection, unless a separately approved contract establishes a
deterministic threshold and consequence.

### 13.3 LLM review

Ambiguous changes can be escalated with compact before/after context and the
specific preservation concern. The LLM should not reread the whole document
unless the evidence requires it.

## 14. Fenced code blocks

Fenced code is first a Markdown block owned by the document. Its literal
content, declared language, fence form, and source mapping belong to
`DocumentIR`.

Optional source-language analysis may compile a fence on demand using existing
language grammars. Such compilation must:

- retain the document and fence as provenance;
- map virtual offsets back to Markdown source offsets;
- avoid publishing the fence as an independent physical source file;
- avoid polluting `WorkspaceIndex` unless a future contract explicitly
  defines that behavior; and
- degrade safely for unknown or malformed language tags.

## 15. Phased investigation and delivery

Each phase has an evidence gate. Completing a phase does not authorize later
phases automatically.

### Phase 0 - Corpus and benchmark definition

**Goal:** Establish representative work before choosing architecture by
intuition.

Work:

- inventory Markdown shapes in the Clean-CTX corpus;
- select large and small documents, duplicate headings, front matter, tables,
  nested lists, blockquotes, links, HTML, and code fences;
- add synthetic adversarial documents only for uncovered syntax cases;
- define representative read, write, edit, and cross-document tasks;
- hand-label the sections necessary to answer read tasks;
- define full-document baselines;
- define token, latency, accuracy, and safety measurements; and
- specify a document-level train/evaluation split for semantic experiments.

Exit evidence:

- a versioned benchmark manifest;
- a reviewed query/task set;
- expected relevant-section labels; and
- baseline raw token costs.

No production code or public tool changes are required.

### Phase 1 - Parser and `DocumentIR` spike

**Goal:** Prove authoritative structural representation.

Work:

- compare `markdown-rs`, `markdown-syntax`, and Comrak on the benchmark;
- verify UTF-8 byte spans and exact source slicing;
- build heading hierarchy and preamble ownership;
- represent blocks, links, front matter, and code fences;
- define structural diagnostics;
- test duplicate headings and malformed input; and
- measure parse time and memory.

Exit evidence:

- parser decision record;
- minimum `DocumentIR` schema;
- tracked parser/hierarchy regressions;
- exact span round-trip evidence; and
- no dependence on semantic models.

Approval gate:

- adding the selected parser dependency and canonical document schema.

### Phase 2 - Deterministic single-document retrieval

**Goal:** Demonstrate useful token reduction without embeddings.

Work:

- index headings and section text;
- implement lexical/BM25 candidate selection;
- include structural ancestry and provenance;
- render selected sections honestly as partial context;
- compare answers and selected sections with the full-document baseline; and
- measure token reduction and latency.

Exit evidence:

- retrieval recall and precision against hand labels;
- task-answer comparison with full-document context;
- token economics by document and task;
- failure analysis for missed context; and
- evidence for or against a dedicated public read surface.

Approval gate:

- public read tool shape, response contract, and any candidate/budget limits.

### Phase 3 - Workspace discovery and cross-document retrieval

**Goal:** Support real tasks whose evidence spans documents.

Work:

- discover Markdown through authorized, ignore-aware traversal;
- parse explicit links into a document graph;
- resolve links under configured-root boundaries;
- rank sections across files;
- disclose unresolved and external links;
- distinguish explicit links from inferred relatedness; and
- evaluate multi-document tasks.

Exit evidence:

- scoped discovery and path-security regressions;
- cross-document provenance in every result;
- multi-document retrieval measurements; and
- an explicit completeness statement for the indexed evidence.

Approval gate:

- workspace indexing lifecycle, freshness, and completeness semantics.

### Phase 4 - Optional embeddings and hybrid ranking

**Goal:** Determine whether semantic retrieval materially improves the
deterministic baseline.

Work:

- select one or more candidate embedding models;
- record model size, license, latency, memory, and platform behavior;
- compare dense, sparse, and hybrid retrieval;
- test paraphrases and vocabulary mismatch;
- test linear vector scanning before approximate indexing;
- evaluate optional reranking of a small candidate set; and
- measure incremental value over Phase 2 and Phase 3.

Exit evidence:

- retrieval quality delta over the deterministic baseline;
- resource and distribution cost;
- offline and cache behavior;
- confidence/failure analysis; and
- a recommendation to adopt, defer, or reject model-backed retrieval.

Approval gate:

- model/runtime dependency, model acquisition, cache ownership, threading,
  and fallback policy.

### Phase 5 - Semantic role classification

**Goal:** Determine whether inexpensive role inference improves task-specific
selection.

Work:

- normalize weak heading labels;
- create a manually reviewed evaluation set;
- test rules, nearest-neighbor methods, and lightweight classifiers;
- support multiple roles and `unknown`;
- calibrate confidence;
- evaluate differently worded and heading-free sections; and
- test whether roles improve retrieval after controlling for heading terms.

Exit evidence:

- per-role precision, recall, and calibration;
- confusion and abstention analysis;
- measured retrieval improvement; and
- a reduced useful taxonomy rather than an aspirational list.

Approval gate:

- production role vocabulary, model packaging, thresholds, and presentation.

### Phase 6 - Write-context support

**Goal:** Improve generated documents by retrieving constraints and related
evidence before generation.

Work:

- define write intents and required context classes;
- assemble provenance-bearing generation briefs;
- validate generated Markdown structurally;
- check explicit links and metadata;
- compare generated results with and without document intelligence; and
- identify instruction and append-only document special cases.

Exit evidence:

- constraint-retention measurements;
- structural validity results;
- human review of generated-document quality; and
- clear separation between advisory suggestions and authoritative failures.

Approval gate:

- any public write workflow or explicit metadata convention.

### Phase 7 - Section-addressed editing

**Goal:** Make precise Markdown edits without collateral rewriting.

Work:

- establish tracked document baselines;
- implement unambiguous section identity;
- add optimistic concurrency;
- perform byte-preserving section splices;
- reparse the complete result;
- compare structural ownership before and after;
- preserve line endings and untouched bytes; and
- define atomic state publication.

Exit evidence:

- tracked edit regressions covering every operation;
- duplicate-heading and stale-baseline rejection;
- malformed-result rejection;
- untouched-byte equivalence; and
- complete production lifecycle trace for document state.

Approval gate:

- public edit operations, persistence contract, and mutation transaction.

### Phase 8 - Preservation and semantic-drift experiments

**Goal:** Detect dangerous valid-Markdown changes without pretending that
inference is authority.

Work:

- implement deterministic checks for explicit protected facts;
- assemble labeled safe and unsafe edit pairs;
- evaluate embedding drift, contradiction, and role-change signals;
- produce compact escalation evidence;
- measure false positives and false negatives; and
- test whether LLM escalation improves ambiguous decisions.

Exit evidence:

- deterministic rejection coverage;
- advisory detector precision/recall;
- escalation rate and cost;
- examples of undetectable or ambiguous changes; and
- an explicit statement of what remains human/LLM judgment.

Approval gate:

- any protected syntax, semantic threshold, automatic rejection, or required
  escalation policy.

### Phase 9 - Persistence and production integration

**Goal:** Integrate the proven capability without weakening existing source
contracts.

Work:

- decide separate versus generalized document state ownership;
- align source hash, `DocumentIR`, annotations, and model versions;
- define freshness and invalidation;
- define session/workspace lifecycle;
- add public MCP schemas and responses;
- expose honest completeness and provenance;
- update tooling documentation; and
- perform live end-to-end validation on external repositories.

Required lifecycle trace:

```text
Markdown source
  -> parser
  -> DocumentIR
  -> retained owner/index
  -> session/workspace lifecycle
  -> selector/renderer or editor
  -> public MCP response
  -> live agent consumption
```

Exit evidence:

- the complete local verification gate;
- architectural audit;
- production lifecycle evidence;
- live tasks on non-Clean-CTX repositories;
- token and quality measurements; and
- removal of obsolete or duplicate experimental paths.

Approval gate:

- persistence schema, public contracts, default enablement, and release plan.

## 16. Evaluation metrics

No single compression percentage is sufficient.

### 16.1 Read metrics

- relevant-section recall at selected context size;
- irrelevant-section exclusion;
- answer equivalence or task success versus full-document context;
- raw and selected tokens using supported tokenizers;
- fixed envelope cost;
- parse, index, selection, and rendering latency;
- provenance accuracy; and
- rate of required fallback to full document.

### 16.2 Cross-document metrics

- relevant-file and relevant-section recall;
- explicit-link resolution accuracy;
- unresolved-link honesty;
- stale-evidence rate;
- task success when evidence spans files; and
- incremental cost of workspace discovery.

### 16.3 Write metrics

- required-constraint retention;
- contradiction rate;
- structural validity;
- link validity;
- unnecessary context cost; and
- human/LLM review preference.

### 16.4 Edit metrics

- correct target resolution;
- stale and ambiguous target rejection;
- untouched-byte preservation;
- resulting structural validity;
- hierarchy preservation;
- explicit-constraint preservation;
- semantic-drift detector precision/recall; and
- escalation cost.

## 17. Test strategy

Tracked tests must live under `src/tests/**` when implementation begins.
Scratch corpora or live harnesses may aid observation but cannot satisfy a
regression requirement.

The test layers should be:

1. parser adapter tests;
2. `DocumentIR` ownership and identity tests;
3. retrieval/ranking tests;
4. cross-document scope and path-security tests;
5. renderer and public-envelope tests;
6. edit transaction tests;
7. persistence and recovery tests; and
8. registered-tool end-to-end tests.

The Clean-CTX documentation corpus should not be embedded wholesale into unit
tests. Select small representative fixtures and keep the larger corpus in a
separate benchmark manifest where appropriate.

## 18. Risks and failure modes

### 18.1 Semantic omission

Retrieval can omit the one qualification that changes a section's meaning.
Mitigations include ancestry, linked context, high recall targets, omission
disclosure, and full-document fallback.

### 18.2 False authority

Classification or similarity may make historical or rejected material look
authoritative. Explicit metadata and provenance must outrank inference, while
inferred authority remains visibly advisory.

### 18.3 Identity instability

Heading renames, duplicate headings, and section moves can invalidate
addresses. Mutations must bind structural identity to current source identity
rather than promise permanent heading IDs.

### 18.4 Model and runtime cost

An embedding model can erase token savings through startup time, memory,
downloads, or deployment complexity. Phase 4 must prove net value.

### 18.5 Parser dialect mismatch

Repositories use extensions beyond CommonMark/GFM. Unsupported constructs must
remain safely represented as source spans with diagnostics rather than being
silently misclassified.

### 18.6 Stale indexes

Document structure, embeddings, and semantic roles may become inconsistent
after external edits. Source hash and producer version must qualify every
reusable projection.

### 18.7 Instruction truncation

Applying ordinary relevance filtering to governing instructions could remove
applicable policy. Instruction injection remains outside advisory selection.

### 18.8 Premature unification

Generalizing source and document state before both contracts are understood
could weaken canonical source semantics. Start with sibling typed boundaries
and extract shared abstractions only after real duplication is demonstrated.

## 19. Explicit non-goals for the initial investigation

Do not begin by:

- building a general-purpose RAG framework;
- adding a vector database;
- making embeddings mandatory;
- treating inferred roles as authoritative;
- sending every section through an LLM classifier;
- automatically rewriting documents;
- reserializing entire files for small edits;
- replacing the source-code semantic substrate;
- publishing fenced code as independent source files;
- inventing global token caps or thresholds without evidence and approval; or
- optimizing only for Clean-CTX's documentation vocabulary.

## 20. Architectural decisions requiring approval

The investigation should produce recommendations, but implementation must stop
for explicit approval before deciding any of the following:

1. the canonical parser dependency and `DocumentIR` schema;
2. a new public tool versus broadening an existing tool;
3. response envelopes and partial-result semantics;
4. workspace discovery completeness and indexing lifecycle;
5. global candidate limits, token budgets, thresholds, and fallbacks;
6. embedding/runtime/model acquisition and cache policy;
7. authoritative front matter or protected-section syntax;
8. persistent schema and state ownership;
9. public edit operations and atomic mutation behavior;
10. semantic-drift rejection or mandatory escalation; and
11. default enablement and external behavior.

## 21. Open investigation questions

- Which parser preserves authoritative spans across the required dialects?
- What is the smallest stable `DocumentIR` that supports both retrieval and
  editing?
- How should section IDs be presented without implying permanence?
- How much does deterministic lexical retrieval save before embeddings?
- Which task intents require different context assembly?
- How should explicit document authority be configured across arbitrary
  repositories?
- How should superseded and historical documents be represented?
- What completeness can workspace-wide Markdown discovery honestly claim?
- When should a whole document be returned instead of a section selection?
- Do embeddings materially improve recall on real engineering terminology?
- Which semantic roles improve decisions rather than merely describe text?
- Can advisory drift detection reach useful precision without excessive
  escalation?
- Which fenced-code analyses are valuable enough to justify cross-language
  compilation?
- Which state and transaction patterns can be shared with source workflows
  without weakening either domain?

## 22. Definition of success

The investigation succeeds when it demonstrates, on both the controlled
Clean-CTX corpus and unrelated live repositories, that Clean-CTX can:

1. parse Markdown into exact, trustworthy structural units;
2. retrieve the context needed for real tasks with materially fewer tokens;
3. disclose provenance, omissions, and uncertainty honestly;
4. support cross-document work under workspace security boundaries;
5. improve writing by supplying the right constraints before generation;
6. edit precise units without modifying unrelated bytes;
7. reject deterministic structural and concurrency failures;
8. identify suspicious semantic drift without promoting inference to
   authority; and
9. complete the full production lifecycle from source to live agent use.

Until those conditions are met, semantic document intelligence remains an
investigation rather than a completed Clean-CTX capability.
