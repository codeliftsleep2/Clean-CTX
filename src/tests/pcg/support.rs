use crate::pcg::{Error, identity, import, profile::Trusted};
use serde_json::{Value, json};
use std::{fs, path::PathBuf, process::Command};

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub root: PathBuf,
    pub profile_path: PathBuf,
    pub manifest: Value,
    pub profile: Value,
}
impl Fixture {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        fs::create_dir(&root).unwrap();
        let document = include_bytes!("../../../verification/pcg-175/fixtures/decision.md");
        fs::write(root.join("decision.md"), document).unwrap();
        let git = |args: &[&str]| {
            let output = Command::new("git")
                .arg("-C")
                .arg(&root)
                .args(args)
                .env("GIT_CONFIG_COUNT", "0")
                .env_remove("GIT_DIR")
                .env_remove("GIT_WORK_TREE")
                .env("GIT_TERMINAL_PROMPT", "0")
                .output()
                .unwrap();
            assert!(output.status.success(), "fixture Git operation failed");
            String::from_utf8(output.stdout).unwrap().trim().to_owned()
        };
        git(&["init", "--quiet"]);
        git(&["config", "core.autocrlf", "false"]);
        git(&["add", "--", "decision.md"]);
        git(&[
            "-c",
            "user.name=PCG Fixture",
            "-c",
            "user.email=pcg@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "core.hooksPath=",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ]);
        let commit = git(&["rev-parse", "HEAD"]);
        let blob = git(&["rev-parse", "HEAD:decision.md"]);
        let manifest = json!({
            "schema_version":1,"parser_version":"pcg-json-v1","corpus_id":"fixture","source_namespace":"fixture:git",
            "enrollment":{"owner_label":"fixture-owner","policy_revision":"fixture-v1","complete":true},
            "sources":[{"id":"source","path":"decision.md","commit":commit,"blob":blob,"sha256":identity::sha(document),"check":"current"}],
            "selectors":[{"id":"coverage","purpose":"coverage","workspace":"fixture","from":null,"until":null},
                {"id":"binding","purpose":"applicability","workspace":"fixture","file":"decision.md","operation":"edit","task":"modify","from":null,"until":null}],
            "evidence":[{"id":"quote","source":"source","start":0,"end":document.len(),"sha256":identity::sha(document),"heading":"# Fixture decision"}],
            "records":[{"id":"decision","role":"decision","required":true,"primary":"quote",
                "authority":{"class":"explicitly_approved_decision","attributed_to":"fixture source"},
                "verification":{"outcome":"supported","subject":"retain_reads","execution":"report_of_execution"},
                "lifecycle":"active","subject":"read_boundary","disposition":"PROTECTED","evidence_refs":[{"id":"quote","required":true}]}],
            "edges":[{"id":"binding","kind":"CONSTRAINS","from":"decision","to_selector":"binding","evidence":"quote"}]
        });
        let profile_path = dir.path().join("profile.json");
        let profile = json!({"schema_version":1,"partition_id":"fixture","owner_label":"fixture-owner","workspace_id":"fixture",
            "workspace_root":root.canonicalize().unwrap(),"database_path":dir.path().join("projection.db"),
            "manifest_path":dir.path().join("manifest.json"),"approved_manifest_sha256":"",
            "policy_revision":"fixture-v1","history_allowed":true,
            "limits":{"records":4096,"edges":8192,"evidence_bytes":4194304,"token_budget":8192}});
        let mut fixture = Self {
            dir,
            root,
            profile_path,
            manifest,
            profile,
        };
        fixture.approve();
        fixture
    }
    pub fn approve(&mut self) {
        let bytes = serde_json::to_vec(&self.manifest).unwrap();
        self.profile["approved_manifest_sha256"] = json!(identity::sha(&bytes));
        fs::write(self.dir.path().join("manifest.json"), bytes).unwrap();
        self.write_profile();
    }
    pub fn write_profile(&self) {
        fs::write(
            &self.profile_path,
            serde_json::to_vec(&self.profile).unwrap(),
        )
        .unwrap();
    }
    pub fn prepare(&self) -> Result<import::Prepared, Error> {
        import::prepare(&self.profile_path, false)
    }
    pub fn run(&self, rebuild: bool, audit: bool) -> Result<import::Report, Error> {
        import::finish(import::prepare(&self.profile_path, audit)?, rebuild, audit)
    }
    pub fn trusted(&self) -> Trusted {
        Trusted::load(&self.profile_path).unwrap()
    }
    pub fn database(&self) -> PathBuf {
        self.dir.path().join("projection.db")
    }
    pub fn add_claim(&mut self, id: &str) {
        let mut record = self.manifest["records"][0].clone();
        record["id"] = json!(id);
        record["role"] = json!("claim");
        record["required"] = json!(false);
        record["subject"] = json!(id);
        self.manifest["records"]
            .as_array_mut()
            .unwrap()
            .push(record);
    }
}

pub fn fails<T>(result: Result<T, Error>, code: &str) {
    match result {
        Ok(_) => panic!("expected {code}"),
        Err(e) => assert_eq!(e.0, code),
    }
}
