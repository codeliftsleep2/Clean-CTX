# Angular Ecosystem Deepening Plan

> **Status:** 📋 proposed · **Last updated:** 2026-07-01
>
> **Target:** Phase 2 of the .NET + Angular pilot — 4-6 days
>
> **Prerequisite:** Angular Meta-Layer (Phase 1-3) — ✅ complete

---

## Decisions Locked

| Question | Decision |
|----------|----------|
| Compiler approach | String-based extraction on tree-sitter TS captures (same strategy as existing Angular Meta-Layer). No re-parse of AST. |
| Marker approach | `Φ`-prefixed markers, no new opcodes. Extends existing `PhiLineKind` enum. |
| Default state | On, opt-out via `.clean-ctx.json`. Non-Angular files pay zero overhead. |
| Workspace scope | Per-file markers work in both modes. Cross-file graph edges are workspace-only. |
| New dependencies | None. Uses existing `tree-sitter-typescript` grammar. |
| Integration | New subsystems are called from `angular_meta::run_meta_layer` as additional Φ block sections. |

---

## New Φ Markers

### RxJS

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φsubject:` | `Subject<T>` | Subject, BehaviorSubject, ReplaySubject, AsyncSubject |
| `Φobs:` | `Observable<T>` | Observable declarations and creations (`of()`, `from()`, `interval()`, `timer()`) |
| `Φpipe:` | `pipe()` | Pipe chains (main operator sequence) |
| `Φmap:` | `map()` | `map()`, `mergeMap()`, `switchMap()`, `concatMap()`, `exhaustMap()` |
| `Φtap:` | `tap()` | Side effects (`tap()`) |
| `Φfilter:` | `filter()` | `filter()` |
| `Φcatch:` | `catchError()` | `catchError()` |
| `Φfinalize:` | `finalize()` | `finalize()` |
| `Φdelay:` | `delay()` | `delay()`, `debounceTime()`, `throttleTime()` |
| `Φcombine:` | `combineLatest()` | `combineLatest()`, `forkJoin()`, `zip()`, `race()` |
| `Φshare:` | `share()` | `share()`, `shareReplay()` |
| `Φto:` | `firstValueFrom()` | `toPromise()` (legacy), `firstValueFrom()`, `lastValueFrom()` |
| `Φwith:` | `withLatestFrom()` | `withLatestFrom()` |
| `Φscan:` | `scan()` | `scan()`, `reduce()` |
| `Φdistinct:` | `distinctUntilChanged()` | `distinctUntilChanged()` |
| `Φretry:` | `retry()` | `retry()`, `retryWhen()` |

### NgRx

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φngrx:` | NgRx store | Store class / feature state |
| `Φaction:` | `createAction` | Action definitions with props |
| `Φreducer:` | `createReducer` | Reducer shape (state transitions) |
| `Φeffect:` | `createEffect` | Side effects with service dependencies |
| `Φselector:` | `createSelector` | Selector definitions |
| `Φentity:` | `createEntityAdapter` | Entity adapter configuration |

### Signals (beyond Phase 2.5)

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φsignal:` | `signal()` / `WritableSignal` | Signal declarations |
| `Φcomputed:` | `computed()` | Computed signal declarations |
| `Φeffect:` | `effect()` | Effect registrations |

### PrimeNG

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φp-table:` | `p-table` | Table, column, data binding, lazy loading, sorting, pagination |
| `Φp-dropdown:` | `p-dropdown` | Dropdown, multi-select, filtering |
| `Φp-dialog:` | `p-dialog` | Dialog, dynamic dialogs, modal patterns |
| `Φp-form:` | `p-input` / `p-textarea` / `p-calendar` | Input, textarea, calendar, checkbox, radio, toggle, etc. |
| `Φp-panel:` | `p-panel` | Panel, accordion, tabview, fieldset |
| `Φp-chart:` | `p-chart` | Charts (bar, line, pie, etc.) |
| `Φp-tree:` | `p-tree` | Tree, tree table |
| `Φp-menu:` | `p-menu` | Menu, menubar, context menu, tiered menu |
| `Φp-lazy:` | `[lazy]` | Lazy loading (`[lazy]="true"`, `onLazyLoad` event) |
| `Φp-template:` | `ng-template` | Template usage (header, footer, body, expansion) |
| `Φp-state:` | Component state | Selection, expansion, editing state management |
| `Φp-event:` | Event handlers | `onRowSelect`, `onSubmit`, `onPage`, etc. |

