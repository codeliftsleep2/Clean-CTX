// src/tests/angular_meta/routing.rs
//
// Unit tests for the Routing Meta-Layer (Phase 4 of the Angular
// Ecosystem Deepening).

use crate::angular_meta::phi::PhiMarker;
use crate::angular_meta::routing::{
    expand_phi, expand_phi_in_line, extract_route_shape, has_router_imports, RouteKind,
};
use crate::compression::Fidelity;

// ── Import gate ────────────────────────────────────────────────────

#[test]
fn detects_router_imports() {
    let src = "import { Routes } from '@angular/router';";
    assert!(has_router_imports(src));
}

#[test]
fn rejects_non_router_imports() {
    let src = "import { Component } from '@angular/core';";
    assert!(!has_router_imports(src));
}

// ── Route extraction ───────────────────────────────────────────────

#[test]
fn extracts_route_with_component_and_guard() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  {
    path: 'users',
    component: UserListComponent,
    canActivate: [AuthGuard],
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 1);
    let route = &shape.routes[0];
    assert_eq!(route.path, "users");
    assert_eq!(route.component.as_deref(), Some("UserListComponent"));
    assert_eq!(route.guards, vec!["AuthGuard"]);
}

#[test]
fn extracts_route_with_load_component() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  {
    path: 'users/:id',
    loadComponent: () => import('./user-detail.component').then(m => m.UserDetailComponent),
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 1);
    let route = &shape.routes[0];
    assert_eq!(route.path, "users/:id");
    assert_eq!(route.load_component.as_deref(), Some("./user-detail.component"));
}

// ── Round-7 audit: escaped-quote brace scanning ────────────────────
//
// A route path containing an escaped quote (`path: 'user\'s'`) must not
// corrupt the brace-matching scanners. The old naive `in_string = !in_string`
// toggle treated the escaped `'` as a string terminator, breaking the
// enclosing-brace lookup and producing a wrong route object.

#[test]
fn route_with_escaped_quote_in_path() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  {
    path: 'user\'s',
    component: UserProfileComponent,
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 1, "routes: {:?}", shape.routes);
    let route = &shape.routes[0];
    assert_eq!(route.path, "user\\'s", "escaped quote path should be preserved");
    assert_eq!(route.component.as_deref(), Some("UserProfileComponent"));
}

#[test]
fn extracts_route_with_load_children() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  {
    path: 'admin',
    loadChildren: () => import('./admin.routes').then(m => m.adminRoutes),
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 1);
    let route = &shape.routes[0];
    assert_eq!(route.path, "admin");
    assert_eq!(route.load_children.as_deref(), Some("./admin.routes"));
}

// ── Round-10 audit: comment/string `path:` must NOT create phantom routes ──
//
// The route extractor scans globally for `path:` keys. A `path:` inside a
// comment line (e.g. `// path: 'ignored'`) or a template-literal string
// embedded in a route object must be skipped — otherwise it produces a
// phantom route with a bogus path.

#[test]
fn ignores_path_in_comments_and_strings() {
    let src = r#"
import { Routes } from '@angular/router';

// path: 'ignored-comment'
export const appRoutes: Routes = [
  {
    path: 'users',
    component: UserListComponent,
    // path: 'ignored-inner'
    data: { label: 'path: not-a-route' },
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    // Only the real `path: 'users'` should be extracted — no phantom routes
    // from the comment `path:` or the string `'path: not-a-route'`.
    assert_eq!(
        shape.routes.len(), 1,
        "only the real route should be extracted, got: {:?}",
        shape.routes
    );
    assert_eq!(shape.routes[0].path, "users");
}

#[test]
fn extracts_route_with_resolver() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  {
    path: 'users/:id',
    component: UserDetailComponent,
    resolve: { user: UserResolver },
  },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 1);
    let route = &shape.routes[0];
    assert_eq!(route.resolvers, vec!["UserResolver"]);
}

