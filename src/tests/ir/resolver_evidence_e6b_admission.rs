//! Exhaustion during later admission must not erase independently current claims.
use super::*;

#[test]
fn e6b_later_admission_exhaustion_preserves_earlier_current_source() {
    let root = fixture();
    write(root.path(), "b.ts", POSITIVE);
    let session = Session::new(vec![root.path().to_path_buf()], limits()).unwrap();
    let request = session.ledger();
    let competing = std::sync::Arc::new(std::sync::Mutex::new(None));
    let held = competing.clone();
    bounded::test_hooks::set_capture(move |_| {
        bounded::test_hooks::set_capture(move |_| {
            let free =
                request.limits().total.retained_bytes - request.snapshot().used.retained_bytes;
            *held.lock().unwrap() = Some(
                request
                    .reserve_nested(
                        Cost {
                            retained_bytes: free,
                            ..Cost::default()
                        },
                        true,
                    )
                    .unwrap(),
            );
        });
    });
    let mut q = query(root.path(), "users");
    q.source_files = Some(vec![PathBuf::from("app.ts"), PathBuf::from("b.ts")]);
    let out = session.execute(q).unwrap();
    let value = out.dto().value();
    assert_eq!(value["matching_records"]["returned"], 1);
    assert_eq!(
        value["coverage"]["supported_written_source"]["state"],
        "open"
    );
    assert_eq!(value["interpretations"][0]["classification"], "partial");
    assert_eq!(value["execution"]["state"], "exhausted");
    assert!(
        value["failures"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["detail"].as_str().unwrap().contains("b.ts admission"))
    );
    drop(competing);
}
