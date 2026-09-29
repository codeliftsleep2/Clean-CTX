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

// Canonical project ownership for memory/disk graph-query cache entries and
// target-project invalidation across active/non-active project boundaries.
#[path = "../tests/cbm/cache_ownership.rs"]
mod cache_ownership;

// CBM 0.8.1 trace_path wire contract (typed graph_trace parsing +
// direction determination), pinned by verbatim live captures and
// fresh-process probes over a synthetic fixture repo.
#[path = "../tests/cbm/trace_wire.rs"]
mod trace_wire;

// Duplicate bare-name trace resolution contract shared by the structured
// wrapper and the preferred proxy path.
#[cfg(all(test, feature = "rust"))]
#[path = "../tests/cbm/trace_identity_resolution.rs"]
mod trace_identity_resolution;
// CBM handler MCP contract tests (structuredContent, outputSchema conformance).
// Gated behind `feature = "rust"` because these tests share the global
// protocol::CAPTURED_RESPONSES sink with the Phase A/B retirement suites
// (which are also gated behind `feature = "rust"`), and all consumers of
// that sink must hold protocol::HANDLER_RESPONSE_SERIAL to prevent
// parallel-test races on the shared response queue.
#[cfg(all(test, feature = "rust"))]
#[path = "../tests/cbm/handlers.rs"]
mod handlers;

#[cfg(all(test, feature = "rust"))]
#[path = "../tests/cbm/handler_unavailable.rs"]
mod handler_unavailable;

// Raw-proxy failures must remain failures rather than compressed partial data.
#[path = "../tests/cbm/proxy_errors.rs"]
mod proxy_errors;

// CBM 0.8.1 query_graph wire contract (typed graph_query edge extraction,
// strict positional [from, type, to] convention), pinned by verbatim live
// captures and fresh-process probes over a synthetic fixture repo.
#[path = "../tests/cbm/query_wire.rs"]
mod query_wire;
