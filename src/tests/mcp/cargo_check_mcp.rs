use super::*;
use std::ffi::OsString;

fn fixture(mode: &str) -> (tempfile::TempDir, CargoCheckStartupOptions) {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='fixture'\n",
    )
    .unwrap();
    std::fs::write(workspace.path().join(".owned-execution-fixture"), mode).unwrap();
    let options = CargoCheckStartupOptions {
        workspace_root: Some(workspace.path().to_owned()),
        cargo_path: Some(std::env::current_exe().unwrap()),
    };
    (workspace, options)
}

fn state(
    options: &CargoCheckStartupOptions,
    snapshot: &[(OsString, OsString)],
) -> super::super::McpState {
    let mut config = crate::config::CleanCtxConfig::default();
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    let mut state = super::super::McpState::new(config);
    state.cargo_check = CargoCheckSession::new(prepare_cargo_check_startup(options, snapshot));
    state
}

fn response(state: &super::super::McpState, id: Value, arguments: Value) -> Value {
    crate::protocol::captured_responses().clear();
    super::super::tool_dispatch::dispatch_tools_call(
        &id,
        "cargo_check",
        &json!({"arguments":arguments}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}

#[test]
fn cargo_check_mcp_startup_never_uses_discovered_roots_or_bare_cargo() {
    assert_eq!(
        prepare_cargo_check_startup(&Default::default(), &[]),
        Err("workspace_authority_missing")
    );
    let (_workspace, mut options) = fixture("success");
    options.cargo_path = None;
    assert_eq!(
        prepare_cargo_check_startup(&options, &[]),
        Err("cargo_authority_missing")
    );
    options.cargo_path = Some("cargo".into());
    assert_eq!(
        prepare_cargo_check_startup(&options, &[]),
        Err("cargo_admission_failed")
    );
}

#[test]
fn cargo_check_mcp_startup_precedence_is_independent_for_each_authority() {
    let (_workspace, options) = fixture("success");
    let snapshot = vec![
        (
            "CLEAN_CTX_PROJECT_ROOT".into(),
            "PRIVATE_ROOT_CANARY".into(),
        ),
        ("CLEAN_CTX_CARGO_PATH".into(), "PRIVATE_CARGO_CANARY".into()),
    ];
    let invocation = prepare_cargo_check_startup(&options, &snapshot).unwrap();
    let facts = invocation.result_facts();
    assert_eq!(facts.workspace_source, AuthoritySource::StartupOption);
    assert_eq!(facts.cargo_source, AuthoritySource::StartupOption);
    assert!(facts.workspace_environment_shadowed && facts.cargo_environment_shadowed);
    assert!(!serde_json::to_string(&facts).unwrap().contains("PRIVATE_"));
    let mut invalid = options.clone();
    invalid.workspace_root = Some("relative".into());
    let valid_snapshot = vec![(
        "CLEAN_CTX_PROJECT_ROOT".into(),
        options.workspace_root.unwrap().into_os_string(),
    )];
    assert_eq!(
        prepare_cargo_check_startup(&invalid, &valid_snapshot),
        Err("workspace_admission_failed")
    );
}

#[test]
fn cargo_check_mcp_startup_environment_is_frozen_and_sources_disclosed() {
    let (_workspace, options) = fixture("success");
    let mut snapshot = vec![
        (
            "CLEAN_CTX_PROJECT_ROOT".into(),
            options.workspace_root.unwrap().into_os_string(),
        ),
        (
            "CLEAN_CTX_CARGO_PATH".into(),
            options.cargo_path.unwrap().into_os_string(),
        ),
        ("PRIVATE_CREDENTIAL".into(), "DO_NOT_INHERIT".into()),
    ];
    let invocation = prepare_cargo_check_startup(&Default::default(), &snapshot).unwrap();
    snapshot.clear();
    assert_eq!(
        invocation.result_facts().workspace_source,
        AuthoritySource::Environment
    );
    assert_eq!(
        invocation.result_facts().cargo_source,
        AuthoritySource::Environment
    );
    assert!(
        !invocation
            .environment()
            .contains_key(std::ffi::OsStr::new("PRIVATE_CREDENTIAL"))
    );
    let (_workspace, options) = fixture("success");
    let snapshot = vec![(
        "CLEAN_CTX_CARGO_PATH".into(),
        options.cargo_path.clone().unwrap().into_os_string(),
    )];
    let mut mixed = options;
    mixed.cargo_path = None;
    let facts = prepare_cargo_check_startup(&mixed, &snapshot)
        .unwrap()
        .result_facts();
    assert_eq!(facts.workspace_source, AuthoritySource::StartupOption);
    assert_eq!(facts.cargo_source, AuthoritySource::Environment);
}

#[test]
fn cargo_check_mcp_rejects_all_nonempty_or_nonobject_arguments_before_execution() {
    let (_workspace, options) = fixture("success");
    let state = state(&options, &[]);
    for arguments in [
        Value::Null,
        json!([]),
        json!(false),
        json!("canary"),
        json!({"command":"PRIVATE_COMMAND_CANARY"}),
        json!({"cargoPath":"cargo"}),
        json!({"environment":{}}),
        json!({"args":["--release"]}),
    ] {
        let output = response(&state, json!(17), arguments);
        assert_eq!(output["error"]["code"], -32602);
        assert!(!output.to_string().contains("PRIVATE_COMMAND_CANARY"));
    }
    assert!(
        state
            .cargo_check
            .lifecycle
            .lock()
            .unwrap()
            .active
            .is_empty()
    );
}

#[test]
fn cargo_check_mcp_unavailable_catalog_contains_static_admission_explanation() {
    let state = state(&Default::default(), &[]);
    crate::protocol::captured_responses().clear();
    super::super::handlers::handle_tools_list(&json!(1), &state);
    let output = crate::protocol::captured_responses().pop().unwrap();
    let tool = output["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "cargo_check")
        .unwrap();
    assert!(
        tool["description"]
            .as_str()
            .unwrap()
            .contains("Unavailable: workspace_authority_missing")
    );
    assert_eq!(tool["annotations"]["openWorldHint"], true);
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_success_projects_same_core_without_cli_metadata() {
    let (_workspace, options) = fixture("success");
    let state = state(&options, &[]);
    let output = response(&state, json!(21), json!({}));
    assert_eq!(output["id"], 21);
    assert_eq!(output["result"]["isError"], false);
    let structured = &output["result"]["structuredContent"];
    assert_eq!(structured["root_outcome"]["exit_code"], 0);
    assert!(structured.get("cli_outcome").is_none());
    assert!(structured.get("exit_code").is_none());
    assert_eq!(
        structured["authority"]["command"],
        "cargo check --message-format=json"
    );
    let bytes = serde_json::to_vec(structured).unwrap().len();
    assert_eq!(
        structured["semantic"]["result_budget"]["serialized_bytes"],
        bytes
    );
    assert!(bytes <= CargoCheckPolicy::APPROVED.structured_content_bytes);
    let text = output["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.len() <= CargoCheckPolicy::APPROVED.content_bytes);
    assert!(text.lines().count() <= CargoCheckPolicy::APPROVED.content_lines);
    assert!(
        state
            .cargo_check
            .lifecycle
            .lock()
            .unwrap()
            .active
            .is_empty()
    );
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_cargo_failure_preserves_os_status_despite_success_evidence() {
    let (_workspace, options) = fixture("failure-with-success-evidence");
    let output = response(&state(&options, &[]), json!(22), json!({}));
    assert_eq!(output["result"]["isError"], true);
    assert_eq!(
        output["result"]["structuredContent"]["root_outcome"]["exit_code"],
        7
    );
    assert_eq!(
        output["result"]["structuredContent"]["semantic"]["cargo_evidence"]["build_finished_success"],
        true
    );
    assert!(output.get("error").is_none());
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_changed_authority_failure_hides_paths_and_producer_text() {
    let (workspace, options) = fixture("success");
    let state = state(&options, &[]);
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "PRIVATE_MANIFEST_CANARY",
    )
    .unwrap();
    let output = response(&state, json!(23), json!({}));
    assert_eq!(
        output["result"]["structuredContent"]["failure"],
        "authority_changed_before_start"
    );
    assert!(!output.to_string().contains("PRIVATE_MANIFEST_CANARY"));
    assert!(
        !output
            .to_string()
            .contains(workspace.path().to_str().unwrap())
    );
    assert!(output.get("error").is_none());
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_queued_cancellation_prevents_process_start_and_clears_registration() {
    let (_workspace, options) = fixture("success");
    let state = state(&options, &[]);
    assert!(state.cargo_check.reserve(&json!(31)));
    assert!(!state.cargo_check.reserve(&json!(31)));
    super::super::router::dispatch(
        crate::protocol::JsonRpcRequest {
            jsonrpc: "2.0".into(),
            id: None,
            method: "notifications/cancelled".into(),
            params: Some(json!({"requestId":31,"reason":"PRIVATE_CANCEL_CANARY"})),
        },
        &state,
    );
    let output = response(&state, json!(31), json!({}));
    assert_eq!(
        output["result"]["structuredContent"]["failure"],
        "cancelled_before_start"
    );
    assert_eq!(
        output["result"]["structuredContent"]["cancellation_source"],
        "host"
    );
    assert!(!output.to_string().contains("PRIVATE_CANCEL_CANARY"));
    assert!(
        state
            .cargo_check
            .lifecycle
            .lock()
            .unwrap()
            .active
            .is_empty()
    );
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_shutdown_cancels_queued_and_future_calls() {
    let (_workspace, options) = fixture("success");
    let state = state(&options, &[]);
    assert!(state.cargo_check.reserve(&json!(32)));
    state.cargo_check.shutdown();
    for id in [32, 33] {
        let output = response(&state, json!(id), json!({}));
        assert_eq!(
            output["result"]["structuredContent"]["failure"],
            "cancelled_before_start"
        );
    }
    assert!(
        state
            .cargo_check
            .lifecycle
            .lock()
            .unwrap()
            .active
            .is_empty()
    );
}

#[test]
fn cargo_check_mcp_duplicate_live_id_does_not_lose_existing_token() {
    let session = CargoCheckSession::default();
    let active = session.begin(&json!(44)).unwrap();
    assert!(session.begin(&json!(44)).is_err());
    session.cancel(&json!(44));
    assert!(session.lifecycle.lock().unwrap().active.contains_key("44"));
    drop(active);
    assert!(session.lifecycle.lock().unwrap().active.is_empty());
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_both_protocol_surfaces_hide_recognized_producer_secrets() {
    let (_workspace, options) = fixture("mcp-secret-evidence");
    let output = response(&state(&options, &[]), json!(41), json!({}));
    assert_eq!(output["result"]["isError"], true);
    assert!(!output.to_string().contains("abc123456789"));
    let structured = &output["result"]["structuredContent"];
    assert_eq!(
        structured["semantic"]["parser_coverage"]["compiler_messages"],
        1
    );
    assert_eq!(
        structured["semantic"]["parser_coverage"]["unknown_structured"],
        1
    );
    assert!(
        !structured["semantic"]["diagnostics"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(
        output["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("CargoCheck")
    );
}

#[test]
#[cfg(any(target_os = "linux", windows))]
fn cargo_check_mcp_host_cancellation_owns_and_cleans_running_process_tree() {
    let (workspace, options) = fixture("cancel-tree");
    let state = state(&options, &[]);
    std::thread::scope(|scope| {
        let running = scope.spawn(|| state.cargo_check.call(&json!(51)));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !workspace.path().join("descendant-ready").exists()
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        state.cargo_check.cancel(&json!(51));
        let output = running.join().unwrap();
        assert!(
            workspace.path().join("descendant-ready").exists(),
            "fixture must launch owned descendant"
        );
        assert_eq!(output["isError"], true);
        assert_eq!(
            output["structuredContent"]["cleanup"]["cancellation_source"],
            "host"
        );
        assert_eq!(
            output["structuredContent"]["cleanup"]["descendant_quiescence_observed"],
            true
        );
        assert_eq!(
            output["structuredContent"]["cleanup"]["cleanup_uncertain"],
            false
        );
    });
    assert!(
        state
            .cargo_check
            .lifecycle
            .lock()
            .unwrap()
            .active
            .is_empty()
    );
}
