use serde_json::{Value, json};
use std::path::Path;

const SOURCE: &str = r#"
import {Component as View} from '@angular/core';
import {Token,Implementation as Local} from './local';
import * as remote from 'audit-implementation';
const prose='useClass:Phantom';
function noise<T>(remote:T){return remote;}
@View({providers:[{provide:Token,useClass:Local},{provide:Token,useClass:remote.Implementation}]})
export class ProviderHost {}
"#;

fn call(state: &crate::mcp::McpState, root: &Path, tool: &str, mut args: Value) -> Value {
    args["workspaceRoot"] = json!(root);
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(162), tool, &json!({"arguments":args}), state);
    crate::protocol::captured_responses().pop().unwrap()
}

fn targets(
    state: &crate::mcp::McpState,
    root: &Path,
    remote: bool,
    field: &str,
    owner_type: &str,
    view: bool,
) {
    let factory = field == "useFactory";
    let value = field == "useValue";
    let relation = if value {
        if view {
            "RegistersViewProvider"
        } else {
            "RegistersProvider"
        }
    } else if field == "useExisting" {
        "ProviderUsesExisting"
    } else if factory {
        "ProviderUsesFactory"
    } else {
        "ProviderUsesClass"
    };
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"entities_in_file","file_path":root.join("app.ts"),"fidelity":"high"}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let response = call(
        state,
        root,
        "workspace_query",
        json!({"type":"forward_edges","domain":"angular","entity_type":if value {owner_type} else {"ProviderToken"},"name":if value {"ProviderHost"} else {"Token"}}),
    );
    assert!(response.get("error").is_none(), "{response}");
    let mut actual: Vec<_> = response["result"]["structuredContent"]["edges"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|edge| edge["relation"] == relation)
        .map(|edge| {
            if value {
                return crate::dictionary::path::canonical_identity_key(
                    edge["object"]["file"].as_str().unwrap(),
                );
            }
            assert_eq!(edge["object"]["name"], "Implementation");
            assert_eq!(
                crate::dictionary::path::canonical_identity_key(
                    edge["subject"]["file"].as_str().unwrap()
                ),
                crate::dictionary::path::canonical_identity_key(
                    &root.join("local.ts").to_string_lossy()
                )
            );
            crate::dictionary::path::canonical_identity_key(
                edge["object"]["file"].as_str().unwrap(),
            )
        })
        .collect();
    actual.sort();
    let mut expected = vec![crate::dictionary::path::canonical_identity_key(
        &root.join("local.ts").to_string_lossy(),
    )];
    if remote && value {
        expected.push(crate::dictionary::path::canonical_identity_key(
            &root.join("local.ts").to_string_lossy(),
        ));
    } else if remote {
        expected.push(crate::dictionary::path::canonical_identity_key(
            &root
                .join(if factory {
                    "node_modules/audit-implementation/index.ts"
                } else {
                    "node_modules/audit-implementation/index.d.ts"
                })
                .to_string_lossy(),
        ));
    }
    expected.sort();
    assert_eq!(
        actual, expected,
        "local strategy survives independent package target changes"
    );
}

#[test]
fn issue_162_provider_strategy_currentness_package_config_and_ambiguity() {
    currentness("useClass");
}

#[test]
fn issue_162_provider_factory_currentness_package_config_and_ambiguity() {
    currentness("useFactory");
}

#[test]
fn issue_162_provider_existing_currentness_package_config_and_ambiguity() {
    currentness("useExisting");
}

#[test]
fn issue_162_provider_value_currentness_package_config_and_ambiguity() {
    currentness("useValue");
}

fn currentness(field: &str) {
    currentness_owner(field, false);
}

#[test]
fn issue_162_provider_directive_currentness_package_config_and_ambiguity() {
    currentness_owner("useValue", true);
}

fn currentness_owner(field: &str, directive: bool) {
    currentness_position(field, directive, false);
}

#[test]
fn issue_162_view_provider_currentness_export_config_version_and_ambiguity() {
    currentness_position("useValue", false, true);
}

fn currentness_position(field: &str, directive: bool, view: bool) {
    currentness_source(field, directive, view, false);
}

#[test]
fn issue_162_provider_composition_currentness_export_config_version_and_ambiguity() {
    for view in [false, true] {
        currentness_source("useValue", false, view, true);
    }
}

