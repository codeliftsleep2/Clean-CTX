use crate::ir::compiler::CompiledIR;
use crate::ir::opcodes::{
    ControlSummary, CoreOp, DeclarationModifier, ExecutionContextKind, PatternFact, SideEffectKind,
};
pub(super) fn all_variants_with_duplicates() -> CompiledIR {
    let duplicate = CoreOp::SideEffect("M1".into(), SideEffectKind::Io);
    CompiledIR {
        file_id: "src/semantic.ts".into(),
        version: 42,
        instructions: vec![
            CoreOp::DefClass("C1".into(), "Owner".into()),
            CoreOp::DefClass("C2".into(), "UnresolvedOwner".into()),
            CoreOp::DefMethod("C1".into(), "M1".into(), "work".into()),
            CoreOp::DefMethod("C1".into(), "M2".into(), "expression".into()),
            CoreOp::DefField("C1".into(), "F1".into(), "value".into()),
            CoreOp::DefInterface("I1".into(), "Runnable".into()),
            CoreOp::DefInterfaceMethod("I1".into(), "M3".into(), "execute".into()),
            CoreOp::DefInterfaceField("I1".into(), "F2".into(), "version".into()),
            CoreOp::InterfaceModifiers("I1".into(), vec![DeclarationModifier::Export]),
            CoreOp::InterfaceExtends("I1".into(), "BaseInterface".into()),
            CoreOp::Param("M1".into(), "P1".into(), "$s".into(), "input".into()),
            CoreOp::Return("M1".into(), "$v".into()),
            CoreOp::FieldType("F1".into(), "$s".into()),
            CoreOp::MethodModifiers(
                "M1".into(),
                vec![DeclarationModifier::Static, DeclarationModifier::Async],
            ),
            CoreOp::ClassModifiers("C1".into(), vec![DeclarationModifier::Export]),
            CoreOp::ControlSummary(
                "M1".into(),
                vec![ControlSummary::Branch, ControlSummary::Return],
            ),
            CoreOp::PatternFacts(
                "M1".into(),
                vec![
                    PatternFact::Constructor,
                    PatternFact::Getter("value".into()),
                ],
            ),
            CoreOp::Flags("M1".into(), vec!["legacy".into(), "legacy".into()]),
            CoreOp::ClassFlags("C1".into(), vec!["metadata".into()]),
            CoreOp::Extends("C1".into(), "Base".into()),
            CoreOp::BaseTypeRef("C2".into(), "ExternalContract".into()),
            CoreOp::Implements("C1".into(), "I1".into()),
            CoreOp::Injects("C1".into(), vec!["Repo".into(), "Repo".into()]),
            CoreOp::Import("IM1".into(), "pkg".into(), "Thing".into()),
            CoreOp::TypeAlias("T1".into(), "Thing<string>".into()),
            CoreOp::TypeAlias("T1".into(), "Thing<string>".into()),
            CoreOp::Pattern(
                "GETTER".into(),
                vec!["C1".into(), "M1".into(), "value".into()],
            ),
            CoreOp::Body(
                "M1".into(),
                "{\r\n  return \"λ\";\r\n}".into(),
                Some(100),
                Some(124),
            ),
            CoreOp::Body("M2".into(), "expression".into(), None, None),
            CoreOp::DataFlow("M1".into(), "reads".into(), "value".into()),
            CoreOp::ControlFlow("M1".into(), "return".into(), "value".into()),
            duplicate.clone(),
            duplicate,
            CoreOp::ExecutionContext("M1".into(), ExecutionContextKind::ThreadBound),
            CoreOp::Call("M1".into(), "exact".into(), 2, false),
            CoreOp::Call("M1".into(), "spread".into(), 1, true),
        ],
    }
}
