// Additional line-ending transport regressions split from `spans.rs` to keep
// the active test modules within the repository file-size ceiling.

use super::{EditOperation, UnitTable, apply, compile_edit, ts_crlf_source, ts_lf_source};

/// Reverse direction: LF file + CRLF-padded copy → accepted, file stays
/// uniformly LF.
#[test]
fn lf_file_accepts_crlf_padded_copy_and_preserves_lf_on_disk() {
    let source = ts_lf_source();
    let body_start = source.find("{\n    const trimmed").expect("body opener");
    let body_end = source.find("\n  }\n\n  count()").expect("body closer") + "\n  }".len();
    let exact = source[body_start..body_end].to_string();

    let crlf_padded = exact.replace('\n', "\r\n");
    assert_ne!(crlf_padded, exact);

    let ir = compile_edit(&source, "lf_transport");
    let units = UnitTable::from_instructions(&ir.instructions);
    let report = apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: crlf_padded,
            new_text: "{\r\n    return 'ok';\r\n  }".to_string(),
        }],
    )
    .expect("CRLF-padded copy of an LF unit must be accepted");

    let out = &report.new_source;
    assert!(out.contains("return 'ok';"));
    assert!(
        !out.contains("\r\n"),
        "written LF file must not gain CRLF pairs"
    );
    let adapted_new = "{\n    return 'ok';\n  }";
    let expected_delta = adapted_new.len() as i64 - exact.len() as i64;
    assert_eq!(report.operations[0].byte_delta, expected_delta);
}

/// EOL insensitivity must not weaken concurrency semantics: a genuine
/// CONTENT change (beyond line-ending width) is still rejected.
#[test]
fn content_changes_are_still_rejected_regardless_of_eol() {
    let source = ts_crlf_source();
    let body_start = source.find("{\r\n    const trimmed").expect("body opener");
    let body_end = source
        .find("\r\n  }\r\n\r\n  count()")
        .expect("body closer")
        + "\r\n  }".len();
    let mut stale = source[body_start..body_end].to_string();
    stale = stale.replace("return trimmed;", "return FORGED;");
    let stale_lf = stale.replace("\r\n", "\n");

    let ir = compile_edit(&source, "crlf_guard");
    let units = UnitTable::from_instructions(&ir.instructions);
    let err = apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: stale_lf,
            new_text: "{\n    return 'x';\n  }".to_string(),
        }],
    )
    .expect_err("forged content must still be rejected");
    assert!(
        matches!(err, crate::edit::apply::EditError::Mismatch { .. }),
        "expected Mismatch, got {err:?}"
    );
}
