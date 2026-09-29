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

/// The process-global handler-response gate must be available in EVERY test
/// configuration, and a panic that held it must not poison its callers.
///
/// `protocol::handler_response_serial()` serializes the suites that coordinate
/// process-global test state while they drive dispatching handlers. Those
/// suites are registered under different gates (`apply_edit_tests` compiles in
/// every test build, `provider_code_context_signature_tests` needs
/// `feature = "csharp"`, the Phase A/B, contract and CBM handler suites need
/// `feature = "rust"`), so the gate itself must not be gated on any one of
/// them: gating it on `feature = "rust"` broke every test build that compiled
/// a consumer without that feature (E0425: `handler_response_serial` not found
/// in `crate::protocol`). This module is registered unconditionally, so
/// referencing the accessor here is what makes that availability structural —
/// narrowing the gate again fails the build in the cheapest configuration
/// rather than only in one language's.
#[test]
fn handler_response_serial_is_available_and_poison_tolerant() {
    // Sequential acquisition: the guard releases when it drops, so a second
    // holder gets it. (A leaked guard would deadlock every dispatching suite.)
    drop(crate::protocol::handler_response_serial());
    drop(crate::protocol::handler_response_serial());

    // A panicking holder poisons the mutex. The accessor must still hand out a
    // usable guard: one failing test must never cascade into unrelated suites
    // through a poisoned shared gate.
    let panicked = std::panic::catch_unwind(|| {
        let _held = crate::protocol::handler_response_serial();
        panic!("deliberate panic while holding the handler-response gate");
    });
    assert!(panicked.is_err(), "the deliberate panic must unwind");
    // The gate is now POISONED: this acquisition would panic in the default
    // `lock().unwrap()` shape and must succeed through the tolerant accessor.
    drop(crate::protocol::handler_response_serial());
}
