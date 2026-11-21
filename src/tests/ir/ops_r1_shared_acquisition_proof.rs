//! Acquisition receipt/provenance and failure controls.
use super::support::{acquire, fixture, resolver, write};
use crate::ir::typescript_resolution::acquisition_inputs::{Input, visit};
use crate::ir::typescript_resolution::shared_acquisition::{basis, operation, reusable_sources};
use crate::ir::typescript_resolution::{Resolver, TypeScriptAuthority, Witness};
use std::sync::Arc;

#[test]
fn ops_r1_shared_precached_host_unit_project_and_owner_only_proof() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let mut first = resolver(path, "a.ts", &op);
    assert!(first.unit(&a).is_some());
    assert!(first.project_identity(&a).is_ok());
    write(path, "owner-only.txt", "original");
    assert!(first.host.read(&path.join("owner-only.txt")).is_some());
    acquire(&mut first, &a);
    let project = first.request_project(&a).unwrap();
    let key = basis(&first, &a, &project).unwrap();
    let proof = op.slot(key).lock().unwrap().as_ref().unwrap().clone();
    assert_eq!(proof.sources[&a].bytes.as_ref(), "class A { peer?: Z; }");
    let mut source_paths = Vec::new();
    visit(&proof.inputs, |input| {
        if let Input::Source(input) = input {
            source_paths.push(input.path.clone());
        }
    });
    assert!(source_paths.contains(&a) && source_paths.contains(&b));
    assert!(!source_paths.contains(&path.join("owner-only.txt")));
    write(path, "owner-only.txt", "changed");
    assert!(!first.host.current());
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &b);
    assert!(second.host.current());
    assert_eq!(op.counts(), [1, 1, 2]);
    assert_eq!(op.qualification_counts()[1], 1);
}

#[test]
fn ops_r1_shared_seed_is_not_sibling_disk_authority() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let mut first = resolver(path, "a.ts", &op);
    acquire(&mut first, &a);
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &b);
    assert_eq!(op.counts(), [1, 1, 2], "dependency proof must be reused");
    assert_eq!(op.qualification_counts()[1], 1);
    assert!(
        second
            .host
            .witnesses
            .iter()
            .any(|w| matches!(w, Witness::Content {path, ..} if path == &a))
    );
    assert!(
        !second
            .host
            .witnesses
            .iter()
            .any(|w| matches!(w, Witness::Content {path, ..} if path == &b))
    );
    write(path, "a.ts", "class A { changed: string; }");
    assert!(!second.host.current());
}

#[test]
fn ops_r1_shared_divergent_overlay_and_private_cache_are_not_overwritten() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let mut first = resolver(path, "a.ts", &op);
    acquire(&mut first, &a);
    let mut second = Resolver::new(&[path.to_path_buf()]).unwrap();
    second.operation = Some(Arc::clone(&op));
    second.acquisition_policy = "controls".into();
    second.host.witnesses.enable();
    let overlay = "class Different { peer?: Z; }";
    second.host.seed(&a, overlay).unwrap();
    assert!(second.unit(&a).is_some());
    acquire(&mut second, &a);
    assert_eq!(second.host.read(&a).unwrap().as_ref(), overlay);
    assert_eq!(op.counts(), [2, 2, 4]);
}

#[test]
fn ops_r1_shared_failed_partial_and_poisoned_leader_do_not_grant_or_poison() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    write(path, "b.ts", [255u8]);
    let a = path.join("a.ts");
    let mut first = resolver(path, "a.ts", &op);
    assert!(first.program_units(&a).is_err());
    write(path, "b.ts", "class Z {}");
    let mut second = resolver(path, "a.ts", &op);
    acquire(&mut second, &a);
    assert_eq!(op.counts(), [2, 1, 3]);
    let invalid: crate::ir::typescript_resolution::acquisition_inputs::Inputs =
        vec![Input::Observation(Arc::new(Witness::Unreadable {
            path: a.clone(),
        }))]
        .into();
    assert!(reusable_sources(&invalid, &op).is_none());
    assert!(reusable_sources(&Arc::from([]), &op).is_none());
    let other = operation();
    let mut third = resolver(path, "a.ts", &other);
    let project = third.request_project(&a).unwrap();
    let key = basis(&third, &a, &project).unwrap();
    let slot = other.slot(key);
    assert!(
        std::panic::catch_unwind(|| {
            let _guard = slot.lock().unwrap();
            panic!("failed leader");
        })
        .is_err()
    );
    acquire(&mut third, &a);
    assert_eq!(other.counts(), [1, 1, 2]);
}

