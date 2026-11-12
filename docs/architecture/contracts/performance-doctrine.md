# Performance doctrine source authority

Agent: **Agent MaxHeadRoom**. B5, 2026-11-12.

The controlling operating policy is `.clinerules/engineering.md`. Its tracked
portable projection is [AGENTS.md](../../../AGENTS.md), sections AF-PERF-001 and
AF-PERF-002. This document makes those existing, assessment-reserved propositions
addressable; it neither relocates nor duplicates their normative prose. The
complete unchanged tracked policy is the declared artifact for both nodes.
The ignored local policy is not copied into the graph. The two published sections
were compared exactly with their governing local counterparts during B5.

PERF-004-DOCTRINE denotes AF-PERF-002's normative four-criterion optimization
decision rule, including construction/maintenance costs and separate discovery
coverage. PERF-005-BOUNDED denotes AF-PERF-001's implementation constraint and
deterministic verification obligation for modified semantic operations. Their
categories follow those propositions, not the identifiers. IMPLEMENTED describes
operative policy, not measured system compliance or a newly authenticated freeze.

Correctness remains governed by the existing canonical, source-qualified semantic,
currentness and publication contracts. The complete-alternative tokenizer selector
has its own [token-economics owner](token-economics.md). Fewer counted operations,
smaller payloads and warm caches do not independently satisfy the policy.
The [verification owner](../../agent/verification.md) retains its actual tracked
gate, scope limits and distinction between tracked tests and operator evidence.

No new optimization, cache, authority, resource ceiling or approval system follows
from these declarations. Historical owner dispositions remain attributed in their
own records. Policy compliance is not current performance certification.

```architecture-graph
node: PERF-005-BOUNDED
kind: INVARIANT
status: IMPLEMENTED
title: AF-PERF-001 bounded semantic operation work
track: REPOSITORY
artifact: AGENTS.md
preserves:
  - AUTH-001
  - SEMDEP-001
  - PROJ-001
consumes_authority:
  - AUTH.FINAL_VERIFICATION
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```

```architecture-graph
node: PERF-004-DOCTRINE
kind: AUTHORITY
status: IMPLEMENTED
title: AF-PERF-002 exhaustive discovery and selective remediation
track: REPOSITORY
artifact: AGENTS.md
preserves:
  - PERF-005-BOUNDED
  - AUTH.CANONICAL_IR
consumes_authority:
  - AUTH.TOKEN_ECONOMICS
note: Source-attributed bounded subject; historical measurements and execution standing are not current-executable certification.
```
