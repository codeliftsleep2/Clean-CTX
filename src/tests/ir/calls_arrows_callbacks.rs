// src/tests/ir/calls_arrows_callbacks.rs
//
// Bound-arrow callable identity: realistic Angular/TypeScript callback shapes.
//
// Sibling of `calls_arrows.rs` (RED-ARROW1..15, 19..23, 25..26), whose
// compilation and assertion helpers this module reuses. The cases here are the
// ones real Angular/RxJS code actually writes — operator chains, object-form
// `subscribe`, Promise chains, timer callbacks, `effect(...)` — plus the nested
// arrow + spread composition.
//
//   RED-ARROW16 — RxJS operator matrix (`subscribe`, `map`, `filter`, `tap`,
//                 `switchMap`, `mergeMap`) inside a recognized callable.
//   RED-ARROW17 — Promise callbacks (`then` / `catch` / `finally`).
//   RED-ARROW18 — `setTimeout` / `queueMicrotask` callbacks.
//   RED-ARROW24 — nested arrows + a spread inner call (no evidence loss).
//   plus the Angular service/component shapes: a callable initializer owns its
//   chain, and a NON-callable initializer (`readonly vm$ = source.pipe(...)`)
//   honestly owns nothing.
#![cfg(feature = "typescript")]

use super::arrow_tests::{
    call_facts, caller_names, calls_by, calls_shape_by, compile_ts, declared_names,
};

// ── RED-ARROW16: the RxJS operator matrix ────────────────────────────

#[test]
fn red_arrow16_rxjs_operator_chain_inside_a_named_method_keeps_the_method() {
    let ir = compile_ts(
        "import { map, filter, tap, switchMap, mergeMap } from 'rxjs/operators'; \
         class Example { \
           load(): void { \
             this.source.pipe( \
               map(x => this.transform(x)), \
               filter(x => this.predicate(x)), \
               tap(x => this.audit(x)), \
               switchMap(x => this.fetchThing(x)), \
               mergeMap(x => this.loadThing(x)) \
             ).subscribe(); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("pipe".to_string(), 5),
            ("map".to_string(), 1),
            ("transform".to_string(), 1),
            ("filter".to_string(), 1),
            ("predicate".to_string(), 1),
            ("tap".to_string(), 1),
            ("audit".to_string(), 1),
            ("switchMap".to_string(), 1),
            ("fetchThing".to_string(), 1),
            ("mergeMap".to_string(), 1),
            ("loadThing".to_string(), 1),
            ("subscribe".to_string(), 0),
        ],
        "the operator callbacks add no caller: every invocation belongs to `load`"
    );
    assert_eq!(
        caller_names(&ir),
        vec!["load".to_string()],
        "`map` / `filter` / `tap` / `switchMap` / `mergeMap` callbacks are never \
         synthetic callers"
    );
}

#[test]
fn red_arrow16_rxjs_operator_chain_inside_a_property_arrow_takes_the_arrow() {
    let ir = compile_ts(
        "class Example { \
           readonly vm$ = () => this.source.pipe( \
             map(x => this.transform(x)), \
             tap(x => this.audit(x)) \
           ); \
         }",
    );
    assert_eq!(
        calls_by(&ir, "vm$"),
        vec![
            ("pipe".to_string(), 2),
            ("map".to_string(), 1),
            ("transform".to_string(), 1),
            ("tap".to_string(), 1),
            ("audit".to_string(), 1),
        ],
        "a property arrow is the enclosing recognized callable here"
    );
    assert_eq!(caller_names(&ir), vec!["vm$".to_string()]);
}

#[test]
fn red_arrow16_object_form_subscribe_inside_a_property_arrow_takes_the_arrow() {
    let ir = compile_ts(
        "class Example { \
           readonly load = () => { \
             this.source.subscribe({ \
               next: tp => this.save(tp), \
               error: err => this.log(err), \
               complete: () => this.cleanup() \
             }); \
           }; \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("subscribe".to_string(), 1),
            ("save".to_string(), 1),
            ("log".to_string(), 1),
            ("cleanup".to_string(), 0),
        ],
        "the property arrow owns the callback-field invocations"
    );
    assert_eq!(caller_names(&ir), vec!["load".to_string()]);
}

// ── RED-ARROW17: Promise callbacks ───────────────────────────────────

#[test]
fn red_arrow17_promise_chain_inside_a_named_method_keeps_the_method() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             this.promise \
               .then(x => this.save(x)) \
               .catch(e => this.log(e)) \
               .finally(() => this.cleanup()); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("then".to_string(), 1),
            ("save".to_string(), 1),
            ("catch".to_string(), 1),
            ("log".to_string(), 1),
            ("finally".to_string(), 1),
            ("cleanup".to_string(), 0),
        ],
        "callback nesting must not lose caller ownership"
    );
}