### Routing

| Marker | Expansion | Description |
|--------|-----------|-------------|
| `Φroute:` | `Routes` | Route configuration |
| `Φguard:` | `CanActivate` / `CanDeactivate` | Route guards |
| `Φresolver:` | `Resolve<T>` | Route resolvers |
---

## Example Output

```
Φngrx:UserStore
  Φaction:loadUsers, loadUsersSuccess, loadUsersFailure
  Φreducer:users → loading|loaded|error
  Φeffect:loadUsers$ → UserService.getUsers()
  Φselector:selectAllUsers, selectUserById

Φrx:UserService
  Φobs:users$ = this.http.get<User[]>(...)
  Φsubject:refreshTrigger = new Subject<void>()
  Φpipe:users$ = this.refreshTrigger.pipe(
    ΦswitchMap:switchMap(() => this.loadUsers()),
    Φmap:map(users => users.sort(...)),
    Φtap:tap(users => console.log(...)),
    Φcatch:catchError(...)
  )

Φsignal:UserComponent
  Φcomputed:fullName
  Φeffect:logUserChanges

Φp-table:UsersTable
  Φp-column: id, name, email, status, actions
  Φp-lazy: lazy loading + pagination
  Φp-template: row expansion template with nested data

Φp-dialog:UserEditDialog
  Φp-form: reactive form with p-input, p-calendar, p-dropdown
  Φp-event: onSave, onCancel

Φroute:AppModule
  Φroute:/users → loadChildren: () => import('./users/users.module')
  Φroute:/users/:id → UserDetailComponent
  Φguard:AuthGuard → CanActivate
```

---

## Module Structure

### New files in `src/angular_meta/`

| File | Purpose |
|------|---------|
| `rx.rs` | Observable/subject/operator detection |
| `ngrx.rs` | NgRx action/reducer/effect/selector/entity detection |
| `signals.rs` | Deep signal analysis (computed, effect, interop) |
| `routing.rs` | Angular Router route config extraction |
| `primeng.rs` | PrimeNG component tag + attribute detection in templates |

### Modifications to existing files

| File | Purpose |
|------|---------|
| `src/angular_meta/mod.rs` | Call new subsystems from `run_meta_layer` |
| `src/angular_meta/markers.rs` | Add new `PhiLineKind` variants + `build_*` functions |
| `src/angular_meta/graph.rs` | Add NgRx store → selector → effect edges, router → component edges, PrimeNG component → service/data flow |
| `src/angular_meta/detect.rs` | Extend detection for NgRx imports (`@ngrx/store`) and PrimeNG imports (`primeng/`) |
| `src/angular_meta/template.rs` | Add `primeng_tags` field to `TemplateShape` for PrimeNG-specific tag extraction |

### Test files

| File | Purpose |
|------|---------|
| `src/tests/angular_meta/rx.rs` | RxJS extraction tests |
| `src/tests/angular_meta/ngrx.rs` | NgRx extraction tests |
| `src/tests/angular_meta/signals.rs` | Signal extraction tests |
| `src/tests/angular_meta/routing.rs` | Router extraction tests |
| `src/tests/angular_meta/primeng.rs` | PrimeNG extraction tests |

---

## Detection Strategy

### RxJS Detection

Scan for:
- `Observable<T>` / `Subject<T>` / `BehaviorSubject<T>` / `ReplaySubject<T>` type annotations on class fields
- `new Subject<T>()` / `new BehaviorSubject<T>(initial)` / `new ReplaySubject<T>(n)` constructor calls
- `pipe(op1(), op2(), ...)` chains with operator names
- `combineLatest({a, b})`, `forkJoin({a, b})`, `merge(a, b)` static combinators
- `of(value)`, `from(promise)`, `fromEvent(element, event)` creation functions

### NgRx Detection

Scan for:
- `createAction('[Feature] Event', props<...>())` action definitions
- `createReducer(initialState, on(action, ...))` reducer definitions
- `createEffect(() => ...)` effect definitions with `@Injectable()` services
- `createSelector(selectFoo, selectBar, (foo, bar) => ...)` selector definitions
- `createEntityAdapter<T>({ selectId, sortComparer })` entity adapter config
- `Store` DI in constructor: `private store: Store<AppState>`
- `this.store.dispatch(action())` and `this.store.select(selector)` usage

