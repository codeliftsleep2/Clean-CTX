use super::super::super::e1_metrics as metrics;
use super::super::request_budget::Limits;
use super::*;
use serde_json::json;
use std::time::{Duration, Instant};

fn write(root: &Path, name: &str, bytes: &str) {
    let p = root.join(name);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, bytes).unwrap();
}
fn setup(import: &str, expression: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "app.ts",
        &format!("{import}\nexport const routes=[{{path:'users',resolve:{{user:{expression}}}}}];"),
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","types":["node"],"noLib":true},"include":["**/*.ts"]}"#,
    );
    write(
        root.path(),
        "node_modules/@types/node/package.json",
        r#"{"name":"@types/node","version":"0.0.0-invalid","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "node_modules/@types/node/index.d.ts",
        "declare class Broken {",
    );
    root
}
fn request() -> Request {
    Request::new(Limits {
        total: Cost {
            source_bytes: 1 << 20,
            support_bytes: 1 << 20,
            retained_bytes: 8 << 20,
            units: 128,
            edges: 128,
            validation_bytes: 1 << 20,
            validation_checks: 10000,
            response_bytes: 1024,
        },
        finalization: Cost {
            validation_bytes: 1 << 20,
            validation_checks: 10000,
            response_bytes: 1024,
            ..Cost::default()
        },
        deadline: Duration::from_secs(10),
        finalization_time: Duration::from_secs(1),
        active_tasks: 1,
    })
    .unwrap()
}
fn envelopes() -> Envelopes {
    Envelopes {
        parse_retained_bytes: 128 << 10,
        result_retained_bytes: 128 << 10,
    }
}
fn observed(root: &Path) -> Admitted {
    super::super::Candidate::capture(&[root.to_path_buf()], &root.join("app.ts"))
        .unwrap()
        .admit()
        .unwrap()
}
fn direct(root: &Path, obs: &Admitted, req: &Request) -> Evidence {
    capture(req, obs, 0, &[root.to_path_buf()], envelopes()).unwrap()
}

#[test]
fn e4_local_runtime_declaration_is_exact_and_independently_validated() {
    let root = setup("export class LocalResolver {}", "LocalResolver");
    let obs = observed(root.path());
    let req = request();
    metrics::reset();
    let evidence = direct(root.path(), &obs, &req);
    assert!(evidence.module_local_binding_established());
    let id = evidence.direct_target().unwrap();
    assert_eq!(id.name, "LocalResolver");
    assert_eq!(id.kind, "class_declaration");
    assert_eq!(id.source, obs.source());
    assert_eq!(id.source_hash, obs.source_hash());
    let source = std::fs::read_to_string(&id.source).unwrap();
    assert_eq!(
        &source[id.span.start..id.span.end],
        "class LocalResolver {}"
    );
    assert!(*evidence.validate(&req).unwrap().value());
    assert_eq!(metrics::snapshot().acquisition_results, 0);
    assert_eq!(metrics::snapshot().dependency_parses, 0);
    assert_eq!(metrics::snapshot().automatic_type_calls, 0);
    assert_eq!(metrics::snapshot().library_calls, 0);
}
#[test]
fn e4_import_alias_named_barrel_and_namespace_reach_exact_export_but_keep_context_open() {
    for (imports, value, barrel) in [
        ("import {Resolver as Alias} from './dep';", "Alias", false),
        ("import {Resolver} from './barrel';", "Resolver", true),
        ("import * as api from './dep';", "api.Resolver", false),
    ] {
        let root = setup(imports, value);
        write(root.path(), "dep.ts", "export class Resolver {}");
        if barrel {
            write(root.path(), "barrel.ts", "export {Resolver} from './dep';");
        }
        let obs = observed(root.path());
        let req = request();
        metrics::reset();
        let evidence = direct(root.path(), &obs, &req);
        assert_eq!(
            evidence.direct_target().map(|id| id.name.as_str()),
            Some("Resolver"),
            "{:?}",
            evidence.obligations()
        );
        assert!(evidence.direct_target().unwrap().source.ends_with("dep.ts"));
        assert_eq!(
            evidence.obligations(),
            &[Obligation::ContextualContributorsUnchecked]
        );
        assert!(!evidence.module_local_binding_established());
        assert!(*evidence.validate(&req).unwrap().value());
        assert_eq!(metrics::snapshot().acquisition_results, 0);
        assert_eq!(metrics::snapshot().automatic_type_calls, 0);
    }
}
#[test]
fn e4_refusals_preserve_source_observations_and_do_not_invent_targets() {
    for (imports, value, file, contents, expected) in [
        (
            "import type {Resolver} from './dep';",
            "Resolver",
            "dep.ts",
            "export class Resolver {}",
            Obligation::TypeOnly,
        ),
        (
            "import {Resolver} from './missing';",
            "Resolver",
            "dep.ts",
            "",
            Obligation::Resolution(String::new()),
        ),
        (
            "import {Resolver} from './dep';",
            "Resolver",
            "dep.ts",
            "export class Resolver {",
            Obligation::UnsupportedSyntax,
        ),
        (
            "import {Resolver} from './dep';",
            "Resolver",
            "dep.ts",
            "export * from './one';export * from './two';",
            Obligation::UnclosedStarExports,
        ),
        (
            "export class Resolver {} export class Resolver {}",
            "Resolver",
            "dep.ts",
            "",
            Obligation::ConflictingDeclarations,
        ),
        (
            "type Resolver = unknown;",
            "Resolver",
            "dep.ts",
            "",
            Obligation::TypeOnly,
        ),
    ] {
        let root = setup(imports, value);
        write(root.path(), file, contents);
        let obs = observed(root.path());
        let req = request();
        let evidence = direct(root.path(), &obs, &req);
        assert!(evidence.direct_target().is_none());
        assert!(
            evidence
                .obligations()
                .iter()
                .any(|o| std::mem::discriminant(o) == std::mem::discriminant(&expected)),
            "{:?}",
            evidence.obligations()
        );
        assert!(obs.is_current());
        assert_eq!(obs.observations().len(), 1);
    }
}
#[test]
fn e4_shadowed_scope_mutation_and_cancellation_never_borrow_old_identity() {
    let root = setup("export class Resolver {}", "Resolver");
    write(
        root.path(),
        "app.ts",
        "export class Resolver {} function f(Resolver:unknown){return {path:'a',resolve:{user:Resolver}};}",
    );
    let obs = observed(root.path());
    let req = request();
    let evidence = direct(root.path(), &obs, &req);
    assert_eq!(
        evidence.obligations(),
        &[Obligation::UnsupportedLexicalScope]
    );
    write(
        root.path(),
        "app.ts",
        "export class Resolver {} export const x={path:'a',resolve:{user:Resolver}};",
    );
    let obs = observed(root.path());
    let evidence = direct(root.path(), &obs, &req);
    write(root.path(), "app.ts", "export const x=[];");
    assert!(!*evidence.validate(&req).unwrap().value());
    assert_eq!(
        direct(root.path(), &obs, &req).obligations(),
        &[Obligation::SourceChanged]
    );
    let root = setup("export class Resolver {}", "Resolver");
    let obs = observed(root.path());
    let req = request();
    req.cancel_optional();
    assert!(matches!(
        capture(&req, &obs, 0, &[root.path().to_path_buf()], envelopes()),
        Err(Obligation::Budget(Stop::Cancelled))
    ));
    assert!(obs.is_current());
}
#[test]
fn e4_package_versions_and_export_conditions_keep_physical_identity() {
    let root = setup("import {Resolver} from 'pkg';", "Resolver");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"1.0.0","exports":{".":{"types":"./types.d.ts","default":"./runtime.js"}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/types.d.ts",
        "export declare class Resolver {}",
    );
    write(
        root.path(),
        "node_modules/pkg/runtime.js",
        "export class Wrong {}",
    );
    let obs = observed(root.path());
    let req = request();
    metrics::reset();
    let one = direct(root.path(), &obs, &req);
    assert!(one.direct_target().unwrap().source.ends_with("types.d.ts"));
    assert!(
        one.manifests()
            .iter()
            .any(|m| m.name == "pkg" && m.version.as_deref() == Some("1.0.0"))
    );
    assert_eq!(metrics::snapshot().dependency_parses, 1);
    assert!(*one.validate(&req).unwrap().value());
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"2.0.0","exports":{".":{"types":"./second.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/second.d.ts",
        "export declare class Resolver {}",
    );
    assert!(!*one.validate(&req).unwrap().value());
    let two = direct(root.path(), &obs, &req);
    assert!(two.direct_target().unwrap().source.ends_with("second.d.ts"));
    assert_ne!(
        one.direct_target().unwrap().source,
        two.direct_target().unwrap().source
    );
    assert!(obs.is_current());
}
#[test]
fn e4_two_installed_versions_and_custom_conditions_are_not_name_identity() {
    let root = setup("import {Resolver} from 'pkg';", "Resolver");
    write(
        root.path(),
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"1.0.0","exports":{".":{"browser":"./browser.d.ts","default":"./default.d.ts"}}}"#,
    );
    write(
        root.path(),
        "node_modules/pkg/browser.d.ts",
        "export declare class Resolver {}",
    );
    write(
        root.path(),
        "node_modules/pkg/default.d.ts",
        "export declare class Resolver {}",
    );
    write(
        root.path(),
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","customConditions":["browser"],"types":["node"],"noLib":true},"include":["**/*.ts"]}"#,
    );
    let source = std::fs::read_to_string(root.path().join("app.ts")).unwrap();
    write(root.path(), "nested/app.ts", &source);
    write(
        root.path(),
        "nested/node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"2.0.0","types":"index.d.ts"}"#,
    );
    write(
        root.path(),
        "nested/node_modules/pkg/index.d.ts",
        "export declare class Resolver {}",
    );
    let req = request();
    let one = direct(root.path(), &observed(root.path()), &req);
    assert!(
        one.direct_target()
            .unwrap()
            .source
            .ends_with("browser.d.ts")
    );
    let nested = super::super::Candidate::capture(
        &[root.path().to_path_buf()],
        &root.path().join("nested/app.ts"),
    )
    .unwrap()
    .admit()
    .unwrap();
    let two = direct(root.path(), &nested, &req);
    assert!(
        two.direct_target()
            .unwrap()
            .source
            .ends_with("nested/node_modules/pkg/index.d.ts")
    );
    assert_ne!(
        one.direct_target().unwrap().source,
        two.direct_target().unwrap().source
    );
    assert!(
        two.manifests()
            .iter()
            .any(|m| m.name == "pkg" && m.version.as_deref() == Some("2.0.0"))
    );
    assert!(*one.validate(&req).unwrap().value());
    assert!(*two.validate(&req).unwrap().value());
}

