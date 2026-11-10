//! Agent MaxHeadRoom: actual registered diff_commits work, no alternative.
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
#[derive(Default, Clone, Debug, serde::Serialize)]
struct Work {
    commands: std::collections::BTreeMap<String, usize>,
    stdout_bytes: usize,
    stderr_bytes: usize,
    failed_commands: usize,
    changed_files: usize,
    dedup_sets: usize,
    dedup_entries: usize,
    dedup_path_bytes: usize,
    untracked_output_bytes: usize,
    snapshots: usize,
    snapshot_source_bytes: usize,
    admissions: usize,
    admission_bytes: usize,
    body_copies: usize,
    formatted_bytes: usize,
    copied_body_bytes: usize,
}
thread_local! {
    static ACTIVE: Cell<bool> = const {Cell::new(false)};
    static WORK: RefCell<Work> = RefCell::new(Work::default());
}
struct Observation(bool);
impl Observation {
    fn new() -> Self {
        WORK.with(|w| *w.borrow_mut() = Work::default());
        Self(ACTIVE.replace(true))
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        ACTIVE.set(self.0);
    }
}
fn record(f: impl FnOnce(&mut Work)) {
    if ACTIVE.get() {
        WORK.with(|w| f(&mut w.borrow_mut()));
    }
}
pub(crate) fn command(args: &[&str]) {
    record(|w| {
        *w.commands
            .entry(args.first().copied().unwrap_or("").to_owned())
            .or_default() += 1
    });
}
pub(crate) fn output(stdout: usize, stderr: usize, ok: bool) {
    record(|w| {
        w.stdout_bytes += stdout;
        w.stderr_bytes += stderr;
        w.failed_commands += usize::from(!ok);
    });
}
pub(crate) fn changed(count: usize) {
    record(|w| w.changed_files += count);
}
pub(crate) fn dedup(paths: &std::collections::HashSet<String>, bytes: usize) {
    record(|w| {
        w.dedup_sets += 1;
        w.dedup_entries += paths.len();
        w.dedup_path_bytes += paths.iter().map(String::len).sum::<usize>();
        w.untracked_output_bytes += bytes;
    });
}
pub(crate) fn snapshot(text: &str) {
    record(|w| {
        w.snapshots += 1;
        w.snapshot_source_bytes += text.len();
    });
}
pub(crate) fn admit(bytes: usize) {
    record(|w| {
        w.admissions += 1;
        w.admission_bytes += bytes;
    });
}
pub(crate) fn body_copy(formatted: &str, trimmed: &str) {
    record(|w| {
        w.body_copies += 1;
        w.formatted_bytes += formatted.len();
        w.copied_body_bytes += trimmed.len();
    });
}
fn git(root: &std::path::Path, args: &[&str]) {
    let result = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}
