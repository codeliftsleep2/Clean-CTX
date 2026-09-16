// src/tests/angular_meta/constructor_di_edges.rs
//
// RED-DI / RED-W regressions at the semantic-edge and workspace-index level.
//
// These tests drive the REAL production path:
//
//   source → CoreIRPass captures → MetaLayerPass → AngularMetaLayer
//     → extract_graph_entries → extract_constructor_injects
//     → class_to_semantic_edges → SemanticRelation::Injects
//     → WorkspaceIndex.add_edges → forward/reverse edge queries
//
// They exist to prove that the modifier-independence fix is reachable from
// the production entry point — not merely from a unit-level helper — and that
// no workaround was added in `WorkspaceIndex` or the query layer.

use crate::compression::Fidelity;
use crate::ir::compiler::IRCompiler;
use crate::layers::meta::semantic::{SemanticEdge, SemanticRelation};
use crate::workspace::index::WorkspaceIndex;

/// Compile `source` through the production pipeline (captures → meta layers →
/// semantic edges), exactly as `MetaLayerPass` does.
fn compile_edges(source: &str, path: &str) -> Vec<SemanticEdge> {
    let (language, query) =
        crate::compression::language::language_for_extension("ts").expect("TypeScript language");
    let mut compiler = IRCompiler::new();
    compiler
        .compile_focused(
            source,
            "α-test",
            Some(path),
            language,
            query,
            Fidelity::High,
            None,
            None,
        )
        .expect("TypeScript compilation");
    compiler.semantic_edges
}

/// The sorted `Injects` object names of `subject_name` in a compiled edge set.
fn inject_objects(edges: &[SemanticEdge], subject_name: &str) -> Vec<String> {
    let mut out: Vec<String> = edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Injects && e.subject.name == subject_name)
        .map(|e| e.object.name.clone())
        .collect();
    out.sort();
    out
}

/// All `Injects` edges in a compiled edge set (any subject).
fn inject_edges(edges: &[SemanticEdge]) -> Vec<&SemanticEdge> {
    edges
        .iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .collect()
}

/// Build a workspace index from production-compiled sources.
fn index_from(files: &[(&str, &str)]) -> WorkspaceIndex {
    let mut index = WorkspaceIndex::new();
    for (path, source) in files {
        index.add_edges(path, compile_edges(source, path));
    }
    index
}

/// Reverse-edge count for `angular/Service/<service>` using `Injects` only.
fn reverse_inject_count(index: &WorkspaceIndex, service: &str) -> usize {
    index
        .reverse_edges_by_identity("angular", "Service", service)
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .count()
}

/// Forward `Injects` count for `angular/Service/<class>` (an `@Injectable`
/// class is typed as `Service` by the Angular meta layer).
fn forward_inject_count(index: &WorkspaceIndex, class: &str) -> usize {
    index
        .forward_edges_by_identity("angular", "Service", class)
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .count()
}

// ── RED-DI1 — bare typed constructor parameter reaches the edge model ──