### Signal Detection (beyond Phase 2.5)

Scan for:
- `signal<T>(initialValue)` — already detected in Phase 2.5
- `computed(() => ...)` — new: computed signal declarations
- `effect(() => { ... })` — new: effect registrations (with cleanup)
- `toSignal(observable$)` — new: observable-to-signal interop
- `toObservable(signal)` — new: signal-to-observable interop
- `linkedSignal({ source: ..., computation: ... })` — Angular 18+ linked signals

### Router Detection

Scan for:
- `Routes` / `Route[]` type annotations
- `{ path: '...', component: ... }` route objects
- `loadChildren: () => import('...')` lazy loading
- `loadComponent: () => import('...')` lazy component loading
- `canActivate: [Guard]`, `canDeactivate: [Guard]` guard references
- `resolve: { data: Resolver }` resolver references
- `RouterModule.forRoot(routes)` / `provideRouter(routes)` registration

### PrimeNG Detection

Scan for (in template HTML extracted via tree-sitter):
- `p-table`, `p-column`, `p-columnGroup` — table components
- `p-dropdown` — dropdown components
- `p-dialog` — dialog components
- `p-inputText`, `p-inputNumber`, `p-textarea`, `p-calendar`, `p-checkbox`, `p-radioButton`, `p-inputSwitch`, `p-inputMask`, `p-password` — form components
- `p-panel`, `p-accordion`, `p-accordionTab`, `p-tabView`, `p-tabPanel`, `p-fieldset` — panel components
- `p-chart` — chart components
- `p-tree`, `p-treeTable` — tree components
- `p-menu`, `p-menubar`, `p-contextMenu`, `p-tieredMenu`, `p-slideMenu` — menu components
- `[lazy]` attribute with `"true"` — lazy loading flag
- `(onLazyLoad)` — lazy load event
- `ng-template` with `pTemplate="header|footer|body|expansion|summary"` — PrimeNG template slots
- `[(selection)]`, `[(expandedRows)]`, `[(expandedKeys)]` — PrimeNG state bindings
- `(onRowSelect)`, `(onSubmit)`, `(onPage)`, `(onSort)`, `(onFilter)`, `(onCellSelect)` — common PrimeNG events
- `primeng/` or `primeng/api` imports in component TS files

---

## Fidelity Levels (updated)

| Fidelity | RxJS | NgRx | Signals | Routing | PrimeNG |
|----------|------|------|---------|---------|---------|
| Low | Observable names only | Store class + action names | Signal names only | Route paths only | Component tag names only |
| Medium | + Subject types + pipe operators | + Reducer shape + selectors | + Computed signals | + Lazy loading + guards | + Column lists, lazy flags, template slots |
| High | + All operators + combinators | + Effects + entity config | + Effects + interop | + Resolvers + full config | + Events, state bindings, full attribute details |

---

## Cross-Layer Integration (Phase 3)

The Angular ecosystem markers will be linked to the .NET backend via CBM:

- **NgRx effects** → backend API endpoints (e.g., `loadUsers$` → `GET /api/users`)
- **Angular services** → .NET controllers (e.g., `UserService.getUsers()` → `UserController.GetAll()`)
- **Route paths** → controller route templates (e.g., `/users/:id` → `[Route("api/users/{id}")]`)

This cross-layer graph is built during the Phase 3 integration pass and requires CBM to be available.

---

## Completion Criteria

You will know Phase 2 is complete when **all** of the following are true:

**Functional**
- A file with `Observable<User>` + `pipe(map, switchMap)` produces `Φrx:` + `Φpipe:` markers.
- A file with `createAction`, `createReducer`, `createEffect`, `createSelector` produces `Φngrx:` + `Φaction:` + `Φreducer:` + `Φeffect:` + `Φselector:` markers.
- A file with `computed()` and `effect()` produces `Φcomputed:` and `Φeffect:` markers.
- A file with `Routes` config produces `Φroute:` markers with paths, lazy loading, guards.

**Non-regression**
- A non-Angular `.ts` file produces **zero** new Φ markers.
- All existing Angular Meta-Layer tests still pass.
- `cargo clippy --all-targets -- -D warnings` is clean.

**Tests**
- New unit tests: RxJS extraction, NgRx extraction, Signal extraction, Router extraction.
- At least 4 new test files.
- All tests pass.

---

## License

[CC0-1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) — Dedicated to the public domain.