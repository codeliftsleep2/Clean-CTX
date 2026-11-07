//! Agent MaxHeadRoom: exact original UnitTable and ordered refusal oracle.
use super::*;
#[path = "original_end_join_af_perf_002.rs"]
pub(super) mod original;

fn compare(instructions: &[CoreOp], source: &str, extension: &str, duplicate_query: bool) {
    let (language, query) =
        crate::compression::language::language_for_extension(extension).unwrap();
    let query = if duplicate_query {
        format!("{query}\n(method_definition) @method.root\n(method_definition) @method.root")
    } else {
        query.to_owned()
    };
    let old = original::UnitTable::from_instructions_with_declarations(
        instructions,
        source,
        language.clone(),
        &query,
    )
    .unwrap();
    let new =
        UnitTable::from_instructions_with_declarations(instructions, source, language, &query)
            .unwrap();
    assert_eq!(new.len(), old.len());
    assert_eq!(new.is_empty(), old.is_empty());
    assert_eq!(new.iter().count(), old.iter().count());
    assert_eq!(
        (format!("{:?}", new.units), format!("{:?}", new.uneditable)),
        original::frozen_records(&old),
        "complete UnitRecord and ordered UneditableRecord identity"
    );
    let mut targets = vec![
        "missing".to_owned(),
        "Subject".to_owned(),
        "Subject.f0".to_owned(),
        "m0".to_owned(),
        "Subject.m0".to_owned(),
    ];
    for unit in &new.units {
        targets.extend([
            unit.method_id.clone(),
            unit.qualified_name.clone(),
            unit.name.clone(),
            unit.selector(),
        ]);
    }
    for target in targets {
        assert_eq!(
            format!("{:?}", new.resolve(&target)),
            format!("{:?}", old.resolve(&target)),
            "exact ordered resolution/refusal: {target}"
        );
    }
}
#[test]
fn af002_edit_end_frozen_equivalence() {
    use crate::compression::{Fidelity, language::language_for_extension};
    let cases = [
        ("ts", ""),
        (
            "ts",
            "export class Subject { f0:number=0; m0() { return 1; } }",
        ),
        (
            "ts",
            "// café\r\nexport class Subject {\r\n  m0() { return 1; }\r\n}\r\n",
        ),
        (
            "ts",
            "export class Subject { // unowned comment\n m0() { return 1; } m1() { return 2; } }",
        ),
        (
            "ts",
            "class Subject { m0() { function nested() { return 1; } return nested(); } }",
        ),
        (
            "ts",
            "class Subject { m0() { return 1; } } class Other { m0() { return 2; } }",
        ),
        (
            "ts",
            "class Subject { m0(x: string): void; m0(x: number): void; m0(x: any) { return; } }",
        ),
        (
            "cs",
            "class Subject { [System.Obsolete] void m0() { } int f0; void m1() { } }",
        ),
        (
            "cs",
            "class Subject { // leading ownership gap\n void m0() { } }",
        ),
        (
            "java",
            "class Subject { @Deprecated void m0() { } int f0; void m1() { } }",
        ),
    ];
    for (i, (extension, source)) in cases.into_iter().enumerate() {
        let (language, query) = language_for_extension(extension).unwrap();
        let mut compiler = crate::ir::compiler::IRCompiler::new();
        let instructions = if source.is_empty() {
            Vec::new()
        } else {
            compiler
                .compile(source, "end-oracle", language, query, Fidelity::Edit, None)
                .unwrap()
                .instructions
        };
        compare(&instructions, source, extension, false);
        let mut reversed = instructions.clone();
        reversed.reverse();
        compare(&reversed, source, extension, false);
        if extension == "ts" {
            compare(&instructions, source, extension, true);
        }
        // Synthetic body spans exercise duplicate ends, nested/overlapping starts,
        // missing ends and zero ranges. They are helper cases, not admitted authority.
        let mut spans = instructions.clone();
        spans.push(CoreOp::DefClass("SYN".into(), "Synthetic".into()));
        let end = source.len() as u64;
        for (n, start, end) in [(0, 0, end), (1, end, end), (2, 0, 0), (3, 0, end)] {
            let mid = format!("SYN{n}");
            spans.push(CoreOp::DefMethod("SYN".into(), mid.clone(), "m0".into()));
            spans.push(CoreOp::Body(
                mid,
                source[start as usize..end as usize].to_owned(),
                Some(start),
                Some(end),
            ));
        }
        compare(&spans, source, extension, false);
        compare(&[], source, extension, false);
        eprintln!("AF002_EDIT_END_EQUIVALENCE case={i}");
    }
}