#[test]
fn e4_directory_input_work_uses_the_same_edge_budget() {
    let root = setup("export class Resolver {}", "Resolver");
    let req = Request::new(Limits {
        total: Cost {
            retained_bytes: 1 << 20,
            ..Cost::default()
        },
        finalization: Cost::default(),
        deadline: Duration::from_secs(10),
        finalization_time: Duration::ZERO,
        active_tasks: 1,
    })
    .unwrap();
    let mut host =
        crate::ir::typescript_resolution::Host::new(&[root.path().to_path_buf()]).unwrap();
    host.direct_budget = Some(req.clone());
    assert!(host.entries(root.path()).is_err());
    assert_eq!(req.snapshot().last_stop, Some(Stop::Edges));
    assert!(observed(root.path()).is_current());
}

#[test]
fn e4_previous_budget_stop_does_not_relabel_a_missing_import() {
    let root = setup("import {Resolver} from './missing';", "Resolver");
    let req = request();
    assert!(matches!(
        req.reserve_optional(Cost {
            response_bytes: 2048,
            ..Cost::default()
        }),
        Err(Stop::ResponseBytes)
    ));
    let obs = observed(root.path());
    let evidence = direct(root.path(), &obs, &req);
    assert!(evidence.direct_target().is_none());
    assert!(matches!(
        evidence.obligations(),
        [Obligation::Resolution(_)]
    ));
    assert!(obs.is_current());
}

