# Issue #132 Final Forensic Reconciliation

**Agent:** Agent MaxHeadRoom
**Date:** 2026-10-16
**Verdict:** **#132 VERIFIED - SUPERSEDED AND COMPLETE**

## 1. Executive verdict

No production-reachable #132 residual remains. Issue #141 and the ordered
template architecture supersede #132. Current code and tracked tests prove the
verdict; accepted #141 live/final-gate results are historical live evidence.

`TemplateShape` remains reachable, but only as a deliberately lossy Low
projection and marker source. Medium/High/Edit use `OrderedTemplate`.
Selector semantics use structured occurrences with spans/control ancestry.
Low declares `ordering: aggregate_unordered` and omits
`element_ownership`/`control_ancestry`, so adjacency cannot claim nesting.

## 2. Exact #132 acceptance matrix

| Requirement | Verdict | Evidence |
| --- | --- | --- |
| Arbitrary linked HTML | RESOLVED | Shared structured `templateUrl` admission; provide/entity tests. |
| Sequential/nested control ownership | RESOLVED | Ordered events and occurrence ancestry tests. |
| Else/for/empty/switch/defer/let/legacy | RESOLVED | Structure/version-equivalence tests. |
| Text/interpolation/binding ownership | RESOLVED | Ordered content/start tags; honest degradation. |
| Singular component call includes template | SUPERSEDED | Batch context plus canonical `HasTemplate`; no `includeTemplate`. |
| Component/Template/child edges | RESOLVED | `HasTemplate`, neutral `UsesSelector`, unique `UsesComponent`. |
| Partial-result disclosure | RESOLVED / TRANSFERRED | Angular coverage is truthful; general history remains #142. |
| Unsupported HTML wording | RESOLVED | Verbatim is described as raw/uncompressed. |

## 3. Current production call-path trace

`provide_code_context` -> `try_evaluate_angular_template` -> shared ownership
admission -> `compress_template` -> Low flat aggregate or non-Low
`template_ordered::extract().render()` -> MCP content/degradation.

`entities_in_file` performs shared Angular admission before generic rejection
and returns canonical `angular/Template` identity.

TypeScript decorator -> `HasTemplate` -> WorkspaceIndex -> request-local
Angular catalog -> current inline/external source -> structured selector
occurrences -> `UsesSelector` -> exact `HasSelector` lookup ->
`UsesComponent` only for one candidate.

## 4. TemplateShape authority/reachability

Classification: **CURRENT MODEL-FACING PROJECTION ONLY**.
`extract_template_shape(_with_depth)` and `to_marker_line(s)` remain live at
Low. `to_medium_lines`/`to_high_lines` remain methods, but the production
compressor bypasses them for non-Low and uses `OrderedTemplate`. PrimeNG
discovery reads custom-element facts only. Component/template identity and
selector ancestry never derive from flat collections.

## 5. Sequential/nested falsification

`angular_17_sequential_if_blocks_keep_their_conditions_and_children` proves
condition A owns its div and condition B owns the child. Occurrence tests prove
distinct spans/ancestry; balanced regions distinguish nesting from later
siblings. Low asserts no relationship because it is aggregate/unordered.

## 6. Filename-independent authority

Conventional names are hints. Arbitrary linked names gain authority through
canonical structured linkage; unowned HTML remains unsupported. Tracked
provide, entity, and forward/reverse tests cover all three cases.

## 7. Inline/external convergence

External templates use canonical path identity; inline templates use
`inline:<source>#component=<class>`. Both converge at ordered parsing,
occurrence extraction, and relationship refinement.

## 8. Fidelity/degradation honesty

- Low: aggregate; declares ownership, ancestry, text, interpolation-expression,
  and comment omissions.
- Medium: source ordered; declares comments omitted.
- High: retains richer content/comments.
- Edit: structural HTML, no false byte-exact method-body claim.
- Verbatim/raw: exact document, no degradation claim.

## 9. Component/template/child authority chain

Component -> structured decorator -> canonical Template -> current source ->
element occurrence -> neutral selector reference -> exact published selector
authority -> unique Component only. Zero/multiple matches stay neutral with
diagnostics; spelling never manufactures Component identity.

## 10. Lifecycle dependency analysis

The Component owns durable `HasTemplate`; external content is a current
dependency. Tests cover replacement, inline/external transitions, missing
targets, deletion/restoration, refresh, selector changes, and stale-result
rejection. This reuses #139/#141 validity and adds no cache/watcher/index.

## 11. #141 supersession mapping

| #132 area | Authority | Verdict |
| --- | --- | --- |
| Recognition | Shared ownership admission | RESOLVED by #141 |
| Structure/content | Ordered events plus degradation | RESOLVED |
| Component/template access | Batch plus canonical linkage | SUPERSEDED |
| Template/child edges | Durable linkage plus request-local refinement | RESOLVED |
| Partiality | Angular diagnostics; general #142 | RESOLVED / TRANSFERRED |
| Error wording | Ownership-aware error | RESOLVED |

## 12. Residual discovered

None. Flat `TemplateShape` is legacy inside a truthful Low projection, not a
semantic contradiction.

## 13. Implementation performed

Documentation only. No production/test code or architecture changed.

## 14. Tests/evidence reused

`template_structure_regressions.rs`, `template_version_equivalence.rs`,
`provide_angular_template_ownership.rs`,
`workspace_query_angular_templates.rs`,
`workspace_query_angular_selector_occurrences.rs`, and
`angular_template_phase6_diagnostics.rs`. Accepted #141 native 7/7 and the
maintainer-reported final gate are historical live evidence.

## 15. Commands not run

No Cargo test/check/build/Clippy/fmt, binary, MCP server, live harness, model
pilot, or final gate ran. Only read-only inspection and this report edit ran.

## 16. Remaining uncertainty

None material. Singular `includeTemplate` does not exist, but batch context
plus canonical linkage supersedes its functional requirement. General
query-history completeness remains #142.

## 17. Recommended GitHub disposition

**CLOSE #132** as superseded and complete. Preserve the nuance that
`TemplateShape` remains a declared-loss Low projection rather than claiming
it was removed.