#[test]
fn extracts_multiple_routes() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  { path: '', component: HomeComponent },
  { path: 'users', component: UserListComponent },
  { path: '**', redirectTo: '' },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(shape.routes.len(), 3);
    assert_eq!(shape.routes[0].path, "");
    assert_eq!(shape.routes[1].path, "users");
    assert_eq!(shape.routes[2].path, "**");
}

// ── Round-11 audit: trailing comments, sibling-object strings, and block
// comments must NOT create phantom routes/guards/resolvers ────────────
//
// The Round-10 fix only skipped comment lines whose content BEGAN with
// `//` / `*`. A `path:` in a trailing comment (`{ path: 'x' }, // path: 'y'`),
// a `path:` in an unrelated sibling object (`const menu = { path: '/home' }`),
// or a block-comment `implements` still produced phantom artifacts. The
// shared `is_inside_comment_or_string` + `is_routes_context` guards close
// this defect class.

#[test]
fn ignores_path_in_trailing_comment() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  { path: 'users', component: UserListComponent },  // path: 'ignored-trailing'
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(
        shape.routes.len(), 1,
        "trailing comment path must not duplicate the route, got: {:?}",
        shape.routes
    );
    assert_eq!(shape.routes[0].path, "users");
}

#[test]
fn ignores_path_in_sibling_object_literal() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  { path: 'users', component: UserListComponent },
];

const menuItem = { path: '/home', label: 'Home' };
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    assert_eq!(
        shape.routes.len(), 1,
        "sibling object literal path must not be a route, got: {:?}",
        shape.routes
    );
    assert_eq!(shape.routes[0].path, "users");
}

#[test]
fn ignores_implement_resolve_in_block_comments() {
    let src = r#"
import { Injectable } from '@angular/core';
import { CanActivate, Resolve } from '@angular/router';

/* implements CanActivate — block comment, must NOT be a guard */
/* class BlockedGuard implements CanActivate { } */
/* Resolve<User> — block comment, must NOT be a resolver */

@Injectable({ providedIn: 'root' })
export class AuthGuard implements CanActivate {
  canActivate(): boolean { return true; }
}
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect guards");
    assert_eq!(
        shape.guards.len(), 1,
        "only the real guard should be extracted, got: {:?}",
        shape.guards
    );
    assert_eq!(shape.guards[0].name, "AuthGuard");
    assert!(
        shape.resolvers.is_empty(),
        "block-comment Resolve<User> must not be a resolver, got: {:?}",
        shape.resolvers
    );
}

// ── Round-10 audit: comment `implements`/`Resolve<` must NOT create
// phantom guards/resolvers ────────────────────────────────────────────
//
// The guard/resolver extractors scan for `implements`, `Resolve<`, and
// `ResolveFn`. A match inside a comment line must be skipped.

#[test]
fn ignores_implements_and_resolve_in_comments() {
    let src = r#"
import { Injectable } from '@angular/core';
import { CanActivate, Resolve } from '@angular/router';

// implements CanActivate — commented out, must NOT be a guard
// class OldGuard implements CanActivate { }
// Resolve<User> commented, must NOT be a resolver

@Injectable({ providedIn: 'root' })
export class AuthGuard implements CanActivate {
  canActivate(): boolean { return true; }
}

@Injectable({ providedIn: 'root' })
export class UserResolver implements Resolve<User> {
  resolve(): Observable<User> { return of(null); }
}
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect guards/resolvers");
    // Only the real guard and resolver should be extracted — no phantom
    // entries from the commented-out `implements` / `Resolve<User>`.
    assert_eq!(
        shape.guards.len(), 1,
        "only the real guard should be extracted, got: {:?}",
        shape.guards
    );
    assert_eq!(shape.guards[0].name, "AuthGuard");
    assert_eq!(
        shape.resolvers.len(), 1,
        "only the real resolver should be extracted, got: {:?}",
        shape.resolvers
    );
    assert_eq!(shape.resolvers[0].name, "UserResolver");
}

// ── Guard extraction ───────────────────────────────────────────────

