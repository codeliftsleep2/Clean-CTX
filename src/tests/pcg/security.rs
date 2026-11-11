use super::support::{Fixture, fails};
use crate::pcg::{profile::Trusted, security};
use serde_json::json;
use std::fs;

#[test]
fn path_and_symlink_security() {
    for path in [
        "../a",
        "/a",
        "C:/a",
        "a\\b",
        "a/./b",
        "a//b",
        ".git/config",
        "a/.GIT/x",
        "a:b",
        "NUL.txt",
        "a. ",
    ] {
        assert!(security::relative(path).is_err(), "{path}");
    }
    let f = Fixture::new();
    let hardlink = f.root.join("hardlink.md");
    fs::hard_link(f.root.join("decision.md"), &hardlink).unwrap();
    assert_eq!(
        security::physical_identity(&f.root.join("decision.md")).unwrap(),
        security::physical_identity(&hardlink).unwrap()
    );
    let link = f.root.join("escape.md");
    #[cfg(unix)]
    std::os::unix::fs::symlink(f.dir.path().join("manifest.json"), &link).unwrap();
    #[cfg(windows)]
    {
        // A directory junction does not require Windows Developer Mode/admin.
        let status = std::process::Command::new("cmd")
            .arg("/c")
            .arg("mklink")
            .arg("/J")
            .arg(&link)
            .arg(f.dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "junction fixture could not be created");
    }
    assert!(security::source_path(&f.trusted().root, "escape.md").is_err());
    #[cfg(windows)]
    fs::remove_dir(link).unwrap();
}

#[test]
fn secret_signatures() {
    for value in [
        "-----BEGIN PRIVATE KEY-----".to_owned(),
        "-----BEGIN OPENSSH PRIVATE KEY-----".into(),
        format!("ghp_{}", "a".repeat(20)),
        format!("github_pat_{}", "_".repeat(20)),
        format!("AKIA{}", "A".repeat(16)),
        format!("sk-{}", "a".repeat(20)),
        "AUTHORIZATION : Bearer value".into(),
        "api-key = 'abcdefgh'".into(),
        "Password: abcdefgh".into(),
        "paſſword=abcdefgh".into(),
        "api_Key=abcdefgh".into(),
        "authorization\n:\nbearer token".into(),
    ] {
        fails(security::nonsecret(&value), "material_denied");
    }
    security::nonsecret("A public assertion about source currentness. 日本語").unwrap();
    let mut f = Fixture::new();
    f.manifest["records"][0]["authority"]["attributed_to"] = json!("password=abcdefgh");
    f.approve();
    fails(f.prepare(), "material_denied");
    assert!(!f.database().exists());
}

#[test]
fn profile_digest_and_source_boundary() {
    let mut f = Fixture::new();
    fs::write(f.dir.path().join("manifest.json"), b"{}").unwrap();
    fails(f.prepare(), "manifest_digest");
    f.approve();
    let trusted = f.trusted();
    f.profile["owner_label"] = json!("another owner");
    f.write_profile();
    fails(trusted.recheck(), "profile_changed");
    f.profile["owner_label"] = json!("fixture-owner");
    f.write_profile();
    let prepared = f.prepare().unwrap();
    fs::remove_file(f.root.join("decision.md")).unwrap();
    assert!(crate::pcg::import::finish(prepared, false, false).is_err());
    let empty = rusqlite::Connection::open(f.database()).unwrap();
    let n: i64 = empty
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE name='active_generation'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        n, 0,
        "failed initial admission must roll back even schema publication"
    );
    f.profile["workspace_root"] = json!("relative");
    f.write_profile();
    fails(Trusted::load(&f.profile_path), "profile_path");
}
