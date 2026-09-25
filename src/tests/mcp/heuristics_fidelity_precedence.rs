use super::*;

#[test]
fn stored_high_fidelity_precedes_auto_edit_classification() {
    let config = CleanCtxConfig::default();
    let context = ContextState::new();
    let source = r#"
export class WorkerService {
  async run(flag: boolean): Promise<number> {
    if (flag) { return 1; }
    return 0;
  }
}
"#;

    let decision = decide(
        "/project/worker.service.ts",
        None,
        None,
        &config,
        &context,
        source,
        None,
        Some(Fidelity::High),
    )
    .expect("stored fidelity decision");

    assert_eq!(decision.file_class, FileClass::Implementation);
    assert_eq!(
        decision.fidelity,
        Fidelity::High,
        "session fidelity must win over automatic Edit inference"
    );
}
