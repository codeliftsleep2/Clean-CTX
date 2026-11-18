use crate::mcp::tools::dispatch_tools_call;
use serde_json::json;

fn git(root: &std::path::Path, args: &[&str]) {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn diff_commits_exposes_loss_and_comparison_dimensions_in_public_metadata() {
    let _serial = crate::protocol::handler_response_serial();
    let repo = tempfile::tempdir().expect("temp repo");
    git(repo.path(), &["init", "-q"]);
    git(repo.path(), &["config", "user.email", "test@example.com"]);
    git(repo.path(), &["config", "user.name", "Test"]);
    std::fs::write(repo.path().join("Probe.cs"), "class Probe {}\n").unwrap();
    git(repo.path(), &["add", "Probe.cs"]);
    git(repo.path(), &["commit", "-q", "-m", "before"]);
    std::fs::write(
        repo.path().join("Probe.cs"),
        "class Probe { int Get() { return 1; } }\n",
    )
    .unwrap();
    git(repo.path(), &["add", "Probe.cs"]);
    git(repo.path(), &["commit", "-q", "-m", "after"]);

    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(repo.path().to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    dispatch_tools_call(
        &json!(137),
        "diff_commits",
        &json!({"arguments": {
            "workspaceRoot": repo.path(),
            "fromRef": "HEAD~1",
            "toRef": "HEAD",
            "fidelity": "medium"
        }}),
        &state,
    );
    let response = crate::protocol::captured_responses()
        .pop()
        .expect("captured response");
    assert!(response.get("error").is_none(), "{response}");
    let comparison = &response["result"]["_meta"]["comparison"];
    assert_eq!(comparison["lossy"], true);
    assert_eq!(comparison["scope"], "admitted-declarations");
    assert!(
        comparison["compares"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "normalized-body")
    );
    assert!(
        comparison["omits"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value == "unsupported-declarations")
    );
}
