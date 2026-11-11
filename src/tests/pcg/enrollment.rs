use super::support::{Fixture, fails};
use crate::pcg::{enrollment, identity, profile::Trusted};
use serde_json::json;
use std::fs;

#[test]
fn strict_json_and_profile() {
    for bytes in [
        br#"{"a":1,"a":2}"#.as_slice(),
        br#"{"x":{"a":1,"a":2}}"#,
        b"\xef\xbb\xbf{}",
        b"\xff",
        b"{\"a\":1.5}",
        b"{\"a\":1e0}",
        b"{\"a\":NaN}",
        b"{} trailing",
    ] {
        assert!(identity::parse(bytes).is_err());
    }
    let mut f = Fixture::new();
    f.profile["untrusted"] = json!(true);
    f.write_profile();
    fails(Trusted::load(&f.profile_path), "profile_schema");
    f.profile.as_object_mut().unwrap().remove("untrusted");
    f.profile["schema_version"] = json!(2);
    f.write_profile();
    fails(Trusted::load(&f.profile_path), "profile_schema");
}

#[test]
fn manifest_negatives() {
    let f = Fixture::new();
    let trusted = f.trusted();
    let mut cases = Vec::new();
    let mut v = f.manifest.clone();
    v["surprise"] = json!(true);
    cases.push((v, "manifest_schema"));
    let mut v = f.manifest.clone();
    v["records"][0]["approved"] = json!(true);
    cases.push((v, "manifest_schema"));
    let mut v = f.manifest.clone();
    v["parser_version"] = json!("pcg-json-v2");
    cases.push((v, "parser_version"));
    let mut v = f.manifest.clone();
    let duplicate = v["records"][0].clone();
    v["records"].as_array_mut().unwrap().push(duplicate);
    cases.push((v, "duplicate_id"));
    let mut v = f.manifest.clone();
    v["selectors"][0]["until"] = json!("2026-02-30T00:00:00Z");
    cases.push((v, "selector_interval"));
    let mut v = f.manifest.clone();
    v["selectors"][0].as_object_mut().unwrap().remove("from");
    cases.push((v, "selector_interval"));
    let mut v = f.manifest.clone();
    v["sources"][0]["path"] = json!("../escape");
    cases.push((v, "unsafe_path"));
    let mut v = f.manifest.clone();
    v["records"][0]["primary"] = json!("missing");
    cases.push((v, "primary_missing"));
    let mut v = f.manifest.clone();
    v["records"][0]["evidence_refs"][0]["id"] = json!("missing");
    cases.push((v, "required_ref_missing"));
    let mut v = f.manifest.clone();
    v["edges"][0]["evidence"] = json!("missing");
    cases.push((v, "edge_provenance"));
    for (v, code) in cases {
        fails(
            enrollment::decode(&serde_json::to_vec(&v).unwrap(), &trusted),
            code,
        );
    }
}

#[test]
fn graph_negatives() {
    let mut f = Fixture::new();
    f.add_claim("premise");
    let base = f.manifest.clone();
    let edge = json!({"id":"dependency","kind":"DEPENDS_ON","from":"decision","to":"premise","evidence":"quote"});
    f.manifest["edges"]
        .as_array_mut()
        .unwrap()
        .push(edge.clone());
    let reverse = json!({"id":"reverse","kind":"BLOCKED_BY","from":"premise","to":"decision","evidence":"quote",
        "gate":{"outcome":"supported","subject":"retain_reads","authority":"verified_claim"}});
    f.manifest["edges"].as_array_mut().unwrap().push(reverse);
    f.approve();
    fails(f.prepare(), "required_cycle");
    f.manifest = base.clone();
    let mut invalid = edge;
    invalid["kind"] = json!("USES_EVIDENCE");
    f.manifest["edges"].as_array_mut().unwrap().push(invalid);
    f.approve();
    fails(f.prepare(), "edge_role");
    f.manifest = base.clone();
    let mut peer = f.manifest["records"][0].clone();
    peer["id"] = json!("peer");
    peer["disposition"] = json!("REJECT");
    f.manifest["records"].as_array_mut().unwrap().push(peer);
    let mut binding = f.manifest["edges"][0].clone();
    binding["id"] = json!("peer-binding");
    binding["from"] = json!("peer");
    f.manifest["edges"].as_array_mut().unwrap().push(binding);
    f.approve();
    fails(f.prepare(), "authority_conflict");
    f.manifest = base;
    f.manifest["records"][0]["lifecycle"] = json!("superseded");
    let mut successor = f.manifest["records"][0].clone();
    successor["id"] = json!("successor");
    successor["lifecycle"] = json!("active");
    f.manifest["records"]
        .as_array_mut()
        .unwrap()
        .push(successor);
    let mut binding = f.manifest["edges"][0].clone();
    binding["id"] = json!("successor-binding");
    binding["from"] = json!("successor");
    f.manifest["edges"].as_array_mut().unwrap().push(binding);
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"supersession","kind":"SUPERSEDES","from":"successor","to":"decision","evidence":"quote"}));
    f.approve();
    f.prepare().unwrap();
    let valid_supersession = f.manifest.clone();
    f.manifest["edges"]
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()["to"] = json!("successor");
    f.approve();
    fails(f.prepare(), "supersession");
    f.manifest = valid_supersession;
    f.manifest["records"][2]["lifecycle"] = json!("superseded");
    f.manifest["edges"].as_array_mut().unwrap().push(json!({"id":"cycle","kind":"SUPERSEDES","from":"decision","to":"successor","evidence":"quote"}));
    f.approve();
    fails(f.prepare(), "required_cycle");
}

