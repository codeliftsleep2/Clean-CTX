//! Session-owned coordination for equivalent hydration attempts.
//!
//! A flight reports only that one attempt is active or has ended. It stores no
//! candidates, proofs, completion result, or semantic authority. Followers
//! always re-enter the existing hydration path after the leader retires.

use crate::mcp::acquisition::AcquisitionTarget;
use crate::mcp::discovery_cache::DiscoveryMode;
use crate::workspace::index::SemanticFidelity;
use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct HydrationFlightKey {
    roots: Vec<String>,
    discovery: DiscoveryMode,
    query_name: String,
    semantic_fidelity: Option<SemanticFidelity>,
    proof_target: Option<AcquisitionTarget>,
}

impl HydrationFlightKey {
    pub(crate) fn new(
        roots: Vec<String>,
        discovery: DiscoveryMode,
        query_name: impl Into<String>,
        semantic_fidelity: Option<SemanticFidelity>,
        proof_target: Option<AcquisitionTarget>,
    ) -> Self {
        Self {
            roots,
            discovery,
            query_name: query_name.into(),
            semantic_fidelity,
            proof_target,
        }
    }
}

#[derive(Default)]
struct HydrationFlight {
    active: Mutex<bool>,
    ended: Condvar,
}

#[derive(Default)]
pub(crate) struct HydrationFlightRegistry {
    active: Mutex<HashMap<HydrationFlightKey, Arc<HydrationFlight>>>,
    #[cfg(test)]
    test: FlightTestSupport,
}

enum Claim<'a> {
    Leader(HydrationFlightLeader<'a>),
    Follower(Arc<HydrationFlight>),
}

struct HydrationFlightLeader<'a> {
    registry: &'a HydrationFlightRegistry,
    key: HydrationFlightKey,
    flight: Arc<HydrationFlight>,
}

impl HydrationFlightRegistry {
    fn claim(&self, key: &HydrationFlightKey) -> Claim<'_> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(flight) = active.get(key) {
            return Claim::Follower(Arc::clone(flight));
        }
        let flight = Arc::new(HydrationFlight {
            active: Mutex::new(true),
            ended: Condvar::new(),
        });
        active.insert(key.clone(), Arc::clone(&flight));
        Claim::Leader(HydrationFlightLeader {
            registry: self,
            key: key.clone(),
            flight,
        })
    }

    /// Run one hydration attempt after coordinating equivalent active work.
    ///
    /// A follower waits only for the attempt to end, then loops through
    /// `claim` and invokes `attempt` itself. That invocation rechecks the
    /// existing discovery/acquisition/publication authorities; no leader result
    /// crosses this boundary.
    pub(crate) fn run<T>(&self, key: HydrationFlightKey, mut attempt: impl FnMut() -> T) -> T {
        loop {
            match self.claim(&key) {
                Claim::Leader(leader) => {
                    #[cfg(test)]
                    self.test.leader_claimed();
                    let result = attempt();
                    drop(leader);
                    return result;
                }
                Claim::Follower(flight) => {
                    #[cfg(test)]
                    self.test.follower_claimed();
                    let mut active = flight
                        .active
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    while *active {
                        active = flight
                            .ended
                            .wait(active)
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                    }
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn test_arm(&self) {
        self.test.arm();
    }

    #[cfg(test)]
    pub(crate) fn test_wait_for_leader(&self) {
        self.test.wait_for_leader();
    }

    #[cfg(test)]
    pub(crate) fn test_wait_for_followers(&self, count: usize) {
        self.test.wait_for_followers(count);
    }

    #[cfg(test)]
    pub(crate) fn test_release(&self) {
        self.test.release();
    }
}

impl Drop for HydrationFlightLeader<'_> {
    fn drop(&mut self) {
        {
            let mut registry = self
                .registry
                .active
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if registry
                .get(&self.key)
                .is_some_and(|current| Arc::ptr_eq(current, &self.flight))
            {
                registry.remove(&self.key);
            }
        }
        let mut active = self
            .flight
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *active = false;
        self.flight.ended.notify_all();
    }
}

#[cfg(test)]
#[derive(Default)]
struct FlightTestState {
    armed: bool,
    leader_claimed: bool,
    followers: usize,
    released: bool,
}

#[cfg(test)]
#[derive(Default)]
struct FlightTestSupport {
    state: Mutex<FlightTestState>,
    changed: Condvar,
}

#[cfg(test)]
impl FlightTestSupport {
    fn arm(&self) {
        *self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = FlightTestState {
            armed: true,
            ..FlightTestState::default()
        };
    }

    fn leader_claimed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.armed || state.leader_claimed {
            return;
        }
        state.leader_claimed = true;
        self.changed.notify_all();
        while !state.released {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    fn follower_claimed(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.armed {
            state.followers += 1;
            self.changed.notify_all();
        }
    }

    fn wait_for_leader(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while !state.leader_claimed {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    fn wait_for_followers(&self, count: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while state.followers < count {
            state = self
                .changed
                .wait(state)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    fn release(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.released = true;
        self.changed.notify_all();
    }
}
