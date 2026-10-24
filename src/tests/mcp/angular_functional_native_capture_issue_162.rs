use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::{Resolution, TypeScriptAuthority};

#[test]
fn issue_162_functional_native_capture_callable_imports_preserve_shadow_refusal() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install(p);
    std::fs::write(
        p.join("node_modules/@angular/core/index.d.ts"),
        "export declare function inject(token:unknown):unknown;",
    )
    .unwrap();
    let source = r#"
import {inject} from '@angular/core';
import * as api from '@angular/core';
class AuditApi {}
const ordinary=inject(AuditApi);
const guard=()=>{return inject(AuditApi);};
const namespaced=()=>{return api.inject(AuditApi);};
const shadow=(api:{inject:(token:unknown)=>unknown})=>api.inject(AuditApi);
const local=(inject:(token:unknown)=>unknown)=>inject(AuditApi);
"#;
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let authority = TypeScriptAuthority::capture(&file, source, &[p.to_path_buf()]);
    let at = |text: &str, needle: &str| {
        let start = source.find(text).unwrap() + text.rfind(needle).unwrap();
        SourceSpan::new(start, start + needle.len()).unwrap()
    };
    let baseline = authority
        .value_reference_at(at("const ordinary=inject", "inject"))
        .expect("nonempty top-level exact import control");
    assert_eq!(baseline.name, "inject");
    let callable = authority
        .value_reference_at(at("const guard=()=>{return inject", "inject"))
        .expect("callable import must retain exact native value evidence");
    assert_eq!(callable, baseline);
    let namespace = authority
        .value_reference_at(at("const namespaced=()=>{return api.inject", "api.inject"))
        .expect("callable namespace qualification must resolve the same API");
    assert_eq!(namespace, baseline);
    for span in [
        at(
            "const shadow=(api:{inject:(token:unknown)=>unknown})=>api.inject",
            "api.inject",
        ),
        at(
            "const local=(inject:(token:unknown)=>unknown)=>inject",
            "inject",
        ),
    ] {
        assert_ne!(
            authority.value_reference_at(span),
            Some(baseline),
            "local bindings cannot borrow imported API identity"
        );
    }
    assert!(
        authority.reference_sites.iter().any(
            |(_, _, result)| matches!(result, Resolution::Resolved(id) if id.name == "AuditApi")
        )
    );
}
