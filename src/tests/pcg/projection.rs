use super::*;
use crate::pcg::{
    query::{Limits, Mode, Subject},
    support::Fixture,
};
use serde_json::{Value, json};
use std::{fs, process::Command};

fn request(f: &Fixture) -> Request {
    Request {
        workspace_root: f.root.clone(),
        corpus: "fixture".into(),
        generation: f.run(false, false).unwrap().generation,
        mode: Mode::Constraints,
        subject: Subject {
            workspace: "fixture".into(),
            file: Some("decision.md".into()),
            entity: None,
            operation: Some("edit".into()),
            task: Some("modify".into()),
            at: "2026-11-11T12:00:00Z".into(),
        },
        records: vec![],
        include_history: false,
        limits: Limits {
            records: 4096,
            edges: 8192,
            evidence_bytes: 4194304,
        },
    }
}
fn payload(response: &Response) -> Value {
    let outer: Value = serde_json::from_str(&response.serialized_result).unwrap();
    assert_eq!(outer.as_object().unwrap().len(), 2);
    assert_eq!(outer["content"].as_array().unwrap().len(), 1);
    assert_eq!(outer["isError"], false);
    assert!(outer.get("structuredContent").is_none());
    serde_json::from_str(outer["content"][0]["text"].as_str().unwrap()).unwrap()
}
fn document(f: &mut Fixture, text: &str) {
    fs::write(f.root.join("decision.md"), text).unwrap();
    let git = |args: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(&f.root)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    };
    git(&["add", "--", "decision.md"]);
    git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "projection fixture",
    ]);
    f.manifest["sources"][0]["commit"] = json!(git(&["rev-parse", "HEAD"]));
    f.manifest["sources"][0]["blob"] = json!(git(&["rev-parse", "HEAD:decision.md"]));
    f.manifest["sources"][0]["sha256"] = json!(identity::sha(text.as_bytes()));
    f.manifest["evidence"][0]["end"] = json!(text.len());
    f.manifest["evidence"][0]["sha256"] = json!(identity::sha(text.as_bytes()));
    f.approve();
}
fn optional_fixture() -> Fixture {
    let mut f = Fixture::new();
    let original = fs::read_to_string(f.root.join("decision.md")).unwrap();
    let addition = format!(
        "\n## Optional support\n{}",
        "引用 \\\" evidence 日本語\n".repeat(160)
    );
    document(&mut f, &format!("{original}{addition}"));
    f.manifest["evidence"][0]["end"] = json!(original.len());
    f.manifest["evidence"][0]["sha256"] = json!(identity::sha(original.as_bytes()));
    f.manifest["evidence"].as_array_mut().unwrap().push(json!({"id":"optional-quote","source":"source",
        "start":original.len(),"end":original.len()+addition.len(),"sha256":identity::sha(addition.as_bytes())}));
    let mut record = f.manifest["records"][0].clone();
    record["id"] = json!("support");
    record["role"] = json!("evidence");
    record["required"] = json!(true); // Descriptive target never becomes required closure.
    record["primary"] = json!("optional-quote");
    record["evidence_refs"] = json!([]);
    record["subject"] = json!("descriptive_support");
    f.manifest["records"].as_array_mut().unwrap().push(record);
    f.manifest["edges"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"descriptive","kind":"USES_EVIDENCE",
        "from":"decision","to":"support","evidence":"quote"}));
    f.approve();
    f
}

