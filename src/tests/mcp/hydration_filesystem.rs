use super::{
    TraversalStats, is_built_in_excluded_directory, is_supported_source, last_test_traversal_stats,
    root_key, scan,
};
use crate::mcp::McpState;
use crate::mcp::tool_handlers::hydration::TEST_PROJECT_HYDRATION_SERIALIZE;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn serial_guard() -> std::sync::MutexGuard<'static, ()> {
    TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn state(additional_roots: &[PathBuf], exclusions: &[&str]) -> McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional_roots
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();
    config.exclude_patterns = exclusions
        .iter()
        .map(|pattern| (*pattern).to_string())
        .collect();
    let state = McpState::new(config);
    *state.graph_bridge_lock() = None;
    state
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::TempDir::new().unwrap();
    std::fs::create_dir(root.path().join(".git")).unwrap();
    root
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

fn candidates(state: &McpState, roots: &[PathBuf], query: &str) -> Vec<String> {
    scan(state, roots, query).candidates
}

fn assert_one_read(stats: TraversalStats) {
    assert_eq!(stats.supported_files_considered, 1, "{stats:?}");
    assert_eq!(stats.files_read, 1, "{stats:?}");
}

#[test]
fn red_g1_custom_gitignored_directory_is_pruned() {
    let _serial = serial_guard();
    let root = repository();
    write(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    write(
        &root.path().join("coverage/Ignored.ts"),
        "export class Target {}\n",
    );
    write(
        &root.path().join("IgnoredFile.ts"),
        "export class Target {}\n",
    );
    write(
        &root.path().join(".gitignore"),
        "coverage/\nIgnoredFile.ts\n",
    );

    let found = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(found, vec![root_key(&root.path().join("src/Target.ts"))]);
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn red_g2_nested_gitignore_is_honored() {
    let _serial = serial_guard();
    let root = repository();
    write(&root.path().join("src/.gitignore"), "generated/\n");
    write(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    write(
        &root.path().join("src/generated/Ignored.ts"),
        "export class Target {}\n",
    );

    let found = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(found, vec![root_key(&root.path().join("src/Target.ts"))]);
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn red_g3_built_in_exclusions_remain_independent() {
    let _serial = serial_guard();
    let root = repository();
    write(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    for directory in [
        ".git",
        "node_modules",
        "bin",
        "obj",
        "dist",
        "build",
        "target",
        ".venv",
        "venv",
        "__pycache__",
    ] {
        write(
            &root.path().join(directory).join("Ignored.ts"),
            "export class Target {}\n",
        );
    }

    let found = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(found, vec![root_key(&root.path().join("src/Target.ts"))]);
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn red_g4_all_exclusion_layers_are_additive() {
    let _serial = serial_guard();
    let root = repository();
    write(&root.path().join(".gitignore"), "coverage/\n");
    write(
        &root.path().join("src/Target.ts"),
        "export class Target {}\n",
    );
    for directory in ["coverage", "custom-generated", "node_modules"] {
        write(
            &root.path().join(directory).join("Ignored.ts"),
            "export class Target {}\n",
        );
    }

    let found = candidates(
        &state(&[], &["custom-generated"]),
        &[root.path().to_path_buf()],
        "Target",
    );

    assert_eq!(found, vec![root_key(&root.path().join("src/Target.ts"))]);
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn red_g5_gitignored_sources_are_never_read() {
    let _serial = serial_guard();
    let root = repository();
    write(&root.path().join(".gitignore"), "coverage/\n");
    write(&root.path().join("src/Target.rs"), "pub struct Target;\n");
    for index in 0..128 {
        write(
            &root.path().join(format!("coverage/Ignored{index}.rs")),
            "pub struct Target;\n",
        );
    }

    let _ = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_one_read(last_test_traversal_stats());
}

#[test]
fn gitignore_negation_preserves_reincluded_source() {
    let _serial = serial_guard();
    let root = repository();
    write(
        &root.path().join(".gitignore"),
        "generated/*\n!generated/Keep.ts\n",
    );
    write(
        &root.path().join("generated/Drop.ts"),
        "export class Target {}\n",
    );
    write(
        &root.path().join("generated/Keep.ts"),
        "export class Target {}\n",
    );

    let found = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(
        found,
        vec![root_key(&root.path().join("generated/Keep.ts"))]
    );
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn hidden_sources_remain_searchable_without_an_ignore_rule() {
    let _serial = serial_guard();
    let root = repository();
    write(
        &root.path().join(".hidden/Target.ts"),
        "export class Target {}\n",
    );

    let found = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(
        found,
        vec![root_key(&root.path().join(".hidden/Target.ts"))]
    );
    assert_one_read(last_test_traversal_stats());
}

#[test]
fn red_r1_parallel_results_equal_sequential_reference_set() {
    let _serial = serial_guard();
    let root = repository();
    let mut eligible = Vec::new();
    for index in 0..32 {
        let path = root.path().join(format!("src/Part{index}.ts"));
        write(
            &path,
            if index % 3 == 0 {
                "export class Target {}\n"
            } else {
                "export class Other {}\n"
            },
        );
        eligible.push(path);
    }
    let mut expected: Vec<String> = eligible
        .iter()
        .filter(|path| std::fs::read_to_string(path).unwrap().contains("Target"))
        .map(|path| root_key(path))
        .collect();
    expected.sort();

    let actual = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    assert_eq!(actual, expected);
}

#[test]
fn red_r2_parallel_candidate_order_is_deterministic() {
    let _serial = serial_guard();
    let root = repository();
    for index in (0..64).rev() {
        write(
            &root.path().join(format!("src/Part{index:02}.ts")),
            "export class Target {}\n",
        );
    }
    let scan_state = state(&[], &[]);
    let roots = [root.path().to_path_buf()];
    let first = candidates(&scan_state, &roots, "Target");
    for _ in 0..8 {
        assert_eq!(candidates(&scan_state, &roots, "Target"), first);
    }
    assert!(first.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn red_r3_reads_every_eligible_source_exactly_once() {
    let _serial = serial_guard();
    let root = repository();
    write(&root.path().join(".gitignore"), "ignored/\n");
    for index in 0..40 {
        write(
            &root.path().join(format!("src/Eligible{index}.rs")),
            "pub struct Target;\n",
        );
        write(
            &root.path().join(format!("ignored/Skipped{index}.rs")),
            "pub struct Target;\n",
        );
    }

    let _ = candidates(&state(&[], &[]), &[root.path().to_path_buf()], "Target");

    let stats = last_test_traversal_stats();
    assert_eq!(stats.supported_files_considered, 40, "{stats:?}");
    assert_eq!(stats.files_read, 40, "{stats:?}");
}

#[test]
fn red_r4_multi_root_discovery_is_complete_and_deterministic() {
    let _serial = serial_guard();
    let primary = repository();
    let additional_a = repository();
    let additional_b = repository();
    for (root, name) in [
        (primary.path(), "Primary"),
        (additional_a.path(), "AdditionalA"),
        (additional_b.path(), "AdditionalB"),
    ] {
        write(&root.join(".gitignore"), "ignored/\n");
        write(
            &root.join(format!("src/{name}.ts")),
            "export class SharedTarget {}\n",
        );
        write(
            &root.join("ignored/Noise.ts"),
            "export class SharedTarget {}\n",
        );
    }
    let roots = vec![
        primary.path().to_path_buf(),
        additional_a.path().to_path_buf(),
        additional_b.path().to_path_buf(),
    ];

    let found = candidates(&state(&roots[1..], &[]), &roots, "SharedTarget");

    assert_eq!(found.len(), 3);
    assert!(found.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(last_test_traversal_stats().files_read, 3);
}

#[test]
fn red_r5_no_cbm_reverse_edges_remain_authoritative() {
    let _serial = serial_guard();
    let primary = repository();
    let additional = repository();
    write(
        &primary.path().join("src/ServiceA.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable()\nexport class ServiceA {}\n",
    );
    write(&additional.path().join(".gitignore"), "ignored/\n");
    let consumer = "import { Component } from '@angular/core';\nimport { ServiceA } from './ServiceA';\n@Component({ selector: 'x-consumer' })\nexport class Consumer { constructor(private service: ServiceA) {} }\n";
    write(&additional.path().join("src/Consumer.ts"), consumer);
    write(
        &additional.path().join("ignored/IgnoredConsumer.ts"),
        consumer,
    );
    let hydration_state = state(&[additional.path().to_path_buf()], &[]);

    let report = super::super::hydrate_workspace_index(
        &hydration_state,
        "reverse_edges",
        "ServiceA",
        Some(&primary.path().to_string_lossy()),
    );
    let index = hydration_state.workspace_index_read();

    assert_eq!(report.discovery_provider, "filesystem");
    assert!(report.discovery_completed);
    assert_eq!(report.candidates_discovered, 2);
    assert_eq!(report.candidates_compiled, 2);
    assert_eq!(
        index
            .reverse_edges_by_identity("angular", "Service", "ServiceA")
            .len(),
        1
    );
    assert_eq!(last_test_traversal_stats().files_read, 2);
}

#[derive(Debug)]
struct Measurement {
    directories: usize,
    considered: usize,
    read: usize,
    matched: usize,
    traversal: Duration,
    read_match: Duration,
    total: Duration,
}

impl Measurement {
    fn summary(&self) -> String {
        format!(
            "directories={} considered={} read={} matched={} traversal={:?} read_match={:?} total={:?}",
            self.directories,
            self.considered,
            self.read,
            self.matched,
            self.traversal,
            self.read_match,
            self.total
        )
    }
}

fn sequential_unignored_reference(state: &McpState, root: &Path, query: &str) -> Measurement {
    let total_started = Instant::now();
    let traversal_started = Instant::now();
    let config = state.config.clone();
    let mut builder = WalkBuilder::new(root);
    builder
        .standard_filters(false)
        .hidden(false)
        .follow_links(false)
        .filter_entry(move |entry| {
            !is_built_in_excluded_directory(entry)
                && !config.is_excluded(&entry.path().to_string_lossy())
        });
    let mut directories = 0;
    let mut paths = Vec::new();
    for entry in builder.build().filter_map(Result::ok) {
        if entry.file_type().is_some_and(|kind| kind.is_dir()) {
            directories += 1;
        } else if entry.file_type().is_some_and(|kind| kind.is_file())
            && is_supported_source(entry.path())
        {
            paths.push(root_key(entry.path()));
        }
    }
    paths.sort();
    paths.dedup();
    let traversal = traversal_started.elapsed();
    let read_match_started = Instant::now();
    let matched = paths
        .iter()
        .filter(|path| {
            state
                .read_source(path)
                .is_ok_and(|source| source.contains(query))
        })
        .count();
    Measurement {
        directories,
        considered: paths.len(),
        read: paths.len(),
        matched,
        traversal,
        read_match: read_match_started.elapsed(),
        total: total_started.elapsed(),
    }
}

fn parallel_measurement(state: &McpState, root: &Path, query: &str) -> Measurement {
    let result = scan(state, &[root.to_path_buf()], query);
    let stats = last_test_traversal_stats();
    Measurement {
        directories: stats.directories_entered,
        considered: stats.supported_files_considered,
        read: stats.files_read,
        matched: result.candidates.len(),
        traversal: stats.traversal_elapsed,
        read_match: stats.read_match_elapsed,
        total: stats.total_elapsed,
    }
}

#[test]
#[ignore = "development performance measurement"]
fn measure_gitignore_and_parallel_read_performance() {
    let _serial = serial_guard();
    for (scenario, ignored, eligible, bytes_per_file) in [
        ("small", 0, 4, 128),
        ("ignored-heavy", 512, 1, 128),
        ("read-heavy", 0, 128, 32_768),
    ] {
        let root = repository();
        write(&root.path().join(".gitignore"), "ignored/\n");
        let padding = "x".repeat(bytes_per_file);
        for index in 0..eligible {
            write(
                &root.path().join(format!("src/Eligible{index}.rs")),
                &format!("pub struct Target; // {padding}\n"),
            );
        }
        for index in 0..ignored {
            write(
                &root.path().join(format!("ignored/Ignored{index}.rs")),
                &format!("pub struct Target; // {padding}\n"),
            );
        }

        let before = sequential_unignored_reference(&state(&[], &[]), root.path(), "Target");
        let after = parallel_measurement(&state(&[], &[]), root.path(), "Target");
        eprintln!(
            "PERF {scenario} before=[{}] after=[{}]",
            before.summary(),
            after.summary()
        );
    }
}
