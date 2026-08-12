// src/tests/angular_meta/rx.rs
//
// Unit tests for the RxJS Meta-Layer (Phase 1 of the Angular
// Ecosystem Deepening).

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::rx::{
    expand_phi, expand_phi_in_line, extract_rx_shape, has_rxjs_imports,
    RxJsKind, SubjectKind,
};
use crate::compression::Fidelity;

// ── Import gate ────────────────────────────────────────────────────

#[test]
fn detects_rxjs_imports() {
    let src = "import { Observable } from 'rxjs';";
    assert!(has_rxjs_imports(src));
}

#[test]
fn detects_rxjs_operators_import() {
    let src = "import { map } from 'rxjs/operators';";
    assert!(has_rxjs_imports(src));
}

#[test]
fn rejects_non_rxjs_imports() {
    let src = "import { Component } from '@angular/core';";
    assert!(!has_rxjs_imports(src));
}

// ── Observable detection ───────────────────────────────────────────

#[test]
fn detects_observable_field_with_type_annotation() {
    let src = r#"
import { Observable } from 'rxjs';

export class UserService {
  users$: Observable<User[]>;
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.observables.len(), 1);
    assert_eq!(shape.observables[0].name, "users$");
    assert_eq!(shape.observables[0].type_param.as_deref(), Some("User[]"));
}

#[test]
fn detects_observable_from_http_get() {
    let src = r#"
import { Observable } from 'rxjs';
import { HttpClient } from '@angular/common/http';

export class UserService {
  users$ = this.http.get<User[]>('/api/users');
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.observables.len(), 1);
    assert_eq!(shape.observables[0].name, "users$");
    assert!(shape.observables[0].source.as_deref().unwrap_or("").contains("http.get"));
}

// ── Round-9 audit: type-annotated + assigned observable declaration ──
//
// `users$: Observable<User[]> = this.http.get(...)` has BOTH a type
// annotation AND an assignment. The old `extract_service_call_observable`
// took the last whitespace token of the full LHS (`Observable<User[]>`),
// emitting `Φobs:Observable<User[]>` instead of `Φobs:users$`. The type
// annotation must be stripped before the field name is extracted.

#[test]
fn detects_type_annotated_assigned_observable() {
    let src = r#"
import { Observable } from 'rxjs';
import { HttpClient } from '@angular/common/http';

export class UserService {
  users$: Observable<User[]> = this.http.get<User[]>('/api/users');
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.observables.len(), 1, "observables: {:?}", shape.observables);
    assert_eq!(
        shape.observables[0].name, "users$",
        "type-annotated assignment must extract the field name, got: {}",
        shape.observables[0].name
    );
    assert!(
        !shape.observables[0].name.contains("Observable"),
        "the field name must not be the type annotation, got: {}",
        shape.observables[0].name
    );
    // The source must still be captured.
    assert!(
        shape.observables[0].source.as_deref().unwrap_or("").contains("http.get"),
        "source should be captured, got: {:?}",
        shape.observables[0].source
    );
}

// ── Subject detection ──────────────────────────────────────────────

#[test]
fn detects_plain_subject() {
    let src = r#"
import { Subject } from 'rxjs';

export class UserService {
  refreshTrigger = new Subject<void>();
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.subjects.len(), 1);
    assert_eq!(shape.subjects[0].name, "refreshTrigger");
    assert_eq!(shape.subjects[0].kind, SubjectKind::Subject);
    assert_eq!(shape.subjects[0].type_param.as_deref(), Some("void"));
}

#[test]
fn detects_behavior_subject_with_initial_value() {
    let src = r#"
import { BehaviorSubject } from 'rxjs';

export class UserService {
  selectedUser$ = new BehaviorSubject<User | null>(null);
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.subjects.len(), 1);
    assert_eq!(shape.subjects[0].name, "selectedUser$");
    assert_eq!(shape.subjects[0].kind, SubjectKind::BehaviorSubject);
    assert_eq!(shape.subjects[0].initial_value.as_deref(), Some("null"));
}

#[test]
fn detects_replay_subject() {
    let src = r#"
import { ReplaySubject } from 'rxjs';

export class UserService {
  cache$ = new ReplaySubject<User>(1);
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.subjects.len(), 1);
    assert_eq!(shape.subjects[0].name, "cache$");
    assert_eq!(shape.subjects[0].kind, SubjectKind::ReplaySubject);
    assert_eq!(shape.subjects[0].initial_value.as_deref(), Some("1"));
}

// ── Round-9 audit: type-annotated + assigned subject declaration ─────
//
// `selectedUser$: BehaviorSubject<User | null> = new BehaviorSubject(null)`
// has BOTH a type annotation AND an assignment. The old subject extractor
// took the last non-`=` token of the text before `new` — which for
// `selectedUser$: BehaviorSubject<User | null> = ` landed on `|` (from the
// union type), emitting `Φsubject:|`. The declarator must be isolated
// (split on `=` then `:`) before the field name is extracted.

