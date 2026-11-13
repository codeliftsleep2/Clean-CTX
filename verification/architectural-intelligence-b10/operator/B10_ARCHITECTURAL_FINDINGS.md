# Source-supported interim findings

Agent: **Agent MaxHeadRoom**. These findings do not certify B10 completion.

## Exact representation and semantic coverage

The captured graph has 218 statements, 80 qualification components, eight
verification-obligation components, six observations, three outcomes, two each
of rationale/scope/constraint/counterexample, and one each of subject,
rejected-alternative and rejection-reason. These total 326 exact components.
Scopes are 73 invariant, 64 authority, 74 evidence, four proposition, two decision
and one finding. Role counts describe authored representation, not applicability.

The 218-row coverage inventory identifies every source/revision/role and evidence
association, but marks semantic adequacy PENDING. It is deliberately not a claim
that every source's actual meaning has been assessed. Missing roles cannot be
classified as absent meaning, missing requirements or defects without source review.

The eight exact obligation-bearing identities are ARCH-001, ARCH-002,
PIPELINE-001, VALID-001, WIRE-001, PERF-001-REPLAY, DOC-206 and
AUTH.ANGULAR_RESOLVER. Their full exact obligation text and interval/content hashes
are recorded in B10_VERIFICATION_OBLIGATION_COVERAGE.csv.

A concrete broader-component witness is ARCH-003: its statement explicitly has
`**Gate** | Rust compiler and cargo test --all-features` (the source formats the
Cargo command with backticks). It has no verification_obligation role. Likewise
AUTH.FINAL_VERIFICATION is a whole-authority statement containing the complete
final gate, with no separately typed obligation. Thus a role-only obligation
search cannot be treated as all applicable verification requirements. The prose
is retrievable deterministically; interpreting it still requires source reading.
Neither case proves the source needs additional components: the question is
whether real engineering acquisition misses required meaning.

## Decisions and rationale

PERF-001-REPLAY independently represents the registered deletion counterexample,
the bounded earlier-Candidate rejection, rationale, outcome and later-footer
regression obligation. Its source explicitly preserves original behavior and
rejects only that substitution, not all replay optimization.

PERF-002-STARTUP has separate DEFER and REJECT outcomes, but no separately typed
rationale or alternative. The DEFER outcome itself explains small typical savings,
unestablished real exposure, retained count-heavy work and added reader costs.
The exact broader statement contains the measured comparison and the count-removal
counterexample. This is actual rationale within existing selectable prose, not
absent rationale. A universal role=rationale query would not independently discover
that decision's rationale as such. Whether that warrants enrichment remains a
question for observed workflow evidence, not a B10 implementation change.

## Context completeness and authority

B9's context receipt for PIPELINE-001 has one member and zero edges, consistent
with its source-owned declaration having no outgoing relationships. That is a
complete declared closure. Its content nevertheless cites validator and later
transformation responsibilities. A proposed change may require separate discovery
of validation, canonical authority, language layers or serialization constraints.
This observation does not infer a missing DEPENDS_ON edge: the applicability and
correct edge kind require source/decision-specific review.

The before-run canonical authority context has eight units/seven edges/eight
witnesses; semantic-model context has eight/seven/eight; performance doctrine has
17/18/17. These are structural completeness receipts. They do not establish
semantic completeness across all architecture or correctness of an agent decision.

## Workflow and operational boundary

Current on-disk AGENTS.md, engineering.md and pcg-authoring.md direct architectural
preflight through pcg_knowledge_v1, with exact acquisition, typed traversal,
qualifications and source fallback. The actual current user-injected AGENTS text
earlier in this conversation still named pcg_documents_v1. That demonstrates
instruction-surface drift in this session; it does not prove the next host loads
stale instructions. Do not rewrite the authoritative instructions to compensate
for an unverified host-loading problem.

The source authoring workflow expressly distinguishes unavailable host tooling
from authorized stdio preflight, and structural closure from semantic judgment.
An isolated fresh CLI host is the smallest currently supported operational check.
Production host implementation, schema/search changes and speculative ontology
expansion remain outside B10.

## Preservation and next boundary

The actual profile admits only `docs` as document roots. B10 reports and drivers
are under `verification/architectural-intelligence-b10`, outside those roots.
No source-owned identity, declaration, relationship, selector, profile or runtime
is modified. Legacy token_budget remains in the unchanged compatible profile;
it is not a B10 policy reintroduction or a retrieval criterion.

Next supported action: run and inspect the isolated actual-host smoke. Keep all
ten natural discovery probes and sixteen independent A/B runs UNRUN until their
separate launch/eligibility conditions are satisfied. No broad decision-quality
claim is justified by these structural findings.
