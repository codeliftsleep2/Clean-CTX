use super::*;
use crate::ir::source_observations::request_budget::{Cost, Limits};
use crate::ir::typescript_resolution::e1_metrics as metrics;
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant},
};

const POSITIVE: &str = "import type {Routes as R} from '@angular/router'; export class UserResolver {} export const routes:R=[{path:'users',resolve:{user:UserResolver}}];";
fn write(root: &Path, name: &str, text: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}
fn setup(root: &Path, source: &str) {
    write(root, "app.ts", source);
    write(
        root,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","types":["node"],"noLib":true},"files":["app.ts"]}"#,
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
        r#"{"name":"@types/node","types":"index.d.ts"}"#,
    );
    write(
        root,
        "node_modules/@types/node/index.d.ts",
        "declare class Broken {",
    );
}
fn request() -> Request {
    Request::new(Limits {
        total: Cost {
            source_bytes: 1 << 20,
            support_bytes: 1 << 20,
            retained_bytes: 8 << 20,
            units: 256,
            edges: 256,
            validation_bytes: 1 << 20,
            validation_checks: 10000,
            response_bytes: 4096,
        },
        finalization: Cost {
            validation_bytes: 1 << 20,
            validation_checks: 10000,
            response_bytes: 4096,
            ..Cost::default()
        },
        deadline: Duration::from_secs(10),
        finalization_time: Duration::from_secs(1),
        active_tasks: 1,
    })
    .unwrap()
}
fn envelope() -> Envelopes {
    Envelopes {
        parse_retained_bytes: 128 << 10,
        result_retained_bytes: 128 << 10,
    }
}
fn source(root: &Path) -> Admitted {
    crate::ir::source_observations::Candidate::capture(&[root.to_path_buf()], &root.join("app.ts"))
        .unwrap()
        .admit()
        .unwrap()
}
fn acquire(root: &Path, obs: &Admitted, req: &Request) -> (Option<Evidence>, Option<AngularRole>) {
    (
        direct_identity::capture(req, obs, 0, &[root.to_path_buf()], envelope()).ok(),
        AngularRole::acquire(req, obs, 0, &[root.to_path_buf()], envelope()).ok(),
    )
}
fn assert_pure(prepared: &Prepared<'_>) -> Classification {
    metrics::reset();
    let before = metrics::snapshot();
    let witnesses = crate::work_observations_af_perf_002::snapshot().witnesses;
    let result = prepared.classify(false);
    assert_eq!(before, metrics::snapshot());
    assert_eq!(
        witnesses,
        crate::work_observations_af_perf_002::snapshot().witnesses
    );
    result
}
#[test]
fn e5_positive_aliased_angular_role_and_value_declaration_are_qualified_not_canonical() {
    for text in [
        POSITIVE.to_string(),
        POSITIVE
            .replace("{Routes as R}", "* as api")
            .replace("routes:R=", "routes:api.Routes="),
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), &text);
        let obs = source(root.path());
        let req = request();
        let (runtime, role) = acquire(root.path(), &obs, &req);
        let prepared = Prepared::admit(&req, &obs, 0, runtime, role);
        let class = assert_pure(&prepared);
        assert_eq!(
            class.interpretation,
            Interpretation::StronglySupported,
            "{:?}",
            prepared.validation()
        );
        assert_eq!(
            class.observation,
            ObservationClass::WrittenSourceObservation
        );
        assert!(class.angular_api_role);
        assert_eq!(class.coverage, Coverage::Open);
        assert_eq!(
            class.javascript_implementation,
            Unestablished::NotEstablished
        );
        assert_eq!(class.actual_execution, Unestablished::NotEstablished);
        assert_eq!(class.canonical_registration, Unestablished::NotEstablished);
        assert_eq!(prepared.direct_target().unwrap().name, "UserResolver");
        assert!(prepared.role_evidence().unwrap().evidence().type_space());
        assert!(
            prepared
                .role_evidence()
                .unwrap()
                .evidence()
                .obligations()
                .contains(&Obligation::ContextualContributorsUnchecked)
        );
    }
}
#[test]
fn e5_direct_endpoint_and_lookalike_shape_do_not_establish_angular_role() {
    for text in [
        "export class UserResolver {} export const x=[{path:'a',resolve:{user:UserResolver}}];",
        "export type R=unknown[];export class UserResolver {} export const x:R=[{path:'a',resolve:{user:UserResolver}}];",
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), text);
        let obs = source(root.path());
        let req = request();
        let (runtime, role) = acquire(root.path(), &obs, &req);
        let p = Prepared::admit(&req, &obs, 0, runtime, role);
        let c = assert_pure(&p);
        assert!(p.direct_target().is_some());
        assert!(!c.angular_api_role);
        assert_eq!(c.interpretation, Interpretation::Partial);
    }
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    write(root.path(), "lookalike.ts", "export type Routes=unknown[];");
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"moduleResolution":"bundler","baseUrl":".","paths":{"@angular/router":["lookalike.ts"]},"noLib":true,"types":[]},"files":["app.ts"]}"#,
    );
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    assert_eq!(
        assert_pure(&Prepared::admit(&req, &obs, 0, r, a)).interpretation,
        Interpretation::Partial
    );
}

