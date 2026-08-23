# Semantic-Completeness Audit — Clean-CTX (2026-08-22)

**Scope:** Read-only production-path audit. No implementation code was modified, nothing was committed, no roadmap changes made. Every claim below is traced through actual production entry points with exact file/function citations. CBM-specific issues are isolated in §9.

**Reference commit:** `129e080` (Angular decorator fix), reconciled branch `feature/angular-deepening-ngrx-rxjs`.

---

## 0. Executive Assessment

Clean-CTX **does not** provide semantically complete architectural context on its primary LLM-facing production path.

An LLM receiving only `compress_code_context` / `provide_code_context` output for representative C#, Java, Rust, and TS/Angular applications can reconstruct the **shape** (class/method/param/return names, X/I hierarchies, imports) but **cannot** reconstruct application **wiring**: controller→service→repository→database chains, DI registrations, route tables, HTTP/DB/event opera-composition roots.

**Most consequential correction to the handoff's Bug-6:** the language layers (TS/C#/Rust) **are wired into the IR path** and DO run substring heuristics over full method bodies. Those become `df:`/`cf:`/`se:`/`ec:` annotations rendered **only at High**. The real gap is that no production path captures actual call/data-flow/construction/event semantics; the heuristics capture only a tiny, false-positive-prone whitelist, and the renderer drops them below High.

---

## 1. Production paths and the "primary" feed

Two compression paths exist and **diverge**:

- **IR path (PRIMARY — the LLM actually receives this):** `handle_compress_code_context` (src/mcp/tool_handlers/core.rs L175-183), `handle_provide_code_context` FullCompress (L1086-1102) / DeltaTransport (L921-1054), and `handle_restore_context` (L1283-1297) all call `compile_file_ir`/`compile_file_ir_focused` (src/mcp/tool_helpers.rs L232-358). The IR is `ir_to_hierarchical` → `render_hierarchical_for_llm` (SCHEMA v2 text).
- **Text path (secondary):** `compress_workspace` per-file manifests (src/mcp/workspace.rs `compress_pass` → `compress_file_with_source`), `delta_text_context`. This is the path the Angular text fix touched.

The IR path feeds meta-layers through `MetaLayerPass` (src/ir/pipeline.rs L728-742): it collects `CoreOp::DefClass(name)` names → `registry::run_meta_layers_pipeline` → each meta `enrich(ir)` extracts **class NAMES** again.

---

## 2. Current semantic model — what actually survives

### IR path (what LLM sees)

| Concern | Status on IR path | Evidence |
|---|---|---|
| Class names | ✓ | `DefClass` from `extract_class_name(raw)` (CoreIRPass src/ir/pipeline.rs L491-494) |
| Methods + params + return | ✓ | `emit_method_ir` (L199-250); render `M name → p:… → ret` |
| Fields types | ✓ | `DefField` + `Param`; `render_fields` at Med/High |
| Visibility flags | ✓ `fl:` | `Flags`/`ClassFlags` ops → L268 |
| Extends/Implements | ✓ `X`/`I` | language-layer `Extends`/`Implements` → render L128-134 |
| Imports | ✓ `$` | `Import` ops → render L79-89 |
| Attributes/decorators | ✗ ALL | `MetaLayerPass` feeds compacted names → all 3 meta-layers return None |
| Method-body semantics | * weak | language layers detect `subscribe/pipe/async/Task`/SaveChanges/File::/match | → `DataFlow`/`SideEffect`/`ControlFlow`/`ExecutionContext`, **High-only render** |
| DI/web/http/data-flow graph | ✗ | no Call/Construct/PropRead/PropWrite ops; only coarse substring labels |
| Bootstrap/composition | ✗ | no top-level/global-statement captures in any query |

### Text path (`compress_workspace`/`delta_text`)

Same declaration level, PLUS the Angular decorator fix (pipeline.rs L796 `decorator_inclusive_class_text` → `@` only). No language-layer semantics at all. `DF/SE/EC/CF` do not exist here.

---

## 3. Confirmed information-loss bugs (production evidence)

