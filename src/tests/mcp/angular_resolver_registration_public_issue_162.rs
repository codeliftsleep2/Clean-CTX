//! ANG-R2 public capability RED/GREEN: exact keys, provenance and cold F/R.
//! No new production types are referenced; baseline failure must be behavioral.
use super::typescript_route_guard_field_support::{capture, setup};
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::mcp::McpState;
use serde_json::json;

fn witness(direction: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    let source = "import type {Routes} from '@angular/router'; import {Gate} from './guard-a'; export const routes:Routes=[{path:'dashboard',resolve:{primary:Gate,secondary:Gate}}];";
    std::fs::write(p.join("one.ts"), source).unwrap();
    std::fs::remove_file(p.join("two.ts")).unwrap();

    // Establish retained/native prerequisites in an independent state. The actual
    // query state below has no preseeded owner, index mutation or warm-up query.
    let mut proof_config = config.clone();
    proof_config.persistence.db_path = p.join("prerequisite.db").to_string_lossy().into_owned();
    let proof_state = McpState::new(proof_config);
    let authority = capture(&proof_state, p, "one.ts");
    assert_eq!(authority.resolve_mappings.len(), 2, "stage=neutral-proof");
    let memberships = crate::angular_meta::route_resolvers::project(&authority);
    assert_eq!(memberships.len(), 1, "stage=native-membership-proof");
    let route = match memberships[0].subject.canonical.as_ref().unwrap() {
        CanonicalEndpoint::RouteOccurrence { route_occurrence } => {
            serde_json::to_value(route_occurrence).unwrap()
        }
        other => panic!("stage=route-occurrence-proof {other:?}"),
    };
    let resolver = match memberships[0].object.canonical.as_ref().unwrap() {
        CanonicalEndpoint::Declaration { declaration, .. } => {
            serde_json::to_value(declaration).unwrap()
        }
        other => panic!("stage=resolver-declaration-proof {other:?}"),
    };
    let expected: Vec<_> = authority
        .resolve_mappings
        .iter()
        .map(|mapping| {
            let (_, target) = authority
                .resolve_mapping_at(mapping.pair)
                .expect("stage=exact-mapping-proof");
            assert_eq!(target.name, "Gate");
            json!({
                "route":route, "route_path":"dashboard", "key":mapping.name,
                "resolver":resolver,
                "evidence":{
                    "source":mapping.source, "source_hash":mapping.source_hash,
                    "field_pair":mapping.field_pair, "field_key":mapping.field_key,
                    "object":mapping.object, "pair":mapping.pair,
                    "key":mapping.key, "value":mapping.value
                }
            })
        })
        .collect();
    drop(proof_state);

    assert!(!std::path::Path::new(&config.persistence.db_path).exists());
    let state = McpState::new(config);
    let mut args = json!({
        "type":"resolver_registrations", "direction":direction, "workspaceRoot":p
    });
    if direction == "forward" {
        args["route_path"] = json!("dashboard");
    } else {
        args["resolver_name"] = json!("Gate");
    }
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(16279),
        "workspace_query",
        &json!({"arguments":args}),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("stage=public-response");
    assert!(
        response.get("error").is_none(),
        "stage=public-capability {response:#}"
    );
    assert_ne!(
        response["result"]["isError"], true,
        "stage=public-capability {response:#}"
    );
    let answer = &response["result"]["structuredContent"];
    assert_eq!(answer["format_version"], 1, "{response:#}");
    assert_eq!(answer["direction"], direction, "{response:#}");
    assert_eq!(answer["count"], 2, "{response:#}");
    assert_eq!(answer["registrations"], json!(expected), "{response:#}");
    assert_eq!(answer["semantic_omissions"], json!([]), "{response:#}");
    assert_eq!(
        answer["candidate_source_omissions"],
        json!([]),
        "{response:#}"
    );
    assert_eq!(answer["coverage"]["source_complete"], false);
    assert_eq!(answer["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(answer["coverage"]["omitted_possible"], true);
}

#[test]
fn issue_162_resolver_registration_public_cold_forward() {
    witness("forward");
}

#[test]
fn issue_162_resolver_registration_public_cold_reverse() {
    witness("reverse");
}
