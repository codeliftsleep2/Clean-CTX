//! Closed complete-query reuse: exact definitions, independent observations and refusal.
use super::*;
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};
use std::sync::{Arc, Barrier};

fn cases() -> Vec<(RegisteredQuery, Language, &'static str, &'static str)> {
    vec![
        #[cfg(feature = "typescript")]
        (
            RegisteredQuery::TypeScriptIR,
            crate::compression::language::safe_typescript_language().unwrap(),
            crate::queries::TS_QUERY,
            include_str!("fixtures/core_ir_typescript_before.scm"),
        ),
        #[cfg(feature = "csharp")]
        (
            RegisteredQuery::CSharpIR,
            crate::compression::language::safe_csharp_language().unwrap(),
            crate::queries::CS_QUERY,
            include_str!("fixtures/core_ir_csharp_before.scm"),
        ),
    ]
}

fn captures(nodes: Vec<CapturedNode>) -> Vec<(String, String, String, usize, usize, usize)> {
    nodes
        .into_iter()
        .map(|n| {
            (
                n.name,
                n.text,
                n.raw_text,
                n.start_byte,
                n.end_byte,
                n.match_index,
            )
        })
        .collect()
}

fn source(base: &str, index: usize) -> String {
    if base == crate::queries::CS_QUERY {
        format!(
            "class C{index} {{ int Helper() {{ return 1; }} int Run() {{ return Helper(); }} }}"
        )
    } else {
        format!(
            "class C{index} {{ helper(): number {{ return 1; }} run(): number {{ return this.helper(); }} }}"
        )
    }
}

#[test]
fn ci_runtime_complete_definitions_match_frozen_composition() {
    for (_, _, base, frozen) in cases() {
        assert_eq!(
            crate::queries::core_ir::complete_query_definition(base),
            frozen
        );
    }
}

#[test]
fn ci_runtime_complete_queries_preserve_captures_and_ir() {
    for (_, language, base, frozen) in cases() {
        let mut sources = vec![source(base, 0), source(base, 1)];
        if base == crate::queries::TS_QUERY {
            sources.extend([
                include_str!("../../test_files/sample_service.ts").to_string(),
                include_str!("../../test_files/angular/ngrx/user.selectors.ts").to_string(),
                include_str!("../../test_files/angular/ngrx/user.reducer.ts").to_string(),
                "import {Action} from '@ngrx/store'; export class Load implements Action { readonly type = '[X] Load'; }".into(),
            ]);
        } else {
            sources.push(include_str!("../../test_files/dotnet/UserController.cs").into());
        }
        for text in sources {
            for fidelity in [
                Fidelity::Low,
                Fidelity::Medium,
                Fidelity::High,
                Fidelity::Edit,
                Fidelity::Verbatim,
            ] {
                let registered = run_core_capture_pipeline_nodes(
                    language.clone(),
                    base,
                    &text,
                    fidelity,
                    |_, raw, _| Some(raw.into()),
                )
                .unwrap();
                let dynamic = run_capture_pipeline_nodes(
                    language.clone(),
                    frozen,
                    &text,
                    fidelity,
                    |_, raw, _| Some(raw.into()),
                )
                .unwrap();
                assert!(!registered.is_empty());
                assert_eq!(captures(registered), captures(dynamic));
                let make = || {
                    let mut context = PassContext::new(text.clone(), "identity".into(), fidelity);
                    context.language = Some(language.clone());
                    context.query_string = base.into();
                    context
                };
                let mut first = make();
                CoreIRPass.run(&mut first).unwrap();
                reset_query_work_counts();
                reset_parse_count();
                let mut second = make();
                CoreIRPass.run(&mut second).unwrap();
                assert_eq!(
                    format!("{:?}", first.instructions),
                    format!("{:?}", second.instructions)
                );
                assert_eq!(query_compilation_count(), 0);
                assert_eq!(parse_count(), 1);
                assert_eq!(query_cursor_count(), 1);
            }
        }
    }
}

