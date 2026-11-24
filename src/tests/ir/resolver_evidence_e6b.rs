use super::*;
use crate::ir::typescript_resolution::e1_metrics;
use serde_json::{Value, json};
use std::{path::Path, time::Duration};
const POSITIVE: &str = "import type {Routes as R} from '@angular/router'; export class UserResolver {} export const routes:R=[{path:'users',resolve:{user:UserResolver}}];";
fn write(root: &Path, path: &str, text: &str) {
    let p = root.join(path);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "app.ts", POSITIVE);
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","types":[],"noLib":true},"files":["app.ts"]}"#,
    );
    write(
        root.path(),
        "node_modules/@angular/router/package.json",
        r#"{"name":"@angular/router","version":"20.0.0","exports":{".":{"types":"./index.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export interface Route {path?:string;resolve?:Record<string,unknown>;} export type Routes=Route[];",
    );
    write(
        root.path(),
        "node_modules/@types/node/index.d.ts",
        "declare class Broken {",
    );
    root
}
fn limits() -> Limits {
    Limits {
        total: Cost {
            source_bytes: 2 << 20,
            support_bytes: 2 << 20,
            retained_bytes: 64 << 20,
            units: 10000,
            edges: 2000,
            validation_bytes: 8 << 20,
            validation_checks: 10000,
            response_bytes: 1 << 20,
        },
        finalization: Cost {
            validation_bytes: 4 << 20,
            validation_checks: 5000,
            response_bytes: 1 << 20,
            ..Cost::default()
        },
        deadline: Duration::from_secs(10),
        finalization_time: Duration::from_secs(2),
        active_tasks: 2,
    }
}
fn query(root: &Path, selector: &str) -> Query {
    Query::forward(
        root.to_path_buf(),
        selector.into(),
        Some(vec![PathBuf::from("app.ts")]),
    )
}
fn run(root: &Path, selector: &str) -> Output {
    Session::new(vec![root.to_path_buf()], limits())
        .unwrap()
        .execute(query(root, selector))
        .unwrap()
}
#[test]
fn e6b_positive_integrated_current_owned_qualified_and_no_hidden_acquisition() {
    let root = fixture();
    e1_metrics::reset();
    let out = run(root.path(), "users");
    let v = out.dto().value();
    assert_eq!(v["observations"].as_array().unwrap().len(), 1);
    assert_eq!(
        v["interpretations"][0]["classification"],
        "strongly_supported"
    );
    assert_eq!(v["canonical_registrations"], json!([]));
    let metrics = e1_metrics::finish();
    assert_eq!(metrics["io"]["acquisition_results"], 0);
    assert_eq!(metrics["io"]["automatic_type_calls"], 0);
    assert_eq!(metrics["io"]["library_calls"], 0);
    assert_eq!(metrics["io"]["dependency_parses"], 1);
    assert!(metrics["io"]["validation_bytes"].as_u64().unwrap() > 0);
    assert_eq!(serde_json::from_slice::<Value>(out.bytes()).unwrap(), *v);
}
#[test]
fn e6b_selected_absence_reverse_empty_literal_and_explicit_no_inventory() {
    let root = fixture();
    let absent = run(root.path(), "missing");
    let v = absent.dto().value();
    assert_eq!(v["observations"], json!([]));
    assert_eq!(
        v["search_conclusion"]["state"],
        "absent_within_closed_predicate"
    );
    assert_eq!(
        v["execution"]["inventory_work"]["directory_enumerations"],
        0
    );
    assert_eq!(v["coverage"]["workspace"]["state"], "open");
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let mut reverse = query(root.path(), "UserResolver");
    reverse.direction = "reverse".into();
    reverse.selector_basis = "written_reference".into();
    assert_eq!(
        session.execute(reverse).unwrap().dto().value()["matching_records"]["returned"],
        1
    );
    write(
        root.path(),
        "app.ts",
        "export class R {} export const routes=[{path:'',resolve:{x:R}}];",
    );
    assert_eq!(
        run(root.path(), "").dto().value()["matching_records"]["returned"],
        1
    );
}
#[test]
fn e6b_dependency_grammar_failure_and_unsupported_preserve_written_evidence() {
    let root = fixture();
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export interface Broken {",
    );
    let out = run(root.path(), "users");
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
    assert_eq!(
        out.dto().value()["interpretations"][0]["classification"],
        "partial"
    );
    write(
        root.path(),
        "app.ts",
        "const routes=[{path:'users',resolve:{x:factory()}}];",
    );
    let out = run(root.path(), "users");
    assert_eq!(
        out.dto().value()["interpretations"][0]["applicability"],
        "unsupported"
    );
    write(
        root.path(),
        "app.ts",
        "const routes=[{path:'users',resolve:{...other}}];",
    );
    assert_eq!(
        run(root.path(), "users").dto().value()["search_conclusion"]["state"],
        "not_observed_unknown"
    );
}
#[test]
fn e6b_serving_mutation_retracts_source_and_degrades_only_consumed_support() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), || {
            write(root.path(), "app.ts", "export const nothing=1;")
        })
        .unwrap();
    assert_eq!(out.dto().value()["observations"], json!([]));
    assert_eq!(
        out.dto().value()["search_conclusion"]["state"],
        "not_observed_unknown"
    );
    write(root.path(), "app.ts", POSITIVE);
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), || {
            write(
                root.path(),
                "tsconfig.json",
                r#"{"compilerOptions":{"types":["node"],"noLib":true}}"#,
            )
        })
        .unwrap();
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
    assert_eq!(
        out.dto().value()["interpretations"][0]["classification"],
        "partial"
    );
}
#[test]
fn e6b_discovery_prunes_dependencies_and_rejects_unsupported_modes_before_io() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let mut q = query(root.path(), "users");
    q.mode = "exhaustive".into();
    assert!(matches!(session.execute(q), Err(Failure::Invalid(_))));
    assert_eq!(session.ledger().work_counts().metadata_operations, 0);
    let mut q = query(root.path(), "users");
    q.source_files = None;
    assert_eq!(
        session.execute(q).unwrap().dto().value()["matching_records"]["returned"],
        1
    );
}
#[test]
fn e6b_dto_refuses_dangling_crossed_and_canonical_values() {
    let root = fixture();
    let out = run(root.path(), "users");
    for mutation in 0..6 {
        let mut v = out.dto().value().clone();
        match mutation {
            0 => v["interpretations"][0]["value_evidence"] = json!("missing"),
            1 => v["direct_evidence"][0]["namespace"] = json!("type"),
            2 => v["observations"][0]["claim_scope"]["source"]["sha256"] = json!("wrong"),
            3 => v["direct_evidence"][0]["claim_scope"]["mapping_span"]["start"] = json!(999),
            4 => v["canonical_registrations"] = json!([{}]),
            _ => v["direct_evidence"][0]["authorized_roots"] = json!(["C:/elsewhere"]),
        }
        assert!(OwnedDto::checked(v).is_err(), "mutation {mutation}");
    }
}
#[test]
fn e6b_bounded_validation_growth_red_green() {
    let root = fixture();
    let path = root.path().join("app.ts");
    let admitted =
        crate::ir::source_observations::Candidate::capture(&[root.path().to_path_buf()], &path)
            .unwrap()
            .admit()
            .unwrap();
    let mut cap = limits();
    cap.total.validation_bytes = 512;
    cap.finalization.validation_bytes = 512;
    let ledger = if std::env::var_os("OPS_E6_RED").is_some() {
        Request::new(cap).unwrap()
    } else {
        Session::new(vec![root.path().to_path_buf()], cap)
            .unwrap()
            .ledger()
    };
    std::fs::write(&path, format!("{POSITIVE}{}", " ".repeat(1 << 20))).unwrap();
    e1_metrics::reset();
    let _ = ledger.validate(&admitted);
    let measured = e1_metrics::snapshot();
    assert!(
        measured.validation_bytes <= 512,
        "old validation read {} bytes under a 512-byte allowance",
        measured.validation_bytes
    );
}
#[test]
fn e6b_growth_after_metadata_is_capped_and_cannot_publish_current() {
    let root = fixture();
    let path = root.path().join("app.ts");
    let replacement = path.clone();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), || {
            e1_metrics::reset();
            bounded::test_hooks::set(move |p| {
                assert_eq!(p, replacement);
                std::fs::write(&replacement, format!("{POSITIVE}{}", " ".repeat(1 << 20))).unwrap();
            });
        })
        .unwrap();
    assert_eq!(out.dto().value()["observations"], json!([]));
    assert!(e1_metrics::snapshot().validation_bytes <= POSITIVE.len() + 1);
}
#[test]
fn e6b_shared_ledger_cancel_and_all_resource_dimensions() {
    let root = fixture();
    for dimension in 0..9 {
        let mut cap = limits();
        match dimension {
            0 => cap.total.source_bytes = 0,
            1 => cap.total.support_bytes = 0,
            2 => cap.total.retained_bytes = 0,
            3 => cap.total.units = 0,
            4 => cap.total.edges = 0,
            5 => {
                cap.total.validation_bytes = 0;
                cap.finalization.validation_bytes = 0;
            }
            6 => {
                cap.total.validation_checks = 0;
                cap.finalization.validation_checks = 0;
            }
            7 => {
                cap.total.response_bytes = 0;
                cap.finalization.response_bytes = 0;
            }
            _ => cap.active_tasks = 0,
        }
        let result = Session::new(vec![root.path().to_path_buf()], cap)
            .and_then(|session| session.execute(query(root.path(), "users")));
        let expected = [
            Stop::SourceBytes,
            Stop::SupportBytes,
            Stop::RetainedBytes,
            Stop::Units,
            Stop::Edges,
            Stop::ValidationBytes,
            Stop::ValidationChecks,
            Stop::ResponseBytes,
            Stop::ActiveTasks,
        ][dimension];
        match &result {
            Err(Failure::Budget(stop)) => assert_eq!(*stop, expected, "dimension {dimension}"),
            Err(Failure::Invalid(_)) if dimension == 8 => {}
            Err(other) => panic!("unexpected dimension {dimension}: {other:?}"),
            Ok(out) => assert!(
                out.dto().value()["execution"]["stops"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|s| s["code"] == dto::stop_code(expected)),
                "dimension {dimension}"
            ),
        }
        if let Ok(result) = result {
            assert_ne!(
                result.dto().value()["interpretations"]
                    .as_array()
                    .and_then(|v| v.first())
                    .map(|x| x["classification"].as_str().unwrap()),
                Some("strongly_supported"),
                "dimension {dimension}"
            );
        }
    }
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), || {
            session.ledger().cancel_optional()
        })
        .unwrap();
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
    assert_eq!(session.ledger().snapshot().active_tasks, 0);
    let original = session.ledger().snapshot().used;
    assert!(session.clone().execute(query(root.path(), "users")).is_ok());
    assert_eq!(
        session.ledger().snapshot().used.source_bytes,
        original.source_bytes
    );
}
#[test]
fn e6b_response_withholding_preserves_closed_predicate_without_false_absence() {
    let root = fixture();
    let full = run(root.path(), "users");
    let mut cap = limits();
    cap.total.response_bytes = (full.bytes().len() - 1024) as u64;
    cap.finalization.response_bytes = cap.total.response_bytes;
    let result = Session::new(vec![root.path().to_path_buf()], cap)
        .unwrap()
        .execute(query(root.path(), "users"));
    match result {
        Ok(out) => {
            let v = out.dto().value();
            assert_eq!(v["matching_records"]["established"], 1);
            assert_eq!(v["matching_records"]["returned"], 0);
            assert_eq!(v["coverage"]["supported_written_source"]["state"], "closed");
            assert_eq!(v["search_conclusion"]["state"], "observed_not_returned");
        }
        other => panic!("unexpected response failure: {}", other.is_ok()),
    }
}
#[test]
fn e6b_invalid_arithmetic_dependency_owners_and_no_public_activation() {
    let root = fixture();
    let mut cap = limits();
    cap.deadline = Duration::MAX;
    assert!(Session::new(vec![root.path().to_path_buf()], cap).is_err());
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let mut q = query(root.path(), "users");
    q.source_files = Some(vec![PathBuf::from(
        "node_modules/@angular/router/index.d.ts",
    )]);
    assert!(matches!(session.execute(q), Err(Failure::Invalid(_))));
    let schema = include_str!("../../mcp/tool_schemas/resolver_registrations.rs");
    assert!(!schema.contains("resolver-evidence-answer"));
    let dispatch = include_str!("../../mcp/tool_handlers/query/request.rs");
    assert!(!dispatch.contains("Session::"));
}
#[test]
fn e6b_completed_after_stop_keeps_observations_without_identity_upgrade() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let request = session.ledger();
    bounded::test_hooks::set_capture(move |_| request.cancel_optional());
    let out = session.execute(query(root.path(), "users")).unwrap();
    let v = out.dto().value();
    assert_eq!(v["matching_records"]["returned"], 1);
    assert_eq!(v["interpretations"][0]["classification"], "partial");
    assert_eq!(v["execution"]["state"], "cancelled");
    assert!(
        !v["execution"]["completed_after_stop"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}
#[test]
fn e6b_one_serving_boundary_reuses_unique_witnesses_and_keeps_scope() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), e1_metrics::reset)
        .unwrap();
    let work = e1_metrics::finish();
    assert_eq!(work["witness_checks"], work["unique_witnesses"]);
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
}
#[test]
fn e6b_concurrent_operations_share_allowance_and_release_all_active_work() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let child = Session::with_ledger(vec![root.path().to_path_buf()], session.ledger()).unwrap();
    let outputs = std::thread::scope(|scope| {
        let a = scope.spawn(|| session.execute(query(root.path(), "users")));
        let b = scope.spawn(|| child.execute(query(root.path(), "users")));
        vec![a.join().unwrap().unwrap(), b.join().unwrap().unwrap()]
    });
    assert!(
        outputs
            .iter()
            .all(|out| out.dto().value()["matching_records"]["returned"] == 1)
    );
    let peak = session.ledger().snapshot();
    assert!(peak.active_peak <= 2);
    assert_eq!(peak.active_tasks, 0);
    assert!(peak.used.source_bytes <= limits().total.source_bytes);
    drop(outputs);
    drop(child);
    let released = session.ledger().snapshot();
    assert!(released.used.retained_bytes < peak.used.retained_bytes);
    let mut cap = limits();
    cap.active_tasks = 1;
    let session = Session::new(vec![root.path().to_path_buf()], cap).unwrap();
    let held = session.ledger().reserve_optional(Cost::default()).unwrap();
    assert!(matches!(
        session.execute(query(root.path(), "users")),
        Err(Failure::Budget(Stop::ActiveTasks))
    ));
    drop(held);
    assert_eq!(session.ledger().snapshot().active_tasks, 0);
}
#[test]
fn e6b_budget_representability_and_protected_finalization_fail_explicitly() {
    let root = fixture();
    let mut cap = limits();
    cap.total.source_bytes = u64::MAX;
    cap.total.support_bytes = u64::MAX;
    assert!(Session::new(vec![root.path().to_path_buf()], cap).is_err());
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let result = session.before_serving(query(root.path(), "users"), || {
        let cap = session.ledger().limits();
        let used = session.ledger().snapshot().used.validation_checks;
        let _ = session
            .ledger()
            .reserve_nested(
                Cost {
                    validation_checks: cap.total.validation_checks - used,
                    ..Cost::default()
                },
                true,
            )
            .unwrap();
    });
    let out = result.unwrap();
    assert_eq!(out.dto().value()["matching_records"]["returned"], 0);
    assert_eq!(
        out.dto().value()["coverage"]["supported_written_source"]["state"],
        "open"
    );
    assert_eq!(out.dto().value()["execution"]["state"], "exhausted");
    assert!(
        out.dto().value()["execution"]["stops"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["code"] == "validation_checks")
    );
}
#[test]
fn e6b_optional_finalization_exhaustion_keeps_prepaid_source_dto() {
    let root = fixture();
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let mut competing = None;
    let out = session
        .before_serving(query(root.path(), "users"), || {
            // Consume only unprotected optional capacity. Written DTO ownership was
            // already reserved before acquisition; denied optional credits must degrade.
            let s = session.ledger().snapshot();
            let cap = session.ledger().limits();
            let protected = cap
                .finalization
                .retained_bytes
                .saturating_sub(s.finalization_used.retained_bytes);
            let free = cap.total.retained_bytes - s.used.retained_bytes - protected;
            competing = Some(
                session
                    .ledger()
                    .reserve_input(Cost {
                        retained_bytes: free,
                        ..Cost::default()
                    })
                    .unwrap(),
            );
        })
        .unwrap();
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
    assert_eq!(
        out.dto().value()["interpretations"][0]["classification"],
        "partial"
    );
    assert!(
        out.dto().value()["execution"]["stops"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["code"] == "retained_bytes")
    );
    drop(competing);
}
#[test]
fn e6b_wire_fields_match_reviewed_schema_and_support_mutation_is_structured() {
    let root = fixture();
    let out = run(root.path(), "users");
    let schema: Value = serde_json::from_str(include_str!(
        "../../../verification/ops-r1/e6b/schemas/response.schema.json"
    ))
    .unwrap();
    let keys = out
        .dto()
        .value()
        .as_object()
        .unwrap()
        .keys()
        .collect::<std::collections::BTreeSet<_>>();
    let specified = schema["properties"]
        .as_object()
        .unwrap()
        .keys()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(keys, specified);
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session
        .before_serving(query(root.path(), "users"), || {
            write(
                root.path(),
                "node_modules/@angular/router/index.d.ts",
                "export type Routes=never;",
            )
        })
        .unwrap();
    assert_eq!(out.dto().value()["matching_records"]["returned"], 1);
    assert!(
        out.dto().value()["currentness"]["components"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["state"] == "stale"
                && s["component"].as_str().unwrap().starts_with("attempt:role"))
    );
}
#[cfg(windows)]
#[test]
fn e6b_junctions_do_not_become_application_owners() {
    let root = fixture();
    write(root.path(), "node_modules/hidden.ts", POSITIVE);
    let _junction = crate::ir::typescript_resolution::windows_host_tests::junction(
        &root.path().join("bridge"),
        &root.path().join("node_modules"),
    );
    let mut q = query(root.path(), "users");
    q.source_files = None;
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let out = session.execute(q).unwrap();
    assert_eq!(
        out.dto().value()["examined_scope"]["enumerated"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let mut q = query(root.path(), "users");
    q.source_files = Some(vec![PathBuf::from("bridge/hidden.ts")]);
    e1_metrics::reset();
    assert!(matches!(session.execute(q), Err(Failure::Invalid(_))));
    assert_eq!(e1_metrics::snapshot().parses, 0);
}

#[path = "resolver_evidence_e6b_measurements.rs"]
mod measurements;

#[path = "resolver_evidence_e6b_admission.rs"]
mod admission;
