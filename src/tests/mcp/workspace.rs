use super::*;
use crate::config::CleanCtxConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

fn create_ts_file(dir: &Path, name: &str, content: &str) {
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
        dir.path(),
        "keep.ts",
        "export class A { foo(): void {} }\n",
    );
    create_ts_file(
        dir.path(),
        "skip-me.ts",
        "export class B { bar(): void {} }\n",
    );

    let mut config = CleanCtxConfig::default();
    // F-12: use a glob pattern (`skip-me*`) to match the filename
    // prefix. The new segment-based matcher no longer does bare
    // substring matching, so `"skip-me"` would not match `"skip-me.ts"`.
    config.exclude_patterns.push("skip-me*".to_string());
    let mut state = McpState::new(config);

    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &mut state,
    )
    .expect("workspace compress should succeed");

    let manifest = &result.manifest;
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
    // F-13: the structured errors/excluded fields should be populated.
    assert!(
        result.excluded.iter().any(|p| p.contains("skip-me")),
        "excluded vec should contain the skipped file"
    );
}

/// F-09 (FAANG audit): per-file alias cross-reference lines should
/// appear in the workspace manifest so LLM clients can correlate the
/// block with the path map footer.
#[test]
fn workspace_emits_alias_cross_reference() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_path_buf();
    create_ts_file(
        dir.path(),
        "alpha.ts",
        "export class Alpha { run(): void {} }\n",
    );

    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);

    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &mut state,
    )
    .expect("workspace compress should succeed");

    let manifest = &result.manifest;
    // The manifest should contain the per-file alias line.
    assert!(
        manifest.contains("α alias:"),
        "workspace manifest should contain per-file alias cross-reference, got:\n{}",
        manifest
    );
    // The path map footer should also be present.
    assert!(
        manifest.contains("§PATHMAP"),
        "workspace manifest should contain path map footer, got:\n{}",
        manifest
    );
}

/// F-09: calling compress_workspace after compress_code_context on
/// the same file should produce the same alias.
#[test]
fn workspace_shares_aliases_with_per_file_tool() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_path_buf();
    create_ts_file(
        dir.path(),
        "shared.ts",
        "export class Shared { hello(): string { return ''; } }\n",
    );

    let config = CleanCtxConfig::default();
    let mut state = McpState::new(config);

    // First, compress via the per-file tool path (simulated inline).
    let file_path = dir_path.join("shared.ts");
    let compressed = crate::compressor::compress_file(
        file_path.clone(),
        &mut state.dict,
        &mut state.cache,
        Fidelity::Low,
    )
    .expect("per-file compress should succeed");

    // The per-file result should contain an alias.
    assert!(compressed.contains('α'), "per-file output should contain alias");

    // Now compress the workspace — it should reuse the same alias.
    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &mut state,
    )
    .expect("workspace compress should succeed");

    // Both the per-file output and workspace manifest should have
    // the same alias for the same file.
    let per_file_alias = compressed
        .lines()
        .find(|l| l.contains('α'))
        .unwrap();
    let workspace_alias_line = result
        .manifest
        .lines()
        .find(|l| l.contains("α alias:"))
        .unwrap();

    assert!(
        per_file_alias.contains("α1"),
        "per-file alias should be α1, got: {}",
        per_file_alias
    );
    assert!(
        workspace_alias_line.contains("α1"),
        "workspace alias should also be α1, got: {}",
        workspace_alias_line
    );
}

// ---------- F-17: symlink-loop protection ----------

/// F-17: a symlink loop (`loop -> ../loop`) must not cause infinite
/// recursion or an OOM. The workspace walk should detect the cycle
/// via canonical-path tracking and terminate quickly.
#[test]
fn collect_source_files_survives_symlink_loop() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path();

    // Create a subdirectory with a real .ts file.
    let sub = dir_path.join("sub");
    fs::create_dir(&sub).unwrap();
    create_ts_file(
        &sub,
        "good.ts",
        "export class Good {}\n",
    );

    // Create a symlink loop: loop_dir -> dir_path/sub/loop_dir
    // (which we're about to create).
    let loop_dir = sub.join("loop_dir");
    fs::create_dir(&loop_dir).unwrap();
    // loop_dir/loop points back to loop_dir itself (symlink loop).
    #[cfg(unix)]
    std::os::unix::fs::symlink(&loop_dir, loop_dir.join("loop")).unwrap();
    // On Windows, use the `junction` or skip if unsupported.
    #[cfg(not(unix))]
    {
        // Symlink creation requires elevated privileges on Windows;
        // just verify the non-symlink path works.
    }

    let mut entries = Vec::new();
    collect_source_files(dir_path.to_str().unwrap(), &mut entries);

    // The good.ts file should still be found.
    assert!(
        entries.iter().any(|e| e.contains("good.ts")),
        "good.ts should be found despite symlink loop, got: {:?}",
        entries
    );
}

/// F-17: `collect_source_files` respects the max depth limit.
#[test]
fn collect_source_files_respects_max_depth() {
    let dir = TempDir::new().unwrap();
    let mut current = dir.path().to_path_buf();

    // Create a deeply nested directory structure (deeper than MAX_WALK_DEPTH).
    for i in 0..40 {
        current = current.join(format!("d{}", i));
        fs::create_dir(&current).unwrap();
    }
    // Put a .ts file at the deepest level.
    create_ts_file(&current, "deep.ts", "export class Deep {}\n");

    let mut entries = Vec::new();
    collect_source_files(dir.path().to_str().unwrap(), &mut entries);

    // The deep file should NOT be found because it exceeds max depth.
    assert!(
        !entries.iter().any(|e| e.contains("deep.ts")),
        "deep.ts should NOT be found (exceeds max depth), got: {:?}",
        entries
    );
}
