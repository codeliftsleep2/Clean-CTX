// src/cbm/tests.rs
//
// CBM integration, E2E, and regression test suite.
// Loaded via #[cfg(test)] #[path = "tests.rs"] in src/cbm/mod.rs.

#[path = "../tests/cbm/regression.rs"]
mod regression;

#[path = "../tests/cbm/integration.rs"]
mod integration;

#[path = "../tests/cbm/e2e.rs"]
mod e2e;

// Graph-intelligence layer audit (symbol importance, dead code, blast
// radius, architecture, caching / project isolation).
#[path = "../tests/cbm/graph_intel.rs"]
mod graph_intel;
