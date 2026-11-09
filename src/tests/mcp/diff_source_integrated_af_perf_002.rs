//! Current registered handler versus historical copied-source reference, fixed packet.
use super::*;
use sha2::{Digest, Sha256};

fn handler(source: &str) -> &str {
    let start = source
        .find("pub(crate) fn handle_diff_code_context(")
        .unwrap();
    let end = source[start..]
        .find("// ── Handler: delta_code_context")
        .unwrap()
        + start;
    &source[start..end]
}
fn compact(s: &str) -> String {
    s.split_whitespace().collect()
}
fn guarded(actual: &str, original: &str) -> bool {
    let mut expected = compact(original);
    let start = expected.find("Ok(s)=>{").unwrap();
    let end = expected[start..].find("Err(e)=>").unwrap() + start;
    expected.replace_range(start..end,
        "Ok(s)=>{#[cfg(all(test,feature=\"typescript\"))]{ownership_design::current(s)}#[cfg(not(all(test,feature=\"typescript\")))]{s}}");
    compact(actual) == expected
}
#[test]
fn afperf_diff_source_static() {
    let actual = handler(include_str!("../../mcp/tool_handlers/core/delta.rs"));
    let original = include_str!("fixtures/diff_source_original_handler.txt");
    assert!(
        guarded(actual, original),
        "Diff acquired-source ownership static protection"
    );
    assert!(!guarded(
        &actual.replace(
            "ownership_design::current(s)",
            "ownership_design::owned(s.as_str().to_string())"
        ),
        original
    ));
    assert!(!guarded(
        &actual.replace("&source,", "&source.to_string(),"),
        original
    ));
    assert!(!guarded(
        &actual.replace("&mut state.cache_write()", "&mut state.cache_read()"),
        original
    ));
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../mcp/tool_helpers.rs"))
        ),
        "cc2f9368b5a2aebb048c52871acc74597514469b774f8791493071b9a423b9c1",
        "Protected diff helper drift"
    );
    assert_eq!(
        format!(
            "{:x}",
            Sha256::digest(include_bytes!("../../mcp/state/source_cache.rs"))
        ),
        "536fafa26398b1f74b37a471eedc22df2a387ddae1f67d1b86e683ac1c7d9723",
        "Protected source-cache boundary drift"
    );
}
fn run(
    root: &std::path::Path,
    text: &str,
    original: bool,
    changed: bool,
    observe: bool,
) -> (u128, Value) {
    let file = root.join("integrated.ts");
    let prior = if changed {
        format!("{text}// previous\n")
    } else {
        text.to_owned()
    };
    std::fs::write(&file, &prior).unwrap();
    let state = state(root, false);
    let _mode = if original {
        Mode::historical(false, observe)
    } else {
        Mode::new(false, observe)
    };
    let baseline = dispatch(&state, &file, "high");
    assert!(baseline.get("error").is_none());
    std::fs::write(&file, text).unwrap();
    state.invalidate_source_cache(&file.to_string_lossy());
    drop(state.read_source(&file.to_string_lossy()).unwrap());
    WORK.with(|v| *v.borrow_mut() = Work::default());
    let started = std::time::Instant::now();
    let response = dispatch(&state, &file, "high");
    if observe {
        let elapsed = started.elapsed().as_nanos();
        assert!(response.get("error").is_none());
        (
            elapsed,
            json!({"response":response,"identity":identity(&state,&file,"high"),"work":work()}),
        )
    } else {
        assert!(response.get("error").is_none());
        drop(std::hint::black_box(response));
        (started.elapsed().as_nanos(), Value::Null)
    }
}
#[test]
fn af002_diff_source_integrated_cost() {
    let _responses = crate::protocol::handler_response_serial();
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
            let (_, old) = run(root.path(), &text, true, changed, true);
            let (_, new) = run(root.path(), &text, false, changed, true);
            assert_eq!(old["response"], new["response"]);
            assert_eq!(old["identity"], new["identity"]);
            assert_eq!(old["work"]["copied_bytes"], text.len());
            assert_eq!(new["work"]["copied_bytes"], 0);
            for field in [
                "read_attempts",
                "acquired_snapshots",
                "helper_calls",
                "helper_input_bytes",
            ] {
                assert_eq!(old["work"][field], new["work"][field]);
            }
            for _ in 0..3 {
                run(root.path(), &text, true, changed, false);
                run(root.path(), &text, false, changed, false);
            }
            let mut old_samples = Vec::new();
            let mut new_samples = Vec::new();
            for round in 0..24 {
                if round % 2 == 0 {
                    old_samples.push(run(root.path(), &text, true, changed, false).0);
                    new_samples.push(run(root.path(), &text, false, changed, false).0);
                } else {
                    new_samples.push(run(root.path(), &text, false, changed, false).0);
                    old_samples.push(run(root.path(), &text, true, changed, false).0);
                }
            }
            old_samples.sort_unstable();
            new_samples.sort_unstable();
            eprintln!(
                "AF002_DIFF_SOURCE_INTEGRATED_COST {}",
                json!({"methods":methods,"body":body,"padding":padding,
                "changed":changed,"source_bytes":text.len(),"original_median_ns":old_samples[12],
                "current_median_ns":new_samples[12],"original_samples_ns":old_samples,"current_samples_ns":new_samples,
                "original_observation":old,"current_observation":new})
            );
        }
    }
}