#[test]
fn red_arrow17_promise_chain_inside_a_property_arrow_takes_the_arrow() {
    let ir = compile_ts(
        "class Example { \
           load = () => { \
             this.promise.then(x => this.save(x)).catch(e => this.log(e)); \
           }; \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("then".to_string(), 1),
            ("save".to_string(), 1),
            ("catch".to_string(), 1),
            ("log".to_string(), 1),
        ]
    );
}

// ─ RED-ARROW18: timer / microtask callbacks ─────────────────────────

#[test]
fn red_arrow18_timer_and_microtask_callbacks_keep_their_enclosing_callable() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             setTimeout(() => this.save(), 100); \
             queueMicrotask(() => this.cleanup()); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("setTimeout".to_string(), 2),
            ("save".to_string(), 0),
            ("queueMicrotask".to_string(), 1),
            ("cleanup".to_string(), 0),
        ],
        "the callback bodies and the scheduling calls are all owned by `load`"
    );

    let property = compile_ts(
        "class Example { \
           load = () => { setTimeout(() => this.save(), 100); }; \
         }",
    );
    assert_eq!(
        calls_by(&property, "load"),
        vec![("setTimeout".to_string(), 2), ("save".to_string(), 0)]
    );
}

// ── RED-ARROW24: nested arrows + spread inner call ───────────────────

#[test]
fn red_arrow24_nested_arrows_with_a_spread_inner_call_lose_no_evidence() {
    let ir = compile_ts(
        "class Example { \
           load = () => { \
             this.source.subscribe(x => { \
               save(...x.args); \
             }); \
           }; \
         }",
    );
    assert_eq!(
        calls_shape_by(&ir, "load"),
        vec![
            ("subscribe".to_string(), 1, false),
            ("save".to_string(), 1, true),
        ],
        "ownership and spread evidence must both survive the nested arrow"
    );
    assert_eq!(
        call_facts(&ir).len(),
        2,
        "no duplicate and no fabricated fact may appear"
    );
    assert_eq!(caller_names(&ir), vec!["load".to_string()]);
}

// ─ Angular/TypeScript shapes: ownership depends on the initializer ──

#[test]
fn angular_effect_callback_inside_a_named_method_keeps_the_method() {
    let ir = compile_ts(
        "class Example { \
           load(): void { \
             effect(() => { \
               this.store.dispatch(loadItems()); \
             }); \
           } \
         }",
    );
    assert_eq!(
        calls_by(&ir, "load"),
        vec![
            ("effect".to_string(), 1),
            ("dispatch".to_string(), 1),
            ("loadItems".to_string(), 0),
        ],
        "an Angular `effect(...)` callback adds no caller"
    );
    assert_eq!(caller_names(&ir), vec!["load".to_string()]);
}

#[test]
fn angular_effect_callback_inside_a_property_arrow_takes_the_arrow() {
    let ir = compile_ts(
        "class Example { \
           readonly syncEffect = () => { \
             effect(() => { \
               this.store.dispatch(loadItems()); \
             }); \
           }; \
         }",
    );
    assert_eq!(
        calls_by(&ir, "syncEffect"),
        vec![
            ("effect".to_string(), 1),
            ("dispatch".to_string(), 1),
            ("loadItems".to_string(), 0),
        ]
    );
}

#[test]
fn angular_readonly_on_click_property_arrow_owns_its_handler_body() {
    let ir = compile_ts(
        "class Example { \
           readonly onClick = () => { \
             this.service.save(); \
           }; \
         }",
    );
    assert_eq!(calls_by(&ir, "onClick"), vec![("save".to_string(), 0)]);
    assert_eq!(caller_names(&ir), vec!["onClick".to_string()]);
}

#[test]
fn angular_non_callable_initializer_honestly_owns_nothing() {
    // `vm$` is NOT an arrow: it is a pipeline expression in a property
    // initializer, so no callable exists to own `pipe` / `map` / `tap`.
    let ir = compile_ts(
        "class Example { \
           readonly vm$ = source.pipe( \
             map(x => transform(x)), \
             tap(x => this.audit(x)) \
           ); \
         }",
    );
    assert!(
        call_facts(&ir).is_empty(),
        "an initializer with no recognized callable must emit no fact: {:?}",
        call_facts(&ir)
    );
    assert!(
        declared_names(&ir).is_empty(),
        "no callable may be invented for a non-arrow initializer"
    );
}
