//! Registered lazy semantic investigation: neutral proof and native role precede RED.
use super::typescript_children_selection_support::*;
use super::typescript_shared_selection_roles_tests::{selected_role, setup_role};
use serde_json::json;
fn key(p: &std::path::Path) -> String {
    crate::dictionary::path::canonical_identity_key(&p.to_string_lossy())
}
fn question(form: &str, role: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup_role(form, role);
    let p = root.path();
    let state = crate::mcp::McpState::new(config);
    let mut wanted = Vec::new();
    let mut forward_wanted = Vec::new();
    for file in ["one.ts", "two.ts"] {
        let a = capture(&state, p, file);
        let selected = proofs(&a);
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-control"
        );
        assert!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .any(|e| e.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider),
            "stage=native-root-object-provider"
        );
        for (i, proof) in selected.into_iter().enumerate() {
            selected_role(&state, p, &a, &proof.target, role);
            let pair = (key(&a.source), key(&proof.target.source));
            if i != 3 && i != 4 {
                forward_wanted.push(pair.clone());
            }
            wanted.push(pair);
        }
    }
    let forward:Vec<_>=["same","parent/child"].into_iter().map(|name|call(&state,p,"workspace_query",json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name}))).collect();
    let reverse = call(
        &state,
        p,
        "workspace_query",
        json!({"type":"reverse_edges","domain":"angular","entity_type":if role=="module" {"Module"} else {"RouteConfig"},"name":"Screen"}),
    );
    let observed = json!({"forward":forward,"reverse":reverse});
    eprintln!("children-semantic-observed form={form} role={role} public={observed}");
    wanted.sort();
    forward_wanted.sort();
    for reverse in [false, true] {
        let answers = if reverse {
            vec![observed["reverse"].clone()]
        } else {
            observed["forward"].as_array().unwrap().clone()
        };
        let mut actual = answers
            .iter()
            .flat_map(|answer| answer["edges"].as_array().unwrap())
            .filter(|e| e["relation"] == "LoadsChildren" && e["subject"]["name"] != "eager")
            .map(|e| {
                (
                    key(std::path::Path::new(e["subject"]["file"].as_str().unwrap())),
                    key(std::path::Path::new(e["object"]["file"].as_str().unwrap())),
                )
            })
            .collect::<Vec<_>>();
        actual.sort();
        assert_eq!(
            actual,
            if reverse {
                wanted.clone()
            } else {
                forward_wanted.clone()
            },
            "stage=load-children-association direction={} public={observed}",
            if reverse { "reverse" } else { "forward" }
        );
    }
    for (i, name) in ["same", "parent/child"].iter().enumerate() {
        assert_eq!(
            call(
                &state,
                p,
                "workspace_query",
                json!({"type":"forward_edges","domain":"angular","entity_type":"Route","name":name})
            )["edges"],
            observed["forward"][i]["edges"]
        );
    }
}
#[test]
fn issue_162_children_semantic_module_direct() {
    question("direct", "module");
}
#[test]
fn issue_162_children_semantic_module_reexport() {
    question("reexport", "module");
}
#[test]
fn issue_162_children_semantic_module_alias() {
    question("alias", "module");
}
#[test]
fn issue_162_children_semantic_routes_direct() {
    question("direct", "routes");
}
#[test]
fn issue_162_children_semantic_routes_reexport() {
    question("reexport", "routes");
}
#[test]
fn issue_162_children_semantic_routes_alias() {
    question("alias", "routes");
}
