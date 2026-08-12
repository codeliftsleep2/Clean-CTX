# Angular Ecosystem Deepening — Meta-Layer Working Document

> **Status:** ✅ Complete · **Last updated:** 2026-08-12 · **Target:** v0.4.0
> **Effort:** 9–13 days · **Prereqs:** A-11 ✅ · Angular Meta-Layer Phases 1–4 ✅ · DOTNET_META_LAYER.md Phase 2 ✅
> **Roadmap items:** R-23 (NgRx) ✅ · R-24 (RxJS) ✅ · R-25 (Signals + Routing + cross-layer graph) ✅
> **Audit status:** Hardened through Round-5 → Round-11 FAANG audits (Round-11 = comment/string-aware extraction guards). 3,023 tests passing, 0 clippy warnings.
>
> **Post-implementation audit hardening (Round-5 → Round-11):** the four extraction layers were iteratively hardened through FAANG audits. Round-8 centralized all string/depth-aware parsing into `src/meta_util.rs` (no per-layer hand-rolled scanners). Round-9 fixed type-annotated assignment names + false-positive guards. Round-10 added string-aware `@Component` scanning + comment-skip guards. Round-11 added the layer-agnostic `is_inside_comment_or_string` primitive threaded through every scan site, plus the `is_routes_context` gate so a `path:` in an unrelated object literal is not treated as a route. See `docs/CHANGELOG.md` [0.3.0] 2026-08-12.
>

---

## Core Principle

Purely **additive** meta-layers. They never modify existing TS compression output — they append additional `Φ` blocks below the existing compacted class. Existing users see no change; RxJS, NgRx, Signals, and Routing files get enriched semantic output. These four layers are built as **one integrated milestone** because they are tightly coupled in practice: NgRx effects are RxJS observable chains, NgRx selectors use RxJS pipe operators, `@Effects` classes are Angular `@Injectable` services, and modern Angular components mix signals with observable interop.

---

## Decisions Locked

| Question | Decision |
|----------|----------|
| Compiler approach | String-based extraction on existing tree-sitter TS captures. No re-parse. Same as Angular Phases 1–4. |
| Marker approach | `Φ`-prefixed markers, block-scoped. **No new opcodes.** |
| Marker architecture | **Namespaced sub-enums** (`RxJsKind`, `NgRxKind`, `SignalKind`, `RouteKind`) in their own modules, chained into the existing Angular `expand_phi_in_line`. Avoids a 41-variant `PhiLineKind` monolith. |
| Pipe chain marker | **`ΦpipeRx:`** — avoids collision with existing `Φpipe:` (`@Pipe` decorator in `PhiLineKind::Pipe`). |
| `Φeffect:` disambiguation | NgRx `createEffect` → `Φeffect:` (NgRx). Angular Signals `effect()` → `Φsig-effect:` (Signals). No collision. |
| Build order | RxJS → NgRx → Signals → Routing → Cross-layer CBM edges → Config/prompts/tests/docs. Each phase gated by sign-off. |
| Detection gate | Import-based: `rxjs`, `@ngrx/*`, `@angular/core` (signals), `@angular/router` (routing). Zero cost for non-matching files. |
| Fidelity | Three levels per existing system. Low = names. Medium = names + shapes. High = full detail (args, ms values, buffer sizes). |
| Default state | On when `angular` feature enabled. Opt-out per layer via `.clean-ctx.json`. |
| New dependencies | **None.** `tree-sitter-typescript` covers all constructs. |
| Integration point | Called from `angular_meta::run_meta_layer` after existing decorator/bundler/graph passes. Appended as additional `Φ` block sections. |
| CBM cross-layer | **No new tool.** Uses existing `GraphBridge.query_graph()` (Cypher via CBM `query_graph` tool) + `trace_path()`. Single new `GraphBridge` method. Silent-skip when CBM absent. |
| Estimate | **9–13 days** (full scope). Roadmap R-23+R-24 were 6–8 days for RxJS+NgRx alone; deepened scope adds Signals, Routing, config schema, CBM edges. |

---

## Marker Vocabulary

### RxJS — `RxJsKind` (`src/angular_meta/rx.rs`)

| Marker | Expansion | Covers |
|--------|-----------|--------|
| `Φobs:` | Observable field | `Observable<T>` fields, `of()`, `from()`, `interval()`, `timer()`, `fromEvent()` |
| `Φsubject:` | Subject<T> | `Subject`, `BehaviorSubject`, `ReplaySubject`, `AsyncSubject` — initial value for BehaviorSubject |
| `ΦpipeRx:` | pipe() chain | RxJS pipe chain container (renamed — no `@Pipe` collision) |
| `Φmap:` | map operators | `map()`, `mergeMap()`, `switchMap()`, `concatMap()`, `exhaustMap()` — operator name included |
| `Φtap:` | tap() | Side-effect operator |
| `Φfilter:` | filter() | Filter predicate |
| `Φcatch:` | catchError() | Error recovery |
| `Φfinalize:` | finalize() | Cleanup operator |
| `Φdelay:` | delay/debounceTime/throttleTime | ms value at High |
| `Φcombine:` | combineLatest/forkJoin/zip/race | Static combinators |
| `Φshare:` | share/shareReplay | Buffer size at High |
| `Φto:` | firstValueFrom/lastValueFrom/toPromise | Conversion to promise |
| `Φwith:` | withLatestFrom | |
| `Φscan:` | scan/reduce | |
| `Φdistinct:` | distinctUntilChanged | |
| `Φretry:` | retry/retryWhen | Count at High |