#[test]
fn e5_type_declaration_and_nested_foreign_package_are_not_value_or_angular_authority() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "export type Routes=unknown[];export class UserResolver{} export const routes:Routes=[{path:'a',resolve:{user:UserResolver}}];",
    );
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    assert!(
        !a.as_ref()
            .unwrap()
            .evidence()
            .module_local_binding_established()
    );
    assert_eq!(
        Prepared::admit(&req, &obs, 0, r, a)
            .classify(false)
            .interpretation,
        Interpretation::Partial
    );
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export {Routes} from 'foreign';",
    );
    write(
        root.path(),
        "node_modules/@angular/router/node_modules/foreign/package.json",
        r#"{"name":"foreign","version":"1.0.0","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@angular/router/node_modules/foreign/index.d.ts",
        "export type Routes=unknown[];",
    );
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    assert_eq!(
        Prepared::admit(&req, &obs, 0, r, a)
            .classify(false)
            .interpretation,
        Interpretation::Partial
    );
}

#[test]
fn e5_different_authorization_scopes_do_not_silently_compose() {
    let root = tempfile::tempdir().unwrap();
    let extra = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    let obs = source(root.path());
    let req = request();
    let runtime =
        direct_identity::capture(&req, &obs, 0, &[root.path().to_path_buf()], envelope()).unwrap();
    let role = AngularRole::acquire(
        &req,
        &obs,
        0,
        &[root.path().to_path_buf(), extra.path().to_path_buf()],
        envelope(),
    )
    .unwrap();
    assert!(
        role.evidence()
            .support_content()
            .any(|(path, _)| path.ends_with("tsconfig.json"))
    );
    let prepared = Prepared::admit(&req, &obs, 0, Some(runtime), Some(role));
    assert_eq!(
        prepared.validation(),
        (
            Validation::Current,
            Validation::Current,
            Validation::Current
        )
    );
    let class = assert_pure(&prepared);
    assert!(!class.composition_compatible);
    assert_eq!(class.interpretation, Interpretation::Partial);
    assert_eq!(class.direct, DirectClass::ModuleLocalValueDeclaration);
}

#[test]
fn e5_claim_records_preserve_provenance_and_precise_cancelled_acquisition() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    let obs = source(root.path());
    let req = request();
    req.cancel_optional();
    let p = Prepared::admit_results(
        &req,
        &obs,
        0,
        direct_identity::capture(&req, &obs, 0, &[root.path().to_path_buf()], envelope()),
        AngularRole::acquire(&req, &obs, 0, &[root.path().to_path_buf()], envelope()),
    );
    metrics::reset();
    let before = metrics::snapshot();
    let records = p.records(false);
    assert_eq!(before, metrics::snapshot());
    assert_eq!(records.source, obs.source());
    assert_eq!(records.source_hash, obs.source_hash());
    assert_eq!(records.written.unwrap().key, "user");
    assert_eq!(
        records.direct_acquisition_error,
        Some(&Obligation::Budget(Stop::Cancelled))
    );
    assert_eq!(
        records.role_acquisition_error,
        Some(&Obligation::Budget(Stop::Cancelled))
    );
    assert_eq!(
        records.classification.observation,
        ObservationClass::WrittenSourceObservation
    );
    assert_eq!(
        records.classification.interpretation,
        Interpretation::Partial
    );
    assert!(records.runtime_value_endpoint.is_none());
}

