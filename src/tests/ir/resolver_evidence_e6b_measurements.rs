//! Actual assembled pipeline measurement. Output is operator evidence, not field acceptance.
use super::*;
use sha2::{Digest, Sha256};
use std::time::Instant;
fn setup(root: &Path) {
    write(root, "app.ts", POSITIVE);
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","types":[],"noLib":true},"files":["app.ts"]}"#,
    );
    write(
        root,
        "node_modules/@angular/router/package.json",
        r#"{"name":"@angular/router","version":"20.0.0","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root,
        "node_modules/@angular/router/index.d.ts",
        "export interface Route {path?:string;resolve?:Record<string,unknown>;} export type Routes=Route[];",
    );
    write(
        root,
        "node_modules/@types/node/package.json",
        r#"{"name":"@types/node","version":"0.0.0-synthetic","types":"index.d.ts"}"#,
    );
    write(
        root,
        "node_modules/@types/node/index.d.ts",
        "declare class NodeProbe {}",
    );
}
#[test]
fn e6b_integrated_measurement() {
    let Some(output) = std::env::var_os("OPS_E6_OUTPUT").map(PathBuf::from) else {
        return;
    };
    let mut receipts = Vec::new();
    let mut present_bytes = 0;
    for case in [
        "present",
        "absent",
        "irrelevant-malformed",
        "relevant-malformed",
        "conflicting",
        "source-exhausted",
        "response-withheld",
        "deadline",
        "cancelled",
    ] {
        let root = output.join("fixtures").join(case);
        setup(&root);
        let mut cap = limits();
        match case {
            "irrelevant-malformed" => write(
                &root,
                "node_modules/@types/node/index.d.ts",
                "declare class Broken {",
            ),
            "relevant-malformed" => write(
                &root,
                "node_modules/@angular/router/index.d.ts",
                "export interface Routes {",
            ),
            "conflicting" => write(
                &root,
                "node_modules/@angular/router/index.d.ts",
                "export type Routes=unknown[]; export type Routes=unknown[];",
            ),
            "source-exhausted" => cap.total.source_bytes = 0,
            "response-withheld" => {
                cap.total.response_bytes = present_bytes - 512;
                cap.finalization.response_bytes = cap.total.response_bytes;
            }
            "deadline" => {
                write(
                    &root,
                    "app.ts",
                    &format!(
                        "{POSITIVE}\n{}",
                        (0..4096)
                            .map(|i| format!("export const padding{i}={i};\n"))
                            .collect::<String>()
                    ),
                );
                cap.deadline = Duration::from_secs(1);
                cap.finalization_time = Duration::from_millis(995);
            }
            _ => {}
        }
        let session = Session::new(vec![root.clone()], cap).unwrap();
        let cancelled_at = std::sync::Arc::new(std::sync::Mutex::new(None));
        if case == "cancelled" {
            let request = session.ledger();
            let signal = cancelled_at.clone();
            bounded::test_hooks::set_capture(move |_| {
                *signal.lock().unwrap() = Some(Instant::now());
                request.cancel_optional();
            });
        }
        e1_metrics::reset();
        let start = Instant::now();
        let result = session.execute(query(
            &root,
            if case == "absent" { "missing" } else { "users" },
        ));
        let elapsed = start.elapsed().as_micros();
        let cancellation_to_completion_us = cancelled_at
            .lock()
            .unwrap()
            .map(|t: Instant| t.elapsed().as_micros());
        let metrics = e1_metrics::finish();
        assert_eq!(metrics["io"]["acquisition_results"], 0);
        assert_eq!(metrics["io"]["automatic_type_calls"], 0);
        assert_eq!(metrics["io"]["library_calls"], 0);
        let (dto, measurements, error) = match result {
            Ok(out) => {
                if case == "present" {
                    present_bytes = out.bytes().len() as u64;
                }
                (
                    out.dto().value().clone(),
                    out.measurements().clone(),
                    Value::Null,
                )
            }
            Err(error) => (Value::Null, Value::Null, json!(format!("{error:?}"))),
        };
        let source = std::fs::read(root.join("app.ts")).unwrap();
        let config = std::fs::read(root.join("tsconfig.json")).unwrap();
        receipts.push(json!({"case":case,"root":root,"elapsed_us":elapsed,"cancellation_to_completion_us":cancellation_to_completion_us,"optional_deadline_ms":(cap.deadline-cap.finalization_time).as_millis(),"source_sha256":format!("{:x}",Sha256::digest(source)),"config_sha256":format!("{:x}",Sha256::digest(config)),"io":metrics,"measurements":measurements,"dto":dto,"error":error,"final_ledger":{"active":session.ledger().snapshot().active_tasks,"retained":session.ledger().snapshot().used.retained_bytes,"peak":session.ledger().snapshot().retained_peak,"overshoot_us":session.ledger().snapshot().deadline_overshoot.as_micros()}}));
        assert_eq!(session.ledger().snapshot().active_tasks, 0);
    }
    for workers in [1usize, 2] {
        let root = output.join("fixtures").join(format!("workers-{workers}"));
        setup(&root);
        let mut cap = limits();
        cap.active_tasks = workers;
        let session = Session::new(vec![root.clone()], cap).unwrap();
        let start = Instant::now();
        // Identical two-operation workload and total allowance; only the number
        // of simultaneous expensive workers differs. No scheduler sleeps.
        let operation = || {
            e1_metrics::reset();
            let start = Instant::now();
            let result = session.execute(query(&root, "users"));
            json!({"elapsed_us":start.elapsed().as_micros(),"ok":result.is_ok(),"io":e1_metrics::finish()})
        };
        let results = if workers == 1 {
            vec![operation(), operation()]
        } else {
            std::thread::scope(|scope| {
                let a = scope.spawn(operation);
                let b = scope.spawn(operation);
                vec![a.join().unwrap(), b.join().unwrap()]
            })
        };
        receipts.push(json!({"case":format!("workers-{workers}"),"elapsed_us":start.elapsed().as_micros(),"results":results,"active_peak":session.ledger().snapshot().active_peak,"used_source_bytes":session.ledger().snapshot().used.source_bytes,"retained_peak":session.ledger().snapshot().retained_peak}));
    }
    std::fs::write(
        output.join("measurements.json"),
        serde_json::to_vec_pretty(&receipts).unwrap(),
    )
    .unwrap();
}
