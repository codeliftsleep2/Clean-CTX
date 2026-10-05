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
