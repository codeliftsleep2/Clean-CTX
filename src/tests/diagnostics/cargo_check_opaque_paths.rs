use crate::diagnostics::cargo_check::{ApprovedWorkspaceRoot, AuthoritySource, CargoCheckCompiler};
use serde_json::json;

fn fixture() -> (tempfile::TempDir, CargoCheckCompiler, String) {
    let workspace = tempfile::tempdir().unwrap();
    std::fs::write(
        workspace.path().join("Cargo.toml"),
        "[package]\nname='path_fixture'\n",
    )
    .unwrap();
    let authority =
        ApprovedWorkspaceRoot::admit(workspace.path(), AuthoritySource::StartupOption).unwrap();
    let root = authority.canonical_root().to_string_lossy().into_owned();
    (
        workspace,
        CargoCheckCompiler::for_workspace(authority),
        root,
    )
}

#[test]
fn opaque_workspace_path_regression_cargo_progress_evidence_hides_absolute_root() {
    let (_workspace, mut compiler, root) = fixture();
    let line = format!("   Compiling path_fixture v0.1.0 ({root})");
    compiler.observe_stderr_frame(line.as_bytes(), true, false);
    let result = compiler.finish();
    assert_eq!(result.evidence.len(), 1);
    let evidence = &result.evidence[0];
    assert!(
        !evidence.text.contains(&root),
        "retained Cargo stderr must not disclose the approved absolute root"
    );
    assert!(evidence.text.contains("<workspace>"));
    assert_eq!(evidence.source_bytes, line.len());
    assert_eq!(result.parser_coverage.stderr_frames, 1);
}

#[test]
fn opaque_workspace_path_regression_diagnostic_strings_hide_absolute_root() {
    let (_workspace, mut compiler, root) = fixture();
    let record = json!({"reason":"compiler-message","message":{
        "message":format!("failed to load {root}/src/lib.rs"),
        "level":"error","spans":[],
        "children":[{"level":"note","message":format!("workspace: {root}"),"spans":[]}],
        "rendered":format!("error: failed to load {root}/src/lib.rs\nAuthorization: Bearer abc123456789"),
    }});
    compiler.observe_stdout_frame(&serde_json::to_vec(&record).unwrap(), true, false);
    let result = compiler.finish();
    let diagnostic = &result.diagnostics[0];
    assert!(
        !diagnostic.message.contains(&root),
        "diagnostic messages must not disclose the approved absolute root"
    );
    assert!(!diagnostic.children[0].message.contains(&root));
    let rendered = diagnostic.rendered_evidence.as_ref().unwrap();
    assert!(!rendered.contains(&root));
    assert!(!rendered.contains("abc123456789"));
    assert!(diagnostic.message.contains("<workspace>/src/lib.rs"));
    assert_eq!(result.parser_coverage.compiler_messages, 1);
}