#[test]
fn detects_type_annotated_assigned_subject() {
    let src = r#"
import { BehaviorSubject } from 'rxjs';

export class UserService {
  selectedUser$: BehaviorSubject<User | null> = new BehaviorSubject<User | null>(null);
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.subjects.len(), 1, "subjects: {:?}", shape.subjects);
    assert_eq!(
        shape.subjects[0].name, "selectedUser$",
        "type-annotated assignment must extract the field name, got: {}",
        shape.subjects[0].name
    );
    assert!(
        !shape.subjects[0].name.contains('|'),
        "the field name must not contain the union type, got: {}",
        shape.subjects[0].name
    );
    // The initial value and type param must still be captured.
    assert_eq!(shape.subjects[0].initial_value.as_deref(), Some("null"));
    assert!(shape.subjects[0].type_param.as_deref().unwrap_or("").contains("User"));
}

// ── Pipe chain extraction ──────────────────────────────────────────

#[test]
fn extracts_pipe_operator_sequence() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { switchMap, map, catchError, shareReplay } from 'rxjs/operators';

export class UserService {
  users$ = this.refreshTrigger.pipe(
    switchMap(() => this.http.get<User[]>('/api/users')),
    map(users => users.sort((a, b) => a.name.localeCompare(b.name))),
    catchError(err => of([])),
    shareReplay(1)
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.pipes.len(), 1);
    assert_eq!(shape.pipes[0].owner, "users$");
    assert_eq!(shape.pipes[0].operators.len(), 4);
    assert_eq!(shape.pipes[0].operators[0].operator_name, "switchMap");
    assert_eq!(shape.pipes[0].operators[1].operator_name, "map");
    assert_eq!(shape.pipes[0].operators[2].operator_name, "catchError");
    assert_eq!(shape.pipes[0].operators[3].operator_name, "shareReplay");
}

// ── Round-3 audit: pipe owner must be the bare field name, not the
// access modifier or type annotation. `private users$: Observable<User[]>`
// must yield owner `users$`, not `private users$` or `users$: Observable`.
#[test]
fn pipe_owner_strips_modifiers_and_type_annotations() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { switchMap, map } from 'rxjs/operators';

export class UserService {
  private users$: Observable<User[]> = this.refreshTrigger.pipe(
    switchMap(() => this.http.get<User[]>('/api/users')),
    map(users => users)
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.pipes.len(), 1);
    assert_eq!(
        shape.pipes[0].owner, "users$",
        "pipe owner must be the bare field name, got: {}",
        shape.pipes[0].owner
    );
    // The owner must also be registered as an observable field.
    assert!(
        shape.observables.iter().any(|o| o.name == "users$"),
        "pipe owner should be registered as an observable"
    );
}

#[test]
fn low_fidelity_emits_names_only() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { switchMap, map } from 'rxjs/operators';

export class UserService {
  users$ = this.refreshTrigger.pipe(
    switchMap(() => this.http.get<User[]>('/api/users')),
    map(users => users)
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Low).expect("should detect RxJS");
    let rendered = shape.render(Fidelity::Low);
    // Low fidelity: field names only, no operator sequence
    assert!(rendered.contains("Φobs:users$"));
    assert!(!rendered.contains("switchMap"));
}

#[test]
fn medium_fidelity_emits_operator_names() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { switchMap, map } from 'rxjs/operators';

