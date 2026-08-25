// src/tests/edit/spans.rs
//
// Regression coverage for the reported `apply_edit` failure on LF-only
// files: `replace_body` rejecting a byte-exact `expectedOldText` with
// "expected N bytes, actual M bytes".
//
// Invariant under test (docs/plans/APPLY_EDIT_PLAN.md):
//   `CoreOp::Body.start_byte..end_byte` must address EXACTLY the bytes
//   of `text` in the source being edited — regardless of LF or CRLF.
//
// These tests drive the REAL production compiler (IRCompiler, Edit
// fidelity) over whole-file fixtures, then exercise both flows an agent
// can use to obtain `expectedOldText`:
//   A. copying the unit text delivered by the IR itself (`rec.text`), and
//   B. extracting the body byte-for-byte from the source independently,
//      the way an agent with editor access would.

use crate::compression::Fidelity;
use crate::compression::language::detect_language;
use crate::edit::apply;
use crate::edit::locate::UnitTable;
use crate::edit::ops::EditOperation;
use crate::ir::compiler::{CompiledIR, IRCompiler};
use crate::ir::opcodes::CoreOp;

fn compile_edit(source: &str, file_id: &str) -> CompiledIR {
    let (language, query) = detect_language(source);
    let mut compiler = IRCompiler::new();
    compiler
        .compile(source, file_id, language, query, Fidelity::Edit, None)
        .expect("compilation should succeed")
}

/// Collect `(mid, text, start, end)` for every spanned Body op.
fn spanned_bodies(ir: &CompiledIR) -> Vec<(String, String, u64, u64)> {
    ir.instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::Body(mid, text, Some(s), Some(e)) => Some((mid.clone(), text.clone(), *s, *e)),
            _ => None,
        })
        .collect()
}

/// THE INVARIANT: every spanned Body op must satisfy
/// `text == &source[start..end]`, byte for byte.
fn assert_span_invariant(source: &str, ir: &CompiledIR) {
    let bodies = spanned_bodies(ir);
    assert!(
        !bodies.is_empty(),
        "Edit-fidelity compile produced no spanned bodies"
    );
    for (mid, text, start, end) in &bodies {
        let (s, e) = (*start as usize, *end as usize);
        assert!(
            e <= source.len() && s <= e,
            "`{mid}` span {s}..{e} out of bounds (file {} bytes)",
            source.len()
        );
        let disk_slice = &source[s..e];
        let first_diff = text
            .bytes()
            .zip(disk_slice.bytes())
            .position(|(a, b)| a != b);
        assert_eq!(
            text.as_bytes(),
            disk_slice.as_bytes(),
            "`{mid}`: Body.text ({}) bytes != source[span] ({}) bytes; span {s}..{e}; first diff at byte {first_diff:?}",
            text.len(),
            disk_slice.len(),
        );
    }
}

/// Build the unit table and run one replace_body whose expectedOldText is
/// taken directly from the tracked record (flow A: copy tool output).
fn replace_with_record_text(source: &str, target: &str) -> Result<String, String> {
    let ir = compile_edit(source, "spans_test");
    let units = UnitTable::from_instructions(&ir.instructions);
    let old = units
        .resolve(target)
        .map_err(|e| format!("resolve failed: {e}"))?
        .text
        .clone();
    let report = apply::apply(
        source,
        &units,
        &[EditOperation::ReplaceBody {
            target: target.to_string(),
            expected_old_text: old,
            new_text: "{\n    REPLACED;\n  }".to_string(),
        }],
    )
    .map_err(|e| format!("apply rejected record-text edit: {e}"))?;
    Ok(report.new_source)
}

// ── Fixtures ─────────────────────────────────────────────────────────

/// Realistic LF-only TypeScript service: multi-line signature (colon and
/// arrow before the brace), blank lines between methods, nested control
/// flow inside the body.
fn ts_lf_source() -> String {
    [
        "import { Injectable } from '@angular/core';",
        "",
        "@Injectable()",
        "export class OrderService {",
        "  private label = 'orders';",
        "",
        "  async processOrder(",
        "    order: string,",
        "    retries: number,",
        "  ): Promise<string> {",
        "    const trimmed = order.trim();",
        "    if (retries > 0 && trimmed.length === 0) {",
        "      return this.label;",
        "    }",
        "    return trimmed;",
        "  }",
        "",
        "  count(): number {",
        "    return 1;",
        "  }",
        "}",
        "",
    ]
    .join("\n")
}

