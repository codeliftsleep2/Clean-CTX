//! Empty discovery is proven per owner; directory-dependent additions still refuse reuse.
use super::support::{fixture, resolver, write};
use crate::ir::typescript_resolution::shared_acquisition::operation;

#[test]
fn ops_r1_empty_automatic_types_share_without_losing_directory_witnesses() {
    let root = fixture();
    let path = root.path();
    std::fs::remove_file(path.join("a.ts")).unwrap();
    std::fs::remove_file(path.join("b.ts")).unwrap();
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
    write(path, "one/a.ts", "class A { peer?: Z; }");
    write(path, "two/b.ts", "class Z {}");
    let op = operation();
    let mut first = resolver(path, "one/a.ts", &op);
    assert_eq!(
        first.program_units(&path.join("one/a.ts")).unwrap().len(),
        2
    );
    let mut second = resolver(path, "two/b.ts", &op);
    assert_eq!(
        second.program_units(&path.join("two/b.ts")).unwrap().len(),
        2
    );
    assert!(first.host.current());
    assert!(second.host.current());
    assert_eq!(
        op.counts(),
        [1, 1, 2],
        "empty per-owner type discovery must not reacquire the whole program"
    );

    write(
        path,
        "two/node_modules/@types/local/index.d.ts",
        "declare class Local {}",
    );
    assert!(
        !second.host.current(),
        "the recipient must retain its own negative directory lookup"
    );
    let mut changed = resolver(path, "two/b.ts", &op);
    assert_eq!(
        changed.program_units(&path.join("two/b.ts")).unwrap().len(),
        3
    );
    assert_eq!(
        op.counts(),
        [2, 2, 5],
        "nonempty automatic types stay directory-specific"
    );
    write(path, "one/a.ts", "class ChangedA {}");
    let mut changed_source = resolver(path, "one/a.ts", &op);
    assert_eq!(
        changed_source
            .program_units(&path.join("one/a.ts"))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        op.counts(),
        [3, 3, 7],
        "changed project source refuses the old proof"
    );
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"strict":true},"include":["**/*.ts"]}"#,
    );
    let mut changed_config = resolver(path, "one/a.ts", &op);
    assert_eq!(
        changed_config
            .program_units(&path.join("one/a.ts"))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        op.counts(),
        [4, 4, 9],
        "changed compiler options remain a different basis"
    );
}