#[test]
fn e4_unicode_escape_equivalent_names_cannot_hide_a_competitor() {
    let root = setup(
        r"export class Res\u006flver {} export class Resolver {}",
        "Resolver",
    );
    let obs = observed(root.path());
    let req = request();
    let evidence = direct(root.path(), &obs, &req);
    assert!(evidence.direct_target().is_none());
    assert_eq!(evidence.obligations(), &[Obligation::UnsupportedSyntax]);
    assert!(obs.is_current());
}

fn archive_fixture(root: &Path, from: &Path, to: &Path, files: &mut Vec<serde_json::Value>) {
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            archive_fixture(root, &path, to, files);
        } else {
            let relative = path.strip_prefix(root).unwrap();
            let dest = to.join(relative);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            let bytes = std::fs::read(&path).unwrap();
            std::fs::write(dest, &bytes).unwrap();
            files.push(json!({"relative":relative,"original_path":path,"sha256":crate::cache::content_hash(&bytes),"bytes":bytes.len()}));
        }
    }
}

#[test]
fn e4_measured_direct_identity_and_partial_results() {
    let Ok(output) = std::env::var("OPS_E4_OUTPUT") else {
        return;
    };
    let mut rows = Vec::new();
    for form in ["local", "import", "missing", "budget"] {
        let root = if form == "local" {
            setup("export class Resolver {}", "Resolver")
        } else {
            setup("import {Resolver as Alias} from './dep';", "Alias")
        };
        if form != "missing" {
            write(root.path(), "dep.ts", "export class Resolver {}");
        }
        let obs = observed(root.path());
        let req = if form == "budget" {
            let mut l = Limits {
                total: Cost {
                    retained_bytes: 1 << 20,
                    ..Cost::default()
                },
                finalization: Cost::default(),
                deadline: Duration::from_secs(10),
                finalization_time: Duration::ZERO,
                active_tasks: 1,
            };
            l.total.support_bytes = 1;
            Request::new(l).unwrap()
        } else {
            request()
        };
        metrics::reset();
        let start = Instant::now();
        let evidence = direct(root.path(), &obs, &req);
        let capture_ms = start.elapsed().as_secs_f64() * 1000.0;
        let work = metrics::finish();
        let counters = metrics::snapshot();
        assert_eq!(counters.acquisition_results, 0);
        assert_eq!(counters.automatic_type_calls, 0);
        assert_eq!(counters.library_calls, 0);
        let before = metrics::snapshot();
        let local = evidence.module_local_binding_established();
        assert_eq!(before, metrics::snapshot());
        let work_snapshot = req.snapshot();
        metrics::reset();
        let start = Instant::now();
        let valid = if form == "budget" {
            None
        } else {
            Some(*evidence.validate(&req).unwrap().value())
        };
        let validation_ms = start.elapsed().as_secs_f64() * 1000.0;
        let validation_metrics = metrics::finish();
        assert!(obs.is_current());
        let mut fixture_files = Vec::new();
        archive_fixture(
            root.path(),
            root.path(),
            &Path::new(&output).join("fixtures").join(form),
            &mut fixture_files,
        );
        rows.push(json!({"form":form,"source_hash":obs.source_hash(),"observation_count":obs.observations().len(),
            "capture_ms":capture_ms,"validation_ms":validation_ms,"work":work,"validation":validation_metrics,
            "fixture_files":fixture_files,
            "local_binding":local,"target":evidence.direct_target().map(|id|json!({"source":id.source,"hash":id.source_hash,"name":id.name,"kind":id.kind,"span":[id.span.start,id.span.end]})),
            "obligations":format!("{:?}",evidence.obligations()),"edges":evidence.edges().len(),"validated":valid,
            "reserved_support_bytes":work_snapshot.used.support_bytes,"reserved_units":work_snapshot.used.units,
            "reserved_edges":work_snapshot.used.edges,"reserved_retained_peak":work_snapshot.retained_peak,
            "witness_serialized_bytes":evidence.serialized_witness_bytes(),"deadline_overshoot_ms":req.snapshot().deadline_overshoot.as_secs_f64()*1000.0}));
    }
    std::fs::write(
        Path::new(&output).join("measurements.json"),
        serde_json::to_vec_pretty(&rows).unwrap(),
    )
    .unwrap();
}