### BUG-1 (CRITICAL) — .NET meta-layer dead on both paths
- Feed: `decorator_inclusive_class_text` (pipeline.rs L793-814) only finds `@…)` decorators via `find_decorator_inclusive_start` (meta_util.rs L635-693) which recognizes only TS modifiers (`export/abstract/default/declare`, L647-660). C# `[ApiController]` or `public partial class` → returns compacted `cap.text`.
- Registry feed: src/layers/registry.rs L140-146 wraps compacted names as `DefClass("", name)`; `DotNetMetaLayer::enrich` (src/dotnet_meta/mod.rs L192-203) passes them → `run_meta_layer` → `aspnet::extract_aspnet` needs `[ApiController]` (src/dotnet_meta/aspnet.rs) → `None`.
- Tests pass because they call `extract_aspnet` directly with full source.

### BUG-2 (CRITICAL) — Spring meta-layer dead on both paths
- Same feed break. Java `@RestController` preceded by `public` — not in the modifier list (meta_util.rs L647-660).
- `extract_annotations` (src/spring_meta/annotations.rs L50-52) needs `find_tagged head`; a bare `"UserController"` has no `class `/`{` → helper L575-589 returns None → zero markers before any scan.

### BUG-3 (CRITICAL) — Angular decorators dead on IR (primary) path
- Same feed: `AngularMetaLayer::enrich` (src/layers/meta/mod.rs L147-158) extracts `DefClass` names → `extract_decorators("UserCardComponent")` → `find_class_head_end` (no `class `/`{`) → None.
- Exception: RxJS/NgRx/Signals/Routing sub-layers receive the FULL source (angular_meta/mod.rs L181-209) — so `Φobs:`/`ΦpipeRx:` etc. may still appear on the IR path even though `Φcmp:`/`Φsvc:`/… vanish. That masks the defect in tests that only assert the source-based sub-layers.

### BUG-4 (CRITICAL) — Workspace absents RSA+Java; .js always fails
- `COMPRESSIBLE_EXTENSIONS = ["ts","js","cs"]` (src/mcp/workspace_util.rs L28) but language layers: Rust `["rs"]`, Java `["java"]`, TS `["ts","tsx"]` (src/layers/language/mod.rs L74-75, L172-173, L258-259, L344-345).
- Effect: `.rs`/`.java` never collected or compressed.
- `.js`: collected in scan set but `language_for_extension("js")` → None → `compress_file_with_source` returns an error per file; per-file `provide_code_context` likewise errors. So Generic JS is UNSUPPORTED everywhere.

### BUG-5 (CRITICAL) — Composition roots / bootstrap lost everywhere
- CS/Java/RS/TS queries (src/queries.rs) capture declarations only — no top-level `expression_statement`/`global_statement`.
- `Program.cs` DI registrations/middleware/route wiring, Rust `fn main` router/pool/connection bodies, Java `SpringApplication.run`, TS `bootstrapApplication`: all dropped to nothing (method body not stored outside Edit).

### BUG-6 (CORRECTED from handoff) — Language layers are NOT no-ops on IR; the gap is semantic depth + fidelity gating
- Wired: src/mcp/tool_helpers.rs L289-306.
- Full raw method to layer: src/ir/pipeline.rs L576-577.
- Heuristics: typescript.rs L131-246, csharp.rs L124-223, rust.rs L180-250, java.rs — NO execution semantics.
- Produce `CoreOp::DataFlow/SideEffect/ControlFlow/ExecutionContext` which `ir_to_hierarchical` stores (hierarchical.rs L416-467) and `render_llm.rs` L265-315 renders **only at High**.
- What the handoff got right: bodies not stored at non-Edit; text path has no language layers; the heuristics are `contains()` over whole method text (false positives from strings/comments); and HTTP/DB calls (http.get, _repo.FindByIdAsync, PgPool::connect) are outside the tiny whitelist — the single most important wiring facts stay invisible.

### BUG-7 (MEDIUM) — Java language layer lacks execution semantics (asymmetry)
- java.rs implements only flags (static/abstract/visibility) + Extends/Implements; no DataFlow/SideEffect/ExecutionContext. Spring `JpaRepository`, `@Transactional`, `restTemplate/webclient` yield zero semantics.

### BUG-8 (CBM-only, not Clean-CTX correctness) — skip-set never populated in production
- `CbmFilterState.skip_sets` (src/mcp/state.rs) read by `get_skip_set`; no `.insert` anywhere in src except tests. CBM filter-first is inert. (Track 4.)

---

## 4. Intentional compression vs accidental loss

