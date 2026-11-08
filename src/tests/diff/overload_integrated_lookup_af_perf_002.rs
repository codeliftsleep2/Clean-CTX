//! Agent MaxHeadRoom: every equality field and FIFO occurrence ownership.
use super::*;
use crate::diff::snapshot::DeclarationEvidence;

#[test]
fn afperf_overload_full_equality_lookup() {
    let base = CapturedMethod {
        sig: "Run(α)".into(),
        declaration: None,
        markers: vec![],
        body: None,
    };
    let mut values = vec![base.clone()];
    let mut changed = base.clone();
    changed.sig.push('β');
    values.push(changed);
    let mut changed = base.clone();
    changed.body = Some(String::new());
    values.push(changed);
    let mut changed = base.clone();
    changed.body = Some("共有".repeat(8192));
    values.push(changed);
    let mut changed = base.clone();
    changed.markers = vec!["a".into(), "b".into()];
    values.push(changed.clone());
    changed.markers.reverse();
    values.push(changed);
    for return_type in ["", "int"] {
        for attributes in [
            vec![],
            vec!["a".into()],
            vec!["a".into(), "a".into()],
            vec!["a".into(), "b".into()],
            vec!["b".into(), "a".into()],
        ] {
            for modifiers in [
                vec![],
                vec!["public".into()],
                vec!["a".into(), "b".into()],
                vec!["b".into(), "a".into()],
            ] {
                let mut changed = base.clone();
                changed.declaration = Some(DeclarationEvidence {
                    return_type: return_type.into(),
                    attributes: attributes.clone(),
                    modifiers,
                });
                values.push(changed);
            }
        }
    }
    for a in &values {
        for b in &values {
            assert_eq!(method_equal(a, b), a == b, "all derived equality fields");
        }
    }
    let other = values[2].clone();
    let current = vec![&base, &other, &base, &base];
    let mut used = [false; 4];
    let mut lookup = MethodLookup::new(&current);
    for expected in [0, 2, 3] {
        assert_eq!(lookup.exact(&base, &used), Some(expected));
        used[expected] = true;
    }
    assert_eq!(lookup.exact(&base, &used), None);
    assert_eq!(lookup.signature(&base.sig, &used), Some(1));
    used[1] = true;
    assert_eq!(lookup.signature(&base.sig, &used), None);
    let one = [&base];
    let mut lookup = MethodLookup::new(&one);
    assert!(lookup.index.is_none());
    assert_eq!(lookup.exact(&other, &[false]), None);
    assert_eq!(lookup.exact(&base, &[false]), Some(0));
    assert_eq!(lookup.signature(&base.sig, &[true]), None);
    eprintln!(
        "AF002_INTEGRATED_LAYOUT {}",
        serde_json::json!({"index":size_of::<usize>(),
        "lookup_header":size_of::<MethodLookup<'_, '_>>(),"signature_index_header":size_of::<SignatureIndex>(),
        "scope":"type headers/capacities only; not allocator-event or peak-memory accounting"})
    );
}