fn currentness_source(field: &str, directive: bool, view: bool, composed: bool) {
    let owner_type = if directive { "Directive" } else { "Component" };
    let factory = field == "useFactory";
    let _serial = crate::protocol::handler_response_serial();
    for mutation in ["exports", "config", "version", "ambiguity"] {
        let root = tempfile::tempdir().unwrap();
        let p = root.path();
        crate::tests::angular_authority_fixture::install(p);
        if directive {
            crate::tests::angular_authority_fixture::install_inject_apis(p);
        }
        let package = p.join("node_modules/audit-implementation");
        std::fs::create_dir_all(&package).unwrap();
        let file = if factory { "index.ts" } else { "index.d.ts" };
        let manifest = json!({"name":"audit-implementation","version":"1.0.0","types":file,"exports":{".":{"types":format!("./{file}")}}}).to_string();
        std::fs::write(package.join("package.json"), &manifest).unwrap();
        let declaration = if factory {
            "export function Implementation(){return {};}"
        } else {
            "export declare class Implementation {}"
        };
        std::fs::write(package.join(file), declaration).unwrap();
        std::fs::write(
            p.join("local.ts"),
            if factory {
                "export class Token {} export function Implementation(){return {};}"
            } else {
                "export class Token {} export class OtherToken {} export class Implementation {}"
            },
        )
        .unwrap();
        std::fs::write(p.join("erased.ts"), "export interface Implementation {}").unwrap();
        let source = SOURCE.replace("useClass", field);
        let source = if view {
            source.replace("providers:[", "viewProviders:[")
        } else {
            source
        };
        let source = if directive {
            source.replace("Component as View", "Directive as View")
        } else {
            source
        };
        let source = if field == "useValue" {
            source
                .replace("{Token,Implementation", "{Token,OtherToken,Implementation")
                .replace(
                    "provide:Token,useValue:remote",
                    "provide:OtherToken,useValue:remote",
                )
        } else {
            source
        };
        let source = if composed {
            let position = if view { "viewProviders" } else { "providers" };
            source
                .replace(&format!("@View({{{position}:["), "const GROUP=[")
                .replace(
                    "]})\nexport class",
                    &format!(
                        "]; const ALIAS=GROUP; @View({{{position}:[...ALIAS]}})\nexport class"
                    ),
                )
        } else {
            source
        };
        std::fs::write(p.join("app.ts"), &source).unwrap();
        let original_config = std::fs::read(p.join("tsconfig.json")).unwrap();
        let mut config = crate::tests::test_config();
        config.cbm.enabled = false;
        config.persistence.enabled = true;
        config.persistence.db_path = p
            .join("provider-currentness.db")
            .to_string_lossy()
            .into_owned();
        config
            .additional_roots
            .push(p.to_string_lossy().into_owned());
        {
            let state = crate::mcp::McpState::new(config.clone());
            targets(&state, p, true, field, owner_type, view);
            let response = call(
                &state,
                p,
                "compress_code_context",
                json!({"filePath":p.join("app.ts"),"fidelity":"high"}),
            );
            assert!(response.get("error").is_none(), "{response}");
            state.flush_persistence();
        }
        match mutation {
            "exports" => {
                let mut value: Value = serde_json::from_str(&manifest).unwrap();
                value["exports"]["."] = Value::Null;
                std::fs::write(package.join("package.json"), value.to_string()).unwrap();
            }
            "config" => {
                let mut value: Value = serde_json::from_slice(&original_config).unwrap();
                value["compilerOptions"]["paths"] = json!({"audit-implementation":["./erased.ts"]});
                std::fs::write(p.join("tsconfig.json"), value.to_string()).unwrap();
            }
            "version" => std::fs::write(
                package.join("package.json"),
                manifest.replace("1.0.0", "1.0.1"),
            )
            .unwrap(),
            _ => {
                std::fs::write(package.join(file), format!("{declaration} {declaration}")).unwrap()
            }
        }
        let state = crate::mcp::McpState::new(config);
        let response = call(
            &state,
            p,
            "restore_context",
            json!({"filePath":p.join("app.ts")}),
        );
        assert!(
            response.get("error").is_some(),
            "stale {mutation} proof adopted: {response}"
        );
        assert!(
            state
                .alias_for_path(&p.join("app.ts").to_string_lossy())
                .is_none()
        );
        targets(&state, p, mutation == "version", field, owner_type, view);
        std::fs::write(package.join("package.json"), &manifest).unwrap();
        std::fs::write(package.join(file), declaration).unwrap();
        std::fs::write(p.join("tsconfig.json"), &original_config).unwrap();
        targets(&state, p, true, field, owner_type, view);
        assert_eq!(std::fs::read_to_string(p.join("app.ts")).unwrap(), source);
    }
}
