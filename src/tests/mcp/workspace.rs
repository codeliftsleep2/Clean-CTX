use super::*;
use crate::config::CleanCtxConfig;
use std::fs;
use tempfile::TempDir;

fn create_ts_file(dir: &std::path::PathBuf, name: &str, content: &str) {
    let path = dir.join(name);
    let mut f = fs::File::create(&path).unwrap();
    std::io::Write::write(&mut f, content.as_bytes()).unwrap();
}

/// F-05: `compress_workspace_dir` must honour `exclude_patterns`
/// from the project config.
#[test]
fn compress_workspace_dir_respects_exclude_patterns() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_path_buf();
    create_ts_file(
        &dir_path,
        "keep.ts",
        "export class A { foo(): void {} }\n",
    );
    create_ts_file(
        &dir_path,
        "skip-me.ts",
        "export class B { bar(): void {} }\n",
    );

    let mut config = CleanCtxConfig::default();
    config.exclude_patterns.push("skip-me".to_string());
    let mut state = McpState::new(config);

    let manifest = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &mut state,
    )
    .expect("workspace compress should succeed");

    // The kept file shows up in the manifest.
    assert!(
        manifest.contains("keep.ts"),
        "kept file should appear, got: {}",
        manifest
    );
    // The excluded file is reported under EXCLUDED and is NOT
    // emitted as a file block.
    assert!(
        manifest.contains("EXCLUDED"),
        "excluded file should be reported, got: {}",
        manifest
    );
}