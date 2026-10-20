# Issue #155: explicitly excluded follow-up campaigns

Agent: Agent MaxHeadRoom

Status: local follow-up inventory; no GitHub issues have been created by the agent.
These identifiers name separate proposed campaigns, not assigned GitHub numbers.
All findings are static susceptibility findings from the accepted sibling audit;
none is claimed to have newly reproduced runtime RED.

| Campaign | Existing evidence | Separate work / acceptance target |
|---|---|---|
| FU-155-EF-MODEL | `src/mcp/tool_handlers/query/csharp_resolution/dependencies/efcore.rs::build`, `configuration_entities`, `resolve_written_class`; existing EF tests explicitly publish context/configuration/entity files | Model entity references, explicit configurations and assembly configuration universes need their own acquisition obligations and cold/order regressions. C# base closure does not establish model ownership. |
| FU-155-ANGULAR-MODULE | `src/angular_meta/module_metadata.rs::compose`, `resolve_leaf`; direct external reads at imported arrays/declarations; dependency assembly in `src/layers/meta/angular_semantic.rs` covers routing/NgRx | Track imported module/spread-array dependency footprints, missing sources and mutation/retraction; preserve the current supported composition envelope. |
| FU-155-NGRX-AUTHORITY | `src/angular_meta/ngrx/object_actions.rs::dependencies`, `object_registration.rs::hydration_terms`, `src/workspace/index/ngrx_refinement.rs::rebuild_ngrx_import_refinements` | Prove required dispatch/registration/reducer/reexport neighborhoods and their invalidation. Preserve #152 positive authority/retraction; do not replace carriers with name correlation. |
| FU-155-SELECTOR-PROOF | `src/mcp/tool_handlers/query/angular_templates.rs::source_complete` and candidate `unique`; `query/edges.rs::apply_angular_template_metadata` | Bind selector completeness to a closed authoritative source/candidate universe and final evidence rather than discovery status plus template availability. |
| FU-155-TESTCLASS-OWNERSHIP | `src/mcp/tool_handlers/query/dotnet_testing.rs::Catalog::build`; `csharp_resolution.rs::unique_class_declaration_file` | Review terminal-name subject correlation and structured owner identity separately. Preserve legitimate test-subject acquisition. Do not reopen #153/#154 declaration capture identity. |
| FU-155-PRODUCER-CERTIFICATES | Non-C# `query/entities.rs` file certificates/NgRx omissions; producer-specific request-local catalogs and remaining specialized preparation identified by the audit | Define producer-specific semantic obligations and final certificate validation for non-C# consumers. The C# stamp/shared batch barrier added by #155 does not certify Angular/NgRx/other producer closure. |

Audit authority: `docs/agent/ISSUE_155_SIBLING_PATTERN_AUDIT.md`, especially sections
A, B/C, D and E/F. The accepted #155 scope explicitly excludes implementing these
campaigns. No EF, Angular, NgRx, selector, or TestClass ownership producer has been
modified as part of closing the four in-scope blockers.

Other source-local Spring/native projection findings were classified SAFE for the
inspected source-only contract. No new unrelated framework capability is proposed
by this inventory. Neutral interface/base-list and test-subject acquisition paths
remain until their distinct requirements earn an equivalent shared replacement.
