use super::Framer;
use crate::diagnostics::cargo_check::{CargoCheckCompiler, CargoCheckPolicy, EvidenceCategory};

fn policy(stdout: usize, stderr: usize, frame: usize) -> CargoCheckPolicy {
    CargoCheckPolicy {
        stdout_capture_bytes: stdout,
        stderr_capture_bytes: stderr,
        aggregate_capture_bytes: stdout + stderr,
        maximum_frame_bytes: frame,
        ..CargoCheckPolicy::APPROVED
    }
}

#[test]
fn capture_counts_delimiters_but_frame_limit_excludes_them() {
    let mut compiler = CargoCheckCompiler::new(policy(100, 100, 4));
    let mut framer = Framer::new(true, policy(100, 100, 4));
    framer.observe(b"1234\n12345\n", &mut compiler);
    let facts = framer.finish(&mut compiler);
    assert_eq!(facts.observed_bytes, 11);
    assert_eq!(facts.admitted_bytes, 11);
    assert_eq!(facts.complete_frames, 2);
    assert_eq!(facts.over_limit_frames, 1);
    assert_eq!(facts.maximum_observed_frame_bytes, 5);
    let semantic = compiler.finish();
    assert_eq!(semantic.parser_coverage.non_json_stdout, 1);
    assert_eq!(semantic.parser_coverage.over_limit_frames, 1);
}

#[test]
fn capture_cut_record_is_not_compiled_as_complete_json() {
    let message = b"{\"reason\":\"build-finished\",\"success\":true}\n";
    let mut compiler = CargoCheckCompiler::default();
    let mut framer = Framer::new(true, policy(message.len() - 1, 100, 100));
    framer.observe(message, &mut compiler);
    let facts = framer.finish(&mut compiler);
    assert!(facts.capture_limit_activated);
    assert_eq!(facts.discarded_bytes, 1);
    assert_eq!(facts.admission_cut_frames, 1);
    assert_eq!(
        compiler.finish().cargo_evidence.build_finished_success,
        None
    );
}

#[test]
fn framing_survives_arbitrary_read_chunk_boundaries_and_unterminated_eof() {
    let mut compiler = CargoCheckCompiler::default();
    let mut framer = Framer::new(true, CargoCheckPolicy::APPROVED);
    for bytes in [
        b"{\"reason\":\"build-".as_slice(),
        b"finished\",\"success\":true}\n",
        b"tail",
    ] {
        framer.observe(bytes, &mut compiler);
    }
    let facts = framer.finish(&mut compiler);
    assert_eq!(facts.complete_frames, 1);
    assert_eq!(facts.unterminated_frames, 1);
    assert_eq!(
        compiler.finish().cargo_evidence.build_finished_success,
        Some(true)
    );
}

#[test]
fn oversized_frame_is_drained_before_next_record_without_unbounded_buffering() {
    let mut compiler = CargoCheckCompiler::default();
    let mut framer = Framer::new(true, policy(100_000, 10, 64));
    for _ in 0..1000 {
        framer.observe(&[b'x'; 64], &mut compiler);
    }
    assert_eq!(framer.frame.len(), 64);
    framer.observe(
        b"\n{\"reason\":\"build-finished\",\"success\":false}\n",
        &mut compiler,
    );
    let facts = framer.finish(&mut compiler);
    assert_eq!(facts.over_limit_frames, 1);
    assert_eq!(
        compiler.finish().cargo_evidence.build_finished_success,
        Some(false)
    );
}

#[test]
fn independent_stream_budgets_do_not_compete_and_late_stderr_cause_survives() {
    let policy = policy(5, 5, 100);
    let mut compiler = CargoCheckCompiler::default();
    let mut stdout = Framer::new(true, policy);
    let mut stderr = Framer::new(false, policy);
    stdout.observe(b"noise\nmore noise\n", &mut compiler);
    stderr.observe(b"head\nterminal cause\n", &mut compiler);
    let out = stdout.finish(&mut compiler);
    let err = stderr.finish(&mut compiler);
    assert_eq!(out.admitted_bytes, 5);
    assert_eq!(err.admitted_bytes, 5);
    assert_eq!(err.post_limit_stderr_frame_candidates, 1);
    let result = compiler.finish();
    assert!(
        result
            .evidence
            .iter()
            .any(|item| item.category == EvidenceCategory::Stderr
                && item.text.contains("terminal cause"))
    );
    assert!(result.evidence_facts.retained_bytes <= policy.evidence_bytes);
}