#[test]
fn e5_internal_endpoint_name_does_not_replace_the_required_public_export_slot() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        &POSITIVE.replace("Routes as R", "NotRoutes as R"),
    );
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "type Routes=unknown[];export {Routes as NotRoutes};",
    );
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    let p = Prepared::admit(&req, &obs, 0, r, a);
    let c = assert_pure(&p);
    assert!(!c.angular_api_role);
    assert_eq!(c.interpretation, Interpretation::Partial);
}
#[test]
fn e5_known_augmentation_and_type_only_runtime_targets_prevent_strong_interpretation() {
    for text in [
        format!(
            "{POSITIVE} declare module '@angular/router' {{ export interface Route {{conflict:string;}} }}"
        ),
        POSITIVE.replace(
            "export class UserResolver {}",
            "import type {UserResolver} from './resolver';",
        ),
    ] {
        let root = tempfile::tempdir().unwrap();
        setup(root.path(), &text);
        write(root.path(), "resolver.ts", "export class UserResolver {}");
        let obs = source(root.path());
        let req = request();
        let (r, a) = acquire(root.path(), &obs, &req);
        let p = Prepared::admit(&req, &obs, 0, r, a);
        let c = assert_pure(&p);
        assert_eq!(c.interpretation, Interpretation::Partial, "{c:?}");
        assert_eq!(c.observation, ObservationClass::WrittenSourceObservation);
    }
}
#[test]
fn e5_stale_role_missing_support_and_optional_cancellation_preserve_observations() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    write(
        root.path(),
        "node_modules/@angular/router/index.d.ts",
        "export type Routes=unknown[];",
    );
    let p = Prepared::admit(&req, &obs, 0, r, a);
    assert_eq!(p.validation().2, Validation::Stale);
    assert_eq!(assert_pure(&p).interpretation, Interpretation::Partial);
    std::fs::remove_file(root.path().join("node_modules/@angular/router/index.d.ts")).unwrap();
    let (r, a) = acquire(root.path(), &obs, &req);
    assert_eq!(
        assert_pure(&Prepared::admit(&req, &obs, 0, r, a)).interpretation,
        Interpretation::Partial
    );
    req.cancel_optional();
    let (r, a) = acquire(root.path(), &obs, &req);
    assert!(r.is_none() && a.is_none());
    let p = Prepared::admit(&req, &obs, 0, r, a);
    assert_eq!(
        assert_pure(&p).observation,
        ObservationClass::WrittenSourceObservation
    );
    write(root.path(), "app.ts", "export const x=[];");
    let p = Prepared::admit(&req, &obs, 0, None, None);
    assert_eq!(assert_pure(&p).observation, ObservationClass::Unknown);
}
#[test]
fn e5_configuration_mutation_and_budget_exhaustion_do_not_promote_incomplete_role() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), POSITIVE);
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    write(root.path(), "tsconfig.json", "{}");
    let p = Prepared::admit(&req, &obs, 0, r, a);
    assert_eq!(assert_pure(&p).interpretation, Interpretation::Partial);
    let small = Request::new(Limits {
        total: Cost {
            retained_bytes: 1 << 20,
            validation_bytes: 4096,
            validation_checks: 1000,
            ..Cost::default()
        },
        finalization: Cost {
            validation_bytes: 4096,
            validation_checks: 1000,
            ..Cost::default()
        },
        deadline: Duration::from_secs(10),
        finalization_time: Duration::ZERO,
        active_tasks: 1,
    })
    .unwrap();
    let (r, a) = acquire(root.path(), &obs, &small);
    let p = Prepared::admit(&small, &obs, 0, r, a);
    assert_eq!(
        assert_pure(&p).observation,
        ObservationClass::WrittenSourceObservation
    );
    assert_eq!(p.classify(false).interpretation, Interpretation::Partial);
}
#[test]
fn e5_coverage_and_absence_are_independent_and_strictly_source_scoped() {
    let root = tempfile::tempdir().unwrap();
    setup(root.path(), "export const x=[];");
    let obs = source(root.path());
    let req = request();
    let p = Prepared::admit(&req, &obs, 0, None, None);
    let open = assert_pure(&p);
    assert_eq!(
        open.search,
        SearchConclusion::NotObservedUnknownOpenCoverage
    );
    assert_eq!(open.interpretation, Interpretation::Unknown);
    let closed = p.classify(true);
    assert_eq!(closed.coverage, Coverage::ClosedSupportedWrittenSource);
    assert_eq!(
        closed.search,
        SearchConclusion::AbsentInClosedSupportedWrittenSource
    );
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        "export const x=[{...other,path:'a',resolve:{user:U}}];",
    );
    let obs = source(root.path());
    let p = Prepared::admit(&req, &obs, 0, None, None);
    assert_eq!(p.classify(true).coverage, Coverage::Open);
}
#[test]
fn e5_unsupported_expression_and_crossed_mapping_evidence_refuse_promotion() {
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        &POSITIVE.replace("user:UserResolver", "user:factory()"),
    );
    let obs = source(root.path());
    let req = request();
    let (r, a) = acquire(root.path(), &obs, &req);
    let p = Prepared::admit(&req, &obs, 0, r, a);
    assert_eq!(assert_pure(&p).interpretation, Interpretation::Unsupported);
    let root = tempfile::tempdir().unwrap();
    setup(
        root.path(),
        &POSITIVE.replace("user:UserResolver", "user:UserResolver,other:UserResolver"),
    );
    let obs = source(root.path());
    let (r, a) = acquire(root.path(), &obs, &req);
    let p = Prepared::admit(&req, &obs, 1, r, a);
    assert_eq!(p.validation().1, Validation::Incompatible);
    assert_eq!(p.classify(false).interpretation, Interpretation::Partial);
}
#[test]
fn e5_real_angular_acquisition_and_pure_classification_measurements() {
    let Ok(output) = std::env::var("OPS_E5_OUTPUT") else {
        return;
    };
    let mut rows = Vec::new();
    for form in ["angular", "lookalike", "missing"] {
        let root = Path::new(&output).join("fixtures").join(form);
        std::fs::create_dir_all(&root).unwrap();
        setup(
            &root,
            if form == "lookalike" {
                "export class UserResolver {} export const routes=[{path:'users',resolve:{user:UserResolver}}];"
            } else {
                POSITIVE
            },
        );
        if form == "missing" {
            std::fs::remove_file(root.join("node_modules/@angular/router/index.d.ts")).unwrap();
        }
        let obs = source(&root);
        let req = request();
        metrics::reset();
        let start = Instant::now();
        let (r, a) = acquire(&root, &obs, &req);
        let acquisition_ms = start.elapsed().as_secs_f64() * 1000.0;
        let acquisition = metrics::finish();
        assert_eq!(metrics::snapshot().acquisition_results, 0);
        assert_eq!(metrics::snapshot().automatic_type_calls, 0);
        assert_eq!(metrics::snapshot().library_calls, 0);
        metrics::reset();
        let start = Instant::now();
        let p = Prepared::admit(&req, &obs, 0, r, a);
        let admission_ms = start.elapsed().as_secs_f64() * 1000.0;
        let admission = metrics::finish();
        metrics::reset();
        let before = metrics::snapshot();
        let witness_before = crate::work_observations_af_perf_002::snapshot().witnesses;
        let start = Instant::now();
        let class = p.classify(false);
        for _ in 0..1000 {
            std::hint::black_box(p.classify(false));
        }
        let classify_1001_ms = start.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(before, metrics::snapshot());
        assert_eq!(
            witness_before,
            crate::work_observations_af_perf_002::snapshot().witnesses
        );
        if form == "angular" {
            assert_eq!(class.interpretation, Interpretation::StronglySupported);
        } else {
            assert_eq!(class.interpretation, Interpretation::Partial);
        }
        rows.push(json!({"form":form,"source":obs.source(),"source_hash":obs.source_hash(),"classification":format!("{class:?}"),
            "acquisition_ms":acquisition_ms,"admission_ms":admission_ms,"classify_1001_ms":classify_1001_ms,
            "acquisition":acquisition,"admission":admission,"classification_io":metrics::finish(),"retained_reservation_peak":req.snapshot().retained_peak,
            "resolution_edges":req.snapshot().used.edges,"annotation_span":p.annotation_span().map(|s|[s.start,s.end]),
            "endpoint":p.direct_target().map(|d|json!({"source":d.source,"hash":d.source_hash,"name":d.name,"kind":d.kind,"span":[d.span.start,d.span.end]}))}));
    }
    std::fs::write(
        Path::new(&output).join("measurements.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
