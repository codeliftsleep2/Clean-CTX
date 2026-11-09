use super::*;

fn sequence(
    root: &std::path::Path,
    name: &str,
    alternative: bool,
    methods: usize,
    body: usize,
    padding: usize,
    persistent: bool,
) -> Vec<Value> {
    let file = root.join("subject.ts");
    let initial = source(methods, body, padding, false);
    std::fs::write(&file, &initial).unwrap();
    let mut state = state(root, persistent);
    let mut rows = Vec::new();
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
        let _mode = Mode::historical(alternative, true);
        let count = if phase == "repeat" { 4 } else { 1 };
        let mut responses = Vec::new();
        for _ in 0..count {
            responses.push(dispatch(&state, &file, name));
        }
        assert!(
            responses.iter().all(|r| r.get("error").is_none()),
            "{responses:?}"
        );
        let observed = work();
        assert_eq!(observed.read_attempts, count);
        assert_eq!(observed.helper_calls, count);
        assert_eq!(observed.acquired_snapshots, count);
        assert_eq!(observed.helper_input_bytes, current.len() * count);
        assert_eq!(observed.owned_copies, if alternative { 0 } else { count });
        assert_eq!(
            observed.copied_bytes,
            if alternative {
                0
            } else {
                current.len() * count
            }
        );
        assert_eq!(observed.owned_drops, if alternative { 0 } else { count });
        assert_eq!(observed.arc_drops, if alternative { count } else { 0 });
        let identity = identity(&state, &file, name);
        assert_eq!(
            identity["baseline_hash"],
            state.cache_read().compute_hash(current.as_bytes())
        );
        let durable = if persistent {
            use crate::mcp::context_store::ContextStore;
            let guard = state.persistence_store_lock();
            let sqlite = guard.as_ref().unwrap().sqlite().unwrap();
            let metadata = sqlite.load_latest(&file.to_string_lossy()).unwrap();
            assert!(
                metadata.is_none(),
                "Diff unexpectedly created a durable checkpoint"
            );
            json!({"enabled":true,"latest":null})
        } else {
            json!({"enabled":false})
        };
        rows.push(
            json!({"phase":phase,"responses":responses,"identity":identity,"durable":durable}),
        );
        eprintln!(
            "AF002_DIFF_OWNERSHIP_WORK {}",
            json!({"phase":phase,"fidelity":name,
            "alternative":alternative,"methods":methods,"body":body,"padding":padding,
            "persistent":persistent,"work":observed})
        );
    }
    rows
}

#[test]
fn af002_diff_ownership_public_equivalence() {
    let _responses = crate::protocol::handler_response_serial();
    for (methods, body, padding) in [
        (0, 0, 0),
        (1, 0, 0),
        (8, 64, 0),
        (64, 0, 0),
        (8, 4096, 0),
        (8, 0, 4096),
    ] {
        for name in ["low", "medium", "high", "verbatim", "edit"] {
            let root = tempfile::tempdir().unwrap();
            let original = sequence(root.path(), name, false, methods, body, padding, false);
            let alternative = sequence(root.path(), name, true, methods, body, padding, false);
            assert_eq!(
                original, alternative,
                "Complete original/alternative public/cache mismatch"
            );
            for row in original {
                eprintln!(
                    "AF002_DIFF_OWNERSHIP_PUBLIC {}",
                    json!({"fidelity":name,
                "methods":methods,"body":body,"padding":padding,"identity":row})
                );
            }
        }
    }
}

