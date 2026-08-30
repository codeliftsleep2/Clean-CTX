# Clean-CTX Semantic Relationship Model — Implementation Plan

## Consolidated Single Report

*Incorporates the architectural investigation and all six resolved uncertainties.*
*Codebase verified at commit e28f6047.*
*Created: 2026-08-30*

---

## 1. Executive Summary

Clean-CTX has three parallel semantic systems that are not connected:

| System | Purpose | Status |
|--------|---------|--------|
| Meta-layer blocks | Per-file semantic extraction to Phi text markers | **Active** |
| InferenceLayer | Typed edge model with provenance/confidence | **Active, but only CBM-fed** |
| Graph modules | Cross-file semantic dependency graphs | **Dead code, compile but never called** |

**The gap:** Meta-layers discover semantic relationships but flatten them to text. The InferenceLayer has structured edges but never receives meta-layer data. The graph modules contain resolution algorithms that work but have no producer.

**End state:** Meta-layers produce both text (unchanged) and SemanticEdge objects. The InferenceLayer holds both inferred_edges (CBM/structural) and semantic_edges (meta-layer facts). A WorkspaceIndex enables cross-file semantic queries without CBM. Legacy graph modules are deleted after their test callers are migrated.

**Total new code:** about 1000 lines across all phases. No new MCP tools, no graph database, no compress_workspace restoration.

---

## 2. Architectural Target

```
MetaLayer (trait)
  enrich()                -> Option<MetaLayerOutput>   [existing, unchanged]
  extract_semantic_edges() -> Vec<SemanticEdge>        [NEW, default empty]

InferenceLayer (src/ir/inference_layer.rs)
  inferred_edges: Vec<InferenceEdge>    [existing, CBM/structural/heuristic]
  semantic_edges: Vec<SemanticEdge>     [NEW, meta-layer facts]
  annotations: HashMap<...>             [existing, unchanged]

WorkspaceIndex (optional, Phase 4)
  forward/reverse indexes, dedup, file tracking
  resolution algorithms from legacy graphs
```

### Pass Pipeline Integration

```
Pass 3: MetaLayerPass ---------> state.semantic_edges (accumulates)
                                      |
Pass 8: InferenceLayerPass ----------> state.semantic_edges.drain(..)
                                      |
                                      v
                               InferenceLayer
                               +-- semantic_edges (meta-layer)
                               +-- inferred_edges (CBM)
```

---

## 3. Identity Model

**Resolved (U1):** Per-file extraction uses `PassContext.file_id` (a String, the files path). NOT the legacy alpha-N alias.

```
pub struct EntityRef {
    pub domain: &'"'"'static str,       // "angular", "ngrx", "dotnet", "spring"
    pub entity_type: &'"'"'static str,  // "Component", "Service", "Action"
    pub name: String,               // "UserComponent", "UserService"
    pub file: Option<String>,       // file_id from pipeline; None for local-only
}
```

- PartialEq/Hash implemented on (domain, entity_type, name) only -- file excluded for identity matching across files.
- CBM uses its own FQN identity system (e.g., C-Users-...UserComponent.constructor). Semantic edges exist alongside CBM edges in separate vecs, so no identity collision.

---

## 4. SemanticEdge Types

```
pub struct SemanticEdge {
    pub relation: SemanticRelation,
    pub subject: EntityRef,
    pub object: EntityRef,
    pub layer: &'"'"'static str,
}
```

### SemanticRelation Categories

**Angular:** Injects, HasInput, HasOutput, HasModel, HasSelector, HasTemplate, HasStyle, DeclaresInModule, ImportsModule, ExportsFromModule, RouteMapsTo, GuardedBy, ResolvedBy

**NgRx:** Dispatches, Selects, HandlesAction, CallsService, HasStore

**.NET:** ControllerAction, HasRoute, HasEntity, EntityRelationship, MapsFrom, MapsTo, HubMethodTargets

**Spring:** Autowired, EndpointMapsTo, BeanProduces, ConfigurationProperties

**Generic:** Extends, Implements, Defines, Calls