> **Collision note:** `Φmap:` also exists in the .NET meta-layer (AutoMapper Profile). Not a real conflict — the language gate guarantees a `.ts` RxJS file never contains .NET markers. `expand_phi_in_line` runs Angular first, so `.ts` gives RxJS semantics; `.cs` skips Angular and gets .NET semantics.

### NgRx — `NgRxKind` (`src/angular_meta/ngrx.rs`)

`Φngrx:` · `Φaction:` · `Φreducer:` · `Φeffect:` · `Φselector:` · `Φentity:` · `Φstore:` · `Φdispatch:` · `Φselect:` — as per original NGRX_RXJS_META_LAYER_PLAN.md.

### Signals — `SignalKind` (`src/angular_meta/signals.rs`) — from DOTNET_META_LAYER.md Phase 2

| Marker | Expansion | Covers |
|--------|-----------|--------|
| `Φsignal:` | signal() | Writable signals with type |
| `Φcomputed:` | computed() | Derived signals — dependency summary |
| `Φsig-effect:` | effect() | Effect registration (disambiguated from NgRx `Φeffect:`) |
| `ΦtoSignal:` | toSignal() | Observable → signal interop |
| `ΦtoObservable:` | toObservable() | Signal → observable interop |
| `ΦlinkedSignal:` | linkedSignal() | Angular 19+ |

### Routing — `RouteKind` (`src/angular_meta/routing.rs`) — from DOTNET_META_LAYER.md Phase 2

| Marker | Expansion | Covers |
|--------|-----------|--------|
| `Φroute:` | Routes | Path/component/loadComponent/loadChildren |
| `Φguard:` | CanActivate/CanLoad/CanDeactivate | Route guards |
| `Φresolver:` | ResolveFn/Resolve<T> | Route resolvers |

---

## Phase 1 — RxJS (2–3 days)

`src/angular_meta/rx.rs` — `RxShape`, `extract_rx_shape()`, `shape.render(fidelity)`.
**Detection:** import gate `from 'rxjs'` / `from 'rxjs/operators'`. Observable fields (`Observable<T>` / `$` suffix), subject instantiations, pipe chains (`.pipe(`), static combinators, creation functions.
**Fidelity:** Low = names only. Medium = + operator sequence. High = + args, ms values, buffer sizes.
**Tests:** `src/tests/angular_meta/rx.rs` — 21 tests. Fixtures in `src/test_files/angular/rx/`.

## Phase 2 — NgRx (2–3 days)

`src/angular_meta/ngrx.rs` — `NgRxShape`, `extract_ngrx_shape()`, `shape.render(fidelity)`.
**Detection:** gate `from '@ngrx/store'` / `@ngrx/effects` / `@ngrx/entity` / `@ngrx/data`. Actions, reducers (incl. `createFeature` inline), effects (source → service → result), selectors, entity adapters, Store DI, dispatch/select sites. NgRx Data `EntityCollectionServiceBase<T>` (emits `Φentity:T (data-layer)`) + `{ dispatch: false }` handling.
**Tests:** `src/tests/angular_meta/ngrx.rs` — 34 tests. Fixtures in `src/test_files/angular/ngrx/`.

## Phase 3 — Signals (1–2 days)

`src/angular_meta/signals.rs` — `SignalShape`, `extract_signal_shape()`, `shape.render(fidelity)`.
**Detection:** gate `@angular/core`. `signal()`, `computed()`, `effect()`, `toSignal()`, `toObservable()`, `linkedSignal()`.
**Cross-ref:** emits `Φsig-effect:` to disambiguate from NgRx `Φeffect:`.
**Tests:** `src/tests/angular_meta/signals.rs` — 19 tests. Fixtures in `src/test_files/angular/signals/`.

## Phase 4 — Routing (1 day)

`src/angular_meta/routing.rs` — `RouteShape`, `extract_route_shape()`, `shape.render(fidelity)`.
**Detection:** gate `@angular/router`. `Routes` arrays, `RouterModule.forRoot/forChild`, lazy `loadComponent`/`loadChildren`, guards, resolvers. Field-order-agnostic parsing.
**Tests:** `src/tests/angular_meta/routing.rs` — 18 tests. Fixtures in `src/test_files/angular/routing/`.

## Phase 5 — Cross-Layer CBM Edges (2 days)

**Integration:** `src/cbm/bridge.rs` — add one method `resolve_cross_language_endpoint(&mut self, method_name) -> Option<String>` using the existing `query_graph` (Cypher) + TTL/disk cache. **No new tool.**

