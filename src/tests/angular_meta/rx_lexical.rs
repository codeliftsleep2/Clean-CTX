use crate::angular_meta::rx::extract_rx_shape;
use crate::compression::Fidelity;

#[test]
fn rxjs_extraction_does_not_use_legacy_prefix_membership_scans() {
    let source = r#"
import { Observable, combineLatest, map } from 'rxjs';
const users$: Observable<User[]> = service.load().pipe(map(users => users));
const combined$ = combineLatest([users$, selected$]);
"#;
    crate::meta_util::reset_legacy_membership_call_count();

    let shape = extract_rx_shape(source, Fidelity::High).expect("RxJS shape");

    assert!(!shape.is_empty());
    assert_eq!(
        crate::meta_util::legacy_membership_call_count(),
        0,
        "RxJS extraction must query a reusable lexical index"
    );
}
