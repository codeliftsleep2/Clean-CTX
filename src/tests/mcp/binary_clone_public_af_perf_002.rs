//! Original public identities and actual-site zero-owned-vector budget.
use super::*;
use crate::binary_clone_equivalence_af_perf_002::assert_binary;

#[path = "binary_reverse_budget_af_perf_002.rs"]
mod reverse_budget;

fn normalize(value: &mut Value, root: &str) {
    match value {
        Value::String(s) => *s = s.replace(root, "<ROOT>"),
        Value::Array(items) => items.iter_mut().for_each(|item| normalize(item, root)),
        Value::Object(items) => items.values_mut().for_each(|item| normalize(item, root)),
        _ => {}
    }
}
fn rows() -> Vec<(Value, Work)> {
    let mut records = Vec::new();
    let mut cases = vec![
        (1, 1, 0, 1, "edit"),
        (1, 8, 0, 1, "edit"),
        (1, 64, 0, 1, "edit"),
        (1, 256, 0, 1, "edit"),
        (4, 16, 0, 1, "edit"),
        (16, 1, 0, 1, "edit"),
        (1, 16, 64, 1, "edit"),
        (1, 16, 4096, 1, "edit"),
        (1, 16, 0, 4, "edit"),
        (1, 16, 0, 16, "edit"),
    ];
    cases.extend([
        (1, 8, 0, 1, "low"),
        (1, 8, 0, 1, "medium"),
        (1, 8, 0, 1, "high"),
    ]);
    for (classes, methods, padding, repeats, fidelity) in cases {
        let root = tempfile::tempdir().unwrap();
        let file = root
            .path()
            .join("subject.ts")
            .to_string_lossy()
            .into_owned();
        let source = source(classes, methods, padding);
        std::fs::write(&file, &source).unwrap();
        let state = state(&root);
        let mut expected_bytes = None;
        for phase in ["cold", "checkpoint", "repeat"] {
            reset();
            let count = if phase == "repeat" { repeats } else { 1 };
            let mut responses = Vec::new();
            for _ in 0..count {
                let response = if phase == "cold" {
                    dispatch(
                        &state,
                        "compress_code_context",
                        json!({"filePath":file,
                        "workspaceRoot":root.path().to_string_lossy(),"fidelity":fidelity}),
                    )
                } else {
                    dispatch(&state, "save_context", json!({"filePath":file}))
                };
                if phase == "repeat" {
                    assert_eq!(response["result"]["_meta"]["already_durable"], true);
                    assert_eq!(response["result"]["_meta"]["saved"], 0);
                }
                responses.push(response);
            }
            let observed = snapshot();
            let (bytes, ir) = baseline(&state, &file);
            assert_eq!(bytes, assert_binary(&ir));
            assert_encoding(&observed, &ir, count);
            if let Some(expected) = &expected_bytes {
                assert_eq!(&bytes, expected);
            } else {
                expected_bytes = Some(bytes.clone());
            }
            let alias = state.alias_for_path(&file).unwrap();
            let tuples = state.ir_context_read().get_ir(&alias).cloned().unwrap();
            let hash = state
                .ir_context_read()
                .get_source_hash(&alias)
                .cloned()
                .unwrap();
            assert_eq!(state.read_source(&file).unwrap().as_str(), source);
            assert_eq!(state.cache_read().compute_hash(source.as_bytes()), hash);
            assert_eq!(std::fs::read_to_string(&file).unwrap(), source);
            // Exact bytes are compared above for the actual path; normalize only
            // temporary workspace spelling in the cross-run identity record.
            let mut stable_ir = ir.clone();
            stable_ir.file_id = "<ROOT>/subject.ts".into();
            let mut identity = json!({"axes":[classes,methods,padding,repeats,fidelity],"phase":phase,
                "responses":responses,"alias":alias,"tuples":tuples,"source_hash":hash,
                "source":source,"cached_text":state.llm_text_cache_lock().get(&alias).cloned(),
                "fidelity":state.context_fidelity(&alias),
                "semantic_snapshot":state.semantic_edges(&alias),
                "normalized_binary":crate::binary_clone_equivalence_af_perf_002::original::encode(&stable_ir)});
            normalize(&mut identity, &root.path().to_string_lossy());
            records.push((identity, observed));
        }
    }
    let root = tempfile::tempdir().unwrap();
    let file = root
        .path()
        .join("verbatim.ts")
        .to_string_lossy()
        .into_owned();
    let source = "export class Raw { run() { return '共有'; } }\r\n";
    std::fs::write(&file, source).unwrap();
    let state = state(&root);
    reset();
    let response = dispatch(
        &state,
        "compress_code_context",
        json!({"filePath":file,
        "workspaceRoot":root.path().to_string_lossy(),"fidelity":"verbatim"}),
    );
    assert_eq!(response["result"]["content"][0]["text"], source);
    assert!(state.alias_for_path(&file).is_none());
    let mut identity = json!({"phase":"verbatim","response":response});
    normalize(&mut identity, &root.path().to_string_lossy());
    let observed = snapshot();
    assert_eq!(observed.binary_calls, 0);
    records.push((identity, observed));
    records
}
fn check_public(records: &[(Value, Work)]) {
    if let Ok(path) = std::env::var("AF002_BINARY_PUBLIC_ORACLE") {
        let frozen: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        let current: Vec<_> = records
            .iter()
            .map(|(identity, _)| identity.clone())
            .collect();
        assert_eq!(current.len(), frozen.len(), "complete public population");
        for (row, (actual, expected)) in current.iter().zip(&frozen).enumerate() {
            assert!(
                actual == expected,
                "complete frozen public/cache/durable identities: row {row}"
            );
        }
    }
    for (identity, work) in records {
        eprintln!(
            "AF002_BINARY_PUBLIC {}",
            json!({"identity":identity,"work":work})
        );
    }
}
#[test]
fn af002_binary_public_identity() {
    let _serial = crate::protocol::handler_response_serial();
    let records = rows();
    check_public(&records);
}
#[test]
fn af002_binary_public_budget() {
    let _serial = crate::protocol::handler_response_serial();
    let records = rows();
    check_public(&records);
    let cloned: usize = records.iter().map(|(_, work)| work.copied_strings).sum();
    assert_eq!(cloned, 0, "Binary encoder owned-vector clone budget");
}
