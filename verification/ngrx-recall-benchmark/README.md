# NgRx Recall Replacement Benchmark v1

This is a replacement acceptance benchmark for the unrecoverable historical
#140 repo-scale recall corpus/oracle. It preserves the original acceptance
intent but does not claim to reproduce the original corpus, oracle, or
historical measurements.

## Status

- Benchmark version: `ngrx-recall-benchmark-v1`
- Corpus: `tomastrajan/angular-ngrx-material-starter`
- Commit: `3d95daa95b5a53798f33b390815bdccad733995e`
- License: MIT
- Oracle status: independently source-reviewed and frozen before any scored
  Clean-CTX run
- Oracle SHA-256: `64f30ff5eafaab5da0a28ba3aeed8d444e5d1e0daf43a1b11b56784061b35171`

The authoritative corpus metadata is in `corpus.json`. Expected semantic
truth is in `oracle.json`; `exclusions.json` records representative rejected
candidates. `oracle.schema.json` defines the machine-readable contract and
`scoring.md` freezes comparison and execution semantics.

## Why this corpus

The pinned application is public, long-lived, MIT-licensed, and contains a
non-generated Angular 12 / NgRx 12 application split across core and several
feature areas. Its frozen scope provides at least ten independently reviewed
pairs for every required relation without duplication or synthetic inflation:

| Relation | Expected unique pairs |
|---|---:|
| Dispatches | 20 |
| Selects | 21 |
| HandlesAction | 22 |
| TriggersReducer | 21 |

One corpus is sufficient. Adding another would increase operational and
deduplication complexity without repairing a denominator gap. The corpus was
selected from source inspection only; Clean-CTX was not run against any
candidate and did not influence expected truth.

## Frozen scope

The workspace root is the checked-out repository at the pinned commit. Include
only regular, non-symlink `*.ts` files recursively beneath:

`projects/angular-ngrx-material-starter/src/app`

Exclude `*.spec.ts`, tests outside that root, generated files, dependencies,
`node_modules`, build output, coverage output, archives, fixtures outside the
root, and symlinks. There are 79 included non-test TypeScript files in the
pinned snapshot. Files are interpreted exactly as stored at the commit; no
build, dependency installation, or code generation is part of corpus setup.

## Independent oracle method

1. `rg` candidate discovery found direct Store dispatch/select operations,
   `ofType(...)` handlers, and `createReducer(... on(...))` handlers.
2. Every candidate was traced through its import to the exact declaration in
   the pinned source.
3. The enclosing Angular/NgRx semantic owner was identified from source
   declarations and decorators.
4. Candidates were deduplicated by the frozen semantic-pair identity.
5. A separate second pass re-read the relevant owner, action, selector,
   effect, and reducer source and classified each candidate without consulting
   Clean-CTX output.

The same agent performed both passes, so this is independent from the system
under test but not a two-person review. That limitation is recorded rather
than hidden.

## Supported contract

- `Dispatches`: an Angular Component or Service with authoritative injected
  NgRx Store ownership directly calls `store.dispatch(actionCreator(...))`.
- `Selects`: an Angular Component or Service with authoritative injected NgRx
  Store ownership directly supplies a named `createSelector` or
  `createFeatureSelector` declaration to the RxJS `select(...)` operator.
- `HandlesAction`: a named `createEffect` property receives a named
  `createAction` declaration through `ofType(...)`.
- `TriggersReducer`: a named `createAction` declaration is supplied to
  `on(...)` inside a named `createReducer` declaration.

Directions are authoritative: owner to Action, owner to Selector, Effect to
Action, and Action to Reducer. Modern NgRx forms present naturally in this
corpus are scored. Legacy forms remain covered by tracked synthetic #140
regressions but are absent from this selected real-world snapshot.

## Audit examples

- `AppComponent -> Dispatches -> authLogin`: `app/app.component.ts` calls the
  imported creator at line 79; its declaration is `core/auth/auth.actions.ts:3`.
- `TodosContainerComponent -> Selects -> selectTodos`: the component selects
  the exact imported declaration at line 40.
- `SettingsEffects.persistSettings -> HandlesAction ->
  actionSettingsChangeTheme`: the named effect includes the declaration in its
  `ofType` list at line 68.
- `actionStockMarketRetrieveSuccess -> TriggersReducer -> reducer`: the action
  is an `on(...)` handler input at `stock-market.reducer.ts:23`.

Representative rejections are recorded in `exclusions.json`, including the
non-action `INIT` observable and router events merged into effects.

## Freeze rule

Any change to the corpus commit, scope, expected pairs, exclusions,
normalization, ambiguity policy, or scoring semantics creates a new benchmark
version. Documentation-only corrections that cannot change matching may retain
the version but must preserve the frozen `oracle.json` hash.

Do not modify this oracle during a scored run. A genuine oracle omission
invalidates that run; update, independently review, refreeze as a new version,
and run again under separate authorization.
