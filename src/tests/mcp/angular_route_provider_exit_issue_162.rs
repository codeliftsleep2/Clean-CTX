use super::*;
use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;

#[test]
fn issue_162_route_provider_complete_private_identity_roundtrip_before_public_queries() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "export const routes:NativeRoutes=[{path:'same',providers:[Left,Left]},{path:'same',providers:[Left]},{path:'',providers:[Right]},{path:'',providers:[Right]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config.clone());
    capture(&state, p);
    let proof = |state: &crate::mcp::McpState| {
        let alias = state
            .alias_for_path(&p.join("routes.ts").to_string_lossy())
            .unwrap();
        let snapshot = state.semantic_snapshot(&alias).unwrap();
        let identities: Vec<_> = snapshot
            .edges
            .iter()
            .filter(|edge| edge.relation == SemanticRelation::RegistersProvider)
            .map(|edge| json!([edge.subject.canonical, edge.object.canonical]))
            .collect();
        assert_eq!(identities.len(), 5);
        assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
        json!({"authority":snapshot.authority.typescript_authority,"identities":identities})
    };
    let expected = proof(&state);
    call(
        &state,
        p,
        "save_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    drop(state);
    let db = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let encoded: String = db
        .query_row(
            "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path=?1",
            [p.join("routes.ts").to_string_lossy().into_owned()],
            |row| row.get(0),
        )
        .unwrap();
    let durable: Value = serde_json::from_str(&encoded).unwrap();
    let identities: Vec<_> = durable["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == "RegistersProvider")
        .map(|edge| json!([edge["subject"]["canonical"], edge["object"]["canonical"]]))
        .collect();
    assert_eq!(json!(identities), expected["identities"]);
    drop(db);
    let state = crate::mcp::McpState::new(config);
    call(
        &state,
        p,
        "restore_context",
        json!({"filePath":p.join("routes.ts")}),
    );
    assert_eq!(
        proof(&state),
        expected,
        "complete private identity/authority before any query"
    );
    let owner = state.semantic_owner_path(&p.join("routes.ts").to_string_lossy());
    assert_eq!(
        serde_json::to_value(
            state
                .workspace_index_read()
                .retained_typescript_authority(&owner)
                .unwrap()
        )
        .unwrap(),
        expected["authority"]
    );
    answers(&state, p, "same", 2);
    let result = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":"ProviderToken","name":"Token"}),
    );
    assert_eq!(
        result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == "RegistersProvider" && edge["subject"]["name"] == "")
            .count(),
        2
    );
}

#[test]
fn issue_162_route_provider_alias_namespace_roots_composition_and_malformed_siblings() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("");
    let p = root.path();
    std::fs::write(
        p.join("barrel.ts"),
        "export {GROUP as Published} from './group';",
    )
    .unwrap();
    std::fs::write(p.join("fake-type.ts"), "export type Routes=unknown[];").unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from '@angular/router'; export * from './fake-type';",
    )
    .unwrap();
    let state = crate::mcp::McpState::new(config);
    for (extra, root_type, providers, count) in [
        (
            "import type {Routes as R} from '@angular/router';",
            "R",
            "GROUP",
            2,
        ),
        (
            "import type * as router from '@angular/router';",
            "router.Routes",
            "[...GROUP]",
            2,
        ),
        (
            "import {Published as P} from './barrel';",
            "NativeRoutes",
            "[Left,...P]",
            2,
        ),
        (
            "import * as groups from './barrel';",
            "NativeRoutes",
            "[...[groups.Published]]",
            2,
        ),
        (
            "const local=[Right]; const alias=local;",
            "NativeRoutes",
            "[Left,...alias]",
            2,
        ),
        (
            "",
            "NativeRoutes",
            "[Left,{provide:Right,useClass:Right,useValue:1}]",
            1,
        ),
        (
            "",
            "NativeRoutes",
            "[Left,{provide:Right,provide:Left,useValue:1}]",
            1,
        ),
        (
            "const unsupported=()=>[Right];",
            "NativeRoutes",
            "[Left,...unsupported()]",
            1,
        ),
        (
            "const cycle=[...cycle,Right];",
            "NativeRoutes",
            "[Left,...cycle]",
            2,
        ),
        ("", "NativeRoutes", "[Left,...missing]", 1),
        (
            "import type {Token as OnlyType} from './right';",
            "NativeRoutes",
            "[Left,OnlyType]",
            1,
        ),
        (
            "import type {Routes as Ambiguous} from './ambiguous';",
            "Ambiguous",
            "[Left]",
            0,
        ),
    ] {
        std::fs::write(p.join("routes.ts"),format!("{PREFIX}{extra} export const candidate:{root_type}=[{{path:'same',providers:{providers}}}];")).unwrap();
        answers(&state, p, "same", count);
    }
    // A real root must not grant neighboring objects or imported route arrays authority.
    std::fs::write(
        p.join("route-root.ts"),
        "export const imported=[{path:'same',providers:[]}];",
    )
    .unwrap();
    for body in [
        "const fake={path:'same',providers:[Left]}; export const candidate:NativeRoutes=[];",
        "import {imported} from './route-root'; export const candidate:NativeRoutes=imported;",
        "export const candidate:NativeRoutes=[{...{path:'same',providers:[Left]}}];",
        "type Fake=unknown[]; export const candidate:Fake=[{path:'same',providers:[Left]}];",
    ] {
        std::fs::write(p.join("routes.ts"), format!("{PREFIX}{body}")).unwrap();
        answers(&state, p, "same", 0);
    }
}

