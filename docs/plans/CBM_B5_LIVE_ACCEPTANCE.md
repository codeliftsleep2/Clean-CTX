# B5 current pinned CBM live acceptance

## Artifact and authority

Branch: feature/current-cbm-pinned-upgrade. Production B3/B4 commits:
92e736d / 09d418e. Standing user authorization covers the session/branch checks;
only controlled fixtures and owned sessions were used. Full suite remains CI-only.

- Provider source pin: 82b1ec908f975354ceb4bac8a8bfc594942a9bc4.
- Provider artifact: /workspace/cbm-authority-source/build/a5-gate0/codebase-memory-mcp.
- Verified provider SHA256: 6dfaa906ed310795a97ca9b992d4d42b6118a8cf740ea7b4550df6d8146b24f0.
- Clean binary: target/debug/clean-ctx, rebuilt locked/all-features, package binary,
  four build jobs, from the B4 tree.
- Clean SHA256: cd62c9e229f5698875832b792c5f15a22473cbeb9ffd7695ce12de76d2fe8219.
- Environment: existing Linux cloud environment, PowerShell 7.4.19.

The established normal account cache was used with a platform write grant.
No HOME/CBM_CACHE_DIR override, alternate account cohort, daemon kill, activation,
installer, global cache purge or unowned repository mutation was performed.

## Live results

**PROVEN BY LIVE/REPRODUCED BEHAVIOR:** final Phase B PowerShell run:
`target/cbm-phase-b/20261012-002833-15c1e329` — **19/19 case groups PASS**.
The run root contains every request/response, stderr, binary provenance, independent
source manifest, direct-provider cursor evidence and machine-readable verdicts.
Fixture primary/additional roots contain TypeScript/C# sources, including 225
functions in nine files and a 225-call cross-file hub. No language build was run.

| Phase | Established on the real MCP boundary |
| --- | --- |
| B1 | Actual registries/admission; grouped QN/file delivered as typed nodes; valid empty search; consumed architecture tables reach existing overview fields |
| B2 | Separate succeeded job, ready serving graph and best-effort coverage; canonical serving root; search does not rewrite job state; distinct additional-root routing; same-named C# contexts retain the correct namespace |
| B3 | Provider query cursor oracle spans pages; typed query reaches all 226 rows in the original source projection; typed search reaches all 225 probes on first and cached requests; typed trace reaches the direct endpoint selected beyond the first default page |
| B3/B4 | Reindexing only the owned fixture invalidates a real emitted query cursor; direct provider and proxy retain the stale-snapshot failure rather than offset restart |
| B4 | Raw query/search/trace/architecture/project-list pages retain provider facts; manual indexing returns bound terminal success; additional-project proxy calls preserve the active wrapper project; current bridge status reaches metadata; tree whitespace is retained; soft errors remain errors and the internal status operation is not added to the public whitelist |
| B5 | With provider explicitly disabled, high-fidelity C# and TypeScript contexts remain reachable |

The source manifest is the name/file/count oracle; provider captures independently
establish canonical identities and paging contracts. Canonical comparison ignores
only JSON object-key ordering, preserving array order, value types, string bytes,
null and empty values. Startup retries occur only on the explicit temporary
still_indexing flag and stay within a recorded case deadline.

The first live run had four harness errors (PowerShell property-to-bool conversion,
callback/body variable collision, missing required graph_trace.to, and ordered JSON
serialization comparison). These were corrected without changing production or
weakening acceptance. An intermediate run passed 12 groups, then 13 with the
no-provider control; a 14-group run includes stale-cursor evidence. The expanded
19-group run adds raw tool equivalence, project isolation and manual indexing. The initial
failures are retained in their separate run directories, not relabeled green.

## Verification boundaries and declared limitations

B0–B4 focused regression/preservation evidence and final B4 Clippy are recorded
in the slice reports. B5 changed only harness/docs. PowerShell syntax, strict
UTF-8/BOM, active-file size and whitespace checks passed.

The native Angular acceptance harness was also reused against this same binary
with provider explicitly disabled. All **39 live MCP assertions passed**, including
arbitrary template filenames, linkage/occurrence conditions, modern else-if/let/
track/empty/switch/defer companion delivery, lifecycle freshness and truthful
fidelity degradation. Evidence: target/live-acceptance/angular-template-linkage.
High can choose byte-exact passthrough instead of Medium's normalized spelling;
that legitimate presentation was verified against exact source, not mislabeled
as lost structure.

**NOT ESTABLISHED:** Windows/MSVC/antivirus acceptance of this current build;
Codex/Claude host tool selection or channel consumption; typed mid-collection
mutation/daemon restart on a real provider; live engine/budget exhaustion and
uncertain mutation acceptance. Their deterministic contracts remain protected,
but passing fixture checks are not promoted to those live claims. Shared-account
lifecycle acceptance remains the previously accepted A5 report, with its limits.
The harness does not establish all-source completeness or repair #142.

B0–B4 implementation and the available Linux B5 acceptance are green. This is
not an owner declaration that every platform/host gate is accepted or that the
remaining native semantic issues are resolved. No automatic deployment, push,
merge, issue closure or #142/Hybrid implementation was performed.