/// Same file with CRLF endings — the invariant must hold for both.
fn ts_crlf_source() -> String {
    ts_lf_source().replace('\n', "\r\n")
}

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn lf_file_body_spans_address_exact_disk_bytes() {
    let source = ts_lf_source();
    assert!(
        !source.contains('\r'),
        "fixture must be LF-only for this regression"
    );
    let ir = compile_edit(&source, "lf_spans");
    assert_span_invariant(&source, &ir);

    // Flow B: an agent extracts the processOrder body straight from the
    // file bytes (no reliance on our rendering) and submits it verbatim.
    let body_start = source.find("{\n    const trimmed").expect("body opener");
    let body_end = source.find("\n  }\n\n  count()").expect("body closer") + "\n  }".len();
    let extracted = source[body_start..body_end].to_string();

    let ir2 = compile_edit(&source, "lf_spans_flowB");
    let units = UnitTable::from_instructions(&ir2.instructions);
    let report = apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: extracted,
            new_text: "{\n    return 'ok';\n  }".to_string(),
        }],
    )
    .unwrap_or_else(|e| panic!("byte-exact LF edit was rejected: {e}"));
    assert!(report.new_source.contains("return 'ok';"));

    // Flow A: copying the exact text the session delivered must also work.
    let new_source =
        replace_with_record_text(&source, "OrderService.processOrder").expect("flow A");
    assert!(new_source.contains("REPLACED;"));
}

#[test]
fn crlf_file_body_spans_address_exact_disk_bytes() {
    let source = ts_crlf_source();
    assert!(source.contains("\r\n"), "fixture must be CRLF");
    let ir = compile_edit(&source, "crlf_spans");
    assert_span_invariant(&source, &ir);

    let body_start = source.find("{\r\n    const trimmed").expect("body opener");
    let body_end = source
        .find("\r\n  }\r\n\r\n  count()")
        .expect("body closer")
        + "\r\n  }".len();
    let extracted = source[body_start..body_end].to_string();

    let ir2 = compile_edit(&source, "crlf_spans_flowB");
    let units = UnitTable::from_instructions(&ir2.instructions);
    apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: extracted,
            new_text: "{\r\n    return 'ok';\r\n  }".to_string(),
        }],
    )
    .unwrap_or_else(|e| panic!("byte-exact CRLF edit was rejected: {e}"));
}

/// C# shape: leading attributes (prefix stripping) + Allman braces (`{`
/// alone on its own line). LF-only. Pins the brace-delimited body
/// contract: the tracked unit starts AT `{` with no surrounding
/// whitespace, so a natural agent extraction is accepted byte-exactly.
#[test]
fn lf_csharp_allman_attributes_spans_address_exact_disk_bytes() {
    let source = [
        "using System.Threading.Tasks;",
        "",
        "namespace Orders {",
        "  public class OrderService {",
        "    [HttpGet]",
        "    public async Task<string> ProcessOrder(int id, string mode)",
        "    {",
        "        var trimmed = id.ToString();",
        "        if (mode == \"strict\") {",
        "            return trimmed;",
        "        }",
        "        return await Task.FromResult(trimmed);",
        "    }",
        "",
        "    public int Count()",
        "    {",
        "        return 1;",
        "    }",
        "  }",
        "}",
        "",
    ]
    .join("\n");
    let ir = compile_edit(&source, "cs_lf_spans");
    assert_span_invariant(&source, &ir);

    // ── Boundary-choice regression (the reported failure shape) ──────
    // An agent extracts "the method body" the natural way: opening
    // brace through closing brace, NO surrounding indentation. The
    // tracked unit must accept that copy — i.e. unit boundaries must be
    // the brace-delimited body itself, not a wider whitespace-padded
    // line-range.
    let sig = source.find("ProcessOrder").expect("signature");
    let open = sig + source[sig..].find('{').expect("opening brace");
    let close_line = source
        .find("\n    }\n\n    public int Count")
        .expect("closer");
    let close = close_line + "\n    }".len(); // index just past `}`
    let natural = source[open..close].to_string();
    assert!(
        natural.starts_with('{') && natural.ends_with('}'),
        "natural extraction must be brace-delimited"
    );

    let ir2 = compile_edit(&source, "cs_lf_spans_natural");
    let units = UnitTable::from_instructions(&ir2.instructions);
    apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.ProcessOrder".to_string(),
            expected_old_text: natural.clone(),
            new_text: "{\n        return \"ok\";\n    }".to_string(),
        }],
    )
    .unwrap_or_else(|e| {
        panic!(
            "byte-exact brace-to-brace body was rejected — \
             unit boundaries are not brace-delimited: {e}"
        )
    });

    // Post-fix contract: the tracked unit IS the brace-delimited body —
    // text and span start at `{`, end at the capture's `}`, with no
    // surrounding whitespace in either direction.
    let rec = units.resolve("OrderService.ProcessOrder").unwrap();
    assert_eq!(
        rec.start_byte as usize, open,
        "span must start AT the brace"
    );
    assert_eq!(rec.end_byte as usize, close);
    assert_eq!(rec.text, natural, "tracked body must be brace-delimited");
}

