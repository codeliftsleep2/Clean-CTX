// src/tests/dictionary/path.rs
//
// Tests for PathDictionary alias management and footer formatting.

use crate::dictionary::PathDictionary;
use crate::dictionary::path::canonical_identity_key;

/// format_footer_for_aliases with known aliases includes only those aliases.
#[test]
fn footer_for_aliases_includes_requested_aliases_only() {
    let mut dict = PathDictionary::new();
    let a1 = dict.get_or_create_alias("/file/a.ts".to_string());
    let a2 = dict.get_or_create_alias("/file/b.ts".to_string());
    let _a3 = dict.get_or_create_alias("/file/c.ts".to_string());

    // Request only the middle alias.
    let footer = dict.format_footer_for_aliases(&[&a2]);
    assert!(footer.contains("§PATHMAP"), "footer must have header");
    assert!(
        footer.contains(&a2),
        "footer must contain requested alias {a2}"
    );
    assert!(
        !footer.contains(&a1),
        "footer must NOT contain unrequested alias {a1}"
    );
}

/// format_footer_for_aliases with the file's own alias works for single-file use.
#[test]
fn footer_for_aliases_single_alias() {
    let mut dict = PathDictionary::new();
    let alias = dict.get_or_create_alias("/workspace/src/service.ts".to_string());

    let footer = dict.format_footer_for_aliases(&[&alias]);
    assert!(footer.contains("§PATHMAP"), "footer must have header");
    assert!(
        footer.contains(&alias),
        "footer must contain the requested alias {alias}"
    );
    assert!(
        footer.contains(&alias),
        "footer must contain the path for the requested alias"
    );
}

/// format_footer still returns the full session dictionary.
#[test]
fn full_footer_retains_all_aliases() {
    let mut dict = PathDictionary::new();
    let a1 = dict.get_or_create_alias("/file/a.ts".to_string());
    let a2 = dict.get_or_create_alias("/file/b.ts".to_string());

    let footer = dict.format_footer();
    assert!(footer.contains("§PATHMAP"), "footer must have header");
    assert!(footer.contains(&a1), "full footer must contain {a1}");
    assert!(footer.contains(&a2), "full footer must contain {a2}");
}

/// format_footer_for_aliases omits unknown alias IDs gracefully (no crash).
#[test]
fn footer_for_aliases_unknown_alias_is_omitted() {
    let mut dict = PathDictionary::new();
    let _real = dict.get_or_create_alias("/file/real.ts".to_string());

    let footer = dict.format_footer_for_aliases(&["α999"]);
    // The header is emitted, but no alias lines match.
    assert!(footer.contains("§PATHMAP"), "footer must have header");
    // Should contain the real alias but not α999
    assert!(
        !footer.contains("α999"),
        "footer must not contain unknown alias"
    );
}

/// format_footer_for_aliases with empty requested list emits only the header.
#[test]
fn footer_for_aliases_empty_request_only_header() {
    let mut dict = PathDictionary::new();
    let _a1 = dict.get_or_create_alias("/file/any.ts".to_string());

    let footer = dict.format_footer_for_aliases(&[]);
    assert!(footer.contains("§PATHMAP"), "footer must have header");
    // No '=' sign means no alias lines.
    assert!(
        !footer.contains('='),
        "no alias lines when nothing requested"
    );
}

/// Simulates the exact production scenario: prior session aliases do NOT leak.
#[test]
fn prior_session_aliases_do_not_leak_into_scoped_footer() {
    let mut dict = PathDictionary::new();

    // Simulate prior request for a.ts.
    let _alias_a = dict.get_or_create_alias("/file/a.ts".to_string());

    // New request for b.ts — only b.ts's alias should appear.
    let alias_b = dict.get_or_create_alias("/file/b.ts".to_string());

    let footer = dict.format_footer_for_aliases(&[&alias_b]);
    assert!(
        footer.contains(&alias_b),
        "footer must contain current alias"
    );
    assert!(
        !footer.contains("α1"),
        "footer must NOT contain a.ts's alias α1"
    );
}

#[test]
fn canonical_identity_collapses_resolvable_dot_and_dotdot_aliases_only() {
    let root = tempfile::tempdir().expect("identity root");
    let nested = root.path().join("nested");
    std::fs::create_dir(&nested).expect("nested directory");
    let owner = root.path().join("Owner.ts");
    std::fs::write(&owner, "export class Owner {}\n").expect("owner source");

    let canonical = canonical_identity_key(&owner.to_string_lossy());
    let dotted = canonical_identity_key(&root.path().join(".").join("Owner.ts").to_string_lossy());
    let dotdot = canonical_identity_key(&nested.join("..").join("Owner.ts").to_string_lossy());
    let distinct = root.path().join("Distinct.ts");
    std::fs::write(&distinct, "export class Distinct {}\n").expect("distinct source");

    assert_eq!(dotted, canonical);
    assert_eq!(dotdot, canonical);
    assert_ne!(
        canonical_identity_key(&distinct.to_string_lossy()),
        canonical,
        "textually similar routes must not collapse distinct physical owners"
    );
}

#[cfg(unix)]
#[test]
fn canonical_identity_uses_supported_physical_symlink_identity() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("identity root");
    let owner = root.path().join("Owner.ts");
    let alias = root.path().join("OwnerAlias.ts");
    std::fs::write(&owner, "export class Owner {}\n").expect("owner source");
    symlink(&owner, &alias).expect("source symlink");

    assert_eq!(
        canonical_identity_key(&alias.to_string_lossy()),
        canonical_identity_key(&owner.to_string_lossy())
    );
}

#[cfg(not(windows))]
#[test]
fn canonical_identity_preserves_case_distinct_physical_owners() {
    let root = tempfile::tempdir().expect("identity root");
    let upper = root.path().join("Owner.ts");
    let lower = root.path().join("owner.ts");
    std::fs::write(&upper, "export class Upper {}\n").expect("upper owner");
    std::fs::write(&lower, "export class Lower {}\n").expect("lower owner");

    assert_ne!(
        canonical_identity_key(&upper.to_string_lossy()),
        canonical_identity_key(&lower.to_string_lossy())
    );
}

#[cfg(windows)]
#[test]
fn canonical_identity_collapses_windows_case_aliases() {
    let root = tempfile::tempdir().expect("identity root");
    let owner = root.path().join("Owner.ts");
    std::fs::write(&owner, "export class Owner {}\n").expect("owner source");
    let case_alias = root.path().join("owner.ts");

    assert_eq!(
        canonical_identity_key(&case_alias.to_string_lossy()),
        canonical_identity_key(&owner.to_string_lossy())
    );
}
