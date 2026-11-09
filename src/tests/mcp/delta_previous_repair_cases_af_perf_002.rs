//! Frozen fixture sequence; always actual default production, never delayed test mode.
use super::*;
fn reset_db(root: &std::path::Path) {
    for name in ["state.db", "state.db-wal", "state.db-shm"] {
        let path = root.join(name);
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
    }
}
pub(super) fn sequence(
    root: &std::path::Path,
    pop: (usize, usize, usize, usize),
    fidelity: &str,
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
            if fidelity == "verbatim" {
                let current = std::fs::read_to_string(&file).unwrap();
                let cache = state.cache_read();
                let persisted_hash = cache.compute_hash(initial.as_bytes());
                let current_hash = cache.compute_hash(current.as_bytes());
                assert_eq!(
                    response["error"],
                    json!({
                        "code":-32603,
                        "data":{"component":"source","reason":"source_mismatch"},
                        "message":format!("Durable restore rejected: durable source mismatch: persisted={persisted_hash}, current={current_hash}")
                    })
                );
            } else {
                assert!(response.get("error").is_none(), "{response}");
            }
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
        let _mode = Mode::new(false, true);
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
            if persist
                && ((matches!(phase, "structural" | "structural_repeat")
                    && (fidelity == "edit" || response.get("error").is_some()))
                    || (fidelity == "verbatim"
                        && matches!(phase, "changed_warm" | "invalidate_warm")))
            {
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
        if phase == "publish" && fidelity == "verbatim" {
            // The registered Verbatim compression path returns raw source; it
            // does not publish the canonical baseline needed to repair this chain.
            for field in [
                "canonical",
                "edges",
                "fidelity",
                "semantic_edges",
                "provenance",
            ] {
                assert_eq!(
                    before[field], after[field],
                    "Verbatim publication changed {field}"
                );
            }
        }
        if phase != "publish" {
            assert_eq!(observed.initial_reads, count);
            if phase == "cold" || (phase == "restart" && fidelity == "verbatim") {
                assert_eq!(observed.previous_copies, 0);
                assert_eq!(observed.second_reads, 0);
                assert_eq!(observed.changed_captures, 0);
            } else {
                assert_eq!(observed.second_reads, count);
                let cached = responses.iter().all(|r| r["result"]["cached"] == true);
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
                    assert_eq!(observed.previous_copies, count);
                    assert_eq!(
                        observed.capture,
                        json!([before["canonical"]["tuples"], before["canonical"]["hash"]])
                    );
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
        let required = required();
        assert_eq!(required.len(), observed.cached_copies);
        rows.push((json!({"phase":phase,"restored":restored,"responses":responses,"before":before,"after":after,"durable":durable,"required_cached":required}),observed));
    }
    rows
}
