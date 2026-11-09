//! Agent MaxHeadRoom: observations never feed source, cache or publication policy.
use serde_json::{Value, json};
use std::cell::RefCell;

#[derive(Default, Clone, Debug, serde::Serialize)]
struct Work {
    source_copies: usize,
    source_bytes: usize,
    source_capacity: usize,
    previous_copies: usize,
    previous_tuples: usize,
    previous_strings: usize,
    previous_bytes: usize,
    previous_outer_capacity: usize,
    previous_inner_capacity: usize,
    previous_string_capacity: usize,
    cached_returns: usize,
    discarded_previous_copies: usize,
    necessary_cached_tuples: usize,
    second_reads: usize,
    second_read_bytes: usize,
}
thread_local! { static WORK: RefCell<Work> = RefCell::new(Work::default()); }
pub(super) fn source_copy(source: &String) {
    WORK.with(|w| {
        let mut w = w.borrow_mut();
        w.source_copies += 1;
        w.source_bytes += source.len();
        w.source_capacity += source.capacity();
    });
}
pub(super) fn previous_copy(tuples: &Vec<Vec<String>>) {
    WORK.with(|w| {
        let mut w = w.borrow_mut();
        w.previous_copies += 1;
        w.previous_tuples += tuples.len();
        w.previous_outer_capacity += tuples.capacity();
        for tuple in tuples {
            w.previous_inner_capacity += tuple.capacity();
            for s in tuple {
                w.previous_strings += 1;
                w.previous_bytes += s.len();
                w.previous_string_capacity += s.capacity();
            }
        }
    });
}
pub(super) fn second_read(bytes: usize) {
    WORK.with(|w| {
        let mut w = w.borrow_mut();
        w.second_reads += 1;
        w.second_read_bytes += bytes;
    });
}
pub(super) fn cached_return(tuples: &[Vec<String>], previous: bool) {
    WORK.with(|w| {
        let mut w = w.borrow_mut();
        w.cached_returns += 1;
        w.discarded_previous_copies += usize::from(previous);
        w.necessary_cached_tuples += tuples.len();
    });
}
fn reset() {
    WORK.with(|w| *w.borrow_mut() = Work::default());
}
fn snapshot() -> Work {
    WORK.with(|w| w.borrow().clone())
}
fn dispatch(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response
}
fn source(methods: usize, body: usize, comments: usize) -> String {
    let mut s = format!("// {}\nclass Subject {{\n", "共有".repeat(comments));
    for i in 0..methods {
        s.push_str(&format!(
            "m{i}(): string {{ return \"{}-{i}\"; }}\n",
            "共有".repeat(body)
        ));
    }
    s.push_str("}\n");
    s
}
fn identity(state: &crate::mcp::McpState, file: &str) -> Value {
    let alias = state.alias_for_path(file);
    let context = state.ir_context_read();
    json!({"alias":alias,"version":alias.as_ref().and_then(|a|context.file_version(a)),
        "hash":alias.as_ref().and_then(|a|context.get_source_hash(a).cloned()),
        "tuples":alias.as_ref().and_then(|a|context.get_ir(a).cloned())})
}

