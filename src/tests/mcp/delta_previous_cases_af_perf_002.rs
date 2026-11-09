use super::*;
#[test]
fn af002_delta_previous_lock_boundary() {
    let text = include_str!("../../mcp/tool_handlers/core/delta.rs");
    let start = text
        .find("pub(crate) fn handle_delta_code_context(")
        .unwrap();
    let text = &text[start..];
    let lock = text.find("let ir_ctx = state.ir_context_lock()").unwrap();
    let early = text.find("delta_design::historical_early()").unwrap();
    let check = text.find("if ir_ctx.is_source_unchanged").unwrap();
    let required = text.find("let cached_ir = ir_ctx.get_ir").unwrap();
    let capture = text.find("let capture_previous = ||").unwrap();
    let release = text
        .find("drop(ir_ctx); // Release lock before expensive compile")
        .unwrap();
    assert!(
        lock < early
            && early < check
            && check < required
            && required < capture
            && capture < release
    );
    assert!(text[capture..release].contains("ir_ctx.get_ir(&path_alias).cloned()"));
    assert!(text[capture..release].contains("ir_ctx.get_source_hash(&path_alias).cloned()"));
    assert!(text[capture..release].contains("let previous_authority = capture_previous();"));
}
fn reset_db(root: &std::path::Path) {
    for name in ["state.db", "state.db-wal", "state.db-shm"] {
        let path = root.join(name);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }
}
fn sequence(
    root: &std::path::Path,
    pop: (usize, usize, usize, usize),
    fidelity: &str,
    delay: bool,
    persist: bool,
) -> Vec<(Value, Work)> {
    let (methods, body, padding, repeats) = pop;
    reset_db(root);
    let file = root.join("subject.ts");
    let initial = source(methods, body, padding, false);
    std::fs::write(&file, &initial).unwrap();
    let mut state = state(root, persist);
    let mut rows = Vec::new();
    for phase in [
        "cold",
        "warm",
        "repeat",
        "comment",
        "comment_warm",
        "structural",
        "structural_repeat",
        "publish",
        "changed_warm",
        "invalidate_warm",
        "restart",
    ] {
        let restored = if phase == "restart" {
            if !persist {
                continue;
            }
            drop(state);
            state = super::state(root, persist);
            let response = call(&state, &file, "restore_context", fidelity);
            assert!(response.get("error").is_none(), "{response}");
            response
        } else {
            Value::Null
        };
        if phase == "comment" {
            std::fs::write(&file, format!("{initial}// comment edit\n")).unwrap();
        }
        if phase == "structural" {
            std::fs::write(&file, source(methods, body, padding, true)).unwrap();
        }
        if phase == "invalidate_warm" {
            state.invalidate_source_cache(&file.to_string_lossy());
        }
        let count = if phase == "repeat" { repeats } else { 1 };
        let before = identity(&state, &file);
        let _mode = Mode::historical(delay, true);
        let mut responses = Vec::new();
        for _ in 0..count {
            responses.push(call(
                &state,
                &file,
                if phase == "publish" {
                    "compress_code_context"
                } else {
                    "delta_code_context"
                },
                fidelity,
            ));
        }
        for response in &responses {
            if persist && matches!(phase, "structural" | "structural_repeat") {
                assert_eq!(response["error"]["code"], -32603, "{phase}: {response}");
                assert_eq!(
                    response["error"]["message"],
                    "Invalid session canonical IR: Durable baseline does not match session canonical state"
                );
            } else {
                assert!(response.get("error").is_none(), "{phase}: {response}");
            }
        }
        let observed = work();
        let after = identity(&state, &file);
        if phase != "publish" {
            assert_eq!(observed.initial_reads, count);
            if phase == "cold" {
                assert_eq!(observed.previous_copies, 0);
            } else {
                assert_eq!(observed.second_reads, count);
                let cached = responses.iter().all(|r| r["result"]["cached"] == true);
                assert_eq!(
                    observed.previous_copies,
                    if delay && cached { 0 } else { count }
                );
                if cached {
                    assert_eq!(observed.cached_copies, count);
                    for field in [
                        "canonical",
                        "edges",
                        "fidelity",
                        "semantic_edges",
                        "provenance",
                    ] {
                        assert_eq!(
                            before[field], after[field],
                            "Cached {field} changed in {phase}"
                        );
                    }
                    assert_eq!(after["cached"], std::fs::read_to_string(&file).unwrap());
                } else {
                    assert_eq!(observed.changed_captures, count);
                    assert!(!observed.capture.is_null());
                }
            }
        }
        let durable = if persist {
            let guard = state.persistence_store_lock();
            let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
            format!(
                "{:?}",
                sqlite
                    .load_durable_context(&file.to_string_lossy(), None)
                    .unwrap()
            )
        } else {
            assert!(state.persistence_store_lock().is_none());
            "disabled".to_owned()
        };
        rows.push((json!({"phase":phase,"restored":restored,"responses":responses,"before":before,"after":after,"durable":durable}),observed));
    }
    rows
}
fn compare(before: &[(Value, Work)], after: &[(Value, Work)]) {
    assert_eq!(before.len(), after.len());
    for ((a, old), (b, new)) in before.iter().zip(after) {
        assert_eq!(a, b, "Complete delta public/canonical/provenance mismatch");
        assert_eq!(old.initial_reads, new.initial_reads);
        assert_eq!(old.second_reads, new.second_reads);
        assert_eq!(old.initial_bytes, new.initial_bytes);
        assert_eq!(old.second_bytes, new.second_bytes);
        assert_eq!(old.cached_copies, new.cached_copies);
        assert_eq!(old.cached_tuples, new.cached_tuples);
        assert_eq!(old.cached_bytes, new.cached_bytes);
        assert_eq!(old.changed_captures, new.changed_captures);
        assert_eq!(old.capture, new.capture);
        assert_eq!(old.publication_checks, new.publication_checks);
        assert_eq!(old.publication_current, new.publication_current);
        if old.cached_copies > 0 {
            assert_eq!(new.previous_copies, 0);
            assert_eq!(old.discarded_copies, old.cached_copies);
        } else {
            assert_eq!(old.previous_copies, new.previous_copies);
            assert_eq!(old.bytes, new.bytes);
        }
    }
}
#[test]
fn af002_delta_previous_public() {
    let _serial = crate::protocol::handler_response_serial();
    for pop in [
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
        for fidelity in ["high", "edit"] {
            let root = tempfile::tempdir().unwrap();
            let old = sequence(root.path(), pop, fidelity, false, false);
            let new = sequence(root.path(), pop, fidelity, true, false);
            compare(&old, &new);
            for ((identity, old), (_, new)) in old.into_iter().zip(new) {
                eprintln!(
                    "AF002_DELTA_PREVIOUS_PUBLIC {}",
                    json!({"population":pop,"fidelity":fidelity,"identity":identity,"original":old,"candidate":new})
                );
            }
        }
    }
}
#[test]
fn af002_delta_previous_errors_and_interleavings() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("subject.ts");
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
        let mut values = Vec::new();
        for delay in [false, true] {
            let state = state(root.path(), false);
            let _mode = Mode::historical(delay, true);
            let response = call(&state, &file, "delta_code_context", "high");
            assert!(response.get("error").is_some());
            assert_eq!(work().previous_copies, 0);
            values.push(response);
        }
        assert_eq!(values[0], values[1]);
    }
    std::fs::remove_dir(&file).unwrap();
    for scenario in ["between_reads", "publication"] {
        let mut values = Vec::new();
        for delay in [false, true] {
            let initial = source(8, 64, 0, false);
            std::fs::write(&file, &initial).unwrap();
            let state = state(root.path(), false);
            assert!(
                call(&state, &file, "delta_code_context", "high")
                    .get("error")
                    .is_none()
            );
            if scenario == "publication" {
                std::fs::write(&file, format!("{initial}// compile changed\n")).unwrap();
            }
            let _mode = Mode::historical(delay, true);
            let replacement = source(9, 64, 0, true);
            let nested = std::rc::Rc::new(RefCell::new(Value::Null));
            let captured = nested.clone();
            let hook: Hook = Box::new(move |state, path, _| {
                let target = path.to_owned();
                let text = replacement.clone();
                std::thread::spawn(move || std::fs::write(target, text).unwrap())
                    .join()
                    .unwrap();
                state.invalidate_source_cache(path);
                if scenario == "publication" {
                    let published = call(
                        state,
                        std::path::Path::new(path),
                        "compress_code_context",
                        "high",
                    );
                    assert!(published.get("error").is_none(), "{published}");
                    *captured.borrow_mut() = published;
                }
            });
            if scenario == "between_reads" {
                SOURCE_HOOK.with(|h| *h.borrow_mut() = Some(hook));
            } else {
                PUBLISH_HOOK.with(|h| *h.borrow_mut() = Some(hook));
            }
            let response = call(&state, &file, "delta_code_context", "high");
            assert_eq!(response.get("error").is_some(), scenario == "publication");
            assert_eq!(work().changed_captures, 1);
            assert_eq!(work().previous_copies, 1);
            values.push(json!({"response":response,"identity":identity(&state,&file),"capture":work().capture,"nested":nested.borrow().clone()}));
        }
        assert_eq!(values[0], values[1], "Interleaving boundary changed");
        eprintln!(
            "AF002_DELTA_PREVIOUS_INTERLEAVING {}",
            json!({"scenario":scenario,"identity":values[0]})
        );
    }
}
#[test]
fn af002_delta_previous_persistent() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let old = sequence(root.path(), (8, 64, 0, 4), "edit", false, true);
    let new = sequence(root.path(), (8, 64, 0, 4), "edit", true, true);
    compare(&old, &new);
    for ((identity, old), (_, new)) in old.into_iter().zip(new) {
        eprintln!(
            "AF002_DELTA_PREVIOUS_PERSISTENT {}",
            json!({"identity":identity,"original":old,"candidate":new})
        );
    }
}
fn local(root: &std::path::Path, text: &str, delay: bool, changed: bool) -> u128 {
    let file = root.join("timed.ts");
    std::fs::write(&file, text).unwrap();
    let state = state(root, false);
    assert!(
        call(&state, &file, "delta_code_context", "high")
            .get("error")
            .is_none()
    );
    if changed {
        std::fs::write(&file, format!("{text}// comment change\n")).unwrap();
        state.invalidate_source_cache(&file.to_string_lossy());
    }
    drop(state.read_source(&file.to_string_lossy()).unwrap());
    let _mode = Mode::historical(delay, false);
    let started = std::time::Instant::now();
    let response = call(&state, &file, "delta_code_context", "high");
    assert!(response.get("error").is_none());
    drop(std::hint::black_box(response));
    started.elapsed().as_nanos()
}
#[test]
fn af002_delta_previous_cost() {
    let _serial = crate::protocol::handler_response_serial();
    for (methods, body, padding) in [
        (1, 0, 0),
        (8, 0, 0),
        (64, 0, 0),
        (256, 0, 0),
        (8, 64, 0),
        (8, 4096, 0),
        (8, 0, 4096),
        (64, 64, 0),
    ] {
        let root = tempfile::tempdir().unwrap();
        let text = source(methods, body, padding, false);
        for changed in [false, true] {
            for _ in 0..3 {
                local(root.path(), &text, false, changed);
                local(root.path(), &text, true, changed);
            }
            let mut old = Vec::new();
            let mut new = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    old.push(local(root.path(), &text, false, changed));
                    new.push(local(root.path(), &text, true, changed));
                } else {
                    new.push(local(root.path(), &text, true, changed));
                    old.push(local(root.path(), &text, false, changed));
                }
            }
            old.sort_unstable();
            new.sort_unstable();
            eprintln!(
                "AF002_DELTA_PREVIOUS_COST {}",
                json!({"methods":methods,"body":body,"padding":padding,"changed":changed,
                "original_median_ns":old[12],"candidate_median_ns":new[12],"original_samples_ns":old,"candidate_samples_ns":new})
            );
        }
    }
}