/// Multi-byte content (emoji, accents, box-drawing) both BEFORE the
/// method (shifting absolute offsets) and INSIDE the body. If any layer
/// confuses char indices with byte indices, `text == source[span]`
/// breaks here while ASCII-only files stay green.
#[test]
fn lf_multibyte_content_spans_address_exact_disk_bytes() {
    let source = [
        "import { Injectable } from '@angular/core';",
        "",
        "// Résumé: orders ← pipeline ✓",
        "const BANNER = 'café ☕ — prêt ✓';",
        "",
        "@Injectable()",
        "export class OrderService {",
        "  async processOrder(order: string): Promise<string> {",
        "    // état: « en cours » → terminé ✓",
        "    const label = '✔ vérifié — café ☕';",
        "    return order.trim() || label;",
        "  }",
        "}",
        "",
    ]
    .join("\n");
    assert!(!source.contains('\r'));
    let ir = compile_edit(&source, "mb_spans");
    assert_span_invariant(&source, &ir);

    let body_start = source.find("{\n    // état").expect("body opener");
    let body_end = source.find("\n  }\n}").expect("body closer") + "\n  }".len();
    let extracted = source[body_start..body_end].to_string();

    let ir2 = compile_edit(&source, "mb_spans_flowB");
    let units = UnitTable::from_instructions(&ir2.instructions);
    apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: extracted,
            new_text: "{\n    return 'ok ✓';\n  }".to_string(),
        }],
    )
    .unwrap_or_else(|e| panic!("byte-exact multibyte LF edit was rejected: {e}"));
}

/// Expression-bodied members and brace-bearing string literals in the
/// signature region must not derail `find_body_start_in` / span math.
#[test]
fn lf_expression_bodies_and_braces_in_strings_hold_invariant() {
    let source = [
        "export class Calc {",
        "  answer: number;",
        "",
        "  get total(): number { return this.answer; }",
        "",
        "  greet(name = 'wor}d'): string {",
        "    const tmpl = `{hi(${name})}`;",
        "    return tmpl;",
        "  }",
        "}",
        "",
    ]
    .join("\n");
    let ir = compile_edit(&source, "expr_spans");
    assert_span_invariant(&source, &ir);
}

// ── Line-ending transport regression ──────────────────────────────────
//
// The reported residual: on CRLF files, a content-identical body copy
// whose separators were collapsed to LF in transport (editors /
// clipboards / LLM clients normalize) is rejected with
// `actual == expected + number_of_newlines` — indistinguishable from
// "counting each separator as 2 bytes instead of 1".
//
// Contract under test:
//   1. Verification compares content MODULO EOL width.
//   2. Bytes written to disk always follow the FILE's existing EOL
//      convention (incoming text is adapted; endings are never
//      rewritten as a side effect, never mixed).

/// CRLF file + LF-normalized copy → accepted, file stays uniformly CRLF.
#[test]
fn crlf_file_accepts_lf_normalized_copy_and_preserves_crlf_on_disk() {
    let source = ts_crlf_source();
    let body_start = source.find("{\r\n    const trimmed").expect("body opener");
    let body_end = source
        .find("\r\n  }\r\n\r\n  count()")
        .expect("body closer")
        + "\r\n  }".len();
    let exact = source[body_start..body_end].to_string();

    // Simulate transport normalization: every CRLF collapses to LF.
    let lf_only = exact.replace("\r\n", "\n");
    assert_ne!(lf_only, exact, "fixture sanity: normalization must bite");

    let ir = compile_edit(&source, "crlf_transport");
    let units = UnitTable::from_instructions(&ir.instructions);
    let report = apply::apply(
        &source,
        &units,
        &[EditOperation::ReplaceBody {
            target: "OrderService.processOrder".to_string(),
            expected_old_text: lf_only,
            new_text: "{\n    return 'ok';\n  }".to_string(),
        }],
    )
    .expect("LF-normalized copy of a CRLF unit must be accepted");

    let out = &report.new_source;
    assert!(out.contains("return 'ok';"));
    // Uniformly CRLF: every LF is half of a CRLF pair — no bare LF, no
    // mixed endings introduced by the splice.
    assert_eq!(
        out.matches("\r\n").count(),
        out.matches('\n').count(),
        "written file must not contain bare LF outside CRLF pairs"
    );

    // Outcome accounting must reflect the ADAPTED (CRLF) replacement
    // width, not the caller's LF form: adapted replacement is
    // "{\r\n    return 'ok';\r\n  }".
    let adapted_new = "{\r\n    return 'ok';\r\n  }";
    let expected_delta = adapted_new.len() as i64 - exact.len() as i64;
    assert_eq!(report.operations[0].byte_delta, expected_delta);
}

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

