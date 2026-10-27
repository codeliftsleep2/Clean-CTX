//! Demand-scoped invocation acquisition must not relax ordinary native lookups.
use crate::ir::invocation_evidence::SourceSpan;
use crate::ir::typescript_resolution::TypeScriptAuthority;

#[test]
fn issue_162_bootstrap_scope_controls_demanded_imports_and_ordinary_policy() {
    let _serial = crate::protocol::handler_response_serial();
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    let package = p.join("node_modules/@angular/platform-browser");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@angular/platform-browser","version":"20.0.0","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "export declare function bootstrapApplication(root:unknown,config?:unknown):unknown;",
    )
    .unwrap();
    std::fs::write(p.join("tsconfig.json"),r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#).unwrap();
    std::fs::write(p.join("targets.ts"),"export class Root {} export const appConfig={providers:[]}; export function consume(root:unknown){}").unwrap();
    let source = "import {bootstrapApplication as launch} from '@angular/platform-browser'; import {Root,appConfig,consume} from './targets'; if(true){launch(Root,appConfig);consume(Root);} function run(){launch(Root,appConfig);consume(Root);} function parameter(launch:unknown){launch(Root,appConfig);} function local(){const launch=(a:unknown,b:unknown)=>{};launch(Root,appConfig);} function declaration(){function launch(){} launch(Root,appConfig);} function classShadow(){class launch{} launch(Root,appConfig);} function enumShadow(){enum launch{A} launch(Root,appConfig);}";
    let file = p.join("main.ts");
    std::fs::write(&file, source).unwrap();
    let roots = [p.to_path_buf()];
    let ordinary = TypeScriptAuthority::capture_with_declaration_fields(
        &file,
        source,
        &roots,
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
    );
    assert!(ordinary.capture_current && ordinary.disposition.is_none());
    assert!(ordinary.invocation_facets.is_empty());
    let demanded = TypeScriptAuthority::capture_with_demands(
        &file,
        source,
        &roots,
        None,
        crate::angular_meta::module_metadata::DECLARATION_FIELDS,
        crate::angular_meta::bootstrap_references::DEMANDED_APIS,
    );
    assert!(demanded.capture_current && demanded.disposition.is_none());
    assert_eq!(
        demanded.invocation_facets.len(),
        2,
        "only unshadowed native branch/function calls"
    );
    for call in &demanded.invocation_facets {
        let crate::ir::typescript_resolution::declaration_facets::ExpressionValue::Call(
            callee,
            args,
        ) = &call.expression.value
        else {
            panic!()
        };
        let span = SourceSpan::new(callee.start, callee.end).unwrap();
        assert!(
            ordinary.value_reference_at(span).is_none(),
            "ordinary consumer must keep its prior scope policy"
        );
        let api = demanded.value_reference_at(span).unwrap();
        assert_eq!(api.name, "bootstrapApplication");
        for (argument, name) in args.iter().zip(["Root", "appConfig"]) {
            let target = demanded
                .value_reference_at(SourceSpan::new(argument.start, argument.end).unwrap())
                .unwrap();
            assert_eq!(target.name, name);
            assert_eq!(target.source, p.join("targets.ts"));
        }
    }
    for (start, _) in source.match_indices("consume(Root)") {
        let callee = SourceSpan::new(start, start + "consume".len()).unwrap();
        assert!(
            demanded.value_reference_at(callee).is_none(),
            "unrequested callable must not gain scope authority"
        );
        let argument =
            SourceSpan::new(start + "consume(".len(), start + "consume(Root".len()).unwrap();
        assert!(
            demanded.value_reference_at(argument).is_none(),
            "unrequested argument must not gain scope authority"
        );
    }
}
