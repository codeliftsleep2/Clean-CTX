use super::{CapturedFrame, ProducerStream};
use crate::diagnostics::cargo_check::{CargoCheckCompiler, CargoCheckPolicy, EvidenceCategory};

fn json(compiler: &mut CargoCheckCompiler, bytes: &[u8]) {
    compiler.observe_stdout_frame(bytes, true, false);
}

#[test]
fn mixed_frame_denominator_accounts_for_every_stdout_outcome() {
    let mut compiler = CargoCheckCompiler::default();
    for bytes in [
        b"\r \t".as_slice(),
        b"ordinary text",
        b"{broken",
        b"{\"reason\":\"future-record\"}",
        b"{\"reason\":\"build-finished\",\"success\":true}",
        b"{\"reason\":\"build-finished\",\"success\":null}",
        b"\xff",
    ] {
        json(&mut compiler, bytes);
    }
    compiler.observe_stdout_frame(
        b"{\"reason\":\"build-finished\",\"success\":false}",
        false,
        false,
    );
    let result = compiler.finish();
    let coverage = result.parser_coverage;
    assert_eq!(coverage.stdout_frames, 8);
    assert_eq!(
        coverage.stdout_frames,
        coverage.empty_stdout_frames
            + coverage.non_json_stdout
            + coverage.malformed_json
            + coverage.parsed_json_objects
            + coverage.unparsed_stdout_frames
    );
    assert_eq!(
        coverage.json_candidates,
        coverage.parsed_json_objects + coverage.malformed_json
    );
    assert_eq!(coverage.unknown_structured, 1);
    assert_eq!(coverage.incompatible_structured, 1);
    assert_eq!(coverage.invalid_utf8_frames, 1);
    assert_eq!(coverage.truncated_frames, 1);
    assert_eq!(result.cargo_evidence.build_finished_success, Some(true));
}

#[test]
fn ambiguous_fields_nested_in_arrays_are_counted_and_never_compiled() {
    let mut compiler = CargoCheckCompiler::default();
    json(&mut compiler, b"{\"reason\":\"compiler-message\",\"message\":{\"message\":\"failure\",\"level\":\"error\",\"children\":[{\"message\":\"first\",\"message\":\"second\",\"level\":\"help\"}]}}");
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.duplicate_json_fields, 1);
    assert_eq!(result.parser_coverage.incompatible_structured, 1);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn two_json_objects_in_one_frame_are_one_malformed_candidate() {
    let mut compiler = CargoCheckCompiler::default();
    json(&mut compiler, b"{\"reason\":\"build-finished\",\"success\":true} {\"reason\":\"build-finished\",\"success\":false}");
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.malformed_json, 1);
    assert_eq!(result.cargo_evidence.build_finished_success, None);
}

#[test]
fn stderr_json_never_contributes_cargo_structured_facts() {
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_stderr_frame(
        b"{\"reason\":\"build-finished\",\"success\":true}",
        true,
        false,
    );
    compiler.observe_stderr_frame(b"\r\t", true, false);
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.stderr_frames, 2);
    assert_eq!(result.parser_coverage.empty_stderr_frames, 1);
    assert_eq!(result.parser_coverage.json_candidates, 0);
    assert_eq!(result.cargo_evidence.build_finished_success, None);
}

#[test]
fn unavailable_decoding_is_distinct_from_actual_invalid_utf8() {
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_captured_frame(CapturedFrame {
        stream: ProducerStream::Stdout,
        bytes: &[0xff],
        observed_bytes: 100,
        terminated: true,
        over_limit: false,
        admission_cut: true,
        decoding_available: false,
    });
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.decoding_unavailable_frames, 1);
    assert_eq!(result.parser_coverage.invalid_utf8_frames, 0);
    assert_eq!(result.parser_coverage.truncated_frames, 0);
    assert_eq!(result.parser_coverage.admission_cut_frames, 1);
    assert_eq!(result.parser_coverage.unparsed_stdout_bytes, 100);
    let evidence = &result.evidence_facts.categories[&EvidenceCategory::MalformedOrTruncated];
    assert_eq!(evidence.original_bytes, 100);
    assert_eq!(evidence.omitted_bytes, 100);
}

