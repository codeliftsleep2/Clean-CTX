use super::{Namespace, Resolution, Resolver};
use std::path::Path;
fn write(root: &Path, name: &str, text: &str) {
    let file = root.join(name);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(file, text).unwrap();
}
fn setup(root: &Path, options: &str, files: &str, target: &str) {
    write(
        root,
        "tsconfig.json",
        &format!(
            r#"{{"compilerOptions":{{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],{options}}},"files":{files}}}"#
        ),
    );
    write(
        root,
        "package.json",
        &format!(
            r##"{{"name":"local-pkg","imports":{{"#leaf":"{target}"}},"exports":{{".":"{target}"}}}}"##
        ),
    );
}
fn lookup(root: &Path, specifier: &str) -> Resolution {
    write(
        root,
        "src/app.ts",
        &format!("import {{ Leaf }} from '{specifier}';"),
    );
    let mut resolver = Resolver::new(&[root.to_path_buf()]).unwrap();
    resolver.local(&root.join("src/app.ts"), "Leaf", Namespace::Value)
}
#[test]
fn issue_162_local_input_option_allow_js_self_reference_prefers_javascript_input() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#""rootDir":"src","outDir":"dist","allowJs":true,"declaration":true"#,
        r#"["src/app.ts","src/leaf.js"]"#,
        "./dist/leaf.d.ts",
    );
    write(p, "src/leaf.js", "export class Leaf {}");
    write(p, "dist/leaf.d.ts", "export interface Leaf {}");
    let result = lookup(p, "local-pkg");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.js")),
        "{result:?}"
    );
}
#[test]
fn issue_162_local_input_option_module_suffixes_select_current_input_variant() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#""rootDir":"src","outDir":"dist","moduleSuffixes":[".native",""]"#,
        r#"["src/app.ts","src/leaf.ts","src/leaf.native.ts"]"#,
        "./dist/leaf.js",
    );
    write(p, "src/leaf.ts", "export interface Leaf {}");
    write(p, "src/leaf.native.ts", "export class Leaf {}");
    let result = lookup(p, "#leaf");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.native.ts")),
        "{result:?}"
    );
}
#[test]
fn issue_162_local_input_option_inherited_root_and_output_keep_declaring_directory() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#""rootDir":"src","outDir":"dist""#,
        r#"["src/app.ts","src/leaf.ts"]"#,
        "./dist/leaf.js",
    );
    write(
        p,
        "config/base.json",
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[],"rootDir":"../src","outDir":"../dist"}}"#,
    );
    write(
        p,
        "tsconfig.json",
        r#"{"extends":"./config/base.json","files":["src/app.ts","src/leaf.ts"]}"#,
    );
    write(p, "src/leaf.ts", "export class Leaf {}");
    let result = lookup(p, "#leaf");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.ts")),
        "{result:?}"
    );
}
#[test]
fn issue_162_local_input_option_composite_default_root_is_config_directory() {
    let root = tempfile::tempdir().unwrap();
    let p = root.path();
    setup(
        p,
        r#""composite":true,"outDir":"dist""#,
        r#"["src/app.ts","src/leaf.ts"]"#,
        "./dist/src/leaf.js",
    );
    write(p, "src/leaf.ts", "export class Leaf {}");
    let result = lookup(p, "#leaf");
    assert!(
        matches!(result, Resolution::Resolved(ref id) if id.source.ends_with("src/leaf.ts")),
        "{result:?}"
    );
}