export class UserService {
  users$ = this.refreshTrigger.pipe(
    switchMap(() => this.http.get<User[]>('/api/users')),
    map(users => users)
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    let rendered = shape.render(Fidelity::Medium);
    assert!(rendered.contains("ΦpipeRx:users$"));
    assert!(rendered.contains("switchMap"));
    assert!(rendered.contains("map"));
}

#[test]
fn high_fidelity_emits_operator_args() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { debounceTime, map } from 'rxjs/operators';

export class UserService {
  search$ = this.searchTerm$.pipe(
    debounceTime(300),
    map(term => term.trim())
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::High).expect("should detect RxJS");
    let rendered = shape.render(Fidelity::High);
    assert!(rendered.contains("ΦpipeRx:search$"));
    assert!(rendered.contains("debounceTime"));
    assert!(rendered.contains("300"));
}

// ── Combinator detection ───────────────────────────────────────────

#[test]
fn detects_combine_latest() {
    let src = r#"
import { combineLatest } from 'rxjs';

export class UserService {
  combined$ = combineLatest([this.searchTerm$, this.results$]);
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.combinators.len(), 1);
    assert_eq!(shape.combinators[0].name, "combineLatest");
    // The array literal is a single depth-aware argument (commas inside
    // brackets must NOT fragment it).
    assert_eq!(shape.combinators[0].args.len(), 1);
    assert!(shape.combinators[0].args[0].contains("searchTerm$"));
    assert!(shape.combinators[0].args[0].contains("results$"));
}

#[test]
fn detects_fork_join() {
    let src = r#"
import { forkJoin } from 'rxjs';

export class UserService {
  allData$ = forkJoin([this.loadUsers(), this.loadOrders()]);
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.combinators.len(), 1);
    assert_eq!(shape.combinators[0].name, "forkJoin");
    // The array literal is a single depth-aware argument.
    assert_eq!(shape.combinators[0].args.len(), 1);
    assert!(shape.combinators[0].args[0].contains("loadUsers()"));
    assert!(shape.combinators[0].args[0].contains("loadOrders()"));
}

// ── Round-7 audit: string-aware pipe-body scanning ─────────────────
//
// `(`/`)` inside string literals in a pipe body must NOT affect the
// bracket-depth scan that finds the pipe's closing paren, nor the
// operator-splitting scan. The old naive scans would truncate the pipe
// body at a `)` inside a string (e.g. `x.replace('(', ')')`).

#[test]
fn pipe_body_ignores_parens_in_strings() {
    let src = r#"
import { Observable, of } from 'rxjs';
import { map, tap } from 'rxjs/operators';

export class UserService {
  users$ = this.refreshTrigger.pipe(
    map(x => x.replace('(', ')')),
    tap(x => console.log("done)"))
  );
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.pipes.len(), 1, "pipe chain should be detected intact");
    assert_eq!(shape.pipes[0].operators.len(), 2, "operators: {:?}", shape.pipes[0].operators);
    assert_eq!(shape.pipes[0].operators[0].operator_name, "map");
    assert_eq!(shape.pipes[0].operators[1].operator_name, "tap");
}

// ── Round-6 audit: depth-aware combinator argument splitting ───────
//
// A combinator call with an object-literal / nested-array argument
// containing commas (e.g. `combineLatest([a$, b$], { some, options })`)
// must NOT fragment the arguments list. The old naive `body.split(',')`
// would split on the commas inside the nested brackets/braces.

#[test]
fn combinator_args_with_nested_commas() {
    let src = r#"
import { combineLatest } from 'rxjs';

export class UserService {
  combined$ = combineLatest([this.searchTerm$, this.results$], {
    some: 'option',
    other: null,
  });
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium).expect("should detect RxJS");
    assert_eq!(shape.combinators.len(), 1);
    assert_eq!(shape.combinators[0].name, "combineLatest");
    // The two top-level arguments must NOT be fragmented by the commas
    // inside the array literal or the options object.
    assert_eq!(
        shape.combinators[0].args.len(),
        2,
        "combinator args: {:?}",
        shape.combinators[0].args
    );
    assert!(shape.combinators[0].args[0].contains("searchTerm$"));
    assert!(shape.combinators[0].args[0].contains("results$"));
}

// ── No-RxJS no-op ──────────────────────────────────────────────────

#[test]
fn no_rxjs_imports_produces_none() {
    let src = r#"
export class PlainService {
  private data: string[] = [];
  addItem(item: string): void { this.data.push(item); }
}
"#;
    let shape = extract_rx_shape(src, Fidelity::Medium);
    assert!(shape.is_none(), "non-RxJS file should return None");
}

// ── Marker round-trip ──────────────────────────────────────────────

#[test]
fn expand_phi_round_trip() {
    assert_eq!(expand_phi("Φobs"), Some("Observable"));
    assert_eq!(expand_phi("Φsubject"), Some("Subject"));
    assert_eq!(expand_phi("ΦpipeRx"), Some("PipeRx"));
    assert_eq!(expand_phi("Φmap"), Some("Map"));
    assert_eq!(expand_phi("Φtap"), Some("Tap"));
    assert_eq!(expand_phi("Φfilter"), Some("Filter"));
    assert_eq!(expand_phi("Φcatch"), Some("CatchError"));
    assert_eq!(expand_phi("Φfinalize"), Some("Finalize"));
    assert_eq!(expand_phi("Φdelay"), Some("Delay"));
    assert_eq!(expand_phi("Φcombine"), Some("CombineLatest"));
    assert_eq!(expand_phi("Φshare"), Some("Share"));
    assert_eq!(expand_phi("Φto"), Some("FirstValueFrom"));
    assert_eq!(expand_phi("Φwith"), Some("WithLatestFrom"));
    assert_eq!(expand_phi("Φscan"), Some("Scan"));
    assert_eq!(expand_phi("Φdistinct"), Some("DistinctUntilChanged"));
    assert_eq!(expand_phi("Φretry"), Some("Retry"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn expand_phi_in_line_rewrites_rxjs_markers() {
    let line = "  Φobs:users$ → http.get";
    let expanded = expand_phi_in_line(line);
    assert!(expanded.contains("Observable users$"));
}

#[test]
fn rxjs_kind_marker_prefixes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for kind in RxJsKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        assert!(seen.insert(prefix), "duplicate prefix: {}", prefix);
    }
}