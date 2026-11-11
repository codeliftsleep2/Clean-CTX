//! Small real-process storage controls, not a substitute for public CLI verification.
use super::support::Fixture;
use crate::pcg::{import, store};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[test]
fn fixture_worker() {
    let Some(path) = std::env::var_os("PCG_TEST_PROFILE") else {
        return;
    };
    let mode = std::env::var("PCG_TEST_MODE").unwrap();
    let marker = PathBuf::from(std::env::var_os("PCG_TEST_MARKER").unwrap());
    if mode == "reopen" {
        let p = import::prepare(Path::new(&path), true).unwrap();
        let report = import::finish(p, false, true).unwrap();
        fs::write(marker, report.generation).unwrap();
        return;
    }
    let p = import::prepare(Path::new(&path), false).unwrap();
    let (mut connection, new) = store::writer(&p.trusted).unwrap();
    let transaction = store::begin(&mut connection).unwrap();
    store::initialize(&transaction, new).unwrap();
    store::insert(&transaction, &p.trusted, &p.generation).unwrap();
    store::publish(&transaction, &p.trusted, &p.generation).unwrap();
    if mode == "postcommit" {
        store::commit(transaction).unwrap();
        fs::write(&marker, b"committed").unwrap();
        thread::sleep(Duration::from_secs(20));
    } else {
        fs::write(&marker, b"uncommitted").unwrap();
        thread::sleep(Duration::from_secs(20));
        drop(transaction);
    }
}

fn child(f: &Fixture, mode: &str) -> (Child, PathBuf) {
    let marker = f.dir.path().join(format!("{mode}.marker"));
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "pcg::process_tests::fixture_worker",
            "--nocapture",
        ])
        .env("PCG_TEST_PROFILE", &f.profile_path)
        .env("PCG_TEST_MODE", mode)
        .env("PCG_TEST_MARKER", &marker)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    (child, marker)
}
fn ready(child: &mut Child, marker: &Path) {
    let start = Instant::now();
    while !marker.exists() {
        if start.elapsed() > Duration::from_secs(10) || child.try_wait().unwrap().is_some() {
            let _ = child.kill();
            let _ = child.wait();
            panic!("fixture process did not reach its barrier");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn fresh_process_reopen() {
    let f = Fixture::new();
    let report = f.run(false, false).unwrap();
    let (mut worker, marker) = child(&f, "reopen");
    ready(&mut worker, &marker);
    assert!(worker.wait().unwrap().success());
    assert_eq!(fs::read_to_string(marker).unwrap(), report.generation);
}

#[test]
fn crash_transaction_boundaries() {
    for mode in ["precommit", "postcommit"] {
        let f = Fixture::new();
        let expected = f.prepare().unwrap().generation.id;
        let (mut worker, marker) = child(&f, mode);
        ready(&mut worker, &marker);
        worker.kill().unwrap();
        worker.wait().unwrap();
        if mode == "precommit" {
            assert!(store::reader(&f.trusted(), "fixture").is_err());
            // The interrupted initial schema is absent too. Explicit initial
            // admission can initialize that empty file without a repair fallback.
            assert_eq!(f.run(false, false).unwrap().generation, expected);
        } else {
            let reader = store::reader(&f.trusted(), "fixture").unwrap();
            assert_eq!(
                store::head(&reader, &f.trusted(), "fixture").unwrap(),
                Some(expected)
            );
            f.run(false, true).unwrap();
        }
    }
}
