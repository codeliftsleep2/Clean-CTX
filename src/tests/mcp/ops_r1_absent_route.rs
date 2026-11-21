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
    assert!([1, 4, 16].contains(&count));
    if count > 1 {
        assert_eq!(std::env::var("OPS_R1_CAPTURE").ok().as_deref(), Some("1"));
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    crate::tests::angular_authority_fixture::install(&root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","target":"ES2022","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    for i in 0..count {
        let peer = if i + 1 == count {
            "class Peer0 {}\n"
        } else {
            ""
        };
        std::fs::write(
            root.join(format!("owner{i}.ts")),
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
