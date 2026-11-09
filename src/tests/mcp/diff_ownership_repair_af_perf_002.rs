//! Frozen original-only public contract and independent actual-handler copy budget.
use super::*;
#[derive(Default)]
struct Copies {
    count: usize,
    bytes: usize,
}
thread_local! {static COPIES: RefCell<Copies> = RefCell::new(Copies::default());}
pub(crate) fn owned_copy(source: &str) {
    COPIES.with(|v| {
        let mut v = v.borrow_mut();
        v.count += 1;
        v.bytes += source.len();
    });
}
fn add(rows: &mut Vec<Value>, value: Value) {
    eprintln!(
        "AF002_DIFF_SOURCE_ROW {}",
        json!({"row":rows.len(),"value":value})
    );
    rows.push(value);
}
fn fixture() -> std::path::PathBuf {
    let path = std::env::var("AF002_DIFF_SOURCE_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap()
                .join("target/af-perf-002/diff-source-tests")
                .join(format!(
                    "{}-{:?}",
                    std::process::id(),
                    std::thread::current().id()
                ))
        });
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn sequence(
    rows: &mut Vec<Value>,
    root: &std::path::Path,
    name: &str,
    methods: usize,
    body: usize,
    padding: usize,
    persistent: bool,
) {
    use crate::mcp::context_store::ContextStore;
    std::fs::create_dir_all(root).unwrap();
    let file = root.join("subject.ts");
    let initial = source(methods, body, padding, false);
    std::fs::write(&file, &initial).unwrap();
    let mut state = state(root, persistent);
    for phase in [
        "cold",
        "warm",
        "repeat",
        "changed",
        "changed_warm",
        "cache_replaced",
        "restart",
    ] {
        let current = if matches!(
            phase,
            "changed" | "changed_warm" | "cache_replaced" | "restart"
        ) {
            source(methods, body, padding, true)
        } else {
            initial.clone()
        };
        if phase == "changed" {
            std::fs::write(&file, &current).unwrap();
        }
        if phase == "cache_replaced" {
            state.invalidate_source_cache(&file.to_string_lossy());
        }
        if phase == "restart" {
            drop(state);
            state = super::state(root, persistent);
        }
        let _mode = Mode::new(false, true);
        let count = if phase == "repeat" { 4 } else { 1 };
        let mut responses = Vec::new();
        for _ in 0..count {
            responses.push(dispatch(&state, &file, name));
        }
        assert!(
            responses.iter().all(|r| r.get("error").is_none()),
            "{responses:?}"
        );
        let work = work();
        assert_eq!(work.read_attempts, count);
        assert_eq!(work.helper_calls, count);
        assert_eq!(work.acquired_snapshots, count);
        assert_eq!(work.helper_input_bytes, current.len() * count);
        let identity = identity(&state, &file, name);
        assert_eq!(
            identity["baseline_hash"],
            state.cache_read().compute_hash(current.as_bytes())
        );
        assert_eq!(identity["cached_source"], current);
        let durable = if persistent {
            let guard = state.persistence_store_lock();
            let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
            assert!(
                sqlite
                    .load_latest(&file.to_string_lossy())
                    .unwrap()
                    .is_none()
            );
            true
        } else {
            assert!(state.persistence_store_lock().is_none());
            false
        };
        add(
            rows,
            json!({"fidelity":name,"methods":methods,"body":body,"padding":padding,
            "persistent":durable,"phase":phase,"responses":responses,"identity":identity}),
        );
    }
}
fn errors(rows: &mut Vec<Value>, root: &std::path::Path) {
    std::fs::create_dir_all(root).unwrap();
    let file = root.join("error.ts");
    if file.is_file() {
        std::fs::remove_file(&file).unwrap();
    }
    if file.is_dir() {
        std::fs::remove_dir(&file).unwrap();
    }
    for fault in ["missing", "invalid_utf8", "directory"] {
        match fault {
            "invalid_utf8" => std::fs::write(&file, [0xff, 0xfe]).unwrap(),
            "directory" => {
                std::fs::remove_file(&file).unwrap();
                std::fs::create_dir(&file).unwrap();
            }
            _ => {}
        }
        let state = state(root, false);
        let _mode = Mode::new(false, true);
        let response = dispatch(&state, &file, "high");
        assert!(response.get("error").is_some());
        assert_eq!(work().acquired_snapshots, 0);
        assert_eq!(work().helper_calls, 0);
        add(rows, json!({"fault":fault,"response":response}));
    }
    std::fs::remove_dir(&file).unwrap();
}
fn lifetime(rows: &mut Vec<Value>, root: &std::path::Path) {
    std::fs::create_dir_all(root).unwrap();
    let file = root.join("subject.ts");
    for name in ["low", "medium", "high", "verbatim", "edit"] {
        let old = source(8, 64, 0, false);
        let new = format!("{}// replacement\n", source(8, 64, 0, true));
        std::fs::write(&file, &old).unwrap();
        let state = state(root, false);
        let _mode = Mode::new(false, true);
        let expected = old.clone();
        let replacement = new.clone();
        HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(move |state, path, borrowed| {
                let edited = path.to_owned();
                let text = replacement.clone();
                std::thread::spawn(move || std::fs::write(edited, text).unwrap())
                    .join()
                    .unwrap();
                state.invalidate_source_cache(path);
                let cached = state.read_source(path).unwrap();
                assert_eq!(cached.as_str(), replacement);
                assert_eq!(borrowed, expected);
                drop(cached);
            }))
        });
        let response = dispatch(&state, &file, name);
        assert!(response.get("error").is_none());
        let first = identity(&state, &file, name);
        assert_eq!(
            first["baseline_hash"],
            state.cache_read().compute_hash(old.as_bytes())
        );
        assert_eq!(first["cached_source"], new);
        let followup = dispatch(&state, &file, name);
        assert!(followup.get("error").is_none());
        let after = identity(&state, &file, name);
        assert_eq!(
            after["baseline_hash"],
            state.cache_read().compute_hash(new.as_bytes())
        );
        assert_eq!(after["cached_source"], new);
        add(
            rows,
            json!({"lifetime":name,"response":response,"first":first,"followup":followup,"after":after}),
        );
    }
}
fn records() -> Vec<Value> {
    let root = fixture();
    let mut rows = Vec::new();
    COPIES.with(|v| *v.borrow_mut() = Copies::default());
    for (i, (methods, body, padding)) in [
        (0, 0, 0),
        (1, 0, 0),
        (8, 64, 0),
        (64, 0, 0),
        (8, 4096, 0),
        (8, 0, 4096),
    ]
    .into_iter()
    .enumerate()
    {
        for name in ["low", "medium", "high", "verbatim", "edit"] {
            sequence(
                &mut rows,
                &root.join(format!("case-{i}-{name}")),
                name,
                methods,
                body,
                padding,
                false,
            );
        }
    }
    sequence(&mut rows, &root.join("persistent"), "high", 8, 64, 0, true);
    errors(&mut rows, &root.join("errors"));
    lifetime(&mut rows, &root.join("lifetime"));
    assert_eq!(rows.len(), 225);
    rows
}
fn compare(rows: &[Value]) {
    if let Ok(path) = std::env::var("AF002_DIFF_SOURCE_ORACLE") {
        let expected: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(rows.len(), expected.len());
        for (i, (actual, expected)) in rows.iter().zip(&expected).enumerate() {
            assert_eq!(
                actual, expected,
                "Frozen original diff public/cache/currentness row {i}"
            );
        }
    }
    for row in rows {
        eprintln!("AF002_DIFF_SOURCE_PUBLIC {row}");
    }
}
#[test]
fn af002_diff_source_frozen_semantics() {
    let _serial = crate::protocol::handler_response_serial();
    let rows = records();
    compare(&rows);
}
#[test]
fn af002_diff_source_copy_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let rows = records();
    compare(&rows);
    COPIES.with(|v| {
        let v = v.borrow();
        eprintln!(
            "AF002_DIFF_SOURCE_COPY {}",
            json!({"copies":v.count,"bytes":v.bytes})
        );
        assert_eq!(
            (v.count, v.bytes),
            (0, 0),
            "Registered diff full-source-copy budget"
        );
    });
}
