# Issue #162: bootstrap caller discovery after refresh

Agent: Agent MaxHeadRoom
Status: owner APPROVED; query-bounded discovery implementation and controls authored, UNRUN.

## Generation-68 integrated binary acceptance inspected PASS

Inspected bootstrap68-binary-20261027-152030: bootstrap-references PASS 201 checks,
ApplicationConfig preservation PASS 304, imported-owner preservation PASS 332.
Each receipt has zero failed checks/errors, three FINISHED sessions and fewer
than 500 checks. All share the same rebuilt binary and recorded input hash.
The all-feature build Finished without warnings/errors (22.05s); syntax preflight
passed 33 JavaScript files. Full report paths, hashes and binary identity:
target/issue-162/bootstrap68-binary-receipt.json.

This supersedes the previous bootstrap binary failure for these selected suites.
The repaired query-bounded discovery path now passes the refresh/caller-deletion
stage and the later stale-restore/package/config stages. No production or harness
edit accompanies evidence inspection. This is bounded binary/stdio acceptance,
not a full repository gate or Claude field result. AQ14/P3/P1/W04/#162 remain open.

## Query-bounded discovery preservation/lint inspected GREEN

Inspected bootstrap68-exit-20261027-150605: filesystem discovery cache 9 passed
(8.80s), config provider 10 passed (89.67s), route provider 11 passed (103.76s),
zero failed/ignored. The corrected native-package fixture now passes its original
inbound-result and scan-count assertions. Format/fmt logs are empty; Clippy
Finished in 26.02s without warnings/errors for all targets/features, -D warnings.
Operator reports GREEN. Prior eight bootstrap controls and completed refresh/
hydration-cache passes remain valid; the historical invalid-fixture failure is
preserved. Summary/hash receipt:
target/issue-162/bootstrap68-discovery-preservation-receipt.json.

No production edit accompanies this inspection. The post-repair deterministic
checkpoint is GREEN. The existing integrated binary checkpoint may now resume
with one rebuild, bootstrap-references and its config/imported-owner preservation
suites. The previous binary failure is not superseded until new receipts pass.
Final repository/Claude field and broader issue gates remain open.

## Discovery preservation fixture correction, UNRUN

Inspected bootstrap68-exit-20261027-145541: eight integrated bootstrap controls
passed (162.43s). The filesystem discovery-cache filter records eight passes and
one failure: external_refresh_expires_zero_result_inbound_discovery expected a
native Angular constructor edge from a fixture without @angular/core package
proof. Generic inbound discovery did run again; no bootstrap query was involved.
The test now installs the existing canonical Angular fixture before state/source
capture. The original count and two-scan assertions are unchanged. Pre-correction
test/log bytes are under target/issue-162/bootstrap68-filesystem-control-correction/.
No production change accompanies this fixture correction; corrected test UNRUN.

The handoff supports -ResumeAfterFilesystemFailure to rerun the corrected filter,
then config/route preservation and final lint without repeating completed bootstrap,
refresh and hydration-cache batches. Historical failed receipt remains preserved;
no later preservation, lint or binary acceptance is inferred.

## Approved query-bounded discovery implementation authored, UNRUN

The supplied owner approval authorizes conservative TypeScript nomination for the
exact Angular ApplicationBootstrap/bootstrapApplication forward query. The valid
regression and its registration were isolated as raw bytes under
target/issue-162/bootstrap68-refresh-approved-isolation/ before production edits,
then restored byte-identically. HEAD and the normal index were untouched.

A private bootstrap hydration function reuses configured_roots and source_inventory,
selects TypeScript owners without a spelling predicate, and feeds existing High
capture/publication. Only the exact forward identity invokes it. Root exclusions
and source validation remain in existing inventory/publication code; withinPath
is not passed to acquisition. Generic literal discovery, native proof and retained
admission remain unchanged. No new cache, scanner, resolver or lifecycle exists.
Inventory completion does not supply an acquisition satisfaction/completeness proof.
No retained meaning or durable encoding changed; semantic generation remains 68.

One additional tracked control batches cold and post-refresh direct/alias/barrel/
namespace routes, independent caller deletion, generic-query non-expansion and
lower-bound coverage. The original refresh RED is unchanged. Eight integrated
controls plus refresh/discovery-cache/config/route preservation, fmt and Clippy
are now selected by run-bootstrap68-exit-controls.ps1. All post-repair commands
are UNRUN; no agent Cargo, formatter, Node or runtime execution occurred. Binary
acceptance is deferred until this deterministic checkpoint passes.

## Located divergence

The corrected tracked refresh regression passes exact native invocation/admission
prerequisites, removes one caller, refreshes, then fails its first forward query:
zero rows instead of the unchanged barrel sibling's root/config pair. Discovery
reports two compiled candidates and identity_not_indexed, lower_bound. The binary
receipt independently exhibits the same loss after refresh.

The shared discovery mechanism selects source snapshots containing the written
query name (mcp::acquisition::clean::type_ref_candidate_owners). That predicate is
explicitly advisory and not certified for TypeScript. The sibling imports start
as launch, so bootstrapApplication does not occur in its source. Refresh properly
retracts prior retained owners; execution's retained-owner revalidation cannot
recover a retracted caller. Native resolver/interpretation need no replacement.
Archive: target/issue-162/bootstrap68-refresh-discovery-red-evidence/receipt.json.

## Decision required

Approve a bounded candidate acquisition policy for the existing exact Angular
ApplicationBootstrap forward query: discover eligible TypeScript invocation-owner
candidates from the existing configured-root source inventory, independently of
the public display name. Native demanded call/API/argument proof must still pass
before any relation is published. Candidate syntax or inventory membership grants
no role, endpoint authority, completeness certificate or new public meaning.

Recommended first implementation: conservatively select supported TypeScript
source owners from that existing inventory for this exact bootstrap query and
feed them through existing High capture/currentness/publication. Preserve root
exclusions, answers-only withinPath, source witnesses and lower-bound semantics.
No new resolver, registry, cache, budget, retry, lifecycle or global query default.
Cost: potentially more TypeScript captures for this bounded query. A later narrower
neutral call-syntax predicate requires preservation evidence before it can exclude
owners. Do not silently change the global name-bearing predicate.

Alternatives: implement resolver-proved import/export reverse closure (more shared
acquisition scope); require explicit recapture of every alias caller (leaves the
observed independent-retirement contract unmet); retain stale owners through
refresh (violates currentness; reject). Adding textual terminal-name recovery to
semantic admission is not an alternative.

Affected boundary: bootstrap forward-query candidate discovery and its cost;
existing native proof, public fields/selectors, generic discovery defaults and
all other query types retain their contracts. Approval is needed under AGENTS.md's
Architectural Approval Gate because prior approval authorized semantic occurrence
proof and lifecycle reuse, but did not settle a broader candidate acquisition
policy. The rule says: "Do not make any architectural decision with global,
cross-cutting, repository-wide, or externally observable effects unless that
decision was explicitly authorized by the task prompt or approved during the
current work." This is a conservative interpretation of that discovery boundary.

After approval, isolate the unchanged valid regression before production repair,
implement the bounded policy, restore exact tests and batch deterministic refresh/
bootstrap controls plus affected discovery preservation and lint. Only then resume
the existing major binary checkpoint; no immediate rebuild/Node run is requested.
