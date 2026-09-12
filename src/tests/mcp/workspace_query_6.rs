// Filesystem hydration regressions for workspace_query (RED-F1..F10).

use crate::cbm::GraphBridge;
use crate::mcp::tool_handlers::hydration::{
    TEST_PROJECT_HYDRATION_SERIALIZE, TestProjectReadiness, clear_test_project_search_results,
    set_test_project_readiness, set_test_project_search_results,
};
use std::collections::HashMap;
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

fn state(additional: &[PathBuf]) -> crate::mcp::McpState {
    let mut config = crate::tests::test_config();
    config.cbm.enabled = false;
    config.additional_roots = additional
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
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
) -> (
    serde_json::Value,
    super::super::hydration::HydrationReport,
    bool,
) {
    let (result, _, attempted, report) = super::run_query_with_hydration(
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
    (result, report, attempted)
}

fn write_service(root: &Path) {
    std::fs::write(
        root.join("ServiceA.ts"),
        "import { Injectable } from '@angular/core';\n@Injectable()\nexport class ServiceA {}\n",
    )
    .unwrap();
}

fn write_consumer(root: &Path, file: &str, class_name: &str) {
    std::fs::write(
        root.join(file),
        format!(
            "import {{ Component }} from '@angular/core';\nimport {{ ServiceA }} from './ServiceA';\n@Component({{ selector: 'x-{class_name}' }})\nexport class {class_name} {{ constructor(private service: ServiceA) {{}} }}\n"
        ),
    )
    .unwrap();
}

#[test]
fn red_f1_cbm_absent_still_hydrates_uncompiled_file() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(root.path().join("Target.ts"), "export class Target {}\n").unwrap();
    let state = state(&[]);

    let (result, report, attempted) = call(&state, "find_entities", "Target", root.path());

    assert!(attempted);
    assert_eq!(report.discovery_provider, "filesystem");
    assert!(report.discovery_completed);
    assert_eq!(report.candidates_compiled, 1);
    assert_eq!(result.as_array().unwrap().len(), 1);
}

#[test]
fn red_f2_additional_root_is_scanned_without_cbm() {
    let _serial = serial_guard();
    let primary = tempfile::TempDir::new().unwrap();
    let additional = tempfile::TempDir::new().unwrap();
    std::fs::write(
        additional.path().join("AdditionalOnly.ts"),
        "export class AdditionalOnly {}\n",
    )
    .unwrap();
    let state = state(&[additional.path().to_path_buf()]);

    let (result, report, _) = call(&state, "find_entities", "AdditionalOnly", primary.path());

    assert_eq!(report.candidates_compiled, 1);
    assert_eq!(result.as_array().unwrap().len(), 1);
}

#[test]
fn red_f3_reverse_edges_discovers_every_consumer() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    write_service(root.path());
    write_consumer(root.path(), "A.ts", "A");
    write_consumer(root.path(), "B.ts", "B");
    let state = state(&[]);

    let (result, report, _) = call(&state, "reverse_edges", "ServiceA", root.path());

    assert_eq!(report.candidates_discovered, 3);
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[test]
fn red_f4_textual_false_positive_does_not_fabricate_facts() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(
        root.path().join("Noise.ts"),
        "// Phantom\nexport const text = 'Phantom';\n",
    )
    .unwrap();
    let state = state(&[]);

    let (result, report, _) = call(&state, "find_entities", "Phantom", root.path());

    assert_eq!(report.candidates_discovered, 1);
    assert!(result.as_array().unwrap().is_empty());
}

#[test]
fn red_f5_successful_empty_filesystem_discovery_is_truthful() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    let state = state(&[]);

    let (_, report, attempted) = call(&state, "find_entities", "Absent", root.path());

    assert!(attempted);
    assert_eq!(report.discovery_provider, "filesystem");
    assert!(report.discovery_completed);
    assert_eq!(report.discovery_status, "completed");
    assert_eq!(report.candidates_discovered, 0);
}

