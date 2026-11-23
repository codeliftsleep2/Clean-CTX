//! Concern-based registration; shared fixtures remain tracked in the test tree.
#[path = "ops_r1_shared_acquisition_cost.rs"]
mod cost;
#[path = "ops_r1_empty_automatic_types.rs"]
mod empty_automatic_types;
#[path = "ops_r1_shared_acquisition_isolation.rs"]
mod isolation;
#[path = "ops_r1_phase_validation.rs"]
mod phase_validation;
#[path = "ops_r1_shared_acquisition_proof.rs"]
mod proof;
#[path = "ops_r1_shared_acquisition_support.rs"]
mod support;
