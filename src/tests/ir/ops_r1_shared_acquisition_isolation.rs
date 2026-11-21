//! Configuration, ownership, concurrency and invalidation controls.
use super::support::{acquire, fixture, resolver, write};
use crate::ir::typescript_resolution::Resolver;
use crate::ir::typescript_resolution::shared_acquisition::{basis, operation};
use std::sync::Arc;

#[test]
fn ops_r1_shared_config_roots_and_route_changes_stay_independent() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let mut first = resolver(path, "a.ts", &op);
    acquire(&mut first, &a);
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"strict":true},"include":["*.ts"]}"#,
    );
    let mut second = resolver(path, "b.ts", &op);
    acquire(&mut second, &b);
    let second_project = second.project_identity(&b).unwrap();
    assert_eq!(second_project.options["strict"].value, true);
    let second_basis = basis(&second, &b, &second_project).unwrap();
    assert_eq!(op.counts(), [2, 2, 4]);
    let extra = tempfile::tempdir().unwrap();
    let mut third = Resolver::new(&[path.to_path_buf(), extra.path().to_path_buf()]).unwrap();
    third.operation = Some(Arc::clone(&op));
    third.acquisition_policy = "controls".into();
    third.host.witnesses.enable();
    third
        .host
        .seed(&a, &std::fs::read_to_string(&a).unwrap())
        .unwrap();
    acquire(&mut third, &a);
    assert_eq!(op.counts(), [3, 3, 6]);
    let dotted = path.join(".").join("b.ts");
    let mut fourth = resolver(path, "b.ts", &op);
    assert_eq!(fourth.host.normalized(&dotted), fourth.host.normalized(&b));
    assert_eq!(fourth.host.logical(&dotted), fourth.host.logical(&b));
    let fourth_project = fourth.project_identity(&dotted).unwrap();
    assert_eq!(
        basis(&fourth, &dotted, &fourth_project).unwrap(),
        second_basis
    );
    let qualified_before = op.work_counts()[1];
    acquire(&mut fourth, &dotted);
    assert_eq!(op.counts(), [3, 3, 6]);
    assert_eq!(op.work_counts()[1], qualified_before + 1);

    // A changed effective option establishes real authority divergence.
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"strict":false},"include":["*.ts"]}"#,
    );
    let mut changed = resolver(path, "b.ts", &op);
    let changed_project = changed.project_identity(&b).unwrap();
    assert_eq!(changed_project.config, second_project.config);
    assert_eq!(changed_project.options["strict"].value, false);
    assert_ne!(basis(&changed, &b, &changed_project).unwrap(), second_basis);
    let qualified_before = op.work_counts()[1];
    acquire(&mut changed, &b);
    assert_eq!(op.counts(), [4, 4, 8]);
    assert_eq!(op.work_counts()[1], qualified_before);
}

#[test]
fn ops_r1_shared_source_and_package_changes_invalidate_proof() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    write(
        path,
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"1.0.0","types":"index.d.ts"}"#,
    );
    write(
        path,
        "node_modules/pkg/index.d.ts",
        "export declare class Token {}",
    );
    write(
        path,
        "a.ts",
        "import {Token} from 'pkg'; export class A { token?: Token; }",
    );
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let mut first = resolver(path, "a.ts", &op);
    assert_eq!(first.program_units(&a).unwrap().len(), 3);
    write(
        path,
        "node_modules/pkg/package.json",
        r#"{"name":"pkg","version":"2.0.0","types":"index.d.ts"}"#,
    );
    let mut second = resolver(path, "b.ts", &op);
    assert_eq!(second.program_units(&b).unwrap().len(), 3);
    write(
        path,
        "a.ts",
        "import {Token} from 'pkg'; export class Changed { token?: Token; }",
    );
    let mut third = resolver(path, "a.ts", &op);
    assert_eq!(third.program_units(&a).unwrap().len(), 3);
    assert_eq!(op.counts(), [3, 3, 9]);
    assert!(!first.host.current());
}

