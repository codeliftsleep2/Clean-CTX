//! A shared evaluation must distinguish typed facts, not merely source paths.
use super::support::{fixture, write};
use crate::ir::typescript_resolution::{TypeScriptAuthority, Witness};
use crate::workspace::index::{SemanticFidelity, WorkspaceIndex};

#[test]
fn ops_r1_consumption_batch_refuses_conflicting_config_hashes_and_missing_owners() {
    let root = fixture();
    let path = root.path();
    let a = path.join("a.ts");
    let b = path.join("b.ts");
    let a_source = "export class A {}";
    let b_source = "export class B {}";
    write(path, "a.ts", a_source);
    write(path, "b.ts", b_source);
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"strict":false},"include":["*.ts"]}"#,
    );
    let b_proof = TypeScriptAuthority::capture(&b, b_source, &[path.to_path_buf()]);
    assert!(b_proof.capture_current && b_proof.disposition.is_none());
    let old_config = b_proof
        .witnesses
        .iter()
        .find_map(|witness| match witness {
            Witness::Content { path, hash }
                if path.file_name() == Some(std::ffi::OsStr::new("tsconfig.json")) =>
            {
                Some((path.clone(), hash.clone()))
            }
            _ => None,
        })
        .expect("old config is consumed evidence");
    let b_hash = b_proof.source_hash.clone();
    let mut index = WorkspaceIndex::new();
    index
        .replace_semantic_projection_with_retained_authority(
            b.to_str().unwrap(),
            vec![],
            SemanticFidelity::High,
            b_hash.clone(),
            vec![],
            Some(b_proof),
        )
        .unwrap();
    write(
        path,
        "tsconfig.json",
        r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","noLib":true,"types":[],"strict":true},"include":["*.ts"]}"#,
    );
    let a_proof = TypeScriptAuthority::capture(&a, a_source, &[path.to_path_buf()]);
    assert!(a_proof.capture_current && a_proof.disposition.is_none());
    let current_config = a_proof
        .witnesses
        .iter()
        .find_map(|witness| match witness {
            Witness::Content { path, hash }
                if path.file_name() == Some(std::ffi::OsStr::new("tsconfig.json")) =>
            {
                Some((path.clone(), hash.clone()))
            }
            _ => None,
        })
        .expect("current config is consumed evidence");
    assert_eq!(old_config.0, current_config.0);
    assert_ne!(old_config.1, current_config.1);
    let a_hash = a_proof.source_hash.clone();
    index
        .replace_semantic_projection_with_retained_authority(
            a.to_str().unwrap(),
            vec![],
            SemanticFidelity::High,
            a_hash.clone(),
            vec![],
            Some(a_proof),
        )
        .unwrap();
    let a_request = (a.to_str().unwrap(), SemanticFidelity::High, a_hash.as_str());
    let b_request = (b.to_str().unwrap(), SemanticFidelity::High, b_hash.as_str());
    assert!(index.has_current_semantic_projections(&[a_request]));
    assert!(!index.has_current_semantic_projections(&[b_request]));
    assert!(
        !index.has_current_semantic_projections(&[a_request, b_request]),
        "fresh same-path config fact must not authorize the stale hash"
    );
    assert!(!index.has_current_semantic_projections(&[b_request, a_request]));
    assert!(!index.has_current_semantic_projections(&[(
        a.to_str().unwrap(),
        SemanticFidelity::High,
        "wrong-owner-hash"
    )]));
    let missing = path.join("absent.ts");
    assert!(!index.has_current_semantic_projections(&[(
        missing.to_str().unwrap(),
        SemanticFidelity::High,
        "unknown"
    )]));
    std::fs::remove_file(path.join("tsconfig.json")).unwrap();
    assert!(!index.has_current_semantic_projections(&[a_request]));
    assert_eq!(
        index.semantic_projection_fidelity(a.to_str().unwrap()),
        Some(SemanticFidelity::High),
        "failed currentness does not erase retained ownership"
    );
}
