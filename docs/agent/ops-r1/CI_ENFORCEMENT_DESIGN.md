# OPS-R1 CI architectural fitness enforcement design

Agent MaxHeadRoom. Design only. Current workflow and verification.md remain unchanged.

## Existing execution architecture

ci.yml runs default-feature check/Clippy, formatting and source guards, then the
consolidated workspace/all-target/allfeature suite once with one test thread.
Registration/listing compiles and lists but does not execute tests. The following
execution reuses that build. Existing verify-ci-membership.py compares target-
qualified listing/outcomes, explicit fitness groups, empirical companions and
allowed ignores. It rejects nonzero suite status or missing/duplicate required work.
The workflow retains evidence and runs security audit separately.

The existing workflow/runner-hash cache key was owner-approved; warm run 415 took
11m12s and the owner accepted that checkpoint. This is an observation, not a new
global time budget or permission to skip obligations. No further cache change is
proposed. Default feature set includes TS/C#/Angular/DotNet; it is not isolation.

## Proposed integration using existing mechanisms

1. Keep cheap existing guards early. Add only approved scoped structural checks
   with an exact prohibited edge/sink model. Parse selected source units once;
   report unresolved dispatch/macros as review, not arbitrary fail-open acceptance.
2. Add approved GARC identities/configurations to the existing manifest/verifier.
   Do not launch a second golden suite or rerun the same fitness tests by filter.
   Each allfeature required identity executes exactly once in consolidated CI.
3. Add supported C#-only and TS-only compile/list coverage, plus Angular/DotNet or
   other modes only where controlling feature applicability requires. Reuse the
   existing focused controls. Do not rerun full alltarget runtime suites in every
   combination; compile distinct modes as needed and run mode-native witnesses
   when compile/list does not protect the behavior. Exact obligation overlap is
   explicit, not called equivalent merely because names match.
4. Add small deterministic independent-axis/work/resource ownership/progress
   witnesses to normal src/tests registration after their work contracts and
   failure counterexamples are approved. Existing retirement/restore tests remain.
5. Leave repeated empirical campaigns in the existing explicit measurement
   workflow with preserved mandatory assertion companions. Heavy resource profiling
   and real-workspace field runs remain operator paths, not hidden correctness loops.
6. Actual document admission remains a separately attributable required publication
   action. Do not claim text guards or Rust tests admit the current document graph.

## Exact rejection criteria and cost accounting

| Check | Rejects | Does not establish | Incremental cost |
| --- | --- | --- | --- |
| Existing list/outcome + approved GARC mapping | Missing registration/outcome, duplicate execution, unexpected ignore/zero selection, wrong feature identity | Assertion quality or arbitrary unregistered requirement | Offline set/identity comparison; no extra test execution |
| Scoped Rust structural checks | Approved forbidden edge/repeated sink in modeled call paths | All macros/dynamic dispatch or global memory bounds | Selected source parse once; no per-candidate source scan |
| Supported feature compile/list/native controls | Wrong cfg/import boundary, vanished mode-native tests | All supported runtime behaviors in that mode | Additional mode compilation; cached artifacts where valid, no false zero-cost claim |
| Deterministic metamorphic witnesses | Approved work relationship, one-shot failure/lifecycle/progress counterexample | Empirical speedup, allocator leak or enterprise scale | Small cells, exact counters; no paired timing campaign |
| Actual admission | Invalid/unapproved source-owned graph candidate | Runtime semantic correctness or independent certification | Existing document workflow; selected relevant closure, no new service |

The verifier should report AFF/GARC IDs with exact failing identity and retained
evidence. Existing test membership integrity remains even when an AFF group fails.
No blanket ignore/allow-failure/exception, missing artifact warning promoted to
acceptance, or silent deletion of a required configuration is permitted.

## Owner decision and validation

Owner approves the feature configurations, mandatory GARC corpus and structural
scope before workflow/authority changes. Validate verifier changes with existing
adversarial evidence fixtures, not broad suite replay. Compare the resulting CI
membership and observed phase costs to run 415; preserve every existing obligation.
No fixed global duration cap is proposed. Repeated Cargo, fixtures and retained
artifacts are explicitly accounted; native feature builds are not treated as free.