#[test]
fn issue_162_route_provider_crossed_object_field_and_private_identity_refusal() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "export const routes:NativeRoutes=[{path:'same',providers:[Left]},{path:'same',providers:[Right]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p);
    let proof = authority(&state, p);
    let edges = crate::angular_meta::route_providers::project(&proof);
    let registrations: Vec<_> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::RegistersProvider)
        .cloned()
        .collect();
    assert_eq!(registrations.len(), 2);
    assert!(proof.admits_relation_evidence(&registrations));
    for mode in 0..7 {
        let mut bad = proof.clone();
        let root_index = bad
            .declaration_facets
            .iter()
            .position(|f| f.declaration.name == "routes")
            .unwrap();
        match mode {
            0 => bad.declaration_facets[root_index].initializer = None,
            1 => {
                let start = bad.declaration_facets[root_index]
                    .initializer
                    .as_ref()
                    .unwrap()
                    .start;
                let duplicate = bad
                    .demanded_occurrences
                    .iter()
                    .find(|s| s.source == bad.source && s.start == start)
                    .unwrap()
                    .clone();
                bad.demanded_occurrences.push(duplicate);
            }
            2 => bad.declaration_facets[root_index]
                .declaration
                .source_hash
                .push('x'),
            _ => {
                let ExpressionValue::Array(objects) = &mut bad.declaration_facets[root_index]
                    .initializer
                    .as_mut()
                    .unwrap()
                    .value
                else {
                    panic!()
                };
                match mode {
                    3 => objects[0].start += 1,
                    4 => {
                        let ExpressionValue::Object(fields) = &objects[1].value else {
                            panic!()
                        };
                        let crossed = fields
                            .iter()
                            .find(|(name, _)| name == "providers")
                            .unwrap()
                            .1
                            .clone();
                        let ExpressionValue::Object(fields) = &mut objects[0].value else {
                            panic!()
                        };
                        fields
                            .iter_mut()
                            .find(|(name, _)| name == "providers")
                            .unwrap()
                            .1 = crossed;
                    }
                    5 => {
                        let ExpressionValue::Object(fields) = &mut objects[0].value else {
                            panic!()
                        };
                        fields.retain(|(name, _)| name != "providers");
                    }
                    _ => objects.swap(0, 1),
                }
            }
        }
        assert!(!bad.admits_relation_evidence(&registrations), "mode={mode}");
    }
    for mode in 0..5 {
        let mut forged = registrations[0].clone();
        let Some(crate::layers::meta::canonical_endpoint::CanonicalEndpoint::RouteOccurrence {
            route_occurrence,
        }) = &mut forged.subject.canonical
        else {
            panic!()
        };
        match mode {
            0 => route_occurrence.source_hash.push('x'),
            1 => route_occurrence.source = p.join("other.ts"),
            2 => route_occurrence.root.name = "other".into(),
            3 => route_occurrence.end += 1,
            _ => {
                forged.object = registrations[1].object.clone();
            }
        }
        assert!(!proof.admits_relation_evidence(&[forged]), "mode={mode}");
    }
}