/// ── Diagnostic probe (CI-inert) ─────────────────────────────────────
///
/// Reconciles the exact byte counts on a REAL reported file. No-op
/// unless `CLEAN_CTX_SPANS_PROBE` is set:
///
/// ```powershell
/// $env:CLEAN_CTX_SPANS_PROBE = "C:\path\to\reported.ts"
/// cargo test edit::spans_tests::probe_real_file -- --nocapture
/// Remove-Item Env:CLEAN_CTX_SPANS_PROBE
/// ```
///
/// Prints, per spanned body: span window + width, stored-text length in
/// BYTES and CHARS, first-divergence byte offset with context, `\r`
/// census, BOM presence — everything needed to explain an
/// "expected N bytes, actual M bytes" rejection.
#[test]
fn probe_real_file() {
    let Ok(path) = std::env::var("CLEAN_CTX_SPANS_PROBE") else {
        eprintln!("probe: CLEAN_CTX_SPANS_PROBE not set — skipping (CI-inert)");
        return;
    };
    let bytes = std::fs::read(&path).expect("probe: read file");
    let source = String::from_utf8(bytes.clone()).expect("probe: file is UTF-8");

    let crlf_pairs = bytes.windows(2).filter(|w| w == b"\r\n").count();
    let cr_total = bytes.iter().filter(|&&b| b == b'\r').count();
    let non_ascii = bytes.iter().filter(|&&b| b >= 0x80).count();
    println!("== FILE {path}");
    println!(
        "bytes={} chars={} crlf={} lone_cr={} non_ascii_bytes={} bom={}",
        source.len(),
        source.chars().count(),
        crlf_pairs,
        cr_total - crlf_pairs,
        non_ascii,
        bytes.starts_with(&[0xEF, 0xBB, 0xBF]),
    );

    // mid -> method name for readable output.
    let mut names = std::collections::HashMap::new();
    for op in &compile_edit(&source, "probe_names").instructions {
        if let CoreOp::DefMethod(_, mid, name) = op {
            names.insert(mid.clone(), name.clone());
        }
    }

    let ir = compile_edit(&source, "probe");
    let bodies = spanned_bodies(&ir);
    println!("== {} spanned body/bodies", bodies.len());
    for (mid, text, start, end) in bodies {
        let label = names.get(&mid).map(String::as_str).unwrap_or("?");
        let s = start as usize;
        let e = (end as usize).min(source.len());
        let disk = &source[s..e];
        let first_diff = text.bytes().zip(disk.bytes()).position(|(a, b)| a != b);
        println!(
            "-- {mid} ({label}): span {s}..{e} width={} | text {} bytes / {} chars | slice \\r={} | first_diff={first_diff:?}",
            e - s,
            text.len(),
            text.chars().count(),
            disk.bytes().filter(|&b| b == b'\r').count(),
        );
        if let Some(d) = first_diff {
            let lo = d.saturating_sub(24);
            println!("   text@{d}: {:?}", &text[lo..(d + 24).min(text.len())]);
            println!("   disk@{d}: {:?}", &disk[lo..(d + 24).min(disk.len())]);
        }
        // Boundary classification: does the tracked body start with
        // whitespace before the opening brace (line-start backup), or
        // trail whitespace after the closing brace?
        match (text.find('{'), text.rfind('}')) {
            (Some(o), Some(c)) => {
                let lead = &text[..o];
                let trail = &text[c + 1..];
                println!(
                    "   boundaries: leading_before_\"{{\"={} bytes ({:?}) | trailing_after_\"}}\"={} bytes ({:?})",
                    lead.len(),
                    if lead.trim().is_empty() {
                        lead.to_string()
                    } else {
                        "<non-ws>".to_string()
                    },
                    trail.len(),
                    if trail.trim().is_empty() {
                        trail.to_string()
                    } else {
                        "<non-ws>".to_string()
                    },
                );
            }
            _ => println!("   boundaries: no braces found in body text"),
        }
        assert_eq!(
            text.as_bytes(),
            disk.as_bytes(),
            "`{mid}` ({label}) violates the span invariant on the real file"
        );
    }
    println!("== INVARIANT HOLDS on this file");
}
