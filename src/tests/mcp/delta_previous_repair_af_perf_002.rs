//! Agent MaxHeadRoom: frozen all-fidelity original oracle and independent site budget.
use super::*;
#[derive(Default, Debug, serde::Serialize)]
struct Copies {
    discarded: usize,
    tuples: usize,
    strings: usize,
    bytes: usize,
    cached: usize,
    required: Vec<Value>,
}
thread_local! {
    static ENABLED: Cell<bool> = const {Cell::new(false)};
    static COPIES: RefCell<Copies> = RefCell::new(Copies::default());
}
struct Observation(bool);
impl Observation {
    fn new() -> Self {
        COPIES.with(|v| *v.borrow_mut() = Copies::default());
        Self(ENABLED.replace(true))
    }
}
impl Drop for Observation {
    fn drop(&mut self) {
        ENABLED.set(self.0);
    }
}
// Called directly after the REAL cached_ir clone, under the original IR guard.
// Neither design clone counters nor candidate mode determine this observation.
pub(crate) fn cached(
    previous: Option<&(Vec<Vec<String>>, Option<String>)>,
    output: &[Vec<String>],
    canonical: &[Vec<String>],
) {
    if !ENABLED.get() {
        return;
    }
    assert_eq!(
        output, canonical,
        "Required cached output differs from guarded authority"
    );
    COPIES.with(|v| {
        let mut v = v.borrow_mut();
        v.cached += 1;
        v.required.push(json!(output));
        if let Some((tuples, hash)) = previous {
            v.discarded += 1;
            v.tuples += tuples.len();
            for tuple in tuples {
                for text in tuple {
                    v.strings += 1;
                    v.bytes += text.len();
                }
            }
            if let Some(hash) = hash {
                v.strings += 1;
                v.bytes += hash.len();
            }
        }
    });
}
fn required() -> Vec<Value> {
    COPIES.with(|v| std::mem::take(&mut v.borrow_mut().required))
}
#[path = "delta_previous_repair_cases_af_perf_002.rs"]
mod fixtures;
fn root() -> std::path::PathBuf {
    let root = std::env::var("AF002_DELTA_PREVIOUS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap()
                .join("target/af-perf-002/delta-previous-tests")
                .join(format!(
                    "{}-{:?}",
                    std::process::id(),
                    std::thread::current().id()
                ))
        });
    std::fs::create_dir_all(&root).unwrap();
    root
}
fn work_identity(w: &Work) -> Value {
    // Cached discarded copies are work, not consumed authority. Every required
    // capture and necessary read/publication observation remains in the oracle.
    json!({"initial_reads":w.initial_reads,"initial_bytes":w.initial_bytes,
        "second_reads":w.second_reads,"second_bytes":w.second_bytes,
        "cached_copies":w.cached_copies,"cached_tuples":w.cached_tuples,
        "cached_strings":w.cached_strings,"cached_bytes":w.cached_bytes,
        "changed_captures":w.changed_captures,"capture":w.capture,
        "publication_checks":w.publication_checks,"publication_current":w.publication_current})
}
fn add(rows: &mut Vec<Value>, value: Value) {
    eprintln!(
        "AF002_DELTA_PREVIOUS_ROW {}",
        json!({"row":rows.len(),"value":value})
    );
    rows.push(value);
}
fn error_identity(state: &McpState, file: &std::path::Path) -> Value {
    let path = file.to_string_lossy();
    let owner = state.semantic_owner_path(&path);
    let alias = state.alias_for_path(&path);
    let ctx = state.ir_context_read();
    let canonical = json!({"owner":owner,"alias":alias,
        "version":alias.as_ref().and_then(|a|ctx.file_version(a)),
        "tuples":alias.as_ref().and_then(|a|ctx.get_ir(a).cloned()),
        "hash":alias.as_ref().and_then(|a|ctx.get_source_hash(a).cloned())});
    drop(ctx);
    let resolution = crate::mcp::tool_helpers::resolve_file_path_checked(
        &path,
        Some(&file.parent().unwrap().to_string_lossy()),
        &state.config.additional_roots,
    );
    let cached_resolved = resolution
        .as_ref()
        .ok()
        .and_then(|resolved| state.cached_source_snapshot(resolved))
        .map(|s| s.as_str().to_owned());
    let cached_requested = state
        .cached_source_snapshot(&path)
        .map(|s| s.as_str().to_owned());
    json!({"canonical":canonical,"resolution":resolution,
        "cached_resolved":cached_resolved,"cached_requested":cached_requested,
        "edges":state.workspace_index_read().edge_count(),
        "semantic_edges":alias.as_ref().and_then(|a|state.semantic_edges(a)),
        "provenance":alias.as_ref().map(|a|format!("{:?}",state.semantic_snapshot(a))),
        "fidelity":alias.as_ref().and_then(|a|state.context_fidelity(a))})
}
fn lifecycle(rows: &mut Vec<Value>, root: &std::path::Path, fidelity: &str) {
    std::fs::create_dir_all(root).unwrap();
    let file = root.join("subject.ts");
    for fault in ["missing", "utf8", "directory"] {
        if file.is_file() {
            std::fs::remove_file(&file).unwrap();
        }
        if file.is_dir() {
            std::fs::remove_dir(&file).unwrap();
        }
        match fault {
            "utf8" => std::fs::write(&file, [0xff, 0xfe]).unwrap(),
            "directory" => std::fs::create_dir(&file).unwrap(),
            _ => {}
        }
        let state = state(root, false);
        let _mode = Mode::new(false, true);
        let response = call(&state, &file, "delta_code_context", fidelity);
        assert!(response.get("error").is_some());
        assert_eq!(work().previous_copies, 0);
        assert!(required().is_empty());
        add(
            rows,
            json!({"fidelity":fidelity,"fault":fault,"response":response,
            "identity":error_identity(&state,&file),"work":work_identity(&work())}),
        );
    }
    std::fs::remove_dir(&file).unwrap();
    for scenario in ["between_reads", "publication"] {
        let initial = source(8, 64, 0, false);
        std::fs::write(&file, &initial).unwrap();
        let state = state(root, false);
        assert!(
            call(&state, &file, "delta_code_context", fidelity)
                .get("error")
                .is_none()
        );
        if scenario == "publication" {
            std::fs::write(&file, format!("{initial}// compile changed\n")).unwrap();
        }
        let before = identity(&state, &file);
        let _mode = Mode::new(false, true);
        let replacement = source(9, 64, 0, true);
        let nested = std::rc::Rc::new(RefCell::new(Value::Null));
        let captured = nested.clone();
        let fidelity_owned = fidelity.to_owned();
        let hook: Hook = Box::new(move |state, path, _| {
            let target = path.to_owned();
            std::thread::spawn(move || std::fs::write(target, replacement).unwrap())
                .join()
                .unwrap();
            state.invalidate_source_cache(path);
            if scenario == "publication" {
                let response = call(
                    state,
                    std::path::Path::new(path),
                    "compress_code_context",
                    &fidelity_owned,
                );
                assert!(response.get("error").is_none(), "{response}");
                *captured.borrow_mut() = response;
            }
        });
        if scenario == "between_reads" {
            SOURCE_HOOK.with(|h| *h.borrow_mut() = Some(hook));
        } else {
            PUBLISH_HOOK.with(|h| *h.borrow_mut() = Some(hook));
        }
        let response = call(&state, &file, "delta_code_context", fidelity);
        assert_eq!(response.get("error").is_some(), scenario == "publication");
        let w = work();
        assert_eq!(w.previous_copies, 1);
        assert_eq!(w.changed_captures, 1);
        assert_eq!(
            w.capture,
            json!([before["canonical"]["tuples"], before["canonical"]["hash"]])
        );
        assert!(required().is_empty());
        add(
            rows,
            json!({"fidelity":fidelity,"scenario":scenario,"response":response,
            "before":before,"after":identity(&state,&file),"nested":nested.borrow().clone(),
            "work":work_identity(&w)}),
        );
    }
}
fn records() -> Vec<Value> {
    let root = root();
    let mut rows = Vec::new();
    for (i, pop) in [
        (0, 0, 0, 1),
        (1, 0, 0, 1),
        (8, 64, 0, 4),
        (64, 0, 0, 1),
        (8, 4096, 0, 1),
        (8, 0, 4096, 16),
    ]
    .into_iter()
    .enumerate()
    {
        for fidelity in ["low", "medium", "high", "edit", "verbatim"] {
            let path = root.join(format!("population-{i}-{fidelity}"));
            std::fs::create_dir_all(&path).unwrap();
            for (value, w) in fixtures::sequence(&path, pop, fidelity, false) {
                add(
                    &mut rows,
                    json!({"population":i,"fidelity":fidelity,"identity":value,"work":work_identity(&w)}),
                );
            }
        }
    }
    for fidelity in ["low", "medium", "high", "edit", "verbatim"] {
        let path = root.join(format!("persistent-{fidelity}"));
        std::fs::create_dir_all(&path).unwrap();
        for (value, w) in fixtures::sequence(&path, (8, 64, 0, 4), fidelity, true) {
            add(
                &mut rows,
                json!({"persistent":true,"fidelity":fidelity,"identity":value,"work":work_identity(&w)}),
            );
        }
        lifecycle(
            &mut rows,
            &root.join(format!("lifecycle-{fidelity}")),
            fidelity,
        );
    }
    assert_eq!(rows.len(), 380);
    rows
}
fn compare(rows: &[Value]) {
    if let Ok(path) = std::env::var("AF002_DELTA_PREVIOUS_ORACLE") {
        let expected: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(rows.len(), expected.len());
        for (i, (actual, expected)) in rows.iter().zip(&expected).enumerate() {
            assert_eq!(actual, expected, "Frozen delta authority row {i}");
        }
    }
    for row in rows {
        eprintln!("AF002_DELTA_PREVIOUS_FROZEN {row}");
    }
    COPIES.with(|v| {
        assert!(
            v.borrow().cached > 0,
            "Required cached-return site was not exercised"
        )
    });
}
#[test]
fn af002_delta_previous_frozen_semantics() {
    let _serial = crate::protocol::handler_response_serial();
    let _observation = Observation::new();
    compare(&records());
}
#[test]
fn af002_delta_previous_unused_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let _observation = Observation::new();
    compare(&records());
    COPIES.with(|v| {
        let v = v.borrow();
        eprintln!("AF002_DELTA_PREVIOUS_UNUSED {}", json!(&*v));
        assert_eq!(
            (v.discarded, v.tuples, v.strings, v.bytes),
            (0, 0, 0, 0),
            "Registered delta unused previous-authority budget"
        );
    });
}
