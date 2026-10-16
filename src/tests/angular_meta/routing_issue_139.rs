//! Issue #139 structured route ownership and lazy-target regressions.
use crate::angular_meta::routing::extract_route_shape_at;
use crate::compression::Fidelity;
use crate::layers::meta::semantic::SemanticRelation;

#[test]
fn nested_routes_keep_exact_owner_and_full_path() {
    let source = r#"
import { Routes } from '@angular/router';
export const routes: Routes = [
  {
    path: 'admin',
    component: AdminComponent,
    canActivate: [AdminGuard],
    children: [
      { path: 'users', component: UsersComponent, resolve: { users: UsersResolver } },
      { path: '', component: AdminHomeComponent },
      { path: 'settings', component: SettingsComponent }
    ]
  }
];
"#;
    let shape = extract_route_shape_at(source, Fidelity::High, None).unwrap();
    let paths: Vec<_> = shape
        .routes
        .iter()
        .map(|route| route.path.as_str())
        .collect();
    assert_eq!(
        paths,
        vec!["admin", "admin/users", "admin", "admin/settings"]
    );

    let admin = &shape.routes[0];
    assert_eq!(admin.component.as_deref(), Some("AdminComponent"));
    assert_eq!(admin.guards, vec!["AdminGuard"]);
    assert!(admin.resolvers.is_empty());

    let users = &shape.routes[1];
    assert_eq!(users.component.as_deref(), Some("UsersComponent"));
    assert!(users.guards.is_empty());
    assert_eq!(users.resolvers, vec!["UsersResolver"]);
}

#[test]
fn lazy_targets_require_exact_direct_relative_declarations() {
    let root = tempfile::tempdir().unwrap();
    let routes_path = root.path().join("app.routes.ts");
    std::fs::write(
        root.path().join("lazy.component.ts"),
        "import { Component } from '@angular/core'; @Component({selector:'x'}) export class LazyComponent {}",
    )
    .unwrap();
    std::fs::write(
        root.path().join("admin.module.ts"),
        "import { NgModule } from '@angular/core'; @NgModule({}) export class AdminModule {}",
    )
    .unwrap();
    std::fs::write(
        root.path().join("child.routes.ts"),
        "import { Routes } from '@angular/router'; export const childRoutes: Routes = [];",
    )
    .unwrap();
    let source = r#"
import { Routes } from '@angular/router';
export const routes: Routes = [
 { path: 'lazy', loadComponent: () => import('./lazy.component').then(m => m.LazyComponent) },
 { path: 'admin', loadChildren: () => import('./admin.module').then(m => m.AdminModule) },
 { path: 'child', loadChildren: () => import('./child.routes').then(m => m.childRoutes) },
 { path: 'dynamic', loadChildren: chooseLoader() }
];
"#;
    std::fs::write(&routes_path, source).unwrap();
    let shape = extract_route_shape_at(source, Fidelity::High, Some(&routes_path)).unwrap();
    let edges = shape.to_semantic_edges();

    assert!(edges.iter().any(|edge| {
        edge.relation == SemanticRelation::RouteMapsTo
            && edge.subject.name == "lazy"
            && edge.object.entity_type == "Component"
            && edge.object.name == "LazyComponent"
    }));
    assert!(edges.iter().any(|edge| {
        edge.relation == SemanticRelation::LoadsChildren
            && edge.subject.name == "admin"
            && edge.object.entity_type == "Module"
            && edge.object.name == "AdminModule"
    }));
    assert!(edges.iter().any(|edge| {
        edge.relation == SemanticRelation::LoadsChildren
            && edge.subject.name == "child"
            && edge.object.entity_type == "RouteConfig"
            && edge.object.name == "childRoutes"
    }));
    assert!(edges.iter().all(|edge| edge.subject.name != "dynamic"));
}