### Design decisions
- Semantic edges have implicit confidence 1.0. No confidence field on SemanticEdge.
- Provenance tracked via layer (e.g., "angular", "ngrx").
- CBM edges remain in inferred_edges with confidence 0.75 and source: InferenceSource::Cbm.
- Duplicate edges handled at WorkspaceIndex boundary, not during per-file extraction.

---

## 5. Detailed Phase Plan

### Phase 0: Foundation

**Objective:** Define types and extend traits. No production behavior changes.

**Files to modify:**
- src/ir/inference_layer.rs -- add semantic_edges field + query methods
- src/ir/pipeline.rs -- add semantic_edges to PassContext
- src/layers/meta/mod.rs -- add extract_semantic_edges() to MetaLayer trait

**Files to create:**
- src/layers/meta/semantic.rs -- SemanticEdge, EntityRef, SemanticRelation definitions

**Implementation sequence:**

1. Create src/layers/meta/semantic.rs with EntityRef, SemanticRelation enum, SemanticEdge struct.
2. Extend InferenceLayer with semantic_edges field and add_semantic_edge(), semantic_edges(), semantic_edges_for(), all_edges_for() methods.
3. Add semantic_edges: Vec<SemanticEdge> to PassContext in pipeline.rs.
4. Add extract_semantic_edges() to MetaLayer trait with default Vec::new().
5. Export pub mod semantic; from src/layers/meta/mod.rs.
6. Add collect_semantic_edges() to LayerRegistry.

**Tests:** EntityRef equality/hash, SemanticEdge construction, InferenceLayer integration, default empty trait method.

**Risks:** Adding field to InferenceLayer may break pattern matches in test files. Add .. patterns where needed. Import cycles between ir and layers -- keep semantic.rs in lower tier.

**Rollback:** Revert commit. No production behavior changed.

**Scope:** about 200-250 lines.

### Phase 1: Angular Reference Implementation

**Objective:** Implement extract_semantic_edges() for AngularMetaLayer.

**Files to modify:**
- src/angular_meta/mod.rs -- add extract_semantic_edges() function
- src/layers/meta/mod.rs -- wire AngularMetaLayer
- src/angular_meta/ngrx.rs -- add to_ngrx_semantic_edges()

**Files to create:**
- src/angular_meta/semantic.rs -- Angular EntityRef constructors

**Key design:** Zero duplication of parsing logic. Edge extraction reuses the same helper functions that run_meta_layer_with_config() calls: extract_decorators(), extract_graph_entries(), extract_ngrx_shape(), extract_route_shape(), extract_rx_shape().

The existing to_graph_edges() returns Vec<(String, String, NgRxEdgeKind)>. Add parallel to_ngrx_semantic_edges() outputting Vec<SemanticEdge>. Old method stays until Phase 5.

**Relationships extracted (all exact, confidence 1.0):**
Component -> Injects -> Service
Component -> HasInput -> InputField
Component -> HasOutput -> OutputField
Component -> HasSelector -> selector-string
NgModule -> DeclaresInModule -> Component
Module -> ImportsModule -> Module
Route -> RouteMapsTo -> Component
Route -> GuardedBy -> GuardClass
Route -> ResolvedBy -> ResolverClass
Effect -> HandlesAction -> Action
Effect -> CallsService -> ServiceMethod
DispatchSite -> Dispatches -> Action
SelectSite -> Selects -> Selector

**Deduplication (U2):** Per-file extraction does NOT deduplicate. Deduplication happens at WorkspaceIndex.

**Tests:** All Angular relationship types. Edge count matches Phi marker count. Text output unchanged.

**Scope:** about 250-350 lines.

### Phase 2: Pipeline Integration

**Objective:** Wire MetaLayerPass to accumulate edges, InferenceLayerPass to consume them.

**File to modify:** src/ir/pipeline.rs

**MetaLayerPass.run():** After existing Phi marker extraction, call registry.collect_semantic_edges() and annotate each edge with state.file_id. Store in state.semantic_edges.

**InferenceLayerPass.run():** After state.inference_layer = Some(layer), drain state.semantic_edges and call inference_layer.add_semantic_edge() for each.