fn source(methods: usize, padding: usize, changed: bool) -> String {
    let mut text = String::from("export class Sample {\n");
    for i in 0..methods {
        text.push_str(&format!(
            "m{i}(): string {{return '{}-{}';}}\n",
            "共有".repeat(padding),
            usize::from(changed)
        ));
    }
    text.push_str("}\n");
    text
}
fn fixture(
    count: usize,
    methods: usize,
    padding: usize,
    kind: &str,
    committed: bool,
) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    git(root.path(), &["config", "user.name", "Fixture"]);
    git(
        root.path(),
        &["config", "user.email", "fixture@example.invalid"],
    );
    git(root.path(), &["config", "core.autocrlf", "false"]);
    std::fs::write(root.path().join(".gitignore"), "ignored.tmp\n").unwrap();
    for i in 0..count {
        let ext = if kind == "text" { "txt" } else { "ts" };
        if kind != "added" {
            std::fs::write(
                root.path().join(format!("file-{i}.{ext}")),
                source(methods, padding, false),
            )
            .unwrap();
        }
    }
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-q", "-m", "baseline"]);
    for i in 0..count {
        let ext = if kind == "text" { "txt" } else { "ts" };
        let path = root.path().join(format!("file-{i}.{ext}"));
        match kind {
            "deleted" => std::fs::remove_file(path).unwrap(),
            "renamed" => {
                std::fs::rename(path, root.path().join(format!("renamed-{i}.ts"))).unwrap()
            }
            "binary" => std::fs::write(path, [0, 255, 1]).unwrap(),
            "utf8" => std::fs::write(path, [255, 254]).unwrap(),
            _ => std::fs::write(path, source(methods, padding, true)).unwrap(),
        }
    }
    std::fs::write(root.path().join("ignored.tmp"), "ignored").unwrap();
    if !committed && kind == "renamed" {
        git(root.path(), &["add", "-A"]);
    }
    if committed {
        git(root.path(), &["add", "-A"]);
        git(
            root.path(),
            &["commit", "-q", "--allow-empty", "-m", "current"],
        );
    }
    root
}
fn state(root: &std::path::Path, max_files: usize, max_size: usize) -> crate::mcp::McpState {
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.proxy.port = 0;
    cfg.persistence.enabled = false;
    cfg.resource_limits.max_workspace_files = max_files;
    cfg.resource_limits.max_file_size_bytes = max_size;
    cfg.additional_roots
        .push(root.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &crate::mcp::McpState, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        "diff_commits",
        &json!({"arguments":args}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
#[test]
fn af002_gitdiff_registered_work_discovery() {
    let _serial = crate::protocol::handler_response_serial();
    let mut rows = 0;
    for (count, methods, padding, repeats, kind) in [
        (0, 1, 0, 1, "modified"),
        (1, 1, 0, 1, "modified"),
        (8, 1, 0, 1, "modified"),
        (1, 32, 0, 1, "modified"),
        (1, 8, 4096, 1, "modified"),
        (8, 1, 0, 4, "modified"),
        (1, 1, 0, 1, "added"),
        (1, 1, 0, 1, "deleted"),
        (1, 8, 0, 1, "renamed"),
        (1, 1, 0, 1, "text"),
        (1, 1, 0, 1, "binary"),
        (1, 1, 0, 1, "utf8"),
    ] {
        for committed in [false, true] {
            let root = fixture(count, methods, padding, kind, committed);
            let state = state(root.path(), 128, 2_000_000);
            let mut expected_warm = None;
            for request in 0..repeats {
                let _observation = Observation::new();
                let args = if committed {
                    json!({"workspaceRoot":root.path(),"fromRef":"HEAD~1","toRef":"HEAD","fidelity":"edit"})
                } else {
                    json!({"workspaceRoot":root.path(),"fromRef":"HEAD","fidelity":"edit"})
                };
                let response = dispatch(&state, args);
                assert!(response.get("error").is_none(), "{response}");
                assert_eq!(response["result"]["_meta"]["fileCount"], count);
                if let Some(ref expected) = expected_warm {
                    assert_eq!(
                        &response, expected,
                        "Warm cache-hint deduplication contract"
                    );
                } else {
                    assert!(state.config.cache.enabled);
                    assert!(response["result"]["_meta"]["cache_hints"].is_object());
                    // Build the exact expected NEXT response. Retain the complete
                    // actual cold/warm responses in evidence; never normalize them.
                    let mut expected = response.clone();
                    assert!(
                        expected["result"]["_meta"]
                            .as_object_mut()
                            .unwrap()
                            .remove("cache_hints")
                            .is_some()
                    );
                    expected_warm = Some(expected);
                }
                {
                    let metrics = state.cache_metrics_lock();
                    assert_eq!(metrics.misses, 1);
                    assert_eq!(metrics.hits, request);
                }
                let work = WORK.with(|w| w.borrow().clone());
                assert_eq!(work.changed_files, count);
                assert_eq!(work.dedup_sets, usize::from(!committed));
                assert_eq!(work.failed_commands, 0);
                if kind == "modified" && count > 0 {
                    assert_eq!(work.body_copies, count);
                    assert_eq!(work.snapshots, count * 2);
                }
                if matches!(kind, "binary" | "utf8" | "deleted" | "text") {
                    assert_eq!(work.body_copies, 0);
                }
                assert!(state.persistence_store_lock().is_none());
                for i in 0..count {
                    assert!(
                        state
                            .alias_for_path(
                                &root.path().join(format!("file-{i}.ts")).to_string_lossy()
                            )
                            .is_none()
                    );
                }
                eprintln!(
                    "AF002_GITDIFF_DISCOVERY {}",
                    json!({"files":count,"methods":methods,"padding":padding,
                "kind":kind,"committed":committed,"request":request,"response":response,"work":work})
                );
                rows += 1;
            }
        }
    }
    assert_eq!(rows, 30);
}
#[test]
fn af002_gitdiff_limits_discovery() {
    let _serial = crate::protocol::handler_response_serial();
    for (files, size) in [(0, 2_000_000), (1, 2_000_000), (8, 1)] {
        let root = fixture(8, 8, 0, "modified", true);
        let state = state(root.path(), files, size);
        let _observation = Observation::new();
        let response = dispatch(
            &state,
            json!({"workspaceRoot":root.path(),"fromRef":"HEAD~1","toRef":"HEAD"}),
        );
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["result"]["_meta"]["fileCount"], 8);
        let work = WORK.with(|w| w.borrow().clone());
        if files == 0 || size == 1 {
            assert_eq!(work.snapshots, 0);
            assert_eq!(work.body_copies, 0);
        }
        eprintln!(
            "AF002_GITDIFF_LIMIT {}",
            json!({"max_files":files,"max_size":size,"response":response,"work":work})
        );
    }
}
#[test]
fn af002_gitdiff_errors_discovery() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let state = state(root.path(), 8, 1000);
    for args in [
        json!({"workspaceRoot":root.path()}),
        json!({"workspaceRoot":root.path(),"fromRef":"--unsafe"}),
        json!({"workspaceRoot":root.path(),"fromRef":"HEAD"}),
    ] {
        let _observation = Observation::new();
        let response = dispatch(&state, args);
        assert!(response.get("error").is_some());
        assert_eq!(WORK.with(|w| w.borrow().snapshots), 0);
    }
}