#[test]
fn extracts_class_based_guard() {
    let src = r#"
import { Injectable } from '@angular/core';
import { CanActivate } from '@angular/router';

@Injectable({ providedIn: 'root' })
export class AuthGuard implements CanActivate {
  canActivate(): boolean {
    return true;
  }
}
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect guards");
    assert_eq!(shape.guards.len(), 1);
    assert_eq!(shape.guards[0].name, "AuthGuard");
    assert_eq!(shape.guards[0].kind, "CanActivate");
}

#[test]
fn extracts_function_based_guard() {
    let src = r#"
import { CanActivateFn } from '@angular/router';

export const adminGuard: CanActivateFn = () => {
  return true;
};
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect guards");
    assert_eq!(shape.guards.len(), 1);
    assert_eq!(shape.guards[0].name, "adminGuard");
    assert_eq!(shape.guards[0].kind, "CanActivateFn");
}

// ── Resolver extraction ────────────────────────────────────────────

#[test]
fn extracts_class_based_resolver() {
    let src = r#"
import { Injectable } from '@angular/core';
import { Resolve } from '@angular/router';

@Injectable({ providedIn: 'root' })
export class UserResolver implements Resolve<User> {
  resolve(): Observable<User> {
    return of(null);
  }
}
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect resolvers");
    assert_eq!(shape.resolvers.len(), 1);
    assert_eq!(shape.resolvers[0].name, "UserResolver");
}

#[test]
fn extracts_function_based_resolver() {
    let src = r#"
import { ResolveFn } from '@angular/router';

export const userDetailResolver: ResolveFn<User> = (route) => {
  return route.params['id'];
};
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect resolvers");
    assert_eq!(shape.resolvers.len(), 1);
    assert_eq!(shape.resolvers[0].name, "userDetailResolver");
}

// ── No-router no-op ────────────────────────────────────────────────

#[test]
fn no_router_imports_produces_none() {
    let src = r#"
export class PlainService {
  private items: string[] = [];
}
"#;
    let shape = extract_route_shape(src, Fidelity::Medium);
    assert!(shape.is_none(), "non-router file should return None");
}

// ── Fidelity behavior ──────────────────────────────────────────────

#[test]
fn low_fidelity_emits_paths_only() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  { path: 'users', component: UserListComponent, canActivate: [AuthGuard] },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Low).expect("should detect routes");
    let rendered = shape.render(Fidelity::Low);
    assert!(rendered.contains("Φroute:users"));
    assert!(!rendered.contains("component="), "Low fidelity should not include component");
    assert!(!rendered.contains("guards="), "Low fidelity should not include guards");
}

#[test]
fn medium_fidelity_emits_route_details() {
    let src = r#"
import { Routes } from '@angular/router';

export const appRoutes: Routes = [
  { path: 'users', component: UserListComponent, canActivate: [AuthGuard] },
];
"#;
    let shape = extract_route_shape(src, Fidelity::Medium).expect("should detect routes");
    let rendered = shape.render(Fidelity::Medium);
    assert!(rendered.contains("Φroute:users"));
    assert!(rendered.contains("component=UserListComponent"));
    assert!(rendered.contains("guards=[AuthGuard]"));
}

// ── Marker round-trip ──────────────────────────────────────────────

#[test]
fn expand_phi_round_trip() {
    assert_eq!(expand_phi("Φroute"), Some("Route"));
    assert_eq!(expand_phi("Φguard"), Some("Guard"));
    assert_eq!(expand_phi("Φresolver"), Some("Resolver"));
    assert_eq!(expand_phi("Φunknown"), None);
}

#[test]
fn expand_phi_in_line_rewrites_route_markers() {
    let line = "  Φroute:users component=UserListComponent";
    let expanded = expand_phi_in_line(line);
    assert!(expanded.contains("Route users"));
}

#[test]
fn route_kind_marker_prefixes_are_unique() {
    let mut seen = std::collections::HashSet::new();
    for kind in RouteKind::all_in_expand_order() {
        let prefix = kind.marker_prefix();
        assert!(seen.insert(prefix), "duplicate prefix: {}", prefix);
    }
}