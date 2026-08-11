// src/tests/angular_meta/signals.rs
//
// Unit tests for the Signals Meta-Layer (Phase 3 of the Angular
// Ecosystem Deepening).

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::signals::{
    expand_phi, expand_phi_in_line, extract_signal_shape, has_signal_imports,
    SignalKind,
};
use crate::compression::Fidelity;

// ── Import gate ────────────────────────────────────────────────────

#[test]
fn detects_signal_imports() {
    let src = "import { signal } from '@angular/core';";
    assert!(has_signal_imports(src));
}

#[test]
fn detects_computed_imports() {
    let src = "import { computed } from '@angular/core';";
    assert!(has_signal_imports(src));
}

#[test]
fn rejects_non_signal_imports() {
    let src = "import { Component } from '@angular/core';";
    assert!(!has_signal_imports(src));
}

// ── Signal extraction ──────────────────────────────────────────────

#[test]
fn extracts_signal_declaration() {
    let src = r#"
import { signal } from '@angular/core';

export class UserComponent {
  count = signal(0);
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 1);
    assert_eq!(shape.signals[0].name, "count");
    assert_eq!(shape.signals[0].kind, SignalKind::Signal);
}

#[test]
fn extracts_signal_with_type_param() {
    let src = r#"
import { signal } from '@angular/core';

export class UserComponent {
  firstName = signal<string>('John');
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 1);
    assert_eq!(shape.signals[0].name, "firstName");
    assert_eq!(shape.signals[0].type_param.as_deref(), Some("string"));
}

#[test]
fn extracts_computed_signal() {
    let src = r#"
import { computed, signal } from '@angular/core';

export class UserComponent {
  firstName = signal('John');
  fullName = computed(() => this.firstName());
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 2);
    assert_eq!(shape.signals[0].kind, SignalKind::Signal);
    assert_eq!(shape.signals[1].kind, SignalKind::Computed);
    assert_eq!(shape.signals[1].name, "fullName");
}

#[test]
fn extracts_effect_registration() {
    let src = r#"
import { effect, signal } from '@angular/core';

export class UserComponent {
  count = signal(0);

  constructor() {
    effect(() => {
      console.log('Count:', this.count());
    });
  }
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 2);
    assert_eq!(shape.signals[0].kind, SignalKind::Signal);
    assert_eq!(shape.signals[1].kind, SignalKind::SignalEffect);
}

// ── NgRx createEffect collision guard (Round-3 audit) ──────────────
//
// The bare `effect(` pattern matches inside `createEffect(`, which would
// produce garbage `Φsig-effect:createEffect` markers in every NgRx effects
// file. The extractor must reject occurrences preceded by an identifier
// character (`createEffect`, `useEffect`, `myEffect`, `obj.effect`).

#[test]
fn does_not_match_ngrx_create_effect() {
    let src = r#"
import { Injectable } from '@angular/core';
import { createEffect, ofType } from '@ngrx/effects';
import { map, switchMap } from 'rxjs/operators';

@Injectable()
export class UserEffects {
  loadUsers$ = createEffect(() =>
    this.actions$.pipe(
      ofType(loadUsers),
      switchMap(() => this.userService.getUsers()),
      map(users => loadUsersSuccess({ users }))
    )
  );
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium);
    // The file has no @angular/core import, so the import gate rejects it.
    // But even if it did, `createEffect(` must not be captured as `effect(`.
    assert!(shape.is_none(), "NgRx effects file must not produce Signals markers");
}

