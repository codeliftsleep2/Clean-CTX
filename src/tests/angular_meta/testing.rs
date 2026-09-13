use std::path::Path;

use crate::angular_meta::run_meta_layer_with_config_and_path;
use crate::angular_meta::testing::{
    extract_testing_semantic_edges, extract_testing_shape, is_testing_source,
};
use crate::compression::Fidelity;
use crate::config::MetaLayerConfig;
use crate::layers::meta::semantic::SemanticRelation;

const BASIC_SPEC: &str = r#"
describe('foo', () => {
  it('works', () => {});
});
"#;

#[test]
fn red_a1_spec_path_activates_without_angular_decorators() {
    assert!(is_testing_source(BASIC_SPEC, Path::new("foo.spec.ts")));
}

#[test]
fn red_a2_describe_activates_non_spec_without_angular_edge() {
    assert!(is_testing_source(BASIC_SPEC, Path::new("foo.ts")));
    assert!(extract_testing_semantic_edges(BASIC_SPEC, Path::new("foo.ts")).is_empty());
}

#[test]
fn red_a3_testbed_activates_non_spec() {
    let source = "TestBed.configureTestingModule({ providers: [] });";
    assert!(is_testing_source(source, Path::new("setup.ts")));
}

#[test]
fn vi_fn_activates_non_spec() {
    assert!(is_testing_source(
        "const callback = vi.fn();",
        Path::new("utility.ts")
    ));
}

#[test]
fn testing_marker_vocabulary_round_trips_through_shared_phi_registry() {
    use crate::angular_meta::phi::{PhiMarker, expand_phi};
    use crate::angular_meta::testing::TestKind;

    assert_eq!(expand_phi::<TestKind>("Φdescribe"), Some("describe suite"));
    assert_eq!(expand_phi::<TestKind>("Φit"), Some("test case"));
    assert_eq!(
        expand_phi::<TestKind>("ΦtestBed"),
        Some("TestBed configuration")
    );
    assert_eq!(expand_phi::<TestKind>("Φspy"), Some("Vitest spy/mock"));
    let mut prefixes = std::collections::HashSet::new();
    for kind in TestKind::all_in_expand_order() {
        assert!(prefixes.insert(kind.marker_prefix()));
    }
    assert_eq!(
        crate::angular_meta::markers::expand_phi_in_line("Φspy:service.load"),
        "Vitest spy/mock service.load"
    );
}

#[test]
fn red_a4_low_is_one_summary_only() {
    let source = r#"
describe('outer', () => {
  describe('inner', () => {
    it('one', () => {}); it('two', () => {});
  });
  it('three', () => {}); it('four', () => {}); it('five', () => {});
});
TestBed.configureTestingModule({ imports: [FooComponent] });
const callback = vi.fn();
"#;
    let rendered = extract_testing_shape(source, Fidelity::Low)
        .expect("testing shape")
        .render(Fidelity::Low);
    assert_eq!(rendered.matches("Φdescribe:").count(), 1);
    assert!(rendered.contains("Φdescribe:[describes=2 tests=5]"));
    for forbidden in ["Φit:", "ΦtestBed:", "Φspy:"] {
        assert!(
            !rendered.contains(forbidden),
            "unexpected {forbidden}: {rendered}"
        );
    }
}

#[test]
fn red_a5_medium_extracts_only_static_test_descriptions() {
    let source = r#"
describe('suite', () => {
  it('single', () => {});
  it("double", () => {});
  it(`template`, () => {});
  it(`dynamic ${name}`, () => {});
  it(name, () => {});
});
"#;
    let rendered = extract_testing_shape(source, Fidelity::Medium)
        .expect("testing shape")
        .render(Fidelity::Medium);
    for expected in ["Φit:single", "Φit:double", "Φit:template"] {
        assert!(
            rendered.contains(expected),
            "missing {expected}: {rendered}"
        );
    }
    assert!(!rendered.contains("dynamic"));
    assert!(!rendered.contains("Φit:name"));
}

#[test]
fn red_a6_medium_summarizes_testbed_arrays_in_either_order() {
    let source = r#"
await TestBed.configureTestingModule({
  providers: [
    AccountService,
    { provide: API_URL, useValue: 'ignored' }
  ],
  imports: [HttpClientTestingModule]
}).compileComponents();
"#;
    let rendered = extract_testing_shape(source, Fidelity::Medium)
        .expect("testing shape")
        .render(Fidelity::Medium);
    assert!(
        rendered.contains(
            "ΦtestBed:providers=[AccountService,API_URL] imports=[HttpClientTestingModule]"
        )
    );
}

#[test]
fn red_a7_a8_a9_high_extracts_vitest_spies_and_mocks() {
    let source = r#"
const callback = vi.fn().mockReturnValue(value);
vi.spyOn(accountService, 'load').mockResolvedValue(result);
vi.mock('./account.service');
"#;
    let rendered = extract_testing_shape(source, Fidelity::High)
        .expect("testing shape")
        .render(Fidelity::High);
    for expected in [
        "Φspy:callback",
        "Φspy:accountService.load",
        "Φspy:module:./account.service",
    ] {
        assert!(
            rendered.contains(expected),
            "missing {expected}: {rendered}"
        );
    }
}

