//! Agent MaxHeadRoom: actual default delayed capture versus historical original-site reference.
use super::*;
use sha2::{Digest, Sha256};
fn handler(text: &str) -> &str {
    let start = text
        .find("pub(crate) fn handle_delta_code_context(")
        .unwrap();
    let module = text[start..]
        .find("#[path = \"../../../tests/mcp/delta_sequence.rs\"]")
        .unwrap()
        + start;
    let end = text[..module].rfind("#[cfg(test)]").unwrap();
    &text[start..end]
}
fn compact(text: &str) -> String {
    text.split_whitespace().collect()
}
#[test]
fn afperf_delta_previous_static() {
    let actual = handler(include_str!("../../mcp/tool_handlers/core/delta.rs"));
    let expected = include_str!("fixtures/delta_previous_current_handler.txt");
    let original = include_str!("fixtures/delta_previous_original_handler.txt");
    assert_eq!(
        compact(actual),
        compact(expected),
        "Delta capture/guard/read/publication static protection"
    );
    assert_ne!(compact(original), compact(expected));
    for (from, to) in [
        (
            "let previous_authority = capture_previous();",
            "let previous_authority = None;",
        ),
        ("is_source_unchanged", "has_file"),
        (
            "let cached_ir = ir_ctx.get_ir(&path_alias).unwrap().clone();",
            "let cached_ir = Vec::new();",
        ),
        (
            "previous_authority.or_else(capture_previous)",
            "previous_authority",
        ),
        (
            "state.read_source(&resolved_path)",
            "state.read_source(&canonical_path)",
        ),
        (
            "drop(ir_ctx); // Release lock before expensive compile",
            "// lost guard release",
        ),
        (
            "ctx.get_source_hash(&path_alias)",
            "ctx.get_source_hash(&canonical_path)",
        ),
    ] {
        assert!(actual.contains(from), "Missing finite control: {from}");
        assert_ne!(compact(&actual.replace(from, to)), compact(expected));
    }
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../mcp/state/source_cache.rs"))
        ),
        "536fafa26398b1f74b37a471eedc22df2a387ddae1f67d1b86e683ac1c7d9723",
        "Protected src/mcp/state/source_cache.rs"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../mcp/state/publication.rs"))
        ),
        "b9bc08c1b22d0ce4f757d3c9229dc067ae1524462bb429c8b4bc044b7e219cae",
        "Protected src/mcp/state/publication.rs"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../mcp/tool_helpers.rs"))
        ),
        "cc2f9368b5a2aebb048c52871acc74597514469b774f8791493071b9a423b9c1",
        "Protected src/mcp/tool_helpers.rs"
    );
}
fn local(
    root: &std::path::Path,
    text: &str,
    delay: bool,
    changed: bool,
    observe: bool,
) -> (u128, Option<Value>, Work) {
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
    let _mode = if delay {
        Mode::new(false, observe)
    } else {
        Mode::historical(false, observe)
    };
    let started = std::time::Instant::now();
    let response = call(&state, &file, "delta_code_context", "high");
    assert!(response.get("error").is_none());
    let captured = if observe {
        Some(json!({"response":response,"identity":identity(&state,&file)}))
    } else {
        None
    };
    drop(std::hint::black_box(response));
    let elapsed = started.elapsed().as_nanos();
    (elapsed, captured, work())
}
#[test]
fn af002_delta_previous_integrated_cost() {
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
            let (_, old_value, old_work) = local(root.path(), &text, false, changed, true);
            let (_, new_value, new_work) = local(root.path(), &text, true, changed, true);
            assert_eq!(old_value, new_value);
            assert_eq!(old_work.initial_reads, new_work.initial_reads);
            assert_eq!(old_work.second_reads, new_work.second_reads);
            assert_eq!(old_work.second_bytes, new_work.second_bytes);
            assert_eq!(old_work.cached_copies, new_work.cached_copies);
            assert_eq!(old_work.cached_tuples, new_work.cached_tuples);
            assert_eq!(old_work.cached_bytes, new_work.cached_bytes);
            assert_eq!(old_work.capture, new_work.capture);
            assert_eq!(old_work.publication_checks, new_work.publication_checks);
            assert_eq!(old_work.publication_current, new_work.publication_current);
            assert_eq!(old_work.previous_copies, 1);
            assert_eq!(new_work.previous_copies, usize::from(changed));
            assert_eq!(old_work.cached_copies, usize::from(!changed));
            assert_eq!(new_work.changed_captures, usize::from(changed));
            eprintln!(
                "AF002_DELTA_PREVIOUS_INTEGRATED_WORK {}",
                json!({"methods":methods,"body":body,
                "padding":padding,"changed":changed,"original":old_work,"current":new_work})
            );
            for _ in 0..3 {
                local(root.path(), &text, false, changed, false);
                local(root.path(), &text, true, changed, false);
            }
            let mut old = Vec::new();
            let mut new = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    old.push(local(root.path(), &text, false, changed, false).0);
                    new.push(local(root.path(), &text, true, changed, false).0);
                } else {
                    new.push(local(root.path(), &text, true, changed, false).0);
                    old.push(local(root.path(), &text, false, changed, false).0);
                }
            }
            old.sort_unstable();
            new.sort_unstable();
            eprintln!(
                "AF002_DELTA_PREVIOUS_INTEGRATED_COST {}",
                json!({"methods":methods,"body":body,"padding":padding,"changed":changed,
                "original_median_ns":old[12],"candidate_median_ns":new[12],"original_samples_ns":old,"candidate_samples_ns":new})
            );
        }
    }
}
