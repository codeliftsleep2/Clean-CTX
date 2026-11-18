use std::sync::{Condvar, Mutex};

struct AttemptState {
    owner: Option<String>,
    attempted: bool,
}

static STATE: Mutex<AttemptState> = Mutex::new(AttemptState {
    owner: None,
    attempted: false,
});
static SIGNAL: Condvar = Condvar::new();

pub(super) fn arm(owner: &str) {
    *STATE.lock().expect("publication attempt state") = AttemptState {
        owner: Some(owner.to_string()),
        attempted: false,
    };
}

pub(super) fn publication_attempted(owner: &str) {
    let mut state = STATE.lock().expect("publication attempt state");
    if state.owner.as_deref() == Some(owner) {
        state.attempted = true;
        SIGNAL.notify_all();
    }
}

pub(super) fn wait_until_attempted() {
    let mut state = STATE.lock().expect("publication attempt state");
    while !state.attempted {
        state = SIGNAL.wait(state).expect("publication attempt wait");
    }
    state.owner = None;
}

/// Exercise the existing hooks without requiring a language producer. A failed
/// newer publication must not retire an otherwise-current older candidate.
#[test]
fn ci_runtime_publication_hooks_preserve_owner_order_and_failure() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let owner = root.path().join("owner.cs").to_string_lossy().into_owned();
    let sibling = root
        .path()
        .join("sibling.cs")
        .to_string_lossy()
        .into_owned();
    let state = crate::mcp::McpState::new(crate::tests::test_config());
    state.arm_semantic_publication_attempt(&owner);
    let older = state.begin_semantic_publication(&owner);
    state.wait_for_semantic_publication_attempt();
    let independent = state.begin_semantic_publication(&sibling);
    let newer = state.begin_semantic_publication(&owner);
    assert_eq!(
        newer.commit(|| Ok::<_, &str>(true), || Ok("newer")),
        Ok(Some("newer"))
    );
    assert_eq!(
        older.commit::<&str, &str>(
            || panic!("stale ticket must not check source"),
            || panic!("stale ticket must not publish")
        ),
        Ok(None)
    );
    assert_eq!(
        independent.commit(|| Ok::<_, &str>(true), || Ok("sibling")),
        Ok(Some("sibling"))
    );
    let not_current = state.begin_semantic_publication(&owner);
    assert_eq!(
        not_current.commit::<&str, &str>(
            || Ok(false),
            || panic!("noncurrent source must not publish")
        ),
        Ok(None)
    );
    let retained = state.begin_semantic_publication(&owner);
    let failing = state.begin_semantic_publication(&owner);
    assert_eq!(
        failing.commit::<&str, &str>(|| Ok(true), || Err("publication failed")),
        Err("publication failed")
    );
    assert_eq!(
        retained.commit(|| Ok::<_, &str>(true), || Ok("retained")),
        Ok(Some("retained"))
    );
}
