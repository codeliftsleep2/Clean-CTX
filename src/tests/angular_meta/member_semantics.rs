use crate::layers::meta::semantic::SemanticRelation;

fn production_edges(source: &str) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("ts").expect("TypeScript language");
    let mut compiler = crate::ir::compiler::IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "issue-138-member-production",
            Some("C:/fixture/issue138.ts"),
            language,
            query,
            crate::compression::Fidelity::High,
            None,
            None,
        )
        .expect("TypeScript production compilation");
    let evidence = compiler
        .invocation_evidence
        .as_ref()
        .expect("production invocation evidence");
    let view = evidence
        .view("C:/fixture/issue138.ts", source)
        .expect("matching production invocation evidence");
    let directly_projected = super::project("C:/fixture/issue138.ts", source, &view);
    assert!(
        !directly_projected.is_empty(),
        "production invocation evidence did not admit Angular member semantics: {:?}",
        view.occurrences()
    );
    assert!(
        compiler.semantic_edges.iter().any(|edge| {
            matches!(
                edge.relation,
                SemanticRelation::HasInput | SemanticRelation::HasOutput
            )
        }),
        "direct projection succeeded but Angular registry integration dropped it: {:?}",
        compiler.semantic_edges
    );
    compiler.semantic_edges
}

fn edges(source: &str) -> Vec<crate::layers::meta::semantic::SemanticEdge> {
    let source_id = "standalone-typescript";
    let evidence = crate::ir::invocation_evidence::collect_typescript_source(source).unwrap();
    let view = evidence.view(source_id, source).unwrap();
    super::project(source_id, source, &view)
}

#[test]
fn structured_signal_initializers_publish_input_output_and_model_relations() {
    let source = "import { Component, input, model, output } from '@angular/core';\n@Component({}) class Panel { title = input<string>(); requiredTitle = input.required<string>(); changed = output<string>(); selected = model(false); }";
    let edges = edges(source);
    let relations = |name: &str| {
        edges
            .iter()
            .filter(|edge| edge.object.name == name)
            .map(|edge| edge.relation)
            .collect::<Vec<_>>()
    };
    assert_eq!(relations("title"), [SemanticRelation::HasInput]);
    assert_eq!(relations("requiredTitle"), [SemanticRelation::HasInput]);
    assert_eq!(relations("changed"), [SemanticRelation::HasOutput]);
    assert_eq!(
        relations("selected"),
        [SemanticRelation::HasInput, SemanticRelation::HasOutput]
    );
    assert!(edges.iter().all(|edge| edge.subject.name == "Panel"));
}

#[test]
fn structured_signals_reach_the_production_compiler_boundary() {
    let source = "import { Component, input, model, output } from '@angular/core';\n@Component({ selector: 'signal-probe', template: '' })\nexport class SignalProbe { title = input<string>(); requiredTitle = input.required<string>(); changed = output<string>(); selected = model(false); }";
    let edges = production_edges(source);
    let mut inputs = edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::HasInput && edge.subject.name == "SignalProbe"
        })
        .map(|edge| edge.object.name.as_str())
        .collect::<Vec<_>>();
    inputs.sort_unstable();
    assert_eq!(inputs, ["requiredTitle", "selected", "title"]);
    let mut outputs = edges
        .iter()
        .filter(|edge| {
            edge.relation == SemanticRelation::HasOutput && edge.subject.name == "SignalProbe"
        })
        .map(|edge| edge.object.name.as_str())
        .collect::<Vec<_>>();
    outputs.sort_unstable();
    assert_eq!(outputs, ["changed", "selected"]);
}

#[test]
fn same_spelling_local_receiver_and_nested_signal_calls_are_negative() {
    let source = "import { Component } from '@angular/core';\nfunction input<T>() { return 1; }\n@Component({}) class Panel { local = input<string>(); receiver = api.input(); nested = wrap(input()); }";
    assert!(edges(source).is_empty());
}

#[test]
fn functional_guard_authorities_are_joined() {
    let source = "import { inject } from '@angular/core';\nimport { CanActivateFn } from '@angular/router';\nclass AuditService {}\nexport const auditGuard: CanActivateFn = () => inject(AuditService).allowed();";
    let source_id = "standalone-typescript";
    let evidence = crate::ir::invocation_evidence::collect_typescript_source(source).unwrap();
    let view = evidence.view(source_id, source).unwrap();
    let call = view
        .occurrences()
        .iter()
        .find(|call| {
            matches!(call.callee(), crate::ir::invocation_evidence::WrittenCallee::Free { terminal }
                if view.slice(*terminal).unwrap() == "inject")
        })
        .expect("direct-free inject occurrence");
    let context = call.declaration_context().expect("callable owner context");
    assert_eq!(
        view.slice(context.written_name().expect("callable owner name"))
            .unwrap(),
        "auditGuard"
    );
    let language = crate::compression::language::safe_typescript_language().unwrap();
    let mut parser = tree_sitter::Parser::new();
    parser.set_language(&language).unwrap();
    let tree = parser.parse(source, None).unwrap();
    let declaration = super::exact_node(tree.root_node(), context.declaration())
        .expect("exact callable declaration node");
    let role_declaration =
        super::ancestor(declaration, "variable_declarator").unwrap_or(declaration);
    let role = super::functional_role(source_id, source, role_declaration)
        .expect("Angular functional role import authority");
    assert_eq!(role.0, "Guard");
    let argument = call.arguments().first().expect("inject token argument");
    let argument_node =
        super::exact_node(tree.root_node(), argument.span()).expect("exact inject token node");
    assert_eq!(
        crate::angular_meta::provider_semantics::reference_identity(
            source_id,
            source,
            tree.root_node(),
            argument_node,
        )
        .as_deref(),
        Some("AuditService")
    );
    assert!(
        super::functional_inject(source_id, source, tree.root_node(), &view, call).is_some(),
        "final functional inject projection"
    );
}
#[test]
fn functional_guard_inject_joins_role_owner_and_resolved_token() {
    let source = "import { inject } from '@angular/core';\nimport { CanActivateFn } from '@angular/router';\nclass AuditService {}\nexport const auditGuard: CanActivateFn = () => inject(AuditService).allowed();";
    let edges = edges(source);
    assert!(edges.iter().any(|edge| {
        edge.relation == SemanticRelation::Injects
            && edge.subject.entity_type == "Guard"
            && edge.subject.name == "auditGuard"
            && edge.object.entity_type == "Service"
            && edge.object.name == "AuditService"
    }));
}

#[test]
fn functional_inject_rejects_receiver_untyped_and_unresolved_forms() {
    let receiver = "import { inject } from '@angular/core';\nimport { CanActivateFn } from '@angular/router';\nconst guard: CanActivateFn = () => api.inject(Missing);";
    let untyped = "import { inject } from '@angular/core';\nclass AuditService {}\nconst helper = () => inject(AuditService);";
    let unresolved = "import { inject } from '@angular/core';\nimport { CanActivateFn } from '@angular/router';\nconst guard: CanActivateFn = () => inject(Missing);";
    assert!(edges(receiver).is_empty());
    assert!(edges(untyped).is_empty());
    assert!(edges(unresolved).is_empty());
}
