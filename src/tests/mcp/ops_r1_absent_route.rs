//! Proposed observational witnesses: no unapproved work/time budget asserted.
use crate::mcp::McpState;
use serde_json::json;
fn fixture() -> (
    tempfile::TempDir,
    std::path::PathBuf,
    usize,
    std::path::PathBuf,
) {
    let count: usize = std::env::var("OPS_R1_UNITS")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .unwrap();
    assert!([1, 4, 16, 64].contains(&count));
    if count > 1 {
        assert_eq!(std::env::var("OPS_R1_CAPTURE").ok().as_deref(), Some("1"));
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    crate::tests::angular_authority_fixture::install(&root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","target":"ES2022","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    let directory_case = std::env::var("OPS_R1_LAYOUT").unwrap_or_default();
    if directory_case == "directories" || directory_case == "directories-empty-types" {
        let mut options = json!({"module":"esnext", "moduleResolution":"bundler",
            "target":"ES2022", "moduleDetection":"legacy", "noLib":true});
        if directory_case == "directories-empty-types" {
            options["types"] = json!([]);
        }
        std::fs::write(
            root.join("tsconfig.json"),
            serde_json::to_vec(&json!({
            "compilerOptions":options,"include":["**/*.ts"]}))
            .unwrap(),
        )
        .unwrap();
    }
    for i in 0..count {
        let directory = if directory_case.starts_with("directories") {
            let path = root.join(format!("feature{i}"));
            std::fs::create_dir(&path).unwrap();
            path
        } else {
            root.clone()
        };
        let peer = if i + 1 == count {
            "class Peer0 {}\n"
        } else {
            ""
        };
        std::fs::write(
            directory.join(format!("owner{i}.ts")),
            format!("{peer}class Probe{i} {{ peer?: Peer0; }}\n"),
        )
        .unwrap();
    }
    let evidence = std::env::var("OPS_R1_EVIDENCE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| temp.path().join("evidence"));
    std::fs::create_dir_all(&evidence).unwrap();
    let mut files: Vec<_> = std::fs::read_dir(&root).unwrap().filter_map(|entry| {
        let path = entry.unwrap().path();
        path.is_file().then(|| json!({"file":path.file_name().unwrap().to_string_lossy(),"sha256":crate::cache::content_hash(&std::fs::read(&path).unwrap())}))
    }).collect();
    if directory_case.starts_with("directories") {
        for i in 0..count {
            let relative = format!("feature{i}/owner{i}.ts");
            files.push(json!({"file":relative,"sha256":crate::cache::content_hash(&std::fs::read(root.join(&relative)).unwrap())}));
        }
    }
    for relative in [
        "node_modules/@angular/core/package.json",
        "node_modules/@angular/core/index.d.ts",
    ] {
        files.push(json!({"file":relative,"sha256":crate::cache::content_hash(&std::fs::read(root.join(relative)).unwrap())}));
    }
    if std::env::var("OPS_R1_CAPTURE").ok().as_deref() == Some("1") {
        for input in &files {
            let relative = input["file"].as_str().unwrap();
            let destination = evidence.join("fixture-inputs").join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(root.join(relative), destination).unwrap();
        }
    }
    std::fs::write(
        evidence.join("fixture.json"),
        serde_json::to_vec_pretty(
            &json!({"root":root,"units":count,"files":files,"resolver_mappings":0}),
        )
        .unwrap(),
    )
    .unwrap();
    (temp, root, count, evidence)
}
#[test]
fn ops_r1_absent_route_observation() {
    let _serial = crate::protocol::handler_response_serial();
    let (_temp, root, count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    let state = McpState::new(config);
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let response = super::angular_resolver_registration_support::raw(
        &state,
        json!({
            "workspaceRoot":root, "type":"resolver_registrations", "direction":"forward", "route_path":"__ops_r1_absent__"
        }),
    );
    std::fs::write(
        evidence.join("query-response.json"),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
    assert!(response.get("error").is_none(), "{response:#}");
    let result = &response["result"];
    assert_ne!(result["isError"], true, "{result:#}");
    let result = &result["structuredContent"];
    assert_eq!(result["count"], 0);
    assert_eq!(result["registrations"], json!([]));
    assert_eq!(result["coverage"]["source_complete"], false);
    assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(
        std::fs::read_dir(&root)
            .unwrap()
            .filter(|entry| entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|extension| extension == "ts"))
            .count(),
        count
    );
}
#[test]
fn ops_r1_fixed_job_project_observation() {
    let (_temp, root, _count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    let state = McpState::new(config);
    let file = root.join("owner0.ts").to_string_lossy().into_owned();
    let source = std::sync::Arc::new(std::fs::read_to_string(&file).unwrap());
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let results = state.semantic_compiler().compile(
        &state.config,
        vec![super::executor::CompilationJob {
            authorized_roots: vec![root],
            typescript_project: None,
            file_path: file.clone(),
            source,
            fidelity: crate::compression::Fidelity::High,
            path_alias: "ops-r1-owner".into(),
            previous_version: 0,
            canonical_owner: file,
            skip_set: None,
        }],
    );
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
    assert_eq!(results.len(), 1);
    assert!(results[0].result.is_ok(), "component compilation failed");
    if crate::ops_r1_observation::active() {
        assert_eq!(
            crate::ops_r1_observation::COUNTS[4].load(std::sync::atomic::Ordering::Relaxed),
            1
        );
    }
}

/// Same public dispatch as the fixture; external inputs remain on their machine.
/// The default case runs in CI. An external run is diagnostic, never fixture RED/GREEN.
#[test]
fn ops_r1_workspace_cost_observation() {
    let _serial = crate::protocol::handler_response_serial();
    let external = std::env::var_os("OPS_R1_WORKSPACE");
    let (_fixture, root, evidence) = if let Some(root) = external.as_ref() {
        assert_eq!(std::env::var("OPS_R1_CAPTURE").as_deref(), Ok("1"));
        let root = std::fs::canonicalize(root).unwrap();
        assert!(root.is_dir());
        let evidence = std::path::PathBuf::from(std::env::var_os("OPS_R1_EVIDENCE_DIR").unwrap());
        assert!(evidence.is_dir());
        assert!(
            !evidence.starts_with(&root),
            "evidence must be outside source roots"
        );
        (None, root, evidence)
    } else {
        let (temp, root, _, evidence) = fixture();
        (Some(temp), root, evidence)
    };
    let mut config = if external.is_some() {
        let path =
            std::env::var_os("OPS_R1_CONFIG").expect("explicit effective JSON config required");
        serde_json::from_slice::<crate::config::CleanCtxConfig>(&std::fs::read(path).unwrap())
            .unwrap()
    } else {
        crate::tests::test_config()
    };
    // Isolate auxiliary services and durable state, not semantic compiler options.
    config.cbm.enabled = false;
    config.persistence.enabled = false;
    config.proxy.auto_start = false;
    let args = json!({"workspaceRoot":root, "type":"resolver_registrations",
        "direction":"forward", "route_path":std::env::var("OPS_R1_ROUTE")
            .unwrap_or_else(|_| "__ops_r1_absent__".into())});
    std::fs::write(
        evidence.join("request.json"),
        serde_json::to_vec_pretty(&args).unwrap(),
    )
    .unwrap();
    let state = McpState::new(config);
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let started = std::time::Instant::now();
    let response = super::angular_resolver_registration_support::raw(&state, args);
    std::fs::write(
        evidence.join("query-response.json"),
        serde_json::to_vec_pretty(&response).unwrap(),
    )
    .unwrap();
    let metrics = json!({"elapsed_seconds":started.elapsed().as_secs_f64(),
        "external_workspace":external.is_some(), "stages":crate::ops_r1_observation::snapshot(),
        "isolation":"fresh McpState; CBM, persistence and proxy disabled; public in-process dispatch, not stdio"});
    std::fs::write(
        evidence.join("completed.json"),
        serde_json::to_vec_pretty(&metrics).unwrap(),
    )
    .unwrap();
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    // External selectors may expose a legitimate registration or refusal. Preserve
    // their response verbatim rather than imposing the synthetic fixture oracle.
    if external.is_none() {
        assert_eq!(response["result"]["structuredContent"]["count"], 0);
    }
}

#[test]
fn ops_r1_empty_automatic_types_preserve_imported_registrations_and_unknown_absence() {
    let _serial = crate::protocol::handler_response_serial();
    use super::angular_resolver_registration_support::{fixture, keys, query, raw};
    for direction in ["forward", "reverse"] {
        let (root, config) = fixture("barrel");
        let config_path = root.path().join("tsconfig.json");
        let mut project: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&config_path).unwrap()).unwrap();
        project["compilerOptions"]
            .as_object_mut()
            .unwrap()
            .remove("types");
        std::fs::write(config_path, serde_json::to_vec(&project).unwrap()).unwrap();
        std::fs::create_dir(root.path().join("feature")).unwrap();
        std::fs::write(
            root.path().join("feature/unrelated.ts"),
            "export class Unrelated {} ",
        )
        .unwrap();
        let state = McpState::new(config);
        let answer = query(&state, root.path(), direction);
        assert_eq!(keys(&answer), ["primary", "secondary"]);
        assert_eq!(answer["count"], 2);
        let absent = raw(
            &state,
            json!({"workspaceRoot":root.path(),
            "type":"resolver_registrations", "direction":"forward", "route_path":"__absent__"}),
        );
        assert!(absent.get("error").is_none(), "{absent:#}");
        assert_ne!(absent["result"]["isError"], true, "{absent:#}");
        let answer = &absent["result"]["structuredContent"];
        assert_eq!(answer["count"], 0);
        assert_eq!(answer["coverage"]["source_complete"], false);
        assert_eq!(answer["coverage"]["omitted_possible"], true);
        assert_eq!(answer["coverage"]["result_semantics"], "lower_bound");
    }
}
