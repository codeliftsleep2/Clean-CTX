use crate::compression::Fidelity;
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::opcodes::{CoreOp, ExecutionContextKind, SideEffectKind};
use crate::ir::pipeline::{PassContext, PassPipeline};
use std::collections::{HashMap, HashSet};

fn compile(source: &str) -> PassContext {
    let mut context = PassContext::new(
        source.to_string(),
        "nested_semantics.cs".into(),
        Fidelity::Low,
    );
    context.language =
        Some(crate::compression::language::safe_csharp_language().expect("csharp grammar enabled"));
    context.query_string = crate::queries::CS_QUERY.to_string();
    context.set_language_layers(vec![Box::new(CSharpLayer::new())]);
    PassPipeline::default_production()
        .run(&mut context)
        .expect("production pipeline should compile C# fixture");
    context
}

fn method_semantics(context: &PassContext) -> HashMap<String, (bool, bool)> {
    let method_names: HashMap<_, _> = context
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::DefMethod(_, id, name) => Some((id.clone(), name.clone())),
            _ => None,
        })
        .collect();
    let realtime: HashSet<_> = context
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::ExecutionContext(id, ExecutionContextKind::Realtime) => Some(id.clone()),
            _ => None,
        })
        .collect();
    let io: HashSet<_> = context
        .instructions
        .iter()
        .filter_map(|op| match op {
            CoreOp::SideEffect(id, SideEffectKind::Io) => Some(id.clone()),
            _ => None,
        })
        .collect();

    method_names
        .into_iter()
        .map(|(id, name)| (name, (realtime.contains(&id), io.contains(&id))))
        .collect()
}

#[test]
fn csharp_class_semantics_follow_nested_lexical_ownership() {
    let source = r#"
class OuterHub : Hub
{
    void HubBefore() { }
    class PlainNested { void PlainInsideHub() { } }
    void HubAfter() { }
}

class PlainAroundHub
{
    class NestedHub : Hub { void NestedHubMethod() { } }
    void PlainAfterHub() { }
}

class OuterDisposable : SomeBase, IDisposable
{
    void DisposableBefore() { }
    class PlainNested { void PlainInsideDisposable() { } }
    void DisposableAfter() { }
}

class PlainAroundDisposable
{
    class NestedDisposable : SomeBase, IAsyncDisposable
    {
        void NestedDisposableMethod() { }
    }
    void PlainAfterDisposable() { }
}

class SiblingHub : Hub { void SiblingHubMethod() { } }
class PlainSibling { void PlainSiblingMethod() { } }
class SiblingDisposable : SomeBase, IDisposable { void SiblingDisposableMethod() { } }
class PlainAfterDisposableSibling { void PlainDisposableSiblingMethod() { } }
"#;
    let semantics = method_semantics(&compile(source));

    for name in [
        "HubBefore",
        "HubAfter",
        "NestedHubMethod",
        "SiblingHubMethod",
    ] {
        assert_eq!(semantics.get(name), Some(&(true, false)), "{name}");
    }
    for name in ["PlainInsideHub", "PlainAfterHub", "PlainSiblingMethod"] {
        assert_eq!(semantics.get(name), Some(&(false, false)), "{name}");
    }
    for name in [
        "DisposableBefore",
        "DisposableAfter",
        "NestedDisposableMethod",
        "SiblingDisposableMethod",
    ] {
        assert_eq!(semantics.get(name), Some(&(false, true)), "{name}");
    }
    for name in [
        "PlainInsideDisposable",
        "PlainAfterDisposable",
        "PlainDisposableSiblingMethod",
    ] {
        assert_eq!(semantics.get(name), Some(&(false, false)), "{name}");
    }
}
