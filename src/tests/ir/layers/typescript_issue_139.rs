//! Issue #139 TypeScript generic-constraint/heritage regressions.
use crate::compression::Fidelity;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::layers::{LanguageLayer, LayerContext};
use crate::ir::opcodes::CoreOp;

fn operations(capture: &str, interface: bool) -> Vec<CoreOp> {
    let mut layer = TypeScriptLayer::new();
    let mut context = LayerContext::new(capture, Fidelity::Low);
    if interface {
        context.current_interface = Some("I1".into());
        layer.process_capture("interface.root", capture, &mut context)
    } else {
        context.current_class = Some("C1".into());
        layer.process_capture("class.root", capture, &mut context)
    }
}

#[test]
fn generic_constraints_are_not_heritage_or_members() {
    let class = operations(
        "class Box<T extends { name: string }> { name!: string; }",
        false,
    );
    assert!(
        class
            .iter()
            .all(|op| !matches!(op, CoreOp::Extends(..) | CoreOp::Implements(..))),
        "{class:?}"
    );
    let interface = operations(
        "interface Box<T extends { name: string }, U extends Wrapper<{ id: string }>> { name: string; }",
        true,
    );
    assert!(
        interface
            .iter()
            .all(|op| !matches!(op, CoreOp::InterfaceExtends(..))),
        "{interface:?}"
    );
}

#[test]
fn real_heritage_survives_beside_generic_constraints() {
    let class = operations(
        "class Box<T extends { name: string }> extends Base<T> implements Named, Wrapped<T> {}",
        false,
    );
    assert!(
        class
            .iter()
            .any(|op| matches!(op, CoreOp::Extends(_, name) if name == "Base<T>")),
        "{class:?}"
    );
    assert!(
        class
            .iter()
            .any(|op| matches!(op, CoreOp::Implements(_, name) if name == "Named")),
        "{class:?}"
    );
    assert!(
        class
            .iter()
            .any(|op| matches!(op, CoreOp::Implements(_, name) if name == "Wrapped<T>")),
        "{class:?}"
    );

    let interface = operations(
        "interface Box<T extends { name: string }> extends Base<T>, Named {}",
        true,
    );
    assert!(
        interface
            .iter()
            .any(|op| matches!(op, CoreOp::InterfaceExtends(_, name) if name == "Base<T>")),
        "{interface:?}"
    );
    assert!(
        interface
            .iter()
            .any(|op| matches!(op, CoreOp::InterfaceExtends(_, name) if name == "Named")),
        "{interface:?}"
    );
}