#[test]
fn ops_r1_shared_request_directory_and_inherited_project_remain_authoritative() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    write(
        path,
        "sub/tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]},"files":["c.ts"]}"#,
    );
    write(path, "sub/c.ts", "class C {}");
    let a = path.join("a.ts");
    let c = path.join("sub/c.ts");
    let mut first = resolver(path, "sub/c.ts", &op);
    assert_eq!(first.program_units(&c).unwrap().len(), 1);
    let mut second = resolver(path, "sub/c.ts", &op);
    let inherited = second
        .in_source_project(&a, |resolver| resolver.program_units(&c))
        .unwrap();
    assert_eq!(inherited.len(), 2);
    assert_eq!(op.counts(), [2, 2, 3]);
    assert!(
        second
            .program_evidence
            .contains_key(&path.join("tsconfig.json"))
    );
}

#[test]
fn ops_r1_shared_multiple_projects_and_operations_do_not_share_authority() {
    let left = fixture();
    let right = fixture();
    let op = operation();
    let mut a = resolver(left.path(), "a.ts", &op);
    acquire(&mut a, &left.path().join("a.ts"));
    let mut b = resolver(right.path(), "a.ts", &op);
    acquire(&mut b, &right.path().join("a.ts"));
    assert_eq!(op.counts(), [2, 2, 4]);
    let next = operation();
    let mut c = resolver(left.path(), "b.ts", &next);
    acquire(&mut c, &left.path().join("b.ts"));
    assert_eq!(next.counts(), [1, 1, 2]);
}

#[test]
fn ops_r1_shared_same_config_different_automatic_type_lookup_stays_distinct() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true},"include":["**/*.ts"]}"#,
    );
    write(
        path,
        "node_modules/typescript/package.json",
        r#"{"name":"typescript","version":"5.9.3"}"#,
    );
    write(path, "sub/c.ts", "class C { local?: Local; }");
    write(
        path,
        "sub/node_modules/@types/local/index.d.ts",
        "declare class Local {}",
    );
    let mut first = resolver(path, "a.ts", &op);
    assert_eq!(first.program_units(&path.join("a.ts")).unwrap().len(), 3);
    let first_project = first.request_project(&path.join("a.ts")).unwrap();
    let first_basis = basis(&first, &path.join("a.ts"), &first_project).unwrap();
    let mut second = resolver(path, "sub/c.ts", &op);
    let second_project = second.request_project(&path.join("sub/c.ts")).unwrap();
    assert_eq!(first_project.config, second_project.config);
    assert_ne!(
        first_basis,
        basis(&second, &path.join("sub/c.ts"), &second_project).unwrap(),
        "directory-dependent automatic types require distinct acquisition bases"
    );
    assert_eq!(
        second.program_units(&path.join("sub/c.ts")).unwrap().len(),
        4
    );
    assert_eq!(op.counts(), [2, 2, 7]);
}

#[test]
fn ops_r1_shared_concurrent_owners_and_nested_acquisition_are_isolated() {
    let root = fixture();
    let path = root.path();
    let op = operation();
    let barrier = std::sync::Barrier::new(2);
    std::thread::scope(|scope| {
        let handles: Vec<_> = ["a.ts", "b.ts"]
            .into_iter()
            .map(|owner| {
                let op = Arc::clone(&op);
                let barrier = &barrier;
                scope.spawn(move || {
                    let mut resolver = resolver(path, owner, &op);
                    barrier.wait();
                    acquire(&mut resolver, &path.join(owner));
                    assert!(resolver.host.current());
                })
            })
            .collect();
        for handle in handles {
            handle.join().unwrap();
        }
    });
    assert_eq!(op.counts(), [1, 1, 2]);
    let mut nested = resolver(path, "a.ts", &op);
    nested.program_active.insert(path.join("other-config.json"));
    acquire(&mut nested, &path.join("a.ts"));
    assert_eq!(
        op.counts(),
        [2, 2, 4],
        "nested acquisitions do not wait on cross-group locks"
    );
    let mut recursive = resolver(path, "a.ts", &op);
    recursive.program_active.insert(path.join("tsconfig.json"));
    assert_eq!(
        recursive.program_units(&path.join("a.ts")).unwrap_err(),
        "Recursive program acquisition"
    );
}
