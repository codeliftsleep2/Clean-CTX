# Issue #162 Phase 1 host audit

Agent: Agent MaxHeadRoom

Date: 2026-10-21

Status: **host implementation checkpoint audited; runtime UNVERIFIED**.
This is not Phase 1 runtime acceptance, an Angular consumer migration receipt,
or evidence that #162's canonical questions are answerable yet.

## Actual production path

Batch acquisition uses `hydration/publication.rs::prepare_candidate` to create
`CompilationJob`. The job carries existing authorized roots and optional TS
project context through `SemanticCompilerExecutor::compile` into
`CompilationRequest`, `compile_source_artifact`, and
`SemanticCompilationArtifact::from_compiler`.

Direct file, focused provide, explicit-file query, calls, compress/delta and
in-memory edit paths use `tool_helpers.rs` candidates/focused compilation and
the same request/compiler/artifact boundary. Root policy comes from existing
`hydration::dependency_roots`; a source parent does not authorize a workspace.

For TS-family files, artifact construction calls
`TypeScriptAuthority::capture` or `capture_in_project`. Resolver creates Host,
seeds the supplied immutable source bytes, captures canonical declarations,
selects project inputs and resolves initial references. The capture is returned
in the real artifact, not populated only by a custom test pipeline.

Existing ordinary production callers pass no explicit project. The actual
executor transports an explicit project when supplied, as covered by an authored
compilation regression. Angular producers still need to derive applicable
explicit context from authoritative project/application evidence. This optional
input has not been demonstrated through public dispatch with such a producer.

`into_parts` still drops the neutral authority capture. It is therefore not yet
a persistent semantic owner, publication proof or query-consumable authority.
That known Phase 3 boundary prevents a production-integration completion claim.

## Boundary audit

| Requirement | Implementation and static finding | Regression authority / limits |
|---|---|---|
| Authorized snapshots | Host admits explicit roots, captures exact read bytes and hashes those bytes; supplied target source is seeded independently of current disk text | Snapshot immutability, seeded-byte and source-only tests authored; target publication must separately validate supplied source hash |
| Source-only scope | Only the supplied file is authorized when no roots exist; project/dependency access is not inferred from its parent | Source-only and foreign-config tests; all unrun |
| Physical identity | Host normalizes physical routes before Unit parsing/caching and declaration construction; capture retains both physical source and original source route | Unix alias and Windows junction capture tests authored; preserveSymlinks interpretation remains Phase 2 work |
| Lossless path representation | New authority host rejects unrepresentable physical paths before lossy component comparisons could conflate identities | Unix invalid-byte physical-root test authored; no global path policy changed |
| Negative/error observations | Missing, kind, physical, unreadable and outside-scope observations stay distinct; invalid UTF-8 and failed canonicalization do not certify absence | Appearance, directory, invalid-text, dangling-link and sharing-denial tests authored |
| Windows reads | Final opened-handle path must be in authorized roots before reading that handle; API failures remain unresolved | Junction escape/retarget and sharing-denial tests authored; native FFI not compile/runtime verified |
| Config inheritance | JSONC/BOM parsing preserves raw source hashes; options retain origins and null resets through multiple bases; root lists inherit separately; references do not inherit | Multiple-base, origin, null-reset, nullable-root and BOM tests authored |
| Project ownership | Files/include/exclude and explicit reference graph determine root candidates; inferred overlap/cycles fail closed; explicit requested context is distinguished | Reference selection, overlap, cycle, restoration and explicit-context tests authored |
| Jsconfig | Discovery records missing tsconfig before jsconfig use; jsconfig owns its defaults before explicit options | Jsconfig precedence, inherited/default/explicit allowJs cases authored |
| Defaults/compatibility | Installed compiler manifest is a consumed witness; reviewed version rules feed shared effective-module/default and compatibility interpretation | Version mutation, legacy/TS6 and invalid-option tests authored; not blanket conformance for every installed compiler release |
| Retained inputs | Capture now retains all effective compiler-option values and declaring directories; unknown options are inputs, not certified semantics | Serialization/invalidation regression authored; later resolvers must interpret relevant options rather than ignore them |

## Corrections made during this audit

1. Effective options previously disappeared after a subset was projected into
   mode/base/paths/conditions. Capture now retains effective inputs and origins
   for the remaining resolution policies. This is prerequisite data, not a claim
   that every option is already implemented.
2. Source extraction could parse with an alias route while reference resolution
   parsed with the physical path. Unit parsing/caching now uses the same physical
   identity; `source_route` separately retains the original input route.
3. Scope comparisons used existing string-component mechanics. The new host now
   rejects non-lossless physical identities rather than allowing replacement
   characters to equate different native paths.

## Remaining ownership by phase

These are required work, not exceptions or capability exclusions.

- **Phase 2:** lexical spans/namespaces/export authority and mode-specific module
  resolution; interpret retained options such as preserveSymlinks, rootDirs,
  module suffixes and package imports/exports controls. Keeping those values is
  not implementing their meaning. Preserve logical reference context while
  canonical declaration identity stays physical.
- **Phase 3:** carry capture through returned parts, actual persistent owner,
  currentness, publication, reset/deletion and durable reopen. Project inputs,
  roots and producer generations must participate in reuse identity. Neither
  `capture_current` nor equal target hashes certify the surviving publication.
- **Phases 4–6:** Angular API/declaration roles, producer-owned project-context
  derivation, shared composition and all consumer migrations. Existing callers'
  None inputs cannot masquerade as a proved application build context.
- **Phases 7–8:** run authored regressions/preservation gates and applicable real
  binary/public-query acceptance. No test count, warning-free compilation,
  runtime GREEN or live acceptance has been established for this implementation.
- **Cross-phase conformance:** any materially required compiler/config/platform
  form uncovered by implementation must get an explicit question/test/repair
  record. An unresolved result protects truth but cannot close an in-envelope
  feature. Do not silently generalize the reviewed release rules to other versions.

## Static evidence

Direct rustfmt, UTF-8/no-BOM, active-file size checks and `git diff --check`
were used. No Cargo, binaries, tests, MCP/provider processes or live verification
were run. Supplemental tests are tracked under `src/tests/**`; all are unrun.
The exact original public-dispatch RED remains frozen in its existing stash.

Audit supplemental snapshots:

- `typescript_project_context_issue_162.rs`: nine authored tests, including two
  Unix-only cases; SHA-256
  `FF9BA26745C37E96D67FFD2F7576D9B814AD851A7AED0DD48315D602FDA358B9`.
- `typescript_windows_host_issue_162.rs`: four authored Windows-only tests;
  SHA-256 `529D4D7A4D6D273FE05A4F5E5E3A08FF0A65F4101B139D5351C1E06DF75C013A`.

Four tests were added during this audit: retained effective-option origins and
serialization/invalidation, Unix physical alias identity, rejection of lossy
physical root identity, and Windows junction capture identity. None has run.

Canonical question linkage: AQ01 and AQ25 plus shared authority/dependency/
lifecycle prerequisites for all Section 6A questions. Query reachability and
correct completeness presentation remain open; this audit does not satisfy
those later gates.
