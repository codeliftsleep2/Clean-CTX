//! Shared test-local fixtures; production owns all authority and resolution.
pub(super) use super::angular_route_component_support::{call, capture};
use crate::ir::typescript_resolution::{TypeScriptAuthority, arrow_selection::ArrowSelection};
use std::path::Path;
pub(super) fn setup(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = super::angular_route_component_support::setup("");
    let p = root.path();
    std::fs::write(p.join("tsconfig.json"),r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[]}}"#).unwrap();
    for (file, target, alias) in [
        ("lazy-a.ts", "screen-a", "Screen"),
        ("lazy-b.ts", "screen-b", "Screen"),
        ("alias-a.ts", "screen-a", "PublicScreen"),
        ("alias-b.ts", "screen-b", "PublicScreen"),
    ] {
        std::fs::write(
            p.join(file),
            format!("export {{Screen as {alias}}} from './{target}';"),
        )
        .unwrap();
    }
    let (left, right, member) = match form {
        "direct" => ("./screen-a", "./screen-b", "Screen"),
        "reexport" => ("./lazy-a", "./lazy-b", "Screen"),
        _ => ("./alias-a", "./alias-b", "PublicScreen"),
    };
    let a = format!("()=>import('{left}').then(m=>m.{member})");
    let b = format!("()=>import('{right}').then(m=>m.{member})");
    let source = format!(
        "import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Eager}},{{path:'same',loadComponent:{a},providers:[Token]}},{{path:'same',loadComponent:{b}}},{{path:'same',loadComponent:{a}}},{{path:'',loadComponent:{a}}},{{path:'',loadComponent:{b}}},{{path:'parent',children:[{{path:'child',loadComponent:{a}}},{{path:'child',loadComponent:{b}}}]}}];"
    );
    for file in ["one.ts", "two.ts"] {
        std::fs::write(p.join(file), &source).unwrap();
    }
    (root, config)
}
pub(super) fn proofs(a: &TypeScriptAuthority) -> Vec<&ArrowSelection> {
    let proofs: Vec<_> = a
        .arrow_selections
        .iter()
        .filter(|p| p.source == a.source)
        .collect();
    assert_eq!(proofs.len(), 7);
    for p in &proofs {
        assert_eq!(a.arrow_selection_at(p.outer), Some(*p));
    }
    proofs
}
pub(super) fn rewrite(p: &Path, callback: &str) {
    std::fs::write(p.join("one.ts"),format!("import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Eager,providers:[Token]}},{{path:'lazy',loadComponent:{callback}}}];")).unwrap();
}
