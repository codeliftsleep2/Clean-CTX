use std::sync::{Condvar, Mutex};

#[derive(Default)]
struct PauseState {
    path: Option<String>,
    paused: bool,
    released: bool,
}

static STATE: Mutex<PauseState> = Mutex::new(PauseState {
    path: None,
    paused: false,
    released: false,
});
static SIGNAL: Condvar = Condvar::new();

pub(crate) fn arm(path: &str) {
    *STATE.lock().expect("apply_edit pause state") = PauseState {
        path: Some(crate::dictionary::path::canonical_identity_key(path)),
        paused: false,
        released: false,
    };
}

pub(crate) fn pause_before_commit(path: &str) {
    let canonical = crate::dictionary::path::canonical_identity_key(path);
    let mut state = STATE.lock().expect("apply_edit pause state");
    if state.path.as_deref() != Some(canonical.as_str()) || state.paused {
        return;
    }
    state.paused = true;
    SIGNAL.notify_all();
    while !state.released {
        state = SIGNAL.wait(state).expect("apply_edit pause wait");
    }
    state.path = None;
}

pub(crate) fn wait_until_paused() {
    let mut state = STATE.lock().expect("apply_edit pause state");
    while !state.paused {
        state = SIGNAL.wait(state).expect("apply_edit pause wait");
    }
}

pub(crate) fn release() {
    let mut state = STATE.lock().expect("apply_edit pause state");
    state.released = true;
    SIGNAL.notify_all();
}
