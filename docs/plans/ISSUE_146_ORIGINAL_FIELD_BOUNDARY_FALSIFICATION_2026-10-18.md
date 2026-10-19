# Issue #146 Original-Field Boundary Falsification

**Date:** 2026-10-18  
**Investigator:** Agent MaxHeadRoom  
**Repository:** `codeliftsleep2/Clean-CTX`  
**Authoritative branch:** `(fix)Performance-Regression-Issues`  
**Scope:** Original-field causal localization only. No production repair.

## 1. Executive verdict

The original-field record does not contain enough evidence to identify the
first unequal authority boundary for the two `ContextA` owners.

The strongest field facts are:

- the workspace used a primary C# root (Root A) and an additional C# root
  (Root B), alongside two other configured roots;
- native text search found a `public class ContextA : DbContext, IContextA`
  header in both C# roots;
- the public unfiltered `find_entities(name=ContextA)` response exposed both
  source declarations through builtin identities, but exposed a durable
  `dotnet/DbContext/ContextA` occurrence only for Root B; and
- the later synthetic two-root regression passed with equivalent, explicitly
  relationship-producing class bodies.

Those facts do not supply either actual field class body. In particular, they
do not establish whether both isolated class captures contained a supported
`DbSet<T>` member. Under the current durable producer, the shared class header
can render an EF marker while producing no durable DbContext endpoint. The
endpoint exists only when at least one semantic edge is emitted.

The field record also contains no per-owner discovery/admission receipt,
source/capture hash, semantic-generation identity, coverage-currentness
decision, producer invocation/output, artifact, publication ticket, or direct
WorkspaceIndex snapshot. Selecting source evidence, acquisition, currentness,
producer execution, or publication as the cause would therefore be invented.

The investigation stops at the first point where the required comparison
cannot be made: privacy-preserving fingerprints and producer observations for
the two canonical owners and isolated class captures were not retained in the
issue or repository artifacts. The proprietary source files themselves are not
required and must not be disclosed.

No causal production boundary is proven and no production repair is justified.

## 2. Prior synthetic falsification result

The user ran the tracked regression:

```powershell
cargo test --all-features issue_146_equivalent_cross_root_dbcontexts_keep_independent_occurrences -- --nocapture
```

and reported **PASS**. Agent MaxHeadRoom did not run it.

The control used two distinct roots with equivalent `ContextA.cs` bodies, an
EF import, and a supported `DbSet<EntityA>` property. It established that the
normal production path can retain two file-distinct
`dotnet/DbContext/ContextA` occurrences across root order, provider-ready versus
provider-unavailable acquisition, and cold versus warm state.

The PASS falsifies a general defect caused solely by root placement,
requested/additional-root reversal, provider state, same-name occurrence
identity, or ordinary public projection. It does not establish that the field
inputs or lifecycle facts matched the control.

## 3. Actual field-root evidence

The available original-field evidence comes from GitHub Issues #144 and #146
and the earlier static investigation.

| Field fact | Authority | Status |
|---|---|---|
| Root A was the requested primary C# repository | Issue #144 report | Observed |
| Root B was an additional configured C# repository | Issue #144 report | Observed |
| Root C was TypeScript/Angular and Root D was a small C# proxy | Issue #144 follow-up | Observed, not shown relevant to #146 |
| Both roots contained text matching `public class ContextA : DbContext, IContextA` | Native grep reported in #144 | Observed textual header only |
| Root A had builtin Class/Method occurrences | Public response summarized in #144 | Observed |
| Root B had builtin occurrences and one durable `dotnet/DbContext` occurrence | Public response summarized in #144 | Observed |
| Actual canonical `ContextA.cs` path in either root | No retained evidence | UNKNOWN — OBSERVATION REQUIRED |
| Complete source bytes for either class | No retained evidence | UNKNOWN — OBSERVATION REQUIRED |
| Actual isolated class capture passed to `analyze_efcore` | No retained evidence | UNKNOWN — OBSERVATION REQUIRED |
| Supported `DbSet<T>` members in either capture | No retained evidence | UNKNOWN — OBSERVATION REQUIRED |
| Per-owner acquisition, compile, artifact, and publication receipts | No retained evidence | UNKNOWN — OBSERVATION REQUIRED |