#[test]
fn red_di1_bare_typed_parameter_produces_injects_edge_in_production_pass() {
    let source = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(fooService: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/consumer.service.ts");
    assert_eq!(
        inject_objects(&edges, "Consumer"),
        vec!["FooService".to_string()],
        "a bare typed constructor parameter must produce the Injects edge"
    );
}

// ── RED-DI2/3/4 — shorthand controls stay green ───────────────────────

#[test]
fn red_di2_private_shorthand_produces_injects_edge() {
    let source = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(private fooService: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/consumer.service.ts");
    assert_eq!(
        inject_objects(&edges, "Consumer"),
        vec!["FooService".to_string()]
    );
}

#[test]
fn red_di3_readonly_shorthand_produces_injects_edge() {
    let source = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(readonly fooService: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/consumer.service.ts");
    assert_eq!(
        inject_objects(&edges, "Consumer"),
        vec!["FooService".to_string()]
    );
}

#[test]
fn red_di4_private_readonly_shorthand_produces_injects_edge() {
    let source = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(private readonly fooService: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/consumer.service.ts");
    assert_eq!(
        inject_objects(&edges, "Consumer"),
        vec!["FooService".to_string()]
    );
}

#[test]
fn bare_and_shorthand_forms_produce_the_same_injects_edge() {
    let forms = [
        "constructor(fooService: FooService) {}",
        "constructor(private fooService: FooService) {}",
        "constructor(public fooService: FooService) {}",
        "constructor(protected fooService: FooService) {}",
        "constructor(readonly fooService: FooService) {}",
        "constructor(private readonly fooService: FooService) {}",
    ];
    for form in forms {
        let source = format!(
            "import {{ Injectable }} from '@angular/core';\n\n@Injectable()\nexport class Consumer {{\n    {form}\n}}\n"
        );
        let edges = compile_edges(&source, "C:/repo/consumer.service.ts");
        assert_eq!(
            inject_objects(&edges, "Consumer"),
            vec!["FooService".to_string()],
            "form `{form}` must produce exactly one Injects edge"
        );
    }
}

// ── RED-DI5 — multiple mixed parameters ───────────────────────────────

#[test]
fn red_di5_mixed_parameters_produce_three_distinct_edges_once_each() {
    let source = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(
        private alpha: AlphaService,
        beta: BetaService,
        readonly gamma: GammaService
    ) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/consumer.service.ts");
    assert_eq!(
        inject_objects(&edges, "Consumer"),
        vec![
            "AlphaService".to_string(),
            "BetaService".to_string(),
            "GammaService".to_string()
        ]
    );
}

// ── RED-DI6 — Angular component (not only @Injectable) ────────────────

#[test]
fn red_di6_angular_component_bare_parameter_produces_injects_edge() {
    let source = r#"
import { Component } from '@angular/core';

@Component({ selector: 'app-example', template: '' })
export class ExampleComponent {
    constructor(foo: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/example.component.ts");
    let injects = inject_edges(&edges);
    assert_eq!(injects.len(), 1, "expected exactly one Injects edge");
    assert_eq!(injects[0].subject.entity_type, "Component");
    assert_eq!(injects[0].subject.name, "ExampleComponent");
    assert_eq!(injects[0].object.entity_type, "Service");
    assert_eq!(injects[0].object.name, "FooService");
    assert_eq!(injects[0].layer, "angular");
}

// ── RED-DI7 — non-Angular TypeScript class guard ──────────────────────

#[test]
fn red_di7_plain_typescript_class_gains_no_angular_injects_edge() {
    let source = r#"
import { FooService } from './foo.service';

export class PlainClass {
    constructor(fooService: FooService) {}
}
"#;
    let edges = compile_edges(source, "C:/repo/plain.ts");
    assert!(
        inject_edges(&edges).is_empty(),
        "ordinary TypeScript constructors must not gain Angular Injects edges: {:?}",
        inject_edges(&edges)
    );
}

#[test]
fn red_di7_plain_class_does_not_appear_in_reverse_edges() {
    let plain = r#"
import { FooService } from './foo.service';

export class PlainClass {
    constructor(fooService: FooService) {}
}
"#;
    let index = index_from(&[("C:/repo/plain.ts", plain)]);
    assert_eq!(reverse_inject_count(&index, "FooService"), 0);
}

// ── RED-DI8 — decorated injection tokens keep their identity ─────────

#[test]
fn red_di8_optional_and_inject_decorators_preserve_type_identity() {
    let optional = r#"
import { Injectable, Optional } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(@Optional() foo: FooService) {}
}
"#;
    let injected = r#"
import { Injectable, Inject } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(@Inject(API_TOKEN) api: ApiClient) {}
}
"#;
    assert_eq!(
        inject_objects(&compile_edges(optional, "C:/repo/a.service.ts"), "Consumer"),
        vec!["FooService".to_string()]
    );
    assert_eq!(
        inject_objects(&compile_edges(injected, "C:/repo/b.service.ts"), "Consumer"),
        vec!["ApiClient".to_string()],
        "the pre-existing model keeps the parameter TYPE as the injection identity"
    );
}

// ── RED-DI9 — formatter independence ──────────────────────────────────

#[test]
fn red_di9_multiline_constructor_produces_the_same_edges() {
    let single_line = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(fooService: FooService, barService: BarService) {}
}
"#;
    let multiline = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class Consumer {
    constructor(
        fooService: FooService,
        barService: BarService,
    ) {}
}
"#;
    let expected = vec!["BarService".to_string(), "FooService".to_string()];
    assert_eq!(
        inject_objects(&compile_edges(single_line, "C:/repo/s.service.ts"), "Consumer"),
        expected
    );
    assert_eq!(
        inject_objects(&compile_edges(multiline, "C:/repo/m.service.ts"), "Consumer"),
        expected
    );
}

// ─ RED-DI10 — exactly one edge per parameter, no duplicates ─────────

#[test]
fn red_di10_modifier_forms_never_duplicate_the_edge() {
    let forms = [
        "constructor(fooService: FooService) {}",
        "constructor(private fooService: FooService) {}",
        "constructor(readonly fooService: FooService) {}",
        "constructor(private readonly fooService: FooService) {}",
    ];
    for form in forms {
        let source = format!(
            "import {{ Injectable }} from '@angular/core';\n\n@Injectable()\nexport class Consumer {{\n    {form}\n}}\n"
        );
        let path = "C:/repo/consumer.service.ts";
        let edges = compile_edges(&source, path);
        assert_eq!(
            inject_edges(&edges).len(),
            1,
            "form `{form}` produced duplicate Injects edges"
        );
        let index = index_from(&[(path, source.as_str())]);
        assert_eq!(forward_inject_count(&index, "Consumer"), 1);
        assert_eq!(reverse_inject_count(&index, "FooService"), 1);
    }
}

// ── RED-W1 — forward edges expose the bare-parameter injection ────────

#[test]
fn red_w1_forward_edges_show_injects_for_a_bare_parameter() {
    let source = r#"
import { Component } from '@angular/core';

@Component({ selector: 'app-consumer', template: '' })
export class ConsumerComponent {
    constructor(fooService: FooService) {}
}
"#;
    let index = index_from(&[("C:/repo/consumer.component.ts", source)]);
    let outgoing = index.forward_edges_by_identity("angular", "Component", "ConsumerComponent");
    let injects: Vec<_> = outgoing
        .iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .collect();
    assert_eq!(injects.len(), 1);
    assert_eq!(injects[0].object.entity_type, "Service");
    assert_eq!(injects[0].object.name, "FooService");
}

// ── RED-W2 — reverse edges treat both styles equally ──────────────────

#[test]
fn red_w2_reverse_edges_include_shorthand_and_bare_consumers_equally() {
    let shorthand = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class ShorthandConsumer {
    constructor(private fooService: FooService) {}
}
"#;
    let bare = r#"
import { Injectable } from '@angular/core';

@Injectable()
export class BareConsumer {
    constructor(fooService: FooService) {}
}
"#;
    let index = index_from(&[
        ("C:/repo/shorthand.service.ts", shorthand),
        ("C:/repo/bare.service.ts", bare),
    ]);

    let mut subjects: Vec<String> = index
        .reverse_edges_by_identity("angular", "Service", "FooService")
        .into_iter()
        .filter(|e| e.relation == SemanticRelation::Injects)
        .map(|e| e.subject.name.clone())
        .collect();
    subjects.sort();
    assert_eq!(
        subjects,
        vec!["BareConsumer".to_string(), "ShorthandConsumer".to_string()],
        "both constructor-injection styles must appear as consumers"
    );
}

// ─ RED-W3 — mixed workspace: no style-dependent undercount ───────────

#[test]
fn red_w3_mixed_workspace_reverse_edge_count_is_n_plus_m() {
    // N shorthand consumers + M bare-parameter consumers, plus one plain
    // TypeScript class with the same injection shape (must not be counted).
    const N: usize = 3;
    const M: usize = 2;

    let mut files: Vec<(String, String)> = Vec::new();
    for i in 0..N {
        files.push((
            format!("C:/repo/shorthand-{i}.service.ts"),
            format!(
                "import {{ Injectable }} from '@angular/core';\n\n@Injectable()\nexport class Shorthand{i} {{\n    constructor(private fooService: FooService) {{}}\n}}\n"
            ),
        ));
    }
    for i in 0..M {
        files.push((
            format!("C:/repo/bare-{i}.service.ts"),
            format!(
                "import {{ Injectable }} from '@angular/core';\n\n@Injectable()\nexport class Bare{i} {{\n    constructor(fooService: FooService) {{}}\n}}\n"
            ),
        ));
    }
    files.push((
        "C:/repo/plain.ts".to_string(),
        "export class PlainClass {\n    constructor(fooService: FooService) {}\n}\n".to_string(),
    ));

    let mut index = WorkspaceIndex::new();
    for (path, source) in &files {
        index.add_edges(path, compile_edges(source, path));
    }

    assert_eq!(
        reverse_inject_count(&index, "FooService"),
        N + M,
        "every constructor-injected consumer must be counted exactly once, regardless of style"
    );
}