//! Agent MaxHeadRoom: frozen original semantics and construction-inclusive A4 budgets.
use super::*;
use crate::work_observations_af_perf_002 as work;
#[path = "a4_original_obligations_af_perf_002.rs"]
pub(super) mod original;

fn input(name: &str, file: &str, base: Option<&str>) -> ClassInput {
    ClassInput {
        part: CSharpDeclarationPart {
            key: CSharpSymbolKey {
                namespace_path: vec!["Example".into()],
                containing_type_path: Vec::new(),
                name: name.into(),
                generic_arity: 0,
            },
            kind: super::super::super::CSharpDeclarationKind::Class,
            is_partial: true,
            file: file.into(),
            span: CSharpSourceSpan {
                start_byte: 0,
                end_byte: 10,
            },
            base_references: base.into_iter().map(str::to_owned).collect(),
        },
        scope: Scope {
            namespace_path: vec!["Example".into()],
            containing_type_path: Vec::new(),
            ordinary_imports: Vec::new(),
            aliases: CSharpAliasContext::default(),
        },
        raw_class: name.into(),
        method_ids: Vec::new(),
        methods: Vec::new(),
        attributes: Vec::new(),
    }
}
fn record(value: AncestryObligations) -> (String, Vec<String>) {
    (
        format!("{:?}", (value.discovery_terms, value.owners, value.reasons)),
        value
            .requirements
            .iter()
            .map(CSharpAncestryObligation::identity_material)
            .collect(),
    )
}
fn old_record(value: original::AncestryObligations) -> (String, Vec<String>) {
    (
        format!("{:?}", (value.discovery_terms, value.owners, value.reasons)),
        value
            .requirements
            .iter()
            .map(original::CSharpAncestryObligation::identity_material)
            .collect(),
    )
}
fn compare(units: &[CSharpCompilationUnit], seeds: &[&str]) {
    let external = [
        CSharpTypeIdentity::known_external("Framework.Base"),
        CSharpTypeIdentity::known_external("Framework.MarkAttribute"),
    ];
    let old = original::ancestry_obligations_for_files(units, seeds, &external);
    let new = ancestry_obligations_for_files(units, seeds, &external);
    let old: BTreeMap<_, _> = old.into_iter().map(|(f, v)| (f, old_record(v))).collect();
    let new: BTreeMap<_, _> = new.into_iter().map(|(f, v)| (f, record(v))).collect();
    assert_eq!(new, old, "complete ordered batch obligation identity");
    for seed in seeds {
        assert_eq!(
            record(ancestry_obligations(units, seed, &external)),
            old_record(original::ancestry_obligations(units, seed, &external)),
            "complete single-owner obligation identity: {seed}"
        );
    }
}
#[test]
fn af002_a4_lookup_frozen_equivalence() {
    let mut first = input("Derived", "Z.cs", Some("B"));
    first.scope.aliases.directives.push(CSharpAliasDirective {
        alias: "B".into(),
        target: "Framework.Base".into(),
        scope_depth: 0,
        file: "Z.cs".into(),
        span: first.part.span,
    });
    first.attributes.push(AttributeInput {
        written: "Framework.Mark".into(),
        raw: "[Framework.Mark]".into(),
        span: first.part.span,
    });
    first.methods.push(MethodInput {
        name: "Run".into(),
        attributes: first.attributes.clone(),
    });
    let mut second = first.clone();
    second.part.file = "A.cs".into();
    second.scope.aliases.directives[0].target = "Missing.Base".into();
    second.scope.aliases.directives[0].file = "A.cs".into();
    let mut cases = vec![
        vec![],
        vec![first.clone()],
        vec![first.clone(), first.clone()],
        vec![first.clone(), second.clone()],
        vec![second.clone(), first.clone()],
    ];
    let mut nonpartial = second.clone();
    nonpartial.part.is_partial = false;
    cases.push(vec![first.clone(), nonpartial]);
    let mut conflict = second.clone();
    conflict.part.base_references = vec!["Other".into()];
    cases.push(vec![first.clone(), conflict]);
    let mut kind = second.clone();
    kind.part.kind = super::super::super::CSharpDeclarationKind::Interface;
    cases.push(vec![first.clone(), kind]);
    cases.push(vec![input("Whitespace", "Z.cs", Some(" Framework.Base "))]);
    cases.push(vec![
        input("CycleA", "Z.cs", Some("CycleB")),
        input("CycleB", "A.cs", Some("CycleA")),
    ]);
    cases.push(vec![
        input("Child", "Z.cs", Some("Parent")),
        input("Parent", "A.cs", Some("Framework.Base")),
    ]);
    let mut generic = input("Derived", "Z.cs", None);
    generic.part.key.generic_arity = 1;
    let mut nested = first.clone();
    nested.part.key.containing_type_path = vec!["Outer".into()];
    let mut namespace = first.clone();
    namespace.part.key.namespace_path = vec!["Other".into()];
    cases.push(vec![first.clone(), generic, nested, namespace]);
    let mut shadow = first.clone();
    shadow.scope.aliases.directives.push(CSharpAliasDirective {
        alias: "B".into(),
        target: "Missing.Shadow".into(),
        scope_depth: 1,
        file: "Z.cs".into(),
        span: first.part.span,
    });
    cases.push(vec![shadow]);
    for (i, classes) in cases.into_iter().enumerate() {
        let units = vec![CSharpCompilationUnit {
            classes: classes.clone(),
        }];
        compare(&units, &["Z.cs", "A.cs", "Missing.cs", "Z.cs"]);
        let split: Vec<_> = classes
            .into_iter()
            .map(|c| CSharpCompilationUnit { classes: vec![c] })
            .collect();
        compare(&split, &["Z.cs", "A.cs", "Missing.cs"]);
        eprintln!("AF002_A4_EQUIVALENCE case={i}");
    }
}

