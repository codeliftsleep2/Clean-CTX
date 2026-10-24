use crate::ir::typescript_resolution::TypeScriptAuthority;
use crate::layers::meta::canonical_endpoint::CanonicalEndpoint;
use crate::layers::meta::semantic::{EntityRef, SemanticEdge, SemanticRelation};

#[test]
fn issue_162_member_identity_admission_refuses_forged_pairs_and_duplicate_evidence() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source =
        "function input(){return 0;} class Left { value=input(); } class Right { value=input(); }";
    std::fs::write(&file, source).unwrap();
    let authority = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    assert_eq!(authority.member_declarations.len(), 2);
    let left = authority
        .member_declarations
        .iter()
        .find(|pair| pair.owner.name == "Left")
        .unwrap();
    let right = authority
        .member_declarations
        .iter()
        .find(|pair| pair.owner.name == "Right")
        .unwrap();
    let mut subject = EntityRef::new("angular", "Component", &left.owner.name)
        .with_file(left.owner.source.to_string_lossy().into_owned());
    subject.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: (&left.owner).into(),
        site: Some((left.owner.start, left.owner.end)),
    });
    let mut object = EntityRef::new("angular", "InputField", &left.member.name)
        .with_file(left.member.source.to_string_lossy().into_owned());
    object.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: (&left.member).into(),
        site: Some(left.initializer),
    });
    let edge = SemanticEdge {
        relation: SemanticRelation::HasInput,
        subject,
        object,
        layer: "angular",
        call_evidence: None,
    };
    // Neutral pair admission does not assign Angular roles; production tests
    // separately establish API and owner interpretation.
    assert!(authority.admits_relation_evidence(std::slice::from_ref(&edge)));
    let mut mismatched = edge.clone();
    mismatched.object.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: (&right.member).into(),
        site: Some(right.initializer),
    });
    assert!(!authority.admits_relation_evidence(&[mismatched]));
    let mut bad_site = edge.clone();
    bad_site.object.canonical = Some(CanonicalEndpoint::Declaration {
        declaration: (&left.member).into(),
        site: Some((left.initializer.0 + 1, left.initializer.1)),
    });
    assert!(!authority.admits_relation_evidence(&[bad_site]));
    let mut duplicate = authority.clone();
    duplicate.member_declarations.push(left.clone());
    assert!(!duplicate.admits_relation_evidence(std::slice::from_ref(&edge)));
    let mut forged = authority.clone();
    forged
        .member_declarations
        .iter_mut()
        .find(|pair| pair.owner.name == "Left")
        .unwrap()
        .owner = right.owner.clone();
    assert!(!forged.admits_relation_evidence(std::slice::from_ref(&edge)));
    let mut missing = authority.clone();
    missing.member_declarations.clear();
    assert!(!missing.admits_relation_evidence(&[edge]));
}

#[test]
fn issue_162_member_identity_duplicate_fields_remain_refused() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("app.ts");
    let source = "function input(){return 0;} class Owner { value=input(); value=0; }";
    std::fs::write(&file, source).unwrap();
    let authority = TypeScriptAuthority::capture(&file, source, &[root.path().to_path_buf()]);
    let pair = authority
        .member_declarations
        .first()
        .expect("retained refused occurrence");
    assert!(!pair.unique);
    use crate::ir::invocation_evidence::SourceSpan;
    assert!(
        authority
            .member_at(
                SourceSpan::new(pair.member.start, pair.member.end).unwrap(),
                SourceSpan::new(pair.initializer.0, pair.initializer.1).unwrap()
            )
            .is_none()
    );
}