#[test]
fn partial_or_oversized_late_stderr_samples_are_withheld() {
    let mut policy = policy(2, 2, 8);
    policy.evidence_bytes = 4;
    let mut compiler = CargoCheckCompiler::default();
    let mut stderr = Framer::new(false, policy);
    stderr.observe(b"head\nvery long cause\npartial", &mut compiler);
    let facts = stderr.finish(&mut compiler);
    assert_eq!(facts.post_limit_stderr_frames_withheld, 2);
}

#[test]
fn incomplete_raw_evidence_is_withheld_and_late_complete_secret_is_redacted() {
    let mut compiler = CargoCheckCompiler::default();
    let mut stdout = Framer::new(true, policy(4, 4, 100));
    stdout.observe(b"PRIVATE_CANARY_WITHOUT_RECOGNIZED_PREFIX\n", &mut compiler);
    let mut stderr = Framer::new(false, policy(4, 4, 100));
    stderr.observe(b"head\nPASSWORD=late_secret_canary\n", &mut compiler);
    stdout.finish(&mut compiler);
    stderr.finish(&mut compiler);
    let serialized = serde_json::to_string(&compiler.finish()).unwrap();
    assert!(!serialized.contains("PRIVATE_CANARY"));
    assert!(!serialized.contains("late_secret_canary"));
    assert!(serialized.contains("REDACTED"));
}

#[test]
fn terminal_stderr_sample_stays_within_approved_shared_evidence_budget() {
    let mut compiler = CargoCheckCompiler::default();
    let large_head = [b'h'; 16 * 1024];
    for _ in 0..8 {
        compiler.observe_stderr_frame(&large_head, true, false);
    }
    let mut stderr = Framer::new(false, policy(1, 1, 128));
    stderr.observe(b"terminal failure\n", &mut compiler);
    stderr.finish(&mut compiler);
    let result = compiler.finish();
    let stderr_items: Vec<_> = result
        .evidence
        .iter()
        .filter(|item| item.category == EvidenceCategory::Stderr)
        .collect();
    assert!(
        stderr_items
            .last()
            .unwrap()
            .text
            .contains("terminal failure")
    );
    assert!(
        stderr_items
            .iter()
            .map(|item| item.text.len())
            .sum::<usize>()
            <= 48 * 1024
    );
    assert!(result.evidence_facts.retained_bytes <= CargoCheckPolicy::APPROVED.evidence_bytes);
}

#[test]
fn read_failure_does_not_expose_error_text_or_claim_eof() {
    struct BrokenReader;
    impl std::io::Read for BrokenReader {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("private error detail canary"))
        }
    }
    let compiler = std::sync::Mutex::new(CargoCheckCompiler::default());
    let stop = std::sync::atomic::AtomicBool::new(false);
    let facts = super::drain(BrokenReader, true, &compiler, &stop);
    assert!(facts.read_failed);
    assert!(!facts.eof_observed);
    assert!(!serde_json::to_string(&facts).unwrap().contains("canary"));
}

#[test]
fn stopped_reader_never_enters_read_and_discloses_missing_eof() {
    struct NeverRead;
    impl std::io::Read for NeverRead {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            panic!("cancelled capture must not read again");
        }
    }
    let compiler = std::sync::Mutex::new(CargoCheckCompiler::default());
    let stop = std::sync::atomic::AtomicBool::new(true);
    let facts = super::drain(NeverRead, false, &compiler, &stop);
    assert!(facts.stopped_before_eof);
    assert!(!facts.eof_observed);
    assert!(!facts.read_failed);
}
