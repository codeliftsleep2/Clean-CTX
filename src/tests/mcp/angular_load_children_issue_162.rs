//! Approved loadChildren investigation only; no capture or semantic repair.
use super::typescript_lazy_selection_support::{call, capture, setup};
use crate::ir::typescript_resolution::declaration_facets::{ExpressionFacet, ExpressionValue};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::SemanticRelation;
use serde_json::{Value, json};
fn fields(
    a: &crate::ir::typescript_resolution::TypeScriptAuthority,
    value: &ExpressionFacet,
    seen: &mut usize,
    observed: &Value,
) {
    assert!(
        a.admits_demanded_structure(&a.source, &a.source_hash, value),
        "stage=route-object-structure; {observed}"
    );
    match &value.value {
        ExpressionValue::Array(values) => {
            for v in values {
                fields(a, v, seen, observed);
            }
        }
        ExpressionValue::Object(entries) => {
            // Eager-only control and parent container are separate from lazy objects.
            if entries.iter().any(|(k, _)| k == "component") {
                return;
            }
            if let Some((_, children)) = entries.iter().find(|(k, _)| k == "children") {
                fields(a, children, seen, observed);
                return;
            }
            let values: Vec<_> = entries
                .iter()
                .filter(|(k, _)| k == "loadChildren")
                .collect();
            assert_eq!(
                values.len(),
                1,
                "stage=loadChildren-field-prerequisite object={}..{}; public={observed}",
                value.start,
                value.end
            );
            let callback = &values[0].1;
            assert!(
                value.start <= callback.start
                    && callback.start < callback.end
                    && callback.end <= value.end,
                "stage=field-value-containment"
            );
            assert!(
                a.admits_demanded_structure(&a.source, &a.source_hash, callback),
                "stage=callback-structure-prerequisite callback={callback:?}; public={observed}"
            );
            *seen += 1;
        }
        _ => panic!("stage=route-container-prerequisite {value:?}"),
    }
}

fn question(form: &str, module: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    if module {
        let core = p.join("node_modules/@angular/core/index.d.ts");
        let mut declarations = std::fs::read_to_string(&core).unwrap();
        declarations.push_str("\nexport declare function NgModule(metadata:unknown):any;\n");
        std::fs::write(core, declarations).unwrap();
    }

    let target_name = if module {
        "FeatureModule"
    } else {
        "childRoutes"
    };
    for side in ["a", "b"] {
        let target = if module {
            "import {NgModule} from '@angular/core'; @NgModule({}) export class FeatureModule {}"
                .to_owned()
        } else {
            "import {Routes} from '@angular/router'; import {Screen} from './screen-a'; import {Token} from './left'; export const childRoutes:Routes=[{path:'target',component:Screen,providers:[Token]}];".to_owned()
        };
        std::fs::write(p.join(format!("children-{side}.ts")), target).unwrap();
        std::fs::write(
            p.join(format!("children-api-{side}.ts")),
            format!("export {{{target_name}}} from './children-{side}';"),
        )
        .unwrap();
        std::fs::write(
            p.join(format!("children-alias-{side}.ts")),
            format!("export {{{target_name} as PublicTarget}} from './children-{side}';"),
        )
        .unwrap();
    }
    let public_name = if form == "alias" {
        "PublicTarget"
    } else {
        target_name
    };
    let prefix = match form {
        "direct" => "children",
        "reexport" => "children-api",
        _ => "children-alias",
    };
    let callback = |side| format!("()=>import('./{prefix}-{side}').then(m=>m.{public_name})");
    let left = callback("a");
    let right = callback("b");
    for owner in ["one.ts", "two.ts"] {
        std::fs::write(p.join(owner),format!("import type {{Routes}} from '@angular/router'; import {{Screen as Eager}} from './screen-a'; import {{Token}} from './left'; export const routes:Routes=[{{path:'eager',component:Eager}},{{path:'same',loadChildren:{left},providers:[Token]}},{{path:'same',loadChildren:{right}}},{{path:'same',loadChildren:{left}}},{{path:'',loadChildren:{left}}},{{path:'',loadChildren:{right}}},{{path:'parent',children:[{{path:'child',loadChildren:{left}}},{{path:'child',loadChildren:{right}}}]}}];")).unwrap();
    }
    let state = crate::mcp::McpState::new(config);
    for side in ["a", "b"] {
        let a = capture(&state, p, &format!("children-{side}.ts"));
        if module {
            let id = a
                .declarations
                .iter()
                .find(|id| id.name == target_name)
                .unwrap();
            assert_eq!(
                crate::angular_meta::class_role_authority::declaration_role(&a, id),
                Some(crate::angular_meta::decorators::ClassKind::Module),
                "stage=native-Module-control"
            );
        } else {
            assert_eq!(
                crate::angular_meta::route_providers::project(&a)
                    .iter()
                    .filter(|e| e.relation == SemanticRelation::RegistersProvider)
                    .count(),
                1,
                "stage=native-Routes-target-control"
            );
        }
    }
    let mut authorities = Vec::new();
    for owner in ["one.ts", "two.ts"] {
        let a = capture(&state, p, owner);
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-sibling-control"
        );
        let providers = crate::angular_meta::route_providers::project(&a);
        let registrations: Vec<_> = providers
            .iter()
            .filter(|e| e.relation == SemanticRelation::RegistersProvider)
            .collect();
        assert_eq!(registrations.len(), 1, "stage=provider-control");
        assert!(
            matches!(
                registrations[0].subject.canonical,
                Some(CanonicalEndpoint::RouteOccurrence { .. })
            ),
            "stage=native-root-object-control"
        );
        authorities.push(a);
    }
    let forward:Vec<_> = ["same","parent/child"].into_iter().map(|name|call(&state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":if module {"Module"} else {"RouteConfig"},"name":target_name}),
    );
    let public = json!({"forward":forward,"reverse":reverse});
    eprintln!("load-children-observed form={form} module={module} public={public}");
    for a in &authorities {
        let root = a
            .declaration_facets
            .iter()
            .find(|f| f.declaration.source == a.source && f.declaration.name == "routes")
            .unwrap();
        let mut seen = 0;
        fields(a, root.initializer.as_ref().unwrap(), &mut seen, &public);
        assert_eq!(seen, 7);
    }
    // A future successful field/callback probe still needs selected-export and
    // role-specific association controls before any intended semantic RED claim.
}
#[test]
fn issue_162_load_children_module_direct_prerequisites() {
    question("direct", true);
}
#[test]
fn issue_162_load_children_module_reexport_prerequisites() {
    question("reexport", true);
}
#[test]
fn issue_162_load_children_module_alias_prerequisites() {
    question("alias", true);
}
#[test]
fn issue_162_load_children_routes_direct_prerequisites() {
    question("direct", false);
}
#[test]
fn issue_162_load_children_routes_reexport_prerequisites() {
    question("reexport", false);
}
#[test]
fn issue_162_load_children_routes_alias_prerequisites() {
    question("alias", false);
}
