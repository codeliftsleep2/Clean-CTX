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

#[test]
fn a3_direct_precedence_and_extension_ambiguity_remain_exact() {
    let direct = exact_calls(
        r#"
class Target { public void Save() {} }
static class Extensions { public static void Save(Target target) {} }
class Use { void Run(Target target) { target.Save(); } }
"#,
    );
    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].object.name, "Target::Save()->void");
    let extension = exact_calls(
        r#"
class Target {}
static class Extensions { public static void Save(Target target) {} }
class Use { void Run(Target target) { target.Save(); } }
"#,
    );
    assert_eq!(extension.len(), 1);
    assert_eq!(extension[0].object.name, "Extensions::Save(Target)->void");
    let ambiguous = exact_calls(
        r#"
class Target {}
static class A { public static void Save(Target target) {} }
static class B { public static void Save(Target target) {} }
class Use { void Run(Target target) { target.Save(); } }
"#,
    );
    assert!(ambiguous.is_empty());
}

#[test]
fn a3_direct_competitors_never_fall_back_to_extension() {
    let calls = exact_calls(
        r#"
class Target { public void Save(string x) {} public void Save(int x) {} }
static class Extensions { public static void Save(Target target, object value) {} }
class Use { void Run(Target target) { target.Save(default); } }
"#,
    );
    assert!(calls.is_empty());
}

#[test]
fn a3_borrowed_buckets_preserve_duplicates_order_and_fresh_authority() {
    use super::{Declaration, DeclarationLookup};
    let declaration = Declaration {
        id: "m".into(),
        owner: "Target".into(),
        name: "Save".into(),
        parameters: vec![],
        return_type: "void".into(),
        is_static: false,
    };
    let authority = vec![declaration.clone(), declaration];
    let first = DeclarationLookup::new(&authority);
    let bucket = &first.direct[&("Target", "Save", 0)];
    assert_eq!(bucket.len(), 2);
    assert!(std::ptr::eq(bucket[0], &authority[0]));
    assert!(std::ptr::eq(bucket[1], &authority[1]));
    let changed = vec![Declaration {
        name: "Other".into(),
        ..authority[0].clone()
    }];
    let second = DeclarationLookup::new(&changed);
    assert!(!second.direct.contains_key(&("Target", "Save", 0)));
    assert_eq!(second.direct[&("Target", "Other", 0)].len(), 1);
}

#[test]
fn a3_gap_dense_extensions_preserve_competing_authority() {
    use crate::work_observations_af_perf_002 as work;
    for n in [1, 2, 4, 8, 16, 64, 256] {
        let mut source =
            "class Target {} class Use { void Run(Target target) { target.Save(); } }".to_owned();
        for index in 0..n {
            source.push_str(&format!(
                "static class E{index} {{ public static void Save(Target target) {{}} }}"
            ));
        }
        work::reset();
        let start = std::time::Instant::now();
        let calls = exact_calls(&source);
        let counts = work::snapshot();
        assert_eq!(calls.len(), usize::from(n == 1));
        if n == 1 {
            assert_eq!(calls[0].object.name, "E0::Save(Target)->void");
        }
        assert_eq!(counts.csharp_lookup_builds, 1);
        assert_eq!(counts.csharp_lookup_build_visits, n + 1);
        assert_eq!(counts.csharp_declaration_probes[1], n);
        eprintln!(
            "A3_GAP_DENSE n={n} builds={} build_visits={} build_probes={} lookups={} candidate_visits={:?} payload_bytes={} us={} boundary=production-compilation-not-public-query",
            counts.csharp_lookup_builds,
            counts.csharp_lookup_build_visits,
            counts.csharp_lookup_build_probes,
            counts.csharp_lookup_probes,
            counts.csharp_declaration_probes,
            counts.csharp_lookup_payload_bytes,
            start.elapsed().as_micros()
        );
    }
}

#[test]
fn a3_gap_extension_static_arity_and_receiver_refusals() {
    for declaration in [
        "class E { public void Save(Target target) {} }",
        "static class E { public static void Save(Target target, int extra) {} }",
        "static class E { public static void Save(Other target) {} }",
    ] {
        let source = format!(
            "class Target {{}} class Other {{}} {declaration} class Use {{ void Run(Target target) {{ target.Save(); }} }}"
        );
        assert!(
            exact_calls(&source).is_empty(),
            "unsupported extension must refuse: {declaration}"
        );
    }
}
