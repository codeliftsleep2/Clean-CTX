---

## [0.3.0] — 2026-08-12 — Angular Ecosystem Deepening (R-23/R-24/R-25)

### Added

#### RxJS Meta-Layer (R-24) — `src/angular_meta/rx.rs`
- New `RxJsKind` marker namespace (`Φobs:`, `Φsubject:`, `ΦpipeRx:`, `Φmap:`, `Φtap:`, `Φfilter:`, `Φcatch:`, `Φfinalize:`, `Φdelay:`, `Φcombine:`, `Φshare:`, `Φto:`, `Φwith:`, `Φscan:`, `Φdistinct:`, `Φretry:`) with `PhiMarker` impl
- `RxShape` struct + `extract_rx_shape()` — import-gated on `from 'rxjs'` / `rxjs/operators`
- Observable field detection (type annotations, `$` suffix, creation functions, service calls), subject instantiations (`Subject`/`BehaviorSubject`/`ReplaySubject`/`AsyncSubject` with initial value), pipe chains (`.pipe(` with depth/string-aware body capture), static combinators (`combineLatest`/`forkJoin`/`merge`/`zip`/`race`)
- `render(fidelity)` / `render_with_config(fidelity, min_pipe_operators)` — pipe chains suppressed below the configurable operator threshold (default 2)

#### NgRx Meta-Layer (R-23) — `src/angular_meta/ngrx.rs`
- New `NgRxKind` marker namespace (`Φngrx:`, `Φaction:`, `Φreducer:`, `Φeffect:`, `Φselector:`, `Φentity:`, `Φstore:`, `Φdispatch:`, `Φselect:`) with `PhiMarker` impl
- `NgRxShape` struct + `extract_ngrx_shape()` — import-gated on `@ngrx/store|effects|entity|data` with barrel-import fallback
- Action creators, reducers (standalone + inline `createReducer` in `createFeature`), effects (source action → service call → success/failure action, `{ dispatch: false }`), selectors, entity adapters, Store DI, dispatch/select call sites (multi-line + string-aware)
- NgRx Data `EntityCollectionServiceBase<T>` → `Φentity:T (data-layer)` (auto-generated CRUD)
- Cross-layer graph edges (`NgRxEdgeKind`) wired into `AngularGraph` via `to_graph_edges()`: Action→Reducer, Action→Effect, Effect→Service, Effect→Action, Component→Store, Component→Selector

#### Signals Meta-Layer — `src/angular_meta/signals.rs`
- New `SignalKind` marker namespace (`Φsignal:`, `Φcomputed:`, `Φsig-effect:`, `ΦtoSignal:`, `ΦtoObservable:`, `ΦlinkedSignal:`) with `PhiMarker` impl
- `SignalShape` + `extract_signal_shape()` — import-gated on `@angular/core` + signal function usage
- `signal()` / `computed()` / `effect()` / `toSignal()` / `toObservable()` / `linkedSignal()` declarations; `effect()` disambiguated from NgRx `createEffect` (identifier-preceded guard)

#### Routing Meta-Layer — `src/angular_meta/routing.rs`
- New `RouteKind` marker namespace (`Φroute:`, `Φguard:`, `Φresolver:`) with `PhiMarker` impl
- `RouteShape` + `extract_route_shape()` — import-gated on `@angular/router`
- `Routes` arrays, `RouterModule.forRoot/forChild`, lazy `loadComponent`/`loadChildren`, class + function guards, class + function resolvers; field-order-agnostic object-key parsing with escape-aware quoted paths

#### Shared infra
- `src/meta_util.rs` — layer-agnostic string/depth-aware parsing primitives: `split_top_level`, `find_matching_brace`, `find_first_top_level`, `find_enclosing_brace`, `collect_call_body`, `consume_call_expression`, `extract_first_quoted`, `extract_entity_type`, `extract_decl_name`, `is_inside_comment_or_string` (Round-8 structural refactor + Round-11)
- `src/angular_meta/phi.rs` — generic `PhiMarker` trait + `PHI_EXPANDERS` registry (registering a new sub-layer is a 1-line change)
- `src/angular_meta/util.rs` — re-export shim for the meta-layers
- Config sub-layers (`RxJsConfig`, `NgRxConfig`, `SignalsConfig`, `RoutingConfig`) honored via `render_with_config`; `layers/meta/mod.rs` threads them through `run_meta_layer_with_config`

### Fixed

#### Round-7 → Round-11 FAANG audits (the 4 extraction layers)
- **Round-7:** string-aware pipe/brace scans (`collect_call_body`), named effects, multi-line call sites, depth-aware combinator args
- **Round-8:** structural refactor — all string/depth parsing centralized into `src/meta_util.rs`; no per-layer hand-rolled scanners
- **Round-9:** type-annotated assignment names (`users$: Observable<T> = this.http.get(...)` → `Φobs:users$`, not the type token), array-map false-positive guard in effect success-action detection, partial-identifier guard in signals (`= signalName(` ≠ `signal()`)
- **Round-10:** string-aware `@Component` scanner (`find_matching_brace`), comment-skip guards for `path:`/`implements`/`Resolve<`, nearest-class lookup in `class_name_before`, flaky timing-test fix (`audit8_state_new_is_fast`)
- **Round-11:** systematic comment/string-awareness — new `is_inside_comment_or_string` threaded through every scan site (rx, ngrx, signals, routing); `is_routes_context` gate in routing so `path:` in an unrelated object literal is not treated as a route; 16 new regression tests

### Tests
- `src/tests/angular_meta/rx.rs` — 26 tests (observable/subject/combinator/pipe/fidelity/marker round-trip + Round-9/11 regressions)
- `src/tests/angular_meta/ngrx.rs` — 24 tests (actions/reducers/effects/selectors/entity/data-layer/inline-feature/marker round-trip + Round-9/10/11 regressions)
- `src/tests/angular_meta/signals.rs` — 21 tests (signal/computed/effect/toSignal/toObservable/linked/render + Round-9/11 regressions)
- `src/tests/angular_meta/routing.rs` — 23 tests (routes/guards/resolvers/fidelity + Round-10/11 regressions)
- `src/tests/angular_meta/graph_ngrx.rs` — 7 cross-layer graph tests
- `src/tests/angular_meta/util.rs` — 6 new `is_inside_comment_or_string` tests
- Flagged tests: `ngrx` optional-method syntaxes (`OptionalMethod`) and `missing_docs` gates verified

**Verification:** 2,263 tests passing (up from 2,141), 0 clippy warnings under `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

---

