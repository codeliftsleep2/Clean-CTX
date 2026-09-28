use crate::cbm::GraphBridge;
use crate::cbm::bridge::cbm_project_slug;
use crate::cbm::config::CbmConfig;
use crate::cbm::proxy::{proxy_tool_error, resolve_and_apply_proxy_target_project};
use serde_json::json;

#[test]
fn cbm_proxy_rewrites_configured_root_basename_to_canonical_slug() {
    let root = tempfile::TempDir::new().unwrap();
    let canonical_root = root.path().canonicalize().unwrap();
    let basename = canonical_root.file_name().unwrap().to_string_lossy();
    let expected = cbm_project_slug(&canonical_root);
    let config = CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge = GraphBridge::try_create(&config, root.path());
    let params = json!({ "arguments": { "project": basename } });
    let mut tool_params = json!({ "name_pattern": "target", "project": basename });

    let resolved = resolve_and_apply_proxy_target_project(&bridge, &params, &mut tool_params);

    assert_eq!(resolved.as_deref(), Some(expected.as_str()));
    assert_eq!(tool_params["project"], expected);
}

#[test]
fn cbm_proxy_rejects_soft_project_error_before_compressing_partial_data() {
    let raw = r#"{
        "jsonrpc": "2.0",
        "id": 7,
        "result": {
            "content": [{
                "type": "text",
                "text": "{\"results\":[],\"raw_candidates\":0,\"callers\":[],\"error\":\"project not found or indexed\"}"
            }],
            "isError": true
        }
    }"#;

    let error = proxy_tool_error("search_graph", raw)
        .expect("a CBM soft-error envelope must stop the proxy success path");

    assert!(error.contains("project not found or indexed"));
    assert!(
        error.contains("list_projects"),
        "project errors need recovery guidance"
    );
    assert!(!error.contains("raw_candidates"));
    assert!(!error.contains("callers"));
}