#[test]
fn issue_162_route_provider_independent_fields_empty_objects_contributor_and_owner_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "export const routes:NativeRoutes=[{path:'same',providers:GROUP},{path:'same',providers:[Left]},{path:'',providers:[Right]},{path:'',providers:[Right]}];";
    let (root, config) = setup(source);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    capture(&state, p);
    answers(&state, p, "same", 3);
    let empty_count = || {
        let result = call(
            &state,
            p,
            "workspace_query",
            json!({"type":"reverse_edges","domain":"angular","entity_type":"ProviderToken","name":"Token"}),
        );
        result["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|e| e["relation"] == "RegistersProvider" && e["subject"]["name"] == "")
            .count()
    };
    assert_eq!(empty_count(), 2);
    let reduced = source.replacen("{path:'',providers:[Right]},", "", 1);
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{reduced}")).unwrap();
    assert_eq!(empty_count(), 1);
    answers(&state, p, "same", 3);
    let reduced = source.replacen("providers:GROUP", "providers:[]", 1);
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{reduced}")).unwrap();
    answers(&state, p, "same", 1);
    assert_eq!(empty_count(), 2);
    std::fs::write(p.join("routes.ts"), format!("{PREFIX}{source}")).unwrap();
    answers(&state, p, "same", 3);
    std::fs::remove_file(p.join("group.ts")).unwrap();
    answers(&state, p, "same", 1);
    assert_eq!(empty_count(), 2);
    std::fs::write(
        p.join("group.ts"),
        GROUP.replace("[Left,{provide:Right,useValue:1}]", "[Right]"),
    )
    .unwrap();
    answers(&state, p, "same", 2);
    std::fs::write(p.join("group.ts"), GROUP).unwrap();
    answers(&state, p, "same", 3);
    std::fs::remove_file(p.join("routes.ts")).unwrap();
    call(&state, p, "refresh_workspace", json!({}));
    answers(&state, p, "same", 0);
}

#[test]
fn issue_162_route_provider_stale_barrel_config_export_and_root_repair() {
    let _serial = crate::protocol::handler_response_serial();
    let source = "import {P} from './barrel'; export const routes:NativeRoutes=[{path:'same',providers:[Left,...P]}];";
    for mode in 0..4 {
        let (root, config) = setup(source);
        let p = root.path();
        std::fs::write(p.join("barrel.ts"), "export {GROUP as P} from './group';").unwrap();
        std::fs::write(p.join("fake-router.ts"), "export type Routes=unknown[];").unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, p);
        answers(&state, p, "same", 2);
        call(
            &state,
            p,
            "save_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        drop(state);
        let (file,replacement,expected)=match mode {
            0=>("barrel.ts","export type {GROUP as P} from './group';".to_owned(),1),
            1=>("tsconfig.json",json!({"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"paths":{"@angular/router":["./fake-router.ts"]}}}).to_string(),0),
            2=>("node_modules/@angular/router/package.json",json!({"name":"@angular/router","version":"20.0.0","exports":{".":null}}).to_string(),0),
            _=>("routes.ts",format!("{PREFIX}export const routes:NativeRoutes=[];"),0),
        };
        let original = std::fs::read(p.join(file)).unwrap();
        std::fs::write(p.join(file), replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        let response = raw(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("routes.ts")}),
        );
        assert!(
            response.get("error").is_some() || response["result"]["isError"] == true,
            "{response}"
        );
        answers(&state, p, "same", expected);
        std::fs::write(p.join(file), original).unwrap();
        if mode == 3 {
            // Editing the root removed this queried identity entirely. Reacquire
            // the known owner explicitly; automatic rediscovery of an absent
            // identity is not the declared-membership authority under test.
            // Dependency-only modes above still repair without owner acquisition.
            capture(&state, p);
        }
        answers(&state, p, "same", 2);
    }
}