| Aspect | Intentional | Accidental |
|---|---|---|
| Method bodies at Low/Med/High | Omitted for tokens | Replaced by *nothing* informative (only High renders `df/ec` heuristics) |
| Stripping visibility at Low | ✓ | — |
| Meta-layer feed = compact names | — | ✗ BUG 1/2/3 |
| `.rs/.java` exclusion | — | ✗ BUG 4 |
| Composition roots | — | ✗ BUG 5 |
| `http.get`/`SaveChanges` (call graph) | — | ✗ BUG 6 (whitelist misses; High-only render) |
| Java exec semantics | — | ✗ BUG 7 (no heuristics at all) |


---

## 5. Language-by-language matrix (production IR path)

Legend: ✓ preserved; * = preserved only via coarse High-fidelity heuristics; ✗ = lost.

| Construct | C#/.NET | Java/Spring | Rust | TS/Angular IR | Generic JS |
|---|---|---|---|---|---|
| class/interface/struct/enum/record/trait/impl | ✓ | ✓ | ✓ | ✓ | ✗ (.js not routed) |
| methods+params+return+flags | ✓ | ✓ | ✓ | ✓ | ✗ |
| fields+types | ✓ | ✓ | ✓ | ✓ | ✗ |
| attributes/decorators | ✗ (dead feed) | ✗ (dead feed) | ✗ (only fl:/CFG) | ✗ on IR; ✓ text | ✗ |
| route/HTTP endpoint | ✗ | ✗ | n/a | ✗ (http.get not in whitelist) | ✗ |
| DI/bean/repo | ✗ | ✗ | n/a | limited `ec:di_scope` | ✗ |
| DB ops | * df:db_query/se:io (High only) | ✗ (no Java exec layer) | * File::/Tcp | ✗ | ✗ |
| async + await | ✓ | ✓ | ✓ | ✓ | ✗ |
| composition root | ✗ | ✗ | ✗ | ✗ | ✗ |

**Asymmetry (BUG-7):** C#/TS/Rust emit exec-semantics heuristics; Java emits none — a Spring repo-call method yields zero DataFlow/SideEffect while the C# equivalent yields `df/se/ec`. Also `.js` unsupported everywhere; `.rs/.java` absent from workspace.

---

## 6. Meta-layer matrix

| Layer | Input required | Text feed | IR feed | Result |
|---|---|---|---|---|
| Angular decorators | decorator-inclusive class text | ✓ (post-fix) | ✗ (DefClass name) | IR = nothing |
| Angular RxJS/NgRx/Signals/Routing | full source | ✓ | ✓ | works both paths |
| Angular graph | class blocks (workspace only) | workspace | workspace | §ΦGRAPH only in workspace |
| Spring Boot | annotation-inclusive class text | ✗ | ✗ | none either path |
| .NET | attr-inclusive class text | ✗ | ✗ | none either path |
| Rust | — none | — | — | only X/I + fl: |
| Generic JS/TS | — none | — | — | no wiring |

**Cross-file:** Only Angular `graph_pass` (src/mcp/workspace.rs L755-888) builds edges. `.NET`/`Spring` graph modules exist (src/dotnet_meta/graph.rs, src/spring_meta/graph.rs) but are **not wired** into the workspace path — dead code with live logic.

---

## 7. Before/after examples — can the LLM reconstruct the architecture?

### 7.1 C# / ASP.NET
```csharp
[ApiController]
[Route("api/users")]
public class UsersController : ControllerBase {
  private readonly IUserRepository _repo;
  public UsersController(IUserRepository r) { _repo = r; }
  [HttpGet("{id}")]
  public Task<UserDto> GetById(int id) => _repo.Find(id) ...
}
```
**IR-Low output (what the LLM receives):**
```
// SCHEMA v2 ...
// ── UsersController ──
M UsersController → params:
M GetById  → params:id → Task<UserDto>
```
Cannot answer: Is this a controller? Which route/verb? Does it depend on `IUserRepository`? Does it call the repo? Where is `_repo`? All wiring lost (BUG-1, BUG-6).

### 7.2 Java / Spring
```java
@RestController
public class UserController {
  @Autowired UserService svc;
  @GetMapping("/{id}") public UserDto get(@PathVariable Long id) { return svc.getById(id); }
}
```
**IR-Low:** `M get → params:id → UserDto`. No `@RestController`, no `svc`, no route, no service call (BUG-2, BUG-7).

### 7.3 Rust main
```rust
#[tokio::main]
async fn main() {
  let pool = PgPool::connect(&D).await.unwrap();
  let app = Router::new().route("/users/{id}", get(get_user)).with_state(pool);
  axum::serve(TcpListener::bind(":3000").await.unwrap(), app).await.unwrap();
}
```
**IR-Low:** `M main → fl:async`. Route table, `PgPool`, socket address — all gone (BUG-5).