#[test]
fn ci_runtime_custom_invalid_and_mismatched_queries_stay_dynamic() {
    for (_, language, base, _) in cases() {
        for custom in ["(identifier) @custom", "("] {
            reset_query_work_counts();
            for _ in 0..2 {
                let actual = run_core_capture_pipeline_nodes(
                    language.clone(),
                    custom,
                    "class C {}",
                    Fidelity::High,
                    |_, raw, _| Some(raw.into()),
                );
                assert_eq!(actual.is_err(), custom == "(");
            }
            assert_eq!(query_compilation_count(), 2);
        }
        // Even a trailing whitespace change is not the exact registered base.
        let custom = format!("{base}\n");
        reset_query_work_counts();
        for _ in 0..2 {
            run_core_capture_pipeline_nodes(
                language.clone(),
                &custom,
                "class C {}",
                Fidelity::High,
                |_, raw, _| Some(raw.into()),
            )
            .unwrap();
        }
        assert_eq!(query_compilation_count(), 2);
        #[cfg(all(feature = "typescript", feature = "csharp"))]
        {
            let other = if base == crate::queries::TS_QUERY {
                crate::queries::CS_QUERY
            } else {
                crate::queries::TS_QUERY
            };
            reset_query_work_counts();
            let actual = run_core_capture_pipeline_nodes(
                language.clone(),
                other,
                "class C {}",
                Fidelity::High,
                |_, raw, _| Some(raw.into()),
            );
            assert_eq!(
                query_compilation_count(),
                1,
                "mismatched grammar must not select any registration"
            );
            let complete = crate::queries::core_ir::complete_query_definition(other);
            let expected = run_capture_pipeline_nodes(
                language.clone(),
                &complete,
                "class C {}",
                Fidelity::High,
                |_, raw, _| Some(raw.into()),
            );
            match (actual, expected) {
                (Ok(a), Ok(b)) => assert_eq!(captures(a), captures(b)),
                (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
                _ => panic!("mismatched grammar changed failure behavior"),
            }
        }
    }
}

#[test]
fn ci_runtime_disabled_query_features_refuse() {
    let registry = crate::layers::LayerRegistry::global();
    for (extension, enabled) in [
        ("ts", cfg!(feature = "typescript")),
        ("cs", cfg!(feature = "csharp")),
        ("rs", cfg!(feature = "rust")),
        ("java", cfg!(feature = "java")),
    ] {
        assert_eq!(
            registry.language_layer_for_extension(extension).is_some(),
            enabled,
            "language dispatch must match the compiled feature: {extension}"
        );
    }
    for (name, enabled) in [
        ("angular", cfg!(feature = "angular")),
        ("dotnet", cfg!(feature = "dotnet")),
    ] {
        assert_eq!(
            registry
                .meta_layers()
                .iter()
                .any(|layer| layer.name() == name),
            enabled,
            "meta-layer dispatch must match the compiled feature: {name}"
        );
    }
    for identity in [RegisteredQuery::TypeScriptIR, RegisteredQuery::CSharpIR] {
        if identity.language().is_none() {
            let result = run_registered_capture_pipeline_nodes(
                identity,
                "class C {}",
                Fidelity::High,
                |_, raw, _| Some(raw.into()),
            );
            assert!(result.is_err());
        }
    }
}

#[test]
fn ci_runtime_core_concurrent_worker() {
    if std::env::var_os("CI_CORE_QUERY_COLD_WORKER").is_none() {
        return;
    }
    for (identity, language, base, frozen) in cases() {
        let barrier = Arc::new(Barrier::new(4));
        let threads: Vec<_> = (0..4)
            .map(|index| {
                let barrier = barrier.clone();
                let language = language.clone();
                std::thread::spawn(move || {
                    reset_query_work_counts();
                    reset_parse_count();
                    let text = source(base, index);
                    barrier.wait();
                    let actual = run_registered_capture_pipeline_nodes(
                        identity,
                        &text,
                        Fidelity::High,
                        |_, raw, _| Some(raw.into()),
                    )
                    .unwrap();
                    let compiled = query_compilation_count();
                    assert_eq!(parse_count(), 1);
                    assert_eq!(query_cursor_count(), 1);
                    let expected = run_capture_pipeline_nodes(
                        language,
                        frozen,
                        &text,
                        Fidelity::High,
                        |_, raw, _| Some(raw.into()),
                    )
                    .unwrap();
                    assert_eq!(captures(actual), captures(expected));
                    compiled
                })
            })
            .collect();
        let builds: usize = threads.into_iter().map(|t| t.join().unwrap()).sum();
        assert_eq!(
            builds, 1,
            "concurrent first access compiles this exact registration once"
        );
    }
}

#[test]
fn ci_runtime_core_concurrent_first_access_is_process_cold() {
    let result=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","compression::capture_pipeline::core_ir_registration_tests::ci_runtime_core_concurrent_worker","--nocapture"])
        .env("CI_CORE_QUERY_COLD_WORKER","1").output().unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("1 passed; 0 failed; 0 ignored;"));
}
