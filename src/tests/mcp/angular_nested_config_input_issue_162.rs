//! Investigation hypotheses only: direct merged inputs and written graph cycles.
use super::angular_bootstrap_inline_controls_tests::{call, capture};
use super::angular_config_composition_controls_tests::setup as base_setup;
use crate::ir::typescript_resolution::declaration_facets::{DeclarationFacet, ExpressionValue};
use crate::ir::typescript_resolution::demanded_occurrences::DemandedValue;
use crate::ir::typescript_resolution::{DeclarationId, Namespace, Resolution, TypeScriptAuthority};
use serde_json::json;
use std::path::Path;
const TYPES: &str = "import type {ApplicationConfig} from '@angular/core'; import {mergeApplicationConfig as combine} from '@angular/core'; import {leafConfig as Base} from './base'; import {leafConfig as Feature} from './feature';";
fn key(p: &Path, file: &str) -> String {
    crate::dictionary::path::canonical_identity_key(&p.join(file).to_string_lossy())
}
pub(super) fn role<'a>(
    authority: &'a TypeScriptAuthority,
    name: &str,
    source: &Path,
) -> &'a DeclarationFacet {
    let matching = authority
        .declaration_facets
        .iter()
        .filter(|f| f.declaration.name == name && f.declaration.source == source)
        .collect::<Vec<_>>();
    assert_eq!(matching.len(), 1, "stage=result-identity; {name}");
    let facet = matching[0];
    let id = &facet.declaration;
    let annotation = facet.type_annotation.as_ref().expect("stage=native-type");
    let initializer = facet.initializer.as_ref().expect("stage=initializer");
    assert!(
        matches!(authority.demanded_at(&id.source,&id.source_hash,id.start,id.end),Some(DemandedValue::Declaration{id:actual,is_const:true,annotation:Some(a),initializer:Some(i),..}) if actual==id&&*a==(annotation.start,annotation.end)&&*i==(initializer.start,initializer.end)),
        "stage=exact-const-type-initializer"
    );
    let ExpressionValue::Reference(Resolution::Resolved(ty)) = &annotation.value else {
        panic!("stage=type-reference")
    };
    assert!(crate::angular_meta::class_role_authority::public_entry_api(
        authority,
        ty,
        "@angular/core",
        "@angular/core",
        "ApplicationConfig",
        Namespace::Type
    ));
    assert!(
        matches!(authority.demanded_at(&id.source,&id.source_hash,annotation.start,annotation.end),Some(DemandedValue::TypeReference(Resolution::Resolved(actual))) if actual==ty)
    );
    let ExpressionValue::Call(callee, args) = &initializer.value else {
        panic!("stage=native-initializer")
    };
    let ExpressionValue::Reference(Resolution::Resolved(api)) = &callee.value else {
        panic!("stage=native-api")
    };
    assert!(crate::angular_meta::class_role_authority::public_entry_api(
        authority,
        api,
        "@angular/core",
        "@angular/core",
        "mergeApplicationConfig",
        Namespace::Value
    ));
    assert_eq!(
        authority.demanded_reference_at(&id.source, &id.source_hash, callee.start, callee.end),
        Some(api)
    );
    assert!(authority.admits_demanded_structure(&id.source, &id.source_hash, initializer));
    for arg in args {
        let ExpressionValue::Reference(Resolution::Resolved(target)) = &arg.value else {
            panic!("stage=ordinary-argument")
        };
        assert_eq!(
            authority.demanded_reference_at(&id.source, &id.source_hash, arg.start, arg.end),
            Some(target)
        );
    }
    facet
}
fn selected_argument(
    authority: &TypeScriptAuthority,
    facet: &DeclarationFacet,
    position: usize,
) -> DeclarationId {
    let ExpressionValue::Call(_, args) = &facet.initializer.as_ref().unwrap().value else {
        panic!()
    };
    let ExpressionValue::Reference(Resolution::Resolved(id)) = &args[position].value else {
        panic!()
    };
    assert_eq!(
        authority.demanded_reference_at(
            &facet.declaration.source,
            &facet.declaration.source_hash,
            args[position].start,
            args[position].end
        ),
        Some(id)
    );
    id.clone()
}
fn rows(
    state: &crate::mcp::McpState,
    p: &Path,
    name: &str,
    reverse: bool,
) -> Vec<(String, String)> {
    let answer = call(
        state,
        p,
        "workspace_query",
        json!({"type":if reverse{"reverse_edges"}else{"forward_edges"},"domain":"angular","entity_type":"ApplicationConfig","name":name}),
    );
    assert!(
        !answer["edges"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["relation"] == "RegistersProvider"
                && (e["subject"]["name"] == "inner" || e["subject"]["name"] == "outer")),
        "no flattened provider authority"
    );
    let mut result = answer["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["relation"] == "IncludesApplicationConfig")
        .map(|e| {
            (
                crate::dictionary::path::canonical_identity_key(
                    e["subject"]["file"].as_str().unwrap(),
                ),
                crate::dictionary::path::canonical_identity_key(
                    e["object"]["file"].as_str().unwrap(),
                ),
            )
        })
        .collect::<Vec<_>>();
    result.sort();
    result
}
fn setup(
    form: &str,
    local: bool,
    repeated: bool,
    standalone: bool,
) -> (tempfile::TempDir, crate::mcp::McpState) {
    let (root, config) = base_setup("Base,Feature");
    let p = root.path();
    std::fs::write(
        p.join("nested-api.ts"),
        "export {mergeApplicationConfig as combine} from '@angular/core';",
    )
    .unwrap();
    for side in ["left", "right"] {
        let inner = format!("{TYPES} export const inner:ApplicationConfig=combine(Base,Feature);");
        std::fs::write(p.join(format!("{side}-inner.ts")), &inner).unwrap();
        std::fs::write(
            p.join(format!("{side}-barrel.ts")),
            format!("export {{inner as PublicInner}} from './{side}-inner';"),
        )
        .unwrap();
        let (api, callee, input, reference) = match form {
            "direct" => (
                "import {mergeApplicationConfig} from '@angular/core';".to_owned(),
                "mergeApplicationConfig",
                format!("import {{inner}} from './{side}-inner';"),
                "inner",
            ),
            "alias" => (
                "import {mergeApplicationConfig as combine} from '@angular/core';".to_owned(),
                "combine",
                format!("import {{inner as Selected}} from './{side}-inner';"),
                "Selected",
            ),
            "barrel" => (
                "import {combine} from './nested-api';".to_owned(),
                "combine",
                format!("import {{PublicInner as Selected}} from './{side}-barrel';"),
                "Selected",
            ),
            "namespace" => (
                "import * as native from '@angular/core';".to_owned(),
                "native.mergeApplicationConfig",
                format!("import * as configs from './{side}-inner';"),
                "configs.inner",
            ),
            _ => panic!(),
        };
        let source = if local {
            format!(
                "{inner} export const outer:ApplicationConfig=combine(inner,{});",
                if repeated { "inner" } else { "Base" }
            )
        } else {
            format!(
                "{api} import type {{ApplicationConfig}} from '@angular/core'; import {{leafConfig as Base}} from './base'; {input} export const outer:ApplicationConfig={callee}({reference},{});",
                if repeated { reference } else { "Base" }
            )
        };
        std::fs::write(p.join(format!("{side}-outer.ts")), source).unwrap();
        if !standalone {
            std::fs::write(p.join(format!("{side}-bootstrap.ts")),format!("import {{bootstrapApplication}} from '@angular/platform-browser'; import {{Root}} from './root'; import {{outer}} from './{side}-outer'; bootstrapApplication(Root,outer);")).unwrap();
        }
    }
    (root, crate::mcp::McpState::new(config))
}
fn run(form: &str, local: bool, repeated: bool, standalone: bool) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, state) = setup(form, local, repeated, standalone);
    let p = root.path();
    // Query the production path cold before any explicit native capture.
    let _cold = rows(&state, p, "outer", false);
    let baseline = capture(&state, p, "result.ts");
    assert_eq!(
        crate::angular_meta::config_composition::project(&baseline).len(),
        2,
        "stage=leaf-config-baseline"
    );
    for side in ["left", "right"] {
        let outer_file = format!("{side}-outer.ts");
        let inner_file = if local {
            outer_file.clone()
        } else {
            format!("{side}-inner.ts")
        };
        let inner_authority = capture(&state, p, &inner_file);
        let inner = role(&inner_authority, "inner", &inner_authority.source);
        let inner_pairs = crate::angular_meta::config_composition::project(&inner_authority)
            .into_iter()
            .filter(|e| e.subject.name == "inner")
            .collect::<Vec<_>>();
        assert_eq!(inner_pairs.len(), 2, "stage=inner-leaf-pairs");
        assert!(inner_authority.admits_relation_evidence(&inner_pairs));
        let authority = capture(&state, p, &outer_file);
        let outer = role(&authority, "outer", &authority.source);
        let target = selected_argument(&authority, outer, 0);
        assert_eq!(
            target, inner.declaration,
            "stage=exact-outer-inner-identity"
        );
        assert_ne!(outer.declaration, target);
        let imported = role(&authority, "inner", &target.source);
        assert_eq!(imported.declaration, target);
        if repeated {
            assert_eq!(selected_argument(&authority, outer, 1), target);
            let ExpressionValue::Call(_, args) = &outer.initializer.as_ref().unwrap().value else {
                panic!()
            };
            assert_ne!(args[0].start, args[1].start);
        }
        if !standalone {
            let boot = capture(&state, p, &format!("{side}-bootstrap.ts"));
            let edges = crate::angular_meta::bootstrap_references::project(&boot);
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e.relation
                        == crate::layers::meta::semantic::SemanticRelation::DeclaresBootstrapRoot)
                    .count(),
                1
            );
            assert_eq!(
                edges
                    .iter()
                    .filter(|e| e.relation
                        == crate::layers::meta::semantic::SemanticRelation::UsesApplicationConfig)
                    .count(),
                1
            );
        }
    }
    let actual = rows(&state, p, "outer", false);
    let mut wanted = Vec::new();
    for side in ["left", "right"] {
        let outer = format!("{side}-outer.ts");
        let inner_file = if local {
            outer.clone()
        } else {
            format!("{side}-inner.ts")
        };
        wanted.push((key(p, &outer), key(p, &inner_file)));
        if !repeated {
            wanted.push((key(p, &outer), key(p, "base.ts")));
        }
    }
    wanted.sort();
    assert_eq!(
        actual, wanted,
        "stage=nested-input; form={form}; local={local}; repeated={repeated}; standalone={standalone}"
    );
    assert_eq!(_cold, actual, "stage=cold-warm-direct-pair-equivalence");
    let reverse = rows(&state, p, "inner", true);
    let expected = wanted
        .iter()
        .filter(|(_, target)| target != &key(p, "base.ts"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(reverse, expected, "stage=reverse-direct-pairs");
    assert_eq!(
        rows(&state, p, "outer", false),
        actual,
        "stage=repeated-query"
    );
}
#[test]
fn issue_162_nested_config_input_direct() {
    run("direct", false, false, false);
}
#[test]
fn issue_162_nested_config_input_alias() {
    run("alias", false, false, false);
}
#[test]
fn issue_162_nested_config_input_barrel() {
    run("barrel", false, false, false);
}
#[test]
fn issue_162_nested_config_input_namespace() {
    run("namespace", false, false, false);
}
#[test]
fn issue_162_nested_config_input_local() {
    run("alias", true, false, false);
}
#[test]
fn issue_162_nested_config_input_repeated() {
    run("barrel", false, true, false);
}
#[test]
fn issue_162_nested_config_input_standalone() {
    run("namespace", false, false, true);
}
