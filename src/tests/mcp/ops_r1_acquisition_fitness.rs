//! Public-path work witness: this exact assertion is shared by RED and GREEN.
use crate::ir::typescript_resolution::shared_acquisition::observe;
use crate::mcp::McpState;
use serde_json::json;

#[test]
fn ops_r1_shared_acquisition_public_fitness() {
    let _serial = crate::protocol::handler_response_serial();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("workspace");
    std::fs::create_dir(&root).unwrap();
    crate::tests::angular_authority_fixture::install(&root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","target":"ES2022","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    for i in 0..4 {
        let declaration = if i == 3 { "class Peer0 {}\n" } else { "" };
        std::fs::write(
            root.join(format!("owner{i}.ts")),
            format!("{declaration}class Probe{i} {{ peer?: Peer0; }}\n"),
        )
        .unwrap();
    }
    if let Ok(evidence) = std::env::var("OPS_R1_EVIDENCE_DIR") {
        let evidence = std::path::PathBuf::from(evidence);
        let mut names: Vec<_> = (0..4).map(|i| format!("owner{i}.ts")).collect();
        names.extend(
            [
                "tsconfig.json",
                "node_modules/@angular/core/package.json",
                "node_modules/@angular/core/index.d.ts",
            ]
            .into_iter()
            .map(str::to_owned),
        );
        let mut files = Vec::new();
        for name in names {
            let bytes = std::fs::read(root.join(&name)).unwrap();
            let destination = evidence.join("fixture-inputs").join(&name);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::write(destination, &bytes).unwrap();
            files.push(json!({"file":name,"sha256":crate::cache::content_hash(&bytes)}));
        }
        std::fs::write(
            evidence.join("fixture.json"),
            serde_json::to_vec_pretty(&json!({"files":files,"units":4,"resolver_mappings":0}))
                .unwrap(),
        )
        .unwrap();
    }
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    let state = McpState::new(config);
    // Only the explicitly isolated operator capture enables the existing sampler.
    let _sample = std::env::var("OPS_R1_EVIDENCE_DIR")
        .ok()
        .map(|path| crate::ops_r1_observation::sample(std::path::Path::new(&path)));
    let (response, operations) = observe(|| {
        super::angular_resolver_registration_support::raw(
            &state,
            json!({
                "workspaceRoot":root,"type":"resolver_registrations","direction":"forward","route_path":"__ops_r1_absent__"
            }),
        )
    });
    if let Ok(evidence) = std::env::var("OPS_R1_EVIDENCE_DIR") {
        std::fs::write(
            std::path::Path::new(&evidence).join("query-response.json"),
            serde_json::to_vec_pretty(&response).unwrap(),
        )
        .unwrap();
    }
    assert!(response.get("error").is_none(), "{response:#}");
    assert_ne!(response["result"]["isError"], true, "{response:#}");
    let result = &response["result"]["structuredContent"];
    assert_eq!(result["registrations"], json!([]));
    assert_eq!(result["count"], 0);
    assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(result["coverage"]["source_complete"], false);
    assert_eq!(result["discovery"]["compiled"], 4);
    assert_eq!(state.test_full_semantic_parses(), 4);
    assert_eq!(
        operations.len(),
        1,
        "one executor operation owns the entire batch"
    );
    eprintln!("OPS_R1_SHARED_COUNTS {:?}", operations[0].counts());
    eprintln!("OPS_R1_FINAL {}", crate::ops_r1_observation::snapshot());
    eprintln!(
        "OPS_R1_SHARED_QUALIFICATION {:?}",
        operations[0].work_counts()
    );
    if let Ok(evidence) = std::env::var("OPS_R1_EVIDENCE_DIR") {
        std::fs::write(std::path::Path::new(&evidence).join("work-counts.json"), serde_json::to_vec_pretty(&json!({
            "acquisition_attempts_completed_unit_visits":operations[0].counts(),
            "qualification_attempts_reuses_seal_checks_validation_checks_input_visits":operations[0].work_counts(),
            "owner_compilations":state.test_full_semantic_parses(),"publications":result["discovery"]["compiled"]
        })).unwrap()).unwrap();
    }
    assert_eq!(
        operations[0].counts(),
        [1, 1, 4],
        "equivalent project acquired once; owner compilation/publication remains four"
    );
}

#[test]
fn ops_r1_warm_resolver_consumption_checks_shared_constraints_once() {
    use std::collections::HashSet;
    let _serial = crate::protocol::handler_response_serial();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    crate::tests::angular_authority_fixture::install(root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    for i in 0..4 {
        let peer = if i == 3 { "class Peer0 {}\n" } else { "" };
        std::fs::write(
            root.join(format!("owner{i}.ts")),
            format!("{peer}class Probe{i} {{ peer?: Peer0; }}\n"),
        )
        .unwrap();
    }
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    config.persistence.enabled = false;
    let state = McpState::new(config);
    let query = || {
        super::angular_resolver_registration_support::raw(
            &state,
            json!({"workspaceRoot":root,"type":"resolver_registrations","direction":"forward","route_path":"__ops_r1_absent__"}),
        )
    };
    let cold = query();
    assert!(cold.get("error").is_none(), "{cold:#}");
    assert_ne!(cold["result"]["isError"], true);
    assert_eq!(state.test_full_semantic_parses(), 4);
    let (per_owner, shared) = {
        let index = state.workspace_index_read();
        let mut total = 0;
        let mut union = HashSet::new();
        for i in 0..4 {
            let owner =
                state.semantic_owner_path(&root.join(format!("owner{i}.ts")).to_string_lossy());
            let proof = index
                .retained_typescript_authority(&owner)
                .expect("current native owner proof");
            let unique: HashSet<_> = proof.witnesses.iter().cloned().collect();
            assert!(!unique.is_empty());
            total += unique.len();
            union.extend(unique);
        }
        (total, union.len())
    };
    assert!(
        shared < per_owner,
        "owners demonstrably share exact typed constraints"
    );
    crate::work_observations_af_perf_002::reset();
    let warm = query();
    let checks = crate::work_observations_af_perf_002::snapshot().witnesses;
    assert!(warm.get("error").is_none(), "{warm:#}");
    assert_ne!(warm["result"]["isError"], true);
    let result = &warm["result"]["structuredContent"];
    assert_eq!(result["registrations"], json!([]));
    assert_eq!(result["count"], 0);
    assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
    assert_eq!(result["coverage"]["source_complete"], false);
    assert_eq!(
        state.test_full_semantic_parses(),
        4,
        "warm query must not compile again"
    );
    eprintln!(
        "OPS_R1_PHASE_COUNTS preparation={per_owner} consumption_union={shared} actual={checks}"
    );
    assert_eq!(
        checks,
        per_owner + shared,
        "one independent preparation phase and one shared consumption validation"
    );
}

#[test]
fn ops_r1_repeated_absent_queries_preserve_retained_owner_evidence() {
    let _serial = crate::protocol::handler_response_serial();
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    crate::tests::angular_authority_fixture::install(root);
    std::fs::write(root.join("tsconfig.json"), r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"include":["*.ts"]}"#).unwrap();
    for i in 0..4 {
        let peer = if i == 3 { "class Peer0 {}\n" } else { "" };
        std::fs::write(
            root.join(format!("owner{i}.ts")),
            format!("{peer}class Probe{i} {{ peer?: Peer0; }}\n"),
        )
        .unwrap();
    }
    let mut config = crate::tests::test_config();
    config.proxy.auto_start = false;
    config.persistence.enabled = false;
    let state = McpState::new(config);
    let retained = || {
        let index = state.workspace_index_read();
        (0..4)
            .map(|i| {
                let owner =
                    state.semantic_owner_path(&root.join(format!("owner{i}.ts")).to_string_lossy());
                let proof = index
                    .retained_typescript_authority(&owner)
                    .expect("retained native owner authority");
                serde_json::to_vec(&proof.witnesses).unwrap()
            })
            .collect::<Vec<_>>()
    };
    let mut original = None;
    let mut counts = Vec::new();
    for request in 0..13 {
        let (response, operations) = observe(|| {
            super::angular_resolver_registration_support::raw(
                &state,
                json!({"workspaceRoot":root,"type":"resolver_registrations","direction":"forward","route_path":format!("__ops_r1_absent_{request}__")}),
            )
        });
        assert!(response.get("error").is_none(), "{response:#}");
        assert_ne!(response["result"]["isError"], true, "{response:#}");
        let result = &response["result"]["structuredContent"];
        assert_eq!(result["registrations"], json!([]));
        assert_eq!(result["count"], 0);
        assert_eq!(result["coverage"]["result_semantics"], "lower_bound");
        assert_eq!(result["coverage"]["source_complete"], false);
        assert_eq!(state.test_full_semantic_parses(), 4);
        let actual: Vec<_> = operations.iter().map(|op| op.counts()).collect();
        if request == 0 {
            assert_eq!(actual, vec![[1, 1, 4]]);
            original = Some(retained());
        } else {
            assert!(actual.iter().all(|counts| counts == &[0, 0, 0]));
            assert_eq!(retained(), *original.as_ref().unwrap());
        }
        counts.push(actual);
    }
    let bytes: Vec<_> = original.unwrap().iter().map(Vec::len).collect();
    eprintln!("OPS_R1_REPEATED_REQUESTS requests=13 parses=4 retained_witness_bytes={bytes:?}");
    if let Ok(folder) = std::env::var("OPS_R1_EVIDENCE_DIR") {
        std::fs::write(
            std::path::Path::new(&folder).join("repeated-requests.json"),
            serde_json::to_vec_pretty(&json!({
                "requests":13,"owner_count":4,"semantic_parses":4,
                "per_request_acquisitions":counts,"retained_witness_bytes_per_owner":bytes,
                "retained_witness_bytes_unchanged":true,
                "scope":"one state, distinct absent selectors; not allocator-leak proof"
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
