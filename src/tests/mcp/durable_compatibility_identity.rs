use super::derive_with_catalog;
use super::identity::ProducerKey;
use super::identity::{
    CanonicalProducerIdentity, CompatibilityIdentities, SemanticProducerIdentity,
};
use super::producer::GenerationCatalog;
use crate::config::{CleanCtxConfig, MetaLayerConfig};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const ANGULAR_SOURCE: &str = r#"
import { Component } from '@angular/core';
@Component({ selector: 'app-root', template: '<main></main>' })
export class AppComponent {}
"#;

#[test]
fn identity_serialization_is_deterministic_across_config_insertion_order() {
    let mut first = CleanCtxConfig::default();
    first
        .type_aliases
        .insert("$user".to_string(), "UserRecord".to_string());
    first
        .type_aliases
        .insert("$map".to_string(), "HashMap".to_string());

    let mut second = CleanCtxConfig::default();
    second
        .type_aliases
        .insert("$map".to_string(), "HashMap".to_string());
    second
        .type_aliases
        .insert("$user".to_string(), "UserRecord".to_string());

    let first_identity = derive_with_catalog(
        "export class Plain {}",
        Path::new("plain.ts"),
        &first,
        &GenerationCatalog::current(),
    )
    .expect("first identity");
    let second_identity = derive_with_catalog(
        "export class Plain {}",
        Path::new("plain.ts"),
        &second,
        &GenerationCatalog::current(),
    )
    .expect("second identity");

    assert_eq!(first_identity, second_identity);
    assert_eq!(
        serde_json::to_vec(&first_identity.canonical_config).unwrap(),
        serde_json::to_vec(&second_identity.canonical_config).unwrap()
    );
    assert_eq!(
        serde_json::to_vec(&first_identity.canonical_producers).unwrap(),
        serde_json::to_vec(&second_identity.canonical_producers).unwrap()
    );
}

#[test]
fn unrelated_typescript_generation_does_not_change_csharp_identity() {
    let config = CleanCtxConfig::default();
    let baseline = derive_with_catalog(
        "public class Plain {}",
        Path::new("Plain.cs"),
        &config,
        &GenerationCatalog::current(),
    )
    .expect("baseline");
    let changed = derive_with_catalog(
        "public class Plain {}",
        Path::new("Plain.cs"),
        &config,
        &GenerationCatalog::current().with_generation(ProducerKey::TypeScriptCanonical, 99),
    )
    .expect("changed catalog");

    assert_eq!(baseline.canonical_producers, changed.canonical_producers);
    assert_eq!(baseline.semantic_producers, changed.semantic_producers);
}

#[test]
fn marker_only_config_changes_canonical_but_not_semantic_identity() {
    let first = angular_config();
    let mut second = first.clone();
    second
        .meta_layers
        .get_mut("angular")
        .expect("angular config")
        .rxjs
        .min_pipe_operators = 7;

    let first_identity = derive_angular(&first);
    let second_identity = derive_angular(&second);

    assert_ne!(
        first_identity.canonical_config,
        second_identity.canonical_config
    );
    assert_eq!(
        first_identity.semantic_config,
        second_identity.semantic_config
    );
}

#[test]
fn semantic_producing_config_changes_semantic_identity() {
    let first = angular_config();
    let mut second = first.clone();
    second
        .meta_layers
        .get_mut("angular")
        .expect("angular config")
        .ngrx
        .enabled = false;

    let first_identity = derive_angular(&first);
    let second_identity = derive_angular(&second);

    assert_ne!(
        first_identity.semantic_config,
        second_identity.semantic_config
    );
}

#[test]
fn relevant_producer_set_addition_and_removal_compare_bidirectionally() {
    let config = angular_config();
    let present = derive_with_catalog(
        ANGULAR_SOURCE,
        Path::new("app.component.ts"),
        &config,
        &GenerationCatalog::current(),
    )
    .expect("present producer");
    let absent = derive_with_catalog(
        ANGULAR_SOURCE,
        Path::new("app.component.ts"),
        &config,
        &GenerationCatalog::current()
            .without(ProducerKey::AngularMarkers)
            .without(ProducerKey::AngularSemantic),
    )
    .expect("absent producer");

    assert!(
        present
            .canonical_producers
            .differs_bidirectionally(&absent.canonical_producers)
    );
    assert!(
        absent
            .canonical_producers
            .differs_bidirectionally(&present.canonical_producers)
    );
    assert!(
        present
            .semantic_producers
            .differs_bidirectionally(&absent.semantic_producers)
    );
    assert!(
        absent
            .semantic_producers
            .differs_bidirectionally(&present.semantic_producers)
    );

    assert_producer_maps_are_order_independent(
        &present.canonical_producers,
        &present.semantic_producers,
    );
}

fn angular_config() -> CleanCtxConfig {
    let mut config = CleanCtxConfig::default();
    config
        .meta_layers
        .insert("angular".to_string(), MetaLayerConfig::default());
    config
}

fn derive_angular(config: &CleanCtxConfig) -> CompatibilityIdentities {
    derive_with_catalog(
        ANGULAR_SOURCE,
        Path::new("app.component.ts"),
        config,
        &GenerationCatalog::current(),
    )
    .expect("Angular identity")
}

fn assert_producer_maps_are_order_independent(
    canonical: &CanonicalProducerIdentity,
    semantic: &SemanticProducerIdentity,
) {
    let canonical_hash: HashMap<_, _> = canonical
        .relevant_producers
        .iter()
        .map(|(key, generation)| (*key, *generation))
        .collect();
    let rebuilt_canonical = CanonicalProducerIdentity {
        schema_version: canonical.schema_version,
        relevant_producers: canonical_hash.into_iter().collect::<BTreeMap<_, _>>(),
    };
    assert_eq!(
        serde_json::to_vec(canonical).unwrap(),
        serde_json::to_vec(&rebuilt_canonical).unwrap()
    );

    let semantic_hash: HashMap<_, _> = semantic
        .relevant_producers
        .iter()
        .map(|(key, generation)| (*key, *generation))
        .collect();
    let rebuilt_semantic = SemanticProducerIdentity {
        schema_version: semantic.schema_version,
        relevant_producers: semantic_hash.into_iter().collect::<BTreeMap<_, _>>(),
    };
    assert_eq!(
        serde_json::to_vec(semantic).unwrap(),
        serde_json::to_vec(&rebuilt_semantic).unwrap()
    );
}