#[test]
fn af002_diff_ownership_errors_and_snapshot_lifetime() {
    let _responses = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("subject.ts");
    for fault in ["missing", "invalid_utf8", "directory"] {
        if file.is_file() {
            std::fs::remove_file(&file).unwrap();
        }
        if file.is_dir() {
            std::fs::remove_dir(&file).unwrap();
        }
        match fault {
            "invalid_utf8" => std::fs::write(&file, [0xff, 0xfe]).unwrap(),
            "directory" => std::fs::create_dir(&file).unwrap(),
            _ => {}
        }
        let mut results = Vec::new();
        for alternative in [false, true] {
            let state = state(root.path(), false);
            let _mode = Mode::historical(alternative, true);
            let response = dispatch(&state, &file, "high");
            assert!(response.get("error").is_some());
            let observed = work();
            assert_eq!(observed.acquired_snapshots, 0);
            assert_eq!(observed.helper_calls, 0);
            results.push(response);
        }
        assert_eq!(results[0], results[1], "Error boundary differs for {fault}");
    }
    std::fs::remove_dir(&file).unwrap();
    for name in ["low", "medium", "high", "verbatim", "edit"] {
        let old = source(8, 64, 0, false);
        let new = format!("{}// externally replaced\n", source(8, 64, 0, true));
        let mut results = Vec::new();
        for alternative in [false, true] {
            std::fs::write(&file, &old).unwrap();
            let state = state(root.path(), false);
            let _mode = Mode::historical(alternative, true);
            let expected_old = old.clone();
            let replacement = new.clone();
            HOOK.with(|h| {
                *h.borrow_mut() = Some(Box::new(move |state, path, borrowed| {
                    let path = path.to_owned();
                    let edited = path.clone();
                    let text = replacement.clone();
                    std::thread::spawn(move || std::fs::write(edited, text).unwrap())
                        .join()
                        .unwrap();
                    state.invalidate_source_cache(&path);
                    let replaced = state.read_source(&path).unwrap();
                    assert_eq!(replaced.as_str(), replacement);
                    assert_eq!(borrowed, expected_old);
                    drop(replaced);
                }))
            });
            let response = dispatch(&state, &file, name);
            let snapshot = identity(&state, &file, name);
            assert_eq!(
                snapshot["baseline_hash"],
                state.cache_read().compute_hash(old.as_bytes())
            );
            assert_eq!(snapshot["cached_source"], new);
            let followup = dispatch(&state, &file, name);
            let final_identity = identity(&state, &file, name);
            assert_eq!(
                final_identity["baseline_hash"],
                state.cache_read().compute_hash(new.as_bytes())
            );
            results.push(json!({"response":response,"snapshot":snapshot,"followup":followup,"final":final_identity}));
        }
        assert_eq!(results[0], results[1], "Acquired snapshot boundary changed");
        eprintln!(
            "AF002_DIFF_OWNERSHIP_LIFETIME {}",
            json!({"fidelity":name,"identity":results[0]})
        );
    }
}

#[test]
fn af002_diff_ownership_persistence_and_restart() {
    let _responses = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    // Diff is a rolling in-memory baseline, not a durable checkpoint producer.
    let original = sequence(root.path(), "high", false, 8, 64, 0, true);
    let alternative = sequence(root.path(), "high", true, 8, 64, 0, true);
    assert_eq!(original, alternative);
    for row in original {
        eprintln!("AF002_DIFF_OWNERSHIP_PERSISTENT {row}");
    }
}

fn local(root: &std::path::Path, text: &str, alternative: bool, changed: bool) -> u128 {
    let file = root.join("timed.ts");
    std::fs::write(&file, text).unwrap();
    let state = state(root, false);
    let path = file.to_string_lossy().into_owned();
    let _mode = Mode::historical(alternative, false);
    let initial = if changed {
        format!("{text}// prior\n")
    } else {
        text.to_owned()
    };
    super::super::diff_code_context_handler(
        file.clone(),
        &initial,
        &mut state.cache_write(),
        fidelity("high"),
    )
    .unwrap();
    // Same existing source-cache state for both measured ownership paths.
    drop(state.read_source(&path).unwrap());
    let started = std::time::Instant::now();
    let snapshot = state.read_source(&path).unwrap();
    let source = acquire(snapshot);
    let output = super::super::diff_code_context_handler(
        file,
        &source,
        &mut state.cache_write(),
        fidelity("high"),
    )
    .unwrap();
    drop(std::hint::black_box(output));
    drop(source);
    started.elapsed().as_nanos()
}

#[test]
fn af002_diff_ownership_local_cost() {
    for (methods, body, padding) in [
        (0, 0, 0),
        (1, 0, 0),
        (8, 0, 0),
        (64, 0, 0),
        (256, 0, 0),
        (8, 64, 0),
        (8, 4096, 0),
        (64, 64, 0),
        (1, 4096, 4096),
        (8, 0, 4096),
    ] {
        let root = tempfile::tempdir().unwrap();
        let text = source(methods, body, padding, false);
        for changed in [false, true] {
            for _ in 0..3 {
                local(root.path(), &text, false, changed);
                local(root.path(), &text, true, changed);
            }
            let mut original = Vec::new();
            let mut alternative = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    original.push(local(root.path(), &text, false, changed));
                    alternative.push(local(root.path(), &text, true, changed));
                } else {
                    alternative.push(local(root.path(), &text, true, changed));
                    original.push(local(root.path(), &text, false, changed));
                }
            }
            original.sort_unstable();
            alternative.sort_unstable();
            eprintln!(
                "AF002_DIFF_OWNERSHIP_COST {}",
                json!({"methods":methods,"body":body,"padding":padding,
                "changed":changed,"source_bytes":text.len(),"original_median_ns":original[12],
                "alternative_median_ns":alternative[12],"original_samples_ns":original,"alternative_samples_ns":alternative})
            );
        }
    }
}