#[test]
fn red_a10_nested_describes_are_high_only() {
    let source = r#"
describe('AccountComponent', () => {
  describe('save', () => { it('works', () => {}); });
});
"#;
    let low = extract_testing_shape(source, Fidelity::Low)
        .unwrap()
        .render(Fidelity::Low);
    let medium = extract_testing_shape(source, Fidelity::Medium)
        .unwrap()
        .render(Fidelity::Medium);
    let high = extract_testing_shape(source, Fidelity::High)
        .unwrap()
        .render(Fidelity::High);
    assert_eq!(low.matches("Φdescribe:").count(), 1);
    assert_eq!(medium.matches("Φdescribe:").count(), 1);
    assert!(high.contains("Φdescribe:AccountComponent"));
    assert!(high.contains("Φdescribe:AccountComponent > save"));
}

#[test]
fn red_a11_configuration_disables_testing_only() {
    let source = r#"
import { signal } from '@angular/core';
const count = signal(0);
describe('suite', () => { it('works', () => {}); });
TestBed.createComponent(AccountComponent);
"#;
    let mut config = MetaLayerConfig::default();
    config.testing.enabled = false;
    let block = run_meta_layer_with_config_and_path(
        source,
        &[],
        Fidelity::High,
        Some(&config),
        Path::new("account.component.spec.ts"),
    )
    .expect("signals remain enabled");
    let rendered = block.render();
    assert!(rendered.contains("Φsignal:count"));
    assert!(!rendered.contains("Testing Meta"));
    assert!(!rendered.contains("Φdescribe:"));
    assert!(!rendered.contains("Φit:"));
    assert!(!rendered.contains("ΦtestBed:"));
    assert!(!rendered.contains("Φspy:"));

    let project_config: crate::config::CleanCtxConfig = serde_json::from_str(
        r#"{ "meta_layers": { "angular": { "testing": { "enabled": false } } } }"#,
    )
    .expect("testing opt-out config");
    let semantic_edges = crate::layers::LayerRegistry::global().collect_semantic_edges_with_path(
        source,
        Path::new("account.component.spec.ts"),
        &[],
        Fidelity::High,
        Some(&project_config),
    );
    assert!(
        semantic_edges
            .iter()
            .all(|edge| edge.relation != SemanticRelation::Tests)
    );
}

#[test]
fn red_s1_create_component_infers_angular_tests_edge() {
    let source = "const fixture = TestBed.createComponent(AccountComponent);";
    let edges = extract_testing_semantic_edges(source, Path::new("custom.spec.ts"));
    assert_eq!(edges.len(), 1);
    let edge = &edges[0];
    assert_eq!(edge.relation, SemanticRelation::Tests);
    assert_eq!(edge.layer, "angular");
    assert_eq!(edge.subject.domain, "angular");
    assert_eq!(edge.subject.entity_type, "TestArtifact");
    assert_eq!(edge.object.name, "AccountComponent");
    assert_eq!(edge.object.entity_type, "Component");
}

#[test]
fn red_s2_filename_inference_uses_actual_path() {
    let edges =
        extract_testing_semantic_edges(BASIC_SPEC, Path::new("src/account.component.spec.ts"));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].object.name, "AccountComponent");
}

#[test]
fn red_s3_agreement_deduplicates_edge() {
    let source = "TestBed.createComponent(AccountComponent);";
    let edges = extract_testing_semantic_edges(source, Path::new("src/account.component.spec.ts"));
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].object.name, "AccountComponent");
}

#[test]
fn red_s4_conflicting_filename_and_explicit_sut_omits_edge() {
    let source = "TestBed.createComponent(AdminComponent);";
    let edges = extract_testing_semantic_edges(source, Path::new("src/account.component.spec.ts"));
    assert!(edges.is_empty());
}

#[test]
fn red_s5_generic_vitest_spec_has_no_angular_edge() {
    let source = r#"
describe('utility', () => {
  it('works', () => { const callback = vi.fn(); });
});
"#;
    assert!(extract_testing_shape(source, Fidelity::High).is_some());
    assert!(extract_testing_semantic_edges(source, Path::new("utility.spec.ts")).is_empty());
}

#[test]
fn multiple_explicit_components_emit_multiple_consistent_tests_edges() {
    let source = r#"
TestBed.createComponent(FooComponent);
TestBed.createComponent(BarComponent);
"#;
    let edges = extract_testing_semantic_edges(source, Path::new("combined.spec.ts"));
    assert_eq!(edges.len(), 2);
    assert!(
        edges
            .iter()
            .all(|edge| edge.relation == SemanticRelation::Tests)
    );
    assert_eq!(edges[0].object.name, "BarComponent");
    assert_eq!(edges[1].object.name, "FooComponent");
}

#[test]
fn path_context_does_not_change_source_only_output() {
    let source = r#"
import { signal } from '@angular/core';
const count = signal(0);
"#;
    let captures = Vec::new();
    let without_path = crate::layers::LayerRegistry::global().run_meta_layers_pipeline(
        source,
        &captures,
        Fidelity::Medium,
        None,
    );
    let with_path = crate::layers::LayerRegistry::global().run_meta_layers_pipeline_with_path(
        source,
        Path::new("src/count.ts"),
        &captures,
        Fidelity::Medium,
        None,
    );
    let old_rendered: Vec<_> = without_path.iter().map(|output| &output.rendered).collect();
    let new_rendered: Vec<_> = with_path.iter().map(|output| &output.rendered).collect();
    assert_eq!(old_rendered, new_rendered);
}

#[test]
fn canonical_path_reaches_registry_semantic_extraction() {
    let edges = crate::layers::LayerRegistry::global().collect_semantic_edges_with_path(
        BASIC_SPEC,
        Path::new("src/account.component.spec.ts"),
        &[],
        Fidelity::Medium,
        None,
    );
    let tests: Vec<_> = edges
        .iter()
        .filter(|edge| edge.relation == SemanticRelation::Tests)
        .collect();
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].object.name, "AccountComponent");
}
