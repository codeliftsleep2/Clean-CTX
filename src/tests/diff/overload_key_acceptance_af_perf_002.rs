//! Agent MaxHeadRoom: every equality field and FIFO occurrence ownership.
use super::*;

#[test]
fn afperf_overload_full_key_fifo() {
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
            assert_eq!(
                ExactKey::new(a).cmp(&ExactKey::new(b)) == Ordering::Equal,
                a == b,
                "complete Eq partition"
            );
            assert_eq!(
                ExactKey::new(a).cmp(&ExactKey::new(b)),
                ExactKey::new(b).cmp(&ExactKey::new(a)).reverse()
            );
        }
    }
    let other = values[2].clone();
    let current = vec![&base, &other, &base, &base];
    let mut lookup = ExactLookup::new(&current);
    assert_eq!(lookup.take(&base), Some(0));
    assert_eq!(lookup.take(&base), Some(2));
    assert_eq!(lookup.take(&base), Some(3));
    assert_eq!(lookup.take(&base), None);
    drop(lookup);
    let mut signatures = SignatureLookup::new(&current, &[true, false, true, true]);
    assert_eq!(signatures.take(&base.sig), Some(1));
    assert_eq!(signatures.take(&base.sig), None);
    eprintln!(
        "AF002_OVERLOAD_LAYOUT {}",
        serde_json::json!({"usize":size_of::<usize>(),
        "exact_key":size_of::<ExactKey<'_>>(),"signature_key":size_of::<SignatureKey<'_>>(),
        "queue_header":size_of::<VecDeque<usize>>(),"exact_map_header":size_of::<BTreeMap<ExactKey<'_>,VecDeque<usize>>>(),
        "scope":"type sizes only; B-tree nodes and allocator overhead unmeasured"})
    );
}
