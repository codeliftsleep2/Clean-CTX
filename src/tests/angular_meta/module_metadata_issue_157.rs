use super::project;

#[test]
fn issue_157_ambiguous_and_type_only_bindings_do_not_gain_authority() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("leaf.ts"), "import { Directive } from '@angular/core'; @Directive({selector:'[x]'}) export class Shared {}").unwrap();
    let path = root.path().join("app.module.ts");
    for binding in [
        "import type { Shared } from './leaf';",
        "import { Shared } from './leaf'; import { Shared } from './other';",
    ] {
        let source = format!(
            "import {{ NgModule }} from '@angular/core'; {binding} @NgModule({{declarations:[Shared]}}) export class AppModule {{}}"
        );
        let raw = &source[source.find("@NgModule").unwrap()..];
        let projection = project("AppModule", raw, &source, Some(&path), &[]);
        assert!(projection.edges.is_empty());
        assert!(projection.dependencies.is_empty());
    }
}

#[test]
fn issue_157_nested_imported_spread_stays_unresolved_without_reading_its_target() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("app.module.ts");
    std::fs::write(
        root.path().join("shared.ts"),
        "import { NESTED } from './nested'; export const SHARED = [...NESTED];",
    )
    .unwrap();
    std::fs::write(root.path().join("nested.ts"), "export const NESTED = [];").unwrap();
    let source = "import { NgModule } from '@angular/core'; import { SHARED } from './shared'; @NgModule({declarations:[...SHARED]}) export class AppModule {}";
    let raw = &source[source.find("@NgModule").unwrap()..];
    let projection = project("AppModule", raw, source, Some(&path), &[]);
    assert!(projection.edges.is_empty());
    assert_eq!(projection.unresolved_spreads, vec!["NESTED"]);
    assert!(
        projection
            .dependencies
            .iter()
            .any(|dependency| dependency.file_path.ends_with("shared.ts"))
    );
    assert!(
        !projection
            .dependencies
            .iter()
            .any(|dependency| dependency.file_path.ends_with("nested.ts"))
    );
}