#[test]
fn exact_serialized_result_and_unicode() {
    let mut f = Fixture::new();
    let text = "# Fixture decision\nKeep all conditions: 日本語 🚀 \\\"quoted\\\"\nNo omitted exception.\n";
    document(&mut f, text);
    let q = request(&f);
    for name in ["cl100k", "o200k"] {
        let response = project(&f.profile_path, &q, name, 8192).unwrap();
        let engine = create_tokenizer(TokenizerKind::from_str_opt(name).unwrap()).unwrap();
        assert_eq!(
            engine.count_tokens(&response.serialized_result),
            response.used_tokens
        );
        let p = payload(&response);
        assert_eq!(p["result_state"], "complete_within_registered_scope");
        assert_eq!(p["citations"][0]["text"], text);
        assert_eq!(
            p["citations"][0]["quote_sha256"],
            identity::sha(text.as_bytes())
        );
        assert_eq!(p["budget"]["required_material_fit"], true);
        assert_eq!(p["coverage"]["continuous_currentness"], false);
        assert_eq!(p["coverage"]["atomic_filesystem_snapshot"], false);
        assert_eq!(
            p["coverage"]["observations"]["working_completion"],
            "matched"
        );
        assert_eq!(response.counts.citation_encodings, 1);
        assert!(engine.count_tokens(p.to_string().as_str()) < response.used_tokens);
    }
}

#[test]
fn optional_first_dropping_preserves_required_material() {
    let f = optional_fixture();
    let q = request(&f);
    let full = project(&f.profile_path, &q, "o200k", 8192).unwrap();
    let full_payload = payload(&full);
    assert_eq!(full_payload["support"].as_array().unwrap().len(), 1);
    let bound = full.used_tokens as u64 - 1;
    let reduced = project(&f.profile_path, &q, "o200k", bound).unwrap();
    let p = payload(&reduced);
    assert_eq!(p["result_state"], "complete_within_registered_scope");
    assert_eq!(p["required_records"], full_payload["required_records"]);
    assert!(p["support"].as_array().unwrap().is_empty());
    assert_eq!(p["citations"].as_array().unwrap().len(), 1);
    assert_eq!(p["citations"][0], full_payload["citations"][0]);
    assert_eq!(p["budget"]["optional_truncated"], true);
    assert!(
        p["budget"]["optional_omissions"]
            .as_array()
            .unwrap()
            .contains(&json!("budget"))
    );
    assert!(reduced.used_tokens as u64 <= bound);
    assert_eq!(reduced.counts.citation_encodings, 2); // Each revision encoded once, despite repacking.
}

#[test]
fn mandatory_overflow_blocks_and_tiny_control_rejects() {
    let mut f = Fixture::new();
    document(
        &mut f,
        &format!(
            "# Fixture decision\n{}",
            "Mandatory condition 日本語\n".repeat(500)
        ),
    );
    let q = request(&f);
    let a = query::acquire(&f.profile_path, &q).unwrap();
    let trusted = Trusted::load(&f.profile_path).unwrap();
    let mut meter = Meter {
        engine: create_tokenizer(TokenizerKind::O200k).unwrap(),
        limit: 8192,
        name: "o200k",
        counts: Counts::default(),
    };
    let (_, control_tokens) = payload::control(&a, &q, &trusted, &mut meter, true).unwrap();
    let budget = control_tokens as u64 + 8;
    let result = project(&f.profile_path, &q, "o200k", budget).unwrap();
    let p = payload(&result);
    assert_eq!(p["result_state"], "blocked");
    assert_eq!(p["budget"]["required_material_fit"], false);
    assert_eq!(p["reasons"][0]["code"], "required_budget_exceeded");
    for key in [
        "required_records",
        "relationships",
        "citations",
        "support",
        "history",
    ] {
        assert!(p[key].as_array().unwrap().is_empty());
    }
    assert!(result.used_tokens as u64 <= budget);
    assert_eq!(
        project(&f.profile_path, &q, "o200k", 1).unwrap_err().0,
        "token_budget_too_small"
    );
}