#[test]
fn ops_r1_shared_standalone_capture_and_missing_module_qualification() {
    let root = fixture();
    let path = root.path();
    let a = path.join("a.ts");
    let source = std::fs::read_to_string(&a).unwrap();
    let before = TypeScriptAuthority::capture(&a, &source, &[path.to_path_buf()]);
    let op = operation();
    let after = TypeScriptAuthority::capture_in_operation(
        &a,
        &source,
        &[path.to_path_buf()],
        None,
        (&[], &[]),
        Some(op),
    );
    assert_eq!(before.source_hash, after.source_hash);
    assert_eq!(before.capture_current, after.capture_current);
    assert_eq!(before.disposition, after.disposition);
    assert_eq!(
        serde_json::to_value(&before.references).unwrap(),
        serde_json::to_value(&after.references).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&before).unwrap(),
        serde_json::to_value(&after).unwrap(),
        "receipt recording alone preserves standalone authority"
    );
    write(path, "a.ts", "import './absent'; class A { peer?: Z; }");
    let op = operation();
    let mut first = resolver(path, "a.ts", &op);
    acquire(&mut first, &a);
    assert!(
        !first.program_evidence[&path.join("tsconfig.json")]
            .missing_modules
            .is_empty()
    );
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &path.join("b.ts"));
    assert_eq!(
        op.counts(),
        [1, 1, 2],
        "missing-module proof must be reused"
    );
    assert_eq!(op.qualification_counts()[1], 1);
    assert!(second.host.current());
    assert!(
        !second.program_evidence[&path.join("tsconfig.json")]
            .missing_modules
            .is_empty()
    );
}

#[test]
fn ops_r1_shared_precached_attached_unit_replays_only_its_source() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let mut first = resolver(path, "a.ts", &op);
    acquire(&mut first, &a);
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &b);
    assert_eq!(
        op.counts(),
        [1, 1, 2],
        "attached-unit control requires reuse"
    );
    assert_eq!(op.qualification_counts()[1], 1);
    assert_eq!(second.attached_programs.len(), 1);
    let scope = second.host.witnesses.begin();
    assert!(second.unit(&a).is_some());
    let inputs = second.host.witnesses.finish(scope);
    let mut sources = Vec::new();
    visit(&inputs, |input| {
        if let Input::Source(input) = input {
            sources.push(input.path.clone());
        }
    });
    assert!(sources.contains(&a));
    assert!(!sources.contains(&b));
    assert!(!sources.contains(&path.join("tsconfig.json")));
}

#[test]
fn ops_r1_shared_failed_partial_after_one_unit_keeps_local_failure() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    write(path, "a.ts", [255u8]);
    let b = path.join("b.ts");
    let mut first = resolver(path, "b.ts", &op);
    assert!(first.program_units(&b).is_err());
    assert_eq!(op.counts(), [1, 0, 2]);
    write(path, "a.ts", "class A { peer?: Z; }");
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &b);
    assert_eq!(op.counts(), [2, 1, 4]);
    assert!(
        first.program_units(&b).is_err(),
        "existing owner-local failure memoization remains local"
    );
    assert_eq!(op.counts(), [2, 1, 4]);
}

#[test]
fn ops_r1_shared_precached_unaccounted_values_never_seal_from_nonempty_witnesses() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let mut late = Resolver::new(&[path.to_path_buf()]).unwrap();
    late.host
        .seed(&a, &std::fs::read_to_string(&a).unwrap())
        .unwrap();
    assert!(late.unit(&a).is_some());
    assert!(late.project_identity(&a).is_ok());
    assert!(!late.host.witnesses.is_empty());
    late.operation = Some(Arc::clone(&op));
    late.acquisition_policy = "controls".into();
    late.host.witnesses.enable();
    acquire(&mut late, &a);
    let project = late.request_project(&a).unwrap();
    let key = basis(&late, &a, &project).unwrap();
    assert!(op.slot(key).lock().unwrap().is_none());
    let mut accounted = resolver(path, "b.ts", &op);
    acquire(&mut accounted, &path.join("b.ts"));
    assert_eq!(op.counts(), [2, 2, 4]);
}