The issue's phrase “both declarations derive from `DbContext`” is not evidence
that both durable producers had relationship-producing inputs. It establishes
the header substring only.

## 4. Root A / Root B boundary matrix

| Boundary | Root A | Root B | Equal? |
|---|---|---|---|
| Canonical owner/path | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Effective/configured root | Primary requested C# Root A; physical path unavailable | Additional configured C# Root B; physical path unavailable | Different roles; both admitted roots were reported |
| Candidate discovered | UNKNOWN — OBSERVATION REQUIRED for the observed query cycle | UNKNOWN — OBSERVATION REQUIRED for the observed query cycle | UNKNOWN |
| Candidate admitted | Builtin occurrence proves admission/publication at some time, not necessarily the observed cycle | Builtin and dotnet occurrences prove admission/publication at some time, not necessarily the observed cycle | UNKNOWN for same generation |
| Current source hash | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Observed semantic generation | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Isolated class body/capture hash | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Requested fidelity | `find_entities` production path statically requests `LegacyEdit`, normalized to High adequacy | Same query-level requirement | Equal by code; actual receipt unavailable |
| Stored/current fidelity | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Coverage considered current | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Producer-generation fingerprint | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Compile/skip decision | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| `.NET` meta-layer applicable | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| `analyze_efcore` invoked | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Detected `DbSet<T>` count | UNKNOWN — OBSERVATION REQUIRED | At least one relationship-producing edge existed in the current index, but its generation and producer receipt are unknown | UNKNOWN |
| Emitted semantic edges | UNKNOWN — OBSERVATION REQUIRED | At least one Root-B-provenanced edge endpoint survived in the index | Not equal at final observed projection; producer equality unknown |
| Artifact contains `dotnet/DbContext/ContextA` endpoint | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED; final index is not the artifact receipt | UNKNOWN |
| Publication ticket accepted | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN — OBSERVATION REQUIRED | UNKNOWN |
| Projection replacement committed | No current Root-A dotnet endpoint observed; commit history unknown | A Root-B dotnet endpoint was present; exact commit receipt unknown | UNKNOWN at lifecycle boundary |
| WorkspaceIndex contains endpoint for owner | No, according to final public response and deterministic serialization | Yes, according to final public response | No |
| Final `find_entities` occurrence present | No durable Root-A dotnet occurrence | Yes, durable Root-B dotnet occurrence | No |

The final two rows prove an asymmetric stored projection. They do not move the
earliest divergence upstream.

## 5. Source/capture comparison

### Root A

```text
EF marker?          UNKNOWN — OBSERVATION REQUIRED
Detected DbSets?    UNKNOWN — OBSERVATION REQUIRED
Emitted HasEntity?  UNKNOWN — OBSERVATION REQUIRED
```

### Root B

```text
EF marker?          UNKNOWN — OBSERVATION REQUIRED
Detected DbSets?    UNKNOWN — OBSERVATION REQUIRED
Emitted HasEntity?  A published edge had the Root B DbContext endpoint, but the
                    producer invocation and detected member list were not retained.
```

The native grep established only a shared header. It did not capture the exact
parser-owned class span supplied to the meta-layer. It therefore cannot answer
whether either isolated class included a textual form supported by the current
`DbSet<T>` detector.

This is the earliest unavailable authority-bearing comparison. Investigation
cannot lawfully skip it and attribute the difference to a later boundary.

## 6. Acquisition/admission comparison

Builtin occurrences show that both physical declarations had been admitted and
published at some point. They do not establish any of the following for the
query cycle that produced the asymmetric response:

- that both candidates were rediscovered;
- that both current source versions were admitted;
- that both passed candidate selection rather than one being skipped as
  already current;
- that both were compiled at the requested fidelity; or
- that either compilation failed or was superseded.

Accordingly the case cannot be classified as candidate absent, discovered but
not admitted, admitted but considered current, or selected but failed.

## 7. Semantic coverage/currentness comparison

Static code establishes that `SemanticCoverage::has_current_semantic_projection`
compares owner, source hash, sufficient fidelity, and declared semantic
dependencies. The durable .NET meta-layer declares no semantic dependencies.
The artifact identity separately includes producer-generation fingerprints.

