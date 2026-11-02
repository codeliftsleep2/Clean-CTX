//! Test-local neutral field fixtures and exact existing-carrier inspection.
pub(super) use super::typescript_children_selection_support::{call, capture};
use super::typescript_shared_selection_roles_tests::setup_role;
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority, Witness};
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Element {
    pub start: usize,
    pub end: usize,
    pub reference: Resolution,
}
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Field {
    pub object: (usize, usize),
    pub span: (usize, usize),
    pub elements: Vec<Element>,
}
pub(super) fn setup(form: &str) -> (tempfile::TempDir, crate::config::CleanCtxConfig) {
    let (root, config) = setup_role("direct", "module");
    let p = root.path();
    let router = p.join("node_modules/@angular/router/index.d.ts");
    let mut declarations = std::fs::read_to_string(&router).unwrap();
    // Minimal native fixture type, not a claim about runtime callable eligibility.
    declarations.push_str("\nexport interface Route {canActivate?:unknown[];} export type CanActivateFn=(route:unknown,state:unknown)=>boolean;\n");
    std::fs::write(router, declarations).unwrap();
    for side in ["a", "b"] {
        std::fs::write(p.join(format!("guard-{side}.ts")),"import type {CanActivateFn} from '@angular/router'; export const Gate:CanActivateFn=()=>true;").unwrap();
    }
    std::fs::write(
        p.join("guard-api.ts"),
        "export {Gate as A} from './guard-a'; export {Gate as B} from './guard-b';",
    )
    .unwrap();
    let (imports, a, b) = match form {
        "direct" => (
            "import {Gate} from './guard-a'; import {Gate as Right} from './guard-b';",
            "Gate",
            "Right",
        ),
        "alias" => (
            "import {Gate as A} from './guard-a'; import {Gate as B} from './guard-b';",
            "A",
            "B",
        ),
        "barrel" => ("import {A,B} from './guard-api';", "A", "B"),
        "namespace" => (
            "import * as left from './guard-a'; import * as right from './guard-b';",
            "left.Gate",
            "right.Gate",
        ),
        _ => (
            "function Gate(){return true;} function RightGate(){return false;}",
            "Gate",
            "RightGate",
        ),
    };
    let body = format!(
        "import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; {imports} export const routes:Routes=[{{path:'controls',component:Eager,loadComponent:()=>import('./screen-b').then(m=>m.Screen),loadChildren:()=>import('./feature-a').then(m=>m.Screen),providers:[Token]}},{{path:'same',canActivate:[{a},{b},{a}],providers:[Token]}},{{path:'same',canActivate:[{b}]}},{{path:'same',canActivate:[{a}]}},{{path:'',canActivate:[{a}]}},{{path:'',canActivate:[{b}]}},{{path:'parent',children:[{{path:'child',canActivate:[{a}]}},{{path:'child',canActivate:[{b}]}}]}}];"
    );
    for owner in ["one.ts", "two.ts"] {
        std::fs::write(p.join(owner), &body).unwrap();
    }
    (root, config)
}
fn walk(a: &TypeScriptAuthority, v: &ExpressionFacet, out: &mut Vec<Field>) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, v),
        "stage=neutral-object-structure"
    );
    match &v.value {
        ExpressionValue::Array(values) => {
            for v in values {
                walk(a, v, out);
            }
        }
        ExpressionValue::Object(fields) => {
            for (name, array) in fields {
                if name == "children" {
                    walk(a, array, out);
                } else if name == "canActivate" {
                    assert!(
                        v.start <= array.start && array.start < array.end && array.end <= v.end
                    );
                    assert!(
                        a.admits_demanded_structure(&a.source, &a.source_hash, array),
                        "stage=neutral-guard-array"
                    );
                    let ExpressionValue::Array(values) = &array.value else {
                        panic!("ordinary Array required")
                    };
                    let mut elements = Vec::new();
                    for element in values {
                        assert!(
                            array.start <= element.start
                                && element.start < element.end
                                && element.end <= array.end
                        );
                        let ExpressionValue::Reference(reference) = &element.value else {
                            panic!("stage=neutral-guard-reference {element:?}")
                        };
                        if let Resolution::Resolved(target) = reference {
                            assert_eq!(
                                a.demanded_reference_at(
                                    &a.source,
                                    &a.source_hash,
                                    element.start,
                                    element.end
                                ),
                                Some(target)
                            );
                            assert!(target.source==a.source&&target.source_hash==a.source_hash||a.witnesses.iter().any(|w|matches!(w,Witness::Content{path,hash} if path==&target.source&&hash==&target.source_hash)),"stage=neutral-target-witness");
                        } else {
                            assert!(
                                a.demanded_reference_at(
                                    &a.source,
                                    &a.source_hash,
                                    element.start,
                                    element.end
                                )
                                .is_none()
                            );
                        }
                        elements.push(Element {
                            start: element.start,
                            end: element.end,
                            reference: reference.clone(),
                        });
                    }
                    out.push(Field {
                        object: (v.start, v.end),
                        span: (array.start, array.end),
                        elements,
                    });
                }
            }
        }
        _ => panic!("ordinary Object/Array required"),
    }
}
pub(super) fn retained(a: &TypeScriptAuthority) -> Vec<Field> {
    let root = a
        .declaration_facets
        .iter()
        .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
        .expect("typed source routes facet");
    assert_eq!(root.declaration.source_hash, a.source_hash);
    let mut out = Vec::new();
    walk(a, root.initializer.as_ref().unwrap(), &mut out);
    out
}