#[test]
fn red_f6_healthy_cbm_zero_is_distinct_from_filesystem_zero() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    let state = state(&[]);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    let bridge = GraphBridge::try_create_with_roots(&bridge_config, root.path(), &[]);
    let slug = crate::cbm::bridge::cbm_project_slug(&root.path().canonicalize().unwrap());
    *state.graph_bridge_lock() = Some(bridge);
    set_test_project_search_results(HashMap::from([(slug, Ok(Vec::new()))]));

    let (_, report, _) = call(&state, "find_entities", "Absent", root.path());

    assert_eq!(report.discovery_provider, "cbm");
    assert!(!report.fallback_occurred);
    clear_test_project_search_results();
}

#[test]
fn red_f7_healthy_cbm_does_not_run_filesystem_fallback() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(root.path().join("Hidden.ts"), "export class Hidden {}\n").unwrap();
    let state = state(&[]);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        root.path(),
        &[],
    ));
    let slug = crate::cbm::bridge::cbm_project_slug(&root.path().canonicalize().unwrap());
    set_test_project_search_results(HashMap::from([(slug, Ok(Vec::new()))]));

    let (result, report, _) = call(&state, "find_entities", "Hidden", root.path());

    assert_eq!(report.discovery_provider, "cbm");
    assert_eq!(report.candidates_compiled, 0);
    assert!(result.as_array().unwrap().is_empty());
    clear_test_project_search_results();
}

#[test]
fn red_f8_degraded_cbm_falls_back_to_filesystem() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    std::fs::write(
        root.path().join("Fallback.ts"),
        "export class Fallback {}\n",
    )
    .unwrap();
    let state = state(&[]);
    let bridge_config = crate::cbm::config::CbmConfig {
        enabled: false,
        ..Default::default()
    };
    *state.graph_bridge_lock() = Some(GraphBridge::try_create_with_roots(
        &bridge_config,
        root.path(),
        &[],
    ));
    let slug = crate::cbm::bridge::cbm_project_slug(&root.path().canonicalize().unwrap());
    set_test_project_search_results(HashMap::from([(slug.clone(), Ok(Vec::new()))]));
    set_test_project_readiness(HashMap::from([(slug, TestProjectReadiness::Unavailable)]));

    let (result, report, _) = call(&state, "find_entities", "Fallback", root.path());

    assert_eq!(report.discovery_provider, "filesystem");
    assert!(report.fallback_occurred);
    assert_eq!(report.fallback_reason, Some("cbm_unavailable"));
    assert_eq!(result.as_array().unwrap().len(), 1);
    clear_test_project_search_results();
}

#[test]
fn red_f9_filesystem_discovery_is_exhaustive() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    for index in 0..7 {
        std::fs::write(
            root.path().join(format!("Target{index}.ts")),
            "export class RepeatedTarget {}\n",
        )
        .unwrap();
    }
    let state = state(&[]);

    let (result, report, _) = call(&state, "find_entities", "RepeatedTarget", root.path());

    assert_eq!(report.candidates_discovered, 7);
    assert_eq!(report.candidates_compiled, 7);
    assert_eq!(result.as_array().unwrap().len(), 7);
}

#[test]
fn red_f10_duplicate_hits_and_already_indexed_files_compile_once() {
    let _serial = serial_guard();
    let root = tempfile::TempDir::new().unwrap();
    let indexed = root.path().join("Indexed.ts");
    std::fs::write(&indexed, "export class Target {}\n// Target Target\n").unwrap();
    std::fs::write(root.path().join("Fresh.ts"), "export class Target {}\n").unwrap();
    let state = state(&[]);
    assert!(super::super::hydration::compile_candidate(
        &state,
        &indexed.to_string_lossy(),
        Some(&root.path().to_string_lossy()),
    ));

    let (result, report, _) = call(&state, "find_entities", "Target", root.path());

    assert_eq!(report.candidates_discovered, 2);
    assert_eq!(report.candidates_compiled, 1);
    assert_eq!(result.as_array().unwrap().len(), 2);
}
