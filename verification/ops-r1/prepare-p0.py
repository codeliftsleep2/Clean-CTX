"""Prepare unapplied test-only P0 instrumentation/regression diff; never executes it."""
import difflib
import hashlib
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "verification/ops-r1/proposals/p0"

OBSERVER = r'''//! Isolated-child OPS-R1 diagnostics, not production authority or a fitness budget.
use std::sync::atomic::{AtomicU64, Ordering};
static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub(crate) fn active() -> bool { ACTIVE.load(Ordering::Relaxed) }
pub(crate) static COUNTS: [AtomicU64; 15] = [const { AtomicU64::new(0) }; 15];
const NAMES: [&str; 15] = ["inventory_attempts", "inventory_entries", "nominations",
    "nominated_owners", "compile_attempts", "compile_completed", "project_acquisitions",
    "project_units", "host_read_misses", "query_compilations", "witness_checks",
    "publication_attempts", "publication_completed", "project_loop_iterations", "returned_batch_high_water"];
pub(crate) fn batch_results(count: usize) {
    if active() { COUNTS[14].fetch_max(count as u64, Ordering::Relaxed); }
}
pub(crate) fn record(slot: usize, count: usize) {
    if active() { COUNTS[slot].fetch_add(count as u64, Ordering::Relaxed); }
}
pub(crate) fn snapshot() -> serde_json::Value {
    let values: serde_json::Map<String, serde_json::Value> = NAMES.iter().enumerate()
        .map(|(i, name)| ((*name).to_string(), COUNTS[i].load(Ordering::Relaxed).into())).collect();
    serde_json::Value::Object(values)
}
pub(crate) fn sample(directory: &std::path::Path) -> Sample {
    use std::sync::{Arc, atomic::AtomicBool};
    let stop = Arc::new(AtomicBool::new(false));
    if std::env::var("OPS_R1_CAPTURE").ok().as_deref() != Some("1") {
        return Sample { stop, worker: None };
    }
    for count in &COUNTS { count.store(0, Ordering::Relaxed); }
    ACTIVE.store(true, Ordering::Relaxed);
    let worker_stop = Arc::clone(&stop);
    let directory = directory.to_path_buf();
    let worker = std::thread::spawn(move || {
        use std::io::Write;
        let mut file = std::fs::File::create(directory.join("stages.jsonl")).unwrap();
        while !worker_stop.load(Ordering::Relaxed) {
            writeln!(file, "{}", snapshot()).unwrap();
            file.flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        writeln!(file, "{}", snapshot()).unwrap();
    });
    Sample { stop, worker: Some(worker) }
}
pub(crate) struct Sample {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Drop for Sample {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() { worker.join().unwrap(); }
        ACTIVE.store(false, Ordering::Relaxed);
    }
}
'''

