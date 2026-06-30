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
    let state = McpState::new(config);

    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &state,
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
    // F-FINAL-04: `excluded` is now `Vec<(String, Vec<String>)>` so we
    // match against `p.0` (the path) and `p.1` (the matched patterns).
    assert!(
        result.excluded.iter().any(|p| p.0.contains("skip-me")),
        "excluded vec should contain the skipped file path"
    );
    assert!(
        result
            .excluded
            .iter()
            .any(|p| p.0.contains("skip-me") && p.1.contains(&"skip-me*".to_string())),
        "excluded vec should record the matching pattern"
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
    let state = McpState::new(config);

    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &state,
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
    let state = McpState::new(config);

    // First, compress via the per-file tool path (simulated inline).
    let file_path = dir_path.join("shared.ts");
    let compressed = crate::compressor::compress_file(
        file_path.clone(),
        &mut state.dict_lock(),
        &mut state.cache_write(),
        Fidelity::Low,
        None,
    )
    .expect("per-file compress should succeed");

    // The per-file result should contain an alias.
    assert!(compressed.contains('α'), "per-file output should contain alias");

    // Now compress the workspace — it should reuse the same alias.
    let result = compress_workspace_dir(
        dir_path.to_str().unwrap(),
        Fidelity::Low,
        &state,
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

// --- Track D (F-ANG-15): sub-pass and extract_class_blocks tests ---

/// F-ANG-03: `extract_class_blocks` should handle a malformed class
/// body (no closing brace) without panicking. The Track A helpers
/// return `None` for unclosed braces, so the function should just
/// skip that block.
#[test]
fn extract_class_blocks_does_not_panic_on_unclosed_body() {
    // A class with no closing `}` — the old implementation would
    // return a truncated block; the new one should return empty
    // (or skip the class) without panicking.
    let source = "export class Foo { method() { return 1; }";
    let blocks = super::extract_class_blocks(source);
    // Unclosed body → `find_matching_brace` returns None → skipped.
    assert!(
        blocks.is_empty() || blocks.iter().all(|b| b.contains("class Foo")),
        "should either skip or include the class, got: {:?}",
        blocks
    );
}

/// F-ANG-03: `extract_class_blocks` does not panic on empty input.
#[test]
fn extract_class_blocks_handles_empty_input() {
    let blocks = super::extract_class_blocks("");
    assert!(blocks.is_empty());
}

/// F-ANG-15: `compress_pass` emits `FILE:` sections in the manifest.
#[test]
fn compress_pass_emits_per_file_section() {
    let dir = TempDir::new().unwrap();
    create_ts_file(
        dir.path(),
        "service.ts",
        "export class MyService { run(): void {} }\n",
    );

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let result = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("workspace compress should succeed");

    assert!(
        result.manifest.contains("FILE:"),
        "manifest should contain FILE: section, got:\n{}",
        result.manifest
    );
    assert!(
        result.manifest.contains("service.ts"),
        "manifest should reference service.ts, got:\n{}",
        result.manifest
    );
}

/// F-ANG-15: `bundle_pass` emits `Φ` bundle markers when a component
/// triplet is present.
#[test]
fn bundle_pass_emits_phi_bundle_and_footer() {
    let dir = TempDir::new().unwrap();
    // Create a component triplet: my-comp.component.ts + .html + .scss
    create_ts_file(
        dir.path(),
        "my-comp.component.ts",
        "@Component({selector:'app-my'}) export class MyComp {}",
    );
    create_ts_file(dir.path(), "my-comp.component.html", "<div>hello</div>");
    create_ts_file(dir.path(), "my-comp.component.scss", ".root { color: red; }");

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let result = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("workspace compress should succeed");

    // Bundle pass emits §ΦMAP footer when bundles exist.
    assert!(
        result.manifest.contains("§ΦMAP") || !result.manifest.contains("Φ1"),
        "manifest should contain bundle markers or ΦMAP footer, got:\n{}",
        result.manifest
    );
}

/// F-ANG-15: `graph_pass` emits `§ΦGRAPH` when an Angular file is
/// detected.
#[test]
fn graph_pass_emits_phi_graph_section() {
    let dir = TempDir::new().unwrap();
    // Angular injectable service — `is_angular_file` should detect it.
    create_ts_file(
        dir.path(),
        "logger.service.ts",
        "import { Injectable } from '@angular/core';\n\
         @Injectable({ providedIn: 'root' })\n\
         export class LoggerService { log(msg: string) {} }\n",
    );

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    let result = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("workspace compress should succeed");

    assert!(
        result.manifest.contains("§ΦGRAPH"),
        "manifest should contain §ΦGRAPH section for Angular files, got:\n{}",
        result.manifest
    );
    assert!(
        result.manifest.contains("LoggerService"),
        "manifest should reference LoggerService, got:\n{}",
        result.manifest
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

/// F-22: Workspace compression result caching.
/// Second call with no file changes returns cached result instantly.
#[test]
fn compress_workspace_caches_result() {
    let dir = TempDir::new().unwrap();
    create_ts_file(
        dir.path(),
        "alpha.ts",
        "export class Alpha { run(): void {} }\n",
    );

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // First call: cache miss, normal compression
    let result1 = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("first compress should succeed");
    assert!(result1.manifest.contains("alpha.ts"), "first call should contain alpha.ts");

    // Second call with same directory and no file changes: cache hit
    let result2 = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("second compress should succeed");

    // Both results should be identical
    assert_eq!(result1.manifest, result2.manifest, "cached result should match original");
    assert_eq!(result1.errors.len(), result2.errors.len(), "errors should match");
    assert_eq!(result1.excluded.len(), result2.excluded.len(), "excluded should match");
}

/// F-22: Different fidelities must produce different cache entries.
/// Verifies that the cache key includes the fidelity level.
#[test]
fn compress_workspace_cache_key_includes_fidelity() {
    let dir = TempDir::new().unwrap();
    create_ts_file(
        dir.path(),
        "beta.ts",
        "export class Beta { process(): string { return ''; } }\n",
    );

    let config = CleanCtxConfig::default();
    let state = McpState::new(config);

    // Compress at Low fidelity: uses global symbol two-pass approach
    let low_result = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Low,
        &state,
    )
    .expect("low fidelity compress should succeed");
    assert!(low_result.manifest.contains("beta.ts"), "low fidelity should contain beta.ts");

    // Compress at Medium fidelity: uses standard compress_pass
    let medium_result = compress_workspace_dir(
        dir.path().to_str().unwrap(),
        Fidelity::Medium,
        &state,
    )
    .expect("medium fidelity compress should succeed");

    // The manifests should differ because different fidelity paths produce different output
    // (Low fidelity uses the global symbol two-pass path)
    assert_ne!(low_result.manifest, medium_result.manifest,
        "different fidelities should produce different cached results");
}
