//! Bounded operator growth observations; ordinary correctness defaults stay one cell.
use crate::mcp::McpState;
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::Instant,
};

fn owner(root: &Path, i: usize) -> PathBuf {
    if std::env::var("OPS_R1_LAYOUT").as_deref() == Ok("directories") {
        root.join(format!("cell{i}/owner{i}.ts"))
    } else {
        root.join(format!("owner{i}.ts"))
    }
}
fn fixture() -> (tempfile::TempDir, PathBuf, usize, PathBuf) {
    let count: usize = std::env::var("OPS_R1_UNITS")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .unwrap();
    assert!([1, 4, 16, 64].contains(&count));
    let layout = std::env::var("OPS_R1_LAYOUT").unwrap_or_else(|_| "flat".into());
    let form = std::env::var("OPS_R1_FORM").unwrap_or_else(|_| "scripts".into());
    assert!(["flat", "directories"].contains(&layout.as_str()));
    assert!(["scripts", "modules"].contains(&form.as_str()));
    if count > 1 || layout != "flat" || form != "scripts" {
        assert_eq!(std::env::var("OPS_R1_CAPTURE").ok().as_deref(), Some("1"));
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    crate::tests::angular_authority_fixture::install(&root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","target":"ES2022","moduleDetection":"legacy","noLib":true,"types":[]},"include":["**/*.ts"]}"#).unwrap();
    let mut names = vec![
        "tsconfig.json".to_string(),
        "node_modules/@angular/core/package.json".into(),
        "node_modules/@angular/core/index.d.ts".into(),
    ];
    for i in 0..count {
        let path = owner(&root, i);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let prefix = if form == "modules" {
            if i + 1 == count {
                "export class Peer0 {}\n".to_string()
            } else {
                let route = if layout == "directories" {
                    format!("../cell{}/owner{}", count - 1, count - 1)
                } else {
                    format!("./owner{}", count - 1)
                };
                format!("import type {{Peer0}} from '{route}';\n")
            }
        } else if i + 1 == count {
            "class Peer0 {}\n".into()
        } else {
            String::new()
        };
        let export = if form == "modules" { "export " } else { "" };
        std::fs::write(
            &path,
            format!("{prefix}{export}class Probe{i} {{ peer?: Peer0; }}\n"),
        )
        .unwrap();
        names.push(
            path.strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
        );
    }
    let evidence = std::env::var("OPS_R1_EVIDENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| temp.path().join("evidence"));
    std::fs::create_dir_all(&evidence).unwrap();
    let mut files = Vec::new();
    for name in names {
        let bytes = std::fs::read(root.join(&name)).unwrap();
        if std::env::var("OPS_R1_CAPTURE").ok().as_deref() == Some("1") {
            let destination = evidence.join("fixture-inputs").join(&name);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(destination, &bytes).unwrap();
        }
        files.push(json!({"file":name,"sha256":crate::cache::content_hash(&bytes)}));
    }
    std::fs::write(evidence.join("fixture.json"), serde_json::to_vec_pretty(&json!({"root":root,"units":count,"layout":layout,"form":form,"files":files,"resolver_mappings":0})).unwrap()).unwrap();
    (temp, root, count, evidence)
}
fn retained_shape(state: &McpState, root: &Path, count: usize) -> Value {
    let mut shapes = Vec::new();
    let selected: BTreeSet<_> = [0, count - 1].into_iter().collect();
    for i in selected {
        let path = owner(root, i);
        let canonical = state.semantic_owner_path(&path.to_string_lossy());
        let index = state.workspace_index_read();
        if let Some(witnesses) = index.retained_typescript_witnesses_for_observation(&canonical) {
            let unique: BTreeSet<_> = witnesses
                .iter()
                .map(|w| serde_json::to_string(w).unwrap())
                .collect();
            shapes.push(json!({"owner":i,"witnesses":witnesses.len(),"unique_witnesses":unique.len(),"serialized_witness_bytes":serde_json::to_vec(witnesses).unwrap().len()}));
        }
    }
    json!(shapes)
}
#[test]
fn ops_r1_absent_route_observation() {
    let _serial = crate::protocol::handler_response_serial();
    let (_temp, root, count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    config.persistence.enabled = false;
    let state = McpState::new(config);
    let requests: usize = std::env::var("OPS_R1_REQUESTS")
        .unwrap_or_else(|_| "1".into())
        .parse()
        .unwrap();
    assert!([1, 2, 3].contains(&requests));
    if requests > 1 {
        assert_eq!(std::env::var("OPS_R1_CAPTURE").ok().as_deref(), Some("1"));
    }
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let mut records = Vec::new();
    for request in 0..requests {
        let before = crate::ops_r1_observation::snapshot();
        let started = Instant::now();
        let dispatch = || {
            crate::ir::typescript_resolution::shared_acquisition::observe(|| {
                super::angular_resolver_registration_support::raw(
                    &state,
                    json!({"workspaceRoot":root,"type":"resolver_registrations","direction":"forward","route_path":"__ops_r1_absent__"}),
                )
            })
        };
        let (response, operations) = if request == 0 {
            dispatch()
        } else {
            crate::work_observations_af_perf_002::unobserved(dispatch)
        };
        let query_ms = started.elapsed().as_secs_f64() * 1000.0;
        let after = crate::ops_r1_observation::snapshot();
        assert!(response.get("error").is_none(), "{response:#}");
        assert_ne!(response["result"]["isError"], true);
        let result = &response["result"]["structuredContent"];
        assert_eq!(result["count"], 0);
        assert_eq!(result["registrations"], json!([]));
        assert_eq!(result["coverage"]["source_complete"], false);
        assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
        let work: Vec<_> = operations
            .iter()
            .map(|o| json!({"acquisitions":o.counts(),"qualification":o.work_counts()}))
            .collect();
        // Shape sampling is outside the query timer and does not call current().
        let shapes = if std::env::var("OPS_R1_CAPTURE").ok().as_deref() == Some("1") {
            retained_shape(&state, &root, count)
        } else {
            json!([])
        };
        records.push(json!({"request":request,"query_elapsed_ms":query_ms,"counters_before":before,"counters_after":after,"operations":work,"compiled_reported":result["discovery"]["compiled"],"retained_shape_samples":shapes}));
        std::fs::write(
            evidence.join("performance.json"),
            serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
        std::fs::write(
            evidence.join("query-response.json"),
            serde_json::to_vec_pretty(&response).unwrap(),
        )
        .unwrap();
    }
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
}
#[test]
fn ops_r1_fixed_job_project_observation() {
    let (_temp, root, count, evidence) = fixture();
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    config.persistence.enabled = false;
    let state = McpState::new(config);
    let file = owner(&root, 0).to_string_lossy().into_owned();
    let source = std::sync::Arc::new(std::fs::read_to_string(&file).unwrap());
    let _sample = crate::ops_r1_observation::sample(&evidence);
    let started = Instant::now();
    let (results, operations) =
        crate::ir::typescript_resolution::shared_acquisition::observe(|| {
            state.semantic_compiler().compile(
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
            )
        });
    let elapsed = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(results.len(), 1);
    assert!(results[0].result.is_ok());
    let work: Vec<_> = operations
        .iter()
        .map(|o| json!({"acquisitions":o.counts(),"qualification":o.work_counts()}))
        .collect();
    std::fs::write(evidence.join("performance.json"),serde_json::to_vec_pretty(&json!({"project_units":count,"compile_elapsed_ms":elapsed,"operations":work,"counters":crate::ops_r1_observation::snapshot()})).unwrap()).unwrap();
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
}
