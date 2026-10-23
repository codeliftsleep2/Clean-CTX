use std::path::Path;

/// Supply canonical package evidence for hydration fixtures that use Angular APIs.
/// Package inputs stay under node_modules, outside source discovery candidates.
pub(crate) fn install(root: &Path) {
    let package = root.join("node_modules/@angular/core");
    std::fs::create_dir_all(&package).expect("Angular fixture package");
    let files = [
        (
            package.join("package.json"),
            r#"{"name":"@angular/core","version":"20.0.0","types":"index.d.ts"}"#,
        ),
        (
            package.join("index.d.ts"),
            "export declare function Injectable(metadata?: unknown): any;\nexport declare function Component(metadata: unknown): any;\n",
        ),
        (
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"module":"esnext","moduleResolution":"bundler","moduleDetection":"legacy","noLib":true,"types":[]}}"#,
        ),
    ];
    for (path, contents) in files {
        if path.exists() {
            assert_eq!(
                std::fs::read_to_string(&path).expect("existing fixture input"),
                contents,
                "conflicting Angular fixture input: {}",
                path.display()
            );
        } else {
            std::fs::write(path, contents).expect("Angular fixture input");
        }
    }
}

/// Canonical package inputs for the existing DI preservation fixtures.
#[cfg(all(feature = "typescript", feature = "angular"))]
pub(crate) fn install_inject_apis(root: &Path) {
    install(root);
    std::fs::write(
        root.join("node_modules/@angular/core/index.d.ts"),
        "export declare function Injectable(metadata?: unknown): any;\nexport declare function Component(metadata: unknown): any;\nexport declare function Directive(metadata: unknown): any;\nexport declare function Pipe(metadata: unknown): any;\nexport declare function inject(token: unknown, options?: unknown): unknown;\n",
    )
    .expect("DI fixture package declarations");
}