No field evidence records Root A's coverage row or selection decision. It is
therefore unknown whether Root A was skipped as current, recompiled, or absent
from the observed acquisition cycle.

The static compatibility concern is real, but it is not causal evidence for
Issue #146.

## 8. `.NET` producer invocation and output

The durable path is statically:

```text
isolated class capture
→ DotNetMetaLayer applicability
→ analyze_efcore(...)
→ literal `: DbContext` recognition
→ supported DbSet<T> detection
→ zero or more HasEntity edges
```

No invocation receipt exists for either actual owner. Root B's final endpoint
proves that some published semantic edge referred to that endpoint; it does not
retain the producer input, output, or generation needed for a side-by-side
comparison.

## 9. Artifact comparison

No actual `SemanticCompilationArtifact` was retained for either field owner.
The following are all unknown for both roots:

- artifact canonical owner;
- artifact source hash;
- artifact fidelity;
- producer-generation fingerprint;
- semantic edge payload; and
- declared semantic dependencies.

Artifact equivalence or divergence cannot be established.

## 10. Publication comparison

No publication ticket, source-hash validation result, generation validation,
supersession result, or replacement receipt was retained.

The final index proves only that Root B had an endpoint and Root A did not at
serialization time. It does not distinguish:

```text
artifact lacked edge
artifact had edge but ticket was rejected
artifact was superseded
replacement committed an empty edge set
older projection survived because no current publication occurred
```

## 11. WorkspaceIndex comparison

The public response is sufficient to infer the final occurrence-map state:
Root B's file-distinct dotnet endpoint existed and Root A's did not.

Static index behavior excludes same-name collapse as the immediate cause.
`WorkspaceIndex::add_edges` registers file-provenanced endpoints before
adjacency deduplication. If equivalent file-distinct edges had both committed,
both occurrences would have survived.

This does not prove that both publications committed equivalent edges.

## 12. Public projection comparison

`find_entities` serializes stored `EntityRef` occurrences and does not relabel
a builtin Class as a DbContext. The asymmetric public response therefore
faithfully reflected the asymmetric index state described above.

There is no evidence of a public filtering or serialization defect. That
boundary is downstream of an already unequal index and is not causal.

## 13. Earliest proven divergence

The earliest *observed* unequal fact is the final WorkspaceIndex occurrence
state: Root B had a durable dotnet endpoint and Root A did not.

The earliest *causal* divergence is not proven. The investigation reaches an
observation gap first at canonical-owner/source-capture comparison, before it
can distinguish acquisition from semantic production or lifecycle behavior.

Stopping here is required. Continuing downstream would merely restate the
known consequence; choosing an upstream cause would manufacture evidence.

## 14. Producer-generation/currentness audit

The code permits the following static question:

> Could a projection produced by an older or incompatible .NET producer
> generation be treated as current when owner, source hash, fidelity, and an
> empty dependency set still match?

At the isolated `SemanticCoverage::has_current_semantic_projection` boundary,
producer generation is not one of the compared fields. Artifact identity does
carry producer-generation fingerprints, and restore/adoption paths have other
compatibility validation. The field record does not reveal which path Root A
took or which generation produced its resident projection.

Required Root A values remain:

```text
coverage owner:                         UNKNOWN — OBSERVATION REQUIRED
coverage source hash:                   UNKNOWN — OBSERVATION REQUIRED
coverage fidelity:                      UNKNOWN — OBSERVATION REQUIRED
running producer generation:            UNKNOWN — OBSERVATION REQUIRED
resident artifact producer generation:  UNKNOWN — OBSERVATION REQUIRED
reason currentness succeeded:            UNKNOWN — OBSERVATION REQUIRED
```

Therefore the static possibility must not be declared the #146 cause.

## 15. Provider-state relevance

CBM can affect candidate discovery and acquisition cost. It is not authority
for durable DbContext semantics. The passing control already establishes that
provider-ready and provider-unavailable routes can converge for equivalent
acquired evidence.

The original field response did not retain per-owner discovery provider or
admission receipts. Provider state remains relevant only as an unproven
acquisition variable.