```rust
pub fn resolve_cross_language_endpoint(&mut self, method_name: &str) -> Option<String> {
    if !self.is_available() { return None; } // graceful skip
    let escaped = method_name.replace('\'', "\\'");
    let cypher = format!(
        "MATCH (f:Function) WHERE f.name =~ '(?i).*{escaped}.*' AND f.file_path =~ '.*\\.cs$' RETURN f.name, f.file_path"
    );
    let result = self.query_graph(&cypher);
    // best-match → "UserController.GetAll@α12"; empty → None
}
```

**Edges added to `AngularGraph` (`src/angular_meta/graph.rs`):**

| Edge | From | To | Resolution |
|------|------|----|------------|
| Action → Reducer | `Φaction:loadUsers` | `Φreducer:users` | Local `on(loadUsers)` |
| Action → Effect | `Φaction:loadUsers` | `Φeffect:loadUsers$` | Local `ofType(loadUsers)` |
| Effect → Service | `Φeffect:loadUsers$` | `UserService@α3` | Local service registry |
| Effect → Action | `Φeffect:loadUsers$` | `Φaction:loadUsersSuccess` | Local `map(loadUsersSuccess)` |
| Component → Store | `UserComponent@α7` | `Φngrx:UserFeature` | Local `Store<AppState>` DI |
| Component → Selector | `UserComponent@α7` | `Φselector:selectAllUsers` | Local `store.select(...)` |
| Effect → .NET endpoint | `Φeffect:loadUsers$` | `UserController.GetAll@α12` | **CBM** `resolve_cross_language_endpoint()` — workspace + CBM only |

CBM resolution is best-effort + incremental. Zero candidates → silent skip, no error, no graph line.

**Tests:** `src/tests/angular_meta/graph_ngrx.rs` — 7 tests (incl. CBM-absent graceful skip).

## Phase 6 — Config + Prompts + Tests + Docs + Audit (1–2 days)

**Config (`src/config.rs`):** Extend `MetaLayerConfig` with `rxjs`, `ngrx`, `signals`, `routing` sub-configs — all `Option`/`#[serde(default)]`, backward-compatible. `RxJsConfig.min_pipe_operators` (default 2), `NgRxConfig.include_dispatch_sites/include_select_sites/entity_selectors/cross_layer_cbm`, `SignalsConfig.enabled`, `RoutingConfig.enabled`.

**SYSTEM_PROMPT (`src/mcp/prompts.rs`):** one consolidated "Angular Ecosystem Deepening Meta Markers" section covering all four namespaces.

**Non-regression:** all 2,255 workspace tests pass; `cargo clippy --all-targets -- -D warnings` clean; non-matching `.ts`/`.cs` files byte-identical.

**Test totals:** 99 new tests (RxJS 21, NgRx 34, Signals 19, Routing 18, Graph 7).

**Docs:** this document + `docs/ROADMAP.md` (R-23/R-24 🚧→✅) + `docs/CHANGELOG.md` (test count delta) + `docs/DOTNET_META_LAYER.md` reconciliation note.

---

## Gotchas & Edge Cases

- **`Φpipe:` vs `ΦpipeRx:`** — `Φpipe:` stays `@Pipe`; all RxJS pipe chains use `ΦpipeRx:`. Locked by user decision.
- **`Φeffect:` vs `Φsig-effect:`** — NgRx vs Signals, block-scoped, no collision.
- **`Φmap:` cross-language** — safe due to language gate.
- **NgRx Data `EntityCollectionServiceBase<T>`** — no explicit createAction/createReducer; emit `Φentity:T (data-layer)` noting auto-generated CRUD.
- **Inline `createReducer` in `createFeature`** — handle both standalone and inline.
- **`{ dispatch: false }` effects** — emit `Φeffect:name$ (no-dispatch)`.
- **Barrel imports** — import gate may miss; fallback scan for `createAction(`/`createReducer(`/`createEffect(` presence.
- **Entity adapter `getSelectors()` destructuring** — broader brace/multi-line capture needed.
- **Signal interop naming** — `toSignal`/`toObservable` must not be confused with user functions; mitigated by `@angular/core` gate.
- **Route config shape variance** — field-order-agnostic object-key parsing.

---

## Token Savings Estimate (validated with pilot data post-implementation)

| File type | Raw tokens | Compressed (Medium) | Savings |
|-----------|-----------|--------------------|---------|
| `*.actions.ts` (10 actions) | ~800 | ~120 | ~85% |
| `*.reducer.ts` (complex) | ~600 | ~90 | ~85% |
| `*.effects.ts` (3 effects) | ~700 | ~150 | ~79% |
| `*.selectors.ts` (8 selectors) | ~400 | ~60 | ~85% |
| Full NgRx feature module | ~2,500 | ~420 | ~83% |

---

## Tracking

Each phase ends with: passing suite (`cargo test --workspace --all-targets --all-features`), clean linter (`cargo clippy --all-targets -- -D warnings`), ROADMAP status update, CHANGELOG entry with test-count delta, user sign-off. Next phase does not start until current is signed off.

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/)