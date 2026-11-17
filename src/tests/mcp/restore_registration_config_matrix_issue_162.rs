//! Actual configuration remains separate from optional compatibility identity.
use super::angular_bootstrap_inline_controls_tests::{CALL, call, capture, setup};
use crate::compression::Fidelity;
use crate::mcp::compatibility::validator::{
    CompatibilityFailure, validate_current_context, validate_historical_context, validate_semantic,
};
use serde_json::json;

#[test]
fn issue_162_restore_registration_config_preservation_matrix() {
    let _serial = crate::protocol::handler_response_serial();
    for (native, present_identity, enabled) in [
        (true, false, true),
        (true, true, true),
        (true, false, false),
        (false, false, true),
    ] {
        let (root, mut config) = setup(CALL);
        if present_identity {
            let path = root.path().join("main.ts");
            let source = std::fs::read_to_string(&path).unwrap();
            std::fs::write(
                path,
                format!(
                    "{source}\nimport {{Component}} from '@angular/core'; \
                    @Component({{selector:'x-probe',template:''}}) export class Probe {{}}"
                ),
            )
            .unwrap();
        } else if !native {
            std::fs::write(root.path().join("main.ts"), "export const plain = 1;").unwrap();
        }
        config
            .meta_layers
            .entry("angular".into())
            .or_default()
            .routing
            .enabled = enabled;
        let project = crate::tests::angular_authority_fixture::isolate_program(root.path());
        config.additional_roots = vec![project.to_string_lossy().into_owned()];
        let path = project.join("main.ts");
        let source = std::fs::read_to_string(&path).unwrap();
        let dependency = root.path().join("external-dependency.txt");
        std::fs::write(&dependency, "before").unwrap();
        let state = crate::mcp::McpState::new(config.clone());
        capture(&state, &project, "main.ts");
        call(&state, &project, "save_context", json!({"filePath":path}));
        let owner = state.durable_owner_path(&path.to_string_lossy());
        let loaded = {
            let guard = state.persistence_store_lock();
            guard
                .as_ref()
                .unwrap()
                .sqlite()
                .unwrap()
                .load_durable_context(&owner, None)
                .unwrap()
                .unwrap()
        };
        drop(state);
        let identities =
            crate::mcp::compatibility::derive_identities(&source, &path, &config).unwrap();
        assert_eq!(
            identities.semantic_config.angular.is_some(),
            present_identity
        );
        let authority = loaded.authority.typescript_authority.as_ref().unwrap();
        let native_scope = crate::layers::meta::angular_applicability::registrations_apply(
            &source,
            &path,
            loaded.fidelity,
            &config,
            authority,
        );
        assert_eq!(native_scope, native && enabled);
        assert_eq!(loaded.authority.resolver_registration_scope, None);
        assert_eq!(
            loaded.authority.resolver_registrations.is_some(),
            native_scope
        );

        let mut rebound = loaded.clone();
        rebound.authority.resolver_registration_scope = Some(native_scope);
        let low_level = validate_semantic(&rebound, &identities);
        if native_scope && !present_identity {
            assert!(matches!(
                low_level,
                Err(CompatibilityFailure::SemanticDependencyStale)
            ));
        } else {
            low_level.expect("existing identity-only boundary");
        }
        for historical in [false, true] {
            let validate = |snapshot| {
                if historical {
                    validate_historical_context(snapshot, &source, &path, &config, Fidelity::High)
                } else {
                    validate_current_context(snapshot, &source, &path, &config, Fidelity::High)
                }
            };
            let valid = validate(loaded.clone()).expect("configured unchanged restore");
            assert_eq!(
                valid.semantic.authority.resolver_registration_scope,
                Some(native_scope),
                "both entry points must rebind decoded scope"
            );
            if native_scope {
                let mut missing = loaded.clone();
                missing.authority.resolver_registrations = None;
                assert!(matches!(
                    validate(missing),
                    Err(CompatibilityFailure::SemanticDependencyStale)
                ));
                let mut forged = loaded.clone();
                forged
                    .authority
                    .resolver_registrations
                    .as_mut()
                    .unwrap()
                    .format_version = 2;
                assert!(matches!(
                    validate(forged),
                    Err(CompatibilityFailure::SemanticDependencyStale)
                ));
            } else {
                let mut unexpected = loaded.clone();
                unexpected.authority.resolver_registrations = Some(Default::default());
                assert!(matches!(
                    validate(unexpected),
                    Err(CompatibilityFailure::SemanticDependencyStale)
                ));
            }
            let mut incompatible = loaded.clone();
            incompatible.compatibility.semantic_producers = None;
            assert!(matches!(
                validate(incompatible),
                Err(CompatibilityFailure::MissingLegacyIdentity { .. })
            ));
            let mut insufficient = loaded.clone();
            insufficient.fidelity = Fidelity::Low;
            assert!(matches!(
                validate(insufficient),
                Err(CompatibilityFailure::InsufficientFidelity { .. })
            ));
            let mut stale = loaded.clone();
            stale
                .semantic_dependencies
                .push(crate::layers::meta::semantic::SemanticDependency {
                    file_path: dependency.to_string_lossy().into_owned(),
                    source_hash: Some(crate::cache::content_hash(b"wrong source")),
                });
            assert!(stale.authority.is_current(&stale.source_hash));
            assert!(matches!(
                validate(stale),
                Err(CompatibilityFailure::SemanticDependencyStale)
            ));
        }
        let changed = format!("{source}\n// source changed");
        std::fs::write(&path, &changed).unwrap();
        assert!(matches!(
            validate_current_context(loaded.clone(), &changed, &path, &config, Fidelity::High),
            Err(CompatibilityFailure::SourceMismatch { .. })
        ));
        // Historical replay preserves H1; it must not claim H2 currentness.
        let historical =
            validate_historical_context(loaded.clone(), &changed, &path, &config, Fidelity::High)
                .expect("compatible historical authority retains its original identity");
        assert_eq!(historical.semantic.source_hash, loaded.source_hash);
        assert_ne!(
            historical.semantic.source_hash,
            crate::cache::content_hash(changed.as_bytes())
        );
        let mut inconsistent = loaded;
        inconsistent.source_hash = "not-the-retained-authority-source".into();
        assert!(matches!(
            validate_historical_context(inconsistent, &changed, &path, &config, Fidelity::High),
            Err(CompatibilityFailure::SemanticDependencyStale)
        ));
    }
}