#[test]
fn declared_complete_prefix_cannot_override_observed_length() {
    let mut compiler = CargoCheckCompiler::default();
    let bytes = b"{\"reason\":\"build-finished\",\"success\":true}";
    compiler.observe_captured_frame(CapturedFrame {
        stream: ProducerStream::Stdout,
        bytes,
        observed_bytes: bytes.len() as u64 + 10,
        terminated: true,
        over_limit: false,
        admission_cut: false,
        decoding_available: true,
    });
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.decoding_unavailable_frames, 1);
    assert_eq!(result.cargo_evidence.build_finished_success, None);
}

#[test]
fn invalid_terminal_sample_is_separate_from_physical_frame_decoding_counts() {
    let mut compiler = CargoCheckCompiler::default();
    compiler.observe_stderr_terminal_sample(b"private-canary-\xff");
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.stderr_frames, 0);
    assert_eq!(result.parser_coverage.stderr_terminal_samples, 1);
    assert_eq!(result.parser_coverage.invalid_utf8_frames, 0);
    assert_eq!(result.parser_coverage.invalid_utf8_terminal_samples, 1);
    assert!(
        !serde_json::to_string(&result)
            .unwrap()
            .contains("private-canary")
    );
}

#[test]
fn malformed_field_shapes_are_not_silently_coerced_into_semantic_values() {
    let mut compiler = CargoCheckCompiler::default();
    for bytes in [
        b"{\"reason\":\"compiler-message\",\"message\":{\"message\":\"failure\",\"level\":\"error\",\"code\":true}}".as_slice(),
        b"{\"reason\":\"compiler-message\",\"message\":{\"message\":\"failure\",\"level\":\"error\",\"children\":[null]}}",
        b"{\"reason\":\"compiler-artifact\",\"filenames\":[true]}",
        b"{\"reason\":\"build-script-executed\",\"env\":[[\"key\"]]}",
    ] { json(&mut compiler, bytes); }
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.incompatible_structured, 4);
    assert_eq!(result.cargo_evidence.artifact_records, 0);
    assert_eq!(result.cargo_evidence.build_script_records, 0);
    assert!(result.diagnostics.is_empty());
}

#[test]
fn real_shaped_metadata_and_unknown_future_fields_remain_compatible() {
    let mut compiler = CargoCheckCompiler::default();
    json(&mut compiler, b"{\"reason\":\"compiler-artifact\",\"package_id\":\"path+file:///fixture#0.1.0\",\"target\":{},\"profile\":{},\"features\":[],\"filenames\":[\"target/out\"],\"executable\":null,\"fresh\":true,\"future\":17}");
    json(&mut compiler, b"{\"reason\":\"build-script-executed\",\"package_id\":\"fixture\",\"out_dir\":\"target/out\",\"linked_libs\":[],\"linked_paths\":[],\"cfgs\":[],\"env\":[[\"name\",\"value\"]]}");
    let result = compiler.finish();
    assert_eq!(result.cargo_evidence.artifact_records, 1);
    assert_eq!(result.cargo_evidence.build_script_records, 1);
    assert_eq!(result.parser_coverage.incompatible_structured, 0);
}

#[test]
fn serde_recursion_limit_fails_closed_without_changing_the_policy() {
    let mut compiler = CargoCheckCompiler::default();
    let mut nested = String::from("{\"reason\":\"future-record\",\"nested\":");
    nested.push_str(&"[".repeat(256));
    nested.push('0');
    nested.push_str(&"]".repeat(256));
    nested.push('}');
    json(&mut compiler, nested.as_bytes());
    let result = compiler.finish();
    assert_eq!(result.parser_coverage.malformed_json, 1);
    assert_eq!(result.parser_coverage.unknown_structured, 0);
    assert!(result.evidence_facts.retained_bytes <= CargoCheckPolicy::APPROVED.evidence_bytes);
}
