use super::view_lifecycle_cases::{position_answers_as, view_proof};
use super::*;

const GROUP: &str = "import {AuditApi as Left} from './left'; import {AuditApi as Right} from './right'; export const GROUP=[{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}];";
const FILES: &[(&str, &str)] = &[
    ("group.ts", GROUP),
    (
        "alias.ts",
        "import {GROUP} from './group'; export const ALIAS=GROUP;",
    ),
    (
        "barrel.ts",
        "export {GROUP} from './group'; export {ALIAS} from './alias';",
    ),
];

fn source() -> String {
    SOURCE.replace("@View", "import {GROUP,ALIAS} from './barrel'; @View")
        .replace("providers:[{provide:Left,useValue:1},{provide:Right,useValue:2},{provide:Left,useValue:3}]", "providers:GROUP,viewProviders:[...[ALIAS]]")
}

#[test]
fn issue_162_provider_imported_composition_lifecycle_complete_sqlite_restore_and_retirement() {
    complete_restore("Component");
}

#[test]
fn issue_162_provider_owner_composition_lifecycle_complete_sqlite_restore_and_retirement() {
    for api in ["Directive", "NgModule"] {
        complete_restore(api);
    }
}

fn owner_answers(
    state: &crate::mcp::McpState,
    p: &Path,
    api: &str,
    ordinary: &[&str],
    view: &[&str],
    tokens: &[&str],
) {
    let owner_type = if api == "NgModule" { "Module" } else { api };
    position_answers_as(
        state,
        p,
        owner_type,
        ordinary,
        if api == "Component" { view } else { &[] },
        tokens,
    );
}

fn source_for(api: &str) -> String {
    let marker = match api {
        "Directive" => "// Directive owner",
        "NgModule" => "// Module owner",
        _ => "",
    };
    format!("{}\n{marker}", source())
}

fn complete_restore(api: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let source = source_for(api);
    let (root, config, _) = setup_with_files(&source, FILES);
    let p = root.path();
    let database = rusqlite::Connection::open(&config.persistence.db_path).unwrap();
    let mut expected = Vec::new();
    for name in ["app.ts", "neighbor.ts"] {
        let encoded: String = database
            .query_row(
                "SELECT edges_json FROM semantic_edge_snapshots WHERE file_path = ?1",
                [p.join(name).to_string_lossy().into_owned()],
                |row| row.get(0),
            )
            .unwrap();
        let durable: Value = serde_json::from_str(&encoded).unwrap();
        let occurrences = durable["typescript_authority"]["demanded_occurrences"]
            .as_array()
            .unwrap();
        assert!(
            occurrences
                .iter()
                .any(|site| site["source"].as_str().unwrap().ends_with("group.ts"))
        );
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
        assert_eq!(registrations.len(), if api == "Component" { 6 } else { 3 });
        expected.push(
            json!({"authority":durable["typescript_authority"],"registrations":registrations}),
        );
    }
    drop(database);
    let state = crate::mcp::McpState::new(config);
    for (name, expected) in ["app.ts", "neighbor.ts"].into_iter().zip(expected) {
        let file = p.join(name).to_string_lossy().into_owned();
        let response = call(&state, p, "restore_context", json!({"filePath":file}));
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(
            view_proof(&state, &file),
            expected,
            "complete imported proof before any query"
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
    let owners: &[&str] = &["app.ts", "neighbor.ts"];
    let tokens: &[&str] = &["left.ts", "right.ts"];
    owner_answers(&state, p, api, owners, owners, tokens);
    for (field, ordinary, view) in [
        ("providers:GROUP", &["neighbor.ts"][..], owners),
        ("viewProviders:[...[ALIAS]]", owners, &["neighbor.ts"][..]),
    ] {
        let name = field.split_once(':').unwrap().0;
        std::fs::write(
            p.join("app.ts"),
            source.replace(field, &format!("{name}:[]")),
        )
        .unwrap();
        owner_answers(&state, p, api, ordinary, view, tokens);
        std::fs::write(p.join("app.ts"), &source).unwrap();
        owner_answers(&state, p, api, owners, owners, tokens);
    }
    std::fs::write(p.join("app.ts"), "export class ProviderHost {}").unwrap();
    owner_answers(&state, p, api, &["neighbor.ts"], &["neighbor.ts"], tokens);
    std::fs::remove_file(p.join("neighbor.ts")).unwrap();
    let response = call(&state, p, "refresh_workspace", json!({}));
    assert!(response.get("error").is_none(), "{response}");
    owner_answers(&state, p, api, &[], &[], &[]);
}

#[test]
fn issue_162_provider_imported_composition_stale_contributor_token_route_and_package_repair() {
    stale_restore("Component");
}

#[test]
fn issue_162_provider_owner_composition_stale_contributor_token_route_and_package_repair() {
    for api in ["Directive", "NgModule"] {
        stale_restore(api);
    }
}

fn stale_restore(api: &str) {
    let _serial = crate::protocol::handler_response_serial();
    let source = source_for(api);
    for mutation in [
        "entry", "mutable", "barrel", "delete", "replace", "token", "api", "version",
    ] {
        let (root, config, _) = setup_with_files(&source, FILES);
        let p = root.path();
        let (file, changed) = match mutation {
            "entry" => (
                p.join("group.ts"),
                Some(GROUP.replace("{provide:Right,useValue:2},", "")),
            ),
            "mutable" => (
                p.join("group.ts"),
                Some(GROUP.replace("const GROUP=", "let GROUP=")),
            ),
            "barrel" => (
                p.join("barrel.ts"),
                Some(
                    "export type {GROUP} from './group'; export type {ALIAS} from './alias';"
                        .to_owned(),
                ),
            ),
            "delete" => (p.join("group.ts"), None),
            "replace" => (
                p.join("group.ts"),
                Some("export const GROUP=[];".to_owned()),
            ),
            "token" => (
                p.join("right.ts"),
                Some("export interface AuditApi {}".to_owned()),
            ),
            "api" => (
                p.join("api.ts"),
                Some("export function View(metadata:unknown){return metadata;}".to_owned()),
            ),
            _ => {
                let file = p.join("node_modules/@angular/core/package.json");
                let changed = std::fs::read_to_string(&file)
                    .unwrap()
                    .replace("20.0.0", "20.0.1");
                (file, Some(changed))
            }
        };
        let original = std::fs::read(&file).unwrap();
        if let Some(changed) = changed {
            std::fs::write(&file, changed).unwrap();
        } else {
            std::fs::remove_file(&file).unwrap();
        }
        let state = crate::mcp::McpState::new(config);
        for name in ["app.ts", "neighbor.ts"] {
            let file = p.join(name).to_string_lossy().into_owned();
            let response = call(&state, p, "restore_context", json!({"filePath":file}));
            assert!(
                response.get("error").is_some(),
                "stale {mutation} imported proof adopted: {response}"
            );
            assert!(state.alias_for_path(&file).is_none());
        }
        let tokens: &[&str] = match mutation {
            "entry" | "token" => &["left.ts"],
            "version" => &["left.ts", "right.ts"],
            _ => &[],
        };
        let owners = &["app.ts", "neighbor.ts"];
        owner_answers(&state, p, api, owners, owners, tokens);
        std::fs::write(file, original).unwrap();
        owner_answers(&state, p, api, owners, owners, &["left.ts", "right.ts"]);
        for name in owners {
            assert_eq!(std::fs::read_to_string(p.join(name)).unwrap(), source);
        }
    }
}
