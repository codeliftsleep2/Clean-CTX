use super::a2_support as fixture;

#[test]
fn blocked_provider_admission_does_not_block_clean_initialize_readiness() {
    let root = tempfile::tempdir().expect("workspace fixture");
    let log = root.path().join("blocked-admission.json");
    let mut config = crate::tests::test_config();
    config.cbm.enabled = true;
    config.cbm.auto_launch = true;
    config.cbm.binary_path = Some(
        std::env::current_exe()
            .expect("current test executable")
            .to_string_lossy()
            .into_owned(),
    );
    config.cbm.query_timeout_ms = 100;

    fixture::with_options(&log, "a3-notify-only", || {
        let state = crate::mcp::McpState::new(config);
        let requests = std::fs::read_to_string(log.with_extension("requests")).unwrap_or_default();

        assert!(
            requests.lines().all(|line| {
                serde_json::from_str::<serde_json::Value>(line)
                    .ok()
                    .is_none_or(|request| request["method"] != "initialize")
            }),
            "core Clean state construction started provider admission before the client could complete Clean initialize: {requests}"
        );

        drop(state);
    });
}