### 7.4 TS/Angular
```ts
@Injectable({ providedIn: 'root' })
export class UserService {
  constructor(private http: HttpClient) {}
  getUsers() : Observable<User[]> { return this.http.get<User[]>('/api/users'); }
}
```
**IR-Low:** `M getUsers → Observable<User[]>`. `@Injectable`, `HttpClient`, `/api/users` — gone. `http.get` is not in the TS whitelist and the decorator feed is broken on IR (BUG-3, BUG-6).

**Reconstruction verdict:** In all four, `controller→service→repo→db`, `route→handler`, and `bootstrap→DI wiring` are unreconstructable. Even at High, the best available is a coarse `df:reads:observable`-style label — no callee/endpoint names.

---

## 8. Required architectural invariants (proposed, for docs + tests)

1. **C-13 (existing, must be widened):** meta-layer feed must be decorator/attribute-inclusive full class source text — `@` (TS/Java) **and** `[` (C#) — not a compact class name. Add a canonical `find_attribute_inclusive_start` handling both bracket forms plus intervening modifiers (`public`, `partial`, `final`, `export`, `abstract`, `static`).
2. **C-14 (existing, now critical):** IR path and text path must feed the SAME class-capture contract to meta-layers. The primary LLM-facing IR path currently violates this (BUG-3).
3. **C-15 (existing):** `COMPRESSIBLE_EXTENSIONS` must match language layers' `extensions()`. Add `.rs`, `.java`; resolve `.js` (either a JS grammar layer or explicit `unsupported`).
4. **C-17 (NEW):** meta-layers must operate on the primary IR path — the `MetaLayerPass` output must be asserted in production-shape tests (through `render_hierarchical_for_llm`).
5. **C-18 (NEW):** every language's top-level bootstrap (C# global statements, Java `main`, Rust `main`, TS bootstrap) must be captured as `statement.root`/`bootstrap.root` and either rendered as markers or fed to a composition meta-layer.
6. **C-19 (NEW):** semantic capture must be statement/span-based (AST anchors) — not whole-method `contains()` substring scans that match inside comments/strings.
7. **C-20 (NEW):** `df`/`cf`/`se`/`ec` rendered at `Medium` as well as `High` (these are the primary wiring signal).
8. **C-21 (NEW):** no CBM dependency for any Clean-CTX semantic guarantee; CBM only adds selection/targeting (see §9).

---

## 9. CBM section (targeting/enrichment only — NOT Clean-CTX correctness)

The architectural premise is respected throughout this audit: CBM is optional, never required to recover information Clean-CTX should have preserved. Do not add `InferenceLayerPass` to `default_production()` as a correctness fix.

CBM-specific findings (not Clean-CTX correctness bugs):

1. **CBM skip-set never populated in production** (BUG-8): `CbmFilterState.skip_sets` (src/mcp/state.rs) is ready and read by `get_skip_set`, but no production code path inserts entries (only tests do). CBM filter-first targeting is inert. This is a **targeting/enrichment** issue — `InferenceLayerPass` would consume CBM data only if present. Fix belongs in CBM wiring, not in Clean-CTX compression.
2. **CBM graph data never enters the default LLM context**: even with CBM installed, the compressed text only contains what Clean-CTX itself preserves; `graph_search`/`graph_query`/`trace_path`/`get_architecture` are separate tool calls the orchestrator must invoke. This is by design (optional enrichment) and not a Clean-CTX defect.
3. **Angular cross-layer graph (`graph_pass`)** already has CBM bridge hooks for `EffectEndpoint` resolution (workspace.rs L827-863), gated by `cross_layer_cbm`. When CBM is absent this degrades gracefully. No correctness action needed.

**Do not** extend the default pipeline for these; treat them in the Oracle-only enrichment path.

---

## 10. Prioritized remediation plan (by semantic impact)

Priority ordering is by **how much architectural wiring an LLM can reconstruct**, not by task count.

| Priority | Change | Fixes | Semantic impact | Key files |
|---|---|---|---|---|
| P0 | Fix meta-layer feed on the **IR primary path** (decorator/attribute-inclusive class text via a single canonical helper) | BUG 1/2/3 | High — restores `Φctrl/Φapi/Φrest/Φsvc/Φcmp/Φsvc` on the path the LLM receives | `src/layers/meta/mod.rs` `enrich()`, `src/ir/pipeline.rs` MetaLayerPass, `src/layers/registry.rs`, `src/meta_util.rs` |
| P0 | Fix the text path as well (C-13/C-14 unified) so `compress_workspace` is equivalent | same | `src/compression/pipeline.rs decorator_inclusive`, `src/meta_util.rs` |
| P1 | Capture bootstrap/composition roots (`statement.root`/`global_statement`) for CS/Java/RS/TS; render as a `composition` section or meta-feed | `-5` | **High** — restores DI registrations, middleware, routes, `SpringApplication.run`, `axum Router` | `src/queries.rs` (all 4), new standpoint handling in `src/ir/pipeline.rs` CoreIRPass, renderer |
| P2 | Statement-level **Call/Construct/PropRead/PropWrite/Return** semantic ops in the TS/CS/Rust/Java language layers (span-based, replacing substring `contains`), rendered at Med (and High) | `-6, -7` | **High** — makes `controller →repo →db` and `component →service →HTTP` recoverable per-token-budget | `src/ir/layers/*`, `src/ir/opcodes.rs` (new ops), `src/ir/hierarchical.rs`, `src/ir/render_llm.rs` |
| P3 | Workspace parity: add `.rs`/`.java` to `COMPRESSIBLE_EXTENSIONS`; tests; resolve `.js` (either tree-sitter-javascript grammar or explicit unsupported error) | `-4, -B` | **Medium** | `src/mcp/workspace_util.rs`, `src/compression/language.rs` |
| P4 | Java execution-semantics layer (mirror C#/TS/FWT patterns: `@Transactional`, `JpaRepository`, `RestTemplate/WebClient`, `EntityManager`) | `-7` | Medium — closes the largest cross-language asymmetry | `src/ir/layers/java.rs` |
| P5 | Wire `.NET`/`Spring` graph modules into workspace `graph_pass` (currently dead code) | cross-file | Medium — adds controller→service→repo edges for Cs/Java to match Angular | `src/mcp/workspace.rs`, `src/dotnet_meta/graph.rs`, `src/springing_meta/graph.rs` |
| P6 | `df/cf/se/ec` at **Medium** (not only High) | `-6` | Small — exposes the heuristics that exist today to the default fidelity | `src/ir/render_llm.rs` |
| P7 | Regression tests on **production IR/MCP path** (golden asserts through `render_hierarchical_for_llm`) for Angular/.NET/Spring | every bug | Critical gating | `src/tests/integration/` |

Notes: P0 (meta-feed) is the most cost-effective fix — it resurrects the entire existing meta-layer stack (Angular, Spring, .NET) on the path the LLM actually receives, with zero semantic-depth work. P1 then restores composition wiring, and P2 gives real method-level semantics without emitting bodies.

---

## 11. Direct answer to the audit question

> If an LLM received only Clean-CTX compressed output for a representative C#, Java, Rust, and TS/Angular application, what important architectural facts could it currently NOT reconstruct, and what is the smallest architectural change needed to preserve each?

| Language app | Unreconstructable fact | Smallest change |
|---|---|---|
| C# ASP.NET | Controller identity, HTTP route/verb, `IUserRepository` DI + call, `Program.cs` service registrations/middleware | `find_attribute_inclusive_start` (handles `[`+public) + attribute-inclusive meta feed on IR (P0); `statement.root` for Program.cs (P1) |
| Java/Spring | `@RestController`/`@Service`/`@Autowired`, route mappings, service/repo call chain, bean config | same P0 + C-14; Java exec layer (P4) for DI/calls |
| Rust | `main` router/DB-pool wiring, `#[tokio]`/derives/cfg–encoded behavior, connection/route table | P1 bootstrap capture + extend Rust layer whitelist (P2) |
| TS/Angular | `@Injectable`/`@Component`, `HttpClient` DI, `/api/users` endpoint, component→service call | P0 (IR feed) + P2 `http.get` / `inject()` call ops |
| All | Cross-file controller→service→repo→db, producer→consumer, interface→impl | P5 (wire .NET/.NET/Spring graphs) + P0 meta feed; CBM remains optional targeting (F90) |

The smallest single change with the highest leverage is **P0: a canonical `attribute_inclusive_class_text` helper + unify text/IR meta feeds**. It re-enables all three existing meta-layers on the primary IR path with minimal code movement.



