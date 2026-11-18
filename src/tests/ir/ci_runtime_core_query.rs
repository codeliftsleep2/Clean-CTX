//! Proposed bounded-work witness; production query reuse is not yet authorized.
use crate::compression::Fidelity;
use crate::compression::capture_pipeline::{
    parse_count, query_compilation_count, query_cursor_count, reset_parse_count,
    reset_query_work_counts,
};
use crate::ir::pipeline::{CoreIRPass, IRPass, PassContext};

fn observe() -> PassContext {
    let mut context = PassContext::new(
        "export class Probe { run(): number { return target(); } }".into(),
        "probe.ts".into(),
        Fidelity::High,
    );
    context.language = crate::compression::language::safe_typescript_language();
    context.query_string = crate::queries::TS_QUERY.into();
    CoreIRPass::new().run(&mut context).unwrap();
    context
}

#[test]
fn ci_runtime_core_query_warm_observation_avoids_recompiling_binary_owned_query() {
    let first = observe();
    assert!(!first.captures.is_empty());
    reset_query_work_counts();
    reset_parse_count();
    let started = std::time::Instant::now();
    let second = observe();
    let seconds = started.elapsed().as_secs_f64();
    assert_eq!(
        format!("{:?}", first.captures),
        format!("{:?}", second.captures)
    );
    assert_eq!(
        format!("{:?}", first.instructions),
        format!("{:?}", second.instructions)
    );
    assert_eq!(parse_count(), 1, "each source still requires its own parse");
    assert_eq!(
        query_cursor_count(),
        1,
        "each observation requires its own cursor"
    );
    eprintln!(
        "CI_RUNTIME_CORE_QUERY seconds={seconds} parses={} cursors={} query_compilations={}",
        parse_count(),
        query_cursor_count(),
        query_compilation_count()
    );
    assert_eq!(
        query_compilation_count(),
        0,
        "proposed budget: unchanged binary-owned IR query must not be rebuilt for each source"
    );
}
