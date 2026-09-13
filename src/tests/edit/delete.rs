use crate::compression::Fidelity;
use crate::compression::language::language_for_extension;
use crate::edit::apply;
use crate::edit::locate::UnitTable;
use crate::edit::ops::EditOperation;
use crate::ir::compiler::IRCompiler;

fn units(source: &str, extension: &str) -> UnitTable {
    let (language, query) = language_for_extension(extension).expect("language enabled");
    let mut compiler = IRCompiler::new();
    let ir = compiler
        .compile(
            source,
            "delete_regression",
            language,
            query,
            Fidelity::Edit,
            None,
        )
        .expect("compile fixture");
    let (language, query) = language_for_extension(extension).expect("language enabled");
    UnitTable::from_instructions_with_declarations(&ir.instructions, source, language, query)
        .expect("map declaration spans")
}

fn delete(source: &str, extension: &str, target: &str) -> String {
    let units = units(source, extension);
    let expected = units.resolve(target).expect("resolve target").text.clone();
    let report = apply::apply(
        source,
        &units,
        &[EditOperation::Delete {
            target: target.to_string(),
            expected_old_text: expected,
        }],
    )
    .expect("delete splice");
    apply::verify_syntax(&report.new_source, extension).expect("post-delete syntax");
    report.new_source
}

#[test]
fn red_d1_csharp_final_attributed_method_deletes_whole_declaration() {
    let source = "using System;\nclass S {\n  public void First() {}\n  [Obsolete]\n  public async System.Threading.Tasks.Task Last() {}\n}\n";
    let output = delete(source, "cs", "S.Last");
    assert_eq!(
        output,
        "using System;\nclass S {\n  public void First() {}\n  \n}\n"
    );
}

#[test]
fn red_d2_csharp_first_and_middle_controls_delete_whole_declarations() {
    let source = "class S {\n  void First() {}\n  void Middle() {}\n  void Last() {}\n}\n";
    let first = delete(source, "cs", "S.First");
    assert!(!first.contains("void First"));
    assert!(first.contains("void Middle") && first.contains("void Last"));

    let middle = delete(source, "cs", "S.Middle");
    assert!(!middle.contains("void Middle"));
    assert!(middle.contains("void First") && middle.contains("void Last"));
}

#[test]
fn red_d3_and_d4_final_and_middle_members_work_in_other_brace_languages() {
    let cases = [
        (
            "ts",
            "class S {\n  first() {}\n  middle() {}\n  last() {}\n}\n",
            "middle",
            "last",
            "middle()",
            "last()",
        ),
        (
            "java",
            "class S {\n  void first() {}\n  void middle() {}\n  void last() {}\n}\n",
            "middle",
            "last",
            "middle()",
            "last()",
        ),
        (
            "rs",
            "struct S;\nimpl S {\n  fn first() {}\n  fn middle() {}\n  fn last() {}\n}\n",
            "middle",
            "last",
            "middle()",
            "last()",
        ),
    ];
    for (extension, source, middle_target, final_target, middle_text, final_text) in cases {
        let middle = delete(source, extension, middle_target);
        assert!(
            !middle.contains(middle_text),
            "{extension} middle member remained"
        );
        let final_output = delete(source, extension, final_target);
        assert!(
            !final_output.contains(final_text),
            "{extension} final member remained"
        );
    }
}

#[test]
fn red_d5_leading_metadata_is_owned_by_the_deleted_declaration() {
    let cases = [
        (
            "cs",
            "using System;\nclass S {\n  [Obsolete]\n  void Last() {}\n}\n",
            "Last",
            "Obsolete",
        ),
        (
            "ts",
            "declare const dec: MethodDecorator;\nclass S {\n  @dec\n  last() {}\n}\n",
            "last",
            "@dec",
        ),
        (
            "java",
            "class S {\n  @Deprecated\n  void last() {}\n}\n",
            "last",
            "Deprecated",
        ),
        (
            "rs",
            "struct S;\nimpl S {\n  #[deprecated]\n  fn last() {}\n}\n",
            "last",
            "deprecated",
        ),
    ];
    for (extension, source, target, metadata) in cases {
        let output = delete(source, extension, target);
        assert!(!output.contains(metadata), "{extension} metadata remained");
    }
}

#[test]
fn red_d6_and_d7_final_unit_handles_tight_and_blank_line_boundaries() {
    let tight = "class S {\n  void First() {}\n  void Last() {}\n}\n";
    let blank = "class S {\n  void First() {}\n  void Last() {}\n\n}\n";
    assert!(!delete(tight, "cs", "S.Last").contains("void Last"));
    assert!(!delete(blank, "cs", "S.Last").contains("void Last"));
}

#[test]
fn red_d8_replace_body_still_uses_the_body_span_in_every_language() {
    let cases = [
        (
            "cs",
            "class S { void Run() { return; } }",
            "Run",
            "{ return; }",
        ),
        ("ts", "class S { run() { return; } }", "run", "{ return; }"),
        (
            "java",
            "class S { void run() { return; } }",
            "run",
            "{ return; }",
        ),
        (
            "rs",
            "struct S; impl S { fn run() { return; } }",
            "run",
            "{ return; }",
        ),
    ];
    for (extension, source, target, replacement) in cases {
        let units = units(source, extension);
        let record = units.resolve(target).expect("resolve target");
        let report = apply::apply(
            source,
            &units,
            &[EditOperation::ReplaceBody {
                target: target.to_string(),
                expected_old_text: record.text.clone(),
                new_text: replacement.to_string(),
            }],
        )
        .expect("replace body");
        assert!(
            report
                .new_source
                .contains(target.rsplit('.').next().unwrap())
        );
        apply::verify_syntax(&report.new_source, extension).expect("replacement syntax");
    }
}

#[test]
fn red_d9_batched_final_delete_uses_the_same_declaration_range() {
    let source = "class S {\n  int First() { return 1; }\n  void Last() {}\n}\n";
    let units = units(source, "cs");
    let first = units.resolve("S.First").unwrap().text.clone();
    let last = units.resolve("S.Last").unwrap().text.clone();
    let report = apply::apply(
        source,
        &units,
        &[
            EditOperation::ReplaceBody {
                target: "S.First".to_string(),
                expected_old_text: first,
                new_text: "{ return 2; }".to_string(),
            },
            EditOperation::Delete {
                target: "S.Last".to_string(),
                expected_old_text: last,
            },
        ],
    )
    .expect("batch apply");
    assert!(report.new_source.contains("return 2"));
    assert!(!report.new_source.contains("void Last"));
    apply::verify_syntax(&report.new_source, "cs").expect("batch syntax");
}

#[test]
fn delete_does_not_treat_empty_expected_text_as_a_stale_check_bypass() {
    let source = "class S { void Last() {} }";
    let units = units(source, "cs");
    let error = apply::apply(
        source,
        &units,
        &[EditOperation::Delete {
            target: "S.Last".to_string(),
            expected_old_text: String::new(),
        }],
    )
    .expect_err("empty expected text must not bypass verification");
    assert!(matches!(error, apply::EditError::Mismatch { .. }));
}
