use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::ir::layers::csharp::CSharpLayer;
use crate::ir::layers::patterns::CodePatternRecognizer;
use crate::ir::patterns::CompressingPatternRecognizer;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use crate::queries::CS_QUERY;

fn exact_calls(source: &str) -> Vec<SemanticEdge> {
    let language =
        crate::compression::language::safe_csharp_language().expect("csharp grammar enabled");
    let mut compiler = IRCompiler::new();
    compiler.add_language_layer(Box::new(CSharpLayer::new()));
    compiler.add_pattern_recognizer(Box::new(CodePatternRecognizer::new()));
    compiler.add_pattern_recognizer(Box::new(CompressingPatternRecognizer::new()));
    compiler
        .compile(
            source,
            "Example.cs",
            language,
            CS_QUERY,
            Fidelity::Medium,
            None,
        )
        .expect("production compilation succeeds");
    compiler
        .semantic_edges
        .into_iter()
        .filter(|edge| edge.layer == super::LAYER && edge.relation == SemanticRelation::Calls)
        .collect()
}

#[test]
fn same_written_name_resolves_to_receiver_owned_declarations() {
    let calls = exact_calls(
        r#"
class Alpha { public void Save() {} }
class Beta { public void Save() {} }
class Use {
    void Run(Alpha alpha, Beta beta) {
        alpha.Save();
        beta.Save();
    }
}
"#,
    );
    let targets: Vec<_> = calls.iter().map(|edge| edge.object.name.as_str()).collect();
    assert_eq!(targets, ["Alpha::Save()->void", "Beta::Save()->void"]);
    assert!(
        calls
            .iter()
            .all(|edge| edge.subject.name == "Use::Run(Alpha,Beta)->void")
    );
}

#[test]
fn overloads_resolve_only_when_written_arity_is_unique() {
    let calls = exact_calls(
        r#"
class Target {
    public void Save() {}
    public void Save(string value) {}
}
class Use { void Run(Target target) { target.Save(); target.Save("x"); } }
"#,
    );
    let targets: Vec<_> = calls.iter().map(|edge| edge.object.name.as_str()).collect();
    assert_eq!(
        targets,
        ["Target::Save()->void", "Target::Save(string)->void"]
    );
}

#[test]
fn unresolved_ambiguous_and_missing_targets_fail_closed() {
    let calls = exact_calls(
        r#"
class Target {
    public void Save(string left) {}
    public void Save(int right) {}
}
class Use {
    void Run(dynamic unknown, Target target) {
        target.Save(default);
        unknown.Save();
        missing.Save();
    }
}
"#,
    );
    assert!(calls.is_empty(), "no exact target may be manufactured");
}

#[test]
fn expression_bodied_property_keeps_its_caller_owner() {
    let calls = exact_calls(
        r#"
class Target { public int Read() => 1; }
class Use {
    private Target target;
    public int Value => target.Read();
}
"#,
    );
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].object.name, "Target::Read()->int");
    assert!(
        calls[0].subject.name.starts_with("Use::"),
        "property-generated callable must retain its declaring owner"
    );
}