TESTS = r'''//! Proposed observational witnesses: no unapproved work/time budget asserted.
use crate::mcp::McpState;
use serde_json::json;
fn fixture() -> (tempfile::TempDir, std::path::PathBuf, usize, std::path::PathBuf) {
    let count: usize = std::env::var("OPS_R1_UNITS").unwrap_or_else(|_| "1".into()).parse().unwrap();
    assert!([1, 4, 16].contains(&count));
    if count > 1 { assert_eq!(std::env::var("OPS_R1_CAPTURE").ok().as_deref(), Some("1")); }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    crate::tests::angular_authority_fixture::install(&root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","target":"ES2022","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    for i in 0..count {
        let peer = if i + 1 == count { "class Peer0 {}\n" } else { "" };
        std::fs::write(root.join(format!("owner{i}.ts")), format!("{peer}class Probe{i} {{ peer?: Peer0; }}\n")).unwrap();
    }
    let evidence = std::env::var("OPS_R1_EVIDENCE_DIR").map(std::path::PathBuf::from).unwrap_or_else(|_| temp.path().join("evidence"));
    std::fs::create_dir_all(&evidence).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(&root).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        path.is_file().then(|| json!({"file":path.file_name().unwrap().to_string_lossy(),"sha256":crate::cache::content_hash(&std::fs::read(&path).unwrap())}))
    }).collect();
    for relative in ["node_modules/@angular/core/package.json", "node_modules/@angular/core/index.d.ts"] {
        files.push(json!({"file":relative,"sha256":crate::cache::content_hash(&std::fs::read(root.join(relative)).unwrap())}));
    }
    if std::env::var("OPS_R1_CAPTURE").ok().as_deref() == Some("1") {
        for input in &files {
            let relative = input["file"].as_str().unwrap();
            let destination = evidence.join("fixture-inputs").join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(root.join(relative), destination).unwrap();
        }
    }
    std::fs::write(evidence.join("fixture.json"), serde_json::to_vec_pretty(&json!({"root":root,"units":count,"files":files,"resolver_mappings":0})).unwrap()).unwrap();
    (temp, root, count, evidence)
}
#[test]
fn ops_r1_absent_route_observation() {
    let _serial = crate::protocol::handler_response_serial();
    let (_temp, root, count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    let state = McpState::new(config);
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let response = super::angular_resolver_registration_support::raw(&state, json!({
        "workspaceRoot":root, "type":"resolver_registrations", "direction":"forward", "route_path":"__ops_r1_absent__"
    }));
    std::fs::write(evidence.join("query-response.json"), serde_json::to_vec_pretty(&response).unwrap()).unwrap();
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
    assert!(response.get("error").is_none(), "{response:#}");
    let result = &response["result"];
    assert_ne!(result["isError"], true, "{result:#}");
    let result = &result["structuredContent"];
    assert_eq!(result["count"], 0);
    assert_eq!(result["registrations"], json!([]));
    assert_eq!(result["coverage"]["source_complete"], false);
    assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(std::fs::read_dir(&root).unwrap().filter(|entry| entry.as_ref().unwrap().path().extension().is_some_and(|extension| extension == "ts")).count(), count);
}
#[test]
fn ops_r1_fixed_job_project_observation() {
    let (_temp, root, _count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    let state = McpState::new(config);
    let file = root.join("owner0.ts").to_string_lossy().into_owned();
    let source = std::sync::Arc::new(std::fs::read_to_string(&file).unwrap());
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let results = state.semantic_compiler().compile(&state.config, vec![super::executor::CompilationJob {
        authorized_roots:vec![root], typescript_project:None, file_path:file.clone(), source,
        fidelity:crate::compression::Fidelity::High, path_alias:"ops-r1-owner".into(),
        previous_version:0, canonical_owner:file, skip_set:None
    }]);
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
    assert_eq!(results.len(), 1);
    assert!(results[0].result.is_ok(), "component compilation failed");
    if crate::ops_r1_observation::active() {
        assert_eq!(crate::ops_r1_observation::COUNTS[4].load(std::sync::atomic::Ordering::Relaxed), 1);
    }
}
'''


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    changes = {}
    def text(path):
        return changes.get(path, (ROOT / path).read_text(encoding="utf-8"))
    def hook(path, old, new):
        value = text(path)
        assert value.count(old) == 1, (path, old[:90], value.count(old))
        changes[path] = value.replace(old, new)
    def observe(slot, expression="1", indent="    "):
        return f'{indent}#[cfg(all(test, feature = "angular"))]\n{indent}crate::ops_r1_observation::record({slot}, {expression});\n'

    changes["src/tests/architecture/ops_r1_observation.rs"] = OBSERVER
    changes["src/tests/mcp/ops_r1_absent_route.rs"] = TESTS
    changes["src/lib.rs"] = text("src/lib.rs") + '\n#[cfg(all(test, feature = "angular"))]\n#[path = "tests/architecture/ops_r1_observation.rs"]\npub(crate) mod ops_r1_observation;\n'
    changes["src/mcp/semantic_compilation.rs"] = text("src/mcp/semantic_compilation.rs") + '\n#[cfg(all(test, feature = "angular"))]\n#[path = "../tests/mcp/ops_r1_absent_route.rs"]\nmod ops_r1_repro_tests;\n'
    hook("src/mcp/tool_handlers/hydration/filesystem.rs",
         '    let (paths, attempted, traversal_completed, _) = enumerate_source_paths(state, roots);\n    build_inventory(state, paths, attempted, traversal_completed)',
         observe(0) + '    let (paths, attempted, traversal_completed, _) = enumerate_source_paths(state, roots);\n    let inventory = build_inventory(state, paths, attempted, traversal_completed);\n' + observe(1, 'inventory.entries.len()') + '    inventory')
    hook("src/mcp/tool_handlers/hydration/resolver_registrations.rs", '    let discovered = candidates.len();',
         observe(2) + observe(3, 'candidates.len()') + '    let discovered = candidates.len();')
    hook("src/mcp/semantic_compilation/executor.rs", '            jobs.into_par_iter()', '            let results: Vec<_> = jobs.into_par_iter()')
    hook("src/mcp/semantic_compilation/executor.rs", '                .collect()\n        })',
         '                .collect();\n            #[cfg(all(test, feature = "angular"))]\n            crate::ops_r1_observation::batch_results(results.len());\n            results\n        })')
    hook("src/mcp/semantic_compilation/executor.rs", '                .map(|job| {\n',
         '                .map(|job| {\n' + observe(4, indent='                    '))
    hook("src/mcp/semantic_compilation/executor.rs", '                    CompilationBatchResult {\n',
         observe(5, indent='                    ') + '                    CompilationBatchResult {\n')
    hook("src/ir/typescript_resolution/program.rs", '        let result = (|| {\n',
         observe(6, indent='        ') + '        let result = (|| {\n')
    hook("src/ir/typescript_resolution/program.rs", '            loop {\n',
         '            loop {\n' + observe(13, indent='                '))
    hook("src/ir/typescript_resolution/program.rs", '                let unit = self\n',
         observe(7, indent='                ') + '                let unit = self\n')
    hook("src/ir/typescript_resolution/host.rs", '        let source = match self.read_bytes(&path) {',
         observe(8, indent='        ') + '        let source = match self.read_bytes(&path) {')
    hook("src/ir/typescript_resolution/host.rs", '        crate::work_observations_af_perf_002::witness(self);',
         '        crate::work_observations_af_perf_002::witness(self);\n' + observe(10, indent='        ').rstrip())
    hook("src/compression/capture_pipeline.rs", '        Query::new(\n',
         observe(9, indent='        ') + '        Query::new(\n')
    hook("src/compression/capture_pipeline.rs", '            dynamic_query = Query::new(&language, query_string)?;',
         observe(9, indent='            ') + '            dynamic_query = Query::new(&language, query_string)?;')
    hook("src/mcp/tool_handlers/hydration/publication.rs", '    debug_assert_eq!(compiled.file_path, publication.validated);',
         observe(11) + '    debug_assert_eq!(compiled.file_path, publication.validated);')
    # Completion is observed at the caller only after a successful commit returns.
    hook("src/mcp/tool_handlers/hydration/publication.rs", '        terminal.push(commit_candidate(state, publication, compiled)?);',
         '        terminal.push(commit_candidate(state, publication, compiled)?);\n' + observe(12, indent='        ').rstrip())

    metadata = []
    patches = {"registration-and-tests": [], "test-only-observations": []}
    for path, candidate in changes.items():
        baseline = (ROOT / path).read_text(encoding="utf-8") if (ROOT / path).exists() else ""
        assert len(candidate.splitlines()) <= 615, path
        patch = "".join(difflib.unified_diff(baseline.splitlines(True), candidate.splitlines(True),
            fromfile="a/" + path if baseline else "/dev/null", tofile="b/" + path))
        category = "registration-and-tests" if path.startswith("src/tests/") or path in ["src/lib.rs", "src/mcp/semantic_compilation.rs"] else "test-only-observations"
        patches[category].append(patch)
        metadata.append({"path":path,"baseline_bytes_sha256":digest((ROOT/path).read_bytes()) if (ROOT/path).exists() else None,
            "candidate_normalized_lf_sha256":digest(candidate.encode('utf-8')),"candidate_lines":len(candidate.splitlines())})
    for category, parts in patches.items():
        (OUT / (category + ".patch")).write_text("".join(parts), encoding="utf-8", newline="\n")
    inputs = subprocess.check_output(["git", "ls-files", "-z", "--", "src", "Cargo.toml", "Cargo.lock", "build.rs", ".cargo"], cwd=ROOT).decode().split("\0")
    baseline_rows = []
    for name in sorted(n for n in inputs if n):
        data = (ROOT/name).read_bytes()
        if pathlib.PurePosixPath(name).suffix in ['.rs','.toml'] or name=='Cargo.lock': data=data.replace(b'\r\n',b'\n')
        baseline_rows.append({'path':name,'sha256':digest(data)})
    input_rows = []
    for name in sorted(set(n for n in inputs if n) | set(changes)):
        data = changes[name].encode('utf-8') if name in changes else (ROOT/name).read_bytes()
        if pathlib.PurePosixPath(name).suffix in ['.rs','.toml'] or name=='Cargo.lock': data=data.replace(b'\r\n',b'\n')
        input_rows.append({'path':name,'sha256':digest(data)})
    value = {"standing":"UNAPPLIED, UNCOMPILED AND UNRUN; final operator approval required",
        "source_head":subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip(),
        "files":metadata,"source_tree_algorithm":"SHA256 of compact JSON sorted tracked src/build-input path/SHA256 rows; Rust/TOML/lock normalized LF, other fixtures/query inputs byte-exact; separators comma/colon",
        "baseline_source_tree_sha256":digest(json.dumps(baseline_rows,separators=(',',':')).encode()),
        "candidate_source_tree_sha256":digest(json.dumps(input_rows,separators=(',',':')).encode()),
        "candidate_source_file_count":len(input_rows),
        "test_identities":["mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_absent_route_observation",
                           "mcp::semantic_compilation::ops_r1_repro_tests::ops_r1_fixed_job_project_observation"],
        "instrumentation_limits":["Counters are isolated-child test observations, not authority or thresholds.",
            "No exact retained-object/allocation-byte counter is proposed; pending batch size and OS working set remain distinct from leak proof.",
            "Project-loop iterations observe program expansion, not all subsystem retry loops.",
            "Query::new hooks cover capture pipeline sites, not every query compiler in the repository.",
            "Host read misses are host snapshot acquisition attempts, not all filesystem IO.",
            "These fixtures establish observations and empty-result semantics; no work-budget RED is claimed before the legitimate relationship is established."]}
    (OUT/'proposal-index.json').write_text(json.dumps(value,indent=2)+'\n',encoding='utf-8')
    print('Prepared unapplied P0 source registration, fixtures and 15 test-only observations. No test was built or executed.')


if __name__ == '__main__':
    main()
