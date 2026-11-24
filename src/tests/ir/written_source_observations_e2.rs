use super::super::e1_metrics as metrics;
use super::*;

fn fixture(source: &str) -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("app.ts"), source).unwrap();
    std::fs::write(
        root.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"types":["node"],"noLib":true},"files":["app.ts"]}"#,
    )
    .unwrap();
    let package = root.path().join("node_modules/@types/node");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@types/node","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(package.join("index.d.ts"), "declare class Broken {").unwrap();
    root
}
fn capture(root: &Path) -> Candidate {
    Candidate::capture(&[root.to_path_buf()], &root.join("app.ts")).unwrap()
}
const SOURCE: &str = "import type {Routes} from '@angular/router'; export class UserResolver {} export const routes:Routes=[{path:'users',resolve:{user:UserResolver}}];";

#[test]
fn e2_source_observations_do_not_acquire_dependency_programs() {
    let root = fixture(SOURCE);
    metrics::reset();
    let candidate = capture(root.path());
    assert_eq!(
        metrics::snapshot().acquisition_results,
        0,
        "source observations expanded the program"
    );
    assert_eq!(metrics::snapshot().dependency_parses, 0);
    assert_eq!(metrics::snapshot().automatic_type_calls, 0);
    assert_eq!(metrics::snapshot().library_calls, 0);
    let admitted = candidate.admit().unwrap();
    assert_eq!(admitted.observations().len(), 1);
    let o = &admitted.observations()[0];
    assert_eq!(
        (&o.route_path, &o.key),
        (&"users".to_string(), &"user".to_string())
    );
    assert_eq!(&SOURCE[o.key_span.start..o.key_span.end], "user");
    assert!(
        matches!(&o.expression,WrittenExpression::ReferenceText{text,span} if text=="UserResolver" && &SOURCE[span.start..span.end]==text)
    );
}
#[test]
fn e2_source_and_configuration_mutation_have_claim_scoped_effects() {
    let root = fixture(SOURCE);
    let candidate = capture(root.path());
    std::fs::write(root.path().join("tsconfig.json"), "not valid config").unwrap();
    let admitted = candidate.admit().unwrap();
    assert!(admitted.is_current());
    std::fs::write(root.path().join("app.ts"), SOURCE.replace("users", "other")).unwrap();
    assert!(!admitted.is_current());
    let candidate = capture(root.path());
    std::fs::write(root.path().join("app.ts"), SOURCE).unwrap();
    assert!(matches!(candidate.admit(), Err(Refusal::StaleSource)));
}
#[test]
fn e2_duplicate_fields_keys_and_spreads_preserve_only_defensible_observations() {
    for body in [
        "path:'a',path:'b',resolve:{user:U}",
        "path:'a',resolve:{user:U},resolve:{v:V}",
        "path:'a',resolve:{...other,user:U}",
        "...other,path:'a',resolve:{user:U}",
        "path:'a',resolve:{[computed]:U,user:U}",
    ] {
        let root = fixture(&format!("const x=[{{{body}}}];"));
        assert!(
            capture(root.path())
                .admit()
                .unwrap()
                .observations()
                .is_empty(),
            "{body}"
        );
    }
    let root = fixture("const x=[{path:'a',resolve:{user:U,'user':call(),ok:Good}}];");
    let admitted = capture(root.path()).admit().unwrap();
    assert_eq!(admitted.observations().len(), 1);
    assert_eq!(admitted.observations()[0].key, "ok");
    assert!(
        admitted
            .omissions()
            .iter()
            .any(|o| o.reason == OmissionReason::DuplicateMappingKey)
    );
}
#[test]
fn e2_unsupported_expressions_and_lexical_shadowing_remain_written_only() {
    let source = "const UserResolver=7;function f(UserResolver:unknown){return {path:'a',resolve:{user:UserResolver,bad:factory()}};}";
    let root = fixture(source);
    let admitted = capture(root.path()).admit().unwrap();
    assert_eq!(admitted.observations().len(), 2);
    assert!(admitted.observations().iter().any(|o|matches!(&o.expression,WrittenExpression::Unsupported{syntax_kind,..} if syntax_kind=="call_expression")));
    assert!(admitted.observations().iter().any(|o|matches!(&o.expression,WrittenExpression::ReferenceText{text,..} if text=="UserResolver")));
    metrics::reset();
    let before = metrics::snapshot();
    for o in admitted.observations() {
        let _classification = matches!(o.expression, WrittenExpression::ReferenceText { .. });
    }
    assert_eq!(before, metrics::snapshot());
}
#[test]
fn e2_malformed_and_unauthorized_sources_refuse() {
    let root = fixture("const x=[{path:'a',resolve:{user:U}");
    assert!(matches!(
        Candidate::capture(&[root.path().to_path_buf()], &root.path().join("app.ts")),
        Err(Refusal::MalformedSource)
    ));
    let outside = fixture(SOURCE);
    assert!(matches!(
        Candidate::capture(&[root.path().to_path_buf()], &outside.path().join("app.ts")),
        Err(Refusal::UnreadableOrUnauthorized)
    ));
}
