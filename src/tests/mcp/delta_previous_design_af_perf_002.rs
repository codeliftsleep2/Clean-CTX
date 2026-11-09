//! Agent MaxHeadRoom: test-only delayed capture, never release authority.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
type Hook = Box<dyn FnOnce(&McpState, &str, &str)>;
thread_local! {
    static HISTORICAL: Cell<bool>=const {Cell::new(false)};
    static DELAY: Cell<bool>=const {Cell::new(false)};
    static ACTIVE: Cell<bool>=const {Cell::new(false)};
    static OBSERVE: Cell<bool>=const {Cell::new(true)};
    static WORK: RefCell<Work>=RefCell::new(Work::default());
    static SOURCE_HOOK: RefCell<Option<Hook>>=const {RefCell::new(None)};
    static PUBLISH_HOOK: RefCell<Option<Hook>>=const {RefCell::new(None)};
}
#[derive(Default, Clone, Debug, serde::Serialize)]
struct Work {
    previous_copies: usize,
    tuples: usize,
    strings: usize,
    bytes: usize,
    outer_capacity: usize,
    inner_capacity: usize,
    string_capacity: usize,
    cached_copies: usize,
    cached_tuples: usize,
    cached_strings: usize,
    cached_bytes: usize,
    discarded_copies: usize,
    initial_reads: usize,
    initial_bytes: usize,
    second_reads: usize,
    second_bytes: usize,
    changed_captures: usize,
    publication_checks: usize,
    publication_current: usize,
    capture: Value,
    events: Vec<&'static str>,
}
struct Mode(bool, bool, bool, bool);
impl Mode {
    fn historical(delay: bool, observe: bool) -> Self {
        let mode = Self::new(delay, observe);
        HISTORICAL.set(true);
        mode
    }
    fn new(delay: bool, observe: bool) -> Self {
        WORK.with(|w| *w.borrow_mut() = Work::default());
        Self(
            DELAY.replace(delay),
            ACTIVE.replace(true),
            OBSERVE.replace(observe),
            HISTORICAL.replace(false),
        )
    }
}
impl Drop for Mode {
    fn drop(&mut self) {
        DELAY.set(self.0);
        ACTIVE.set(self.1);
        OBSERVE.set(self.2);
        HISTORICAL.set(self.3);
        drop(SOURCE_HOOK.with(|h| h.borrow_mut().take()));
        drop(PUBLISH_HOOK.with(|h| h.borrow_mut().take()));
    }
}
fn record(f: impl FnOnce(&mut Work)) {
    if ACTIVE.get() && OBSERVE.get() {
        WORK.with(|w| f(&mut w.borrow_mut()));
    }
}
pub(super) fn historical_early() -> bool {
    HISTORICAL.get() && !DELAY.get()
}
pub(super) fn observing() -> bool {
    OBSERVE.get()
}
pub(super) fn previous(tuples: &Vec<Vec<String>>) {
    record(|w| {
        w.previous_copies += 1;
        w.tuples += tuples.len();
        w.outer_capacity += tuples.capacity();
        w.events.push("previous_clone");
        for t in tuples {
            w.inner_capacity += t.capacity();
            for s in t {
                w.strings += 1;
                w.bytes += s.len();
                w.string_capacity += s.capacity();
            }
        }
    });
}
pub(super) fn cached(tuples: &[Vec<String>], previous: bool) {
    record(|w| {
        w.cached_copies += 1;
        w.cached_tuples += tuples.len();
        w.discarded_copies += usize::from(previous);
        for t in tuples {
            for s in t {
                w.cached_strings += 1;
                w.cached_bytes += s.len();
            }
        }
        w.events.push("cached_return_clone");
    });
}
pub(super) fn second(bytes: usize) {
    record(|w| {
        w.second_reads += 1;
        w.second_bytes += bytes;
        w.events.push("second_read");
    });
}
pub(super) fn after_source(state: &McpState, path: &str, source: &str) {
    record(|w| {
        w.initial_reads += 1;
        w.initial_bytes += source.len();
        w.events.push("initial_read");
    });
    if let Some(h) = SOURCE_HOOK.with(|h| h.borrow_mut().take()) {
        h(state, path, source);
    }
}
pub(super) fn changed(previous: Option<&(Vec<Vec<String>>, Option<String>)>) {
    record(|w| {
        w.changed_captures += usize::from(previous.is_some());
        w.capture = json!(previous);
        w.events.push("release_for_compile");
    });
}
pub(super) fn before_publish(state: &McpState, path: &str) {
    record(|w| w.events.push("before_publication"));
    if let Some(h) = PUBLISH_HOOK.with(|h| h.borrow_mut().take()) {
        h(state, path, "");
    }
}
pub(super) fn publication(current: bool) {
    record(|w| {
        w.publication_checks += 1;
        w.publication_current += usize::from(current);
        w.events.push("publication_equality");
    });
}
fn work() -> Work {
    WORK.with(|w| w.borrow().clone())
}
fn state(root: &std::path::Path, persist: bool) -> McpState {
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.proxy.port = 0;
    cfg.persistence.enabled = persist;
    cfg.persistence.auto_save = persist;
    cfg.persistence.db_path = root.join("state.db").to_string_lossy().into_owned();
    cfg.additional_roots = vec![root.to_string_lossy().into_owned()];
    let state = McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    state
}
fn call(state: &McpState, file: &std::path::Path, tool: &str, fidelity: &str) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        tool,
        &json!({"arguments":{
        "filePath":file.to_string_lossy(),"workspaceRoot":file.parent().unwrap().to_string_lossy(),"fidelity":fidelity}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
fn source(methods: usize, body: usize, padding: usize, changed: bool) -> String {
    let mut text = format!("// {}\nexport class Subject {{\n", "共有".repeat(padding));
    for i in 0..methods {
        text.push_str(&format!(
            "m{i}(): string {{return \"{}-{i}\";}}\n",
            "共有".repeat(body)
        ));
    }
    if changed {
        text.push_str("added(): void {}\n");
    }
    text.push_str("}\n");
    text
}
fn identity(state: &McpState, file: &std::path::Path) -> Value {
    let path = file.to_string_lossy();
    let owner = state.semantic_owner_path(&path);
    let alias = state.alias_for_path(&path);
    let ctx = state.ir_context_read();
    let canonical = json!({"owner":owner,"alias":alias,"version":alias.as_ref().and_then(|a|ctx.file_version(a)),
        "tuples":alias.as_ref().and_then(|a|ctx.get_ir(a).cloned()),
        "hash":alias.as_ref().and_then(|a|ctx.get_source_hash(a).cloned())});
    drop(ctx);
    let resolved = crate::mcp::tool_helpers::resolve_file_path_checked(
        &path,
        Some(&file.parent().unwrap().to_string_lossy()),
        &state.config.additional_roots,
    )
    .unwrap();
    let cached = state
        .cached_source_snapshot(&resolved)
        .map(|s| s.as_str().to_owned());
    json!({"canonical":canonical,"cached":cached,"edges":state.workspace_index_read().edge_count(),
        "semantic_edges":alias.as_ref().and_then(|a|state.semantic_edges(a)),
        "provenance":alias.as_ref().map(|a|format!("{:?}",state.semantic_snapshot(a))),
        "fidelity":alias.as_ref().and_then(|a|state.context_fidelity(a))})
}
#[path = "delta_previous_cases_af_perf_002.rs"]
mod cases;

#[path = "delta_previous_repair_af_perf_002.rs"]
pub(crate) mod repair;

#[path = "delta_previous_integrated_af_perf_002.rs"]
mod integrated;
