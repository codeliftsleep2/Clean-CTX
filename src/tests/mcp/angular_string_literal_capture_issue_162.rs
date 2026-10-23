use crate::ir::typescript_resolution::declaration_facets::ExpressionValue;

#[test]
fn issue_162_literal_units_real_capture_preserves_owner_occurrence_and_roundtrip() {
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
const misleading = "@Pick('wrong')";
function unrelated(Pick:unknown) {return Pick;}
@Service() export class Owner {
  constructor(@Pick('audit.api') first:unknown,
    @Pick("audit\u002eapi") second:unknown,
    @Pick('\ud800') third:unknown) {}
  method(first:string) { return misleading + first; }
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
    assert!(authority.capture_current);
    assert_eq!(
        authority.source_hash,
        crate::cache::content_hash(source.as_bytes())
    );
    let parameters = &authority.constructor_parameters;
    assert_eq!(parameters.len(), 3);
    let expected = [
        "audit.api".encode_utf16().collect::<Vec<_>>(),
        "audit.api".encode_utf16().collect(),
        vec![0xd800],
    ];
    for (parameter, expected) in parameters.iter().zip(expected) {
        assert_eq!(parameter.owner.name, "Owner");
        assert_eq!(parameter.owner.source_hash, authority.source_hash);
        let [decorator] = parameter.decorator_facets.as_slice() else {
            panic!("one decorator");
        };
        let [argument] = decorator.arguments.as_slice() else {
            panic!("one exact argument");
        };
        let ExpressionValue::StringValue(units) = &argument.value else {
            panic!("exact string evidence: {argument:?}");
        };
        assert_eq!(units, &expected);
        assert!(source.get(argument.start..argument.end).is_some());
        assert!(argument.start > parameter.parameter.0 && argument.end < parameter.parameter.1);
    }
    let before = serde_json::to_value(authority).unwrap();
    let restored: crate::ir::typescript_resolution::TypeScriptAuthority =
        serde_json::from_value(before.clone()).unwrap();
    assert_eq!(
        serde_json::to_value(restored).unwrap(),
        before,
        "all retained facts and witnesses, including lone surrogates, must roundtrip"
    );
}
