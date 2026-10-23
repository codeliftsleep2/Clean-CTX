use crate::layers::meta::canonical_endpoint::{CanonicalEndpoint, string_label};
use crate::layers::meta::semantic::SemanticRelation;

#[test]
fn issue_162_string_token_carrier_provenance_forgery_dedup_and_durable_encoding() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    crate::tests::angular_authority_fixture::install_inject_apis(p);
    let core = p.join("node_modules/@angular/core/index.d.ts");
    let mut api = std::fs::read_to_string(&core).unwrap();
    api.push_str("export declare function Inject(token:any):any;");
    std::fs::write(core, api).unwrap();
    let mut config = crate::tests::test_config();
    config
        .additional_roots
        .push(p.to_string_lossy().into_owned());
    let state = crate::mcp::McpState::new(config);
    let source = r#"
import {Injectable as Service, Inject as Pick} from '@angular/core';
const unrelated = "@Pick('phantom')";
@Service() export class Owner {
 constructor(@Pick('audit.api') a:unknown, @Pick("audit\u002eapi") b:unknown,
 @Pick('\ud800') lone:unknown) {}
 method(Pick:unknown) { return Pick; }
}
"#;
    let file = p.join("app.ts");
    std::fs::write(&file, source).unwrap();
    let artifact = crate::mcp::tool_helpers::compile_source_ir_candidate(
        file.to_str().unwrap(),
        source,
        crate::compression::Fidelity::High,
        &state,
        p.to_str(),
    )
    .unwrap();
    let authority = artifact.typescript_authority();
    let (edges, _, _) = artifact.published_semantics();
    let injected: Vec<_> = edges
        .into_iter()
        .filter(|edge| edge.relation == SemanticRelation::Injects)
        .collect();
    assert!(!injected.is_empty());
    assert!(authority.admits_relation_evidence(&injected));
    let edge = injected
        .iter()
        .find(|edge| edge.object.name == "\"audit.api\"")
        .unwrap();
    let mut equivalent = edge.clone();
    let CanonicalEndpoint::StringValue { proof, .. } =
        equivalent.object.canonical.as_mut().unwrap()
    else {
        panic!("string endpoint");
    };
    let second = authority.constructor_parameters.iter()
        .flat_map(|parameter| &parameter.decorator_facets)
        .flat_map(|decorator| &decorator.arguments)
        .find(|argument| argument.start != proof.start
            && matches!(&argument.value,
                crate::ir::typescript_resolution::declaration_facets::ExpressionValue::StringValue(units)
                    if string_label(units) == "\"audit.api\""))
        .expect("a second equivalent value at a different occurrence");
    proof.start = second.start;
    proof.end = second.end;
    assert!(authority.admits_relation_evidence(std::slice::from_ref(&equivalent)));
    assert_eq!(
        edge.object.canonical.as_ref().unwrap().key(),
        equivalent.object.canonical.as_ref().unwrap().key()
    );
    assert_ne!(
        edge.object.canonical, equivalent.object.canonical,
        "value identity must not erase distinct retained occurrence proof"
    );

    for mutation in 0..4 {
        let mut forged = edge.clone();
        let CanonicalEndpoint::StringValue {
            string_value,
            proof,
        } = forged.object.canonical.as_mut().unwrap()
        else {
            panic!("string endpoint");
        };
        match mutation {
            0 => proof.source_hash = "unconsumed".into(),
            1 => proof.start += 1,
            2 => string_value.push(0xd800),
            _ => proof.source = p.join("unrelated.ts"),
        }
        forged.object.name = string_label(string_value);
        assert!(
            !authority.admits_relation_evidence(&[forged]),
            "forgery {mutation}"
        );
    }
    let mut index = crate::workspace::index::WorkspaceIndex::new();
    index.add_edges(file.to_str().unwrap(), vec![edge.clone(), equivalent]);
    assert_eq!(
        index
            .forward_edges_by_identity("angular", "Service", "Owner")
            .len(),
        1
    );
    index.remove_file(file.to_str().unwrap());
    assert!(
        index
            .forward_edges_by_identity("angular", "Service", "Owner")
            .is_empty()
    );

    use crate::mcp::state::durable_semantics::DurableSemanticSnapshot;
    let durable = DurableSemanticSnapshot::new(
        file.to_string_lossy().into_owned(),
        authority.source_hash.clone(),
        1,
        &injected,
    );
    let encoded = serde_json::to_value(&durable).unwrap();
    let restored: DurableSemanticSnapshot = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), encoded);
    let restored = restored.restore_edges().unwrap();
    assert!(authority.admits_relation_evidence(&restored));
    assert_eq!(
        restored
            .iter()
            .map(|edge| &edge.object.canonical)
            .collect::<Vec<_>>(),
        injected
            .iter()
            .map(|edge| &edge.object.canonical)
            .collect::<Vec<_>>()
    );
    assert!(
        restored
            .iter()
            .any(|edge| edge.object.name == "\"\\ud800\"")
    );
    let public = serde_json::to_value(&injected).unwrap();
    assert!(
        public
            .as_array()
            .unwrap()
            .iter()
            .all(|edge| edge["object"].get("canonical").is_none()
                && edge["subject"].get("canonical").is_none())
    );
}

#[test]
fn issue_162_string_token_carrier_old_declaration_encoding_and_disjoint_key() {
    let declaration = serde_json::json!({"declaration": {
        "source":"owner.ts", "source_hash":"hash", "start":0, "end":10,
        "name":"audit.api", "kind":"class_declaration"}, "site":null});
    let endpoint: CanonicalEndpoint = serde_json::from_value(declaration.clone()).unwrap();
    assert_eq!(serde_json::to_value(&endpoint).unwrap(), declaration);
    let value = CanonicalEndpoint::StringValue {
        string_value: "audit.api".encode_utf16().collect(),
        proof: crate::layers::meta::canonical_endpoint::LiteralProof {
            source: "owner.ts".into(),
            source_hash: "hash".into(),
            start: 0,
            end: 10,
        },
    };
    assert_ne!(value.key(), endpoint.key());
    assert_eq!(
        string_label(&[0x22, 0x5c, 0, 0xd800]),
        "\"\\\"\\\\\\u0000\\ud800\""
    );
}