**Why this works (U5):** MetaLayerPass = Pass 3, InferenceLayerPass = Pass 8. PassContext lives across all passes.

**State lifetime (U6):** PassContext is per-file. After pipeline, inference_layer lives in state.inference_layer. Caller extracts it or feeds into WorkspaceIndex.

**Tests:** Full pipeline produces edges. Skip gracefully when no meta-layers match. Empty edge list. Text output unchanged.

**Scope:** about 30-50 lines.

### Phase 3a: .NET Edge Extraction

**Files to modify:** src/dotnet_meta/mod.rs, src/layers/meta/mod.rs
**Files to create:** src/dotnet_meta/semantic.rs

**.NET relationships:** Controller -> ControllerAction, Controller -> HasRoute, DbContext -> HasEntity, Entity -> EntityRelationship, MapperProfile -> MapsFrom/MapsTo, HubMethod -> HubMethodTargets

**Scope:** about 200 lines.

### Phase 3b: Spring Edge Extraction

**Files to modify:** src/spring_meta/mod.rs, src/layers/meta/mod.rs
**Files to create:** src/spring_meta/semantic.rs

**Spring relationships:** Controller -> EndpointMapsTo, Controller -> Autowired -> Service, Configuration -> BeanProduces, PropertiesClass -> ConfigurationProperties

**Scope:** about 200 lines.

### Phase 4: Workspace Index

**Files to create:** src/workspace/index.rs, src/workspace/mod.rs, src/tests/workspace/index.rs

**Design:** WorkspaceIndex with entities (HashMap), forward/reverse index, file_map for cleanup, edge_set for dedup. Dedup key = (relation, subject:identity, object:identity) where identity ignores file (U2).

**Query API:** find_entities(), forward_edges(), reverse_edges(), resolve_inject_type(), resolve_selector(), has_cycle(), transitive_dependencies().

**File alias (U1):** Index stores both file_id and optional alias. Alias resolved at index time.

**Tests:** Cross-file DI resolution, reverse query, determinism, cycle detection, dedup, file removal, transitive deps.

**Scope:** about 400-500 lines index + about 300 lines tests.

### Phase 5: Legacy Graph Deletion (Four Steps)

**Step 5a:** Extract NgRxEdgeKind from graph.rs into ngrx.rs.

**Step 5b:** Add AngularGraph::from_workspace_index() adapter to keep tests green.

**Step 5c:** Migrate 10 test files to use WorkspaceIndex instead of AngularGraphBuilder/GraphCollector.

**Step 5d:** Delete 8 files (about 2430 lines) after zero imports remain.

**Keep permanently:** src/compression/graph_utils.rs

### Phase 6: CBM Coexistence Verification

**Test strategy (U3):** Use existing new_mock_with_edges() from src/cbm/bridge.rs::test_helpers. Add semantic edges. Verify both types present via all_edges_for(). No new mock infrastructure needed.

**Scope:** about 50-100 lines.

---

## 6. Commit Boundaries

| Commit | Phase | Message |
|--------|-------|---------|
| 1 | 0 | feat(ir): add SemanticEdge, EntityRef types; extend MetaLayer trait; add semantic_edges to InferenceLayer and PassContext |
| 2 | 1 | feat(angular): implement extract_semantic_edges() for AngularMetaLayer |
| 3 | 2 | feat(ir): wire MetaLayerPass to accumulate semantic edges, InferenceLayerPass to consume them |
| 4 | 3a | feat(dotnet): implement extract_semantic_edges() for DotNetMetaLayer |
| 5 | 3b | feat(spring): implement extract_semantic_edges() for SpringBootMetaLayer |
| 6 | 4 | feat(workspace): add WorkspaceIndex for cross-file semantic queries without CBM |
| 7 | 5a | refactor(angular): extract NgRxEdgeKind from graph.rs into ngrx.rs |
| 8 | 5b | test(angular): add WorkspaceIndex to AngularGraph compatibility adapter |
| 9 | 5c | test(migration): migrate 10 test files from legacy graph modules to WorkspaceIndex |
| 10 | 5d | chore(cleanup): delete legacy graph modules, footer, bundler (8 files) |
| 11 | 6 | test(ir): verify semantic edges and CBM inference edges coexist correctly |