#[test]
fn af002_rolling_read_public_characterization() {
    let _responses = crate::protocol::handler_response_serial();
    for (methods, body, comments, repeats) in [
        (0, 0, 0, 1),
        (1, 0, 0, 1),
        (8, 0, 0, 1),
        (64, 0, 0, 1),
        (256, 0, 0, 1),
        (8, 64, 0, 1),
        (8, 4096, 0, 1),
        (8, 0, 4096, 1),
        (8, 0, 0, 4),
        (8, 0, 0, 16),
        (64, 64, 0, 4),
        (1, 4096, 4096, 1),
    ] {
        for tool in ["diff_code_context", "delta_code_context"] {
            // Preserve this original-copy discovery as historical characterization.
            let _historical = (tool == "diff_code_context")
                .then(|| super::ownership_design::Mode::historical(false, true));
            let root = tempfile::tempdir().unwrap();
            let file = root.path().join("subject.ts");
            let file = file.to_string_lossy().into_owned();
            let text = source(methods, body, comments);
            std::fs::write(&file, &text).unwrap();
            let mut cfg = crate::tests::test_config();
            cfg.cbm.enabled = false;
            cfg.persistence.enabled = false;
            cfg.proxy.port = 0;
            cfg.additional_roots = vec![root.path().to_string_lossy().into_owned()];
            let state = crate::mcp::McpState::new(cfg);
            *state.graph_bridge_lock() = None;
            let args = json!({"filePath":file,"workspaceRoot":root.path().to_string_lossy(),"fidelity":"high"});
            for phase in ["cold", "warm", "changed", "changed_warm"] {
                if phase == "changed" {
                    std::fs::write(&file, format!("{text}// changed\n")).unwrap();
                }
                let before = identity(&state, &file);
                reset();
                let started = std::time::Instant::now();
                let mut responses = Vec::new();
                let count = if phase == "warm" { repeats } else { 1 };
                for _ in 0..count {
                    responses.push(dispatch(&state, tool, args.clone()));
                }
                let elapsed = started.elapsed().as_nanos();
                let work = snapshot();
                let after = identity(&state, &file);
                let current = std::fs::read(&file).unwrap();
                if tool == "diff_code_context" {
                    assert_eq!(work.source_copies, count);
                    assert_eq!(work.source_bytes, current.len() * count);
                    let body = responses[0]["result"]["content"][0]["text"]
                        .as_str()
                        .unwrap();
                    if phase == "cold" {
                        assert!(body.contains("No baseline snapshot"));
                    }
                    if phase == "warm" || phase == "changed_warm" {
                        assert!(body.contains("No changes since last snapshot"));
                    }
                    assert_eq!(
                        before, after,
                        "Diff must not create/change canonical IR ownership"
                    );
                    let canonical = std::fs::canonicalize(&file)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned();
                    let key = format!(
                        "{}::{}",
                        canonical,
                        crate::compression::Fidelity::High as u8
                    );
                    let hash = state.cache_read().compute_hash(&current);
                    assert_eq!(
                        state.cache_read().get_baseline_hash(&key),
                        Some(hash.as_str())
                    );
                } else {
                    if phase == "cold" {
                        assert_eq!(work.previous_copies, 0);
                    } else {
                        assert_eq!(work.previous_copies, count);
                        assert_eq!(work.second_reads, count);
                        let tuples = before["tuples"].as_array().unwrap();
                        let strings: Vec<_> = tuples
                            .iter()
                            .flat_map(|t| t.as_array().unwrap())
                            .map(|s| s.as_str().unwrap())
                            .collect();
                        assert_eq!(work.previous_tuples, tuples.len() * count);
                        assert_eq!(work.previous_strings, strings.len() * count);
                        assert_eq!(
                            work.previous_bytes,
                            strings.iter().map(|s| s.len()).sum::<usize>() * count
                        );
                    }
                    if phase == "warm" || phase == "changed_warm" {
                        assert_eq!(work.discarded_previous_copies, count);
                        assert_eq!(before, after, "Unchanged delta changed canonical identity");
                        assert!(responses.iter().all(|r| r["result"]["cached"] == true));
                    }
                    assert_eq!(after["hash"], state.cache_read().compute_hash(&current));
                    assert!(after["tuples"].as_array().is_some_and(|v| !v.is_empty()));
                }
                assert!(state.persistence_store_lock().is_none());
                eprintln!(
                    "AF002_ROLLING_READ {}",
                    json!({"tool":tool,"phase":phase,"methods":methods,
                    "body_repetitions":body,"comment_repetitions":comments,"requests":count,
                    "source_bytes":current.len(),"work":work,"elapsed_ns":elapsed,
                    "before":before,"after":after,"responses":responses,"durable_store":null})
                );
            }
        }
    }
}

#[test]
fn af002_rolling_read_errors_do_not_copy() {
    let _responses = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let mut cfg = crate::tests::test_config();
    cfg.cbm.enabled = false;
    cfg.persistence.enabled = false;
    cfg.proxy.port = 0;
    cfg.additional_roots = vec![root.path().to_string_lossy().into_owned()];
    let state = crate::mcp::McpState::new(cfg);
    *state.graph_bridge_lock() = None;
    for tool in ["diff_code_context", "delta_code_context"] {
        reset();
        crate::protocol::captured_responses().clear();
        crate::mcp::tools::dispatch_tools_call(
            &json!(1),
            tool,
            &json!({"arguments":{"filePath":root.path().join("missing.ts").to_string_lossy(),
                "workspaceRoot":root.path().to_string_lossy(),"fidelity":"high"}}),
            &state,
        );
        let response = crate::protocol::captured_responses().pop().unwrap();
        assert!(response.get("error").is_some(), "{response}");
        let work = snapshot();
        assert_eq!(work.source_copies, 0);
        assert_eq!(work.previous_copies, 0);
        assert!(state.persistence_store_lock().is_none());
    }
}
