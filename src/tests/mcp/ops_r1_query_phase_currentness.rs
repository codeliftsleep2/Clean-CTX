//! Preparation and consumption remain independently fresh production boundaries.
use crate::mcp::McpState;
use serde_json::json;

#[test]
fn ops_r1_resolver_consumption_refuses_config_edit_after_preparation() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    crate::tests::angular_authority_fixture::install(root.path());
    let config_file = root.path().join("tsconfig.json");
    std::fs::write(&config_file, r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    std::fs::write(root.path().join("owner.ts"), "export class Owner {}").unwrap();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    config.persistence.enabled = false;
    let state = McpState::new(config);
    let mut context = super::prepare::PreparationContext::new();
    let prepared = super::resolver_registrations::prepare(
        &json!({"workspaceRoot":root.path(),"type":"resolver_registrations","direction":"forward","route_path":"__ops_r1_absent__"}),
        &state,
        &mut context,
    );
    assert!(
        prepared.is_indexed(),
        "must reach the real prepared consumption boundary"
    );
    assert_eq!(state.test_full_semantic_parses(), 1);
    std::fs::write(&config_file, r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"strict":true},"include":["*.ts"]}"#).unwrap();
    let error = match prepared.evaluate_single(&state) {
        Err(error) => error,
        Ok(_) => panic!("consumption accepted config changed after preparation"),
    };
    assert_eq!(error.code, -32603);
    assert_eq!(
        error.message,
        "Registration source/dependency publication changed; retry"
    );
}