---

## 7. What NOT to Build

1. No graph database. HashMap-based in-memory storage is sufficient.
2. No generic ontology. Entity/relation types remain framework-specific.
3. No standalone MCP graph tool. Semantic edges enrich existing pipeline.
4. No cross-language resolver without CBM. Heuristic is as far as meta-layers go.
5. No replacement of Phi text output. Text remains agent-facing format.
6. No compress_workspace restoration. Legacy pipeline gone permanently.
7. No requirement for all meta-layers to produce edges. Default returns empty.
8. No InferenceLayer renaming. semantic_edges coexists with inferred_edges.
9. No InferenceSource on SemanticEdge. Semantic facts are structural by definition.
10. No merging of CBM identity with EntityRef. Different identity systems intentionally.

---

## 8. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Adding field to InferenceLayer breaks pattern matches in tests | Medium | Medium | Check all match/destructure patterns; add .. |
| Import cycles between ir and layers | Low | High | layers::meta::semantic is lower tier; ir imports layers, never reverse |
| Per-file edge extraction duplicates parsing logic | Medium | Medium | Reuse existing helper functions; no new parsing |
| Legacy graph test migration (10 files) is tedious | High | Low | Adapter keeps tests green during migration |
| NgRx to_graph_edges() tied to old NgRxEdgeKind | High | Low | Add parallel to_ngrx_semantic_edges(); delete old with graph.rs |
| WorkspaceIndex performance at 10K+ files | Low | Low | HashMap O(1) lookups; Vec-append edge storage |
| Deleted graph modules have hidden test callers | Low | High | Search all test files before deletion |

---

## 9. Definition of Done

1. SemanticEdge, EntityRef, SemanticRelation defined in src/layers/meta/semantic.rs
2. MetaLayer trait has extract_semantic_edges() with default empty impl
3. InferenceLayer has semantic_edges field with query methods
4. PassContext has semantic_edges: Vec<SemanticEdge> field
5. AngularMetaLayer produces edges for: injects, NgRx, routing, RxJS
6. DotNetMetaLayer produces edges for: controller actions, entities, relationships
7. SpringBootMetaLayer produces edges for: endpoints, autowired, beans
8. MetaLayerPass accumulates edges; InferenceLayerPass consumes them
9. WorkspaceIndex supports cross-file lookup, forward/reverse queries, resolution, cycle detection
10. Legacy graph modules (8 files) deleted after 10 test files migrated
11. All existing Phi text output unchanged
12. All existing tests pass
13. cargo clippy --all-targets -- -D warnings passes with all features
14. CBM and semantic edges coexist in InferenceLayer with distinct types
15. No new MCP tools, no graph database, no compress_workspace restoration

---

## Appendix A: Six Resolved Uncertainties

### U1: File Alias Availability
PassContext has pub file_id: String (line 87 of pipeline.rs). Per-file extraction uses file_id directly. Alpha-N aliases are resolved at workspace-index time.

### U2: Duplicate Semantic Edges
Per-file extraction does NOT deduplicate. WorkspaceIndex deduplicates using key of (relation, subject:identity, object:identity) where identity ignores file.

### U3: CBM Testing
src/cbm/bridge.rs has pub mod test_helpers with new_mock(), new_mock_empty(), new_mock_with_edges(). Phase 6 uses these.

### U4: Legacy Graph Test Callers
10 test files import from legacy graph modules. Four-step migration: extract, adapt, migrate, delete. src/compression/graph_utils.rs kept permanently.

### U5: Pipeline Ordering
MetaLayerPass = Pass 3, InferenceLayerPass = Pass 8. Bridge is state.semantic_edges on PassContext.

### U6: Semantic-Edge State Lifetime
PassContext is per-file. Edges flow: Pass 3 to state.semantic_edges to Pass 8 to inference_layer.semantic_edges. After pipeline, caller extracts and optionally feeds into WorkspaceIndex.

---

*Single consolidated implementation plan.*
*Architectural authority: the investigation.*
*Code authority: commit e28f6047.*
