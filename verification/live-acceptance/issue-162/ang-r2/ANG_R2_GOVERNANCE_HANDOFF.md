# ANG-R2 owner disposition 01: governed candidate handoff

Agent: **Agent MaxHeadRoom**. Candidate prepared; descriptor capture and owner-only
authorization remain pending. No production implementation has begun.

## Concrete review package

- [Final compatibility disposition](ANG_R2_COMPATIBILITY.md#final-disposition-after-owner-disposition-01).
- [Public contract and exact omission/nomination clarifications](ANG_R2_PUBLIC_CONTRACT.md).
- [Representation disposition](ANG_R2_REPRESENTATION_DISPOSITION.md), with the accepted design and existing acquisition mechanics.
- [Exact byte pins](ANG_R2_CANDIDATE_PINS.json), including three source predecessors/candidates and unchanged corpus/protected registries.
- [Owner disposition](ANG_R2_OWNER_DISPOSITION_01.md), retained as owner-supplied provenance, not an exact-digest B11 grant.

The three prepared controlling source candidates are angular-resolver.md,
angular-canonical-relations.md and docs/agent/tooling.md. Review their complete Git
diff alongside the final compatibility amendment table. New clauses are within
existing protected selectors; no corpus/protection membership changes. Resolver
adds the canonical-relations dependency; tooling adds the resolver dependency.
Existing associations, qualifications, historical failures and acceptance scopes
are preserved. Source remains a blocked candidate until actual B11 admission.

## Next operator command: describe only

From C:\Users\MNasty\Desktop\RustContextLayerAI, run:

```powershell
pwsh -NoProfile -File verification/live-acceptance/issue-162/ang-r2/invoke-governance-review.ps1
```

This command is **handed off, not executed**. It checks the exact source/profile/
binary pins and HEAD, invokes existing `pcg documents describe`, and preserves the
full Rust report plus stderr/receipt under a fresh target/ang-r2/governance-review-*
directory. It neither authorizes nor publishes. The expected sole diagnostic is
E_GOVERNANCE_AUTHORIZATION_REQUIRED against the pinned accepted predecessor.
Any other diagnostic is an unresolved candidate problem, not a bypass opportunity.

Return the complete descriptor.json (not only its digest). Review actual workspace,
catalog scope, predecessor_generation, accepted membership/protection/state digests,
proposed_state, proposed_state_digest and previously_protected_candidate_view.
Compare the full state/associations and selected-component deltas to the accepted
predecessor acquisition and exact source diff; expected affected authorities are
only resolver, canonical relations and tooling. Rust decides the descriptor bytes;
static pins and candidate review do not manufacture a candidate generation/digest.

## Owner-only authorization and subsequent admission

After the complete descriptor has been reviewed, only the owner explicitly submits
the established command below, replacing the placeholder with that reviewed digest:

```powershell
& 'C:\Users\MNasty\Desktop\RustContextLayerAI\target\debug\clean-ctx.exe' pcg documents authorize --profile 'C:\Users\MNasty\Desktop\RustContextLayerAI\.clean-ctx\pcg-documents\profile.json' --digest '<exact-reviewed-Rust-descriptor-digest>'
```

This is the documented owner workflow, not a request to authorize an unknown digest.
Agent MaxHeadRoom never invokes it. Contract disposition 01 is not this grant.
Authorization records a grant only; it does not admit the candidate. Afterward a
separately permitted normal knowledge reconciliation must recompute/admit the exact
candidate. Verify current generation, changed authority revisions and both new typed
dependencies, preserved memberships/protection/history and complete affected context.
If source or predecessor drifts, return for the fresh descriptor; never reuse a grant.
Do not begin Phase B until these steps establish admission. No runtime/test/build
permission or broad gate is inferred. Full repository gates remain CI-owned.

## Preflight and pending reconciliation

Reuse the same task's complete accepted context at generation
4f410b2f2f816ce7dc7a4ca74ad04009d19ee0bb2631ed8a36a9ae7103ff57c3.
The same full pin is in ANG_R2_CANDIDATE_PINS.json; source receipt pins
and revision chains remain in ANG_R2_VERIFICATION.md. Only the three deliberate
authority candidates diverge; their protected deltas require the procedure above.
Host pcg_knowledge_v1 remains unavailable. No fresh runtime was started after the
no-execution disposition; retained accepted authority and pending candidate are
explicitly distinct. No knowledge/host/CBM repair is included.