#[test]
fn completion_failure_discards_counted_draft() {
    let f = Fixture::new();
    let q = request(&f);
    let path = f.root.join("decision.md");
    let response = project_with(&f.profile_path, &q, "cl100k", 8192, &mut || {
        fs::write(&path, "changed after packing").unwrap();
    })
    .unwrap();
    let p = payload(&response);
    assert_eq!(p["result_state"], "blocked");
    assert!(p["required_records"].as_array().unwrap().is_empty());
    assert!(p["citations"].as_array().unwrap().is_empty());
    assert!(
        p["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["code"] == "source_changed_during_read")
    );
    assert_eq!(
        p["coverage"]["observations"]["working_completion"],
        "failed"
    );
    assert!(response.counts.tokenization_calls >= 2);
}

#[test]
fn optional_work_limit_preserves_required_completeness() {
    let f = optional_fixture();
    let mut q = request(&f);
    q.limits.records = 4; // Two selectors, one binding and one required record.
    let mandatory = query::acquire(&f.profile_path, &q).unwrap();
    assert_eq!(mandatory.state, State::Ready);
    assert_eq!(
        mandatory.counts.selectors + mandatory.counts.bindings + mandatory.counts.records,
        4
    );
    let response = project(&f.profile_path, &q, "o200k", 8192).unwrap();
    let p = payload(&response);
    assert_eq!(p["result_state"], "complete_within_registered_scope");
    assert_eq!(p["required_records"].as_array().unwrap().len(), 1);
    assert_eq!(
        p["coverage"]["observations"]["working_completion"],
        "matched"
    );
    assert!(
        p["budget"]["optional_omissions"]
            .as_array()
            .unwrap()
            .contains(&json!("work_limit"))
    );
}

#[test]
fn exact_fit_and_deterministic_repeated_projection() {
    let f = Fixture::new();
    let q = request(&f);
    let first = project(&f.profile_path, &q, "o200k", 8192).unwrap();
    assert_eq!(
        first.serialized_result,
        project(&f.profile_path, &q, "o200k", 8192)
            .unwrap()
            .serialized_result
    );
    // The budget metadata itself changes the count: choose the fixed point independently.
    let mut limit = first.used_tokens as u64;
    let mut expected = first.serialized_result;
    let engine = create_tokenizer(TokenizerKind::O200k).unwrap();
    for _ in 0..8 {
        let outer: Value = serde_json::from_str(&expected).unwrap();
        let mut p: Value =
            serde_json::from_str(outer["content"][0]["text"].as_str().unwrap()).unwrap();
        p["budget"]["limit"] = json!(limit);
        // Match the intentional wrapper key order used by the result serializer.
        expected = format!(
            "{{\"content\":[{{\"type\":\"text\",\"text\":{}}}],\"isError\":false}}",
            serde_json::to_string(&identity::canonical(&p)).unwrap()
        );
        let counted = engine.count_tokens(&expected) as u64;
        if counted == limit {
            break;
        }
        limit = counted;
    }
    let exact = project(&f.profile_path, &q, "o200k", limit).unwrap();
    assert_eq!(
        payload(&exact)["result_state"],
        "complete_within_registered_scope"
    );
    assert_eq!(exact.used_tokens as u64, limit);
}

#[test]
fn approximate_tokenizer_and_profile_budget_refuse() {
    let f = Fixture::new();
    let q = request(&f);
    for kind in ["claude", "llama3", "unknown"] {
        assert_eq!(
            project(&f.profile_path, &q, kind, 8192).unwrap_err().0,
            "unsupported_tokenizer"
        );
    }
    assert_eq!(
        project(&f.profile_path, &q, "o200k", 8193).unwrap_err().0,
        "request_limits"
    );
}

#[test]
fn shared_required_witness_encoded_once() {
    let mut f = Fixture::new();
    for id in ["decision-b", "decision-c"] {
        let mut record = f.manifest["records"][0].clone();
        record["id"] = json!(id);
        f.manifest["records"].as_array_mut().unwrap().push(record);
        let mut binding = f.manifest["edges"][0].clone();
        binding["id"] = json!(format!("binding-{id}"));
        binding["from"] = json!(id);
        f.manifest["edges"].as_array_mut().unwrap().push(binding);
    }
    f.approve();
    let q = request(&f);
    let response = project(&f.profile_path, &q, "o200k", 8192).unwrap();
    let p = payload(&response);
    assert_eq!(p["required_records"].as_array().unwrap().len(), 3);
    assert_eq!(p["citations"].as_array().unwrap().len(), 1);
    assert_eq!(response.counts.citation_encodings, 1);
}
