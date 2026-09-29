use crate::compression::Fidelity;
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::layers::java::JavaLayer;
use crate::ir::layers::rust::RustLayer;
use crate::ir::layers::typescript::TypeScriptLayer;
use crate::ir::layers::{LanguageLayer, LayerContext};
use crate::ir::opcodes::{CoreOp, DeclarationModifier};

fn context(source: &str) -> LayerContext {
    LayerContext::new(source, Fidelity::Low)
}

fn projected_class_modifiers(ops: &[CoreOp]) -> Vec<Vec<DeclarationModifier>> {
    ops.iter()
        .filter_map(|op| match op {
            CoreOp::ClassModifiers(_, modifiers) => Some(modifiers.clone()),
            _ => None,
        })
        .collect()
}

fn projected_method_modifiers(ops: &[CoreOp]) -> Vec<Vec<DeclarationModifier>> {
    ops.iter()
        .filter_map(|op| match op {
            CoreOp::MethodModifiers(_, modifiers) => Some(modifiers.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn rust_modifiers_are_owned_by_the_declaration_head() {
    let class_source = r#"
pub struct Outer {
    pub async_value: bool, // unsafe trait
}
"#;
    let mut class_context = context(class_source);
    class_context.current_class = Some("C1".into());
    let class_ops =
        RustLayer::new().process_capture("struct.root", class_source, &mut class_context);
    assert_eq!(
        projected_class_modifiers(&class_ops),
        vec![vec![DeclarationModifier::Export]]
    );

    let method_source = r#"
pub fn work() {
    let async_value = true;
    unsafe { mutate(); }
}
"#;
    let mut method_context = context(method_source);
    method_context.current_method = Some("M1".into());
    let method_ops =
        RustLayer::new().process_capture("method.root", method_source, &mut method_context);
    assert_eq!(
        projected_method_modifiers(&method_ops),
        vec![vec![DeclarationModifier::Export]]
    );
}

#[test]
fn typescript_modifiers_are_owned_by_the_declaration_head() {
    let class_source = r#"
export class Outer {
    // export abstract class Descendant {}
}
"#;
    let mut class_context = context(class_source);
    class_context.current_class = Some("C1".into());
    let class_ops =
        TypeScriptLayer::new().process_capture("class.root", class_source, &mut class_context);
    assert_eq!(
        projected_class_modifiers(&class_ops),
        vec![vec![DeclarationModifier::Export]]
    );

    let method_source = r#"
protected work() {
    // private static async
}
"#;
    let mut method_context = context(method_source);
    method_context.current_method = Some("M1".into());
    let method_ops =
        TypeScriptLayer::new().process_capture("method.root", method_source, &mut method_context);
    assert_eq!(
        projected_method_modifiers(&method_ops),
        vec![vec![DeclarationModifier::Protected]]
    );
}

#[test]
fn java_modifiers_are_owned_by_the_declaration_head() {
    let class_source = r#"
public class Outer {
    abstract static class Descendant {}
}
"#;
    let mut class_context = context(class_source);
    class_context.current_class = Some("C1".into());
    let class_ops =
        JavaLayer::new().process_capture("class.root", class_source, &mut class_context);
    assert_eq!(
        projected_class_modifiers(&class_ops),
        vec![vec![DeclarationModifier::Export]]
    );

    let method_source = r#"
public void work() {
    // private protected static abstract
}
"#;
    let mut method_context = context(method_source);
    method_context.current_method = Some("M1".into());
    let method_ops =
        JavaLayer::new().process_capture("method.root", method_source, &mut method_context);
    assert_eq!(
        projected_method_modifiers(&method_ops),
        vec![vec![DeclarationModifier::Export]]
    );
}

#[test]
fn csharp_modifiers_follow_the_same_declaration_head_contract() {
    let class_source = r#"
public class Outer {
    private abstract static class Descendant {}
}
"#;
    let mut class_context = context(class_source);
    class_context.current_class = Some("C1".into());
    let class_ops =
        CSharpLayer::new().process_capture("class.root", class_source, &mut class_context);
    assert_eq!(
        projected_class_modifiers(&class_ops),
        vec![vec![DeclarationModifier::Export]]
    );

    let method_source = r#"
protected void Work() {
    // private static abstract async
}
"#;
    let mut method_context = context(method_source);
    method_context.current_method = Some("M1".into());
    let method_ops =
        CSharpLayer::new().process_capture("method.root", method_source, &mut method_context);
    assert_eq!(
        projected_method_modifiers(&method_ops),
        vec![vec![DeclarationModifier::Protected]]
    );
}
