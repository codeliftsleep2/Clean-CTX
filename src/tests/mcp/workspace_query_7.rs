// Filesystem hydration safe-default regressions (RED-S1..S9).

use crate::mcp::tool_handlers::hydration::{
    TEST_PROJECT_HYDRATION_SERIALIZE, TraversalStats, clear_test_project_search_results,
    last_test_traversal_stats,
};
use std::path::{Path, PathBuf};

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    let guard = TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    *crate::mcp::tool_handlers::hydration::TEST_HYDRATION_CANDIDATES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = None;
    clear_test_project_search_results();
    guard
}

fn state(additional: &[PathBuf], exclusions: &[&str]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    config.exclude_patterns = exclusions
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    let state = crate::mcp::McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn call(
    state: &crate::mcp::McpState,
    query_type: &str,
    name: &str,
    root: &Path,
) -> (serde_json::Value, super::super::hydration::HydrationReport) {
    let (result, _, _, report) = super::run_query_with_hydration(
        state,
        query_type,
        name,
        Some(&root.to_string_lossy()),
        |index| {
            let value = match query_type {
                "find_entities" => serde_json::to_value(index.find_entities_by_name(name)),
                "reverse_edges" => serde_json::to_value(
                    index.reverse_edges_by_identity("angular", "Service", name),
                ),
                other => panic!("unsupported test query: {other}"),
            }
            .unwrap_or_default();
            let count = value.as_array().map_or(0, Vec::len);
            (value, count)
        },
    );
    (result, report)
}

fn write_source(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

fn assert_single_legitimate_read(stats: TraversalStats) {
    assert_eq!(stats.supported_files_considered, 1, "{stats:?}");
    assert_eq!(stats.files_read, 1, "{stats:?}");
}

#[test]
fn red_s1_default_config_prunes_node_modules_before_content_scanning() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    for index in 0..2048 {
        write_source(
            &root
                .path()
                .join(format!("node_modules/pkg{index}/Noise.ts")),
            "export class Target {}\n",
        );
    }

    let (result, report) = call(&state(&[], &[]), "find_entities", "Target", root.path());

    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(report.candidates_discovered, 1);
    let stats = last_test_traversal_stats();
    assert_eq!(stats.directories_entered, 2, "{stats:?}");
    assert_single_legitimate_read(stats);
}

#[test]
fn red_s2_default_config_prunes_dotnet_build_trees() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("src/Target.cs"),
        "public class Target {}\n",
    );
    write_source(
        &root.path().join("bin/Noise.cs"),
        "public class Target {}\n",
    );
    write_source(
        &root.path().join("obj/Noise.cs"),
        "public class Target {}\n",
    );

    let (_, report) = call(&state(&[], &[]), "find_entities", "Target", root.path());

    assert_eq!(report.candidates_discovered, 1);
    assert_single_legitimate_read(last_test_traversal_stats());
}

#[test]
fn red_s3_default_config_prunes_rust_target_tree() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(&root.path().join("src/Target.rs"), "pub struct Target;\n");
    write_source(
        &root.path().join("target/debug/Noise.rs"),
        "pub struct Target;\n",
    );

    let (_, report) = call(&state(&[], &[]), "find_entities", "Target", root.path());

    assert_eq!(report.candidates_discovered, 1);
    assert_single_legitimate_read(last_test_traversal_stats());
}

#[test]
fn red_s4_default_config_prunes_python_environment_and_cache_trees() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    for directory in [".venv", "venv", "__pycache__"] {
        write_source(
            &root.path().join(directory).join("Noise.ts"),
            "export class Target {}\n",
        );
    }

    let (_, report) = call(&state(&[], &[]), "find_entities", "Target", root.path());

    assert_eq!(report.candidates_discovered, 1);
    assert_single_legitimate_read(last_test_traversal_stats());
}

#[test]
fn red_s5_git_metadata_is_never_recursively_scanned() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    write_source(
        &root.path().join(".git/objects/Noise.ts"),
        "export class Target {}\n",
    );

    let (_, report) = call(&state(&[], &[]), "find_entities", "Target", root.path());

    assert_eq!(report.candidates_discovered, 1);
    assert_single_legitimate_read(last_test_traversal_stats());
}

#[test]
fn red_s6_user_exclusions_are_additive_to_safe_defaults() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_source(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    write_source(
        &root.path().join("generated/Noise.ts"),
        "export class Target {}\n",
    );
    write_source(
        &root.path().join("node_modules/Noise.ts"),
        "export class Target {}\n",
    );

    let (_, report) = call(
        &state(&[], &["generated"]),
        "find_entities",
        "Target",
        root.path(),
    );

    assert_eq!(report.candidates_discovered, 1);
    assert_single_legitimate_read(last_test_traversal_stats());
}

#[test]
fn red_s7_large_legitimate_source_directory_remains_searchable() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    for index in 0..32 {
        write_source(
            &root.path().join(format!("large_domain/Part{index}.ts")),
            if index == 31 {
                "export class LegitimateTarget {}\n"
            } else {
                "export class Unrelated {}\n"
            },
        );
    }

    let (result, report) = call(
        &state(&[], &[]),
        "find_entities",
        "LegitimateTarget",
        root.path(),
    );

    assert_eq!(report.candidates_discovered, 1);
    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(last_test_traversal_stats().files_read, 32);
}

#[test]
fn red_s8_safe_defaults_apply_to_primary_and_additional_roots() {
    let _serial = serial_guard();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_source(
        &primary.path().join("src/Primary.ts"),
        "export class SharedTarget {}\n",
    );
    write_source(
        &additional.path().join("src/Additional.ts"),
        "export class SharedTarget {}\n",
    );
    write_source(
        &primary.path().join("node_modules/Noise.ts"),
        "export class SharedTarget {}\n",
    );
    write_source(
        &additional.path().join("target/Noise.ts"),
        "export class SharedTarget {}\n",
    );

    let (_, report) = call(
        &state(&[additional.path().to_path_buf()], &[]),
        "find_entities",
        "SharedTarget",
        primary.path(),
    );

    assert_eq!(report.candidates_discovered, 2);
    assert_eq!(last_test_traversal_stats().files_read, 2);
}

#[test]
fn red_s9_no_cbm_multi_root_reverse_edges_completes_authoritatively() {
    let _serial = serial_guard();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    write_source(
        &primary.path().join("src/ServiceA.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable()\nexport class ServiceA {}\n",
    );
    write_source(
        &additional.path().join("src/Consumer.ts"),
        "import { Component } from '@angular/core';\nimport { ServiceA } from './ServiceA';\n@Component({ selector: 'x-consumer' })\nexport class Consumer { constructor(private service: ServiceA) {} }\n",
    );
    for directory in [
        "node_modules",
        "bin",
        "obj",
        "dist",
        "build",
        "target",
        ".venv",
        "venv",
        "__pycache__",
        ".git",
    ] {
        write_source(
            &additional.path().join(directory).join("Noise.ts"),
            "export class ServiceA {}\n",
        );
    }

    let (result, report) = call(
        &state(&[additional.path().to_path_buf()], &[]),
        "reverse_edges",
        "ServiceA",
        primary.path(),
    );

    assert_eq!(result.as_array().unwrap().len(), 1);
    assert_eq!(report.discovery_provider, "filesystem");
    assert!(report.discovery_completed);
    assert_eq!(report.candidates_discovered, 2);
    assert_eq!(report.candidates_compiled, 2);
    assert_eq!(last_test_traversal_stats().files_read, 2);
}