## 16. Relationship to #144

Issue #144 proves historical multi-root acquisition and completeness defects,
but the #146 field record does not prove that Root A was absent, not admitted,
or not processed because of that same defect. Root A's builtin occurrence also
shows some successful publication, without proving its current generation or
semantic payload.

No merge is justified unless new evidence shows that the first divergence was
the same authoritative acquisition failure repaired under #144.

## 17. Smallest causal repair boundary, if proven

No causal repair boundary is proven.

The next diagnostic boundary is per-owner source/capture and acquisition state,
not a production change. Depending on the first unequal observation, a future
repair would remain local to acquisition, semantic currentness compatibility,
the .NET producer, or publication. None is authorized now.

## 18. Required RED regression, if a defect is proven

No new RED regression is justified yet. The existing symmetric control is
GREEN and does not reproduce the field asymmetry. A legitimate RED must encode
the first newly proven unequal lifecycle condition while holding earlier
authority-bearing inputs equal.

Examples include a proven producer-generation mismatch, a proven currentness
skip, or a proven publication rejection. Adding another generalized two-root
fixture before obtaining that fact would repeat an already falsified
hypothesis.

## 19. Required runtime observation, if evidence remains insufficient

The field files are proprietary company IP and must remain local. The smallest
required observation is therefore one side-by-side, same-cycle **sanitized
receipt** generated inside the proprietary workspace. It must contain only:

1. opaque owner labels (`RootA/ContextA`, `RootB/ContextA`) and effective root
   role, never absolute paths;
2. salted or run-local source and isolated-capture hashes, capture byte lengths,
   and an equality boolean, never source or captured text;
3. candidate discovery and admission result;
4. current source hash, requested/stored fidelity, and semantic generation;
5. coverage-current result and the exact reason for compile versus skip;
6. running and resident producer-generation fingerprints;
7. .NET applicability, `analyze_efcore` invocation, EF-marker boolean,
   detected DbSet count, and emitted-edge count;
8. normalized artifact edge identities limited to domain, entity type,
   relation, and opaque owner label; proprietary type/member names must be
   replaced consistently with local aliases if disclosure is not permitted;
9. publication ticket validation and projection-replacement outcome; and
10. owner-scoped WorkspaceIndex occurrence immediately after publication.

Static evidence is insufficient because none of those runtime-owned values is
persisted in the issue or tracked repository artifacts. Existing production
diagnostics do not expose a complete per-owner sanitized receipt. Consequently
no truthful, copy-pasteable field command can yet be specified from the
available record: the repository first needs a narrowly scoped test-only or
operator-only observation surface. That surface must accept the real paths
locally without printing, persisting, or transmitting them.

No Cargo, build, test, binary, MCP, or live command was run or is authorized by
this investigation. Before any runtime execution, prepare the smallest
test-only observation hook or operator harness that emits the sanitized receipt
above, then hand the user its exact local command. Any real paths belong only in
local command arguments or private configuration and must not appear in the
captured output, repository, issue, or chat. Instrumentation must observe only
and must not affect discovery, authority, ordering, or publication.

If preserving the original process state is impossible, generate the sanitized
source/capture fingerprint and structural producer facts from the current
files locally. This cannot reconstruct historical state, but it can establish
whether the current field inputs diverge without exposing company IP.

## 20. Final recommendation

Do not request or copy the Root A and Root B `ContextA.cs` files. Design the
smallest local-only observation hook that compares their source and
parser-owned isolated captures through run-local hashes, lengths, equality
flags, and structural EF counts. Its externally shareable output must contain
only opaque root labels and sanitized semantic facts. If those inputs are equal
in all producer-relevant evidence, use the same hook to capture one same-cycle
per-owner acquisition/currentness/artifact/publication receipt and stop at its
first unequal field.

Do not implement a fix, redesign EF authority, modify WorkspaceIndex, add a
second synthetic cross-root regression, or merge #146 into #144 on the current
record.

**FIELD EVIDENCE INSUFFICIENT — EXACT OBSERVATION REQUIRED**

```text
Production repair authorized: NO
```

```text
Merge with Issue #144: NOT PROVEN
```
