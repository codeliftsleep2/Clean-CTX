use super::super::e1_metrics as metrics;
use super::*;
use serde_json::json;
use std::time::Instant;

#[test]
fn e2_admission_survives_optional_semantic_failure() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app.ts");
    std::fs::write(&app, "const x=[{path:'a',resolve:{user:UserResolver}}];").unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"types":["node"],"noLib":true},"files":["app.ts"]}"#,
    )
    .unwrap();
    let node = root.path().join("node_modules/@types/node");
    std::fs::create_dir_all(&node).unwrap();
    std::fs::write(node.join("package.json"), r#"{"types":"index.d.ts"}"#).unwrap();
    std::fs::write(node.join("index.d.ts"), "declare class Broken {").unwrap();
    metrics::reset();
    let admitted = Candidate::capture(&[root.path().to_path_buf()], &app)
        .unwrap()
        .admit()
        .unwrap();
    assert_eq!(metrics::snapshot().acquisition_results, 0);
    let mut resolver = super::super::Resolver::new(&[root.path().to_path_buf()]).unwrap();
    assert!(resolver.program_units(&app).is_err());
    assert_eq!(metrics::snapshot().acquisition_results, 1);
    assert_eq!(admitted.observations().len(), 1);
    assert!(admitted.is_current());
}

#[test]
fn e2_escaped_keys_and_utf8_spans_are_conservative() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app.ts");
    let source = "const x=[{path:'café',resolve:{'üser':User}}];";
    std::fs::write(&app, source).unwrap();
    let admitted = Candidate::capture(&[root.path().to_path_buf()], &app)
        .unwrap()
        .admit()
        .unwrap();
    let observation = &admitted.observations()[0];
    assert_eq!(observation.route_path, "café");
    assert_eq!(observation.key, "üser");
    assert_eq!(
        &source[observation.key_span.start..observation.key_span.end],
        "'üser'"
    );
    std::fs::write(&app, r"const x=[{path:'a',resolve:{user:A,u\u0073er:B}}];").unwrap();
    let admitted = Candidate::capture(&[root.path().to_path_buf()], &app)
        .unwrap()
        .admit()
        .unwrap();
    assert!(admitted.observations().is_empty());
    std::fs::remove_file(&app).unwrap();
    assert!(!admitted.is_current());
}

#[test]
fn e2_observation_cost() {
    let Ok(root) = std::env::var("OPS_E2_WORKSPACE") else {
        return;
    };
    let root = PathBuf::from(root);
    let output = PathBuf::from(std::env::var("OPS_E2_OUTPUT").unwrap());
    let app = root.join("app.ts");
    let mut samples = Vec::new();
    for _ in 0..30 {
        metrics::reset();
        let start = Instant::now();
        let candidate = Candidate::capture(std::slice::from_ref(&root), &app).unwrap();
        let capture_ms = start.elapsed().as_secs_f64() * 1000.0;
        let capture_metrics = metrics::finish();
        assert_eq!(metrics::snapshot().parses, 1);
        assert_eq!(metrics::snapshot().dependency_parses, 0);
        assert_eq!(metrics::snapshot().acquisition_results, 0);
        assert_eq!(metrics::snapshot().automatic_type_calls, 0);
        assert_eq!(metrics::snapshot().library_calls, 0);
        metrics::reset();
        let start = Instant::now();
        let admitted = candidate.admit().unwrap();
        let validation_ms = start.elapsed().as_secs_f64() * 1000.0;
        let validation_metrics = metrics::finish();
        assert_eq!(admitted.observations().len(), 1);
        metrics::reset();
        let before = metrics::snapshot();
        let observations:Vec<_>=admitted.observations().iter().map(|o|{
            let (text,span)=match &o.expression {
                WrittenExpression::ReferenceText{text,span}|WrittenExpression::Unsupported{text,span,..}=>(text,span),
            };
            json!({"route_path":o.route_path,"key":o.key,"expression":text,
                "object_span":[o.object.start,o.object.end],"key_span":[o.key_span.start,o.key_span.end],
                "expression_span":[span.start,span.end],"class":"written_source_observation"})
        }).collect();
        assert_eq!(before, metrics::snapshot());
        samples.push(
            json!({"capture_ms":capture_ms,"validation_ms":validation_ms,
            "capture":capture_metrics,"validation":validation_metrics,"source":admitted.source(),
            "source_hash":admitted.source_hash(),"observations":observations,
            "serialized_witness_bytes":admitted.serialized_witness_bytes(),
            "retained_record_text_bytes":admitted.retained_record_text_bytes(),
            "retained_source_bytes":0,"retained_program_units":0,"production_publications":0}),
        );
    }
    std::fs::write(
        output.join("observations.json"),
        serde_json::to_vec_pretty(&samples).unwrap(),
    )
    .unwrap();
}
