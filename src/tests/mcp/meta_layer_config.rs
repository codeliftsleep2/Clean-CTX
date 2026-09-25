#![cfg(feature = "typescript")]

use crate::config::MetaLayerConfig;
use crate::layers::meta::semantic::SemanticRelation;
use crate::mcp::tools::dispatch_tools_call;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

fn dispatch_provide(state: &crate::mcp::McpState, path: &Path, root: &Path) -> Value {
    crate::protocol::captured_responses().clear();
    dispatch_tools_call(
        &json!(1),
        "provide_code_context",
        &json!({
            "arguments": {
                "filePath": path.to_string_lossy(),
                "workspaceRoot": root.to_string_lossy(),
                "fidelity": "high"
            }
        }),
        state,
    );
    crate::protocol::captured_responses()
        .pop()
        .expect("registered handler response")
}

fn write_fixture(root: &tempfile::TempDir, name: &str, source: &str) -> PathBuf {
    let path = root.path().join(name);
    let padded = format!(
        "{source}\n{}",
        "// economics-padding-0123456789abcdef\n".repeat(1_000)
    );
    std::fs::write(&path, padded).unwrap();
    path
}

fn state_with(
    root: &tempfile::TempDir,
    configure: impl FnOnce(&mut crate::config::CleanCtxConfig),
) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(root.path().to_string_lossy().into_owned());
    configure(&mut config);
    crate::mcp::McpState::new(config)
}

fn compiled_edges(
    state: &crate::mcp::McpState,
    path: &Path,
) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    let path_text = path.to_string_lossy();
    let alias = state
        .alias_for_path(&path_text)
        .expect("registered provide assigns an alias");
    state
        .semantic_edges(&alias)
        .expect("registered provide publishes semantic edges")
}

fn schema_text(response: &Value) -> &str {
    assert!(response.get("error").is_none(), "{response}");
    let text = response["result"]["content"][0]["text"]
        .as_str()
        .expect("model-visible text");
    assert!(text.starts_with("// SCHEMA v5"), "{text}");
    text
}

fn angular_source() -> &'static str {
    r#"import { Injectable } from '@angular/core';
@Injectable()
export class Consumer {
  constructor(private repo: Repo) {}
}"#
}

#[test]
fn registered_provide_keeps_default_angular_and_builtin_layers_enabled() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = write_fixture(&root, "consumer.service.ts", angular_source());
    let state = state_with(&root, |_| {});

    let response = dispatch_provide(&state, &path, root.path());
    let text = schema_text(&response);
    assert!(text.contains("T @svc = Consumer"), "{text}");
    let edges = compiled_edges(&state, &path);
    assert!(
        edges
            .iter()
            .any(|edge| edge.layer == "angular" && edge.relation == SemanticRelation::Injects),
        "default configuration must preserve Angular semantic extraction"
    );
    assert!(
        edges.iter().any(|edge| edge.layer == "builtin"),
        "the always-on builtin layer must remain registered"
    );
}

#[test]
fn registered_provide_honors_angular_framework_opt_out() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = write_fixture(&root, "consumer.service.ts", angular_source());
    let state = state_with(&root, |config| {
        let angular = MetaLayerConfig {
            enabled: false,
            ..MetaLayerConfig::default()
        };
        config.meta_layers.insert("angular".to_string(), angular);
    });

    let response = dispatch_provide(&state, &path, root.path());
    let text = schema_text(&response);
    assert!(!text.contains("T @svc ="), "{text}");
    assert!(!text.contains("T @injects ="), "{text}");
    let edges = compiled_edges(&state, &path);
    assert!(
        edges.iter().all(|edge| edge.layer != "angular"),
        "disabled Angular metadata must not reach the production result"
    );
    assert!(
        edges.iter().any(|edge| edge.layer == "builtin"),
        "framework opt-out must not disable the builtin layer"
    );
}

#[cfg(feature = "dotnet")]
#[test]
fn registered_provide_honors_dotnet_framework_opt_out() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = write_fixture(
        &root,
        "health_controller.cs",
        r#"[ApiController]
[Route("api/health")]
public class HealthController : ControllerBase {}"#,
    );
    let state = state_with(&root, |config| {
        let dotnet = MetaLayerConfig {
            enabled: false,
            ..MetaLayerConfig::default()
        };
        config.meta_layers.insert("dotnet".to_string(), dotnet);
    });

    let response = dispatch_provide(&state, &path, root.path());
    let text = schema_text(&response);
    assert!(!text.contains("T @ctrl ="), "{text}");
    let edges = compiled_edges(&state, &path);
    assert!(
        edges.iter().all(|edge| edge.layer != "dotnet"),
        "disabled .NET metadata must not reach the production result"
    );
    assert!(
        edges.iter().any(|edge| edge.layer == "builtin"),
        "framework opt-out must not disable the builtin layer"
    );
}

#[cfg(feature = "dotnet")]
#[test]
fn registered_provide_honors_dotnet_testing_sub_layer_opt_out() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let path = write_fixture(
        &root,
        "accounts_controller_tests.cs",
        r#"[TestClass]
public class AccountsControllerTests {
  [TestMethod]
  public async Task Returns_account() {}
}

[ApiController]
[Route("api/health")]
public class HealthController : ControllerBase {}"#,
    );
    let state = state_with(&root, |config| {
        let mut dotnet = MetaLayerConfig::default();
        dotnet.testing.enabled = false;
        config.meta_layers.insert("dotnet".to_string(), dotnet);
    });

    let response = dispatch_provide(&state, &path, root.path());
    let text = schema_text(&response);
    assert!(text.contains("T @ctrl = HealthController"), "{text}");
    assert!(!text.contains("T @testcls ="), "{text}");
    let edges = compiled_edges(&state, &path);
    assert!(
        edges
            .iter()
            .all(|edge| edge.relation != SemanticRelation::Tests),
        "the disabled testing sub-layer must emit no testing edges"
    );
    assert!(
        edges
            .iter()
            .any(|edge| { edge.layer == "dotnet" && edge.relation == SemanticRelation::HasRoute }),
        "unrelated .NET metadata must remain enabled"
    );
    assert!(edges.iter().any(|edge| edge.layer == "builtin"));
}
