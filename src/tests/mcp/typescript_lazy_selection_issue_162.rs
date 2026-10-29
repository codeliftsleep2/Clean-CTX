//! Neutral structural/export proof and adversarial replay; no Angular publication.
use super::typescript_lazy_selection_support::*;
use crate::ir::typescript_resolution::TypeScriptAuthority;
#[test]
fn issue_162_lazy_selection_exact_forms_serde_and_absent_old_payload() {
    let _serial = crate::protocol::handler_response_serial();
    for form in ["direct", "reexport", "alias"] {
        let (root, config) = setup(form);
        let p = root.path();
        let state = crate::mcp::McpState::new(config);
        for file in ["one.ts", "two.ts"] {
            let a = capture(&state, p, file);
            let selected = proofs(&a);
            assert_eq!(
                selected
                    .iter()
                    .filter(|proof| proof.target.source.ends_with("screen-a.ts"))
                    .count(),
                4
            );
            assert_eq!(
                selected
                    .iter()
                    .filter(|proof| proof.target.source.ends_with("screen-b.ts"))
                    .count(),
                3
            );
            let mut spans = selected.iter().map(|p| p.outer).collect::<Vec<_>>();
            spans.sort();
            spans.dedup();
            assert_eq!(spans.len(), 7);
            for proof in selected {
                assert_eq!(proof.target.name, "Screen");
                assert_eq!(proof.source, a.source);
                assert_eq!(proof.source_hash, a.source_hash);
            }
            let restored: TypeScriptAuthority =
                serde_json::from_value(serde_json::to_value(&a).unwrap()).unwrap();
            assert_eq!(restored.arrow_selections, a.arrow_selections);
            proofs(&restored);
            let mut old = serde_json::to_value(&a).unwrap();
            old.as_object_mut().unwrap().remove("arrow_selections");
            let old: TypeScriptAuthority = serde_json::from_value(old).unwrap();
            assert!(
                old.arrow_selection_at(a.arrow_selections[0].outer)
                    .is_none()
            );
        }
    }
}
#[test]
fn issue_162_lazy_selection_borrowed_crossed_missing_competing_proof() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, root.path(), "one.ts");
    proofs(&a);
    let at = a.arrow_selections[0].outer;
    for mode in 0..24 {
        let mut bad = a.clone();
        let other = bad.arrow_selections[1].clone();
        let p = &mut bad.arrow_selections[0];
        match mode {
            0 => p.source_hash.push('x'),
            1 => p.source = root.path().join("two.ts"),
            2 => p.returned = other.returned,
            3 => p.parameters = other.parameters,
            4 => p.import = other.import,
            5 => p.literal = other.literal,
            6 => p.module = other.module,
            7 => p.selector = other.selector,
            8 => p.parameter = other.parameter,
            9 => p.member = other.member,
            10 => p.receiver = other.receiver,
            11 => p.property = other.property,
            12 => p.target = other.target,
            13 => p.export = "Other".into(),
            14 => p.access = other.access,
            15 => p.import_keyword = other.import_keyword,
            16 => p.then_property = other.then_property,
            17 => p.specifier = "./screen-b".into(),
            18 => bad.demanded_occurrences.clear(),
            19 => bad.module_requests.clear(),
            20 => bad.arrow_selections.push(a.arrow_selections[0].clone()),
            21 => bad.declaration_facets.clear(),
            22 => bad.capture_current = false,
            _ => bad.demanded_occurrences.retain(|site| {
                !(site.start == a.arrow_selections[0].parameter.0
                    && site.end == a.arrow_selections[0].parameter.1)
            }),
        }
        assert!(
            bad.arrow_selection_at(at).is_none(),
            "forged/missing {mode}"
        );
    }
}
#[test]
fn issue_162_lazy_selection_syntax_export_refusals_and_neutral_roles() {
    let _serial = crate::protocol::handler_response_serial();
    let (root, config) = setup("direct");
    let p = root.path();
    std::fs::write(
        p.join("private.ts"),
        "import {Component} from '@angular/core'; @Component({}) class Screen {}",
    )
    .unwrap();
    std::fs::write(
        p.join("type-only.ts"),
        "export type {Screen} from './screen-a';",
    )
    .unwrap();
    std::fs::write(
        p.join("ambiguous.ts"),
        "export * from './screen-a'; export * from './screen-b';",
    )
    .unwrap();
    std::fs::write(p.join("ordinary.ts"), "export class Screen {}").unwrap();
    let refused = [
        "()=>import('./screen-a').then(m=>other.Screen)",
        "()=>import('./missing').then(m=>m.Screen)",
        "()=>import('./screen-a').then(m=>m.Missing)",
        "()=>import('./private').then(m=>m.Screen)",
        "()=>import('./type-only').then(m=>m.Screen)",
        "()=>import('./ambiguous').then(m=>m.Screen)",
        "()=>import(specifier).then(m=>m.Screen)",
        "()=>import('./screen-a').then(m=>m['Screen'])",
        "async()=>import('./screen-a').then(m=>m.Screen)",
        "()=>import('./screen-a').then(async m=>m.Screen)",
        "()=>condition?import('./screen-a').then(m=>m.Screen):other",
        "()=>{return import('./screen-a').then(m=>m.Screen);}",
        "()=>import('./screen-a').then((m,extra)=>m.Screen)",
        "()=>other.then(m=>m.Screen)",
        "()=>import('./screen-a').then(m=>{return m.Screen;})",
        "(x)=>import('./screen-a').then(m=>m.Screen)",
        "()=>import('./screen-a',{}).then(m=>m.Screen)",
        "()=>import('./screen-a').then(m=>m.Screen,other)",
        "()=>import('./screen-a')?.then(m=>m.Screen)",
    ];
    for callback in refused {
        rewrite(p, callback);
        let state = crate::mcp::McpState::new(config.clone());
        let a = capture(&state, p, "one.ts");
        assert!(a.arrow_selections.is_empty(), "unsupported {callback}");
        assert_eq!(
            crate::angular_meta::route_components::project(&a).len(),
            1,
            "stage=eager-sibling-prerequisite callback={callback}; current={} disposition={:?} facets={:?}",
            a.capture_current,
            a.disposition,
            a.declaration_facets
        );
        assert!(
            crate::angular_meta::route_providers::project(&a)
                .iter()
                .any(|e| e.relation
                    == crate::layers::meta::semantic::SemanticRelation::RegistersProvider)
        );
    }
    rewrite(p, "()=>import('./ordinary').then(m=>m.Screen)");
    let state = crate::mcp::McpState::new(config);
    let a = capture(&state, p, "one.ts");
    let proof = a.arrow_selection_at(a.arrow_selections[0].outer).unwrap();
    assert!(
        crate::angular_meta::class_role_authority::declaration_role(&a, &proof.target).is_none()
    );
    assert_eq!(
        a.demanded_reference_at(
            &a.source,
            &a.source_hash,
            proof.property.0,
            proof.property.1
        ),
        Some(&proof.target)
    );
}
