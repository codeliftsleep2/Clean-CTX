use super::*;
use crate::layers::meta::semantic::SemanticRelation;

fn both_positions() -> String {
    SOURCE.replace("})\nexport class", ",viewProviders:[{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}]})\nexport class")
}

fn view_proof(state: &crate::mcp::McpState, file: &str) -> Value {
    let snapshot = state
        .semantic_snapshot(&state.alias_for_path(file).unwrap())
        .unwrap();
    assert!(snapshot.authority.admits_relation_evidence(&snapshot.edges));
    let registrations: Vec<_> = snapshot
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::RegistersProvider | SemanticRelation::RegistersViewProvider
            )
        })
        .map(|edge| {
            assert!(edge.subject.canonical.is_some() && edge.object.canonical.is_some());
            assert_eq!(edge.subject.entity_type, "Component");
            json!([edge.relation, edge.subject.canonical, edge.object.canonical])
        })
        .collect();
    assert_eq!(
        registrations.len(),
        6,
        "three occurrences per position before index deduplication"
    );
    json!({"authority":snapshot.authority.typescript_authority,"registrations":registrations})
}

fn position_answers(
    state: &crate::mcp::McpState,
    p: &Path,
    ordinary: &[&str],
    view: &[&str],
    tokens: &[&str],
) {
    for file in ["app.ts", "neighbor.ts"] {
        if p.join(file).exists() {
            let response = call(
                state,
                p,
                "workspace_query",
                json!({"type":"entities_in_file","file_path":p.join(file),"fidelity":"high"}),
            );
            assert!(response.get("error").is_none(), "{response}");
        }
    }
    let response = call(
        state,
        p,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":"Component","name":"ProviderHost"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    for (relation, owners) in [
        ("RegistersProvider", ordinary),
        ("RegistersViewProvider", view),
    ] {
        let mut actual: Vec<_> = response["result"]["structuredContent"]["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["relation"] == relation)
            .map(|edge| {
                assert_eq!(edge["object"]["name"], "AuditApi");
                (
                    crate::dictionary::path::canonical_identity_key(
                        edge["subject"]["file"].as_str().unwrap(),
                    ),
                    crate::dictionary::path::canonical_identity_key(
                        edge["object"]["file"].as_str().unwrap(),
                    ),
                )
            })
            .collect();
        let mut expected: Vec<_> = owners
            .iter()
            .flat_map(|owner| {
                tokens.iter().map(move |token| {
                    (
                        crate::dictionary::path::canonical_identity_key(
                            &p.join(owner).to_string_lossy(),
                        ),
                        crate::dictionary::path::canonical_identity_key(
                            &p.join(token).to_string_lossy(),
                        ),
                    )
                })
            })
            .collect();
        actual.sort();
        expected.sort();
        assert_eq!(actual, expected, "exact independent {relation} facts");
    }
}

#[test]
fn issue_162_view_provider_lifecycle_sqlite_restore_and_independent_position_retirement() {
    let _serial = crate::protocol::handler_response_serial();
    let source = both_positions();
    let (root, config, _) = setup(&source);
    let p = root.path();
    let database = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let mut expected = Vec::new();
    for file in ["app.ts", "neighbor.ts"] {
        let encoded: String = database
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [p.join(file).to_string_lossy().into_owned()],
                |row| row.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        let registrations: Vec<_> = durable["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| {
                matches!(
                    edge["relation"].as_str(),
                    Some("RegistersProvider" | "RegistersViewProvider")
                )
            })
            .map(|edge| {
                json!([
                    edge["relation"],
                    edge["subject"]["canonical"],
                    edge["object"]["canonical"]
                ])
            })
            .collect();
        assert_eq!(
            registrations.len(),
            6,
            "nonempty complete durable positions"
        );
        expected.push(
            json!({"authority":durable["typescript_authority"],"registrations":registrations}),
        );
    }
    drop(database);
    let state = crate::mcp::McpState::new(config);
    for (file, expected) in ["app.ts", "neighbor.ts"].into_iter().zip(expected) {
        let file = p.join(file).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(
            view_proof(&state, &file),
            expected,
            "complete authority/endpoints before query reacquisition"
        );
        let owner = state.semantic_owner_path(&file);
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
    }
    let files: &[&str] = &["app.ts", "neighbor.ts"];
    let tokens = &["left.ts", "right.ts"];
    position_answers(&state, p, files, files, tokens);
    for (field, ordinary, view) in [
        ("providers", &["neighbor.ts"][..], files),
        ("viewProviders", files, &["neighbor.ts"][..]),
    ] {
        let array = format!(
            "{field}:[{{provide:Left,useValue:1}},{{provide:Right,useValue:2}},{{provide:Left,useValue:3}}]"
        );
        std::fs::write(
            p.join("app.ts"),
            source.replace(&array, &format!("{field}:[]")),
        )
        .unwrap();
        position_answers(&state, p, ordinary, view, tokens);
        std::fs::write(p.join("app.ts"), &source).unwrap();
        position_answers(&state, p, files, files, tokens);
    }
    std::fs::write(p.join("app.ts"), "export class ProviderHost {}").unwrap();
    position_answers(&state, p, &["neighbor.ts"], &["neighbor.ts"], tokens);
    std::fs::remove_file(p.join("neighbor.ts")).unwrap();
    let response = call(&state, p, "refresh_workspace", json!({}));
    assert!(response.get("error").is_none(), "{response}");
    position_answers(&state, p, &[], &[], &[]);
}

#[test]
fn issue_162_view_provider_lifecycle_stale_token_api_package_refusal_and_repair() {
    let _serial = crate::protocol::handler_response_serial();
    for mutation in ["token", "api", "version"] {
        let source = both_positions();
        let (root, config, _) = setup(&source);
        let p = root.path();
        let (file, replacement) = match mutation {
            "token" => (
                p.join("right.ts"),
                "export interface AuditApi {}".to_owned(),
            ),
            "api" => (
                p.join("api.ts"),
                "export function View(metadata:unknown){return metadata;}".to_owned(),
            ),
            _ => (
                p.join("node_modules/@angular/core/package.json"),
                std::fs::read_to_string(p.join("node_modules/@angular/core/package.json"))
                    .unwrap()
                    .replace("20.0.0", "20.0.1"),
            ),
        };
        let original = std::fs::read(&file).unwrap();
        std::fs::write(&file, replacement).unwrap();
        let state = crate::mcp::McpState::new(config);
        for name in ["app.ts", "neighbor.ts"] {
            let file = p.join(name).to_string_lossy().into_owned();
            let response = call(&state, p, "restore_context", json!({"filePath":file}));
            assert!(
                response.get("error").is_some(),
                "stale {mutation} positions adopted: {response}"
            );
            assert!(state.alias_for_path(&file).is_none());
        }
        let tokens: &[&str] = match mutation {
            "token" => &["left.ts"],
            "api" => &[],
            _ => &["left.ts", "right.ts"],
        };
        position_answers(
            &state,
            p,
            &["app.ts", "neighbor.ts"],
            &["app.ts", "neighbor.ts"],
            tokens,
        );
        std::fs::write(file, original).unwrap();
        position_answers(
            &state,
            p,
            &["app.ts", "neighbor.ts"],
            &["app.ts", "neighbor.ts"],
            &["left.ts", "right.ts"],
        );
        for name in ["app.ts", "neighbor.ts"] {
            assert_eq!(std::fs::read_to_string(p.join(name)).unwrap(), source);
        }
    }
}
