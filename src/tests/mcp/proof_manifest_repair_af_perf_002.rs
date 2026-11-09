//! Frozen original public/proof baseline and independent actual-site unused budget.
use super::super::{Work, reset, snapshot};
use super::{Mode, phases, state};
use crate::mcp::tool_handlers::hydration;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[path = "manifest_baseline_lifecycle_af_perf_002.rs"]
mod lifecycle;

#[derive(Default)]
struct Records {
    rows: Vec<Value>,
    unused: [usize; 6],
}
impl Records {
    fn add(&mut self, case: Value, work: &Work, required: bool) {
        self.unused[0] += work.unused_predicates;
        self.unused[1] += work.unused_clones;
        self.unused[2] += work.unused_string_bytes;
        self.unused[3] += work.unused_manifests;
        self.unused[4] += work.unused_entries;
        self.unused[5] += work.unused_capacity;
        let mut combined = work.combined_receipts.clone();
        for v in &mut combined {
            v.as_object_mut().unwrap().remove("capacity");
        }
        let row = json!({"case":case,"combined":combined,
            "required_root_manifests":if required {json!(work.root_receipts)}else{json!(null)},
            "proofs":work.proof_receipts});
        eprintln!(
            "AF002_MANIFEST_WORK {}",
            json!({"row":self.rows.len(),"work":work})
        );
        self.rows.push(row);
    }
}
fn fixture() -> PathBuf {
    let path = std::env::var("AF002_MANIFEST_FIXTURE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::current_dir()
                .unwrap()
                .join("target/af-perf-002/manifest-test-fixtures")
                .join(format!(
                    "{}-{:?}",
                    std::process::id(),
                    std::thread::current().id()
                ))
        });
    std::fs::create_dir_all(&path).unwrap();
    path
}
fn call(state: &crate::mcp::McpState, tool: &str, args: Value) -> Value {
    crate::protocol::captured_responses().clear();
    crate::mcp::tools::dispatch_tools_call(&json!(1), tool, &json!({"arguments":args}), state);
    let response = crate::protocol::captured_responses().pop().unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response
}
fn inbound(state: &crate::mcp::McpState, root: &Path) -> Value {
    call(
        state,
        "workspace_query",
        json!({"type":"reverse_edges","name":"ContextA",
        "domain":"builtin","entity_type":"TypeRef","workspaceRoot":root.to_string_lossy()}),
    )
}
fn assert_inbound(response: &Value) {
    let value = &response["result"]["structuredContent"];
    assert_eq!(
        value["coverage"]["source_complete"], true,
        "Positive inbound proof missing: {value}"
    );
}
fn ordinary_cases(records: &mut Records, base: &Path) {
    for (index, (count, files, padding, overlap, missing)) in [
        (1, 1, 0, false, false),
        (1, 32, 0, false, false),
        (4, 8, 0, false, false),
        (8, 4, 0, false, false),
        (1, 32, 4096, false, false),
        (4, 8, 4096, false, false),
        (4, 8, 0, true, false),
        (8, 4, 0, true, false),
        (4, 8, 4096, true, false),
        (4, 0, 0, false, false),
        (4, 8, 0, false, true),
        (4, 8, 0, true, true),
    ]
    .into_iter()
    .enumerate()
    {
        let base = base.join(format!("population-{index}"));
        let mut roots = Vec::new();
        let mut sources = Vec::new();
        for i in 0..count {
            let root = if overlap {
                base.join("shared")
                    .join((0..i).map(|_| "nested").collect::<PathBuf>())
            } else {
                base.join(format!("r{i}"))
            };
            std::fs::create_dir_all(&root).unwrap();
            for j in 0..files {
                let path = root.join(format!("f{i}_{j}.rs"));
                let text = format!("// {}\nstruct Wanted{i}_{j};\n", "共有".repeat(padding));
                std::fs::write(&path, &text).unwrap();
                sources.push((path, text));
            }
            roots.push(root);
        }
        roots.push(roots[0].clone());
        if missing {
            roots.push(base.join("missing"));
        }
        if missing && overlap {
            std::fs::write(roots[0].join("malformed.rs"), [0xff, 0xfe]).unwrap();
        }
        for (phase, (row, work)) in phases(&state(&roots), &roots, &sources, false)
            .into_iter()
            .enumerate()
        {
            records.add(
                json!({"population":index,"phase":phase,"value":row}),
                &work,
                phase >= 3,
            );
        }
    }
}
fn records() -> Records {
    let base = fixture();
    let mut records = Records::default();
    ordinary_cases(&mut records, &base);
    lifecycle::inbound_and_invalidation(&mut records, &base);
    lifecycle::provider_cases(&mut records, &base);
    lifecycle::durable_case(&mut records, &base);
    let root = base.join("unknown");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("a.rs"), "struct A;\n").unwrap();
    let state = state(std::slice::from_ref(&root));
    let _mode = Mode::new(false, true);
    reset();
    let scan = super::super::super::scan(&state, std::slice::from_ref(&root), "A");
    let work = snapshot();
    assert!(work.root_manifests > 0);
    assert_eq!(work.skipped_calls, 0);
    records.add(json!({"unknown":true,"attempted":scan.attempted,"complete":scan.completed,"candidates":scan.candidates}),&work,true);
    assert_eq!(records.rows.len(), 70);
    records
}
fn compare(records: &Records) {
    if let Ok(path) = std::env::var("AF002_MANIFEST_ORACLE") {
        let expected: Vec<Value> = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(expected.len(), records.rows.len());
        for (i, (a, b)) in records.rows.iter().zip(&expected).enumerate() {
            assert!(
                a == b,
                "Frozen original public/proof/currentness mismatch row {i}, field {}",
                first_difference(a, b, "$".to_owned())
            );
        }
    }
    for row in &records.rows {
        eprintln!("AF002_MANIFEST_PUBLIC {row}");
    }
}
fn first_difference(a: &Value, b: &Value, path: String) -> String {
    match (a, b) {
        (Value::Object(left), Value::Object(right)) => {
            for key in left.keys().chain(right.keys()) {
                if left.get(key) != right.get(key) {
                    return first_difference(&a[key], &b[key], format!("{path}.{key}"));
                }
            }
        }
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            for (i, (x, y)) in left.iter().zip(right).enumerate() {
                if x != y {
                    return first_difference(x, y, format!("{path}[{i}]"));
                }
            }
        }
        _ => {}
    }
    path
}
#[test]
fn af002_manifest_frozen_public_semantics() {
    let _serial = hydration::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _responses = crate::protocol::handler_response_serial();
    let records = records();
    compare(&records);
}
#[test]
fn af002_manifest_unused_budget() {
    let _serial = hydration::TEST_PROJECT_HYDRATION_SERIALIZE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _responses = crate::protocol::handler_response_serial();
    let records = records();
    compare(&records);
    assert_eq!(
        records.unused, [0; 6],
        "Explicit-demand unused per-root manifest budget"
    );
}
