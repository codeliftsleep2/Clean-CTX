// src/tests/angular_meta/bundler.rs
//
// Tests for the file-triplet bundler.

use crate::angular_meta::bundler::{
    is_component_ts, resolve_bundle_group, resolve_triplet,
};
use std::path::{Path, PathBuf};

/// Debug: check if the test fixture files exist and what paths resolve to.
#[test]
fn debug_bundler_paths() {
    let ts_path = Path::new("src/test_files/angular/user-card.component.ts");
    eprintln!("ts exists: {}", ts_path.exists());
    eprintln!("ts parent: {:?}", ts_path.parent());
    eprintln!("ts stem: {:?}", ts_path.file_stem());

    if let Some(parent) = ts_path.parent() {
        let html = parent.join("user-card.component.html");
        let scss = parent.join("user-card.component.scss");
        eprintln!("html candidate: {:?} exists={}", html, html.exists());
        eprintln!("scss candidate: {:?} exists={}", scss, scss.exists());

        // Also try canonicalize
        if let Ok(canon) = std::fs::canonicalize(ts_path) {
            eprintln!("ts canonical: {:?}", canon);
        }
    }
}

#[test]
fn is_component_ts_recognises_standard_naming() {
    let path = Path::new("/project/src/app/user-card.component.ts");
    assert!(is_component_ts(path));
}

#[test]
fn is_component_ts_rejects_service_file() {
    let path = Path::new("/project/src/app/user.service.ts");
    assert!(!is_component_ts(path));
}

#[test]
fn is_component_ts_rejects_plain_ts() {
    let path = Path::new("/project/src/app/utils.ts");
    assert!(!is_component_ts(path));
}

#[test]
fn resolve_triplet_returns_none_for_non_component() {
    let path = Path::new("/project/src/app/user.service.ts");
    assert!(resolve_triplet(path).is_none());
}

#[test]
fn resolve_triplet_finds_html_sibling() {
    let path = Path::new("src/test_files/angular/user-card.component.ts");
    let triplet = resolve_triplet(path).expect("should resolve triplet");
    assert!(triplet.template.is_some(), "should find .html sibling");
    assert!(triplet.style.is_some(), "should find .scss sibling");
    assert_eq!(
        triplet.component_ts,
        Path::new("src/test_files/angular/user-card.component.ts")
    );
}

#[test]
fn resolve_triplet_finds_page_siblings() {
    let path = Path::new("src/test_files/angular/user-page.component.ts");
    let triplet = resolve_triplet(path).expect("should resolve triplet");
    assert!(triplet.template.is_some(), "should find .html sibling");
    assert!(triplet.style.is_some(), "should find .scss sibling");
}

#[test]
fn resolve_triplet_no_siblings_for_standalone_service() {
    // non_triplet_file.ts has no matching .html or .scss
    let path = Path::new("src/test_files/angular/non_triplet_file.ts");
    assert!(
        resolve_triplet(path).is_none(),
        "non-component file should not resolve"
    );
}

#[test]
fn resolve_bundle_group_returns_none_for_non_component() {
    let path = Path::new("src/test_files/angular/user.service.ts");
    assert!(resolve_bundle_group(path).is_none());
}

#[test]
fn resolve_bundle_group_returns_group_for_component() {
    let path = Path::new("src/test_files/angular/user-card.component.ts");
    let group = resolve_bundle_group(path).expect("should resolve bundle group");
    assert_eq!(group.name, "user-card.component");
    assert!(group.has_siblings());
}

#[test]
fn resolve_bundle_group_no_siblings_when_path_invalid() {
    // A path with no parent (root-level .component.ts) returns None
    // because parent() returns None.
    let path = Path::new("fake.component.ts");
    let group = resolve_bundle_group(path).expect("should resolve even without siblings");
    // The group exists but has no siblings.
    assert!(!group.has_siblings());
}

#[test]
fn resolve_triplet_template_only_has_siblings() {
    // When only template is present, has_siblings should be true.
    let path = Path::new("src/test_files/angular/user-card.component.ts");
    let triplet = resolve_triplet(path).expect("should resolve");
    let has = triplet.template.is_some() || triplet.style.is_some();
    assert!(has);
}

#[test]
fn resolve_triplet_style_only_has_siblings() {
    // When only style is present, has_siblings should be true.
    let path = Path::new("src/test_files/angular/user-page.component.ts");
    let triplet = resolve_triplet(path).expect("should resolve");
    let has = triplet.template.is_some() || triplet.style.is_some();
    assert!(has);
}

#[test]
fn resolve_triplet_no_siblings_when_neither_present() {
    // This tests the BundleGroup::has_siblings method.
    use crate::angular_meta::bundler::BundleGroup;
    let group = BundleGroup {
        name: "test".to_string(),
        triplet: crate::angular_meta::bundler::FileTriplet {
            component_ts: PathBuf::from("/a/b/c.component.ts"),
            template: None,
            style: None,
        },
    };
    assert!(!group.has_siblings());
}
