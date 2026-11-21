//! Exact constraint compaction must preserve dependency receipts and fresh truth.
use crate::ir::typescript_resolution::Witness;
use crate::ir::typescript_resolution::acquisition_inputs::{Input, InputLog, Inputs, visit};
use std::sync::Arc;

#[test]
fn ops_r1_shared_empty_automatic_types_reuses_project_across_owner_directories() {
    use super::support::{acquire, fixture, resolver, write};
    use crate::ir::typescript_resolution::shared_acquisition::operation;

    let root = fixture();
    let path = root.path();
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["a.ts","sub/b.ts"]}"#,
    );
    write(path, "sub/b.ts", "class Z {}");
    let op = operation();
    let a = path.join("a.ts");
    let b = path.join("sub/b.ts");
    assert_ne!(
        a.parent(),
        b.parent(),
        "genuinely different owner directories"
    );
    let mut first = resolver(path, "a.ts", &op);
    let first_project = first.project_identity(&a).unwrap();
    acquire(&mut first, &a);
    let mut second = resolver(path, "sub/b.ts", &op);
    let second_project = second.project_identity(&b).unwrap();
    assert_eq!(first_project.config, second_project.config);
    assert_eq!(
        serde_json::to_value(&first_project.options).unwrap(),
        serde_json::to_value(&second_project.options).unwrap()
    );
    assert_eq!(second_project.options["types"].value, serde_json::json!([]));
    acquire(&mut second, &b);
    assert!(first.host.current());
    assert!(second.host.current());
    assert_eq!(op.counts(), [1, 1, 2], "one equivalent project acquisition");
    assert_eq!(
        op.work_counts()[1],
        1,
        "recipient must qualify independently"
    );
}

#[test]
fn ops_r1_shared_import_compacts_exact_constraints_without_losing_receipts() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("owner.ts");
    std::fs::write(&path, "export class Before {}").unwrap();
    let before = Witness::Content {
        path: path.clone(),
        hash: crate::cache::content_hash(&std::fs::read(&path).unwrap()),
    };
    let inputs: Inputs = vec![
        Input::Observation(Arc::new(before.clone())),
        Input::Observation(Arc::new(before.clone())),
    ]
    .into();
    let mut log = InputLog::new(vec![before.clone()]);
    log.enable();
    let scope = log.begin();
    log.import(vec![before.clone(), before.clone()], &inputs);
    let receipt = log.finish(scope);
    let mut observations = 0;
    visit(&receipt, |input| {
        if let Input::Observation(witness) = input {
            assert_eq!(witness.as_ref(), &before);
            observations += 1;
        }
    });
    assert_eq!(
        observations, 2,
        "dependency receipt nodes remain attributable"
    );
    assert!(log.iter().all(Witness::current));
    assert_eq!(log.len(), 1, "exact imported constraints are idempotent");

    std::fs::write(&path, "export class After {}").unwrap();
    assert!(
        !log.iter().all(Witness::current),
        "freshness is never cached"
    );
    let after = Witness::Content {
        path,
        hash: crate::cache::content_hash(b"export class After {}"),
    };
    log.import(vec![after.clone(), after.clone()], &inputs);
    assert_eq!(
        log.len(),
        2,
        "different content identities must remain distinct"
    );
    assert_eq!(&log[0], &before, "stable first-seen constraint order");
    assert_eq!(&log[1], &after);
    assert!(
        !log.iter().all(Witness::current),
        "stale evidence is retained"
    );
}
