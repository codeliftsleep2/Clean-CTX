//! Exact constraint compaction must preserve dependency receipts and fresh truth.
use crate::ir::typescript_resolution::Witness;
use crate::ir::typescript_resolution::acquisition_inputs::{Input, InputLog, Inputs, visit};
use std::sync::Arc;

#[test]
fn ops_r1_shared_retained_currentness_checks_exact_constraints_once_per_call() {
    use super::support::{fixture, write};
    use crate::ir::typescript_resolution::TypeScriptAuthority;
    use crate::workspace::index::{SemanticFidelity, WorkspaceIndex};
    use std::collections::HashSet;

    let root = fixture();
    let path = root.path();
    let source = "export class A {}";
    write(path, "a.ts", source);
    let file = path.join("a.ts");
    let mut authority = TypeScriptAuthority::capture(&file, source, &[path.to_path_buf()]);
    assert!(authority.capture_current);
    assert!(
        authority.disposition.is_none(),
        "{:?}",
        authority.disposition
    );
    assert!(!authority.witnesses.is_empty());
    let config = authority
        .witnesses
        .iter()
        .find_map(|witness| match witness {
            Witness::Content { path, .. }
                if path.file_name() == Some(std::ffi::OsStr::new("tsconfig.json")) =>
            {
                Some(path.clone())
            }
            _ => None,
        })
        .expect("configuration is an actually consumed disk-backed dependency");
    assert_eq!(
        config.canonicalize().unwrap(),
        path.join("tsconfig.json").canonicalize().unwrap()
    );
    authority.witnesses.extend(authority.witnesses.clone());
    let unique = authority.witnesses.iter().collect::<HashSet<_>>().len();
    let hash = authority.source_hash.clone();
    let mut index = WorkspaceIndex::new();
    index
        .replace_semantic_projection_with_retained_authority(
            file.to_str().unwrap(),
            vec![],
            SemanticFidelity::High,
            hash.clone(),
            vec![],
            Some(authority.clone()),
        )
        .expect("current native authority is admitted");
    for _ in 0..2 {
        crate::work_observations_af_perf_002::reset();
        assert!(index.has_current_semantic_projection(
            file.to_str().unwrap(),
            SemanticFidelity::High,
            &hash
        ));
        assert_eq!(
            crate::work_observations_af_perf_002::snapshot().witnesses,
            unique,
            "one exact retained-currentness constraint per call"
        );
    }
    let mut conflicting = authority;
    conflicting.witnesses.push(Witness::Content {
        path: config.clone(),
        hash: crate::cache::content_hash(b"different source"),
    });
    assert!(
        WorkspaceIndex::new()
            .replace_semantic_projection_with_retained_authority(
                file.to_str().unwrap(),
                vec![],
                SemanticFidelity::High,
                hash.clone(),
                vec![],
                Some(conflicting),
            )
            .is_err(),
        "distinct hash constraints must not be merged by path"
    );
    assert!(
        !index.has_current_semantic_projection(
            file.to_str().unwrap(),
            SemanticFidelity::High,
            &crate::cache::content_hash(b"export class Changed {}"),
        ),
        "the caller supplies current owner identity"
    );
    std::fs::write(
        &config,
        r#"{"compilerOptions":{"strict":true,"noLib":true,"types":[]}}"#,
    )
    .unwrap();
    assert!(!index.has_current_semantic_projection(
        file.to_str().unwrap(),
        SemanticFidelity::High,
        &hash
    ));
}

#[test]
fn ops_r1_shared_seal_checks_each_exact_constraint_once_per_call() {
    use crate::ir::typescript_resolution::shared_acquisition::{operation, reusable_sources};

    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("dependency.ts");
    std::fs::write(&path, "class Original {}").unwrap();
    let witness = Witness::Content {
        path: path.clone(),
        hash: crate::cache::content_hash(b"class Original {}"),
    };
    let inputs: Inputs = vec![
        Input::Observation(Arc::new(witness.clone())),
        Input::Observation(Arc::new(witness)),
    ]
    .into();
    let op = operation();
    for _ in 0..2 {
        let before = op.work_counts()[2];
        assert!(reusable_sources(&inputs, &op).is_some());
        assert_eq!(
            op.work_counts()[2] - before,
            1,
            "one exact seal constraint per call"
        );
    }
    std::fs::write(&path, "class Changed {}").unwrap();
    assert!(
        reusable_sources(&inputs, &op).is_none(),
        "later calls must revalidate"
    );
}

#[test]
fn ops_r1_shared_qualification_checks_each_exact_constraint_once_per_call() {
    use super::support::{acquire, fixture, resolver, write};
    use crate::ir::typescript_resolution::shared_acquisition::{basis, operation};
    use std::collections::HashSet;

    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let mut leader = resolver(path, "a.ts", &op);
    acquire(&mut leader, &a);
    let project = leader.project_identity(&a).unwrap();
    let key = basis(&leader, &a, &project).unwrap();
    let proof = op.slot(key).lock().unwrap().as_ref().unwrap().clone();
    for _ in 0..2 {
        let mut recipient = resolver(path, "b.ts", &op);
        let before = op.work_counts()[3];
        let qualified = proof
            .qualify(&mut recipient, &op)
            .expect("equivalent recipient");
        let mut unique = HashSet::new();
        let mut observations = 0;
        visit(&qualified.bound.inputs, |input| {
            if let Input::Observation(witness) = input {
                unique.insert(witness.as_ref().clone());
                observations += 1;
            }
        });
        assert!(
            observations > unique.len(),
            "duplicate receipt nodes remain attributable"
        );
        assert_eq!(
            op.work_counts()[3] - before,
            unique.len(),
            "one exact qualification constraint per call"
        );
    }
    write(path, "a.ts", "class Changed {}");
    let mut changed = resolver(path, "b.ts", &op);
    assert!(
        proof.qualify(&mut changed, &op).is_none(),
        "fresh recipient rejects changed disk authority"
    );
}

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