#[test]
fn af002_a4_lookup_construction_budget() {
    let mut violations = Vec::new();
    // Classes, owners, partial family size, ancestry depth.
    for (count, files, density, depth) in [
        (0, 1, 1, 0),
        (1, 1, 1, 0),
        (2, 2, 1, 0),
        (8, 2, 1, 0),
        (64, 8, 1, 0),
        (256, 16, 1, 0),
        (64, 8, 8, 0),
        (64, 8, 64, 0),
        (64, 8, 1, 8),
        (256, 16, 4, 16),
    ] {
        let classes = (0..count)
            .map(|i| {
                let name = format!("Type{}", i / density);
                let base = (i < depth).then(|| format!("Type{}", i / density + 1));
                input(&name, &format!("Owner{}.cs", i % files), base.as_deref())
            })
            .collect();
        let units = vec![CSharpCompilationUnit { classes }];
        let names: Vec<_> = (0..files).map(|i| format!("Owner{i}.cs")).collect();
        let seeds: Vec<_> = names.iter().map(String::as_str).collect();
        // Semantic checks occur before the measured interval and before any RED.
        compare(&units, &seeds);
        work::reset();
        let start = std::time::Instant::now();
        let result = ancestry_obligations_for_files(&units, &seeds, &[]);
        let elapsed = start.elapsed().as_micros();
        let w = work::snapshot().a4;
        assert_eq!(result.len(), files);
        eprintln!(
            "AF002_A4_BUDGET {}",
            serde_json::json!({"classes":count,"owners":files,
            "partial_density":density,"depth":depth,"work":w,"us":elapsed})
        );
        if w.seed_class_visits + w.family_class_visits + w.base_find_visits != 0 {
            violations.push((count, files, density, depth));
        }
        // Both baseline and repair must honestly expose their own construction.
        if w.lookup_input_visits != 0 {
            assert_eq!(w.lookup_input_visits, count);
            assert_eq!(w.lookup_insert_attempts, [count; 3]);
            assert_eq!(w.retained_input_references, count);
            assert!(w.retained_seed_references <= count);
            assert!(w.retained_family_vectors <= count);
            assert!(w.retained_file_sets <= files);
            assert!(w.family_reference_capacity >= count);
            assert_eq!(w.lookup_calls[0], files);
            if count > 1 {
                assert!(w.lookup_key_comparisons.iter().sum::<usize>() > 0);
            }
        }
    }
    assert!(
        violations.is_empty(),
        "A4 redundant full-input retrieval budget: {violations:?}"
    );
}
