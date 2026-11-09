//! Agent MaxHeadRoom: test-only ownership comparison at the actual handler boundary.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Default, Clone, Debug, serde::Serialize)]
struct Work {
    read_attempts: usize,
    acquired_snapshots: usize,
    owned_copies: usize,
    copied_bytes: usize,
    retained_arc_transfers: usize,
    owned_drops: usize,
    arc_drops: usize,
    helper_calls: usize,
    helper_input_bytes: usize,
}
type Hook = Box<dyn FnOnce(&McpState, &str, &str)>;
thread_local! {
    static ALTERNATIVE: Cell<bool> = const { Cell::new(false) };
    static OBSERVE: Cell<bool> = const { Cell::new(false) };
    static TIMED: Cell<bool> = const { Cell::new(false) };
    static HISTORICAL: Cell<bool> = const { Cell::new(false) };
    static WORK: RefCell<Work> = RefCell::new(Work::default());
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}
pub(super) struct Mode(bool, bool, bool, bool);
impl Mode {
    fn new(alternative: bool, observe: bool) -> Self {
        WORK.with(|w| *w.borrow_mut() = Work::default());
        Self(
            ALTERNATIVE.replace(alternative),
            OBSERVE.replace(observe),
            TIMED.replace(!observe),
            HISTORICAL.replace(false),
        )
    }
    pub(super) fn historical(alternative: bool, observe: bool) -> Self {
        let mode = Self::new(alternative, observe);
        HISTORICAL.set(!alternative);
        mode
    }
}
impl Drop for Mode {
    fn drop(&mut self) {
        ALTERNATIVE.set(self.0);
        OBSERVE.set(self.1);
        TIMED.set(self.2);
        HISTORICAL.set(self.3);
        drop(HOOK.with(|h| h.borrow_mut().take()));
    }
}
fn record(f: impl FnOnce(&mut Work)) {
    if OBSERVE.get() {
        WORK.with(|w| f(&mut w.borrow_mut()));
    }
}
fn work() -> Work {
    WORK.with(|w| w.borrow().clone())
}
pub(super) fn read_attempt() {
    record(|w| w.read_attempts += 1);
}
pub(super) enum Source {
    Owned(String),
    Shared(Arc<String>),
}
impl Deref for Source {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            Self::Owned(s) => s.as_str(),
            Self::Shared(s) => s.as_str(),
        }
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        record(|w| match self {
            Self::Owned(_) => w.owned_drops += 1,
            Self::Shared(_) => w.arc_drops += 1,
        });
    }
}
pub(super) fn acquire(snapshot: Arc<String>) -> Source {
    if ALTERNATIVE.get() {
        shared(snapshot)
    } else {
        owned(snapshot.as_str().to_string())
    }
}
fn shared(snapshot: Arc<String>) -> Source {
    record(|w| {
        w.acquired_snapshots += 1;
        w.retained_arc_transfers += 1;
    });
    Source::Shared(snapshot)
}
pub(super) fn current(snapshot: Arc<String>) -> Source {
    // Keep the restoration observer type-checked without inventing copy work.
    let _restoration_observer: fn(&str) = repair::owned_copy;
    if HISTORICAL.get() {
        acquire(snapshot)
    } else {
        shared(snapshot)
    }
}
pub(super) fn owned(source: String) -> Source {
    if !TIMED.get() {
        super::discovery_work::source_copy(&source);
    }
    record(|w| {
        w.acquired_snapshots += 1;
        w.owned_copies += 1;
        w.copied_bytes += source.len();
    });
    Source::Owned(source)
}
pub(super) fn before_helper(state: &McpState, path: &str, source: &str) {
    let hook = HOOK.with(|h| h.borrow_mut().take());
    if let Some(hook) = hook {
        hook(state, path, source);
    }
    record(|w| {
        w.helper_calls += 1;
        w.helper_input_bytes += source.len();
    });
}
fn state(root: &std::path::Path, persistent: bool) -> McpState {
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.proxy.port = 0;
    cfg.persistence.enabled = persistent;
    cfg.persistence.auto_save = false;
    cfg.persistence.db_path = root.join("state.db").to_string_lossy().into_owned();
    cfg.additional_roots = vec![root.to_string_lossy().into_owned()];
    let state = McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    state
}
fn dispatch(state: &McpState, file: &std::path::Path, fidelity: &str) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(
        &json!(1),
        "diff_code_context",
        &json!({"arguments":{"filePath":file.to_string_lossy(),
        "workspaceRoot":file.parent().unwrap().to_string_lossy(),"fidelity":fidelity}}),
        state,
    );
    crate::protocol::captured_responses().pop().unwrap()
}
fn source(methods: usize, body: usize, padding: usize, changed: bool) -> String {
    let mut s = format!("// {}\nclass Subject {{\n", "共有".repeat(padding));
    for i in 0..methods {
        s.push_str(&format!(
            "m{i}(): string {{ return \"{}-{i}-{}\"; }}\n",
            "共有".repeat(body),
            changed
        ));
    }
    s.push_str("}\n");
    if changed {
        s.push_str("// changed-source\n");
    }
    s
}
fn fidelity(name: &str) -> crate::compression::Fidelity {
    match name {
        "low" => crate::compression::Fidelity::Low,
        "medium" => crate::compression::Fidelity::Medium,
        "high" => crate::compression::Fidelity::High,
        "verbatim" => crate::compression::Fidelity::Verbatim,
        "edit" => crate::compression::Fidelity::Edit,
        _ => unreachable!(),
    }
}
fn identity(state: &McpState, file: &std::path::Path, name: &str) -> Value {
    let path = file.canonicalize().unwrap().to_string_lossy().into_owned();
    let key = format!("{}::{}", path, fidelity(name) as u8);
    let owner = state.semantic_owner_path(&path);
    let alias = state.alias_for_path(&path);
    let cache = state.cache_read();
    let snapshot = format!("{:?}", cache.get_baseline(&key));
    let hash = cache.get_baseline_hash(&key).map(str::to_owned);
    drop(cache);
    let resolved = crate::mcp::tool_helpers::resolve_file_path_checked(
        &file.to_string_lossy(),
        Some(&file.parent().unwrap().to_string_lossy()),
        &state.config.additional_roots,
    )
    .unwrap();
    let source = state
        .cached_source_snapshot(&resolved)
        .map(|s| s.as_str().to_owned());
    let context = state.ir_context_read();
    let tuples = alias.as_ref().and_then(|a| context.get_ir(a).cloned());
    json!({"path":path,"owner":owner,"alias":alias,"tuples":tuples,
        "baseline":snapshot,"baseline_hash":hash,"cached_source":source})
}

#[path = "diff_ownership_cases_af_perf_002.rs"]
mod cases;
#[path = "diff_source_integrated_af_perf_002.rs"]
mod integrated;
#[path = "diff_ownership_repair_af_perf_002.rs"]
pub(super) mod repair;