#[test]
fn does_not_match_use_effect_or_method_effect() {
    let src = r#"
import { effect } from '@angular/core';

export class UserComponent {
  constructor() {
    // These are NOT Angular Signals effect() calls.
    useEffect(() => {});
    this.obj.effect(() => {});
    // This one IS a genuine Angular effect().
    effect(() => {});
  }
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    // Only the genuine `effect(` should be captured.
    assert_eq!(shape.signals.len(), 1, "only genuine effect() should be captured");
    assert_eq!(shape.signals[0].kind, SignalKind::SignalEffect);
}

// ── High-fidelity generic rendering (Round-3 audit) ────────────────
//
// `signal<number>()` must render as `signal<number>()`, not `signal()number`.

#[test]
fn high_fidelity_renders_generic_type_param_correctly() {
    let src = r#"
import { signal } from '@angular/core';

export class UserComponent {
  count = signal<number>(0);
}
"#;
    let shape = extract_signal_shape(src, Fidelity::High).expect("should detect signals");
    let rendered = shape.render(Fidelity::High);
    assert!(
        rendered.contains("signal<number>()"),
        "should render signal<number>(), got: {}",
        rendered
    );
    assert!(
        !rendered.contains("signal()number"),
        "must not render signal()number, got: {}",
        rendered
    );
}

// ── Single-colon marker rendering (Round-6 audit) ───────────────────
//
// `marker_prefix()` already includes the trailing `:` (e.g. `"Φsignal:"`).
// The renderer must NOT add a second colon — `Φsignal::count` is a bug.
// Prior audits missed this because the round-trip tests used a
// correctly-formatted single-colon input and the generic-param test only
// checked `contains("signal<number>()")`, which passes despite the broken
// prefix.

#[test]
fn renders_single_colon_markers() {
    let src = r#"
import { signal, computed, effect } from '@angular/core';

export class UserComponent {
  count = signal(0);
  fullName = computed(() => 'x');
  constructor() {
    effect(() => {});
  }
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    let rendered = shape.render(Fidelity::Medium);
    // Every marker line must have exactly one colon after the Φ prefix.
    for line in rendered.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Φ") {
            assert!(
                !trimmed.contains("::"),
                "marker must not contain double colon, got: {}",
                trimmed
            );
        }
    }
    // Spot-check the exact expected forms.
    assert!(rendered.contains("Φsignal:count"), "got: {}", rendered);
    assert!(rendered.contains("Φcomputed:fullName"), "got: {}", rendered);
    assert!(rendered.contains("Φsig-effect:?"), "got: {}", rendered);
}

#[test]
fn low_fidelity_renders_single_colon_markers() {
    let src = r#"
import { signal } from '@angular/core';

export class UserComponent {
  count = signal(0);
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Low).expect("should detect signals");
    let rendered = shape.render(Fidelity::Low);
    assert!(rendered.contains("Φsignal:count"), "got: {}", rendered);
    assert!(!rendered.contains("::"), "got: {}", rendered);
}

#[test]
fn extracts_to_signal_interop() {
    let src = r#"
import { toSignal } from '@angular/core/rxjs-interop';
import { of } from 'rxjs';

export class UserComponent {
  users$ = of([{ id: 1 }]);
  users = toSignal(this.users$, { initialValue: [] });
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 1);
    assert_eq!(shape.signals[0].kind, SignalKind::ToSignal);
    assert_eq!(shape.signals[0].name, "users");
}

#[test]
fn extracts_to_observable_interop() {
    let src = r#"
import { toObservable } from '@angular/core/rxjs-interop';
import { signal } from '@angular/core';

export class UserComponent {
  count = signal(0);
  count$ = toObservable(this.count);
}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium).expect("should detect signals");
    assert_eq!(shape.signals.len(), 2);
    assert_eq!(shape.signals[0].kind, SignalKind::Signal);
    assert_eq!(shape.signals[1].kind, SignalKind::ToObservable);
    assert_eq!(shape.signals[1].name, "count$");
}

// ── No-signal no-op ────────────────────────────────────────────────

#[test]
fn no_signal_imports_produces_none() {
    let src = r#"
import { Component } from '@angular/core';

@Component({ selector: 'app-plain' })
export class PlainComponent {}
"#;
    let shape = extract_signal_shape(src, Fidelity::Medium);
    assert!(shape.is_none(), "non-signal file should return None");
}

// ── Marker round-trip ──────────────────────────────────────────────

#[test]
fn expand_phi_round_trip() {
    assert_eq!(expand_phi("Φsignal"), Some("signal()"));
    assert_eq!(expand_phi("Φcomputed"), Some("computed()"));
    assert_eq!(expand_phi("Φsig-effect"), Some("effect()"));
    assert_eq!(expand_phi("ΦtoSignal"), Some("toSignal()"));
    assert_eq!(expand_phi("ΦtoObservable"), Some("toObservable()"));
    assert_eq!(expand_phi("ΦlinkedSignal"), Some("linkedSignal()"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn expand_phi_in_line_rewrites_signal_markers() {
    let line = "  Φsignal:count";
    let expanded = expand_phi_in_line(line);
    assert!(expanded.contains("signal() count"));
}

#[test]
fn signal_kind_marker_prefixes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for kind in SignalKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        assert!(seen.insert(prefix), "duplicate prefix: {}", prefix);
    }
}