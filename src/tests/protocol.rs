use std::sync::{Arc, Barrier};

#[test]
fn captured_responses_are_isolated_per_test_thread() {
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = [11, 22]
        .into_iter()
        .map(|id| {
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                crate::protocol::captured_responses().clear();
                crate::protocol::send_response(&serde_json::json!({ "id": id }));
                barrier.wait();
                crate::protocol::captured_responses()
                    .pop()
                    .expect("calling thread captures its own response")["id"]
                    .as_i64()
                    .expect("numeric response id")
            })
        })
        .collect();

    let captured: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("capture thread"))
        .collect();
    assert_eq!(captured, vec![11, 22]);
}