#[test]
fn git_and_span_negatives() {
    let mut f = Fixture::new();
    let base = f.manifest.clone();
    for (field, value, code) in [
        ("blob", "0".repeat(40), "git_membership"),
        ("commit", "0".repeat(40), "git_object_invalid"),
        ("sha256", "0".repeat(64), "source_hash"),
    ] {
        f.manifest = base.clone();
        f.manifest["sources"][0][field] = json!(value);
        f.approve();
        fails(f.prepare(), code);
    }
    f.manifest = base.clone();
    f.manifest["evidence"][0]["sha256"] = json!("0".repeat(64));
    f.approve();
    fails(f.prepare(), "evidence_hash");
    f.manifest = base.clone();
    f.manifest["evidence"][0]["end"] = json!(99999);
    f.approve();
    fails(f.prepare(), "evidence_span");
    f.manifest = base.clone();
    f.manifest["evidence"][0]["heading"] = json!("# Missing");
    f.approve();
    fails(f.prepare(), "evidence_heading");
    f.manifest = base.clone();
    let bytes = fs::read(f.root.join("decision.md")).unwrap();
    let start = std::str::from_utf8(&bytes).unwrap().find("日本語").unwrap() + 1;
    f.manifest["evidence"][0]["start"] = json!(start);
    f.manifest["evidence"][0]["end"] = json!(start + 2);
    f.manifest["evidence"][0]["sha256"] = json!(identity::sha(&bytes[start..start + 2]));
    f.approve();
    fails(f.prepare(), "evidence_span");
    f.manifest = base;
    f.approve();
    fs::write(f.root.join("decision.md"), b"changed source").unwrap();
    fails(f.prepare(), "source_stale");
}

#[test]
fn canonical_vectors() {
    // Independent published SHA-256 empty/abc vectors and a literal canonical string.
    assert_eq!(
        identity::sha(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        identity::sha(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        identity::canonical(&json!({"z":null,"a":[true,"日本語\n",1]})),
        "{\"a\":[true,\"日本語\\n\",1],\"z\":null}"
    );
    // Literal canonical UTF-8 digest independently calculated with .NET SHA256.
    assert_eq!(
        identity::hash(&json!({"z":null,"a":[true,"日本語\n",1]})),
        "5ce35d7a18702f8bd145d38d342e2cfe097d3d7f0d7db28342d163d8e676d3f8"
    );
    assert_eq!(
        identity::set([json!("b"), json!("a"), json!("a")]),
        json!(["a", "b"])
    );
    assert_ne!(
        identity::hash(&json!("é")),
        identity::hash(&json!("e\u{301}"))
    );
    let mut f = Fixture::new();
    f.add_claim("second");
    f.approve();
    let a = f.prepare().unwrap();
    let b = f.prepare().unwrap();
    assert_eq!(a.generation.id, b.generation.id);
    assert_eq!(a.generation.rows, b.generation.rows);
    f.manifest["records"].as_array_mut().unwrap().reverse();
    f.manifest["selectors"].as_array_mut().unwrap().reverse();
    f.approve();
    let permuted = f.prepare().unwrap();
    // Raw manifest approval bytes participate in G. Set ordering preserves row
    // revisions, while a newly approved raw digest necessarily changes G.
    assert_ne!(a.generation.id, permuted.generation.id);
    for table in [
        "source_revision",
        "evidence_revision",
        "record_revision",
        "selector",
        "binding",
    ] {
        let without_generation = |rows: &crate::pcg::model::Rows| {
            rows.0[table]
                .iter()
                .map(|r| r[3..].to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            without_generation(&a.generation.rows),
            without_generation(&permuted.generation.rows)
        );
    }
}
